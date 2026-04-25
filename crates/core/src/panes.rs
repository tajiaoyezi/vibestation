//! Pane storage · MVP-05 Phase A.
//!
//! This module owns the `panes` table DAO plus the ts-rs IPC contract types for
//! Pane layout state. IPC commands are intentionally deferred to MVP-05 Phase B.

use crate::db::{DbError, DbPool};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MAX_LAYOUT_SPLIT_DEPTH: usize = 2;
pub const MAX_LAYOUT_PANES: usize = 4;

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

    pub fn validate_mvp_05(&self) -> Result<(), PaneError> {
        let pane_count = self.validate_inner(None, 0)?;
        if pane_count > MAX_LAYOUT_PANES {
            return Err(PaneError::InvalidLayout(format!(
                "pane count {pane_count} exceeds MVP-05 max {MAX_LAYOUT_PANES}"
            )));
        }
        Ok(())
    }

    fn validate_inner(
        &self,
        parent_direction: Option<SplitDir>,
        split_depth: usize,
    ) -> Result<usize, PaneError> {
        match self {
            Self::Single { .. } => Ok(1),
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let next_depth = split_depth + 1;
                if next_depth > MAX_LAYOUT_SPLIT_DEPTH {
                    return Err(PaneError::InvalidLayout(format!(
                        "split depth {next_depth} exceeds MVP-05 max {MAX_LAYOUT_SPLIT_DEPTH}"
                    )));
                }
                if parent_direction == Some(direction.clone()) {
                    return Err(PaneError::InvalidLayout(format!(
                        "nested {direction:?} split would create a 3-pane row/column"
                    )));
                }
                if !(*ratio > 0.0 && *ratio < 1.0) {
                    return Err(PaneError::InvalidLayout(format!(
                        "split ratio must be between 0 and 1, got {ratio}"
                    )));
                }

                let first_count = first.validate_inner(Some(direction.clone()), next_depth)?;
                let second_count = second.validate_inner(Some(direction.clone()), next_depth)?;
                Ok(first_count + second_count)
            }
        }
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

#[derive(Debug, thiserror::Error)]
pub enum PaneError {
    #[error("pane not found: {0}")]
    NotFound(String),
    #[error("invalid pane layout: {0}")]
    InvalidLayout(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

/// MVP-05 §C · Smart Layout 预设种类。
///
/// - `Solo`：保留当前聚焦 Pane · 关闭其他所有 Pane。
/// - `AiAndRunner`：保留当前聚焦 Pane + 第一个非聚焦 Pane · 强制右分屏 50/50 · 关闭其他。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmartLayoutKind {
    Solo,
    AiAndRunner,
}

/// MVP-05 §H.2 · 在指定 Pane 处右分 / 下分一个新 Pane（pure function · 不修改原 layout）。
///
/// 在 `layout` 中找到 `parent_pane_id` 对应的 `Single` 节点 · 用 `Split { direction, ratio: 0.5,
/// first: Single(parent), second: Single(new) }` 替换。新布局通过 `validate_mvp_05()` 校验：
/// 若深度 / 同向嵌套 / pane 数量超限 · 返回 `Err(PaneError::InvalidLayout(_))`。
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
/// - `Err(PaneError::InvalidLayout)`：split 会导致深度 / 嵌套 / 数量超限 · 或 `new_pane_id`
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
    let new_layout = replace_single(
        layout,
        parent_pane_id,
        LayoutNode::Split {
            direction,
            ratio: 0.5,
            first: Box::new(LayoutNode::Single {
                pane_id: parent_pane_id.to_string(),
            }),
            second: Box::new(LayoutNode::Single {
                pane_id: new_pane_id,
            }),
        },
    );
    new_layout.validate_mvp_05()?;
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
    new_layout.validate_mvp_05()?;
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
/// - `new_ratio`：新比例 · 必须在 (0.0, 1.0) 开区间
///
/// 出参：
/// - `Ok(LayoutNode)`：ratio 已更新的新布局
/// - `Err(PaneError::NotFound)`：找不到匹配的 Split 节点
/// - `Err(PaneError::InvalidLayout)`：`new_ratio` 不在 (0, 1) 区间
pub fn update_split_ratio(
    layout: &LayoutNode,
    parent_pane_id: &str,
    new_ratio: f32,
) -> Result<LayoutNode, PaneError> {
    if !(new_ratio > 0.0 && new_ratio < 1.0) {
        return Err(PaneError::InvalidLayout(format!(
            "split ratio must be between 0 and 1, got {new_ratio}"
        )));
    }
    update_ratio_inner(layout, parent_pane_id, new_ratio)
        .ok_or_else(|| PaneError::NotFound(parent_pane_id.to_string()))
}

/// MVP-05 §C · 应用 Smart Layout 预设（pure function）。
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
            new_layout.validate_mvp_05()?;
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
            new_layout.validate_mvp_05()?;
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
fn collect_pane_ids(layout: &LayoutNode) -> Vec<String> {
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

/// 把 layout 中所有 `Single { pane_id == target }` 替换为 `replacement`（递归 deep-copy）。
fn replace_single(layout: &LayoutNode, target: &str, replacement: LayoutNode) -> LayoutNode {
    match layout {
        LayoutNode::Single { pane_id } if pane_id == target => replacement,
        LayoutNode::Single { pane_id } => LayoutNode::Single {
            pane_id: pane_id.clone(),
        },
        LayoutNode::Split {
            direction,
            ratio,
            first,
            second,
        } => LayoutNode::Split {
            direction: direction.clone(),
            ratio: *ratio,
            first: Box::new(replace_single(first, target, replacement.clone())),
            second: Box::new(replace_single(second, target, replacement)),
        },
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
                return Some(LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: *ratio,
                    first: Box::new(new_first),
                    second: second.clone(),
                });
            }
            if layout_contains_pane(second, pane_id) {
                let new_second =
                    remove_pane(second, pane_id).expect("pane known to exist in second subtree");
                return Some(LayoutNode::Split {
                    direction: direction.clone(),
                    ratio: *ratio,
                    first: first.clone(),
                    second: Box::new(new_second),
                });
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
        layout.validate_mvp_05().unwrap();
    }

    #[test]
    fn h2_horizontal_2pane_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_horizontal_2pane();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 2);
        layout.validate_mvp_05().unwrap();
    }

    #[test]
    fn h2_vertical_2pane_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_vertical_2pane();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 2);
        layout.validate_mvp_05().unwrap();
    }

    #[test]
    fn h2_2x2_layout_is_valid() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_2x2_layout();
        let layout = current_layout(&conn);
        assert_eq!(layout.pane_count(), 4);
        layout.validate_mvp_05().unwrap();
    }

    #[test]
    fn h2_invalid_3horizontal_layout_is_rejected_and_original_layout_unchanged() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_horizontal_2pane();
        let before = current_layout(&conn);
        let invalid = mvp_05_helpers::create_fixture_invalid_3horizontal();
        let result = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(invalid[0].clone()),
            second: Box::new(invalid[1].clone()),
        }
        .validate_mvp_05();
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        assert_eq!(current_layout(&conn), before);
    }

    #[test]
    fn h2_invalid_3vertical_layout_is_rejected_and_original_layout_unchanged() {
        let (_dir, conn) = mvp_05_helpers::create_fixture_vertical_2pane();
        let before = current_layout(&conn);
        let invalid = mvp_05_helpers::create_fixture_invalid_3vertical();
        let result = LayoutNode::Split {
            direction: SplitDir::Vertical,
            ratio: 0.5,
            first: Box::new(invalid[0].clone()),
            second: Box::new(invalid[1].clone()),
        }
        .validate_mvp_05();
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
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
        result.validate_mvp_05().unwrap();
        assert_eq!(result, horizontal_2pane("p1", "p2"));
        // 原 layout 未被修改（pure function 不变性）
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn split_layout_vertical_2_pane_legal() {
        let layout = solo_layout("p1");
        let result = split_layout(&layout, "p1", SplitDir::Vertical, "p2".to_string()).unwrap();
        assert_eq!(result.pane_count(), 2);
        result.validate_mvp_05().unwrap();
        assert_eq!(result, vertical_2pane("p1", "p2"));
        assert_eq!(layout, solo_layout("p1"));
    }

    #[test]
    fn split_layout_2x2_legal() {
        // 起点：水平 2 pane → 在右 pane (p2) 下分一个新 p3 → 应得 horizontal { left=p1, right=vertical{p2, p3}}
        let layout = horizontal_2pane("p1", "p2");
        let result = split_layout(&layout, "p2", SplitDir::Vertical, "p3".to_string()).unwrap();
        assert_eq!(result.pane_count(), 3);
        result.validate_mvp_05().unwrap();
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
        result_2x2.validate_mvp_05().unwrap();
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

    // --- §H.2 case 5-6 · 非法 split ----------------------------------

    #[test]
    fn split_layout_3_horizontal_illegal() {
        // 在水平 2 pane 的 right pane 上再右分 → 同向嵌套 · 触发 InvalidLayout
        let layout = horizontal_2pane("p1", "p2");
        let result = split_layout(&layout, "p2", SplitDir::Horizontal, "p3".to_string());
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // 原 layout 不变
        assert_eq!(layout, horizontal_2pane("p1", "p2"));
    }

    #[test]
    fn split_layout_3_vertical_illegal() {
        // 在垂直 2 pane 的 bottom pane 上再下分 → 同向嵌套 · 触发 InvalidLayout
        let layout = vertical_2pane("p1", "p2");
        let result = split_layout(&layout, "p2", SplitDir::Vertical, "p3".to_string());
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        assert_eq!(layout, vertical_2pane("p1", "p2"));
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
        // case 2 · 深度超限 · pure function 返回 Err · layout 不变
        let layout = LayoutNode::Split {
            direction: SplitDir::Horizontal,
            ratio: 0.5,
            first: Box::new(solo_layout("p1")),
            second: Box::new(vertical_2pane("p2", "p3")),
        };
        let original = layout.clone();
        // 在 p3 上再下分（已经在 vertical split 的 second 子树 · 再下分会触发 depth 3）
        // 因为 split_layout 内部会先校验同向嵌套；这里 vertical 2pane 的 p3 再 vertical split
        // 会触发"3 vertical illegal"分支（同向嵌套）· 也属于 InvalidLayout
        let result = split_layout(&layout, "p3", SplitDir::Vertical, "p4".to_string());
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        assert_eq!(layout, original);

        // 反向：在 p3 上 horizontal 分 → 会让 vertical{p2, horizontal{p3, p4}} · depth 3 → 超限
        let result_h = split_layout(&layout, "p3", SplitDir::Horizontal, "p4".to_string());
        assert!(matches!(result_h, Err(PaneError::InvalidLayout(_))));
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
        // ratio = 0 · 区间外
        let result = update_split_ratio(&layout, "p1", 0.0);
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // ratio = 1 · 区间外
        let result = update_split_ratio(&layout, "p1", 1.0);
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
}
