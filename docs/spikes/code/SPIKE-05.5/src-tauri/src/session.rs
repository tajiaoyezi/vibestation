use std::collections::VecDeque;
use std::io::{self, Write};
use std::os::fd::RawFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use portable_pty::{Child, MasterPty, PtySize};

use crate::ipc::{DrainResponse, SessionSummary};

pub const DEFAULT_QUEUE_CAPACITY: usize = 256;

#[derive(Debug, Clone, Copy)]
pub enum DropPolicy {
    DropOldest,
}

#[derive(Debug)]
pub struct ChunkEntry {
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct QueueState {
    pub entries: VecDeque<ChunkEntry>,
    pub queued_bytes: usize,
    pub capacity_chunks: usize,
}

impl QueueState {
    pub fn new(capacity_chunks: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            queued_bytes: 0,
            capacity_chunks,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SessionMetrics {
    pub total_read_bytes: u64,
    pub total_drained_bytes: u64,
    pub dropped_chunks: u64,
    pub dropped_bytes: u64,
    pub depth_samples: u64,
    pub depth_sum: u64,
    pub max_queue_depth: usize,
    pub read_calls: u64,
    pub total_read_syscall_ns: u128,
    pub total_enqueue_ns: u128,
}

pub struct SessionState {
    pub id: String,
    pub label: String,
    pub command: String,
    pub token: usize,
    pub fd: RawFd,
    pub created_at_ms: u128,
    pub reader_strategy: String,
    pub master: Mutex<Box<dyn MasterPty + Send>>,
    pub writer: Mutex<Box<dyn Write + Send>>,
    pub child: Mutex<Box<dyn Child + Send>>,
    pub queue: Mutex<QueueState>,
    pub metrics: Mutex<SessionMetrics>,
    pub exit_status: Mutex<Option<String>>,
    pub closed: AtomicBool,
}

impl SessionState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        label: String,
        command: String,
        token: usize,
        fd: RawFd,
        reader_strategy: String,
        master: Box<dyn MasterPty + Send>,
        writer: Box<dyn Write + Send>,
        child: Box<dyn Child + Send>,
        queue_capacity: usize,
    ) -> Self {
        Self {
            id,
            label,
            command,
            token,
            fd,
            created_at_ms: now_ms(),
            reader_strategy,
            master: Mutex::new(master),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
            queue: Mutex::new(QueueState::new(queue_capacity)),
            metrics: Mutex::new(SessionMetrics::default()),
            exit_status: Mutex::new(None),
            closed: AtomicBool::new(false),
        }
    }

    pub fn enqueue_bytes(&self, data: &[u8], policy: DropPolicy, read_syscall_ns: u128) {
        if data.is_empty() || self.closed.load(Ordering::Relaxed) {
            return;
        }

        let enqueue_started = Instant::now();
        let mut queue = lock(&self.queue);
        let mut metrics = lock(&self.metrics);
        metrics.read_calls += 1;
        metrics.total_read_bytes += data.len() as u64;
        metrics.total_read_syscall_ns += read_syscall_ns;

        if queue.entries.len() >= queue.capacity_chunks {
            match policy {
                DropPolicy::DropOldest => {
                    if let Some(oldest) = queue.entries.pop_front() {
                        queue.queued_bytes = queue.queued_bytes.saturating_sub(oldest.data.len());
                        metrics.dropped_chunks += 1;
                        metrics.dropped_bytes += oldest.data.len() as u64;
                    }
                }
            }
        }

        queue.queued_bytes += data.len();
        queue.entries.push_back(ChunkEntry { data: data.to_vec() });
        metrics.max_queue_depth = metrics.max_queue_depth.max(queue.entries.len());
        metrics.depth_sum += queue.entries.len() as u64;
        metrics.depth_samples += 1;
        metrics.total_enqueue_ns += enqueue_started.elapsed().as_nanos();
    }

    pub fn write_input(&self, input: &str) -> io::Result<()> {
        let mut writer = lock(&self.writer);
        writer.write_all(input.as_bytes())?;
        writer.flush()
    }

    pub fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let master = lock(&self.master);
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)
    }

    pub fn try_wait(&self) -> io::Result<Option<String>> {
        let mut child = lock(&self.child);
        if let Some(status) = child.try_wait()? {
            let text = format!("{:?}", status);
            *lock(&self.exit_status) = Some(text.clone());
            Ok(Some(text))
        } else {
            Ok(None)
        }
    }

    pub fn kill(&self) {
        self.closed.store(true, Ordering::Relaxed);
        let _ = lock(&self.child).kill();
        let _ = self.try_wait();
    }

    pub fn drain(&self, max_chunks: usize, max_bytes: usize) -> DrainResponse {
        let mut queue = lock(&self.queue);
        let mut metrics = lock(&self.metrics);
        let mut drained = Vec::new();
        let mut drained_bytes = 0usize;

        while drained.len() < max_chunks {
            let Some(front) = queue.entries.front() else {
                break;
            };
            if !drained.is_empty() && drained_bytes + front.data.len() > max_bytes {
                break;
            }
            let entry = queue.entries.pop_front().expect("front checked");
            queue.queued_bytes = queue.queued_bytes.saturating_sub(entry.data.len());
            drained_bytes += entry.data.len();
            drained.push(String::from_utf8_lossy(&entry.data).to_string());
        }

        metrics.total_drained_bytes += drained_bytes as u64;
        metrics.depth_sum += queue.entries.len() as u64;
        metrics.depth_samples += 1;

        let avg_queue_depth = if metrics.depth_samples == 0 {
            0.0
        } else {
            metrics.depth_sum as f64 / metrics.depth_samples as f64
        };
        let avg_read_bytes = if metrics.read_calls == 0 {
            0.0
        } else {
            metrics.total_read_bytes as f64 / metrics.read_calls as f64
        };
        let avg_read_syscall_us = if metrics.read_calls == 0 {
            0.0
        } else {
            metrics.total_read_syscall_ns as f64 / metrics.read_calls as f64 / 1_000.0
        };
        let avg_enqueue_us = if metrics.read_calls == 0 {
            0.0
        } else {
            metrics.total_enqueue_ns as f64 / metrics.read_calls as f64 / 1_000.0
        };

        DrainResponse {
            chunks: drained,
            queue_depth: queue.entries.len(),
            queued_bytes: queue.queued_bytes,
            avg_queue_depth,
            max_queue_depth: metrics.max_queue_depth,
            total_read_bytes: metrics.total_read_bytes,
            total_drained_bytes: metrics.total_drained_bytes,
            dropped_chunks: metrics.dropped_chunks,
            dropped_bytes: metrics.dropped_bytes,
            exit_status: lock(&self.exit_status).clone(),
            reader_strategy: self.reader_strategy.clone(),
            read_calls: metrics.read_calls,
            avg_read_bytes,
            avg_read_syscall_us,
            avg_enqueue_us,
        }
    }

    pub fn summary(&self) -> SessionSummary {
        let queue = lock(&self.queue);
        let metrics = lock(&self.metrics);
        let avg_queue_depth = if metrics.depth_samples == 0 {
            0.0
        } else {
            metrics.depth_sum as f64 / metrics.depth_samples as f64
        };
        let avg_read_bytes = if metrics.read_calls == 0 {
            0.0
        } else {
            metrics.total_read_bytes as f64 / metrics.read_calls as f64
        };
        let avg_read_syscall_us = if metrics.read_calls == 0 {
            0.0
        } else {
            metrics.total_read_syscall_ns as f64 / metrics.read_calls as f64 / 1_000.0
        };
        let avg_enqueue_us = if metrics.read_calls == 0 {
            0.0
        } else {
            metrics.total_enqueue_ns as f64 / metrics.read_calls as f64 / 1_000.0
        };

        SessionSummary {
            id: self.id.clone(),
            label: self.label.clone(),
            command: self.command.clone(),
            queue_depth: queue.len(),
            queued_bytes: queue.queued_bytes,
            avg_queue_depth,
            max_queue_depth: metrics.max_queue_depth,
            total_read_bytes: metrics.total_read_bytes,
            total_drained_bytes: metrics.total_drained_bytes,
            dropped_chunks: metrics.dropped_chunks,
            dropped_bytes: metrics.dropped_bytes,
            exit_status: lock(&self.exit_status).clone(),
            created_at_ms: self.created_at_ms,
            reader_strategy: self.reader_strategy.clone(),
            read_calls: metrics.read_calls,
            avg_read_bytes,
            avg_read_syscall_us,
            avg_enqueue_us,
        }
    }
}

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
