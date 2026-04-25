use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    time::Duration,
};

use git2::Repository;
use tempfile::TempDir;
use vibestation_core::GitStatusWatcher;

fn fixture_repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    Repository::init(dir.path()).unwrap();
    dir
}

fn assert_no_event(rx: &Receiver<()>, timeout: Duration) {
    match rx.recv_timeout(timeout) {
        Err(RecvTimeoutError::Timeout) => {}
        Ok(()) => panic!("unexpected git status watch callback"),
        Err(error) => panic!("callback channel closed unexpectedly: {error}"),
    }
}

#[test]
fn git_status_watch_triggers_once_for_file_write() {
    let repo = fixture_repo();
    let (tx, rx) = mpsc::channel();

    let _watcher = GitStatusWatcher::spawn(repo.path().to_path_buf(), move || {
        let _ = tx.send(());
    })
    .unwrap();

    std::fs::write(repo.path().join("watched.txt"), "changed\n").unwrap();

    rx.recv_timeout(Duration::from_secs(4))
        .expect("file write should trigger git status watch callback");
    assert_no_event(&rx, Duration::from_millis(500));
}

#[test]
fn git_status_watch_ignores_git_index_lock() {
    let repo = fixture_repo();
    let (tx, rx) = mpsc::channel();

    let _watcher = GitStatusWatcher::spawn(repo.path().to_path_buf(), move || {
        let _ = tx.send(());
    })
    .unwrap();

    std::fs::write(repo.path().join(".git/index.lock"), "lock").unwrap();

    assert_no_event(&rx, Duration::from_millis(700));
}
