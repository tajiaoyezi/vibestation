//! Pane service · MVP-05 Phase B Step 2 IPC 层
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

use crate::db::{DbError, DbPool};
use crate::panes::{
    apply_smart_layout, close_pane_in_layout, split_layout, update_split_ratio, LayoutApplyRequest,
    LayoutNode, PaneCloseRequest, PaneCreateRequest, PaneError, PaneFocusRequest, PaneListResponse,
    PanesDao, SmartLayoutKind, SplitRatioUpdateRequest,
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

    let current_layout = read_tab_layout(&tx, &req.tab_id)?;

    // idempotent: 已初始化过则直接返回当前状态
    if let LayoutNode::Single { pane_id } = &current_layout {
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

    tx.execute(
        "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
         VALUES (?1, ?2, ?3, ?4, '[]', ?5)",
        params![new_pane_id, req.tab_id, req.shell, req.cwd, created_at],
    )
    .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    write_tab_layout(&tx, &req.tab_id, &new_layout)?;
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

    let current_layout = read_tab_layout(&tx, &req.tab_id)?;
    let new_layout = split_layout(
        &current_layout,
        &req.parent_pane_id,
        req.direction.clone(),
        new_pane_id.clone(),
    )?;
    new_layout.validate_mvp_05()?;

    tx.execute(
        "INSERT INTO panes (pane_id, tab_id, shell, cwd, scroll_back, created_at)
         VALUES (?1, ?2, ?3, ?4, '[]', ?5)",
        params![new_pane_id, req.tab_id, req.shell, cwd, created_at],
    )
    .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    write_tab_layout(&tx, &req.tab_id, &new_layout)?;
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
    let current_layout = read_tab_layout(&tx, &tab_id)?;
    let new_layout = close_pane_in_layout(&current_layout, &req.pane_id)?;
    new_layout.validate_mvp_05()?;

    let rows = tx
        .execute("DELETE FROM panes WHERE pane_id = ?1", [&req.pane_id])
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
    if rows == 0 {
        return Err(PaneError::NotFound(req.pane_id.clone()));
    }

    write_tab_layout(&tx, &tab_id, &new_layout)?;
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

    let layout = read_tab_layout(&tx, &req.tab_id)?;
    let pane_ids = collect_pane_ids(&layout);
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
    let current_layout = read_tab_layout(&tx, &req.tab_id)?;

    let (new_layout, closed_pane_ids) =
        apply_smart_layout(&current_layout, kind, &focused_pane_id)?;
    new_layout.validate_mvp_05()?;

    for pane_id in &closed_pane_ids {
        tx.execute("DELETE FROM panes WHERE pane_id = ?1", [pane_id])
            .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;
    }

    write_tab_layout(&tx, &req.tab_id, &new_layout)?;
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
    let current_layout = read_tab_layout(&tx, &tab_id)?;
    let new_layout = update_split_ratio(&current_layout, &req.pane_id, req.new_ratio)?;
    new_layout.validate_mvp_05()?;

    write_tab_layout(&tx, &tab_id, &new_layout)?;

    tx.commit()
        .map_err(|e| PaneError::Db(DbError::Query(e.to_string())))?;

    read_pane_list(pool, &tab_id)
}

// ---- 内部 helpers ----

fn read_tab_layout(conn: &rusqlite::Connection, tab_id: &str) -> Result<LayoutNode, PaneError> {
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
    serde_json::from_str(&layout_json)
        .map_err(|e| PaneError::InvalidLayout(format!("layout JSON parse: {e}")))
}

fn write_tab_layout(
    tx: &rusqlite::Transaction<'_>,
    tab_id: &str,
    layout: &LayoutNode,
) -> Result<(), PaneError> {
    let layout_json = serde_json::to_string(layout)
        .map_err(|e| PaneError::InvalidLayout(format!("layout JSON serialize: {e}")))?;
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
    let layout = read_tab_layout(&conn, tab_id)?;
    Ok(PaneListResponse { panes, layout })
}

fn collect_pane_ids(layout: &LayoutNode) -> Vec<String> {
    let mut out = Vec::new();
    collect_pane_ids_inner(layout, &mut out);
    out
}

fn collect_pane_ids_inner(layout: &LayoutNode, out: &mut Vec<String>) {
    match layout {
        LayoutNode::Single { pane_id } => out.push(pane_id.clone()),
        LayoutNode::Split { first, second, .. } => {
            collect_pane_ids_inner(first, out);
            collect_pane_ids_inner(second, out);
        }
    }
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
        // 写 layout = Single{paneId: pane_id} + 设置 focus
        let pool_clone = pool.clone();
        let conn = pool_clone.get().unwrap();
        let layout_json = format!(r#"{{"kind":"single","paneId":"{pane_id}"}}"#);
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
        // 制造 §H 限制 illegal 场景：连续 3 次 horizontal split → 第 3 次 invalid
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
        assert_eq!(r1.panes.len(), 2);
        let pane_2 = r1
            .panes
            .iter()
            .find(|p| p.pane_id != "p1")
            .map(|p| p.pane_id.clone())
            .unwrap();

        let r2 = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: pane_2.clone(),
                direction: SplitDir::Vertical,
                shell: "/bin/zsh".to_string(),
            },
        )
        .unwrap();
        assert_eq!(r2.panes.len(), 3);

        // 第 3 次 split 在 pane_2 上同向（Vertical）会触发 InvalidLayout
        let pane_3 = r2
            .panes
            .iter()
            .find(|p| p.pane_id != "p1" && p.pane_id != pane_2)
            .map(|p| p.pane_id.clone())
            .unwrap();
        let result = apply_pane_split(
            &pool,
            &PaneCreateRequest {
                tab_id: tab_id.clone(),
                parent_pane_id: pane_3.clone(),
                direction: SplitDir::Vertical,
                shell: "/bin/zsh".to_string(),
            },
        );
        assert!(matches!(result, Err(PaneError::InvalidLayout(_))));
        // rollback：依然 3 pane（不是 4）
        assert_eq!(pane_count(&pool, &tab_id), 3);
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
}
