//! Pane service · MVP-14 Phase A LayoutEnvelope
//!
//! 把 [`crate::panes`] 的 4 layout pure functions 包成事务（panes 表 + tabs.layout +
//! tabs.focused_pane_id 原子写）· 供 Tauri IPC 层（`crates/app/src/lib.rs`）调用。
//!
//! §H.3 atomicity：任何中间步骤失败 → 整个 transaction rollback · 不留半成品（部分 panes
//! 已插但 layout 没更新 等）· 由 rusqlite `Transaction` 保证。
//!
//! ## 6 commands
//!
//! - [`apply_pane_init_for_tab`] · 为 layout 为空（paneId == ""）的 tab 初始化第一个 Pane · idempotent
//! - [`apply_pane_split`] · §H.2 split + INSERT panes + UPDATE layout/focused_pane_id
//! - [`apply_pane_close`] · §H.2 close + DELETE panes + UPDATE layout/focused_pane_id
//! - [`apply_pane_focus`] · 仅 UPDATE focused_pane_id（无 layout 改动）
//! - [`apply_layout_preset`] · §C smart layout + DELETE 关闭的 panes + UPDATE layout
//! - [`apply_split_ratio_update`] · §D ratio 调整 + UPDATE layout（无 panes 改动）
//!
//! ## MVP-14 Phase A · LayoutEnvelope 迁移
//!
//! - 所有 layout 读写统一使用 [`LayoutEnvelope`]（v1 envelope 格式）
//! - 读取时兼容 MVP-05 裸 [`LayoutNode`]（legacy fallback）
//! - 写入时输出 v1 envelope JSON（含 `version`、`updated_at`、`focused_pane_id`）

use crate::db::{DbError, DbPool};
use crate::panes::{
    apply_smart_layout, build_layout_for_preset, close_pane_in_layout, collect_pane_ids,
    find_split_ratio, split_layout, update_split_ratio, LayoutApplyAdvancedRequest,
    LayoutApplyRequest, LayoutApplyResult, LayoutEnvelope, LayoutNode, PaneCloseRequest,
    PaneCreateRequest, PaneError, PaneFocusRequest, PaneListResponse, PaneMaximizeRequest,
    PaneMaximizeResult, PaneNavDirection, PaneNavigateRequest, PaneNavigateResult,
    PaneResizeStepRequest, PanesDao, SmartLayoutKind, SplitRatioUpdateRequest, MAX_SPLIT_RATIO,
    MIN_SPLIT_RATIO,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// 为 tab 初始化第一个 Pane（layout 为 `Single { paneId: "" }` 时调用 · idempotent）。
///
/// 触发场景：MVP-04 创建 tab 时 layout 默认为空 paneId · MVP-05 Phase C 前端在第一次激活 tab
/// 时调用此命令 · 把 tab 切换到 Pane 模式（创建一个 Pane row + 改 layout 为指向 Pane 的 Single
/// + 设置 focused_pane_id）。
///
/// 若 tab 已有 Pane（layout.paneId 非空 + panes 表有对应行）· 返回当前 PaneListResponse · 不重复创建。
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneInitRequest {
    pub tab_id: String,
    pub shell: String,
    pub cwd: String,
}

pub fn apply_pane_init_for_tab(
    pool: &DbPool,
    req: &PaneInitRequest,
) -> Result<PaneListResponse, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let current_envelope = read_tab_layout(&tx, &req.tab_id)?;

    // idempotent: 已初始化过则直接返回当前状态
    if let LayoutNode::Single { pane_id } = &current_envelope.root {
        if !pane_id.is_empty() {
            drop(tx);
            return read_pane_list(pool, &req.tab_id);
        }
    }

    let new_pane_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().timestamp();
    let new_layout = LayoutNode::Single {
        pane_id: new_pane_id.clone(),
    };
    let envelope = LayoutEnvelope::from_legacy_node(new_layout, Some(new_pane_id.clone()));

    tx.execute(
        "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
         VALUES (?1, ?2, ?3, ?4, '[]', ?5)",
        params![new_pane_id, req.tab_id, req.shell, req.cwd, created_at],
    )
    .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    write_tab_layout(&tx, &req.tab_id, &envelope)?;
    write_focused_pane(&tx, &req.tab_id, Some(&new_pane_id))?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &req.tab_id)
}

/// 在 [`PaneCreateRequest::parent_pane_id`] 处发起一次 split · 生成新 Pane 并写入 panes 表 ·
/// 同步更新 `tabs.layout` + `tabs.focused_pane_id`（新 Pane 获得焦点）。
///
/// 全过程在单个 rusqlite transaction 内 · 任意一步 fail → 全部 rollback。
pub fn apply_pane_split(
    pool: &DbPool,
    req: &PaneCreateRequest,
) -> Result<PaneListResponse, PaneError> {
    let new_pane_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().timestamp();
    // MVP-05 §H.2: cwd inherited from tab default · Phase C 接 UI 时再做 parent pane cwd 继承
    let cwd = read_tab_cwd(pool, &req.tab_id)?;

    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let envelope = read_tab_layout_envelope(&tx, &req.tab_id)?;
    let new_layout = split_layout(
        &envelope.root,
        &req.parent_pane_id,
        req.direction.clone(),
        new_pane_id.clone(),
    )?;
    new_layout.validate_layout()?;

    tx.execute(
        "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
         VALUES (?1, ?2, ?3, ?4, '[]', ?5)",
        params![new_pane_id, req.tab_id, req.shell, cwd, created_at],
    )
    .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let new_envelope = LayoutEnvelope::from_legacy_node(new_layout, Some(new_pane_id.clone()));
    write_tab_layout(&tx, &req.tab_id, &new_envelope)?;
    write_focused_pane(&tx, &req.tab_id, Some(&new_pane_id))?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &req.tab_id)
}

/// 关闭 [`PaneCloseRequest::pane_id`] 对应的 Pane · 删除 panes 行 · 更新 `tabs.layout` ·
/// 把 `focused_pane_id` 切到 layout 中的 first pane（若 layout 为空则置 NULL）。
pub fn apply_pane_close(
    pool: &DbPool,
    req: &PaneCloseRequest,
) -> Result<PaneListResponse, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let tab_id = read_pane_tab_id(&tx, &req.pane_id)?;
    let envelope = read_tab_layout_envelope(&tx, &tab_id)?;
    let new_layout = close_pane_in_layout(&envelope.root, &req.pane_id)?;
    new_layout.validate_layout()?;

    let rows = tx
        .execute("DELETE FROM panes WHERE pane_id = ?1", [&req.pane_id])
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
    if rows == 0 {
        return Err(PaneError::NotFound(req.pane_id.clone()));
    }

    let new_envelope =
        LayoutEnvelope::from_legacy_node(new_layout.clone(), first_pane_id(&new_layout));
    write_tab_layout(&tx, &tab_id, &new_envelope)?;
    let next_focus = first_pane_id(&new_layout);
    write_focused_pane(&tx, &tab_id, next_focus.as_deref())?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &tab_id)
}

/// 仅更新 `tabs.focused_pane_id` · 不动 layout / panes。
///
/// 若 `focused_pane_id` 不在当前 layout 内 · 返回 [`PaneError::NotFound`]。
pub fn apply_pane_focus(
    pool: &DbPool,
    req: &PaneFocusRequest,
) -> Result<PaneListResponse, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let envelope = read_tab_layout_envelope(&tx, &req.tab_id)?;
    let pane_ids = collect_pane_ids(&envelope.root);
    if !pane_ids.contains(&req.focused_pane_id) {
        return Err(PaneError::NotFound(req.focused_pane_id.clone()));
    }
    write_focused_pane(&tx, &req.tab_id, Some(&req.focused_pane_id))?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &req.tab_id)
}

/// 应用 [`crate::panes::SmartLayoutKind`] 预设 · 生成新 layout + 关闭多余 panes（DELETE）。
///
/// `req.preset`：
/// - `"solo"` → [`SmartLayoutKind::Solo`]
/// - `"aiAndRunner"` → [`SmartLayoutKind::AiAndRunner`]
/// - 其他 → [`PaneError::InvalidLayout`]
///
/// 注：`AiAndRunner` 在单 Pane layout 上会返回 [`PaneError::InvalidLayout`] · caller 应先 spawn
/// 第二个 Pane（与 [`apply_pane_split`] 一致 · caller 指定 shell 触发）。
pub fn apply_layout_preset(
    pool: &DbPool,
    req: &LayoutApplyRequest,
) -> Result<PaneListResponse, PaneError> {
    let kind = match req.preset.as_str() {
        "solo" => SmartLayoutKind::Solo,
        "aiAndRunner" => SmartLayoutKind::AiAndRunner,
        other => {
            return Err(PaneError::InvalidLayout(format!(
                "unknown preset '{other}' · expected 'solo' or 'aiAndRunner'"
            )))
        }
    };

    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let focused_pane_id = read_focused_pane(&tx, &req.tab_id)?.ok_or_else(|| {
        PaneError::InvalidLayout(format!("tab {} has no focused pane", req.tab_id))
    })?;
    let envelope = read_tab_layout_envelope(&tx, &req.tab_id)?;

    let (new_layout, closed_pane_ids) = apply_smart_layout(&envelope.root, kind, &focused_pane_id)?;
    new_layout.validate_layout()?;

    for pane_id in &closed_pane_ids {
        tx.execute("DELETE FROM panes WHERE pane_id = ?1", [pane_id])
            .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
    }

    let new_envelope = LayoutEnvelope::from_legacy_node(new_layout, Some(focused_pane_id));
    write_tab_layout(&tx, &req.tab_id, &new_envelope)?;
    // focused_pane_id 不变（preset 保留聚焦 Pane）

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &req.tab_id)
}

/// 调整 split 节点的 ratio · 仅 UPDATE `tabs.layout` · 无 panes 改动。
pub fn apply_split_ratio_update(
    pool: &DbPool,
    req: &SplitRatioUpdateRequest,
) -> Result<PaneListResponse, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let tab_id = read_pane_tab_id(&tx, &req.pane_id)?;
    let envelope = read_tab_layout_envelope(&tx, &tab_id)?;
    let new_layout = update_split_ratio(&envelope.root, &req.pane_id, req.new_ratio)?;
    new_layout.validate_layout()?;

    let new_envelope =
        LayoutEnvelope::from_legacy_node(new_layout, envelope.focused_pane_id.clone());
    write_tab_layout(&tx, &tab_id, &new_envelope)?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &tab_id)
}

/// MVP-14 · 应用高级 Smart Layout 预设（支持 DualAi / TripleReview / Quad）。
///
/// 与 `apply_layout_preset` 的区别：
/// - 输入用 [`LayoutPresetKind`] enum 代替字符串
/// - 支持 5 个预设（Solo / AiAndRunner / DualAi / TripleReview / Quad）
/// - 返回复用 / 创建 / 关闭的 pane ids 清单
/// - `preserve_instances` 为 true 时优先按 pane content identity 复用（当前实现按 id 顺序复用）
pub fn apply_layout_preset_advanced(
    pool: &DbPool,
    req: &LayoutApplyAdvancedRequest,
) -> Result<LayoutApplyResult, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let focused_pane_id = read_focused_pane(&tx, &req.tab_id)?.ok_or_else(|| {
        PaneError::InvalidLayout(format!("tab {} has no focused pane", req.tab_id))
    })?;
    let envelope = read_tab_layout_envelope(&tx, &req.tab_id)?;

    // 收集当前所有 pane id（按 focus + DFS 顺序）
    let available_panes = collect_pane_ids(&envelope.root);
    let new_layout = build_layout_for_preset(req.preset, &available_panes)?;
    new_layout.validate_layout()?;

    // 计算 reused / created / closed（简单实现：复用所有现有 pane，不创建/关闭）
    let reused_pane_ids = available_panes.clone();
    let created_pane_ids: Vec<String> = Vec::new();
    let closed_pane_ids: Vec<String> = Vec::new();

    let new_envelope = LayoutEnvelope::from_legacy_node(new_layout, Some(focused_pane_id));
    write_tab_layout(&tx, &req.tab_id, &new_envelope)?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let response = read_pane_list(pool, &req.tab_id)?;
    Ok(LayoutApplyResult {
        response,
        reused_pane_ids,
        created_pane_ids,
        closed_pane_ids,
    })
}

/// MVP-14 · 方向键导航：从 `from_pane_id` 向 `direction` 方向找几何相邻 pane。
///
/// 当前实现：按 DFS 顺序找下一个/上一个 pane（简化版几何相邻）。
/// Phase C 会替换为真正的基于 DOMRect 投影重叠算法。
pub fn apply_pane_navigate(
    pool: &DbPool,
    req: &PaneNavigateRequest,
) -> Result<PaneNavigateResult, PaneError> {
    let conn = pool.get().map_err(DbError::from)?;
    let envelope = read_tab_layout_envelope(&conn, &req.tab_id)?;

    let pane_ids = collect_pane_ids(&envelope.root);
    let Some(current_idx) = pane_ids.iter().position(|id| id == &req.from_pane_id) else {
        return Ok(PaneNavigateResult { to_pane_id: None });
    };

    let to_pane_id = match req.direction {
        PaneNavDirection::Left | PaneNavDirection::Up => {
            if current_idx > 0 {
                Some(pane_ids[current_idx - 1].clone())
            } else {
                None
            }
        }
        PaneNavDirection::Right | PaneNavDirection::Down => {
            if current_idx + 1 < pane_ids.len() {
                Some(pane_ids[current_idx + 1].clone())
            } else {
                None
            }
        }
    };

    Ok(PaneNavigateResult { to_pane_id })
}

/// MVP-14 · 临时最大化 toggle（session-only，不写 DB）。
///
/// 当前实现：返回当前 layout 作为 restore 候选；真正的 maximize state 由 frontend 维护。
/// 若 `toggle == false` 或当前未最大化，返回 `maximized: false`；若 `toggle == true`，返回
/// `maximized: true` 与当前 layout 供 frontend 进入 maximize 模式。
pub fn apply_pane_maximize(
    pool: &DbPool,
    req: &PaneMaximizeRequest,
) -> Result<PaneMaximizeResult, PaneError> {
    let conn = pool.get().map_err(DbError::from)?;
    let envelope = read_tab_layout_envelope(&conn, &req.tab_id)?;

    if req.toggle {
        Ok(PaneMaximizeResult {
            maximized: true,
            restored_layout: Some(envelope.root.clone()),
            restored_focused_pane_id: envelope.focused_pane_id.clone(),
        })
    } else {
        Ok(PaneMaximizeResult {
            maximized: false,
            restored_layout: Some(envelope.root.clone()),
            restored_focused_pane_id: envelope.focused_pane_id.clone(),
        })
    }
}

/// MVP-14 · 键盘步进调整 split ratio（±5% step）。
///
/// 找到 `pane_id` 所在 Split 节点，按 `direction` 和 `step_ratio` 调整 ratio。
/// `step_ratio` 为正时向 `direction` 方向增大 first 子树比例。
pub fn apply_pane_resize_step(
    pool: &DbPool,
    req: &PaneResizeStepRequest,
) -> Result<PaneListResponse, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let envelope = read_tab_layout_envelope(&tx, &req.tab_id)?;
    let current_ratio = find_split_ratio(&envelope.root, &req.pane_id).unwrap_or(0.5);
    let new_ratio = (current_ratio + req.step_ratio).clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO);

    let new_layout = update_split_ratio(&envelope.root, &req.pane_id, new_ratio)?;
    new_layout.validate_layout()?;

    let new_envelope =
        LayoutEnvelope::from_legacy_node(new_layout, envelope.focused_pane_id.clone());
    write_tab_layout(&tx, &req.tab_id, &new_envelope)?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &req.tab_id)
}

// ---- 内部 helpers ----

fn read_tab_layout(conn: &rusqlite::Connection, tab_id: &str) -> Result<LayoutEnvelope, PaneError> {
    let layout_json: String = conn
        .query_row(
            "SELECT layout FROM tabs WHERE tab_id = ?1",
            [tab_id],
            |row| row.get(0),
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => PaneError::NotFound(tab_id.to_string()),
            other => PaneError::Db(DbError::Query(other.to_string())),
        })?;

    // 兼容初始空 tab（paneId == ""）· 不经过 validate_layout
    if let Ok(node) = serde_json::from_str::<LayoutNode>(&layout_json) {
        if let LayoutNode::Single { pane_id } = &node {
            if pane_id.is_empty() {
                return Ok(LayoutEnvelope::from_legacy_node(node, None));
            }
        }
    }

    LayoutEnvelope::try_from_json(&layout_json)
        .map_err(|e| PaneError::InvalidLayout(format!("layout envelope parse (tab={tab_id}): {e}")))
}

fn read_tab_layout_envelope(
    conn: &rusqlite::Connection,
    tab_id: &str,
) -> Result<LayoutEnvelope, PaneError> {
    let layout_json: String = conn
        .query_row(
            "SELECT layout FROM tabs WHERE tab_id = ?1",
            [tab_id],
            |row| row.get(0),
        )
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => PaneError::NotFound(tab_id.to_string()),
            other => PaneError::Db(DbError::Query(other.to_string())),
        })?;
    LayoutEnvelope::try_from_json(&layout_json)
        .map_err(|e| PaneError::InvalidLayout(format!("layout envelope parse (tab={tab_id}): {e}")))
}

fn write_tab_layout(
    tx: &rusqlite::Transaction<'_>,
    tab_id: &str,
    envelope: &LayoutEnvelope,
) -> Result<(), PaneError> {
    let layout_json = envelope
        .to_json()
        .map_err(|e| PaneError::InvalidLayout(format!("layout envelope serialize: {e}")))?;
    let rows = tx
        .execute(
            "UPDATE tabs SET layout = ?1 WHERE tab_id = ?2",
            params![layout_json, tab_id],
        )
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
    if rows == 0 {
        return Err(PaneError::NotFound(tab_id.to_string()));
    }
    Ok(())
}

fn write_focused_pane(
    tx: &rusqlite::Transaction<'_>,
    tab_id: &str,
    focused_pane_id: Option<&str>,
) -> Result<(), PaneError> {
    tx.execute(
        "UPDATE tabs SET focused_pane_id = ?1 WHERE tab_id = ?2",
        params![focused_pane_id, tab_id],
    )
    .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
    Ok(())
}

fn read_focused_pane(
    conn: &rusqlite::Connection,
    tab_id: &str,
) -> Result<Option<String>, PaneError> {
    conn.query_row(
        "SELECT focused_pane_id FROM tabs WHERE tab_id = ?1",
        [tab_id],
        |row| row.get(0),
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => PaneError::NotFound(tab_id.to_string()),
        other => PaneError::Db(DbError::Query(other.to_string())),
    })
}

fn read_pane_tab_id(conn: &rusqlite::Connection, pane_id: &str) -> Result<String, PaneError> {
    conn.query_row(
        "SELECT tab_id FROM panes WHERE pane_id = ?1",
        [pane_id],
        |row| row.get(0),
    )
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => PaneError::NotFound(pane_id.to_string()),
        other => PaneError::Db(DbError::Query(other.to_string())),
    })
}

fn read_tab_cwd(pool: &DbPool, tab_id: &str) -> Result<String, PaneError> {
    let conn = pool.get().map_err(DbError::from)?;
    conn.query_row("SELECT cwd FROM tabs WHERE tab_id = ?1", [tab_id], |row| {
        row.get(0)
    })
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => PaneError::NotFound(tab_id.to_string()),
        other => PaneError::Db(DbError::Query(other.to_string())),
    })
}

fn read_pane_list(pool: &DbPool, tab_id: &str) -> Result<PaneListResponse, PaneError> {
    let panes = PanesDao::list_by_tab(pool, tab_id)?;
    let conn = pool.get().map_err(DbError::from)?;
    let envelope = read_tab_layout_envelope(&conn, tab_id)?;
    // focused_pane_id 从独立列读取（write_focused_pane 只写此列 · 不更新 layout JSON）
    // 若独立列无值 · fallback 到 envelope JSON 里的值（兼容旧数据）
    let focused_pane_id = read_focused_pane(&conn, tab_id)?.or(envelope.focused_pane_id);
    Ok(PaneListResponse {
        panes,
        layout: envelope.root,
        focused_pane_id,
    })
}

// ---- workspace-level helpers ----

/// 迁移整个 workspace 的所有 tab layout 从 legacy 到 v1 envelope。
///
/// 遍历 tabs 表 · 读取 layout JSON · 用 [`LayoutEnvelope::try_from_json`] 解析（含 legacy fallback）
/// · 再写回数据库 · 实现原地迁移。
pub fn migrate_workspace_layout_state(pool: &DbPool) -> Result<usize, PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tab_ids: Vec<String> = conn
        .prepare("SELECT tab_id FROM tabs")
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?
        .query_map([], |row| row.get(0))
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    let mut migrated = 0;
    for tab_id in &tab_ids {
        let tx = conn
            .transaction()
            .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

        // 读取 legacy layout（直接作为裸字符串）
        let layout_json: String = tx
            .query_row(
                "SELECT layout FROM tabs WHERE tab_id = ?1",
                [tab_id],
                |row| row.get(0),
            )
            .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

        // 尝试解析为 envelope（含 legacy fallback）
        match LayoutEnvelope::try_from_json(&layout_json) {
            Ok(envelope) => {
                // 如果已经是 v1 envelope 格式（layout_json 以 {"version": 开头），
                // 且解析成功，说明无需迁移
                if layout_json.trim_start().starts_with("{\"version\"") {
                    drop(tx);
                    continue;
                }
                // 否则写回 v1 envelope 格式
                write_tab_layout(&tx, tab_id, &envelope)?;
                tx.commit()
                    .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
                migrated += 1;
            }
            Err(e) => {
                // 跳过无法解析的 layout（保留原样）
                drop(tx);
                eprintln!("[migrate_workspace_layout_state] skip tab_id={tab_id}: {e}");
            }
        }
    }

    Ok(migrated)
}

/// 为指定 tab 写入完整的 workspace layout state（envelope + focused_pane_id）。
///
/// 用于 workspace 恢复时一次性写入 layout + focus 状态。
pub fn write_workspace_layout_state(
    pool: &DbPool,
    tab_id: &str,
    envelope: &LayoutEnvelope,
) -> Result<(), PaneError> {
    let mut conn = pool.get().map_err(DbError::from)?;
    let tx = conn
        .transaction()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    write_tab_layout(&tx, tab_id, envelope)?;
    write_focused_pane(&tx, tab_id, envelope.focused_pane_id.as_deref())?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    Ok(())
}

fn first_pane_id(layout: &LayoutNode) -> Option<String> {
    match layout {
        LayoutNode::Single { pane_id } => Some(pane_id.clone()),
        LayoutNode::Split { first, .. } => first_pane_id(first),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::panes::{PaneState, SplitDir};
    use crate::tabs::{TabCreateRequest, TabsDao};
    use crate::workspace::WorkspaceStore;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool, String) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = db::open_pool(&db_path).unwrap();
        let ws_dir = dir.path().join("ws");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        let tab = TabsDao::create(
            &pool,
            &TabCreateRequest {
                workspace_id: ws.workspace_id,
                name: Some("Pane Service Test".to_string()),
                shell: Some("/bin/zsh".to_string()),
                cwd: Some("/tmp".to_string()),
            },
        )
        .unwrap();
        (dir, pool, tab.tab_id)
    }

    fn seed_initial_pane(pool: &DbPool, tab_id: &str, pane_id: &str) {
        PanesDao::insert(
            pool,
            PaneState {
                pane_id: pane_id.to_string(),
                tab_id: tab_id.to_string(),
                shell: "/bin/zsh".to_string(),
                cwd: "/tmp".to_string(),
                created_at: 1,
            },
        )
        .unwrap();
        // 写 layout = v1 envelope + 设置 focus
        let pool_clone = pool.clone();
        let conn = pool_clone.get().unwrap();
        let envelope = LayoutEnvelope::new_solo(pane_id);
        let layout_json = envelope.to_json().unwrap();
        conn.execute(
            "UPDATE tabs SET layout = ?1, focused_pane_id = ?2 WHERE tab_id = ?3",
            params![layout_json, pane_id, tab_id],
        )
        .unwrap();
    }

    fn assert_persisted_focus(pool: &DbPool, tab_id: &str) -> Option<String> {
        let conn = pool.get().unwrap();
        conn.query_row(
            "SELECT focused_pane_id FROM tabs WHERE tab_id = ?1",
            [tab_id],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn pane_count(pool: &DbPool, tab_id: &str) -> usize {
        PanesDao::list_by_tab(pool, tab_id).unwrap().len()
    }

    #[test]
    fn pane_init_creates_first_pane_for_empty_tab() {
        let (_dir, pool, tab_id) = setup();
        // 默认状态：tab.layout = {single, paneId: ""} · 没有 pane row
        assert_eq!(pane_count(&pool, &tab_id), 0);

        let response = apply_pane_init_for_tab(
            &pool,
            &PaneInitRequest {
                tab_id: tab_id.clone(),
                shell: "/bin/zsh".to_string(),
                cwd: "/tmp".to_string(),
            },
        )
        .unwrap();

        assert_eq!(response.panes.len(), 1);
        assert_eq!(pane_count(&pool, &tab_id), 1);
        assert!(
            matches!(response.layout, LayoutNode::Single { ref pane_id } if !pane_id.is_empty())
        );
        let pane_id = match &response.layout {
            LayoutNode::Single { pane_id } => pane_id.clone(),
            _ => panic!("expected Single layout"),
        };
        assert_eq!(
            assert_persisted_focus(&pool, &tab_id).as_deref(),
            Some(pane_id.as_str())
        );
    }

    #[test]
    fn pane_init_is_idempotent() {
        let (_dir, pool, tab_id) = setup();
        let r1 = apply_pane_init_for_tab(
            &pool,
            &PaneInitRequest {
                tab_id: tab_id.clone(),
                shell: "/bin/zsh".to_string(),
                cwd: "/tmp".to_string(),
            },
        )
        .unwrap();
        let pane_id_1 = match &r1.layout {
            LayoutNode::Single { pane_id } => pane_id.clone(),
            _ => panic!("expected Single"),
        };

        // 二次调用 · 应返回相同 PaneListResponse · 不创建新 pane
        let r2 = apply_pane_init_for_tab(
            &pool,
            &PaneInitRequest {
                tab_id: tab_id.clone(),
                shell: "/bin/zsh".to_string(),
                cwd: "/tmp".to_string(),
            },
        )
        .unwrap();
        let pane_id_2 = match &r2.layout {
            LayoutNode::Single { pane_id } => pane_id.clone(),
            _ => panic!("expected Single"),
        };
        assert_eq!(pane_id_1, pane_id_2);
        assert_eq!(pane_count(&pool, &tab_id), 1);
    }

    #[test]
    fn split_creates_pane_layout_and_focuses_new() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        let response = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();

        assert_eq!(response.panes.len(), 2);
        assert_eq!(pane_count(&pool, &tab_id), 2);
        assert!(matches!(response.layout, LayoutNode::Split { .. }));
        // focused 是新 pane
        let focus = assert_persisted_focus(&pool, &tab_id).unwrap();
        assert_ne!(focus, "p1");
    }

    #[test]
    fn split_rejects_unknown_parent() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        let result = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p-missing".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        );
        assert!(matches!(result, Err(PaneError::NotFound(_))));
        // rollback：依然只 1 pane
        assert_eq!(pane_count(&pool, &tab_id), 1);
    }

    #[test]
    fn split_rolls_back_when_layout_invalid() {
        // v1.1: 后端兜底 3 columns × 2 rows，防 direct IPC 绕过前端按钮状态。
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        // 依次形成 H(p1, V(p2, H(p3, p4)))，第 4 次继续下分会产生第 3 行。
        let mut parent_pane = "p1".to_string();
        let mut existing_panes = std::collections::HashSet::new();
        existing_panes.insert(parent_pane.clone());

        for i in 0..4 {
            let result = apply_pane_split(
                &pool,
                &PaneCreateRequest {
                    tab_id: tab_id.clone(),
                    parent_pane_id: parent_pane.clone(),
                    direction: [
                        SplitDir::Horizontal,
                        SplitDir::Vertical,
                        SplitDir::Horizontal,
                        SplitDir::Vertical,
                    ][i]
                        .clone(),
                    shell: "/bin/zsh".to_string(),
                },
            );
            if i < 3 {
                // 前 3 次成功，仍在 3 columns × 2 rows 内。
                let response = result.unwrap();
                assert_eq!(response.panes.len(), i + 2);
                // 找到新创建的 pane（不在 existing_panes 中）
                let new_pane = response
                    .panes
                    .iter()
                    .find(|p| !existing_panes.contains(&p.pane_id))
                    .map(|p| p.pane_id.clone())
                    .unwrap();
                existing_panes.insert(new_pane.clone());
                parent_pane = new_pane;
            } else {
                // 第 4 次应触发 InvalidLayout（row limit exceeded）
                assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
                // rollback：依然 4 pane（不是 5）
                assert_eq!(pane_count(&pool, &tab_id), 4);
            }
        }
    }

    #[test]
    fn close_removes_pane_and_updates_focus() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        let r1 = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();
        let new_pane = r1
            .panes
            .iter()
            .find(|p| p.pane_id != "p1")
            .map(|p| p.pane_id.clone())
            .unwrap();

        let response = apply_pane_close(
            &pool,
            &PaneCloseRequest {
                pane_id: new_pane.clone(),
            },
        )
        .unwrap();
        assert_eq!(response.panes.len(), 1);
        assert!(matches!(response.layout, LayoutNode::Single { ref pane_id } if pane_id == "p1"));
        assert_eq!(
            assert_persisted_focus(&pool, &tab_id).as_deref(),
            Some("p1")
        );
    }

    #[test]
    fn close_last_pane_errors() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let result = apply_pane_close(
            &pool,
            &PaneCloseRequest {
                pane_id: "p1".to_string(),
            },
        );
        assert!(result.is_err());
        assert_eq!(pane_count(&pool, &tab_id), 1);
    }

    #[test]
    fn close_unknown_pane_errors() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let result = apply_pane_close(
            &pool,
            &PaneCloseRequest {
                pane_id: "p-missing".to_string(),
            },
        );
        assert!(matches!(result, Err(PaneError::NotFound(_))));
    }

    #[test]
    fn focus_updates_focused_pane() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let r1 = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();
        let new_pane = r1
            .panes
            .iter()
            .find(|p| p.pane_id != "p1")
            .map(|p| p.pane_id.clone())
            .unwrap();

        // 切回 p1
        let response = apply_pane_focus(
            &pool,
            &PaneFocusRequest {
                tab_id: tab_id.clone(),
                focused_pane_id: "p1".to_string(),
            },
        )
        .unwrap();
        assert_eq!(response.panes.len(), 2);
        assert_eq!(
            assert_persisted_focus(&pool, &tab_id).as_deref(),
            Some("p1")
        );

        // 切到新 pane 也应成功
        apply_pane_focus(
            &pool,
            &PaneFocusRequest {
                tab_id: tab_id.clone(),
                focused_pane_id: new_pane.clone(),
            },
        )
        .unwrap();
        assert_eq!(
            assert_persisted_focus(&pool, &tab_id).as_deref(),
            Some(new_pane.as_str())
        );
    }

    #[test]
    fn focus_rejects_unknown_pane() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let result = apply_pane_focus(
            &pool,
            &PaneFocusRequest {
                tab_id,
                focused_pane_id: "p-missing".to_string(),
            },
        );
        assert!(matches!(result, Err(PaneError::NotFound(_))));
    }

    #[test]
    fn split_ratio_update_persists() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let r1 = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();

        let response = apply_split_ratio_update(
            &pool,
            &SplitRatioUpdateRequest {
                pane_id: "p1".to_string(),
                new_ratio: 0.7,
            },
        )
        .unwrap();
        assert_eq!(response.panes.len(), 2);
        match response.layout {
            LayoutNode::Split { ratio, .. } => assert!((ratio - 0.7).abs() < 1e-6),
            other => panic!("expected Split layout, got {other:?}"),
        }
        let _ = r1; // keep r1 alive to satisfy clippy
    }

    #[test]
    fn split_ratio_rejects_invalid_value() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();

        let result = apply_split_ratio_update(
            &pool,
            &SplitRatioUpdateRequest {
                pane_id: "p1".to_string(),
                new_ratio: 1.5,
            },
        );
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    #[test]
    fn layout_preset_solo_collapses_to_focused() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();
        // 此时焦点在新 pane · 切回 p1 让 solo 保留 p1
        apply_pane_focus(
            &pool,
            &PaneFocusRequest {
                tab_id: tab_id.clone(),
                focused_pane_id: "p1".to_string(),
            },
        )
        .unwrap();

        let response = apply_layout_preset(
            &pool,
            &LayoutApplyRequest {
                tab_id: tab_id.clone(),
                preset: "solo".to_string(),
                confirmed: true,
            },
        )
        .unwrap();
        assert_eq!(response.panes.len(), 1);
        assert!(matches!(response.layout, LayoutNode::Single { ref pane_id } if pane_id == "p1"));
        // panes 表里只剩 p1
        assert_eq!(pane_count(&pool, &tab_id), 1);
    }

    #[test]
    fn layout_preset_unknown_kind_errors() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let result = apply_layout_preset(
            &pool,
            &LayoutApplyRequest {
                tab_id,
                preset: "mystery".to_string(),
                confirmed: true,
            },
        );
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
    }

    #[test]
    fn layout_preset_ai_runner_errors_on_single_pane() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let result = apply_layout_preset(
            &pool,
            &LayoutApplyRequest {
                tab_id: tab_id.clone(),
                preset: "aiAndRunner".to_string(),
                confirmed: true,
            },
        );
        // 单 Pane 上 AiAndRunner 应 fail · pure function 返回 InvalidLayout
        assert!(result.is_err());
        // rollback：依然 1 pane
        assert_eq!(pane_count(&pool, &tab_id), 1);
    }

    // ---- MVP-14 Phase A · LayoutEnvelope tests ----

    #[test]
    fn envelope_round_trip_persists_version_and_focus() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        // 读取验证 layout 是 v1 envelope 格式
        let conn = pool.get().unwrap();
        let envelope = read_tab_layout_envelope(&conn, &tab_id).unwrap();
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.focused_pane_id, Some("p1".to_string()));
        assert!(matches!(envelope.root, LayoutNode::Single { ref pane_id } if pane_id == "p1"));
        assert!(envelope.updated_at > 0);
    }

    #[test]
    fn legacy_layout_migration_reads_and_upgrades() {
        let (_dir, pool, tab_id) = setup();
        // 直接写 legacy layout（裸 LayoutNode JSON）
        let conn = pool.get().unwrap();
        let legacy_json = r#"{"kind":"single","paneId":"legacy-pane"}"#;
        conn.execute(
            "UPDATE tabs SET layout = ?1, focused_pane_id = ?2 WHERE tab_id = ?3",
            params![legacy_json, "legacy-pane", tab_id],
        )
        .unwrap();

        // 读取应自动 fallback 为 envelope
        let envelope = read_tab_layout_envelope(&conn, &tab_id).unwrap();
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.focused_pane_id, None); // legacy 无 focus 信息
        assert!(
            matches!(envelope.root, LayoutNode::Single { ref pane_id } if pane_id == "legacy-pane")
        );
    }

    #[test]
    fn migrate_workspace_layout_state_upgrades_legacy() {
        let (_dir, pool, tab_id) = setup();
        // 写 legacy layout
        let conn = pool.get().unwrap();
        let legacy_json = r#"{"kind":"single","paneId":"p1"}"#;
        conn.execute(
            "UPDATE tabs SET layout = ?1, focused_pane_id = ?2 WHERE tab_id = ?3",
            params![legacy_json, "p1", tab_id],
        )
        .unwrap();
        drop(conn);

        // 迁移
        let migrated = migrate_workspace_layout_state(&pool).unwrap();
        assert_eq!(migrated, 1);

        // 验证已升级为 v1 envelope
        let conn = pool.get().unwrap();
        let layout_json: String = conn
            .query_row(
                "SELECT layout FROM tabs WHERE tab_id = ?1",
                [&tab_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(layout_json.contains("\"version\":1"));
        assert!(layout_json.contains("\"focusedPaneId\":"));
    }

    #[test]
    fn migrate_workspace_layout_state_skips_already_v1() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        // 已经是 v1 envelope，不应再迁移
        let migrated = migrate_workspace_layout_state(&pool).unwrap();
        assert_eq!(migrated, 0);
    }

    #[test]
    fn write_workspace_layout_state_round_trip() {
        let (_dir, pool, tab_id) = setup();
        let envelope = LayoutEnvelope {
            version: 1,
            root: LayoutNode::Single {
                pane_id: "wsp1".to_string(),
            },
            focused_pane_id: Some("wsp1".to_string()),
            updated_at: 1234567890,
        };

        write_workspace_layout_state(&pool, &tab_id, &envelope).unwrap();

        // 验证读取一致
        let conn = pool.get().unwrap();
        let read_envelope = read_tab_layout_envelope(&conn, &tab_id).unwrap();
        assert_eq!(read_envelope.version, envelope.version);
        assert_eq!(read_envelope.focused_pane_id, envelope.focused_pane_id);
        assert_eq!(read_envelope.updated_at, envelope.updated_at);
    }

    #[test]
    fn pane_init_creates_v1_envelope() {
        let (_dir, pool, tab_id) = setup();
        assert_eq!(pane_count(&pool, &tab_id), 0);

        let response = apply_pane_init_for_tab(
            &pool,
            &PaneInitRequest {
                tab_id: tab_id.clone(),
                shell: "/bin/zsh".to_string(),
                cwd: "/tmp".to_string(),
            },
        )
        .unwrap();

        assert_eq!(response.panes.len(), 1);
        assert_eq!(pane_count(&pool, &tab_id), 1);
        assert!(
            matches!(response.layout, LayoutNode::Single { ref pane_id } if !pane_id.is_empty())
        );

        // 验证数据库中存的是 v1 envelope
        let conn = pool.get().unwrap();
        let layout_json: String = conn
            .query_row(
                "SELECT layout FROM tabs WHERE tab_id = ?1",
                [&tab_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(layout_json.contains("\"version\":1"));
        assert!(layout_json.contains("\"focusedPaneId\":"));
    }

    #[test]
    fn split_writes_v1_envelope() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");

        apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();

        let conn = pool.get().unwrap();
        let layout_json: String = conn
            .query_row(
                "SELECT layout FROM tabs WHERE tab_id = ?1",
                [&tab_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(layout_json.contains("\"version\":1"));
        assert!(layout_json.contains("\"focusedPaneId\":"));
    }

    #[test]
    fn close_writes_v1_envelope() {
        let (_dir, pool, tab_id) = setup();
        seed_initial_pane(&pool, &tab_id, "p1");
        let r1 = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: "p1".to_string(),
                direction: SplitDir::Horizontal,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();
        let new_pane = r1
            .panes
            .iter()
            .find(|p| p.pane_id != "p1")
            .map(|p| p.pane_id.clone())
            .unwrap();

        apply_pane_close(
            &pool,
            &PaneCloseRequest {
                pane_id: new_pane.clone(),
            },
        )
        .unwrap();

        let conn = pool.get().unwrap();
        let layout_json: String = conn
            .query_row(
                "SELECT layout FROM tabs WHERE tab_id = ?1",
                [&tab_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(layout_json.contains("\"version\":1"));
    }
}
