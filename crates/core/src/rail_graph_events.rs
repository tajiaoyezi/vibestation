//! MVP-12 Phase A · Rail Graph IPC event payload structs
//!
//! 仅定义 4 个 ts-rs payload struct · phase A 只建 contract · phase D 实施 emit。
//! 不含 tauri::command 实现 · 不含 invoke_handler 注册。

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Rail graph 视口同步事件（前端滚动时触发 · phase D emit）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphViewportSyncPayload {
    pub workspace_id: String,
    pub scroll_top: f64,
    pub row_height: f32,
    pub viewport_start: u32,
    pub viewport_end: u32,
}

/// Branch 变化事件（git:branch-changed 联动 · refs_hash 用于失效判断）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphBranchChangedPayload {
    pub workspace_id: String,
    pub head_oid: Option<String>,
    pub refs_hash: String,
    pub branch_count: u32,
}

/// Rebase 状态同步（MVP-16 overlay 联动 · phase D emit）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphRebaseStatePayload {
    pub workspace_id: String,
    /// "in_progress" | "done" | "aborted"
    pub state: String,
}

/// 性能采样（phase D perf budget 验收）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RailGraphPerfSample {
    pub workspace_id: String,
    /// "firstPaint" | "scroll" | "hover" | "branchChanged"
    pub phase: String,
    #[ts(type = "number")]
    pub duration_ms: f32,
    pub commit_count: u32,
    pub branch_count: u32,
}
