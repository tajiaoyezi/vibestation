//! Pane storage · MVP-05 Phase A + MVP-14 Phase A.
//!
//! This module owns the `panes` table DAO plus the ts-rs IPC contract types for
//! Pane layout state. IPC commands are in [`crate::pane_service`] and
//! [`crate::pane_layout_advanced`] (MVP-14).

use crate::db::{DbError, DbPool};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MAX_LAYOUT_SPLIT_DEPTH: usize = 5;
pub const MAX_LAYOUT_PANES: usize = 6;
pub const MAX_LAYOUT_COLUMNS: usize = 3;
pub const MAX_LAYOUT_ROWS: usize = 2;
pub const MIN_SPLIT_RATIO: f32 = 0.05;
pub const MAX_SPLIT_RATIO: f32 = 0.95;

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export, rename_all_fields = "camelCase")]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LayoutNode {
    Single {
        pane_id: String,
    },
    Split {
        direction: SplitDir,
        #[ts(type = "number")]
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    pub fn pane_count(&self) -> usize {
        match self {
            Self::Single { .. } => 1,
            Self::Split { first, second, .. } => first.pane_count() + second.pane_count(),
        }
    }

    /// MVP-14 · v0.2 validator · 替换 MVP-05 `validate_mvp_05`。
    ///
    /// - depth 上限 5（原为 2）
    /// - 同向连续 split **允许**（不再拒绝 H(H(...))）
    /// - ratio clamp 到 [0.05, 0.95]
    /// - pane count 上限 6（前端 3 列 / 2 行维度限制的总量保护）
    /// - grid 上限 3 列 / 2 行（后端兜底，防 direct IPC / 并发绕过前端按钮状态）
    /// - 空 layout / 缺 pane id / 重复 pane id 拒绝
    pub fn validate_layout(&self) -> Result<(), PaneLayoutError> {
        let mut seen = std::collections::HashSet::new();
        let pane_count = self.validate_inner(0, &mut seen)?;
        if pane_count > MAX_LAYOUT_PANES {
            return Err(PaneLayoutError::PresetApplyFailed {
                preset: LayoutPresetKind::Solo,
                reason: format!("pane count {pane_count} exceeds v0.2 max {MAX_LAYOUT_PANES}"),
            });
        }
        let (columns, rows) = self.grid_dimensions();
        if columns > MAX_LAYOUT_COLUMNS {
            return Err(PaneLayoutError::PresetApplyFailed {
                preset: LayoutPresetKind::Solo,
                reason: format!("layout columns {columns} exceeds max {MAX_LAYOUT_COLUMNS}"),
            });
        }
        if rows > MAX_LAYOUT_ROWS {
            return Err(PaneLayoutError::PresetApplyFailed {
                preset: LayoutPresetKind::Solo,
                reason: format!("layout rows {rows} exceeds max {MAX_LAYOUT_ROWS}"),
            });
        }
        Ok(())
    }

    pub fn grid_dimensions(&self) -> (usize, usize) {
        match self {
            Self::Single { .. } => (1, 1),
            Self::Split {
                direction,
                first,
                second,
                ..
            } => {
                let (first_columns, first_rows) = first.grid_dimensions();
                let (second_columns, second_rows) = second.grid_dimensions();
                match direction {
                    SplitDir::Horizontal => {
                        (first_columns + second_columns, first_rows.max(second_rows))
                    }
                    SplitDir::Vertical => {
                        (first_columns.max(second_columns), first_rows + second_rows)
                    }
                }
            }
        }
    }

    fn validate_inner(
        &self,
        split_depth: usize,
        seen: &mut std::collections::HashSet<String>,
    ) -> Result<usize, PaneLayoutError> {
        match self {
            Self::Single { pane_id } => {
                if pane_id.is_empty() {
                    return Err(PaneLayoutError::PaneNotFound {
                        pane_id: "(empty)".to_string(),
                    });
                }
                if !seen.insert(pane_id.clone()) {
                    return Err(PaneLayoutError::DuplicatePane {
                        pane_id: pane_id.clone(),
                    });
                }
                Ok(1)
            }
            Self::Split {
                direction: _,
                ratio,
                first,
                second,
            } => {
                let next_depth = split_depth + 1;
                if next_depth > MAX_LAYOUT_SPLIT_DEPTH {
                    return Err(PaneLayoutError::MaxDepthExceeded {
                        max_depth: MAX_LAYOUT_SPLIT_DEPTH as u32,
                        attempted_depth: next_depth as u32,
                    });
                }
                if *ratio < MIN_SPLIT_RATIO || *ratio > MAX_SPLIT_RATIO {
                    return Err(PaneLayoutError::InvalidRatio {
                        ratio: *ratio,
                        min: MIN_SPLIT_RATIO,
                        max: MAX_SPLIT_RATIO,
                    });
                }

                let first_count = first.validate_inner(next_depth, seen)?;
                let second_count = second.validate_inner(next_depth, seen)?;
                Ok(first_count + second_count)
            }
        }
    }

    /// 保留 MVP-05 兼容接口 · 内部转调新 validator。
    #[deprecated(since = "0.2.0", note = "use validate_layout instead")]
    pub fn validate_mvp_05(&self) -> Result<(), PaneError> {
        self.validate_layout().map_err(PaneError::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum SplitDir {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneState {
    pub pane_id: String,
    pub tab_id: String,
    pub shell: String,
    pub cwd: String,
    #[ts(type = "number")]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneCreateRequest {
    pub tab_id: String,
    pub parent_pane_id: String,
    pub direction: SplitDir,
    pub shell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneCloseRequest {
    pub pane_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutApplyRequest {
    pub tab_id: String,
    pub preset: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SplitRatioUpdateRequest {
    pub pane_id: String,
    #[ts(type = "number")]
    pub new_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneFocusRequest {
    pub tab_id: String,
    pub focused_pane_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneListResponse {
    pub panes: Vec<PaneState>,
    pub layout: LayoutNode,
    pub focused_pane_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneScrollbackFetchRequest {
    pub pane_id: String,
    #[ts(type = "number")]
    pub offset: i64,
    #[ts(type = "number")]
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PanePtySpawnRequest {
    pub pane_id: String,
    pub shell: String,
    pub cwd: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PanePtyStdoutEvent {
    pub pane_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PanePtyExitedEvent {
    pub pane_id: String,
    pub exit_code: Option<i32>,
}

// ============================================================
// MVP-14 · Advanced layout IPC request/result types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutApplyAdvancedRequest {
    pub tab_id: String,
    pub preset: LayoutPresetKind,
    pub preserve_instances: bool,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutApplyResult {
    pub response: PaneListResponse,
    pub reused_pane_ids: Vec<String>,
    pub created_pane_ids: Vec<String>,
    pub closed_pane_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PaneNavDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneNavigateRequest {
    pub tab_id: String,
    pub from_pane_id: String,
    pub direction: PaneNavDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneNavigateResult {
    pub to_pane_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneMaximizeRequest {
    pub tab_id: String,
    pub pane_id: String,
    pub toggle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneMaximizeResult {
    pub maximized: bool,
    pub restored_layout: Option<LayoutNode>,
    pub restored_focused_pane_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneResizeStepRequest {
    pub tab_id: String,
    pub pane_id: String,
    pub direction: SplitDir,
    pub step_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutHistoryEntry {
    pub preset: LayoutPresetKind,
    #[ts(type = "number")]
    pub timestamp: i64,
    pub pane_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLayoutState {
    pub workspace_id: String,
    pub envelope: Option<LayoutEnvelope>,
}

// ============================================================
// MVP-14 · LayoutEnvelope v1 + LayoutPresetKind + PaneLayoutError
// ============================================================

/// LayoutNode v1 envelope · 顶层版本化容器。
///
/// 旧 `{"kind":"single","paneId":"p"}` 通过 [`LayoutEnvelope::from_legacy_node`]
/// 包装为 v1 envelope；序列化时输出包含 `version: 1` 的完整 JSON。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LayoutEnvelope {
    #[ts(type = "number")]
    pub version: u32,
    pub root: LayoutNode,
    pub focused_pane_id: Option<String>,
    #[ts(type = "number")]
    pub updated_at: i64,
}

impl LayoutEnvelope {
    pub fn new_solo(pane_id: &str) -> Self {
        Self {
            version: 1,
            root: LayoutNode::Single {
                pane_id: pane_id.to_string(),
            },
            focused_pane_id: Some(pane_id.to_string()),
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn from_legacy_node(node: LayoutNode, focused: Option<String>) -> Self {
        Self {
            version: 1,
            root: node,
            focused_pane_id: focused,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn try_from_json(json: &str) -> Result<Self, PaneLayoutError> {
        // 先尝试作为 envelope 解析
        if let Ok(envelope) = serde_json::from_str::<LayoutEnvelope>(json) {
            if envelope.version > 1 {
                return Err(PaneLayoutError::MigrationFailed {
                    version: Some(envelope.version),
                    message: "future version".to_string(),
                });
            }
            envelope.root.validate_layout()?;
            return Ok(envelope);
        }
        // 回退：尝试作为裸 LayoutNode 解析（MVP-05 旧格式）
        let node: LayoutNode =
            serde_json::from_str(json).map_err(|e| PaneLayoutError::MigrationFailed {
                version: None,
                message: format!("neither envelope nor legacy LayoutNode: {e}"),
            })?;
        node.validate_layout()?;
        Ok(Self::from_legacy_node(node, None))
    }

    pub fn to_json(&self) -> Result<String, PaneLayoutError> {
        serde_json::to_string(self).map_err(|e| PaneLayoutError::DbError {
            message: format!("serialize envelope: {e}"),
        })
    }
}

/// v0.2 Smart Layout 预设种类 · ts-rs 新 binding。
///
/// 保留旧 [`SmartLayoutKind`] 作为内部兼容类型 · 提供 `From` 转换。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum LayoutPresetKind {
    Solo,
    AiAndRunner,
    DualAi,
    TripleReview,
    Quad,
}

impl From<SmartLayoutKind> for LayoutPresetKind {
    fn from(kind: SmartLayoutKind) -> Self {
        match kind {
            SmartLayoutKind::Solo => Self::Solo,
            SmartLayoutKind::AiAndRunner => Self::AiAndRunner,
        }
    }
}

/// MVP-14 · advanced layout 错误 tagged union。
///
/// 通过 `From<PaneLayoutError> for PaneError` 兼容 MVP-05 的 [`PaneError::InvalidLayout`]。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PaneLayoutError {
    MaxDepthExceeded {
        max_depth: u32,
        attempted_depth: u32,
    },
    InvalidRatio {
        #[ts(type = "number")]
        ratio: f32,
        #[ts(type = "number")]
        min: f32,
        #[ts(type = "number")]
        max: f32,
    },
    PaneNotFound {
        pane_id: String,
    },
    DuplicatePane {
        pane_id: String,
    },
    PresetApplyFailed {
        preset: LayoutPresetKind,
        reason: String,
    },
    MigrationFailed {
        version: Option<u32>,
        message: String,
    },
    DbError {
        message: String,
    },
}

impl From<PaneLayoutError> for PaneError {
    fn from(e: PaneLayoutError) -> Self {
        PaneError::InvalidLayout(format!("{e:?}"))
    }
}

impl std::fmt::Display for PaneLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for PaneLayoutError {}

#[derive(Debug, thiserror::Error)]
pub enum PaneError {
    #[error("pane not found: {0}")]
    NotFound(String),
    #[error("invalid pane layout: {0}")]
    InvalidLayout(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

/// MVP-05 §C · Smart Layout 预设种类（向后兼容 · 内部使用）。
///
/// - `Solo`：保留当前聚焦 Pane · 关闭其他所有 Pane。
/// - `AiAndRunner`：保留当前聚焦 Pane + 第一个非聚焦 Pane · 强制右分屏 50/50 · 关闭其他。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartLayoutKind {
    Solo,
    AiAndRunner,
}

/// MVP-14 · 根据 preset 构建 LayoutNode（pure function · 不改 PTY · 不写 DB）。
///
/// 输入 `available_panes` 为当前存在的 pane id 列表（按 focus / DFS 顺序）·
/// 输出对应预设的 layout tree。
pub fn build_layout_for_preset(
    preset: LayoutPresetKind,
    available_panes: &[String],
) -> Result<LayoutNode, PaneLayoutError> {
    if available_panes.is_empty() {
        return Err(PaneLayoutError::PresetApplyFailed {
            preset,
            reason: "no available panes".to_string(),
        });
    }

    let solo = |idx: usize| -> Result<LayoutNode, PaneLayoutError> {
        available_panes
            .get(idx)
            .cloned()
            .ok_or_else(|| PaneLayoutError::PresetApplyFailed {
                preset,
                reason: format!(
                    "need pane at index {idx}, only {} available",
                    available_panes.len()
                ),
            })
            .map(|pane_id| LayoutNode::Single { pane_id })
    };

    match preset {
        LayoutPresetKind::Solo => solo(0),
        LayoutPresetKind::AiAndRunner => {
            if available_panes.len() < 2 {
                return Err(PaneLayoutError::PresetApplyFailed {
                    preset,
                    reason: "AI+Runner requires at least 2 panes".to_string(),
                });
            }
            Ok(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(solo(0)?),
                second: Box::new(solo(1)?),
            })
        }
        LayoutPresetKind::DualAi => {
            if available_panes.len() < 2 {
                return Err(PaneLayoutError::PresetApplyFailed {
                    preset,
                    reason: "Dual AI requires at least 2 panes".to_string(),
                });
            }
            Ok(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(solo(0)?),
                second: Box::new(solo(1)?),
            })
        }
        LayoutPresetKind::TripleReview => {
            if available_panes.len() < 3 {
                return Err(PaneLayoutError::PresetApplyFailed {
                    preset,
                    reason: "Triple Review requires at least 3 panes".to_string(),
                });
            }
            Ok(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(solo(0)?),
                second: Box::new(LayoutNode::Split {
                    direction: SplitDir::Vertical,
                    ratio: 0.5,
                    first: Box::new(solo(1)?),
                    second: Box::new(solo(2)?),
                }),
            })
        }
        LayoutPresetKind::Quad => {
            if available_panes.len() < 4 {
                return Err(PaneLayoutError::PresetApplyFailed {
                    preset,
                    reason: "Quad requires at least 4 panes".to_string(),
                });
            }
            Ok(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(LayoutNode::Split {
                    direction: SplitDir::Vertical,
                    ratio: 0.5,
                    first: Box::new(solo(0)?),
                    second: Box::new(solo(1)?),
                }),
                second: Box::new(LayoutNode::Split {
                    direction: SplitDir::Vertical,
                    ratio: 0.5,
                    first: Box::new(solo(2)?),
                    second: Box::new(solo(3)?),
                }),
            })
        }
    }
}

/// MVP-05 §H.2 · 在指定 Pane 处右分 / 下分一个新 Pane（pure function · 不修改原 layout）。
///
/// 在 `layout` 中找到 `parent_pane_id` 对应的 `Single` 节点 · 用 `Split { direction,
/// first: Single(parent), second: Single(new) }` 替换。若命中处位于同方向连续 split 组内，
/// 会按该组当前 item 总数重新平均 ratio，避免新增 pane 只平分当前 pane。新布局通过
/// `validate_layout()` 校验。
///
/// 入参：
/// - `layout`：当前布局（不被修改）
/// - `parent_pane_id`：在哪个 Pane 上发起 split（必须为现有 `Single` 节点）
/// - `direction`：分割方向（`Horizontal` 右分 · `Vertical` 下分）
/// - `new_pane_id`：新 Pane 的 ID（caller 生成 · 通常 UUID）
///
/// 出参：
/// - `Ok(LayoutNode)`：split 后的新布局
/// - `Err(PaneError::NotFound)`：`parent_pane_id` 不存在于 layout 中
/// - `Err(PaneError::InvalidLayout)`：split 会导致深度 / pane 数量超限 · 或 `new_pane_id`
///   已存在于 layout 中
pub fn split_layout(
    layout: &LayoutNode,
    parent_pane_id: &str,
    direction: SplitDir,
    new_pane_id: String,
) -> Result<LayoutNode, PaneError> {
    if !layout_contains_pane(layout, parent_pane_id) {
        return Err(PaneError::NotFound(parent_pane_id.to_string()));
    }
    if layout_contains_pane(layout, &new_pane_id) {
        return Err(PaneError::InvalidLayout(format!(
            "new pane id {new_pane_id} already exists in layout"
        )));
    }
    let new_layout = split_layout_inner(layout, parent_pane_id, &direction, &new_pane_id)
        .ok_or_else(|| PaneError::NotFound(parent_pane_id.to_string()))?;
    new_layout.validate_layout().map_err(PaneError::from)?;
    Ok(new_layout)
}

/// MVP-05 §H.2 · 关闭指定 Pane · 重排 layout（pure function）。
///
/// 在 layout tree 中找到包含 `pane_id` 的 `Single` 节点 · 把它的父 `Split` 替换为 sibling 子树
/// （即 sibling 占满 split 空间 · 新 layout 减少一个 Split 层）。
///
/// 入参：
/// - `layout`：当前布局
/// - `pane_id`：要关闭的 Pane
///
/// 出参：
/// - `Ok(LayoutNode)`：关闭后的新布局
/// - `Err(PaneError::NotFound)`：`pane_id` 不在 layout
/// - `Err(PaneError::InvalidLayout)`：layout 顶层只剩此 Pane（caller 应该转去关 Tab）
pub fn close_pane_in_layout(layout: &LayoutNode, pane_id: &str) -> Result<LayoutNode, PaneError> {
    if let LayoutNode::Single { pane_id: existing } = layout {
        if existing == pane_id {
            return Err(PaneError::InvalidLayout(format!(
                "cannot close last pane {pane_id}; close the tab instead"
            )));
        }
    }
    if !layout_contains_pane(layout, pane_id) {
        return Err(PaneError::NotFound(pane_id.to_string()));
    }
    // 调用 helper · 已确保 pane_id 存在 · 必返回 Some
    let new_layout = remove_pane(layout, pane_id).expect("pane existence checked above");
    new_layout.validate_layout().map_err(PaneError::from)?;
    Ok(new_layout)
}

/// MVP-05 §D · 调整指定 Split 节点的分割比例（pure function）。
///
/// 找到 `first` 子树包含 `parent_pane_id` 的 `Split` 节点 · 把它的 `ratio` 改为 `new_ratio`。
/// 这与 UI 拖拽分隔条的语义对齐：每个 Split 用其 first 子树代表 Pane 作为 dragger 关联点。
///
/// 入参：
/// - `layout`：当前布局
/// - `parent_pane_id`：first 子树包含此 Pane 的 Split 节点是目标
/// - `new_ratio`：新比例 · 必须在 [0.05, 0.95] 区间
///
/// 出参：
/// - `Ok(LayoutNode)`：ratio 已更新的新布局
/// - `Err(PaneError::NotFound)`：找不到匹配的 Split 节点
/// - `Err(PaneError::InvalidLayout)`：`new_ratio` 不在 [0.05, 0.95] 区间
pub fn update_split_ratio(
    layout: &LayoutNode,
    parent_pane_id: &str,
    new_ratio: f32,
) -> Result<LayoutNode, PaneError> {
    if !(MIN_SPLIT_RATIO..=MAX_SPLIT_RATIO).contains(&new_ratio) {
        return Err(PaneError::InvalidLayout(format!(
            "split ratio must be between {MIN_SPLIT_RATIO} and {MAX_SPLIT_RATIO}, got {new_ratio}"
        )));
    }
    update_ratio_inner(layout, parent_pane_id, new_ratio)
        .ok_or_else(|| PaneError::NotFound(parent_pane_id.to_string()))
}

/// MVP-14 · 查找包含 `pane_id` 的 Split 节点的 ratio（纯查询 · 不修改 layout）。
///
/// 返回 `Some(ratio)` 若找到；`None` 若 `pane_id` 不在任何 Split 的 first 子树中。
pub fn find_split_ratio(layout: &LayoutNode, pane_id: &str) -> Option<f32> {
    match layout {
        LayoutNode::Single { .. } => None,
        LayoutNode::Split {
            ratio,
            first,
            second,
            ..
        } => {
            if layout_contains_pane(first, pane_id) {
                Some(*ratio)
            } else if layout_contains_pane(second, pane_id) {
                // pane 在 second 子树中，递归查找（可能在内层 Split）
                find_split_ratio(second, pane_id).or(Some(*ratio))
            } else {
                None
            }
        }
    }
}

/// MVP-05 §C · 应用 Smart Layout 预设（pure function · 向后兼容）。
///
/// - `Solo`：返回 `Single { focused_pane_id }` · 关闭所有非聚焦 Pane。
/// - `AiAndRunner`：保留聚焦 Pane + 第一个非聚焦 Pane（按 layout DFS 顺序）· 强制水平右分屏
///   50/50 · 关闭其他。若 layout 只有一个 Pane（即聚焦 Pane）· 返回 `Err(PaneError::InvalidLayout)` ·
///   caller 应先 spawn 第二个 Pane 后再调用。
///
/// 入参：
/// - `layout`：当前布局
/// - `kind`：预设类型
/// - `focused_pane_id`：当前聚焦的 Pane（必须存在于 layout）
///
/// 出参：
/// - `Ok((LayoutNode, Vec<String>))`：新布局 + 被关闭的 pane_ids（caller 用于 PTY cleanup）
/// - `Err(PaneError::NotFound)`：`focused_pane_id` 不在 layout
/// - `Err(PaneError::InvalidLayout)`：AiAndRunner 在单 Pane 布局上调用
pub fn apply_smart_layout(
    layout: &LayoutNode,
    kind: SmartLayoutKind,
    focused_pane_id: &str,
) -> Result<(LayoutNode, Vec<String>), PaneError> {
    let all_pane_ids = collect_pane_ids(layout);
    if !all_pane_ids.iter().any(|id| id == focused_pane_id) {
        return Err(PaneError::NotFound(focused_pane_id.to_string()));
    }

    match kind {
        SmartLayoutKind::Solo => {
            let closed: Vec<String> = all_pane_ids
                .into_iter()
                .filter(|id| id != focused_pane_id)
                .collect();
            let new_layout = LayoutNode::Single {
                pane_id: focused_pane_id.to_string(),
            };
            new_layout.validate_layout().map_err(PaneError::from)?;
            Ok((new_layout, closed))
        }
        SmartLayoutKind::AiAndRunner => {
            let secondary = all_pane_ids
                .iter()
                .find(|id| id.as_str() != focused_pane_id)
                .cloned()
                .ok_or_else(|| {
                    PaneError::InvalidLayout(
                        "AI+Runner requires at least 2 panes; spawn a second pane first".into(),
                    )
                })?;
            let closed: Vec<String> = all_pane_ids
                .into_iter()
                .filter(|id| id != focused_pane_id && id != &secondary)
                .collect();
            let new_layout = LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(LayoutNode::Single {
                    pane_id: focused_pane_id.to_string(),
                }),
                second: Box::new(LayoutNode::Single { pane_id: secondary }),
            };
            new_layout.validate_layout().map_err(PaneError::from)?;
            Ok((new_layout, closed))
        }
    }
}

// === pure function 内部辅助 ============================================

/// 检查 layout 中是否存在某 pane_id。
fn layout_contains_pane(layout: &LayoutNode, pane_id: &str) -> bool {
    match layout {
        LayoutNode::Single { pane_id: existing } => existing == pane_id,
        LayoutNode::Split { first, second, .. } => {
            layout_contains_pane(first, pane_id) || layout_contains_pane(second, pane_id)
        }
    }
}

/// 收集 layout 中所有 pane_id（DFS · 保留遍历顺序）。
pub fn collect_pane_ids(layout: &LayoutNode) -> Vec<String> {
    let mut ids = Vec::new();
    collect_pane_ids_inner(layout, &mut ids);
    ids
}

fn collect_pane_ids_inner(layout: &LayoutNode, ids: &mut Vec<String>) {
    match layout {
        LayoutNode::Single { pane_id } => ids.push(pane_id.clone()),
        LayoutNode::Split { first, second, .. } => {
            collect_pane_ids_inner(first, ids);
            collect_pane_ids_inner(second, ids);
        }
    }
}

fn split_layout_inner(
    layout: &LayoutNode,
    target: &str,
    split_direction: &SplitDir,
    new_pane_id: &str,
) -> Option<LayoutNode> {
    match layout {
        LayoutNode::Single { pane_id } if pane_id == target => Some(LayoutNode::Split {
            direction: split_direction.clone(),
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: pane_id.clone(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: new_pane_id.to_string(),
            }),
        }),
        LayoutNode::Single { .. } => None,
        LayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            if layout_contains_pane(first, target) {
                let updated_first =
                    split_layout_inner(first, target, split_direction, new_pane_id)?;
                let updated = LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: *ratio,
                    first: Box::new(updated_first),
                    second: second.clone(),
                };
                return Some(rebalance_matching_split_group(updated, split_direction));
            }
            if layout_contains_pane(second, target) {
                let updated_second =
                    split_layout_inner(second, target, split_direction, new_pane_id)?;
                let updated = LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: *ratio,
                    first: first.clone(),
                    second: Box::new(updated_second),
                };
                return Some(rebalance_matching_split_group(updated, split_direction));
            }
            None
        }
    }
}

fn rebalance_matching_split_group(layout: LayoutNode, split_direction: &SplitDir) -> LayoutNode {
    match layout {
        LayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } if direction == *split_direction => rebalance_split_group(
            LayoutNode::Split {
                direction,
                ratio,
                first,
                second,
            },
            split_direction,
        ),
        other => other,
    }
}

fn rebalance_split_group(layout: LayoutNode, split_direction: &SplitDir) -> LayoutNode {
    match layout {
        LayoutNode::Split {
            direction,
            first,
            second,
            ..
        } if direction == *split_direction => {
            let first = rebalance_split_group(*first, split_direction);
            let second = rebalance_split_group(*second, split_direction);
            let first_items = split_axis_item_count(&first, split_direction);
            let second_items = split_axis_item_count(&second, split_direction);
            let total_items = first_items + second_items;
            LayoutNode::Split {
                direction,
                ratio: first_items as f32 / total_items as f32,
                first: Box::new(first),
                second: Box::new(second),
            }
        }
        other => other,
    }
}

fn split_axis_item_count(layout: &LayoutNode, split_direction: &SplitDir) -> usize {
    match layout {
        LayoutNode::Split {
            direction,
            first,
            second,
            ..
        } if direction == split_direction => {
            split_axis_item_count(first, split_direction)
                + split_axis_item_count(second, split_direction)
        }
        _ => 1,
    }
}

/// 从 layout 中删除指定 pane_id · 把其父 Split 折叠为 sibling 子树。
///
/// **前置条件**：caller 必须先用 `layout_contains_pane` 确认 pane_id 存在。
///
/// 返回值：
/// - `Some(new_layout)`：删除成功 · 新 layout（首层 Split 折叠为 sibling 子树）
/// - `None`：仅当传入的 layout 整体就是 `Single { pane_id }`（caller 的 close 入口已处理此场景）
fn remove_pane(layout: &LayoutNode, pane_id: &str) -> Option<LayoutNode> {
    match layout {
        LayoutNode::Single { pane_id: existing } if existing == pane_id => None,
        LayoutNode::Single { .. } => {
            // 不该到这（caller 已 layout_contains_pane 检查）· 但保留 clone 以防直接调用
            Some(layout.clone())
        }
        LayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            // 直接子节点命中（无论 Single 还是 Split 命中）· 折叠为 sibling
            if first_layout_contains_only_target(first, pane_id) {
                return Some((**second).clone());
            }
            if first_layout_contains_only_target(second, pane_id) {
                return Some((**first).clone());
            }
            // 否则递归到含 pane_id 那一边
            if layout_contains_pane(first, pane_id) {
                let new_first =
                    remove_pane(first, pane_id).expect("pane known to exist in first subtree");
                let new_layout = LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: *ratio,
                    first: Box::new(new_first),
                    second: second.clone(),
                };
                return Some(rebalance_matching_split_group(new_layout, direction));
            }
            if layout_contains_pane(second, pane_id) {
                let new_second =
                    remove_pane(second, pane_id).expect("pane known to exist in second subtree");
                let new_layout = LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: *ratio,
                    first: first.clone(),
                    second: Box::new(new_second),
                };
                return Some(rebalance_matching_split_group(new_layout, direction));
            }
            // pane_id 不在任何子树（前置条件违反）· 原样返回
            Some(layout.clone())
        }
    }
}

/// 判断子树是否就是 `Single { pane_id }`（即关掉它后该 split 折叠为 sibling）。
fn first_layout_contains_only_target(subtree: &LayoutNode, pane_id: &str) -> bool {
    matches!(subtree, LayoutNode::Single { pane_id: id } if id == pane_id)
}

/// 在 layout 中找到 first 子树包含 `parent_pane_id` 的 Split 节点 · 改它的 ratio。
/// 返回 `None` 表示找不到。
fn update_ratio_inner(
    layout: &LayoutNode,
    parent_pane_id: &str,
    new_ratio: f32,
) -> Option<LayoutNode> {
    match layout {
        LayoutNode::Single { .. } => None,
        LayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            if layout_contains_pane(first, parent_pane_id) {
                // first 子树含目标 · 命中本节点 → 改 ratio · 不动子树（dragger 关联本节点）
                return Some(LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: new_ratio,
                    first: first.clone(),
                    second: second.clone(),
                });
            }
            if layout_contains_pane(second, parent_pane_id) {
                // first 子树不含 · 但 second 含 → 递归到 second 找下一层 Split
                if let Some(updated_second) = update_ratio_inner(second, parent_pane_id, new_ratio)
                {
                    return Some(LayoutNode::Split {
                        direction: direction.clone(),
                        ratio: *ratio,
                        first: first.clone(),
                        second: Box::new(updated_second),
                    });
                }
            }
            None
        }
    }
}

pub struct PanesDao;

impl PanesDao {
    fn row_to_pane(row: &rusqlite::Row<'_>) -> Result<PaneState, rusqlite::Error> {
        Ok(PaneState {
            pane_id: row.get(0)?,
            tab_id: row.get(1)?,
            shell: row.get(2)?,
            cwd: row.get(3)?,
            created_at: row.get(4)?,
        })
    }

    pub fn insert(pool: &DbPool, pane: PaneState) -> Result<(), PaneError> {
        let conn = pool.get().map_err(DbError::from)?;
        conn.execute(
            "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
             VALUES (?1, ?2, ?3, ?4, '[]', ?5)",
            rusqlite::params![
                pane.pane_id,
                pane.tab_id,
                pane.shell,
                pane.cwd,
                pane.created_at
            ],
        )
        .map_err(|error| PaneError::Db(DbError::Query(error.to_string())))?;
        Ok(())
    }

    pub fn update(pool: &DbPool, pane: PaneState) -> Result<(), PaneError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute(
                "UPDATE panes
                 SET tab_id = ?2, shell = ?3, cwd = ?4, created_at = ?5
                 WHERE pane_id = ?1",
                rusqlite::params![
                    pane.pane_id,
                    pane.tab_id,
                    pane.shell,
                    pane.cwd,
                    pane.created_at,
                ],
            )
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(PaneError::NotFound(pane.pane_id));
        }
        Ok(())
    }

    pub fn delete(pool: &DbPool, pane_id: &str) -> Result<(), PaneError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute("DELETE FROM panes WHERE pane_id = ?1", [pane_id])
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(PaneError::NotFound(pane_id.to_string()));
        }
        Ok(())
    }

    pub fn list_by_tab(pool: &DbPool, tab_id: &str) -> Result<Vec<PaneState>, PaneError> {
        let conn = pool.get().map_err(DbError::from)?;
        let mut stmt = conn
            .prepare(
                "SELECT pane_id, tab_id, shell, cwd, created_at
                 FROM panes WHERE tab_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(DbError::from)?;

        let rows = stmt
            .query_map([tab_id], Self::row_to_pane)
            .map_err(DbError::from)?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(DbError::from)?);
        }
        Ok(result)
    }

    pub fn get(pool: &DbPool, pane_id: &str) -> Result<Option<PaneState>, PaneError> {
        let conn = pool.get().map_err(DbError::from)?;
        conn.query_row(
            "SELECT pane_id, tab_id, shell, cwd, created_at
             FROM panes WHERE pane_id = ?1",
            [pane_id],
            Self::row_to_pane,
        )
        .optional()
        .map_err(|error| PaneError::Db(DbError::Query(error.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::tabs::{TabCreateRequest, TabsDao};
    use crate::workspace::WorkspaceStore;
    use rusqlite::Connection;
    use tempfile::TempDir;

    mod mvp_05_helpers {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/mvp_05_helpers.rs"
        ));
    }

    fn setup() -> (TempDir, DbPool, String, String) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_panes.db");
        let pool = db::open_pool(&db_path).unwrap();
        let ws_dir = dir.path().join("test-ws");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        let tab = TabsDao::create(
            &pool,
            &TabCreateRequest {
                workspace_id: ws.workspace_id.clone(),
                name: Some("Pane Test".to_string()),
                shell: Some("/bin/zsh".to_string()),
                cwd: Some("/tmp".to_string()),
            },
        )
        .unwrap();
        (dir, pool, ws.workspace_id, tab.tab_id)
    }

    fn pane(pane_id: &str, tab_id: &str, created_at: i64) -> PaneState {
        PaneState {
            pane_id: pane_id.to_string(),
            tab_id: tab_id.to_string(),
            shell: "/bin/zsh".to_string(),
            cwd: "/tmp".to_string(),
            created_at,
        }
    }

    #[test]
    fn insert_pane_basic() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        let p1 = pane("p1", &tab_id, 1);
        PanesDao::insert(&pool, p1.clone()).unwrap();
        assert_eq!(PanesDao::get(&pool, "p1").unwrap(), Some(p1));
    }

    #[test]
    fn insert_rejects_missing_tab_fk() {
        let (_dir, pool, _ws_id, _tab_id) = setup();
        let result = PanesDao::insert(&pool, pane("p1", "missing-tab", 1));
        assert!(result.is_err(), "FK violation must reject orphan panes");
    }

    #[test]
    fn insert_rejects_duplicate_pane_id() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        PanesDao::insert(&pool, pane("p1", &tab_id, 1)).unwrap();
        let result = PanesDao::insert(&pool, pane("p1", &tab_id, 2));
        assert!(result.is_err(), "PRIMARY KEY must reject duplicate pane_id");
    }

    #[test]
    fn update_pane_fields() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        PanesDao::insert(&pool, pane("p1", &tab_id, 1)).unwrap();
        let updated = PaneState {
            pane_id: "p1".to_string(),
            tab_id: tab_id.clone(),
            shell: "/bin/bash".to_string(),
            cwd: "/Users/leaf/项目/very/long/path/with/unicode".to_string(),
            created_at: 99,
        };
        PanesDao::update(&pool, updated.clone()).unwrap();
        assert_eq!(PanesDao::get(&pool, "p1").unwrap(), Some(updated));
    }

    #[test]
    fn update_missing_pane_errors() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        let result = PanesDao::update(&pool, pane("missing", &tab_id, 1));
        assert!(matches!(result, Err(PaneError::NotFound(id)) if id == "missing"));
    }

    #[test]
    fn update_rejects_missing_tab_fk() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        PanesDao::insert(&pool, pane("p1", &tab_id, 1)).unwrap();
        let result = PanesDao::update(&pool, pane("p1", "missing-tab", 2));
        assert!(
            result.is_err(),
            "FK violation must reject moving pane to missing tab"
        );
    }

    #[test]
    fn delete_pane_removes_row() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        PanesDao::insert(&pool, pane("p1", &tab_id, 1)).unwrap();
        PanesDao::delete(&pool, "p1").unwrap();
        assert_eq!(PanesDao::get(&pool, "p1").unwrap(), None);
    }

    #[test]
    fn delete_missing_pane_errors() {
        let (_dir, pool, _ws_id, _tab_id) = setup();
        let result = PanesDao::delete(&pool, "missing");
        assert!(matches!(result, Err(PaneError::NotFound(id)) if id == "missing"));
    }

    #[test]
    fn delete_tab_cascades_panes() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        PanesDao::insert(&pool, pane("p1", &tab_id, 1)).unwrap();
        PanesDao::insert(&pool, pane("p2", &tab_id, 2)).unwrap();
        TabsDao::delete(&pool, &tab_id).unwrap();
        assert!(PanesDao::list_by_tab(&pool, &tab_id).unwrap().is_empty());
    }

    #[test]
    fn list_by_tab_returns_empty_for_unknown_tab() {
        let (_dir, pool, _ws_id, _tab_id) = setup();
        let panes = PanesDao::list_by_tab(&pool, "unknown-tab").unwrap();
        assert!(panes.is_empty());
    }

    #[test]
    fn list_by_tab_orders_created_at_desc() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        PanesDao::insert(&pool, pane("older", &tab_id, 10)).unwrap();
        PanesDao::insert(&pool, pane("newer", &tab_id, 20)).unwrap();
        let panes = PanesDao::list_by_tab(&pool, &tab_id).unwrap();
        let ids: Vec<_> = panes.iter().map(|p| p.pane_id.as_str()).collect();
        assert_eq!(ids, vec!["newer", "older"]);
    }

    #[test]
    fn list_by_tab_filters_other_tabs() {
        let (_dir, pool, ws_id, tab_id) = setup();
        let tab2 = TabsDao::create(
            &pool,
            &TabCreateRequest {
                workspace_id: ws_id,
                name: Some("Pane Test 2".to_string()),
                shell: Some("/bin/zsh".to_string()),
                cwd: Some("/tmp".to_string()),
            },
        )
        .unwrap();
        PanesDao::insert(&pool, pane("p1", &tab_id, 1)).unwrap();
        PanesDao::insert(&pool, pane("p2", &tab2.tab_id, 2)).unwrap();
        let panes = PanesDao::list_by_tab(&pool, &tab_id).unwrap();
        assert_eq!(panes.len(), 1);
        assert_eq!(panes[0].pane_id, "p1");
    }

    #[test]
    fn get_returns_none_for_missing_pane() {
        let (_dir, pool, _ws_id, _tab_id) = setup();
        assert_eq!(PanesDao::get(&pool, "missing").unwrap(), None);
    }

    #[test]
    fn get_handles_utf8_and_long_paths() {
        let (_dir, pool, _ws_id, tab_id) = setup();
        let pane = PaneState {
            pane_id: "pane-猫".to_string(),
            tab_id,
            shell: "/bin/zsh".to_string(),
            cwd: format!("/tmp/{}", "长路径".repeat(64)),
            created_at: 1,
        };
        PanesDao::insert(&pool, pane.clone()).unwrap();
        assert_eq!(PanesDao::get(&pool, "pane-猫").unwrap(), Some(pane));
    }

    #[test]
    fn layout_node_serde_roundtrip_tagged_union() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: "p1".to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: "p2".to_string(),
            }),
        };
        let json = serde_json::to_string(&layout).unwrap();
        assert!(json.contains(r#""kind":"split""#));
        assert!(json.contains(r#""paneId":"p1""#));
        assert!(json.contains(r#""direction":"horizontal""#));
        let decoded: LayoutNode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, layout);
    }

    #[test]
    fn h2_solo_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_solo_layout();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 1);
        layout.validate_layout().unwrap();
    }

    #[test]
    fn h2_horizontal_2pane_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_horizontal_2pane();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 2);
        layout.validate_layout().unwrap();
    }

    #[test]
    fn h2_vertical_2pane_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_vertical_2pane();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 2);
        layout.validate_layout().unwrap();
    }

    #[test]
    fn h2_2x2_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_2x2_layout();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 4);
        layout.validate_layout().unwrap();
    }

    #[test]
    fn h2_invalid_depth_6_layout_is_rejected_and_original_layout_unchanged() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_horizontal_2pane();
        let before = current_layout(&conn);
        // depth 6 布局应被新 validator 拒绝
        let invalid = layout_depth_6_alternating();
        let result = invalid.validate_layout();
        assert!(matches!(
            result,
            Err(PaneLayoutError::MaxDepthExceeded { max_depth: 5, .. })
        ));
        assert_eq!(current_layout(&conn), before);
    }

    #[test]
    fn h2_invalid_ratio_layout_is_rejected_and_original_layout_unchanged() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_vertical_2pane();
        let before = current_layout(&conn);
        // ratio 0.04 应被新 validator 拒绝
        let invalid = LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.04,
            first: Box::new(solo_layout("p1")),
            second: Box::new(solo_layout("p2")),
        };
        let result = invalid.validate_layout();
        assert!(matches!(result, Err(PaneLayoutError::InvalidRatio { .. })));
        assert_eq!(current_layout(&conn), before);
    }

    #[derive(Debug, Clone, Copy)]
    enum FailurePoint {
        PanesInsert,
        LayoutUpdate,
        PanesDelete,
        PanesBatchDelete,
        FocusedPaneUpdate,
    }

    #[derive(Debug, PartialEq)]
    struct PersistedState {
        layout: LayoutNode,
        pane_ids: Vec<String>,
        focused_pane_id: Option<String>,
    }

    fn persisted_state(conn: &Connection) -> PersistedState {
        let layout = current_layout(conn);
        let pane_ids = pane_ids(conn);
        let focused_pane_id = conn
            .query_row(
                "SELECT focused_pane_id FROM tabs WHERE tab_id = 't1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        PersistedState {
            layout,
            pane_ids,
            focused_pane_id,
        }
    }

    fn pane_ids(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT pane_id FROM panes WHERE tab_id = 't1' ORDER BY pane_id")
            .unwrap();
        stmt.query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn current_layout(conn: &Connection) -> LayoutNode {
        let layout_json: String = conn
            .query_row("SELECT layout FROM tabs WHERE tab_id = 't1'", [], |row| {
                row.get(0)
            })
            .unwrap();
        serde_json::from_str(&layout_json).unwrap()
    }

    fn persist_layout(tx: &rusqlite::Transaction<'_>, layout: &LayoutNode) -> rusqlite::Result<()> {
        let layout_json = serde_json::to_string(layout).unwrap();
        tx.execute(
            "UPDATE tabs SET layout = ?1 WHERE tab_id = 't1'",
            [layout_json],
        )?;
        Ok(())
    }

    fn split_with_mock_failure(
        conn: &mut Connection,
        failure: FailurePoint,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
             VALUES ('p2', 't1', '/bin/zsh', '/tmp', '[]', 2)",
            [],
        )?;
        if matches!(failure, FailurePoint::PanesInsert) {
            return Err("mock panes insert failure".into());
        }

        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: "p1".to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: "p2".to_string(),
            }),
        };
        if matches!(failure, FailurePoint::LayoutUpdate) {
            return Err("mock layout update failure".into());
        }
        persist_layout(&tx, &layout)?;
        tx.execute(
            "UPDATE tabs SET focused_pane_id = 'p2' WHERE tab_id = 't1'",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn close_with_mock_failure(
        conn: &mut Connection,
        failure: FailurePoint,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM panes WHERE pane_id = 'p2'", [])?;
        if matches!(failure, FailurePoint::PanesDelete) {
            return Err("mock panes delete failure".into());
        }

        let layout = LayoutNode::Single {
            pane_id: "p1".to_string(),
        };
        if matches!(failure, FailurePoint::LayoutUpdate) {
            return Err("mock layout update failure".into());
        }
        persist_layout(&tx, &layout)?;
        tx.execute(
            "UPDATE tabs SET focused_pane_id = 'p1' WHERE tab_id = 't1'",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    fn layout_apply_with_mock_failure(
        conn: &mut Connection,
        failure: FailurePoint,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM panes WHERE tab_id = 't1' AND pane_id <> 'p1'",
            [],
        )?;
        if matches!(failure, FailurePoint::PanesBatchDelete) {
            return Err("mock panes batch delete failure".into());
        }

        let layout = LayoutNode::Single {
            pane_id: "p1".to_string(),
        };
        persist_layout(&tx, &layout)?;
        if matches!(failure, FailurePoint::FocusedPaneUpdate) {
            return Err("mock focused pane update failure".into());
        }
        tx.execute(
            "UPDATE tabs SET focused_pane_id = 'p1' WHERE tab_id = 't1'",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }

    #[test]
    fn split_atomicity_fails_during_panes_insert() {
        let (_dir, mut conn) = mvp_05_helpers::create_fixture_solo_layout();
        let before = persisted_state(&conn);
        let result = split_with_mock_failure(&mut conn, FailurePoint::PanesInsert);
        assert!(result.is_err());
        assert_eq!(persisted_state(&conn), before);
    }

    #[test]
    fn split_atomicity_fails_during_layout_update() {
        let (_dir, mut conn) = mvp_05_helpers::create_fixture_solo_layout();
        let before = persisted_state(&conn);
        let result = split_with_mock_failure(&mut conn, FailurePoint::LayoutUpdate);
        assert!(result.is_err());
        assert_eq!(persisted_state(&conn), before);
    }

    #[test]
    fn close_atomicity_fails_during_panes_delete() {
        let (_dir, mut conn) = mvp_05_helpers::create_fixture_horizontal_2pane();
        let before = persisted_state(&conn);
        let result = close_with_mock_failure(&mut conn, FailurePoint::PanesDelete);
        assert!(result.is_err());
        assert_eq!(persisted_state(&conn), before);
    }

    #[test]
    fn close_atomicity_fails_during_layout_update() {
        let (_dir, mut conn) = mvp_05_helpers::create_fixture_horizontal_2pane();
        let before = persisted_state(&conn);
        let result = close_with_mock_failure(&mut conn, FailurePoint::LayoutUpdate);
        assert!(result.is_err());
        assert_eq!(persisted_state(&conn), before);
    }

    #[test]
    fn layout_apply_atomicity_fails_during_panes_batch_delete() {
        let (_dir, mut conn) = mvp_05_helpers::create_fixture_2x2_layout();
        let before = persisted_state(&conn);
        let result = layout_apply_with_mock_failure(&mut conn, FailurePoint::PanesBatchDelete);
        assert!(result.is_err());
        assert_eq!(persisted_state(&conn), before);
    }

    #[test]
    fn layout_apply_atomicity_fails_during_focused_pane_update() {
        let (_dir, mut conn) = mvp_05_helpers::create_fixture_2x2_layout();
        let before = persisted_state(&conn);
        let result = layout_apply_with_mock_failure(&mut conn, FailurePoint::FocusedPaneUpdate);
        assert!(result.is_err());
        assert_eq!(persisted_state(&conn), before);
    }

    // ============================================================
    // MVP-05 Phase B Step 2 · pure function 单元测试
    // ============================================================
    //
    // 涵盖：
    // - §H.2 6 case：split 合法 4 + split 非法 2
    // - §H.3.1 pure function 等价 6 case：split / close / 边界
    // - §C Smart Layout 3 case：Solo / AiAndRunner / 单 Pane Err
    // - §D update_split_ratio 2 case：legal + invalid ratio

    fn solo_layout(pane_id: &str) -> LayoutNode {
        LayoutNode::Single {
            pane_id: pane_id.to_string(),
        }
    }

    fn horizontal_2pane(left: &str, right: &str) -> LayoutNode {
        LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout(left)),
            second: Box::new(solo_layout(right)),
        }
    }

    fn vertical_2pane(top: &str, bottom: &str) -> LayoutNode {
        LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.5,
            first: Box::new(solo_layout(top)),
            second: Box::new(solo_layout(bottom)),
        }
    }

    // --- §H.2 case 1-4 · 合法 split -----------------------------------

    #[test]
    fn split_layout_horizontal_2_pane_legal() {
        let layout = solo_layout("p1");
        let result = split_layout(&layout, "p1", SplitDir::Horizontal, "p2".to_string()).unwrap();
        assert_eq!(result.pane_count(), 2);
        result.validate_layout().unwrap();
        assert_eq!(result, horizontal_2pane("p1", "p2"));
        // 原 layout 未被修改（pure function 不变性）
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn split_layout_vertical_2_pane_legal() {
        let layout = solo_layout("p1");
        let result = split_layout(&layout, "p1", SplitDir::Vertical, "p2".to_string()).unwrap();
        assert_eq!(result.pane_count(), 2);
        result.validate_layout().unwrap();
        assert_eq!(result, vertical_2pane("p1", "p2"));
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn split_layout_2x2_legal() {
        // 起点：水平 2 pane → 在右 pane (p2) 下分一个新 p3 → 应得 horizontal { left=p1, right=vertical{p2, p3}}
        let layout = horizontal_2pane("p1", "p2");
        let result = split_layout(&layout, "p2", SplitDir::Vertical, "p3".to_string()).unwrap();
        assert_eq!(result.pane_count(), 3);
        result.validate_layout().unwrap();
        let expected = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("p1")),
            second: Box::new(vertical_2pane("p2", "p3")),
        };
        assert_eq!(result, expected);

        // 再在 p1 下分 p4 → 真正的 2×2
        let result_2x2 = split_layout(&result, "p1", SplitDir::Vertical, "p4".to_string()).unwrap();
        assert_eq!(result_2x2.pane_count(), 4);
        result_2x2.validate_layout().unwrap();
    }

    #[test]
    fn split_layout_solo_legal() {
        // 起始单 pane · 只能分一次（horizontal 或 vertical）· 验证最简场景
        let layout = solo_layout("only");
        let result =
            split_layout(&layout, "only", SplitDir::Horizontal, "new".to_string()).unwrap();
        assert_eq!(result.pane_count(), 2);
        assert_eq!(result, horizontal_2pane("only", "new"));
    }

    #[test]
    fn split_layout_rebalances_horizontal_group_after_same_direction_split() {
        let layout = horizontal_2pane("p1", "p2");
        let result = split_layout(&layout, "p2", SplitDir::Horizontal, "p3".to_string()).unwrap();

        assert_eq!(result.pane_count(), 3);
        result.validate_layout().unwrap();
        assert_eq!(
            result,
            LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 1.0 / 3.0,
                first: Box::new(solo_layout("p1")),
                second: Box::new(LayoutNode::Split {
                    direction: SplitDir::Horizontal,
                    ratio: 0.5,
                    first: Box::new(solo_layout("p2")),
                    second: Box::new(solo_layout("p3")),
                }),
            }
        );
    }

    #[test]
    fn split_layout_rejects_vertical_rebalance_that_would_create_third_row() {
        let layout = vertical_2pane("p1", "p2");
        let result = split_layout(&layout, "p1", SplitDir::Vertical, "p3".to_string());

        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    #[test]
    fn split_layout_rejects_fourth_column() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 1.0 / 3.0,
            first: Box::new(solo_layout("p1")),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(solo_layout("p2")),
                second: Box::new(solo_layout("p3")),
            }),
        };

        let result = split_layout(&layout, "p3", SplitDir::Horizontal, "p4".to_string());

        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    #[test]
    fn split_layout_rejects_third_row() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.5,
            first: Box::new(solo_layout("p1")),
            second: Box::new(solo_layout("p2")),
        };

        let result = split_layout(&layout, "p2", SplitDir::Vertical, "p3".to_string());

        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    // --- §H.2 case 5-6 · 非法 split ----------------------------------

    #[test]
    fn split_layout_3_horizontal_illegal() {
        // MVP-14: 同向连续 split 现在允许 · 但 depth 6 会被拒绝
        // 测试 depth 超限场景
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(LayoutNode::Split {
                    direction: SplitDir::Horizontal,
                    ratio: 0.5,
                    first: Box::new(LayoutNode::Split {
                        direction: SplitDir::Horizontal,
                        ratio: 0.5,
                        first: Box::new(LayoutNode::Split {
                            direction: SplitDir::Horizontal,
                            ratio: 0.5,
                            first: Box::new(solo_layout("p1")),
                            second: Box::new(solo_layout("p2")),
                        }),
                        second: Box::new(solo_layout("p3")),
                    }),
                    second: Box::new(solo_layout("p4")),
                }),
                second: Box::new(solo_layout("p5")),
            }),
            second: Box::new(solo_layout("p6")),
        };
        // p1 在 depth 5 · 再 split → depth 6 超限
        let result = split_layout(&layout, "p1", SplitDir::Horizontal, "p7".to_string());
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    #[test]
    fn split_layout_3_vertical_illegal() {
        // 垂直方向同理
        let layout = LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.5,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(LayoutNode::Split {
                    direction: SplitDir::Vertical,
                    ratio: 0.5,
                    first: Box::new(LayoutNode::Split {
                        direction: SplitDir::Vertical,
                        ratio: 0.5,
                        first: Box::new(LayoutNode::Split {
                            direction: SplitDir::Vertical,
                            ratio: 0.5,
                            first: Box::new(solo_layout("p1")),
                            second: Box::new(solo_layout("p2")),
                        }),
                        second: Box::new(solo_layout("p3")),
                    }),
                    second: Box::new(solo_layout("p4")),
                }),
                second: Box::new(solo_layout("p5")),
            }),
            second: Box::new(solo_layout("p6")),
        };
        // p1 在 depth 5 · 再 split → depth 6 超限
        let result = split_layout(&layout, "p1", SplitDir::Vertical, "p7".to_string());
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    // --- §H.3.1 pure function 等价 6 case --------------------------

    #[test]
    fn atomic_split_success_returns_new_layout() {
        // case 1 · 正常 split + 验证新 layout · 原 layout 不变
        let layout = solo_layout("p1");
        let new_layout =
            split_layout(&layout, "p1", SplitDir::Horizontal, "p2".to_string()).unwrap();
        assert_eq!(new_layout, horizontal_2pane("p1", "p2"));
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn atomic_split_depth_exceeded_returns_unchanged() {
        // case 2 · 布局上限超限 · pure function 返回 Err · layout 不变
        // 3 columns × 2 rows 已满 · 再 split 会超过总上限 / 轴向上限。
        let layout = layout_full_3x2_grid();
        let original = layout.clone();
        let result = split_layout(&layout, "p1", SplitDir::Horizontal, "p7".to_string());
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        assert_eq!(layout, original);
    }

    #[test]
    fn atomic_close_success_returns_new_layout() {
        // case 3 · 正常 close + 验证 layout 重排（Split 折叠为 Single）
        let layout = horizontal_2pane("p1", "p2");
        let new_layout = close_pane_in_layout(&layout, "p2").unwrap();
        assert_eq!(new_layout, solo_layout("p1"));
        // 原 layout 不变
        assert_eq!(layout, horizontal_2pane("p1", "p2"));
    }

    #[test]
    fn close_pane_rebalances_horizontal_group_after_removal() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 1.0 / 3.0,
            first: Box::new(solo_layout("p1")),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(solo_layout("p2")),
                second: Box::new(solo_layout("p3")),
            }),
        };

        let result = close_pane_in_layout(&layout, "p2").unwrap();

        assert_eq!(result, horizontal_2pane("p1", "p3"));
    }

    #[test]
    fn close_pane_rebalances_vertical_group_after_removal() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 2.0 / 3.0,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(solo_layout("p1")),
                second: Box::new(solo_layout("p2")),
            }),
            second: Box::new(solo_layout("p3")),
        };

        let result = close_pane_in_layout(&layout, "p2").unwrap();

        assert_eq!(result, vertical_2pane("p1", "p3"));
    }

    #[test]
    fn atomic_close_last_pane_errors() {
        // case 4 · close 最后一个 pane · Err
        let layout = solo_layout("p1");
        let result = close_pane_in_layout(&layout, "p1");
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn atomic_invalid_pane_id_errors() {
        // case 5 · pane_id 不存在 · Err
        let layout = horizontal_2pane("p1", "p2");
        let result = close_pane_in_layout(&layout, "ghost");
        assert!(matches!(result, Err(PaneError::NotFound(id)) if id == "ghost"));
        assert_eq!(layout, horizontal_2pane("p1", "p2"));

        // 同样验证 split_layout 的 NotFound
        let split_result = split_layout(&layout, "ghost", SplitDir::Horizontal, "new".to_string());
        assert!(matches!(split_result, Err(PaneError::NotFound(id)) if id == "ghost"));
    }

    #[test]
    fn atomic_split_then_close_roundtrip() {
        // case 6 · split 后 close · layout 还原 Solo
        let layout = solo_layout("p1");
        let after_split =
            split_layout(&layout, "p1", SplitDir::Horizontal, "p2".to_string()).unwrap();
        assert_eq!(after_split.pane_count(), 2);
        let after_close = close_pane_in_layout(&after_split, "p2").unwrap();
        assert_eq!(after_close, solo_layout("p1"));
        // 整个 roundtrip 后回到起点
        assert_eq!(after_close, layout);
    }

    // --- §C Smart Layout 3 case --------------------------------------

    #[test]
    fn smart_layout_solo_keeps_focused_pane() {
        // 起始 2×2 → Solo focus p3 → 仅剩 p3
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(vertical_2pane("p1", "p2")),
            second: Box::new(vertical_2pane("p3", "p4")),
        };
        let (new_layout, closed) =
            apply_smart_layout(&layout, SmartLayoutKind::Solo, "p3").unwrap();
        assert_eq!(new_layout, solo_layout("p3"));
        let closed_set: std::collections::HashSet<_> = closed.into_iter().collect();
        let expected: std::collections::HashSet<_> =
            ["p1", "p2", "p4"].iter().map(|s| s.to_string()).collect();
        assert_eq!(closed_set, expected);
    }

    #[test]
    fn smart_layout_solo_returns_closed_pane_ids() {
        // 起始水平 2 pane · focus p1 · Solo → 关 p2 · closed = ["p2"]
        let layout = horizontal_2pane("p1", "p2");
        let (new_layout, closed) =
            apply_smart_layout(&layout, SmartLayoutKind::Solo, "p1").unwrap();
        assert_eq!(new_layout, solo_layout("p1"));
        assert_eq!(closed, vec!["p2".to_string()]);

        // 起始 Solo · focus 自己 · 无 pane 可关 · closed = []
        let solo = solo_layout("alone");
        let (new_layout, closed) =
            apply_smart_layout(&solo, SmartLayoutKind::Solo, "alone").unwrap();
        assert_eq!(new_layout, solo_layout("alone"));
        assert!(closed.is_empty());

        // focused pane 不存在 · Err
        let layout = horizontal_2pane("p1", "p2");
        let result = apply_smart_layout(&layout, SmartLayoutKind::Solo, "ghost");
        assert!(matches!(result, Err(PaneError::NotFound(id)) if id == "ghost"));
    }

    #[test]
    fn smart_layout_ai_runner_creates_horizontal_split_50_50() {
        // 起始 2×2 · focus p1 · AiAndRunner → horizontal{p1, p2}（second = first non-focused = p2）
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(vertical_2pane("p1", "p2")),
            second: Box::new(vertical_2pane("p3", "p4")),
        };
        let (new_layout, closed) =
            apply_smart_layout(&layout, SmartLayoutKind::AiAndRunner, "p1").unwrap();
        assert_eq!(new_layout, horizontal_2pane("p1", "p2"));
        let closed_set: std::collections::HashSet<_> = closed.into_iter().collect();
        let expected: std::collections::HashSet<_> =
            ["p3", "p4"].iter().map(|s| s.to_string()).collect();
        assert_eq!(closed_set, expected);

        // 起始 vertical 2 · focus top · AiAndRunner → 强制水平 50/50
        let vert = vertical_2pane("top", "bottom");
        let (h_layout, closed) =
            apply_smart_layout(&vert, SmartLayoutKind::AiAndRunner, "top").unwrap();
        assert_eq!(h_layout, horizontal_2pane("top", "bottom"));
        assert!(closed.is_empty());

        // 单 Pane · AiAndRunner → Err（caller 应先 spawn 第二个 Pane）
        let solo = solo_layout("only");
        let result = apply_smart_layout(&solo, SmartLayoutKind::AiAndRunner, "only");
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    // --- §D update_split_ratio 2 case --------------------------------

    #[test]
    fn update_split_ratio_changes_target_split_only() {
        // 起始 2×2 · 改最外层 horizontal split 的 ratio（first 子树含 p1）
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(vertical_2pane("p1", "p2")),
            second: Box::new(vertical_2pane("p3", "p4")),
        };
        let new_layout = update_split_ratio(&layout, "p1", 0.7).unwrap();
        if let LayoutNode::Split { ratio, .. } = &new_layout {
            assert!((ratio - 0.7).abs() < 1e-6);
        } else {
            panic!("expected Split at root, got {new_layout:?}");
        }
        // 原 layout 不变
        if let LayoutNode::Split { ratio, .. } = &layout {
            assert!((ratio - 0.5).abs() < 1e-6);
        }

        // 改 second 子树内的 vertical split ratio（first 子树含 p3）
        let new_layout = update_split_ratio(&layout, "p3", 0.3).unwrap();
        if let LayoutNode::Split { second, .. } = &new_layout {
            if let LayoutNode::Split { ratio, .. } = second.as_ref() {
                assert!((ratio - 0.3).abs() < 1e-6);
            } else {
                panic!("expected nested Split in second");
            }
        }
    }

    #[test]
    fn update_split_ratio_rejects_invalid_ratio_and_unknown_pane() {
        let layout = horizontal_2pane("p1", "p2");
        // ratio = 0.04 · 区间外
        let result = update_split_ratio(&layout, "p1", 0.04);
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // ratio = 0.96 · 区间外
        let result = update_split_ratio(&layout, "p1", 0.96);
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // ratio < 0
        let result = update_split_ratio(&layout, "p1", -0.1);
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // ratio > 1
        let result = update_split_ratio(&layout, "p1", 1.5);
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // unknown pane
        let result = update_split_ratio(&layout, "ghost", 0.5);
        assert!(matches!(result, Err(PaneError::NotFound(id)) if id == "ghost"));
    }

    // ============================================================
    // MVP-14 Phase A · 新增测试
    // ============================================================

    // --- A.1 LayoutEnvelope round-trip --------------------------------

    #[test]
    fn layout_envelope_legacy_roundtrip() {
        let legacy_json = r#"{"kind":"single","paneId":"p1"}"#;
        let envelope = LayoutEnvelope::try_from_json(legacy_json).unwrap();
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.root, solo_layout("p1"));
        assert_eq!(envelope.focused_pane_id, None);

        // 序列化后应包含 version 字段
        let json = envelope.to_json().unwrap();
        assert!(json.contains(r#""version":1"#));
        assert!(json.contains(r#""paneId":"p1""#));
    }

    #[test]
    fn layout_envelope_v1_roundtrip() {
        let envelope = LayoutEnvelope {
            version: 1,
            root: horizontal_2pane("p1", "p2"),
            focused_pane_id: Some("p1".to_string()),
            updated_at: 1760000000000,
        };
        let json = envelope.to_json().unwrap();
        let parsed = LayoutEnvelope::try_from_json(&json).unwrap();
        assert_eq!(parsed, envelope);
    }

    #[test]
    fn layout_envelope_future_version_fails() {
        let future_json = r#"{"version":2,"root":{"kind":"single","paneId":"p1"},"focusedPaneId":null,"updatedAt":0}"#;
        let result = LayoutEnvelope::try_from_json(future_json);
        assert!(matches!(
            result,
            Err(PaneLayoutError::MigrationFailed {
                version: Some(2),
                ..
            })
        ));
    }

    #[test]
    fn layout_envelope_new_solo() {
        let envelope = LayoutEnvelope::new_solo("pane-1");
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.root, solo_layout("pane-1"));
        assert_eq!(envelope.focused_pane_id, Some("pane-1".to_string()));
        assert!(envelope.updated_at > 0);
    }

    // --- A.2 depth validation -----------------------------------------

    fn layout_full_3x2_grid() -> LayoutNode {
        // H(V(p1, p4), H(V(p2, p5), V(p3, p6))) · 3 columns × 2 rows
        LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 1.0 / 3.0,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(solo_layout("p1")),
                second: Box::new(solo_layout("p4")),
            }),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(LayoutNode::Split {
                    direction: SplitDir::Vertical,
                    ratio: 0.5,
                    first: Box::new(solo_layout("p2")),
                    second: Box::new(solo_layout("p5")),
                }),
                second: Box::new(LayoutNode::Split {
                    direction: SplitDir::Vertical,
                    ratio: 0.5,
                    first: Box::new(solo_layout("p3")),
                    second: Box::new(solo_layout("p6")),
                }),
            }),
        }
    }

    fn layout_depth_6_alternating() -> LayoutNode {
        // depth = 6 · 应在 root 的 first 子树下再加一层
        LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(LayoutNode::Split {
                    direction: SplitDir::Horizontal,
                    ratio: 0.5,
                    first: Box::new(LayoutNode::Split {
                        direction: SplitDir::Vertical,
                        ratio: 0.5,
                        first: Box::new(LayoutNode::Split {
                            direction: SplitDir::Horizontal,
                            ratio: 0.5,
                            first: Box::new(LayoutNode::Split {
                                direction: SplitDir::Vertical,
                                ratio: 0.5,
                                first: Box::new(solo_layout("p1")),
                                second: Box::new(solo_layout("p2")),
                            }),
                            second: Box::new(solo_layout("p3")),
                        }),
                        second: Box::new(solo_layout("p4")),
                    }),
                    second: Box::new(solo_layout("p5")),
                }),
                second: Box::new(solo_layout("p6")),
            }),
            second: Box::new(solo_layout("p7")),
        }
    }

    #[test]
    fn full_3x2_grid_passes_validation() {
        let layout = layout_full_3x2_grid();
        assert_eq!(layout.grid_dimensions(), (3, 2));
        layout.validate_layout().unwrap();
    }

    #[test]
    fn depth_6_fails_validation() {
        let layout = layout_depth_6_alternating();
        let result = layout.validate_layout();
        assert!(matches!(
            result,
            Err(PaneLayoutError::MaxDepthExceeded {
                max_depth: 5,
                attempted_depth: 6
            })
        ));
    }

    // --- A.3 ratio clamp ----------------------------------------------

    #[test]
    fn ratio_005_passes_004_fails() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.05,
            first: Box::new(solo_layout("p1")),
            second: Box::new(solo_layout("p2")),
        };
        layout.validate_layout().unwrap();

        let bad = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.049,
            first: Box::new(solo_layout("p1")),
            second: Box::new(solo_layout("p2")),
        };
        let result = bad.validate_layout();
        assert!(matches!(
            result,
            Err(PaneLayoutError::InvalidRatio {
                ratio: 0.049,
                min: 0.05,
                max: 0.95
            })
        ));
    }

    #[test]
    fn ratio_095_passes_096_fails() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.95,
            first: Box::new(solo_layout("p1")),
            second: Box::new(solo_layout("p2")),
        };
        layout.validate_layout().unwrap();

        let bad = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.951,
            first: Box::new(solo_layout("p1")),
            second: Box::new(solo_layout("p2")),
        };
        let result = bad.validate_layout();
        assert!(matches!(
            result,
            Err(PaneLayoutError::InvalidRatio {
                ratio: 0.951,
                min: 0.05,
                max: 0.95
            })
        ));
    }

    // --- A.4 close collapse -------------------------------------------

    #[test]
    fn close_nested_collapse_h_v() {
        // H(A, V(B, C)) 删 B → H(A, C)
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("A")),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.6,
                first: Box::new(solo_layout("B")),
                second: Box::new(solo_layout("C")),
            }),
        };
        let new_layout = close_pane_in_layout(&layout, "B").unwrap();
        let expected = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("A")),
            second: Box::new(solo_layout("C")),
        };
        assert_eq!(new_layout, expected);
    }

    // --- A.5 same-direction consecutive split -------------------------

    #[test]
    fn same_direction_consecutive_split_allowed() {
        // H(A, H(B, C)) 在 v0.2 应 PASS
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("A")),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Horizontal,
                ratio: 0.5,
                first: Box::new(solo_layout("B")),
                second: Box::new(solo_layout("C")),
            }),
        };
        layout.validate_layout().unwrap();
    }

    // --- A.6 empty / missing / duplicate pane id ----------------------

    #[test]
    fn empty_pane_id_rejected() {
        let layout = LayoutNode::Single {
            pane_id: "".to_string(),
        };
        let result = layout.validate_layout();
        assert!(matches!(
            result,
            Err(PaneLayoutError::PaneNotFound { pane_id })
            if pane_id == "(empty)"
        ));
    }

    #[test]
    fn duplicate_pane_id_rejected() {
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("same")),
            second: Box::new(solo_layout("same")),
        };
        let result = layout.validate_layout();
        assert!(matches!(
            result,
            Err(PaneLayoutError::DuplicatePane { pane_id })
            if pane_id == "same"
        ));
    }

    // --- B.1 preset apply pure function -------------------------------

    #[test]
    fn preset_solo_builds_single() {
        let panes = vec!["p1".to_string()];
        let layout = build_layout_for_preset(LayoutPresetKind::Solo, &panes).unwrap();
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn preset_dual_ai_builds_horizontal() {
        let panes = vec!["claude".to_string(), "codex".to_string()];
        let layout = build_layout_for_preset(LayoutPresetKind::DualAi, &panes).unwrap();
        assert_eq!(layout, horizontal_2pane("claude", "codex"));
    }

    #[test]
    fn preset_triple_review_builds_nested() {
        let panes = vec!["ai".to_string(), "runner".to_string(), "log".to_string()];
        let layout = build_layout_for_preset(LayoutPresetKind::TripleReview, &panes).unwrap();
        let expected = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("ai")),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(solo_layout("runner")),
                second: Box::new(solo_layout("log")),
            }),
        };
        assert_eq!(layout, expected);
    }

    #[test]
    fn preset_quad_builds_quad() {
        let panes = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let layout = build_layout_for_preset(LayoutPresetKind::Quad, &panes).unwrap();
        let expected = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(solo_layout("a")),
                second: Box::new(solo_layout("b")),
            }),
            second: Box::new(LayoutNode::Split {
                direction: SplitDir::Vertical,
                ratio: 0.5,
                first: Box::new(solo_layout("c")),
                second: Box::new(solo_layout("d")),
            }),
        };
        assert_eq!(layout, expected);
    }

    #[test]
    fn preset_insufficient_panes_fails() {
        let panes = vec!["only".to_string()];
        let result = build_layout_for_preset(LayoutPresetKind::DualAi, &panes);
        assert!(matches!(
            result,
            Err(PaneLayoutError::PresetApplyFailed {
                preset: LayoutPresetKind::DualAi,
                ..
            })
        ));
    }

    // --- SmartLayoutKind → LayoutPresetKind 转换 ----------------------

    #[test]
    fn smart_layout_kind_into_layout_preset_kind() {
        assert_eq!(
            LayoutPresetKind::from(SmartLayoutKind::Solo),
            LayoutPresetKind::Solo
        );
        assert_eq!(
            LayoutPresetKind::from(SmartLayoutKind::AiAndRunner),
            LayoutPresetKind::AiAndRunner
        );
    }

    // --- PaneLayoutError → PaneError 转换 -----------------------------

    #[test]
    fn pane_layout_error_into_pane_error() {
        let err = PaneLayoutError::MaxDepthExceeded {
            max_depth: 5,
            attempted_depth: 6,
        };
        let pane_err: PaneError = err.into();
        assert!(matches!(pane_err, PaneError::InvalidLayout(_)));
    }
}
