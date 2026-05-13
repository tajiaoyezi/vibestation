#![cfg(feature = "integration")]

use vibestation_app_lib::pane_detach::state::{DetachError, DetachedPaneMap, DetachedWindowInfo};
use vibestation_app_lib::pane_detach::window_manager::{
    detached_window_spec, reattach_closed_window, require_reattach_closed_window,
};
use vibestation_core::{DetachedWindowBounds, PaneDetachAction};

fn info(label: &str) -> DetachedWindowInfo {
    DetachedWindowInfo::new(label, "workspace-1", DetachedWindowBounds::default())
}

#[test]
fn open_close_reattach_state_machine_removes_detached_entry() {
    let map = DetachedPaneMap::new();
    map.insert("pane-1".to_string(), info("pane-detach-1"))
        .expect("detached state inserted");

    let event = require_reattach_closed_window(&map, "pane-detach-1").expect("close command event");

    assert_eq!(event.pane_id, "pane-1");
    assert_eq!(event.action, PaneDetachAction::Attached);
    assert!(event.window_label.is_none());
    assert!(map.get(&"pane-1".to_string()).is_none());
}

#[test]
fn abnormal_destroy_path_is_graceful_when_window_is_already_missing() {
    let map = DetachedPaneMap::new();
    map.insert("pane-1".to_string(), info("pane-detach-1"))
        .expect("detached state inserted");

    let first = reattach_closed_window(&map, "pane-detach-1")
        .expect("destroy handler")
        .expect("first destroy emits");
    let second = reattach_closed_window(&map, "pane-detach-1").expect("second destroy noops");

    assert_eq!(first.pane_id, "pane-1");
    assert!(second.is_none());
}

#[test]
fn repeated_detach_returns_already_detached_without_replacing_existing_window() {
    let map = DetachedPaneMap::new();
    map.insert("pane-1".to_string(), info("pane-detach-first"))
        .expect("first detach");

    let err = map
        .insert("pane-1".to_string(), info("pane-detach-second"))
        .expect_err("duplicate detach rejected");

    assert!(matches!(err, DetachError::AlreadyDetached(pane_id) if pane_id == "pane-1"));
    assert_eq!(
        map.get(&"pane-1".to_string())
            .expect("existing")
            .window_label,
        "pane-detach-first"
    );
}

#[test]
fn app_quit_clear_removes_all_detached_state() {
    let map = DetachedPaneMap::new();
    map.insert("pane-1".to_string(), info("pane-detach-1"))
        .expect("insert 1");
    map.insert("pane-2".to_string(), info("pane-detach-2"))
        .expect("insert 2");

    map.clear().expect("clear on quit");

    assert!(map.list().is_empty());
}

#[test]
fn ipc_channel_race_double_close_is_idempotent() {
    let map = DetachedPaneMap::new();
    map.insert("pane-1".to_string(), info("pane-detach-1"))
        .expect("insert");

    let command_path =
        require_reattach_closed_window(&map, "pane-detach-1").expect("command close");
    let listener_path = reattach_closed_window(&map, "pane-detach-1").expect("listener close");

    assert_eq!(command_path.pane_id, "pane-1");
    assert!(listener_path.is_none());
}

#[test]
fn three_detached_panes_can_coexist_with_distinct_window_specs() {
    let map = DetachedPaneMap::new();
    for index in 1..=3 {
        let pane_id = format!("pane-{index}");
        let label = format!("pane-detach-{index}");
        let spec = detached_window_spec(&pane_id, &label, DetachedWindowBounds::default())
            .expect("window spec");
        map.insert(pane_id, info(&spec.window_label))
            .expect("insert detached pane");
    }

    let entries = map.list_entries();

    assert_eq!(entries.len(), 3);
    assert!(entries.iter().any(|entry| entry.pane_id == "pane-1"));
    assert!(entries.iter().any(|entry| entry.pane_id == "pane-2"));
    assert!(entries.iter().any(|entry| entry.pane_id == "pane-3"));
}
