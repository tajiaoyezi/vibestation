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
}
