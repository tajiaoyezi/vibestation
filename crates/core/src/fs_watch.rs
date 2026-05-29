use std::{
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use notify::{
    event::EventKind, Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};

pub const GIT_STATUS_WATCH_DEBOUNCE: Duration = Duration::from_millis(200);

const WATCH_IDLE_TICK: Duration = Duration::from_millis(50);
const WATCH_START_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, thiserror::Error)]
pub enum GitStatusWatchError {
    #[error("unsupported fs watch platform: {0}")]
    UnsupportedPlatform(String),
    #[error("notify watcher failed: {0}")]
    Notify(String),
    #[error("watcher thread failed to start: {0}")]
    ThreadStart(String),
    #[error("watcher thread did not report startup")]
    StartupTimeout,
    #[error("watcher startup channel closed")]
    StartupChannelClosed,
}

impl From<notify::Error> for GitStatusWatchError {
    fn from(error: notify::Error) -> Self {
        Self::Notify(error.to_string())
    }
}

pub struct GitStatusWatcher {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl GitStatusWatcher {
    pub fn spawn<F>(repo_path: PathBuf, callback: F) -> Result<Self, GitStatusWatchError>
    where
        F: FnMut() + Send + 'static,
    {
        // ADR-006: notify 跨平台，Windows backend 即 ReadDirectoryChangesW（自动选择）。
        // 不再短路 Windows —— 三平台共用同一 RecommendedWatcher 路径，
        // 200ms debounce + .git/index.lock 排除 + dunce::canonicalize 契约一致。
        let repo_path = dunce::canonicalize(&repo_path).unwrap_or(repo_path);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let (startup_tx, startup_rx) = mpsc::channel();

        let join = thread::Builder::new()
            .name("vibestation-git-status-watch".to_string())
            .spawn(move || {
                let (event_tx, event_rx) = mpsc::channel();
                let watcher = RecommendedWatcher::new(
                    move |event| {
                        let _ = event_tx.send(event);
                    },
                    Config::default(),
                )
                .and_then(|mut watcher| {
                    watcher.watch(&repo_path, RecursiveMode::Recursive)?;
                    Ok(watcher)
                });

                match watcher {
                    Ok(watcher) => {
                        let _ = startup_tx.send(Ok(()));
                        run_watch_loop(repo_path, watcher, event_rx, thread_stop, callback);
                    }
                    Err(error) => {
                        let _ = startup_tx.send(Err(error.to_string()));
                    }
                }
            })
            .map_err(|error| GitStatusWatchError::ThreadStart(error.to_string()))?;

        match startup_rx.recv_timeout(WATCH_START_TIMEOUT) {
            Ok(Ok(())) => Ok(Self {
                stop,
                join: Some(join),
            }),
            Ok(Err(error)) => {
                let _ = join.join();
                Err(GitStatusWatchError::Notify(error))
            }
            Err(RecvTimeoutError::Timeout) => {
                stop.store(true, Ordering::Relaxed);
                let _ = join.join();
                Err(GitStatusWatchError::StartupTimeout)
            }
            Err(RecvTimeoutError::Disconnected) => {
                let _ = join.join();
                Err(GitStatusWatchError::StartupChannelClosed)
            }
        }
    }
}

impl Drop for GitStatusWatcher {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn run_watch_loop<F>(
    repo_path: PathBuf,
    _watcher: RecommendedWatcher,
    event_rx: Receiver<notify::Result<Event>>,
    stop: Arc<AtomicBool>,
    mut callback: F,
) where
    F: FnMut() + Send + 'static,
{
    let mut debouncer = WatchDebouncer::new(GIT_STATUS_WATCH_DEBOUNCE);

    while !stop.load(Ordering::Relaxed) {
        if debouncer.flush_if_due(Instant::now()) {
            callback();
            continue;
        }

        let timeout = debouncer
            .next_timeout(Instant::now())
            .unwrap_or(WATCH_IDLE_TICK);
        match event_rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                if should_process_git_status_event(&repo_path, &event) {
                    debouncer.mark_changed(Instant::now());
                }
            }
            Ok(Err(error)) => {
                eprintln!("[mvp-08] fs watch event failed: {error}");
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
}

#[derive(Debug)]
struct WatchDebouncer {
    debounce: Duration,
    deadline: Option<Instant>,
}

impl WatchDebouncer {
    fn new(debounce: Duration) -> Self {
        Self {
            debounce,
            deadline: None,
        }
    }

    fn mark_changed(&mut self, now: Instant) {
        self.deadline = Some(now + self.debounce);
    }

    fn next_timeout(&self, now: Instant) -> Option<Duration> {
        self.deadline
            .map(|deadline| deadline.saturating_duration_since(now))
    }

    fn flush_if_due(&mut self, now: Instant) -> bool {
        let Some(deadline) = self.deadline else {
            return false;
        };
        if now < deadline {
            return false;
        }
        self.deadline = None;
        true
    }
}

fn should_process_git_status_event(repo_path: &Path, event: &Event) -> bool {
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }

    event.paths.is_empty()
        || event
            .paths
            .iter()
            .any(|path| should_process_git_status_path(repo_path, path))
}

fn should_process_git_status_path(repo_path: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(repo_path).unwrap_or(path);
    let mut components = relative.components().filter_map(component_name);

    let Some(first) = components.next() else {
        return false;
    };

    match first.as_str() {
        ".git" => {
            matches!(components.next().as_deref(), Some("index")) && components.next().is_none()
        }
        "node_modules" | "target" => false,
        _ => true,
    }
}

fn component_name(component: Component<'_>) -> Option<String> {
    match component {
        Component::Normal(value) => Some(value.to_string_lossy().to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::RecvTimeoutError;
    use tempfile::TempDir;

    fn assert_no_event<T>(rx: &Receiver<T>, timeout: Duration) {
        match rx.recv_timeout(timeout) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(_) => panic!("unexpected fs watch callback"),
            Err(error) => panic!("callback channel closed unexpectedly: {error}"),
        }
    }

    #[test]
    fn watcher_debounces_rapid_changes() {
        let mut debouncer = WatchDebouncer::new(Duration::from_millis(200));
        let start = Instant::now();

        for offset in 0..10 {
            let now = start + Duration::from_millis(offset * 10);
            debouncer.mark_changed(now);
            assert!(!debouncer.flush_if_due(now));
        }

        assert!(!debouncer.flush_if_due(start + Duration::from_millis(289)));
        assert!(debouncer.flush_if_due(start + Duration::from_millis(290)));
        assert!(!debouncer.flush_if_due(start + Duration::from_millis(500)));
    }

    #[test]
    fn watcher_excludes_git_internals() {
        let repo = Path::new("/tmp/repo");

        assert!(should_process_git_status_path(
            repo,
            &repo.join("src/main.rs")
        ));
        assert!(should_process_git_status_path(
            repo,
            &repo.join(".git/index")
        ));
        assert!(!should_process_git_status_path(
            repo,
            &repo.join(".git/index.lock")
        ));
        assert!(!should_process_git_status_path(
            repo,
            &repo.join(".git/objects/pack/pack-a.idx")
        ));
        assert!(!should_process_git_status_path(
            repo,
            &repo.join(".git/refs/heads/main")
        ));
        assert!(!should_process_git_status_path(
            repo,
            &repo.join("node_modules/pkg/index.js")
        ));
        assert!(!should_process_git_status_path(
            repo,
            &repo.join("target/debug/app")
        ));
    }

    // TEST-3.4.1 (AC1) — Windows spawn must return Ok, not UnsupportedPlatform.
    #[cfg(windows)]
    #[test]
    fn test_3_4_1_windows_spawn_returns_ok() {
        let repo = TempDir::new().unwrap();
        let result = GitStatusWatcher::spawn(repo.path().to_path_buf(), || {});

        assert!(
            result.is_ok(),
            "Windows spawn must return Ok, got {:?}",
            result.err()
        );
    }

    // TEST-3.4.2 (AC2) — file change inside a watched Windows dir triggers the callback.
    #[cfg(windows)]
    #[test]
    fn test_3_4_2_windows_file_change_triggers_callback() {
        let repo = TempDir::new().unwrap();
        let (tx, rx) = mpsc::channel();

        let _watcher = GitStatusWatcher::spawn(repo.path().to_path_buf(), move || {
            let _ = tx.send(());
        })
        .expect("Windows spawn should succeed");

        std::fs::write(repo.path().join("change.txt"), "hello").unwrap();

        // Generous timeout: ReadDirectoryChangesW + 200ms debounce; >> debounce x several.
        rx.recv_timeout(Duration::from_secs(5))
            .expect("file change should trigger callback within timeout");
    }

    // TEST-3.4.5 (AC5) — Windows path with spaces canonicalizes and watches successfully.
    #[cfg(windows)]
    #[test]
    fn test_3_4_5_windows_unc_canonicalize_watch() {
        let parent = TempDir::new().unwrap();
        let repo = parent.path().join("My Repo");
        std::fs::create_dir(&repo).unwrap();
        let (tx, rx) = mpsc::channel();

        let _watcher = GitStatusWatcher::spawn(repo.clone(), move || {
            let _ = tx.send(());
        })
        .expect("spawn on path with spaces should succeed");

        std::fs::write(repo.join("change.txt"), "hello").unwrap();
        rx.recv_timeout(Duration::from_secs(5))
            .expect("change under spaced path should trigger callback");
    }

    #[test]
    fn watcher_handles_workspace_change() {
        let old_workspace = TempDir::new().unwrap();
        let new_workspace = TempDir::new().unwrap();
        let (tx, rx) = mpsc::channel();

        let old_tx = tx.clone();
        let old_watcher = GitStatusWatcher::spawn(old_workspace.path().to_path_buf(), move || {
            let _ = old_tx.send("old");
        })
        .unwrap();

        std::fs::write(old_workspace.path().join("old.txt"), "one").unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(4)).unwrap(), "old");
        drop(old_watcher);

        let new_tx = tx.clone();
        let _new_watcher = GitStatusWatcher::spawn(new_workspace.path().to_path_buf(), move || {
            let _ = new_tx.send("new");
        })
        .unwrap();

        std::fs::write(old_workspace.path().join("old-again.txt"), "two").unwrap();
        assert_no_event(&rx, Duration::from_millis(500));

        std::fs::write(new_workspace.path().join("new.txt"), "three").unwrap();
        assert_eq!(rx.recv_timeout(Duration::from_secs(4)).unwrap(), "new");
    }
}
