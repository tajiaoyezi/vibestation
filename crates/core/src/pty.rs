//! PTY runtime · portable-pty + shared reader + bounded queue + drop-oldest.
//!
//! 架构依据：SPIKE-05 / SPIKE-05.5 + ADR-003 accepted。
//! 这里保留单 shared-reader + mio poll，避免回落到 per-session reader thread。

use crate::app_settings::AppSettingsStore;
use crate::db::DbPool;
use crate::tabs::TabsDao;
use crossbeam_channel::{self, Receiver, Sender, TryRecvError, TrySendError};
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, MasterPty, PtySize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::os::fd::RawFd;
use std::os::unix::fs::PermissionsExt;
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
const SCROLLBACK_FLUSH_INTERVAL: Duration = Duration::from_millis(100);
const SCROLLBACK_FLUSH_THRESHOLD: usize = 100;

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

pub struct PtySession {
    tab_id: Mutex<String>,
    fd: RawFd,
    process_id: Option<u32>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    child: Mutex<Box<dyn Child + Send + Sync>>,
    closed: AtomicBool,
    exit_emitted: AtomicBool,
    scrollback: Mutex<ScrollbackBuffer>,
    scrollback_tx: Sender<ScrollbackCommand>,
}

impl PtySession {
    pub fn set_tab_id(&self, new_id: String) {
        *lock(&self.tab_id) = new_id;
    }

    #[must_use]
    pub fn tab_id_clone(&self) -> String {
        lock(&self.tab_id).clone()
    }

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

    fn emit_stdout(&self, events: &DropOldestSender<PtyEvent>, data: &str) -> Result<(), PtyError> {
        if data.is_empty() || self.closed.load(Ordering::Relaxed) {
            return Ok(());
        }

        events.send(PtyEvent::Stdout(PtyStdoutEvent {
            tab_id: self.tab_id_clone(),
            data: data.to_string(),
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
            fd,
            process_id,
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            child: Mutex::new(child),
            closed: AtomicBool::new(false),
            exit_emitted: AtomicBool::new(false),
            scrollback: Mutex::new(ScrollbackBuffer::default()),
            scrollback_tx: self.scrollback_tx.clone(),
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
        let _ = self.send_control(ReaderCommand::Unregister {
            token: 0,
            fd: session.fd,
        });
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
            let chunk = String::from_utf8_lossy(&buffer[..read as usize]).to_string();
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

fn default_shell_path() -> &'static str {
    if cfg!(target_os = "macos") {
        "/bin/zsh"
    } else {
        "/bin/bash"
    }
}

/// 扫 /etc/shells 找到第一个实际可用的交互 shell · 终极 fallback。
/// 调用 [`list_available_shells`]（已过滤 bash/zsh/fish + 检查可执行）·
/// 若列表为空回退到 [`default_shell_path`]（至少是 OS 标准路径）。
fn find_available_shell() -> String {
    let available = list_available_shells();
    if let Some(shell) = available.first() {
        return shell.path.clone();
    }
    default_shell_path().to_string()
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
        let def = default_shell_path().to_string();
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
        return env_shell.unwrap_or_else(|| default_shell_path().to_string());
    }

    if requested == default_shell_path() {
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
const PRIMARY_SHELL_BASENAMES: &[&str] = &["zsh", "bash", "fish"];

/// 扫 `/etc/shells` · 过滤 commented / 不可执行 / 非主流交互 shell · 返回主流 shell 列表。
/// 读不到 `/etc/shells`（macOS / Linux 都应有 · Windows 没）返回 fallback `[zsh|bash]`。
///
/// 调用方应该用此函数返回值作为 Settings UI dropdown · 让用户只能选系统真有的主流 shell ·
/// 而非自由输入 / hardcoded 列表（fix24/25 · 防 "PTY 启动失败: shell not executable"
/// + 减干扰非主流选项）。
pub fn list_available_shells() -> Vec<ShellInfo> {
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
        if is_executable_file(Path::new(fallback)) {
            let label = Path::new(fallback)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(fallback)
                .to_string();
            shells.push(ShellInfo {
                path: fallback.to_string(),
                label,
            });
        }
    }

    // 去重（macOS /etc/shells 经常 /bin/zsh + /usr/bin/zsh 都列）· 按 label 去重保留首项
    shells.dedup_by(|a, b| a.path == b.path);
    shells
}

pub(crate) fn resolve_shell(shell: &str) -> Option<PathBuf> {
    let path = Path::new(shell);
    if path.components().count() > 1 {
        return is_executable_file(path).then(|| path.to_path_buf());
    }

    resolve_shell_in_path(shell, std::env::var_os("PATH").as_deref())
}

fn resolve_shell_in_path(shell: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    path_var.and_then(|path_var| {
        std::env::split_paths(path_var)
            .map(|dir| dir.join(shell))
            .find(|candidate| is_executable_file(candidate))
    })
}

fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
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
    use std::os::unix::fs::PermissionsExt;
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
            effective_shell_for_spawn(default_shell_path(), Some("/opt/homebrew/bin/fish")),
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

    #[test]
    fn resolve_default_shell_returns_platform_default_when_no_db() {
        let shell = resolve_default_shell(None);
        if cfg!(target_os = "macos") {
            assert_eq!(shell, "/bin/zsh");
        } else {
            assert_eq!(shell, "/bin/bash");
        }
    }

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

    #[test]
    fn check_shell_exists_accepts_valid_shell() {
        assert!(check_shell_exists("/bin/sh").is_ok());
    }

    #[test]
    fn check_shell_exists_rejects_invalid_path() {
        let result = check_shell_exists("/bin/does-not-exist-vibestation-test");
        assert!(matches!(result, Err(PtyError::ShellNotFound(_))));
    }
}
