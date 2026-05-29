use std::fs;
use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;
use tempfile::TempDir;
use vibestation_core::db;
use vibestation_core::{
    PtyEvent, PtyManager, PtySpawnRequest, TabCreateRequest, TabsDao, WorkspaceStore,
};

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
            Err(error) => panic!("timed out waiting for {tab_id} exit event: {error}"),
        }
    }
}

fn contains_subsequence(lines: &[String], expected: &[&str]) -> bool {
    let mut next = 0usize;
    for line in lines {
        if line.contains(expected[next]) {
            next += 1;
            if next == expected.len() {
                return true;
            }
        }
    }

    false
}

#[test]
#[cfg_attr(
    target_os = "linux",
    ignore = "Linux PTY exit timing 在 CI runner 上不稳定 · scrollback integration 依赖同一条 PTY close event 链路 · MVP-04 Phase D Ubuntu runtime 验证时统一补修"
)]
#[cfg_attr(
    target_os = "windows",
    ignore = "硬编码 /bin/sh + printf/exit Unix shell 语义 · Windows 无 /bin/sh · scrollback 持久化的 Windows ConPTY 等价覆盖见 pty_windows_conpty_integration.rs（task-2.2）· ADR-005"
)]
fn stdout_is_persisted_and_forced_flush_keeps_partial_tail() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("vibestation.db");
    let pool = db::open_pool(&db_path).unwrap();

    let workspace_dir = dir.path().join("workspace");
    fs::create_dir_all(&workspace_dir).unwrap();
    let workspace = WorkspaceStore::create(&pool, workspace_dir.to_str().unwrap(), None).unwrap();
    let tab = TabsDao::create(
        &pool,
        &TabCreateRequest {
            workspace_id: workspace.workspace_id.clone(),
            name: Some("Terminal".to_string()),
            shell: Some("/bin/sh".to_string()),
            cwd: Some(workspace.path.clone()),
        },
    )
    .unwrap();

    let manager = PtyManager::new();
    manager.set_pool(pool.clone());
    let events = manager.take_event_receiver().unwrap();

    manager
        .spawn(PtySpawnRequest {
            tab_id: tab.tab_id.clone(),
            shell: tab.shell.clone(),
            cwd: tab.cwd.clone(),
            cols: 80,
            rows: 24,
        })
        .unwrap();
    manager
        .stdin(
            &tab.tab_id,
            "printf 'alpha\\nbeta\\ngamma-tail-without-newline'; exit\n",
        )
        .unwrap();

    let (_output, exit_code) = recv_until_exit(&events, &tab.tab_id, Duration::from_secs(5));
    assert_eq!(exit_code, Some(0));

    drop(events);
    drop(manager);

    let lines = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 0, 64).unwrap();
    assert!(
        contains_subsequence(&lines, &["alpha", "beta", "gamma-tail-without-newline"]),
        "persisted scrollback should contain flushed output in order, got: {lines:?}"
    );
}
