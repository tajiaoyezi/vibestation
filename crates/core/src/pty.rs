//! PTY runtime · portable-pty + shared reader + bounded queue + drop-oldest.
//!
//! 架构依据：SPIKE-05 / SPIKE-05.5 + ADR-003 accepted。
//! 这里保留单 shared-reader + mio poll，避免回落到 per-session reader thread。

use crossbeam_channel::{self, Receiver, Sender, TryRecvError, TrySendError};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::os::fd::RawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};
use ts_rs::TS;

pub const PTY_EVENT_QUEUE_CAPACITY: usize = 128;

const PTY_CONTROL_QUEUE_CAPACITY: usize = 256;
const READ_BUFFER_SIZE: usize = 8192;
const READ_POLL_TIMEOUT: Duration = Duration::from_millis(50);
const IDLE_SLEEP: Duration = Duration::from_millis(25);
const EXIT_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const EXIT_WAIT_STEP: Duration = Duration::from_millis(20);

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
        fd: RawFd,
    },
    Shutdown,
}

enum ReadOutcome {
    Continue,
    Closed,
}

enum SignalTarget {
    ProcessGroup(libc::pid_t),
    Process(libc::pid_t),
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

struct PtySession {
    tab_id: String,
    fd: RawFd,
    process_id: Option<u32>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    closed: AtomicBool,
    exit_emitted: AtomicBool,
}

impl PtySession {
    fn write_input(&self, input: &str) -> Result<(), PtyError> {
        let mut writer = lock(&self.writer);
        writer.write_all(input.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let master = lock(&self.master);
        master
            .resize(PtySize {
                rows: rows.max(1),
                cols: cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| PtyError::OpenFailed(error.to_string()))
    }

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

    fn terminate(&self, events: &DropOldestSender<PtyEvent>) -> Result<Option<i32>, PtyError> {
        self.closed.store(true, Ordering::Relaxed);

        let _ = self.signal("SIGKILL");
        {
            let mut child = lock(&self.child);
            let _ = child.kill();
        }

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

    fn emit_stdout(
        &self,
        events: &DropOldestSender<PtyEvent>,
        bytes: &[u8],
    ) -> Result<(), PtyError> {
        if bytes.is_empty() || self.closed.load(Ordering::Relaxed) {
            return Ok(());
        }

        events.send(PtyEvent::Stdout(PtyStdoutEvent {
            tab_id: self.tab_id.clone(),
            data: String::from_utf8_lossy(bytes).to_string(),
        }))
    }

    fn emit_exit_once(
        &self,
        events: &DropOldestSender<PtyEvent>,
        status: Option<ExitStatus>,
    ) -> Result<Option<i32>, PtyError> {
        self.closed.store(true, Ordering::Relaxed);

        let exit_code = status.as_ref().and_then(exit_code_from_status);
        if self.exit_emitted.swap(true, Ordering::Relaxed) {
            return Ok(exit_code);
        }

        events.send(PtyEvent::Exited(PtyExitedEvent {
            tab_id: self.tab_id.clone(),
            exit_code,
        }))?;
        Ok(exit_code)
    }

    fn signal_target(&self) -> Option<SignalTarget> {
        let foreground = unsafe { libc::tcgetpgrp(self.fd) };
        if foreground > 0 {
            return Some(SignalTarget::ProcessGroup(foreground));
        }

        let leader = lock(&self.master).process_group_leader();
        if let Some(group) = leader.filter(|group| *group > 0) {
            return Some(SignalTarget::ProcessGroup(group));
        }

        self.process_id
            .map(|pid| SignalTarget::Process(pid as libc::pid_t))
    }
}

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    control_tx: Sender<ReaderCommand>,
    reader_thread: Mutex<Option<thread::JoinHandle<()>>>,
    reader_alive: Arc<AtomicBool>,
    next_token: AtomicUsize,
    event_tx: DropOldestSender<PtyEvent>,
    event_rx: Mutex<Option<Receiver<PtyEvent>>>,
}

impl PtyManager {
    #[must_use]
    pub fn new() -> Self {
        let (event_tx, event_rx) = crossbeam_channel::bounded(PTY_EVENT_QUEUE_CAPACITY);
        let event_dispatch = DropOldestSender::new(event_tx, event_rx.clone());
        let (control_tx, control_rx) = crossbeam_channel::bounded(PTY_CONTROL_QUEUE_CAPACITY);
        let sessions = Arc::new(Mutex::new(HashMap::new()));
        let reader_alive = Arc::new(AtomicBool::new(true));

        let reader_sessions = Arc::clone(&sessions);
        let reader_events = event_dispatch.clone();
        let reader_flag = Arc::clone(&reader_alive);
        let reader_thread = thread::Builder::new()
            .name("vibestation-pty-reader".to_string())
            .spawn(move || reader_loop(control_rx, reader_sessions, reader_events, reader_flag))
            .expect("spawn vibestation-pty-reader");

        Self {
            sessions,
            control_tx,
            reader_thread: Mutex::new(Some(reader_thread)),
            reader_alive,
            next_token: AtomicUsize::new(1),
            event_tx: event_dispatch,
            event_rx: Mutex::new(Some(event_rx)),
        }
    }

    pub fn take_event_receiver(&self) -> Option<Receiver<PtyEvent>> {
        lock(&self.event_rx).take()
    }

    pub fn spawn(&self, req: PtySpawnRequest) -> Result<(), PtyError> {
        if lock(&self.sessions).contains_key(&req.tab_id) {
            return Err(PtyError::AlreadyRunning(req.tab_id));
        }

        let resolved_shell =
            resolve_shell(&req.shell).ok_or_else(|| PtyError::ShellNotFound(req.shell.clone()))?;
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: req.rows.max(1),
                cols: req.cols.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| PtyError::OpenFailed(error.to_string()))?;

        let fd = pair
            .master
            .as_raw_fd()
            .ok_or_else(|| PtyError::OpenFailed("portable-pty master has no raw fd".to_string()))?;
        set_fd_nonblocking(fd, true)?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|error| PtyError::OpenFailed(error.to_string()))?;

        let mut command = CommandBuilder::new(resolved_shell.as_os_str());
        command.cwd(&req.cwd);
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
            tab_id: req.tab_id.clone(),
            fd,
            process_id,
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
            closed: AtomicBool::new(false),
            exit_emitted: AtomicBool::new(false),
        });

        lock(&self.sessions).insert(req.tab_id.clone(), Arc::clone(&session));

        let token = self.next_token.fetch_add(1, Ordering::Relaxed);
        if let Err(error) = self.send_control(ReaderCommand::Register { token, session }) {
            lock(&self.sessions).remove(&req.tab_id);
            return Err(error);
        }

        Ok(())
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

        let _ = self.send_control(ReaderCommand::Unregister {
            token: 0,
            fd: session.fd,
        });
        session.terminate(&self.event_tx)
    }

    pub fn close_all_sessions(&self) {
        let tab_ids: Vec<String> = lock(&self.sessions).keys().cloned().collect();
        for tab_id in tab_ids {
            let _ = self.kill(&tab_id);
        }
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
    }
}

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
                        lock(&sessions_by_id).remove(&session.tab_id);
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
                lock(&sessions_by_id).remove(&session.tab_id);
            }
        }
    }
}

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
            if let Err(error) = session.emit_stdout(events, &buffer[..read as usize]) {
                eprintln!(
                    "[mvp-04] stdout emit failed for {}: {error}",
                    session.tab_id
                );
            }
            continue;
        }

        if read == 0 {
            let status = match session.wait_for_exit(EXIT_WAIT_TIMEOUT) {
                Ok(status) => status,
                Err(error) => {
                    eprintln!("[mvp-04] wait failed for {}: {error}", session.tab_id);
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
                        eprintln!("[mvp-04] wait failed for {}: {wait_error}", session.tab_id);
                        None
                    }
                };
                let _ = session.emit_exit_once(events, status);
                return ReadOutcome::Closed;
            }
            _ => {
                eprintln!("[mvp-04] read failed for {}: {error}", session.tab_id);
                let _ = session.emit_exit_once(events, None);
                return ReadOutcome::Closed;
            }
        }
    }
}

fn parse_signal(signal: &str) -> Result<i32, PtyError> {
    match signal {
        "SIGINT" => Ok(libc::SIGINT),
        "SIGTERM" => Ok(libc::SIGTERM),
        "SIGTSTP" => Ok(libc::SIGTSTP),
        "SIGKILL" => Ok(libc::SIGKILL),
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

fn resolve_shell(shell: &str) -> Option<PathBuf> {
    let path = Path::new(shell);
    if path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }

    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|dir| dir.join(shell))
            .find(|candidate| candidate.is_file())
    })
}

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

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::RecvTimeoutError;
    use ts_rs::{Config, TS};

    fn manager_with_events() -> (PtyManager, Receiver<PtyEvent>) {
        let manager = PtyManager::new();
        let events = manager
            .take_event_receiver()
            .expect("event receiver should be available once");
        (manager, events)
    }

    fn spawn_shell(manager: &PtyManager, tab_id: &str) -> Result<(), PtyError> {
        manager.spawn(PtySpawnRequest {
            tab_id: tab_id.to_string(),
            shell: "/bin/sh".to_string(),
            cwd: "/tmp".to_string(),
            cols: 80,
            rows: 24,
        })
    }

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

    #[test]
    fn parse_signal_maps_supported_values() {
        assert_eq!(parse_signal("SIGINT").unwrap(), libc::SIGINT);
        assert_eq!(parse_signal("SIGTERM").unwrap(), libc::SIGTERM);
        assert_eq!(parse_signal("SIGTSTP").unwrap(), libc::SIGTSTP);
        assert_eq!(parse_signal("SIGKILL").unwrap(), libc::SIGKILL);
    }

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
    fn exit_code_maps_none_for_signals() {
        let status = ExitStatus::with_signal("SIGKILL");
        assert_eq!(exit_code_from_status(&status), None);
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

    #[test]
    fn signal_sigterm_exits_exec_session() {
        let (manager, events) = manager_with_events();
        spawn_shell(&manager, "tab-signal").unwrap();
        manager.stdin("tab-signal", "exec sleep 30\n").unwrap();
        thread::sleep(Duration::from_millis(200));
        manager.signal("tab-signal", "SIGTERM").unwrap();

        let (_output, exit_code) = recv_until_exit(&events, "tab-signal", Duration::from_secs(5));
        assert_eq!(exit_code, None);
    }

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

    #[test]
    fn duplicate_spawn_rejected_while_session_alive() {
        let (manager, _events) = manager_with_events();
        spawn_shell(&manager, "tab-dup").unwrap();
        let error = spawn_shell(&manager, "tab-dup").unwrap_err();
        assert!(matches!(error, PtyError::AlreadyRunning(value) if value == "tab-dup"));
    }
}
