//! MVP-17 Phase B · Pane Detach
//!
//! Pane 弹为独立 Tauri WebviewWindow（仍在同一进程 · 共享 PTY backend）·
//! 关闭 detached window 时 Pane 回到原 LayoutNode 位置（前端 `detachedPanes`
//! Solid signal 还原 · LayoutNode schema 0 侵入）。
//!
//! **runtime-only state**：本模块的 `DetachedPaneMap` 是 in-memory HashMap ·
//! 不持久化 · App quit 时全清。重启回到 attached 状态（参 spec D.2 + H.5）。
//!
//! 本文件包含：
//! - 6 个 ts-rs IPC binding struct（5 IPC + 1 event payload）
//! - `DetachedPaneMap` runtime-only HashMap · 提供 insert / remove / get / list / clear
//! - `DetachError` enum · `thiserror` 派生 · 业务边界错误
//! - 单元测试 ≥ 15（DetachedPaneMap 状态机 + DetachError 转换）
//!
//! Tauri WebviewWindow lifecycle（创建 / 关闭 / close listener）在
//! `crates/app/src/pane_detach/window_manager.rs` · 不在本文件（core 层不依
//! 赖 Tauri）。

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

// =============================================================================
// IPC binding · 5 IPC struct + 1 event payload（ts-rs export）
// =============================================================================

/// `pane_detach_open` IPC 入参。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PaneDetachOpenRequest {
    pub pane_id: String,
}

/// `pane_detach_open` IPC 返回。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PaneDetachOpenResult {
    pub window_label: String,
    pub initial_bounds: DetachedWindowBounds,
}

/// `pane_detach_close` IPC 入参。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PaneDetachCloseRequest {
    pub window_label: String,
}

/// `pane_detach_close` IPC 返回 · 标识被重新 attach 的 pane。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PaneDetachCloseResult {
    pub pane_id: String,
}

/// `pane_detach_list` IPC 返回的单项。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PaneDetachListEntry {
    pub pane_id: String,
    pub window_label: String,
    pub bounds: DetachedWindowBounds,
}

/// Detached window 几何（创建时初始 bounds · 用户拖动后不更新到本结构）。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct DetachedWindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Default for DetachedWindowBounds {
    fn default() -> Self {
        Self {
            x: 40,
            y: 40,
            width: 800,
            height: 600,
        }
    }
}

/// `pane_detach_state_changed` 事件 payload。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct PaneDetachStateEvent {
    pub pane_id: String,
    pub action: PaneDetachAction,
    /// detach 时为 `Some(label)` · attach 时为 `None`。
    pub window_label: Option<String>,
}

/// Detach state 转换 action。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PaneDetachAction {
    Detached,
    Attached,
}

// =============================================================================
// 错误 enum · 业务边界错误
// =============================================================================

/// Pane Detach 操作错误。
///
/// 注意 IPC 边界传递时由 `crates/app/src/lib.rs` 的 handler 转 `String`（
/// 因 Tauri command 不直接消费 `thiserror` enum）· 但 core 层错误信息保留
/// 结构化便于内部 routing。
#[derive(Debug, Error, PartialEq)]
pub enum DetachError {
    #[error("pane {pane_id} is already detached (window_label={window_label})")]
    AlreadyDetached {
        pane_id: String,
        window_label: String,
    },

    #[error("pane {pane_id} is not detached (cannot close)")]
    NotDetached { pane_id: String },

    #[error("window_label {window_label} not found in DetachedPaneMap")]
    WindowLabelNotFound { window_label: String },

    #[error("WebviewWindow creation failed: {reason}")]
    WindowCreationFailed { reason: String },

    #[error("pane_id is empty")]
    PaneIdEmpty,
}

// =============================================================================
// DetachedWindowInfo · runtime entry
// =============================================================================

/// Detached window 运行时信息。
#[derive(Debug, Clone, PartialEq)]
pub struct DetachedWindowInfo {
    pub window_label: String,
    pub workspace_id: String,
    pub bounds: DetachedWindowBounds,
    pub created_at: u64, // unix epoch seconds
}

impl DetachedWindowInfo {
    pub fn new(
        window_label: impl Into<String>,
        workspace_id: impl Into<String>,
        bounds: DetachedWindowBounds,
    ) -> Self {
        Self {
            window_label: window_label.into(),
            workspace_id: workspace_id.into(),
            bounds,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}

// =============================================================================
// DetachedPaneMap · runtime-only state
// =============================================================================

/// 运行时 detached pane 状态映射。
///
/// - **不持久化**：App quit 时整体丢弃 · 重启回主窗口（spec D.2 + H.5）
/// - **idempotent**：repeated `insert` 返回 `AlreadyDetached` · 不破坏现有 entry
/// - **线程安全**：内部 `Mutex<HashMap>` · 所有公开方法获取锁后立即释放
#[derive(Debug, Default)]
pub struct DetachedPaneMap {
    inner: Mutex<HashMap<String, DetachedWindowInfo>>,
}

impl DetachedPaneMap {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// 注册一个 detached pane · pane_id 已存在则返回 `AlreadyDetached`。
    pub fn insert(
        &self,
        pane_id: impl Into<String>,
        info: DetachedWindowInfo,
    ) -> Result<(), DetachError> {
        let pane_id = pane_id.into();
        if pane_id.is_empty() {
            return Err(DetachError::PaneIdEmpty);
        }

        let mut map = self.inner.lock().expect("DetachedPaneMap poisoned");
        if let Some(existing) = map.get(&pane_id) {
            return Err(DetachError::AlreadyDetached {
                pane_id,
                window_label: existing.window_label.clone(),
            });
        }
        map.insert(pane_id, info);
        Ok(())
    }

    /// 移除 detached state · 返回原 info（若存在）。
    pub fn remove(&self, pane_id: &str) -> Option<DetachedWindowInfo> {
        let mut map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.remove(pane_id)
    }

    /// 通过 window_label 反查 pane_id + 移除 · close window 路径。
    pub fn remove_by_label(&self, window_label: &str) -> Result<String, DetachError> {
        let mut map = self.inner.lock().expect("DetachedPaneMap poisoned");
        let found = map
            .iter()
            .find(|(_, info)| info.window_label == window_label)
            .map(|(pid, _)| pid.clone());

        match found {
            Some(pane_id) => {
                map.remove(&pane_id);
                Ok(pane_id)
            }
            None => Err(DetachError::WindowLabelNotFound {
                window_label: window_label.to_string(),
            }),
        }
    }

    /// 查询某 pane 是否 detached · 返回 clone（不持锁外部使用）。
    pub fn get(&self, pane_id: &str) -> Option<DetachedWindowInfo> {
        let map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.get(pane_id).cloned()
    }

    /// 全量列举（UI / IPC list 接口）。
    pub fn list(&self) -> Vec<(String, DetachedWindowInfo)> {
        let map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.iter()
            .map(|(pid, info)| (pid.clone(), info.clone()))
            .collect()
    }

    /// 是否有 pane 处于 detached 状态。
    pub fn is_detached(&self, pane_id: &str) -> bool {
        let map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.contains_key(pane_id)
    }

    /// 当前 detached 数量。
    pub fn len(&self) -> usize {
        let map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        let map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.is_empty()
    }

    /// App quit / workspace 切换前清空（无副作用 · IPC 接收方独立 close window）。
    pub fn clear(&self) {
        let mut map = self.inner.lock().expect("DetachedPaneMap poisoned");
        map.clear();
    }

    /// 转 IPC list entries（按 pane_id 字典序稳定）。
    pub fn list_entries(&self) -> Vec<PaneDetachListEntry> {
        let mut entries: Vec<PaneDetachListEntry> = self
            .list()
            .into_iter()
            .map(|(pane_id, info)| PaneDetachListEntry {
                pane_id,
                window_label: info.window_label,
                bounds: info.bounds,
            })
            .collect();
        entries.sort_by(|a, b| a.pane_id.cmp(&b.pane_id));
        entries
    }
}

// =============================================================================
// 工具：生成 window_label
// =============================================================================

/// 生成新 window label · 形如 `pane-detach-<uuid-v4-no-hyphen>`（短化）。
///
/// **不依赖外部 uuid crate**：用 `SystemTime` nanos + counter mix · 满足
/// "Tauri WebviewWindow label 全局唯一" 即可。session 30 可换 uuid crate 若
/// 需要真随机。
pub fn generate_window_label(seed: u64) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    let label_id = (nanos.wrapping_add(seed)).wrapping_mul(2654435761);
    format!("pane-detach-{label_id:016x}")
}

// =============================================================================
// 单元测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_info(label: &str) -> DetachedWindowInfo {
        DetachedWindowInfo::new(label, "workspace-1", DetachedWindowBounds::default())
    }

    #[test]
    fn map_new_is_empty() {
        let map = DetachedPaneMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn insert_then_get_returns_clone() {
        let map = DetachedPaneMap::new();
        let info = sample_info("pane-detach-aaa");
        map.insert("pane-1", info.clone()).expect("insert ok");

        let got = map.get("pane-1").expect("entry present");
        assert_eq!(got, info);
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn insert_empty_pane_id_rejected() {
        let map = DetachedPaneMap::new();
        let err = map
            .insert("", sample_info("pane-detach-aaa"))
            .expect_err("empty pane_id rejected");
        assert_eq!(err, DetachError::PaneIdEmpty);
    }

    #[test]
    fn insert_duplicate_returns_already_detached() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .expect("first insert ok");

        let err = map
            .insert("pane-1", sample_info("pane-detach-bbb"))
            .expect_err("dup rejected");

        match err {
            DetachError::AlreadyDetached {
                pane_id,
                window_label,
            } => {
                assert_eq!(pane_id, "pane-1");
                assert_eq!(window_label, "pane-detach-aaa");
            }
            other => panic!("expected AlreadyDetached · got {other:?}"),
        }
    }

    #[test]
    fn remove_returns_original_info() {
        let map = DetachedPaneMap::new();
        let info = sample_info("pane-detach-aaa");
        map.insert("pane-1", info.clone()).unwrap();

        let removed = map.remove("pane-1").expect("entry removed");
        assert_eq!(removed, info);
        assert!(map.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let map = DetachedPaneMap::new();
        assert!(map.remove("pane-missing").is_none());
    }

    #[test]
    fn remove_idempotent() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .unwrap();
        assert!(map.remove("pane-1").is_some());
        // 第二次 remove · idempotent · 不 panic
        assert!(map.remove("pane-1").is_none());
    }

    #[test]
    fn remove_by_label_finds_pane_id() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .unwrap();
        map.insert("pane-2", sample_info("pane-detach-bbb"))
            .unwrap();

        let pane_id = map.remove_by_label("pane-detach-bbb").expect("found");
        assert_eq!(pane_id, "pane-2");
        assert_eq!(map.len(), 1);
        assert!(map.get("pane-1").is_some());
        assert!(map.get("pane-2").is_none());
    }

    #[test]
    fn remove_by_label_not_found_returns_error() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .unwrap();
        let err = map
            .remove_by_label("pane-detach-missing")
            .expect_err("not found");

        match err {
            DetachError::WindowLabelNotFound { window_label } => {
                assert_eq!(window_label, "pane-detach-missing");
            }
            other => panic!("expected WindowLabelNotFound · got {other:?}"),
        }
    }

    #[test]
    fn is_detached_truthy_after_insert() {
        let map = DetachedPaneMap::new();
        assert!(!map.is_detached("pane-1"));
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .unwrap();
        assert!(map.is_detached("pane-1"));
    }

    #[test]
    fn list_returns_all_entries() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .unwrap();
        map.insert("pane-2", sample_info("pane-detach-bbb"))
            .unwrap();
        map.insert("pane-3", sample_info("pane-detach-ccc"))
            .unwrap();
        let list = map.list();
        assert_eq!(list.len(), 3);
        let ids: Vec<&str> = list.iter().map(|(pid, _)| pid.as_str()).collect();
        assert!(ids.contains(&"pane-1"));
        assert!(ids.contains(&"pane-2"));
        assert!(ids.contains(&"pane-3"));
    }

    #[test]
    fn list_entries_sorted_by_pane_id() {
        let map = DetachedPaneMap::new();
        map.insert("pane-c", sample_info("pane-detach-ccc"))
            .unwrap();
        map.insert("pane-a", sample_info("pane-detach-aaa"))
            .unwrap();
        map.insert("pane-b", sample_info("pane-detach-bbb"))
            .unwrap();
        let entries = map.list_entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].pane_id, "pane-a");
        assert_eq!(entries[1].pane_id, "pane-b");
        assert_eq!(entries[2].pane_id, "pane-c");
    }

    #[test]
    fn clear_removes_all_entries() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1", sample_info("pane-detach-aaa"))
            .unwrap();
        map.insert("pane-2", sample_info("pane-detach-bbb"))
            .unwrap();
        assert_eq!(map.len(), 2);
        map.clear();
        assert!(map.is_empty());
    }

    #[test]
    fn bounds_default_matches_spec() {
        let bounds = DetachedWindowBounds::default();
        assert_eq!(bounds.x, 40);
        assert_eq!(bounds.y, 40);
        assert_eq!(bounds.width, 800);
        assert_eq!(bounds.height, 600);
    }

    #[test]
    fn detach_action_serializes_lowercase() {
        let payload = PaneDetachStateEvent {
            pane_id: "pane-1".to_string(),
            action: PaneDetachAction::Detached,
            window_label: Some("pane-detach-aaa".to_string()),
        };
        let json = serde_json::to_string(&payload).expect("serialize ok");
        assert!(json.contains("\"action\":\"detached\""));
        assert!(json.contains("\"paneId\":\"pane-1\""));
        assert!(json.contains("\"windowLabel\":\"pane-detach-aaa\""));
    }

    #[test]
    fn attached_event_has_null_window_label() {
        let payload = PaneDetachStateEvent {
            pane_id: "pane-1".to_string(),
            action: PaneDetachAction::Attached,
            window_label: None,
        };
        let json = serde_json::to_string(&payload).expect("serialize ok");
        assert!(json.contains("\"action\":\"attached\""));
        assert!(json.contains("\"windowLabel\":null"));
    }

    #[test]
    fn generate_window_label_format() {
        let label = generate_window_label(0);
        assert!(label.starts_with("pane-detach-"));
        assert!(label.len() > "pane-detach-".len());
    }

    #[test]
    fn generate_window_label_different_seeds_yield_different_labels() {
        let a = generate_window_label(0);
        let b = generate_window_label(1);
        // seed 不同 + 时间扰动 → 大概率不同
        // 仅在 0 时间精度下可能撞 · 但 wrapping_mul(2654435761) 让 +1 ≠ 0
        assert_ne!(a, b);
    }

    #[test]
    fn detached_window_info_new_sets_fields() {
        let info = DetachedWindowInfo::new(
            "pane-detach-xxx",
            "workspace-42",
            DetachedWindowBounds::default(),
        );
        assert_eq!(info.window_label, "pane-detach-xxx");
        assert_eq!(info.workspace_id, "workspace-42");
        assert!(info.created_at > 0);
    }
}
