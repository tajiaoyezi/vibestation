use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use portable_pty::{native_pty_system, CommandBuilder};

use crate::ipc::{
    ArtifactReadRequest, ArtifactWriteRequest, DrainResponse, ProcessStats, ResizeSessionRequest,
    SessionSummary, SpawnSessionRequest, WriteSessionRequest,
};
use crate::session::{lock, DropPolicy, SessionState, DEFAULT_QUEUE_CAPACITY};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderStrategy {
    SharedPoll,
    PerSession,
}

impl ReaderStrategy {
    pub fn from_env() -> Self {
        match std::env::var("SPIKE055_STRATEGY")
            .or_else(|_| std::env::var("SPIKE05_STRATEGY"))
            .unwrap_or_else(|_| "shared".to_string())
            .as_str()
        {
            "per-session" | "per_session" | "per" => Self::PerSession,
            _ => Self::SharedPoll,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SharedPoll => "shared-reader",
            Self::PerSession => "per-session-reader",
        }
    }
}

enum ReaderCommand {
    Register { token: usize, fd: i32, session: Arc<SessionState> },
    Unregister { token: usize, fd: i32 },
    Shutdown,
}

pub struct PtyManager {
    strategy: ReaderStrategy,
    sessions: Mutex<HashMap<String, Arc<SessionState>>>,
    control_tx: Option<mpsc::Sender<ReaderCommand>>,
    next_session_id: AtomicUsize,
    next_token: AtomicUsize,
    shared_reader_alive: Arc<AtomicBool>,
    worker_threads: Mutex<HashMap<String, thread::JoinHandle<()>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        let strategy = ReaderStrategy::from_env();
        let shared_reader_alive = Arc::new(AtomicBool::new(matches!(strategy, ReaderStrategy::SharedPoll)));
        let control_tx = if matches!(strategy, ReaderStrategy::SharedPoll) {
            let (tx, rx) = mpsc::channel();
            let alive = Arc::clone(&shared_reader_alive);
            thread::Builder::new()
                .name("spike055-shared-reader".to_string())
                .spawn(move || reader_loop(rx, alive))
                .expect("shared reader thread spawn");
            Some(tx)
        } else {
            None
        };

        Self {
            strategy,
            sessions: Mutex::new(HashMap::new()),
            control_tx,
            next_session_id: AtomicUsize::new(1),
            next_token: AtomicUsize::new(1),
            shared_reader_alive,
            worker_threads: Mutex::new(HashMap::new()),
        }
    }

    pub fn strategy_name(&self) -> &'static str {
        self.strategy.as_str()
    }

    pub fn spawn_session(&self, request: SpawnSessionRequest) -> io::Result<SessionSummary> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(portable_pty::PtySize {
                rows: request.rows,
                cols: request.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let mut command = CommandBuilder::new(shell);
        command.arg("-lc");
        command.arg(request.command.clone());
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("LANG", "en_US.UTF-8");
        command.env("LC_ALL", "en_US.UTF-8");

        let child = pair.slave.spawn_command(command).map_err(io::Error::other)?;
        drop(pair.slave);

        let fd = pair
            .master
            .as_raw_fd()
            .ok_or_else(|| io::Error::other("portable-pty master has no raw fd on this platform"))?;
        set_fd_nonblocking(fd, matches!(self.strategy, ReaderStrategy::SharedPoll))?;
        let writer = pair.master.take_writer().map_err(io::Error::other)?;
        let session_id = format!("session-{:02}", self.next_session_id.fetch_add(1, Ordering::Relaxed));
        let token = self.next_token.fetch_add(1, Ordering::Relaxed);
        let queue_capacity = request.queue_capacity.unwrap_or(DEFAULT_QUEUE_CAPACITY);
        let session = Arc::new(SessionState::new(
            session_id.clone(),
            request.label,
            request.command,
            token,
            fd,
            self.strategy_name().to_string(),
            pair.master,
            writer,
            child,
            queue_capacity,
        ));

        match self.strategy {
            ReaderStrategy::SharedPoll => {
                self.control_tx
                    .as_ref()
                    .expect("shared strategy has control tx")
                    .send(ReaderCommand::Register {
                        token,
                        fd,
                        session: Arc::clone(&session),
                    })
                    .map_err(|error| io::Error::other(format!("reader thread unavailable: {error}")))?;
            }
            ReaderStrategy::PerSession => {
                let reader_session = Arc::clone(&session);
                let handle = thread::Builder::new()
                    .name(format!("spike055-reader-{session_id}"))
                    .spawn(move || per_session_reader_loop(reader_session, DropPolicy::DropOldest))
                    .map_err(io::Error::other)?;
                lock(&self.worker_threads).insert(session_id.clone(), handle);
            }
        }

        lock(&self.sessions).insert(session_id, Arc::clone(&session));
        Ok(session.summary())
    }

    pub fn write_session(&self, request: WriteSessionRequest) -> io::Result<()> {
        let session = self.session(&request.session_id)?;
        session.write_input(&request.data)
    }

    pub fn resize_session(&self, request: ResizeSessionRequest) -> io::Result<()> {
        let session = self.session(&request.session_id)?;
        session.resize(request.cols, request.rows)
    }

    pub fn drain_session(&self, session_id: &str, max_chunks: usize, max_bytes: usize) -> io::Result<DrainResponse> {
        let session = self.session(session_id)?;
        Ok(session.drain(max_chunks, max_bytes))
    }

    pub fn close_session(&self, session_id: &str) -> io::Result<()> {
        let session = lock(&self.sessions)
            .remove(session_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("unknown session: {session_id}")))?;

        if let Some(control_tx) = &self.control_tx {
            let _ = control_tx.send(ReaderCommand::Unregister {
                token: session.token,
                fd: session.fd,
            });
        }

        session.kill();

        if let Some(handle) = lock(&self.worker_threads).remove(session_id) {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn close_all_sessions(&self) {
        let ids: Vec<String> = lock(&self.sessions).keys().cloned().collect();
        for session_id in ids {
            let _ = self.close_session(&session_id);
        }
    }

    pub fn session_snapshot(&self, session_id: &str) -> io::Result<SessionSummary> {
        let session = self.session(session_id)?;
        Ok(session.summary())
    }

    pub fn manager_snapshot(&self) -> Vec<SessionSummary> {
        lock(&self.sessions)
            .values()
            .map(|session| session.summary())
            .collect()
    }

    pub fn sample_process_stats(&self) -> io::Result<ProcessStats> {
        let pid = std::process::id();
        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()?;
        let rss_kb = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u64>()
            .unwrap_or_default();
        let fd_count = fs::read_dir("/dev/fd")?.count();
        let reader_threads = match self.strategy {
            ReaderStrategy::SharedPoll => usize::from(self.shared_reader_alive.load(Ordering::Relaxed)),
            ReaderStrategy::PerSession => lock(&self.worker_threads).len(),
        };
        let reader_thread_alive = match self.strategy {
            ReaderStrategy::SharedPoll => self.shared_reader_alive.load(Ordering::Relaxed),
            ReaderStrategy::PerSession => reader_threads > 0,
        };

        Ok(ProcessStats {
            pid,
            rss_kb,
            fd_count,
            session_count: lock(&self.sessions).len(),
            reader_thread_alive,
            reader_threads,
            reader_strategy: self.strategy_name().to_string(),
        })
    }

    pub fn write_artifact(&self, request: ArtifactWriteRequest) -> io::Result<()> {
        let path = Path::new(&request.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, request.contents)
    }

    pub fn read_artifact(&self, request: ArtifactReadRequest) -> io::Result<String> {
        fs::read_to_string(request.path)
    }

    fn session(&self, session_id: &str) -> io::Result<Arc<SessionState>> {
        lock(&self.sessions)
            .get(session_id)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("unknown session: {session_id}")))
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        if let Some(control_tx) = &self.control_tx {
            let _ = control_tx.send(ReaderCommand::Shutdown);
        }
        self.close_all_sessions();
    }
}

fn reader_loop(control_rx: mpsc::Receiver<ReaderCommand>, reader_alive: Arc<AtomicBool>) {
    let mut poll = Poll::new().expect("poll init");
    let mut events = Events::with_capacity(256);
    let mut sessions = HashMap::<usize, Arc<SessionState>>::new();

    while reader_alive.load(Ordering::Relaxed) {
        while let Ok(command) = control_rx.try_recv() {
            match command {
                ReaderCommand::Register { token, fd, session } => {
                    let mut source = SourceFd(&fd);
                    if let Err(error) = poll.registry().register(&mut source, Token(token), Interest::READABLE) {
                        eprintln!("[SPIKE-05.5] register({token}) failed: {error}");
                        continue;
                    }
                    sessions.insert(token, session);
                }
                ReaderCommand::Unregister { token, fd } => {
                    let mut source = SourceFd(&fd);
                    let _ = poll.registry().deregister(&mut source);
                    sessions.remove(&token);
                }
                ReaderCommand::Shutdown => {
                    reader_alive.store(false, Ordering::Relaxed);
                    return;
                }
            }
        }

        if sessions.is_empty() {
            thread::sleep(Duration::from_millis(25));
            continue;
        }

        if let Err(error) = poll.poll(&mut events, Some(Duration::from_millis(50))) {
            eprintln!("[SPIKE-05.5] poll failed: {error}");
            continue;
        }

        for event in events.iter() {
            let Some(session) = sessions.get(&event.token().0) else {
                continue;
            };
            if event.is_readable() {
                read_session_fd(session, DropPolicy::DropOldest, true);
            }
        }
    }
}

fn per_session_reader_loop(session: Arc<SessionState>, policy: DropPolicy) {
    while !session.closed.load(Ordering::Relaxed) {
        if !read_session_fd(&session, policy, false) {
            break;
        }
    }
}

fn read_session_fd(session: &Arc<SessionState>, policy: DropPolicy, stop_on_would_block: bool) -> bool {
    let mut buffer = [0u8; 8192];

    loop {
        let read_started = Instant::now();
        let read = unsafe {
            libc::read(
                session.fd,
                buffer.as_mut_ptr().cast::<libc::c_void>(),
                buffer.len(),
            )
        };
        let read_syscall_ns = read_started.elapsed().as_nanos();

        if read > 0 {
            session.enqueue_bytes(&buffer[..read as usize], policy, read_syscall_ns);
            continue;
        }

        if read == 0 {
            let _ = session.try_wait();
            return false;
        }

        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(code) if code == libc::EAGAIN || code == libc::EWOULDBLOCK => {
                if stop_on_would_block {
                    return true;
                }
                thread::sleep(Duration::from_millis(1));
                return !session.closed.load(Ordering::Relaxed);
            }
            Some(code) if code == libc::EINTR => continue,
            Some(code) if code == libc::EIO => {
                let _ = session.try_wait();
                return false;
            }
            _ => {
                eprintln!("[SPIKE-05.5] read fd {} failed: {error}", session.fd);
                return false;
            }
        }
    }
}

fn set_fd_nonblocking(fd: i32, enabled: bool) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }

    let next = if enabled { flags | libc::O_NONBLOCK } else { flags & !libc::O_NONBLOCK };
    if unsafe { libc::fcntl(fd, libc::F_SETFL, next) } < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}
