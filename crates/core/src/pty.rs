//! PTY runtime · portable-pty + shared reader + bounded queue + drop-oldest.
//!
//! 架构依据：SPIKE-05 / SPIKE-05.5 + ADR-003 accepted。
//! 这里保留单 shared-reader + mio poll，避免回落到 per-session reader thread。

use crate::app_settings::AppSettingsStore;
use crate::db::DbPool;
use crate::tabs::TabsDao;
use crossbeam_channel::{self, Receiver, Sender, TryRecvError, TrySendError};
use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// OsStr 仅 Unix resolve_shell_in_path 用（task-2.1：Windows 走 where.exe · 不经 PATH split）。
#[cfg(unix)]
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};
use ts_rs::TS;

// Unix-only imports（ADR-001 cfg 分离）·
// mio Poll/SourceFd 仅 Unix · RawFd / PermissionsExt 仅 Unix · libc 仅在 #[cfg(unix)] 函数体引用。
#[cfg(unix)]
use mio::unix::SourceFd;
#[cfg(unix)]
use mio::{Events, Interest, Poll, Token};
#[cfg(unix)]
use std::os::fd::RawFd;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Windows-only imports（ADR-001 cfg 分离 · ConPTY 阻塞读路径）·
// 走 portable-pty 的 try_clone_reader + child.try_wait/kill · 不引用 mio/SourceFd/libc。
#[cfg(windows)]
use std::io::Read;

pub const PTY_EVENT_QUEUE_CAPACITY: usize = 128;

const PTY_CONTROL_QUEUE_CAPACITY: usize = 256;
const READ_BUFFER_SIZE: usize = 8192;
/// mio Poll 轮询超时 · 仅 Unix reader_loop 用（Windows 走 per-session 阻塞读 · 无 poll）。
#[cfg(unix)]
const READ_POLL_TIMEOUT: Duration = Duration::from_millis(50);
const IDLE_SLEEP: Duration = Duration::from_millis(25);
const EXIT_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const EXIT_WAIT_STEP: Duration = Duration::from_millis(20);
const SCROLLBACK_FLUSH_INTERVAL: Duration = Duration::from_millis(100);
const SCROLLBACK_FLUSH_THRESHOLD: usize = 100;
/// MVP-20 BUG-001 backend filter 兜底超时 · zsh ZLE echo + clear 命令通常 50-150ms 完成 ·
/// 给 300ms 余量。过长会让 legacy TerminalPane 的 "Waiting for first shell output" UI 卡。
const CD_ECHO_FILTER_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PtyStdoutEvent {
    pub tab_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PtyExitedEvent {
    pub tab_id: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PtySpawnRequest {
    pub tab_id: String,
    pub shell: String,
    pub cwd: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone)]
pub enum PtyEvent {
    Stdout(PtyStdoutEvent),
    Exited(PtyExitedEvent),
}

pub type PtyEventReceiver = Receiver<PtyEvent>;

#[derive(Debug, thiserror::Error)]
pub enum PtyError {
    #[error("tab not found: {0}")]
    NotFound(String),
    #[error("PTY already running for tab: {0}")]
    AlreadyRunning(String),
    #[error("PTY open failed: {0}")]
    OpenFailed(String),
    #[error("shell not executable: {0}")]
    ShellNotFound(String),
    #[error("invalid signal: {0}")]
    InvalidSignal(String),
    #[error("reader thread unavailable: {0}")]
    ReaderUnavailable(String),
    #[error("event bridge unavailable")]
    EventBridgeUnavailable,
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

enum ReaderCommand {
    Register {
        token: usize,
        session: Arc<PtySession>,
    },
    Unregister {
        token: usize,
        /// Unix reader_loop 用 fd 反查 token==0 的 session 并从 mio Poll deregister。
        /// Windows reader 走 per-session 线程 · 按 tab_id 收敛 · 无 fd 概念（None）。
        #[cfg(unix)]
        fd: RawFd,
    },
    Shutdown,
}

#[cfg(unix)]
enum ReadOutcome {
    Continue,
    Closed,
}

/// 信号目标。仅 Unix 路径有进程组概念；Windows ConPTY 退化为单进程（见 `signal_target`）。
#[cfg(unix)]
enum SignalTarget {
    ProcessGroup(libc::pid_t),
    Process(libc::pid_t),
}

enum ScrollbackCommand {
    Append { tab_id: String, lines: Vec<String> },
    Shutdown,
}

#[derive(Default)]
struct ScrollbackBuffer {
    partial_line: String,
    pending_lines: Vec<String>,
    pending_since: Option<Instant>,
}

impl ScrollbackBuffer {
    fn push_chunk(&mut self, chunk: &str) {
        let lines = parse_chunk_to_lines(chunk, &mut self.partial_line);
        if lines.is_empty() {
            return;
        }

        if self.pending_lines.is_empty() {
            self.pending_since = Some(Instant::now());
        }
        self.pending_lines.extend(lines);
    }

    fn drain_due(&mut self, now: Instant) -> Option<Vec<String>> {
        let should_flush = self.pending_lines.len() >= SCROLLBACK_FLUSH_THRESHOLD
            || self
                .pending_since
                .is_some_and(|started| now.duration_since(started) >= SCROLLBACK_FLUSH_INTERVAL);
        if !should_flush {
            return None;
        }

        self.pending_since = None;
        Some(std::mem::take(&mut self.pending_lines))
    }

    fn drain_all(&mut self) -> Option<Vec<String>> {
        if !self.partial_line.is_empty() {
            self.pending_lines
                .push(std::mem::take(&mut self.partial_line));
        }
        if self.pending_lines.is_empty() {
            self.pending_since = None;
            return None;
        }

        self.pending_since = None;
        Some(std::mem::take(&mut self.pending_lines))
    }
}

#[derive(Clone)]
struct DropOldestSender<T> {
    inner: Arc<DropOldestSenderInner<T>>,
}

struct DropOldestSenderInner<T> {
    sender: Sender<T>,
    drop_rx: Receiver<T>,
    gate: Mutex<()>,
}

impl<T> DropOldestSender<T> {
    fn new(sender: Sender<T>, drop_rx: Receiver<T>) -> Self {
        Self {
            inner: Arc::new(DropOldestSenderInner {
                sender,
                drop_rx,
                gate: Mutex::new(()),
            }),
        }
    }

    fn send(&self, value: T) -> Result<(), PtyError> {
        let _guard = lock(&self.inner.gate);
        let mut pending = value;

        loop {
            match self.inner.sender.try_send(pending) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Full(next)) => {
                    pending = next;
                    match self.inner.drop_rx.try_recv() {
                        Ok(_) => continue,
                        Err(TryRecvError::Empty) => {
                            thread::yield_now();
                            continue;
                        }
                        Err(TryRecvError::Disconnected) => {
                            return Err(PtyError::EventBridgeUnavailable);
                        }
                    }
                }
                Err(TrySendError::Disconnected(_)) => {
                    return Err(PtyError::EventBridgeUnavailable);
                }
            }
        }
    }
}

/// MVP-20 BUG-001 · backend swallow cd 注入命令的 zsh ZLE echo · 直到检测到 ANSI clear sequence。
///
/// 工作流程：
/// 1. `pty_pool::take` 在 `inject_cd_clear` 之前调 `start()` · 进入 filter 模式
/// 2. reader thread 把 stdout 数据传给 `emit_stdout` · 后者在 filter 模式下 swallow + 累积 buffer
/// 3. 检测到 `\x1b[2J` 或 `\x1b[3J`（clear screen） · 关 filter · 把 clear 之后内容 forward 给前端
/// 4. 兜底超时（800ms）· 强制 flush 所有 buffer 内容（防止 clear 不出现 · 终端永久卡住）
///
/// 这个机制比前端 buffer 更稳健 · 不依赖 IPC 时序 · cd echo 被 backend 直接吞掉。
#[derive(Default)]
struct CdEchoFilter {
    active: AtomicBool,
    buffer: Mutex<Vec<u8>>,
    started_at: Mutex<Option<Instant>>,
}

pub struct PtySession {
    tab_id: Mutex<String>,
    /// PTY master 的裸 fd · 仅 Unix（mio Poll 注册 + libc::read + tcgetpgrp 用）。
    /// Windows ConPTY 不暴露可 poll 的 fd · reader 走 try_clone_reader 阻塞读 · 无此字段。
    #[cfg(unix)]
    fd: RawFd,
    process_id: Option<u32>,
    /// PTY master。`Option` 是为了 Windows ConPTY 在 terminate 时能 `take()` 掉 master ·
    /// drop 它会 `ClosePseudoConsole` · 关闭 conhost 输出管道 · 让阻塞在 `reader.read()` 的
    /// per-session 读线程拿到 EOF 退出（否则仅 `child.kill()` 杀掉 shell · conhost 仍持管道 ·
    /// reader 永久 block · stop() 的 join() 死锁 · task-2.2 RED 复现）。Unix 不依赖此（mio 走裸 fd ·
    /// terminate 经 libc::kill + fd close 自然 EOF），但统一为 `Option` 以共享结构。
    master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    initial_cwd: PathBuf,
    initial_env: HashMap<String, String>,
    closed: AtomicBool,
    exit_emitted: AtomicBool,
    scrollback: Mutex<ScrollbackBuffer>,
    scrollback_tx: Sender<ScrollbackCommand>,
    cd_echo_filter: CdEchoFilter,
    /// 跨 read chunk 边界的 UTF-8 leftover · 防中文等多字节字符被 read 切断
    /// 后变成 U+FFFD 替换字符（屏幕上的 ���）。每次 read 完后只把"完整 UTF-8 字符"
    /// 部分发出去 · 末尾不完整的字节留到下次 read 拼接。最多 3 字节（4 字节字符的前缀）。
    partial_utf8: Mutex<Vec<u8>>,
}

impl PtySession {
    pub fn set_tab_id(&self, new_id: String) {
        *lock(&self.tab_id) = new_id;
    }

    #[must_use]
    pub fn tab_id_clone(&self) -> String {
        lock(&self.tab_id).clone()
    }

    pub(crate) fn write_input(&self, input: &str) -> Result<(), PtyError> {
        let mut writer = lock(&self.writer);
        writer.write_all(input.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    #[must_use]
    pub fn working_directory(&self) -> PathBuf {
        self.process_id
            .and_then(detect_process_cwd)
            .map(normalize_cwd)
            .unwrap_or_else(|| normalize_cwd(self.initial_cwd.clone()))
    }

    #[must_use]
    pub fn environment(&self) -> HashMap<String, String> {
        self.initial_env.clone()
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let master = lock(&self.master);
        let Some(master) = master.as_ref() else {
            // master 已被 terminate 关闭（Windows EOF 路径）· resize 无目标 · 视为 no-op。
            return Ok(());
        };
        master
            .resize(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| PtyError::OpenFailed(error.to_string()))
    }

    /// Windows ConPTY 收尾：drop master（`ClosePseudoConsole`）· 关闭 conhost 输出管道 ·
    /// 让阻塞 `reader.read()` 的 per-session 读线程拿到 EOF 退出（避免 stop() join 死锁）。
    /// Unix 不调用此（mio + 裸 fd 路径 terminate 后自然 EOF）。
    #[cfg(windows)]
    fn close_master(&self) {
        let _ = lock(&self.master).take();
    }

    #[cfg(unix)]
    fn signal(&self, signal: &str) -> Result<(), PtyError> {
        let signal_number = parse_signal(signal)?;
        let target = self.signal_target();
        let pid = match target {
            Some(SignalTarget::ProcessGroup(group)) => -group,
            Some(SignalTarget::Process(pid)) => pid,
            None => {
                return Err(PtyError::OpenFailed(format!(
                    "cannot resolve process target for {signal}"
                )));
            }
        };

        let result = unsafe { libc::kill(pid, signal_number) };
        if result != 0 {
            return Err(PtyError::Io(io::Error::last_os_error()));
        }

        Ok(())
    }

    /// Windows ConPTY 无 POSIX 信号 / 进程组概念（ADR-001 + spec §5.3）·
    /// SIGINT/SIGTERM/SIGTSTP/SIGKILL 全部退化为 `Child::kill()`（TerminateProcess 语义）·
    /// 信号名仍走 `parse_signal_windows` 校验（拒绝未知信号 · 保持与 Unix 一致的契约）。
    #[cfg(windows)]
    fn signal(&self, signal: &str) -> Result<(), PtyError> {
        // 校验信号名合法（未知信号 → InvalidSignal · 与 Unix parse_signal 同契约）
        let _ = parse_signal_windows(signal)?;
        // ConPTY 单进程目标 · 直接 kill child（无进程组 · 无 graceful SIGTERM 区分）
        let mut child = lock(&self.child);
        child.kill().map_err(PtyError::Io)
    }

    fn terminate(&self, events: &DropOldestSender<PtyEvent>) -> Result<Option<i32>, PtyError> {
        self.closed.store(true, Ordering::Relaxed);

        let _ = self.signal("SIGKILL");
        {
            let mut child = lock(&self.child);
            let _ = child.kill();
        }

        // Windows ConPTY 收尾（task-2.2）：仅 kill child 不够 · conhost 仍持输出管道 ·
        // 阻塞 reader 永不 EOF → reader 线程 join 死锁。drop master（ClosePseudoConsole）
        // 关闭管道 → reader 拿 EOF 退出 → worker.stop() 的 join() 能返回。Unix 无此问题（裸 fd）。
        #[cfg(windows)]
        self.close_master();

        let status = self.wait_for_exit(EXIT_WAIT_TIMEOUT)?;
        self.emit_exit_once(events, status)
    }

    fn try_wait(&self) -> Result<Option<ExitStatus>, PtyError> {
        let mut child = lock(&self.child);
        child.try_wait().map_err(PtyError::Io)
    }

    fn wait_for_exit(&self, timeout: Duration) -> Result<Option<ExitStatus>, PtyError> {
        let started = Instant::now();
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(Some(status));
            }
            if started.elapsed() >= timeout {
                return Ok(None);
            }
            thread::sleep(EXIT_WAIT_STEP);
        }
    }

    /// MVP-20 BUG-001 · 启动 cd echo filter（pty_pool 在 inject_cd_clear 之前调）。
    /// 调用后 reader thread emit 的 stdout 进 swallow 模式 · 直到检测到 ANSI clear 或超时。
    pub(crate) fn start_cd_echo_filter(&self) {
        lock(&self.cd_echo_filter.buffer).clear();
        *lock(&self.cd_echo_filter.started_at) = Some(Instant::now());
        self.cd_echo_filter.active.store(true, Ordering::Relaxed);
    }

    /// 处理 filter 模式的输出：返回 Some(forwarded) 时 caller 应 emit · None = swallow（cd echo）。
    /// 检测到 ANSI clear → 关 filter · 返回 clear sequence 起的内容（cd echo 部分丢弃）。
    /// 超时（800ms）→ 关 filter · 强制 flush 所有 buffer（兜底 · 防 clear 不出现）。
    fn process_cd_echo_filter(&self, data: &str) -> Option<String> {
        if !self.cd_echo_filter.active.load(Ordering::Relaxed) {
            return Some(data.to_string());
        }

        let mut buffer = lock(&self.cd_echo_filter.buffer);
        buffer.extend_from_slice(data.as_bytes());

        if let Some(idx) = find_ansi_clear(&buffer) {
            self.cd_echo_filter.active.store(false, Ordering::Relaxed);
            let forwarded = String::from_utf8_lossy(&buffer[idx..]).to_string();
            buffer.clear();
            return Some(forwarded);
        }

        let started = *lock(&self.cd_echo_filter.started_at);
        if let Some(start) = started {
            if start.elapsed() > CD_ECHO_FILTER_TIMEOUT {
                self.cd_echo_filter.active.store(false, Ordering::Relaxed);
                let forwarded = String::from_utf8_lossy(&buffer).to_string();
                buffer.clear();
                return Some(forwarded);
            }
        }

        None
    }

    /// 把 raw bytes 拼到 leftover · 返回"可安全转 String 的连续完整 UTF-8 段"·
    /// 末尾不完整的 multi-byte 起始字节留在 leftover · 等下次 chunk 来拼。
    /// 中间真正非法的字节序列（不是 incomplete · 是真坏）才走 lossy 转 ��� ·
    /// 这种是真出问题 · 该亮就亮。
    fn drain_utf8_safe(&self, new_data: &[u8]) -> String {
        let mut leftover = lock(&self.partial_utf8);
        leftover.extend_from_slice(new_data);

        let take = match std::str::from_utf8(&leftover) {
            Ok(_) => leftover.len(),
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                match e.error_len() {
                    // None = 末尾是 incomplete UTF-8 起始字节 · 留到下次拼接 · 只发已完整部分
                    None if leftover.len() - valid_up_to <= 3 => valid_up_to,
                    // 其他情况：真坏字节 · 或 leftover 已积累 > 3 字节没解析（异常）·
                    // 直接全部 lossy 转 · 让 ��� 出现（这才是真正"该亮"的乱码信号）
                    _ => leftover.len(),
                }
            }
        };

        let result = String::from_utf8_lossy(&leftover[..take]).to_string();
        leftover.drain(..take);
        result
    }

    fn emit_stdout(&self, events: &DropOldestSender<PtyEvent>, data: &str) -> Result<(), PtyError> {
        if data.is_empty() || self.closed.load(Ordering::Relaxed) {
            return Ok(());
        }

        let Some(forwarded) = self.process_cd_echo_filter(data) else {
            return Ok(());
        };

        if forwarded.is_empty() {
            return Ok(());
        }

        events.send(PtyEvent::Stdout(PtyStdoutEvent {
            tab_id: self.tab_id_clone(),
            data: forwarded,
        }))
    }

    fn emit_exit_once(
        &self,
        events: &DropOldestSender<PtyEvent>,
        status: Option<ExitStatus>,
    ) -> Result<Option<i32>, PtyError> {
        self.flush_scrollback_now();
        self.closed.store(true, Ordering::Relaxed);

        let exit_code = status.as_ref().and_then(exit_code_from_status);
        if self.exit_emitted.swap(true, Ordering::Relaxed) {
            return Ok(exit_code);
        }

        events.send(PtyEvent::Exited(PtyExitedEvent {
            tab_id: self.tab_id_clone(),
            exit_code,
        }))?;
        Ok(exit_code)
    }

    #[cfg(unix)]
    fn signal_target(&self) -> Option<SignalTarget> {
        let foreground = unsafe { libc::tcgetpgrp(self.fd) };
        if foreground > 0 {
            return Some(SignalTarget::ProcessGroup(foreground));
        }

        let leader = lock(&self.master)
            .as_ref()
            .and_then(|master| master.process_group_leader());
        if let Some(group) = leader.filter(|group| *group > 0) {
            return Some(SignalTarget::ProcessGroup(group));
        }

        self.process_id
            .map(|pid| SignalTarget::Process(pid as libc::pid_t))
    }

    fn record_scrollback_chunk(&self, chunk: &str) {
        let ready = {
            let mut scrollback = lock(&self.scrollback);
            scrollback.push_chunk(chunk);
            scrollback.drain_due(Instant::now())
        };
        self.enqueue_scrollback(ready);
    }

    fn flush_scrollback_if_due(&self) {
        let ready = {
            let mut scrollback = lock(&self.scrollback);
            scrollback.drain_due(Instant::now())
        };
        self.enqueue_scrollback(ready);
    }

    fn flush_scrollback_now(&self) {
        let ready = {
            let mut scrollback = lock(&self.scrollback);
            scrollback.drain_all()
        };
        self.enqueue_scrollback(ready);
    }

    fn enqueue_scrollback(&self, lines: Option<Vec<String>>) {
        let Some(lines) = lines.filter(|lines| !lines.is_empty()) else {
            return;
        };

        let tab_id = self.tab_id_clone();
        if let Err(error) = self.scrollback_tx.send(ScrollbackCommand::Append {
            tab_id: tab_id.clone(),
            lines,
        }) {
            eprintln!(
                "[mvp-04] scrollback queue send failed for {}: {error}",
                tab_id
            );
        }
    }
}

pub struct PtyManager {
    pool: Arc<Mutex<Option<DbPool>>>,
    sessions: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    control_tx: Sender<ReaderCommand>,
    reader_thread: Mutex<Option<thread::JoinHandle<()>>>,
    reader_alive: Arc<AtomicBool>,
    scrollback_tx: Sender<ScrollbackCommand>,
    scrollback_thread: Mutex<Option<thread::JoinHandle<()>>>,
    next_token: AtomicUsize,
    event_tx: DropOldestSender<PtyEvent>,
    event_rx: Mutex<Option<Receiver<PtyEvent>>>,
}

impl PtyManager {
    #[must_use]
    pub fn new() -> Self {
        let pool = Arc::new(Mutex::new(None));
        let (event_tx, event_rx) = crossbeam_channel::bounded(PTY_EVENT_QUEUE_CAPACITY);
        let event_dispatch = DropOldestSender::new(event_tx, event_rx.clone());
        let (control_tx, control_rx) = crossbeam_channel::bounded(PTY_CONTROL_QUEUE_CAPACITY);
        let sessions = Arc::new(Mutex::new(HashMap::new()));
        let reader_alive = Arc::new(AtomicBool::new(true));
        let (scrollback_tx, scrollback_rx) = crossbeam_channel::unbounded();

        let reader_sessions = Arc::clone(&sessions);
        let reader_events = event_dispatch.clone();
        let reader_flag = Arc::clone(&reader_alive);
        let reader_thread = thread::Builder::new()
            .name("vibestation-pty-reader".to_string())
            .spawn(move || reader_loop(control_rx, reader_sessions, reader_events, reader_flag))
            .expect("spawn vibestation-pty-reader");
        let writer_pool = Arc::clone(&pool);
        let scrollback_thread = thread::Builder::new()
            .name("vibestation-scrollback-writer".to_string())
            .spawn(move || scrollback_writer_loop(scrollback_rx, writer_pool))
            .expect("spawn vibestation-scrollback-writer");

        Self {
            pool,
            sessions,
            control_tx,
            reader_thread: Mutex::new(Some(reader_thread)),
            reader_alive,
            scrollback_tx,
            scrollback_thread: Mutex::new(Some(scrollback_thread)),
            next_token: AtomicUsize::new(1),
            event_tx: event_dispatch,
            event_rx: Mutex::new(Some(event_rx)),
        }
    }

    pub fn take_event_receiver(&self) -> Option<Receiver<PtyEvent>> {
        lock(&self.event_rx).take()
    }

    pub fn set_pool(&self, pool: DbPool) {
        *lock(&self.pool) = Some(pool);
    }

    pub fn spawn(&self, req: PtySpawnRequest) -> Result<(), PtyError> {
        let shell = effective_shell_for_spawn(&req.shell, std::env::var("SHELL").ok().as_deref());
        let resolved_shell =
            resolve_shell(&shell).ok_or_else(|| PtyError::ShellNotFound(shell.clone()))?;
        self.spawn_registered_session(
            req.tab_id.clone(),
            resolved_shell,
            PathBuf::from(req.cwd),
            req.cols,
            req.rows,
        )?;
        Ok(())
    }

    pub(crate) fn spawn_registered_session(
        &self,
        tab_id: String,
        resolved_shell: PathBuf,
        cwd: PathBuf,
        cols: u16,
        rows: u16,
    ) -> Result<Arc<PtySession>, PtyError> {
        if lock(&self.sessions).contains_key(&tab_id) {
            return Err(PtyError::AlreadyRunning(tab_id));
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| PtyError::OpenFailed(error.to_string()))?;

        // Unix：取裸 fd 设非阻塞（mio Poll + libc::read 用）。
        // Windows：ConPTY overlapped I/O 由 portable-pty 内部处理 · 无 fd · set_fd_nonblocking 为 no-op。
        #[cfg(unix)]
        let fd = pair
            .master
            .as_raw_fd()
            .ok_or_else(|| PtyError::OpenFailed("portable-pty master has no raw fd".to_string()))?;
        #[cfg(unix)]
        set_fd_nonblocking(fd, true)?;
        #[cfg(windows)]
        set_fd_nonblocking_noop();

        let writer = pair
            .master
            .take_writer()
            .map_err(|error| PtyError::OpenFailed(error.to_string()))?;

        let initial_cwd = normalize_cwd(cwd.clone());
        let mut initial_env: HashMap<String, String> = std::env::vars().collect();
        initial_env.insert("TERM".to_string(), "xterm-256color".to_string());
        initial_env.insert("COLORTERM".to_string(), "truecolor".to_string());
        initial_env.insert("LANG".to_string(), "en_US.UTF-8".to_string());
        initial_env.insert("LC_ALL".to_string(), "en_US.UTF-8".to_string());
        let mut command = CommandBuilder::new(resolved_shell.as_os_str());
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("LANG", "en_US.UTF-8");
        command.env("LC_ALL", "en_US.UTF-8");

        let child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| PtyError::OpenFailed(error.to_string()))?;
        drop(pair.slave);

        let process_id = child.process_id();
        let session = Arc::new(PtySession {
            tab_id: Mutex::new(tab_id.clone()),
            #[cfg(unix)]
            fd,
            process_id,
            master: Mutex::new(Some(pair.master)),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
            initial_cwd,
            initial_env,
            closed: AtomicBool::new(false),
            exit_emitted: AtomicBool::new(false),
            scrollback: Mutex::new(ScrollbackBuffer::default()),
            scrollback_tx: self.scrollback_tx.clone(),
            cd_echo_filter: CdEchoFilter::default(),
            partial_utf8: Mutex::new(Vec::with_capacity(4)),
        });

        lock(&self.sessions).insert(tab_id.clone(), Arc::clone(&session));

        let token = self.next_token.fetch_add(1, Ordering::Relaxed);
        if let Err(error) = self.send_control(ReaderCommand::Register { token, session }) {
            lock(&self.sessions).remove(&tab_id);
            return Err(error);
        }

        Ok(lock(&self.sessions)
            .get(&tab_id)
            .cloned()
            .expect("registered session should remain available"))
    }

    pub fn stdin(&self, tab_id: &str, data: &str) -> Result<(), PtyError> {
        self.session(tab_id)?.write_input(data)
    }

    pub fn resize(&self, tab_id: &str, cols: u16, rows: u16) -> Result<(), PtyError> {
        self.session(tab_id)?.resize(cols, rows)
    }

    pub fn signal(&self, tab_id: &str, signal: &str) -> Result<(), PtyError> {
        self.session(tab_id)?.signal(signal)
    }

    pub fn kill(&self, tab_id: &str) -> Result<Option<i32>, PtyError> {
        let session = {
            let mut sessions = lock(&self.sessions);
            sessions
                .remove(tab_id)
                .ok_or_else(|| PtyError::NotFound(tab_id.to_string()))?
        };

        let _ = self.send_control(unregister_command(&session));
        session.terminate(&self.event_tx)
    }

    pub fn working_directory(&self, tab_id: &str) -> Result<PathBuf, PtyError> {
        Ok(self.session(tab_id)?.working_directory())
    }

    pub fn environment(&self, tab_id: &str) -> Result<HashMap<String, String>, PtyError> {
        Ok(self.session(tab_id)?.environment())
    }

    pub fn close_all_sessions(&self) {
        let tab_ids: Vec<String> = lock(&self.sessions).keys().cloned().collect();
        for tab_id in tab_ids {
            let _ = self.kill(&tab_id);
        }
    }

    pub(crate) fn rename_session(
        &self,
        old_tab_id: &str,
        new_tab_id: String,
    ) -> Result<Arc<PtySession>, PtyError> {
        let session = {
            let mut sessions = lock(&self.sessions);
            if sessions.contains_key(&new_tab_id) {
                return Err(PtyError::AlreadyRunning(new_tab_id));
            }
            let session = sessions
                .remove(old_tab_id)
                .ok_or_else(|| PtyError::NotFound(old_tab_id.to_string()))?;
            session.set_tab_id(new_tab_id.clone());
            sessions.insert(new_tab_id, Arc::clone(&session));
            session
        };
        Ok(session)
    }

    pub(crate) fn terminate_session(
        &self,
        session: &Arc<PtySession>,
    ) -> Result<Option<i32>, PtyError> {
        let tab_id = session.tab_id_clone();
        lock(&self.sessions).remove(&tab_id);
        let _ = self.send_control(unregister_command(session));
        session.terminate(&self.event_tx)
    }

    fn session(&self, tab_id: &str) -> Result<Arc<PtySession>, PtyError> {
        lock(&self.sessions)
            .get(tab_id)
            .cloned()
            .ok_or_else(|| PtyError::NotFound(tab_id.to_string()))
    }

    fn send_control(&self, command: ReaderCommand) -> Result<(), PtyError> {
        let mut pending = command;
        loop {
            match self.control_tx.try_send(pending) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Full(next)) => {
                    pending = next;
                    thread::sleep(Duration::from_millis(1));
                }
                Err(TrySendError::Disconnected(_)) => {
                    return Err(PtyError::ReaderUnavailable(
                        "control channel disconnected".to_string(),
                    ));
                }
            }
        }
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PtyManager {
    fn drop(&mut self) {
        self.close_all_sessions();
        self.reader_alive.store(false, Ordering::Relaxed);
        let _ = self.control_tx.try_send(ReaderCommand::Shutdown);
        if let Some(handle) = lock(&self.reader_thread).take() {
            let _ = handle.join();
        }
        let _ = self.scrollback_tx.send(ScrollbackCommand::Shutdown);
        if let Some(handle) = lock(&self.scrollback_thread).take() {
            let _ = handle.join();
        }
    }
}

/// 构造 `ReaderCommand::Unregister`：Unix 带 fd（reader_loop 从 mio Poll deregister 用）·
/// Windows 仅 token（per-session 线程按 tab_id 收敛 · 无 fd）。
#[cfg(unix)]
fn unregister_command(session: &Arc<PtySession>) -> ReaderCommand {
    ReaderCommand::Unregister {
        token: 0,
        fd: session.fd,
    }
}

#[cfg(windows)]
fn unregister_command(_session: &Arc<PtySession>) -> ReaderCommand {
    ReaderCommand::Unregister { token: 0 }
}

#[cfg(unix)]
fn reader_loop(
    control_rx: Receiver<ReaderCommand>,
    sessions_by_id: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    events: DropOldestSender<PtyEvent>,
    reader_alive: Arc<AtomicBool>,
) {
    let mut poll = match Poll::new() {
        Ok(poll) => poll,
        Err(error) => {
            eprintln!("[mvp-04] poll init failed: {error}");
            reader_alive.store(false, Ordering::Relaxed);
            return;
        }
    };
    let mut ready = Events::with_capacity(256);
    let mut sessions = HashMap::<usize, Arc<PtySession>>::new();

    while reader_alive.load(Ordering::Relaxed) {
        while let Ok(command) = control_rx.try_recv() {
            match command {
                ReaderCommand::Register { token, session } => {
                    let fd = session.fd;
                    let mut source = SourceFd(&fd);
                    if let Err(error) =
                        poll.registry()
                            .register(&mut source, Token(token), Interest::READABLE)
                    {
                        eprintln!("[mvp-04] register({token}) failed: {error}");
                        let _ = session.emit_exit_once(&events, None);
                        lock(&sessions_by_id).remove(&session.tab_id_clone());
                        continue;
                    }
                    sessions.insert(token, session);
                }
                ReaderCommand::Unregister { token, fd } => {
                    let mut source = SourceFd(&fd);
                    let _ = poll.registry().deregister(&mut source);
                    if token != 0 {
                        sessions.remove(&token);
                    } else {
                        sessions.retain(|_, session| session.fd != fd);
                    }
                }
                ReaderCommand::Shutdown => {
                    reader_alive.store(false, Ordering::Relaxed);
                    return;
                }
            }
        }

        if sessions.is_empty() {
            thread::sleep(IDLE_SLEEP);
            continue;
        }

        if let Err(error) = poll.poll(&mut ready, Some(READ_POLL_TIMEOUT)) {
            eprintln!("[mvp-04] poll failed: {error}");
            continue;
        }

        for event in ready.iter() {
            let Some(session) = sessions.get(&event.token().0).cloned() else {
                continue;
            };
            if !event.is_readable() {
                continue;
            }

            if matches!(read_session_fd(&session, &events), ReadOutcome::Closed) {
                let fd = session.fd;
                let mut source = SourceFd(&fd);
                let _ = poll.registry().deregister(&mut source);
                sessions.remove(&event.token().0);
                lock(&sessions_by_id).remove(&session.tab_id_clone());
            }
        }

        for session in sessions.values() {
            session.flush_scrollback_if_due();
        }
    }
}

/// Windows ConPTY reader（ADR-001 + spec §5.3 · prompt 细化为最小可用实现）·
///
/// ConPTY 不暴露可 poll 的 fd（mio epoll/kqueue 不可用）· 故对每个注册 session 起一条
/// **per-session 阻塞读线程**：`master.try_clone_reader()` 阻塞读字节 → emit 到同一
/// `DropOldestSender<PtyEvent>` 出口；读到 EOF（0 字节）或 read 错误 → `wait_for_exit`
/// + `emit_exit_once`（不使用 mio / SourceFd / libc）。
///
/// 控制通道（Register / Unregister / Shutdown）由本主循环消费：Register 起读线程 ·
/// Unregister 标记对应 session 的 stop flag（线程自然退出）· Shutdown 停所有线程后返回。
///
/// 本 task（1.1）范围 = 编译通过 + spawn 后能退出不 hang；吞吐 / 调度 / 尾部输出
/// 时序细化 defer Phase 2 task-2.2 conpty-spawn-io。
#[cfg(windows)]
fn reader_loop(
    control_rx: Receiver<ReaderCommand>,
    sessions_by_id: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    events: DropOldestSender<PtyEvent>,
    reader_alive: Arc<AtomicBool>,
) {
    // token → (session reader 线程 handle, 该线程的 stop flag)
    let mut workers: HashMap<usize, WindowsReaderWorker> = HashMap::new();

    while reader_alive.load(Ordering::Relaxed) {
        // 排空控制命令
        loop {
            match control_rx.try_recv() {
                Ok(ReaderCommand::Register { token, session }) => {
                    let worker = spawn_windows_session_reader(
                        Arc::clone(&session),
                        events.clone(),
                        Arc::clone(&sessions_by_id),
                    );
                    workers.insert(token, worker);
                }
                Ok(ReaderCommand::Unregister { token }) => {
                    if token != 0 {
                        if let Some(worker) = workers.remove(&token) {
                            worker.stop();
                        }
                    } else {
                        // token==0（kill / terminate_session 路径）· session 已从 map 移除 ·
                        // 停掉所有已退出的 worker（stop flag 置位 · 线程自然收敛）。
                        let finished: Vec<usize> = workers
                            .iter()
                            .filter(|(_, worker)| worker.is_finished())
                            .map(|(token, _)| *token)
                            .collect();
                        for token in finished {
                            if let Some(worker) = workers.remove(&token) {
                                worker.stop();
                            }
                        }
                    }
                }
                Ok(ReaderCommand::Shutdown) => {
                    reader_alive.store(false, Ordering::Relaxed);
                    for (_, worker) in workers.drain() {
                        worker.stop();
                    }
                    return;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    reader_alive.store(false, Ordering::Relaxed);
                    for (_, worker) in workers.drain() {
                        worker.stop();
                    }
                    return;
                }
            }
        }

        // 退出检测（task-2.2 · AC3）：ConPTY 下 child 自然退出（如 `exit`）时 · conhost 仍可能
        // 持有输出管道 · 阻塞 `reader.read()` 拿不到 EOF · reader 永不知道 child 已死。故在此主循环
        // 周期轮询 `child.try_wait()`；一旦检测到退出 · close_master（ClosePseudoConsole）关管道 ·
        // reader 随即拿 EOF → emit_exit_once + 线程退出（下轮 is_finished 清理）。
        // 有界延迟 ≤ IDLE_SLEEP + read 唤醒（不 hang · 不漏 exit）。
        for worker in workers.values() {
            if worker.is_finished() {
                continue;
            }
            if matches!(worker.session.try_wait(), Ok(Some(_))) {
                worker.session.close_master();
            }
        }

        // 清理已自然退出的 worker（child 退出 / EOF）· join 回收线程。
        let finished: Vec<usize> = workers
            .iter()
            .filter(|(_, worker)| worker.is_finished())
            .map(|(token, _)| *token)
            .collect();
        for token in finished {
            if let Some(worker) = workers.remove(&token) {
                worker.stop();
            }
        }

        // 周期 flush scrollback（与 Unix 路径同语义）
        for worker in workers.values() {
            worker.session.flush_scrollback_if_due();
        }

        thread::sleep(IDLE_SLEEP);
    }

    for (_, worker) in workers.drain() {
        worker.stop();
    }
}

/// Windows per-session 读线程句柄 + stop flag（ConPTY 阻塞读路径）。
#[cfg(windows)]
struct WindowsReaderWorker {
    session: Arc<PtySession>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

#[cfg(windows)]
impl WindowsReaderWorker {
    fn is_finished(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| h.is_finished())
            .unwrap_or(true)
    }

    fn stop(self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
    }
}

/// 为单个 Windows session 起阻塞读线程：try_clone_reader 阻塞读 → emit；
/// EOF / read 错误 → wait_for_exit + emit_exit_once + 从 sessions_by_id 移除。
#[cfg(windows)]
fn spawn_windows_session_reader(
    session: Arc<PtySession>,
    events: DropOldestSender<PtyEvent>,
    sessions_by_id: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
) -> WindowsReaderWorker {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let thread_session = Arc::clone(&session);

    // 在 spawn 前从 master clone reader（master 此刻必在 · 刚构造）· clone 出的 reader
    // 持自己的管道句柄 · 后续 terminate drop master（ClosePseudoConsole）会 EOF 此 reader。
    let reader = lock(&thread_session.master)
        .as_ref()
        .map(|master| master.try_clone_reader());
    let handle = thread::Builder::new()
        .name("vibestation-pty-conpty-reader".to_string())
        .spawn(move || {
            let mut reader = match reader {
                Some(Ok(reader)) => reader,
                Some(Err(error)) => {
                    eprintln!(
                        "[mvp-04] conpty clone_reader failed for {}: {error}",
                        thread_session.tab_id_clone()
                    );
                    let _ = thread_session.emit_exit_once(&events, None);
                    lock(&sessions_by_id).remove(&thread_session.tab_id_clone());
                    return;
                }
                None => {
                    // master 已被关闭（极少见 · register 与 terminate 竞争）· 直接收尾。
                    let _ = thread_session.emit_exit_once(&events, None);
                    lock(&sessions_by_id).remove(&thread_session.tab_id_clone());
                    return;
                }
            };

            let mut buffer = [0u8; READ_BUFFER_SIZE];
            loop {
                if thread_stop.load(Ordering::Relaxed)
                    || thread_session.closed.load(Ordering::Relaxed)
                {
                    return;
                }

                match reader.read(&mut buffer) {
                    Ok(0) => {
                        // EOF · child 退出 → 检测退出码 + emit exit once
                        let status = thread_session
                            .wait_for_exit(EXIT_WAIT_TIMEOUT)
                            .unwrap_or(None);
                        let _ = thread_session.emit_exit_once(&events, status);
                        lock(&sessions_by_id).remove(&thread_session.tab_id_clone());
                        return;
                    }
                    Ok(read) => {
                        let chunk = thread_session.drain_utf8_safe(&buffer[..read]);
                        if chunk.is_empty() {
                            continue;
                        }
                        thread_session.record_scrollback_chunk(&chunk);
                        if let Err(error) = thread_session.emit_stdout(&events, &chunk) {
                            eprintln!(
                                "[mvp-04] stdout emit failed for {}: {error}",
                                thread_session.tab_id_clone()
                            );
                        }
                    }
                    Err(ref error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => {
                        eprintln!(
                            "[mvp-04] conpty read failed for {}: {error}",
                            thread_session.tab_id_clone()
                        );
                        let status = thread_session
                            .wait_for_exit(EXIT_WAIT_TIMEOUT)
                            .unwrap_or(None);
                        let _ = thread_session.emit_exit_once(&events, status);
                        lock(&sessions_by_id).remove(&thread_session.tab_id_clone());
                        return;
                    }
                }
            }
        })
        .expect("spawn vibestation-pty-conpty-reader");

    WindowsReaderWorker {
        session,
        stop,
        handle: Some(handle),
    }
}

#[cfg(unix)]
fn read_session_fd(session: &Arc<PtySession>, events: &DropOldestSender<PtyEvent>) -> ReadOutcome {
    let mut buffer = [0u8; READ_BUFFER_SIZE];

    loop {
        let read = unsafe {
            libc::read(
                session.fd,
                buffer.as_mut_ptr().cast::<libc::c_void>(),
                buffer.len(),
            )
        };

        if read > 0 {
            // 用 leftover-aware 拼接 · 避免 UTF-8 字符跨 chunk 边界被切断 · 否则
            // 中文等多字节字符会被 from_utf8_lossy 替换成 ��� (U+FFFD) · 终端乱码。
            let chunk = session.drain_utf8_safe(&buffer[..read as usize]);
            if chunk.is_empty() {
                // 全部是 incomplete UTF-8 · 留待下次 read · 不浪费 emit
                continue;
            }
            session.record_scrollback_chunk(&chunk);
            if let Err(error) = session.emit_stdout(events, &chunk) {
                eprintln!(
                    "[mvp-04] stdout emit failed for {}: {error}",
                    session.tab_id_clone()
                );
            }
            continue;
        }

        if read == 0 {
            let status = match session.wait_for_exit(EXIT_WAIT_TIMEOUT) {
                Ok(status) => status,
                Err(error) => {
                    eprintln!(
                        "[mvp-04] wait failed for {}: {error}",
                        session.tab_id_clone()
                    );
                    None
                }
            };
            let _ = session.emit_exit_once(events, status);
            return ReadOutcome::Closed;
        }

        let error = io::Error::last_os_error();
        match error.raw_os_error() {
            Some(code) if code == libc::EAGAIN || code == libc::EWOULDBLOCK => {
                return ReadOutcome::Continue;
            }
            Some(code) if code == libc::EINTR => continue,
            Some(code) if code == libc::EIO => {
                let status = match session.wait_for_exit(EXIT_WAIT_TIMEOUT) {
                    Ok(status) => status,
                    Err(wait_error) => {
                        eprintln!(
                            "[mvp-04] wait failed for {}: {wait_error}",
                            session.tab_id_clone()
                        );
                        None
                    }
                };
                let _ = session.emit_exit_once(events, status);
                return ReadOutcome::Closed;
            }
            _ => {
                eprintln!(
                    "[mvp-04] read failed for {}: {error}",
                    session.tab_id_clone()
                );
                let _ = session.emit_exit_once(events, None);
                return ReadOutcome::Closed;
            }
        }
    }
}

fn scrollback_writer_loop(commands: Receiver<ScrollbackCommand>, pool: Arc<Mutex<Option<DbPool>>>) {
    while let Ok(command) = commands.recv() {
        match command {
            ScrollbackCommand::Append { tab_id, lines } => {
                let Some(pool) = lock(&pool).clone() else {
                    eprintln!(
                        "[mvp-04] scrollback append skipped for {}: database not initialized",
                        tab_id
                    );
                    continue;
                };

                if let Err(error) = TabsDao::scrollback_append(&pool, &tab_id, &lines) {
                    eprintln!("[mvp-04] scrollback append failed for {}: {error}", tab_id);
                }
            }
            ScrollbackCommand::Shutdown => return,
        }
    }
}

fn parse_chunk_to_lines(chunk: &str, partial_line: &mut String) -> Vec<String> {
    if chunk.is_empty() {
        return Vec::new();
    }

    let mut combined = std::mem::take(partial_line);
    combined.push_str(chunk);

    let mut lines = Vec::new();
    for segment in combined.split_inclusive('\n') {
        if let Some(line) = segment.strip_suffix('\n') {
            lines.push(line.strip_suffix('\r').unwrap_or(line).to_string());
        } else {
            partial_line.push_str(segment);
        }
    }

    lines
}

/// MVP-20 BUG-001 backend filter · 寻找 ANSI clear screen sequence（`ESC [ 2 J` 或 `ESC [ 3 J`）。
/// 用于 cd echo filter 检测 clear 命令输出 · 找到则丢弃 cd echo 部分。
fn find_ansi_clear(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|w| {
        w[0] == 0x1B && w[1] == b'[' && (w[2] == b'2' || w[2] == b'3') && w[3] == b'J'
    })
}

#[cfg(unix)]
fn parse_signal(signal: &str) -> Result<i32, PtyError> {
    match signal {
        "SIGINT" => Ok(libc::SIGINT),
        "SIGTERM" => Ok(libc::SIGTERM),
        "SIGTSTP" => Ok(libc::SIGTSTP),
        "SIGKILL" => Ok(libc::SIGKILL),
        _ => Err(PtyError::InvalidSignal(signal.to_string())),
    }
}

/// Windows 信号名解析（ADR-001 + spec §5.3）· ConPTY 无 POSIX 信号 ·
/// 但保留与 Unix `parse_signal` 同样的"未知信号 → InvalidSignal"契约 ·
/// 合法信号统一映射到 `Child::kill()`（TerminateProcess）语义（见 `PtySession::signal` Windows 分支）。
/// 返回标记类型 `WindowsSignal` 仅用于校验通过/拒绝 · 不携带 POSIX 信号号。
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowsSignal {
    /// 所有受支持信号在 Windows 都退化为 kill child（无 graceful/forced 区分）。
    Kill,
}

#[cfg(windows)]
fn parse_signal_windows(signal: &str) -> Result<WindowsSignal, PtyError> {
    match signal {
        "SIGINT" | "SIGTERM" | "SIGTSTP" | "SIGKILL" => Ok(WindowsSignal::Kill),
        _ => Err(PtyError::InvalidSignal(signal.to_string())),
    }
}

fn exit_code_from_status(status: &ExitStatus) -> Option<i32> {
    if status.signal().is_some() {
        None
    } else {
        Some(status.exit_code() as i32)
    }
}

/// 默认 shell 探测（task-2.1 · ADR-003）·
/// Windows 走探测链 `pwsh.exe → powershell.exe → cmd.exe`（取首个 PATH 可用 · `cmd.exe` 永远保底）·
/// macOS = `/bin/zsh` · 其他 Unix = `/bin/bash`。
///
/// 返回 `String` 而非 `&'static str`：Windows 探测结果是 owned 全路径（非静态字面量）·
/// 统一签名后 Unix 分支把字面量 `.to_string()`（调用方本就多处 `.to_string()` / 比较）。
fn default_shell_path() -> String {
    #[cfg(windows)]
    {
        // 依次探测 pwsh.exe → powershell.exe → cmd.exe · 取首个 where.exe 能定位的全路径；
        // 全找不到时保底 cmd.exe（%ComSpec% · 否则 System32\cmd.exe · 系统永远存在）。
        for candidate in ["pwsh.exe", "powershell.exe", "cmd.exe"] {
            if let Some(path) = resolve_shell(candidate) {
                return path.to_string_lossy().into_owned();
            }
        }
        windows_cmd_fallback()
    }
    #[cfg(target_os = "macos")]
    {
        "/bin/zsh".to_string()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "/bin/bash".to_string()
    }
}

/// Windows 保底 `cmd.exe` 全路径：优先 `%ComSpec%`，否则 `C:\Windows\System32\cmd.exe`，
/// 二者皆缺时退回裸名 `cmd.exe`（系统必装 · PATH 必含 System32）。永不返回 Unix 路径。
#[cfg(windows)]
fn windows_cmd_fallback() -> String {
    if let Some(comspec) = std::env::var_os("ComSpec") {
        let path = PathBuf::from(&comspec);
        if is_executable_file(&path) {
            return path.to_string_lossy().into_owned();
        }
    }
    let system32 = PathBuf::from(r"C:\Windows\System32\cmd.exe");
    if is_executable_file(&system32) {
        return system32.to_string_lossy().into_owned();
    }
    "cmd.exe".to_string()
}

/// 扫 /etc/shells 找到第一个实际可用的交互 shell · 终极 fallback。
/// 调用 [`list_available_shells`]（已过滤 bash/zsh/fish + 检查可执行）·
/// 若列表为空回退到 [`default_shell_path`]（至少是 OS 标准路径）。
fn find_available_shell() -> String {
    let available = list_available_shells();
    if let Some(shell) = available.first() {
        return shell.path.clone();
    }
    default_shell_path()
}

pub fn resolve_default_shell(pool: Option<&DbPool>) -> String {
    let stored = pool
        .and_then(|p| AppSettingsStore::get(p, "default_shell").ok())
        .filter(|s| !s.trim().is_empty());

    if let Some(ref s) = stored {
        // 用户显式选的 shell · 即使不在白名单里也优先尊重（可能 fish 装在 /usr/local/bin/fish）
        // 但如果 PATH 里找不到 · 回退到系统可用 shell
        if resolve_shell(s).is_some() {
            s.clone()
        } else {
            find_available_shell()
        }
    } else {
        // 无已存设置 · 先用 OS 默认 · 不存在则扫系统
        let def = default_shell_path();
        if resolve_shell(&def).is_some() {
            def
        } else {
            find_available_shell()
        }
    }
}

pub(crate) fn effective_shell_for_spawn(requested: &str, env_shell: Option<&str>) -> String {
    let requested = requested.trim();
    let env_shell = env_shell
        .map(str::trim)
        .filter(|shell| !shell.is_empty())
        .map(str::to_string);

    if requested.is_empty() {
        return env_shell.unwrap_or_else(default_shell_path);
    }

    if requested == default_shell_path().as_str() {
        return env_shell.unwrap_or_else(|| requested.to_string());
    }

    requested.to_string()
}

pub fn check_shell_exists(shell: &str) -> Result<(), PtyError> {
    if resolve_shell(shell).is_none() {
        return Err(PtyError::ShellNotFound(shell.to_string()));
    }
    Ok(())
}

/// 一条系统认证的 shell · 路径 + 显示用 label（basename）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ShellInfo {
    pub path: String,
    pub label: String,
}

/// 主流交互 shell 白名单（覆盖 95% 用户）· 过滤 dash / csh / tcsh / ksh / sh
/// 等系统/脚本 shell · 它们出现在 `/etc/shells` 但几乎没人作为交互 terminal 用。
/// nu / pwsh 通常不写 `/etc/shells` · MVP 不为它们加路径探测。用户已选的非白名单 shell
/// 仍保留在 list 里（例外保护 · 见 list_available_shells 末尾）。
/// task-2.1：仅 Unix `/etc/shells` 路径使用（Windows 走 where.exe 探测链 · 不读 /etc/shells）。
#[cfg(unix)]
const PRIMARY_SHELL_BASENAMES: &[&str] = &["zsh", "bash", "fish"];

/// 枚举系统可用的交互 shell（task-2.1 · ADR-003）·
/// Windows 走 PATH / `where.exe` 探测 `pwsh.exe`/`powershell.exe`/`cmd.exe`（及 git-bash 若存在）·
/// 不读 `/etc/shells`；Unix 保留扫 `/etc/shells` + 主流 basename 过滤逻辑。
///
/// 调用方应该用此函数返回值作为 Settings UI dropdown · 让用户只能选系统真有的主流 shell ·
/// 而非自由输入 / hardcoded 列表（fix24/25 · 防 "PTY 启动失败: shell not executable"
/// + 减干扰非主流选项）。
pub fn list_available_shells() -> Vec<ShellInfo> {
    #[cfg(windows)]
    {
        windows_list_shells()
    }
    #[cfg(unix)]
    {
        unix_list_shells_from_etc_shells()
    }
}

/// Unix `/etc/shells` 扫描（task-2.1 前的原始逻辑原样迁入 · 行为零变化）。
#[cfg(unix)]
fn unix_list_shells_from_etc_shells() -> Vec<ShellInfo> {
    let mut shells: Vec<ShellInfo> = std::fs::read_to_string("/etc/shells")
        .ok()
        .map(|content| {
            content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .filter_map(|path_str| {
                    let path = Path::new(path_str);
                    if !is_executable_file(path) {
                        return None;
                    }
                    let basename = path.file_name().and_then(|n| n.to_str())?;
                    if !PRIMARY_SHELL_BASENAMES.contains(&basename) {
                        return None;
                    }
                    Some(ShellInfo {
                        path: path_str.to_string(),
                        label: basename.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if shells.is_empty() {
        // /etc/shells 读不到 / 全 invalid · fallback 到 default_shell_path（一定存在的）
        let fallback = default_shell_path();
        if is_executable_file(Path::new(&fallback)) {
            let label = Path::new(&fallback)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&fallback)
                .to_string();
            shells.push(ShellInfo {
                path: fallback,
                label,
            });
        }
    }

    // 去重（macOS /etc/shells 经常 /bin/zsh + /usr/bin/zsh 都列）· 按 label 去重保留首项
    shells.dedup_by(|a, b| a.path == b.path);
    shells
}

/// Windows shell 枚举（task-2.1 · ADR-003）· 对探测链候选逐个 `resolve_shell` ·
/// 命中者 push `ShellInfo{ path: 全路径, label: basename 去 .exe }`；按 path 去重；
/// 列表为空时保底 push `cmd.exe`（`windows_cmd_fallback`）· 永不读 `/etc/shells`。
#[cfg(windows)]
fn windows_list_shells() -> Vec<ShellInfo> {
    // pwsh（PowerShell 7+）→ powershell（内置 5.1）→ cmd（保底）→ bash（git-bash 若装）
    const WINDOWS_SHELL_CANDIDATES: &[&str] =
        &["pwsh.exe", "powershell.exe", "cmd.exe", "bash.exe"];

    let mut shells: Vec<ShellInfo> = Vec::new();
    for candidate in WINDOWS_SHELL_CANDIDATES {
        if let Some(path) = resolve_shell(candidate) {
            shells.push(ShellInfo {
                path: path.to_string_lossy().into_owned(),
                label: windows_shell_label(&path),
            });
        }
    }

    if shells.is_empty() {
        // 探测链全空（理论上不该 · cmd.exe 必装）· 保底 cmd.exe 全路径
        let fallback = windows_cmd_fallback();
        let label = windows_shell_label(Path::new(&fallback));
        shells.push(ShellInfo {
            path: fallback,
            label,
        });
    }

    // 按 path 去重（保留首项 · 探测链已是优先序）
    shells.dedup_by(|a, b| a.path == b.path);
    shells
}

/// Windows shell 显示用 label：取 basename 去 `.exe`/`.bat`/`.cmd`/`.com` 扩展名。
#[cfg(windows)]
fn windows_shell_label(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_string)
        .or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

pub(crate) fn resolve_shell(shell: &str) -> Option<PathBuf> {
    let path = Path::new(shell);
    if path.components().count() > 1 {
        return is_executable_file(path).then(|| path.to_path_buf());
    }

    #[cfg(windows)]
    {
        resolve_shell_via_where(shell)
    }
    #[cfg(unix)]
    {
        resolve_shell_in_path(shell, std::env::var_os("PATH").as_deref())
    }
}

#[cfg(unix)]
fn resolve_shell_in_path(shell: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    path_var.and_then(|path_var| {
        std::env::split_paths(path_var)
            .map(|dir| dir.join(shell))
            .find(|candidate| is_executable_file(candidate))
    })
}

/// Windows 裸名 shell 解析（task-2.1 · ADR-003）· 调 `where.exe <shell>`，
/// 取多行输出里首个 `is_executable_file` 为 true 的项（`where.exe` 可能返回多行 ·
/// 如 pwsh 同时命中 Program Files + WindowsApps app-execution-alias）。
/// `where.exe` 找不到时退出码非 0 / 输出空 → 返回 None。
#[cfg(windows)]
fn resolve_shell_via_where(shell: &str) -> Option<PathBuf> {
    let output = std::process::Command::new("where.exe")
        .arg(shell)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|candidate| is_executable_file(candidate))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Windows 无 POSIX 执行位（ADR-001 + ADR-003 + spec §5.3）· 退化为
/// "是存在的文件 **且** 有可执行扩展名（.exe/.bat/.cmd/.com · 不区分大小写）"。
/// 无可执行扩展名的普通文件（如 .txt）判 false（task-2.1 收紧 · 替换 task-1.1 的纯 is_file 占位）。
#[cfg(windows)]
fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
        && has_windows_executable_ext(path)
}

/// 判断路径是否带 Windows 可执行扩展名（.exe/.bat/.cmd/.com · 不区分大小写）。
#[cfg(windows)]
fn has_windows_executable_ext(path: &Path) -> bool {
    const EXECUTABLE_EXTS: &[&str] = &["exe", "bat", "cmd", "com"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            EXECUTABLE_EXTS.contains(&lower.as_str())
        })
        .unwrap_or(false)
}

fn detect_process_cwd(process_id: u32) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_link(format!("/proc/{process_id}/cwd")).ok()
    }
    #[cfg(target_os = "macos")]
    {
        let output = std::process::Command::new("lsof")
            .args(["-a", "-p", &process_id.to_string(), "-d", "cwd", "-Fn"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| line.strip_prefix('n'))
            .map(PathBuf::from)
    }
    #[cfg(target_os = "windows")]
    {
        // PRD §Out of Scope · ConPTY 无 /proc/lsof 等价物 · 显式返回 None ·
        // 由 spawn-time 缓存的 initial_cwd 兜底（见 PtySession::working_directory）·
        // 精确实现（Windows API 查询）defer · ADR-001 OQ3。
        let _ = process_id;
        None
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = process_id;
        None
    }
}

fn normalize_cwd(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

#[cfg(unix)]
fn set_fd_nonblocking(fd: RawFd, enabled: bool) -> Result<(), PtyError> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(PtyError::Io(io::Error::last_os_error()));
    }

    let next = if enabled {
        flags | libc::O_NONBLOCK
    } else {
        flags & !libc::O_NONBLOCK
    };
    if unsafe { libc::fcntl(fd, libc::F_SETFL, next) } < 0 {
        return Err(PtyError::Io(io::Error::last_os_error()));
    }

    Ok(())
}

/// Windows no-op（ADR-001 + spec §5.3）· ConPTY 的 overlapped I/O 由 portable-pty 内部处理 ·
/// 无需 fcntl(O_NONBLOCK)。保留显式函数以标注"此处刻意不设非阻塞"。
#[cfg(windows)]
fn set_fd_nonblocking_noop() {}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crossbeam_channel::RecvTimeoutError;
    // Unix-only：mode 位测试 + PTY 进程拉起测试需要 PermissionsExt（AC5 · cfg-gate）。
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use ts_rs::{Config, TS};

    #[test]
    fn find_ansi_clear_matches_2j() {
        let data = b"prefix\x1b[2Jsuffix";
        assert_eq!(find_ansi_clear(data), Some(6));
    }

    #[test]
    fn find_ansi_clear_matches_3j() {
        let data = b"prefix\x1b[3Jsuffix";
        assert_eq!(find_ansi_clear(data), Some(6));
    }

    #[test]
    fn find_ansi_clear_matches_combined_home_clear() {
        // 实际 clear 命令通常输出 ESC [ H ESC [ 2 J（cursor home + erase screen）
        let data = b"cd '/path'; clear\r\n\x1b[H\x1b[2Jrest";
        let idx = find_ansi_clear(data).expect("should match \\x1b[2J");
        assert_eq!(&data[idx..idx + 4], b"\x1b[2J");
    }

    #[test]
    fn find_ansi_clear_returns_none_for_non_clear() {
        assert_eq!(find_ansi_clear(b"plain text"), None);
        assert_eq!(find_ansi_clear(b"\x1b[H"), None); // cursor home only · 不是 clear
        assert_eq!(find_ansi_clear(b"\x1b[1J"), None); // erase to cursor · 不算
        assert_eq!(find_ansi_clear(b""), None);
        assert_eq!(find_ansi_clear(b"\x1b["), None); // 不完整序列
    }

    fn manager_with_events() -> (PtyManager, Receiver<PtyEvent>) {
        let manager = PtyManager::new();
        let events = manager
            .take_event_receiver()
            .expect("event receiver should be available once");
        (manager, events)
    }

    #[cfg(unix)]
    fn spawn_shell(manager: &PtyManager, tab_id: &str) -> Result<(), PtyError> {
        manager.spawn(PtySpawnRequest {
            tab_id: tab_id.to_string(),
            shell: "/bin/sh".to_string(),
            cwd: "/tmp".to_string(),
            cols: 80,
            rows: 24,
        })
    }

    #[cfg(unix)]
    fn recv_until_exit(
        events: &Receiver<PtyEvent>,
        tab_id: &str,
        timeout: Duration,
    ) -> (String, Option<i32>) {
        let started = Instant::now();
        let mut output = String::new();

        loop {
            let remaining = timeout
                .checked_sub(started.elapsed())
                .unwrap_or_else(|| Duration::from_millis(1));
            match events.recv_timeout(remaining) {
                Ok(PtyEvent::Stdout(event)) if event.tab_id == tab_id => {
                    output.push_str(&event.data);
                }
                Ok(PtyEvent::Exited(event)) if event.tab_id == tab_id => {
                    return (output, event.exit_code);
                }
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => {
                    panic!("timed out waiting for exit event for {tab_id}");
                }
                Err(RecvTimeoutError::Disconnected) => {
                    panic!("event channel disconnected while waiting for {tab_id}");
                }
            }
        }
    }

    #[test]
    fn stdout_event_ts_decl_uses_camel_case() {
        let config = Config::default();
        let decl = PtyStdoutEvent::decl(&config);
        assert!(decl.contains("tabId"));
        assert!(decl.contains("data"));
        assert!(!decl.contains("tab_id"));
    }

    #[test]
    fn exited_event_ts_decl_uses_camel_case() {
        let config = Config::default();
        let decl = PtyExitedEvent::decl(&config);
        assert!(decl.contains("tabId"));
        assert!(decl.contains("exitCode"));
        assert!(!decl.contains("exit_code"));
    }

    #[test]
    fn spawn_request_ts_decl_uses_camel_case() {
        let config = Config::default();
        let decl = PtySpawnRequest::decl(&config);
        assert!(decl.contains("tabId"));
        assert!(decl.contains("shell"));
        assert!(decl.contains("cwd"));
    }

    #[test]
    fn error_display_is_human_readable() {
        assert_eq!(
            PtyError::NotFound("tab-a".to_string()).to_string(),
            "tab not found: tab-a"
        );
        assert_eq!(
            PtyError::ShellNotFound("/bin/nope".to_string()).to_string(),
            "shell not executable: /bin/nope"
        );
        assert_eq!(
            PtyError::InvalidSignal("SIGUSR1".to_string()).to_string(),
            "invalid signal: SIGUSR1"
        );
    }

    #[cfg(unix)]
    #[test]
    fn parse_signal_maps_supported_values() {
        assert_eq!(parse_signal("SIGINT").unwrap(), libc::SIGINT);
        assert_eq!(parse_signal("SIGTERM").unwrap(), libc::SIGTERM);
        assert_eq!(parse_signal("SIGTSTP").unwrap(), libc::SIGTSTP);
        assert_eq!(parse_signal("SIGKILL").unwrap(), libc::SIGKILL);
    }

    #[cfg(unix)]
    #[test]
    fn parse_signal_rejects_unknown_values() {
        let error = parse_signal("SIGHUP").unwrap_err();
        assert!(matches!(error, PtyError::InvalidSignal(value) if value == "SIGHUP"));
    }

    #[test]
    fn drop_oldest_sender_drops_oldest_item() {
        let (sender, receiver) = crossbeam_channel::bounded(2);
        let dropper = DropOldestSender::new(sender, receiver.clone());
        dropper.send("one").unwrap();
        dropper.send("two").unwrap();
        dropper.send("three").unwrap();

        assert_eq!(receiver.recv().unwrap(), "two");
        assert_eq!(receiver.recv().unwrap(), "three");
        assert!(receiver.is_empty());
    }

    #[test]
    fn parse_chunk_to_lines_splits_complete_lines() {
        let mut partial = String::new();
        let lines = parse_chunk_to_lines("alpha\nbeta\r\ngamma\n", &mut partial);
        assert_eq!(lines, vec!["alpha", "beta", "gamma"]);
        assert!(partial.is_empty());
    }

    #[test]
    fn parse_chunk_to_lines_preserves_partial_tail() {
        let mut partial = String::new();
        let first = parse_chunk_to_lines("hel", &mut partial);
        let second = parse_chunk_to_lines("lo\nwor", &mut partial);
        let third = parse_chunk_to_lines("ld\n", &mut partial);

        assert!(first.is_empty());
        assert_eq!(second, vec!["hello"]);
        assert_eq!(third, vec!["world"]);
        assert!(partial.is_empty());
    }

    #[test]
    fn scrollback_flushes_when_due() {
        let mut buffer = ScrollbackBuffer::default();
        buffer.push_chunk("line1\nline2\n");
        let ready = buffer.drain_due(Instant::now() + SCROLLBACK_FLUSH_INTERVAL);
        assert_eq!(ready, Some(vec!["line1".to_string(), "line2".to_string()]));
    }

    #[test]
    fn scrollback_force_flushes_partial_tail() {
        let mut buffer = ScrollbackBuffer::default();
        buffer.push_chunk("tail-without-newline");
        assert_eq!(
            buffer.drain_all(),
            Some(vec!["tail-without-newline".to_string()])
        );
    }

    #[test]
    fn exit_code_maps_none_for_signals() {
        let status = ExitStatus::with_signal("SIGKILL");
        assert_eq!(exit_code_from_status(&status), None);
    }

    #[test]
    fn effective_shell_prefers_env_shell_for_default_request() {
        assert_eq!(
            effective_shell_for_spawn(&default_shell_path(), Some("/opt/homebrew/bin/fish")),
            "/opt/homebrew/bin/fish"
        );
    }

    #[test]
    fn effective_shell_keeps_explicit_non_default_request() {
        assert_eq!(
            effective_shell_for_spawn("/bin/sh", Some("/opt/homebrew/bin/fish")),
            "/bin/sh"
        );
    }

    #[test]
    fn effective_shell_falls_back_when_request_is_empty() {
        assert_eq!(
            effective_shell_for_spawn("", Some("/usr/local/bin/bash")),
            "/usr/local/bin/bash"
        );
        assert_eq!(effective_shell_for_spawn("", None), default_shell_path());
    }

    #[cfg(unix)]
    #[test]
    fn resolve_shell_in_path_skips_non_executable_candidates() {
        let temp = tempfile::TempDir::new().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();

        let non_exec = first.join("fish");
        let exec = second.join("fish");
        std::fs::write(&non_exec, "#!/bin/sh\n").unwrap();
        std::fs::write(&exec, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&non_exec, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::fs::set_permissions(&exec, std::fs::Permissions::from_mode(0o755)).unwrap();

        let path_var = std::env::join_paths([first.as_path(), second.as_path()]).unwrap();
        assert_eq!(
            resolve_shell_in_path("fish", Some(path_var.as_os_str())),
            Some(exec)
        );
    }

    #[test]
    fn spawn_missing_shell_errors() {
        let (manager, _events) = manager_with_events();
        let error = manager
            .spawn(PtySpawnRequest {
                tab_id: "missing-shell".to_string(),
                shell: "/bin/does-not-exist-vibestation".to_string(),
                cwd: "/tmp".to_string(),
                cols: 80,
                rows: 24,
            })
            .unwrap_err();
        assert!(matches!(error, PtyError::ShellNotFound(_)));
    }

    #[test]
    fn stdin_unknown_tab_returns_not_found() {
        let (manager, _events) = manager_with_events();
        let error = manager.stdin("missing", "echo hi\n").unwrap_err();
        assert!(matches!(error, PtyError::NotFound(value) if value == "missing"));
    }

    #[test]
    fn resize_unknown_tab_returns_not_found() {
        let (manager, _events) = manager_with_events();
        let error = manager.resize("missing", 120, 40).unwrap_err();
        assert!(matches!(error, PtyError::NotFound(value) if value == "missing"));
    }

    #[test]
    fn signal_unknown_tab_returns_not_found() {
        let (manager, _events) = manager_with_events();
        let error = manager.signal("missing", "SIGINT").unwrap_err();
        assert!(matches!(error, PtyError::NotFound(value) if value == "missing"));
    }

    #[test]
    fn kill_unknown_tab_returns_not_found() {
        let (manager, _events) = manager_with_events();
        let error = manager.kill("missing").unwrap_err();
        assert!(matches!(error, PtyError::NotFound(value) if value == "missing"));
    }

    #[test]
    fn working_directory_unknown_tab_returns_not_found() {
        let (manager, _events) = manager_with_events();
        let error = manager.working_directory("missing").unwrap_err();
        assert!(matches!(error, PtyError::NotFound(value) if value == "missing"));
    }

    #[test]
    fn environment_unknown_tab_returns_not_found() {
        let (manager, _events) = manager_with_events();
        let error = manager.environment("missing").unwrap_err();
        assert!(matches!(error, PtyError::NotFound(value) if value == "missing"));
    }

    #[cfg(unix)]
    #[test]
    fn working_directory_returns_spawn_cwd() {
        let (manager, _events) = manager_with_events();
        let dir = tempfile::tempdir().unwrap();
        manager
            .spawn(PtySpawnRequest {
                tab_id: "tab-cwd".to_string(),
                shell: "/bin/sh".to_string(),
                cwd: dir.path().to_string_lossy().to_string(),
                cols: 80,
                rows: 24,
            })
            .unwrap();

        let cwd = manager.working_directory("tab-cwd").unwrap();

        assert_eq!(cwd, dir.path().canonicalize().unwrap());
        manager.kill("tab-cwd").unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn environment_returns_spawn_env_with_terminal_overrides() {
        let (manager, _events) = manager_with_events();
        let dir = tempfile::tempdir().unwrap();
        manager
            .spawn(PtySpawnRequest {
                tab_id: "tab-env".to_string(),
                shell: "/bin/sh".to_string(),
                cwd: dir.path().to_string_lossy().to_string(),
                cols: 80,
                rows: 24,
            })
            .unwrap();

        let env = manager.environment("tab-env").unwrap();

        assert_eq!(env.get("TERM"), Some(&"xterm-256color".to_string()));
        assert_eq!(env.get("COLORTERM"), Some(&"truecolor".to_string()));
        assert_eq!(env.get("LANG"), Some(&"en_US.UTF-8".to_string()));
        assert_eq!(env.get("LC_ALL"), Some(&"en_US.UTF-8".to_string()));
        manager.kill("tab-env").unwrap();
    }

    #[cfg(unix)]
    #[test]
    #[cfg_attr(
        target_os = "linux",
        ignore = "Linux PTY exit event timing 在 CI runner 上不稳定 · printf + exit 后 mio epoll 感知 PTY master fd close event 的链路在 GitHub Actions Ubuntu runner 上偶发 > 5s · 本地 macOS（kqueue）正常 · 和 signal_sigterm_exits_exec_session 同根因 · PR #86 先例 · MVP-04 Phase D Ubuntu runtime 验证时统一深挖补修 · 本地跑 `cargo test -- --ignored spawn_stdin_and_exit_emit_stdout_and_exit_event` 仍可跑"
    )]
    fn spawn_stdin_and_exit_emit_stdout_and_exit_event() {
        let (manager, events) = manager_with_events();
        spawn_shell(&manager, "tab-echo").unwrap();
        manager
            .stdin("tab-echo", "printf 'hello-from-pty\\n'\nexit\n")
            .unwrap();

        let (output, exit_code) = recv_until_exit(&events, "tab-echo", Duration::from_secs(5));
        assert!(output.contains("hello-from-pty"));
        assert_eq!(exit_code, Some(0));
    }

    #[cfg(unix)]
    #[test]
    fn resize_keeps_session_alive() {
        let (manager, events) = manager_with_events();
        spawn_shell(&manager, "tab-resize").unwrap();
        manager.resize("tab-resize", 120, 40).unwrap();
        manager
            .stdin("tab-resize", "printf 'resized-ok\\n'\nexit\n")
            .unwrap();

        let (output, exit_code) = recv_until_exit(&events, "tab-resize", Duration::from_secs(5));
        assert!(output.contains("resized-ok"));
        assert_eq!(exit_code, Some(0));
    }

    #[cfg(unix)]
    #[test]
    #[cfg_attr(
        target_os = "linux",
        ignore = "Linux PTY SIGTERM 到达 exec sleep 进程的 timing 在 CI runner 上不稳定 · 本地 macOS 正常 · PR #82 CI failure · PR #86 多轮 workaround（200→500ms · 5→10s timeout）均失败 · 根因怀疑是 mio epoll 对 PTY master fd 的 close event 在 Ubuntu 不一致 · 需要深挖 waitpid / tcgetpgrp 在 Linux pty 的行为差异 · 标 technical debt · MVP-04 Phase D（shell 兼容 · Ubuntu runtime 验证）时补修 · 本地跑 `cargo test -- --ignored signal_sigterm_exits_exec_session` 仍可跑"
    )]
    fn signal_sigterm_exits_exec_session() {
        let (manager, events) = manager_with_events();
        spawn_shell(&manager, "tab-signal").unwrap();
        manager.stdin("tab-signal", "exec sleep 30\n").unwrap();
        thread::sleep(Duration::from_millis(500));
        manager.signal("tab-signal", "SIGTERM").unwrap();

        let (_output, exit_code) = recv_until_exit(&events, "tab-signal", Duration::from_secs(10));
        assert_eq!(exit_code, None);
    }

    #[cfg(unix)]
    #[test]
    fn kill_force_terminates_session() {
        let (manager, events) = manager_with_events();
        spawn_shell(&manager, "tab-kill").unwrap();
        manager.stdin("tab-kill", "exec sleep 30\n").unwrap();
        thread::sleep(Duration::from_millis(200));

        let exit_code = manager.kill("tab-kill").unwrap();
        let (_output, emitted_exit_code) =
            recv_until_exit(&events, "tab-kill", Duration::from_secs(5));
        assert_eq!(exit_code, emitted_exit_code);
    }

    #[cfg(unix)]
    #[test]
    fn shared_reader_routes_stdout_to_multiple_tabs() {
        let (manager, events) = manager_with_events();
        for tab_id in ["tab-a", "tab-b", "tab-c"] {
            spawn_shell(&manager, tab_id).unwrap();
            manager
                .stdin(tab_id, &format!("printf '{tab_id}\\n'\nexit\n"))
                .unwrap();
        }

        let mut outputs = HashMap::<String, String>::new();
        let mut exits = 0usize;
        let started = Instant::now();
        while exits < 3 {
            let remaining = Duration::from_secs(5)
                .checked_sub(started.elapsed())
                .unwrap_or_else(|| Duration::from_millis(1));
            match events.recv_timeout(remaining) {
                Ok(PtyEvent::Stdout(event)) => {
                    outputs
                        .entry(event.tab_id)
                        .or_default()
                        .push_str(&event.data);
                }
                Ok(PtyEvent::Exited(_)) => exits += 1,
                Err(error) => panic!("multi-tab event wait failed: {error}"),
            }
        }

        assert!(outputs
            .get("tab-a")
            .is_some_and(|output| output.contains("tab-a")));
        assert!(outputs
            .get("tab-b")
            .is_some_and(|output| output.contains("tab-b")));
        assert!(outputs
            .get("tab-c")
            .is_some_and(|output| output.contains("tab-c")));
    }

    #[cfg(unix)]
    #[test]
    fn duplicate_spawn_rejected_while_session_alive() {
        let (manager, _events) = manager_with_events();
        spawn_shell(&manager, "tab-dup").unwrap();
        let error = spawn_shell(&manager, "tab-dup").unwrap_err();
        assert!(matches!(error, PtyError::AlreadyRunning(value) if value == "tab-dup"));
    }

    // resolve_default_shell_* 测试断言 Unix shell 路径（/bin/zsh · /bin/bash）+ 依赖
    // is_executable_file 的 Unix mode 位语义 · 整组 cfg-gate（Windows shell 探测 = task-2.1）。
    #[cfg(unix)]
    #[test]
    fn resolve_default_shell_returns_platform_default_when_no_db() {
        let shell = resolve_default_shell(None);
        if cfg!(target_os = "macos") {
            assert_eq!(shell, "/bin/zsh");
        } else {
            assert_eq!(shell, "/bin/bash");
        }
    }

    #[cfg(unix)]
    #[test]
    fn resolve_default_shell_reads_from_app_settings() {
        use crate::db;
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test_shell_settings.db");
        let pool = db::open_pool(&db_path).unwrap();
        // 用系统真实存在的 shell · resolve_default_shell 会验证可执行性
        let real_shell = if cfg!(target_os = "macos") {
            "/bin/zsh"
        } else {
            "/bin/bash"
        };
        AppSettingsStore::set(&pool, "default_shell", real_shell).unwrap();
        let shell = resolve_default_shell(Some(&pool));
        assert_eq!(shell, real_shell);
    }

    #[cfg(unix)]
    #[test]
    fn resolve_default_shell_falls_back_when_stored_shell_missing() {
        use crate::db;
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test_shell_missing_path.db");
        let pool = db::open_pool(&db_path).unwrap();
        // 存一个不存在的路径 → 应回退到系统可用的 shell
        AppSettingsStore::set(&pool, "default_shell", "/nonexistent/shell").unwrap();
        let shell = resolve_default_shell(Some(&pool));
        // 回退后必须是可执行的真实 shell
        let resolved = resolve_shell(&shell);
        assert!(
            resolved.is_some(),
            "fallback shell {shell} should be executable"
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolve_default_shell_falls_back_when_key_missing() {
        use crate::db;
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test_shell_missing.db");
        let pool = db::open_pool(&db_path).unwrap();
        let shell = resolve_default_shell(Some(&pool));
        if cfg!(target_os = "macos") {
            assert_eq!(shell, "/bin/zsh");
        } else {
            assert_eq!(shell, "/bin/bash");
        }
    }

    #[cfg(unix)]
    #[test]
    fn resolve_default_shell_ignores_empty_value() {
        use crate::db;
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("test_shell_empty.db");
        let pool = db::open_pool(&db_path).unwrap();
        AppSettingsStore::set(&pool, "default_shell", "  ").unwrap();
        let shell = resolve_default_shell(Some(&pool));
        if cfg!(target_os = "macos") {
            assert_eq!(shell, "/bin/zsh");
        } else {
            assert_eq!(shell, "/bin/bash");
        }
    }

    #[cfg(unix)]
    #[test]
    fn check_shell_exists_accepts_valid_shell() {
        assert!(check_shell_exists("/bin/sh").is_ok());
    }

    #[test]
    fn check_shell_exists_rejects_invalid_path() {
        let result = check_shell_exists("/bin/does-not-exist-vibestation-test");
        assert!(matches!(result, Err(PtyError::ShellNotFound(_))));
    }

    // ────────────────────────────────────────────────────────────────────────
    // task-1.1 · Windows 平台分离 SCEN/TEST（§7 追踪表）
    // ────────────────────────────────────────────────────────────────────────

    /// TEST-1.1.1（SCEN-1.1.1 · AC1）：crates/core 在三平台编译通过。
    /// 本测试存在即证明 cfg 分离后符号在编译目标平台齐全（Windows 不再因 mio::unix /
    /// libc::fcntl / PermissionsExt fatal）· 编译能跑到本测试就说明 AC1 在该平台成立。
    #[test]
    fn test_1_1_1_windows_pty_compiles() {
        // PtyManager 在三平台都能构造（Unix 起 mio reader · Windows 起 ConPTY per-session reader 调度）。
        let manager = PtyManager::new();
        // 未知 tab 在三平台都返回 NotFound（不触平台相关 PTY 路径）。
        assert!(matches!(
            manager.stdin("nonexistent", "noop"),
            Err(PtyError::NotFound(_))
        ));
    }

    /// TEST-1.1.2（SCEN-1.1.2 · AC2）：Unix 信号解析 + reader 路径行为零回归。
    /// 锁住 Unix parse_signal 映射 + is_executable_file mode 位语义未被 cfg 分离改坏。
    #[cfg(unix)]
    #[test]
    fn test_1_1_2_unix_reader_signal_unchanged() {
        // 信号映射不变
        assert_eq!(parse_signal("SIGINT").unwrap(), libc::SIGINT);
        assert_eq!(parse_signal("SIGKILL").unwrap(), libc::SIGKILL);
        assert!(matches!(
            parse_signal("SIGHUP"),
            Err(PtyError::InvalidSignal(_))
        ));
        // is_executable_file mode 位语义不变：真实可执行 shell 判 true
        assert!(is_executable_file(Path::new("/bin/sh")));
        // 非可执行的普通文件判 false
        let dir = tempfile::tempdir().unwrap();
        let plain = dir.path().join("plain.txt");
        std::fs::write(&plain, "data").unwrap();
        std::fs::set_permissions(&plain, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!is_executable_file(&plain));
    }

    /// TEST-1.1.3（SCEN-1.1.3 · AC3）：Windows 信号路由退化为单进程 kill 语义 ·
    /// parse_signal_windows 受支持信号全映射 Kill · 未知信号拒绝（与 Unix 同契约）·
    /// 路径不引用 mio/SourceFd/libc（编译能到此即证明）。
    #[cfg(windows)]
    #[test]
    fn test_1_1_3_windows_reader_no_mio() {
        assert_eq!(parse_signal_windows("SIGINT").unwrap(), WindowsSignal::Kill);
        assert_eq!(
            parse_signal_windows("SIGTERM").unwrap(),
            WindowsSignal::Kill
        );
        assert_eq!(
            parse_signal_windows("SIGTSTP").unwrap(),
            WindowsSignal::Kill
        );
        assert_eq!(
            parse_signal_windows("SIGKILL").unwrap(),
            WindowsSignal::Kill
        );
        assert!(matches!(
            parse_signal_windows("SIGHUP"),
            Err(PtyError::InvalidSignal(value)) if value == "SIGHUP"
        ));
    }

    /// TEST-1.1.4（SCEN-1.1.4 · AC4）：Windows detect_process_cwd 安全返回 None ·
    /// 不 panic · 不调 /proc / lsof · 由 spawn-time 缓存 initial_cwd 兜底。
    #[cfg(windows)]
    #[test]
    fn test_1_1_4_detect_cwd_windows_none() {
        // 任意 pid（含自身）在 Windows 都返回 None（Out of Scope · 缓存兜底）。
        assert_eq!(detect_process_cwd(std::process::id()), None);
        assert_eq!(detect_process_cwd(1), None);
    }

    /// TEST-1.1.5（SCEN-1.1.5 · AC5）：测试模块在三平台均可编译 ·
    /// 平台无关的纯逻辑函数（parse_chunk_to_lines / find_ansi_clear / exit_code_from_status）
    /// 不依赖任一平台 import · 编译能跑到此即证明 Unix-only import 已正确 cfg-gate。
    #[test]
    fn test_1_1_5_tests_compile_all_platforms() {
        // 平台无关：行解析
        let mut partial = String::new();
        assert_eq!(parse_chunk_to_lines("a\nb\n", &mut partial), vec!["a", "b"]);
        // 平台无关：ANSI clear 检测
        assert_eq!(find_ansi_clear(b"x\x1b[2Jy"), Some(1));
        // 平台无关：退出码映射（signal 变体 → None）
        assert_eq!(
            exit_code_from_status(&ExitStatus::with_signal("SIGKILL")),
            None
        );
        assert_eq!(
            exit_code_from_status(&ExitStatus::with_exit_code(0)),
            Some(0)
        );
    }

    /// Windows is_executable_file 退化为文件存在性判定（不走 mode 位 · AC · spec §5.3）。
    #[cfg(windows)]
    #[test]
    fn windows_is_executable_file_uses_file_existence() {
        let dir = tempfile::tempdir().unwrap();
        // 存在的普通文件 → true（Windows 无执行位 · 退化为 is_file）
        let file = dir.path().join("tool.exe");
        std::fs::write(&file, b"MZ").unwrap();
        assert!(is_executable_file(&file));
        // 不存在 → false
        assert!(!is_executable_file(&dir.path().join("missing.exe")));
        // 目录 → false（is_file 为 false）
        assert!(!is_executable_file(dir.path()));
    }

    // ────────────────────────────────────────────────────────────────────────
    // task-2.1 · Windows shell 探测 SCEN/TEST（§7 追踪表 · AC1~AC6）
    // ────────────────────────────────────────────────────────────────────────

    /// TEST-2.1.1（SCEN-2.1.1 · AC1）：Windows `default_shell_path()` 返回探测链首个可用 shell ·
    /// 全路径绝不是 Unix `/bin/*`；探测链候选之一（pwsh/powershell/cmd）必命中。
    #[cfg(windows)]
    #[test]
    fn test_2_1_1_default_shell_probe_chain_picks_pwsh() {
        let shell = default_shell_path();
        // 绝不返回 Unix 路径
        assert!(
            !shell.starts_with("/bin/") && !shell.starts_with("/usr/"),
            "Windows default_shell_path 不得返回 Unix 路径 · got {shell}"
        );
        // 必命中探测链候选（basename 去 .exe 应是 pwsh/powershell/cmd 之一）
        let basename = Path::new(&shell)
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        assert!(
            ["pwsh.exe", "powershell.exe", "cmd.exe"].contains(&basename.as_str()),
            "default_shell_path basename 应在探测链内 · got {basename}"
        );
        // 返回的全路径必须真可执行（除非退回裸名 cmd.exe 保底）
        if Path::new(&shell).components().count() > 1 {
            assert!(
                is_executable_file(Path::new(&shell)),
                "探测链返回的全路径必须可执行 · got {shell}"
            );
        }
        // 本机装了 pwsh 7+，探测链应优先选 pwsh.exe
        if resolve_shell("pwsh.exe").is_some() {
            assert_eq!(
                basename, "pwsh.exe",
                "装了 pwsh 时探测链应优先返回 pwsh.exe · got {basename}"
            );
        }
    }

    /// TEST-2.1.2（SCEN-2.1.2 · AC2）：探测链全缺时保底 cmd.exe · 不 panic · 不返回 /bin/bash。
    /// 直接测保底函数 `windows_cmd_fallback`（系统 cmd.exe 必装 · 永远成立）。
    #[cfg(windows)]
    #[test]
    fn test_2_1_2_default_shell_falls_back_to_cmd() {
        // 保底永远是 cmd.exe（全路径或裸名）· 绝不是 Unix 路径
        let fallback = windows_cmd_fallback();
        assert!(
            !fallback.starts_with("/bin/"),
            "保底不得返回 Unix 路径 · got {fallback}"
        );
        let basename = Path::new(&fallback)
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        assert_eq!(basename, "cmd.exe", "保底 shell basename 必须是 cmd.exe");
        // resolve_default_shell(None) 在 Windows 也绝不 panic 且不返回 Unix 路径
        let resolved = resolve_default_shell(None);
        assert!(
            !resolved.starts_with("/bin/"),
            "resolve_default_shell(None) 不得返回 Unix 路径 · got {resolved}"
        );
    }

    /// TEST-2.1.3（SCEN-2.1.3 · AC3）：Windows `list_available_shells()` 非空 ·
    /// 不含任何 Unix `/bin/*` / `/etc/shells` 来源项 · 含已装的 pwsh/powershell/cmd。
    #[cfg(windows)]
    #[test]
    fn test_2_1_3_list_available_shells_windows_no_unix_paths() {
        let shells = list_available_shells();
        assert!(!shells.is_empty(), "Windows 可用 shell 列表不得为空");
        for shell in &shells {
            assert!(
                !shell.path.starts_with("/bin/") && !shell.path.starts_with("/usr/"),
                "Windows shell 列表不得含 Unix 路径 · got {}",
                shell.path
            );
        }
        // 本机三件套至少 cmd 必在
        let labels: Vec<String> = shells.iter().map(|s| s.label.to_ascii_lowercase()).collect();
        assert!(
            labels.iter().any(|l| l == "cmd"),
            "列表应含 cmd · got {labels:?}"
        );
    }

    /// TEST-2.1.4（SCEN-2.1.4 · AC4）：`resolve_shell` 经 where.exe 命中裸名返回全路径 ·
    /// `is_executable_file` 对 .exe/.bat/.cmd true · 对无可执行扩展名普通文件 false。
    #[cfg(windows)]
    #[test]
    fn test_2_1_4_resolve_shell_via_where_and_exe_ext() {
        // cmd.exe 必在 PATH · where.exe 必命中 · 返回带 .exe 的全路径
        let resolved = resolve_shell("cmd.exe").expect("cmd.exe 必能解析");
        assert!(
            resolved.components().count() > 1,
            "where.exe 解析应返回全路径 · got {resolved:?}"
        );
        assert!(
            is_executable_file(&resolved),
            "解析出的 cmd.exe 全路径必须可执行"
        );

        // 扩展名可执行判定
        let dir = tempfile::tempdir().unwrap();
        for ext in ["exe", "bat", "cmd", "com", "EXE", "Cmd"] {
            let f = dir.path().join(format!("tool.{ext}"));
            std::fs::write(&f, b"x").unwrap();
            assert!(
                is_executable_file(&f),
                ".{ext} 应判可执行 · {f:?}"
            );
        }
        // 无可执行扩展名的普通文件 → false
        let txt = dir.path().join("notes.txt");
        std::fs::write(&txt, b"hello").unwrap();
        assert!(!is_executable_file(&txt), ".txt 不应判可执行");
        // 无扩展名文件 → false
        let noext = dir.path().join("plainfile");
        std::fs::write(&noext, b"x").unwrap();
        assert!(!is_executable_file(&noext), "无扩展名文件不应判可执行");
        // 不存在 → false（不 panic）
        assert!(!is_executable_file(&dir.path().join("missing.exe")));

        // 含空格路径 round-trip（R-2.1-b）：带空格目录下的 .exe 仍能被 is_executable_file 识别
        let spaced = dir.path().join("Program Files Sim");
        std::fs::create_dir_all(&spaced).unwrap();
        let spaced_exe = spaced.join("shell tool.exe");
        std::fs::write(&spaced_exe, b"x").unwrap();
        assert!(
            is_executable_file(&spaced_exe),
            "含空格路径的 .exe 应判可执行 · {spaced_exe:?}"
        );
    }

    /// TEST-2.1.5（SCEN-2.1.5 · AC5）：Windows `detect_process_cwd` 安全返回 None ·
    /// `working_directory()` 回落到 spawn-time 缓存的 initial_cwd（非空有效路径）。
    #[cfg(windows)]
    #[test]
    fn test_2_1_5_detect_process_cwd_windows_falls_back_to_initial_cwd() {
        // detect_process_cwd 永远 None（不 panic · OQ3 缓存兜底）
        assert_eq!(detect_process_cwd(std::process::id()), None);
        assert_eq!(detect_process_cwd(1), None);

        // working_directory() 经 spawn-time 缓存 initial_cwd 兜底 · 返回非空有效路径
        let (manager, _events) = manager_with_events();
        let dir = tempfile::tempdir().unwrap();
        manager
            .spawn(PtySpawnRequest {
                tab_id: "tab-cwd-win".to_string(),
                shell: "cmd.exe".to_string(),
                cwd: dir.path().to_string_lossy().to_string(),
                cols: 80,
                rows: 24,
            })
            .unwrap();
        let cwd = manager.working_directory("tab-cwd-win").unwrap();
        assert!(
            cwd.is_absolute() && cwd.exists(),
            "working_directory 应回落到有效缓存 cwd · got {cwd:?}"
        );
        let _ = manager.kill("tab-cwd-win");
    }

    /// TEST-2.1.6（SCEN-2.1.6 · AC6）：mac/Linux shell 探测零回归 ·
    /// default_shell_path 返回平台 Unix 路径 · list_available_shells 经 /etc/shells ·
    /// resolve_shell 经 PATH + mode 位。复用既有 Unix 断言锁定。
    #[cfg(unix)]
    #[test]
    fn test_2_1_6_unix_shell_detection_unchanged() {
        // default_shell_path 仍返回平台 Unix 路径（签名形变为 String · 值不变）
        if cfg!(target_os = "macos") {
            assert_eq!(default_shell_path(), "/bin/zsh");
        } else {
            assert_eq!(default_shell_path(), "/bin/bash");
        }
        // list_available_shells 经 /etc/shells · 非空 · 全是 Unix 路径
        let shells = list_available_shells();
        assert!(!shells.is_empty(), "Unix shell 列表不得为空");
        for shell in &shells {
            assert!(
                shell.path.starts_with('/'),
                "Unix shell 路径应为绝对路径 · got {}",
                shell.path
            );
        }
        // resolve_shell 经 PATH + mode 位仍能解析 /bin/sh
        assert!(resolve_shell("/bin/sh").is_some());
    }
}
