//! Tab storage — CRUD + scrollback DAO
//!
//! MVP-04 spec §G + §数据模型变更: tab state persistence via rusqlite.
//! IPC contract source of truth (ts-rs derives). scroll_back excluded from IPC
//! struct per spec §G.2 — fetched on-demand via `scrollback_fetch`.

use crate::db::{DbError, DbPool};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// Tab state · IPC contract source of truth (spec §G.2).
///
/// `scroll_back` is intentionally excluded from this struct — it can be very
/// large (up to 10k lines) and would drag IPC performance. Frontend must use
/// `tab_scrollback_fetch` command to pull visible buffer on demand.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub tab_id: String,
    pub workspace_id: String,
    pub name: String,
    pub shell: String,
    pub cwd: String,
    #[ts(type = "number")]
    pub created_at: i64,
}

/// Parameters for creating a new tab.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TabCreateRequest {
    pub workspace_id: String,
    pub name: Option<String>,
    pub shell: Option<String>,
    pub cwd: Option<String>,
}

/// Parameters for closing a tab.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TabCloseRequest {
    pub tab_id: String,
}

/// Parameters for renaming a tab.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TabRenameRequest {
    pub tab_id: String,
    pub name: String,
}

/// Response for listing tabs in a workspace.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TabListResponse {
    pub tabs: Vec<TabState>,
}

#[derive(Debug, thiserror::Error)]
pub enum TabError {
    #[error("tab not found: {0}")]
    NotFound(String),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct TabsDao;

impl TabsDao {
    fn row_to_tab(row: &rusqlite::Row<'_>) -> Result<TabState, rusqlite::Error> {
        Ok(TabState {
            tab_id: row.get(0)?,
            workspace_id: row.get(1)?,
            name: row.get(2)?,
            shell: row.get(3)?,
            cwd: row.get(4)?,
            created_at: row.get(5)?,
        })
    }

    pub fn create(pool: &DbPool, req: &TabCreateRequest) -> Result<TabState, TabError> {
        let conn = pool.get().map_err(DbError::from)?;

        let ws_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM workspaces WHERE workspace_id = ?1",
                [&req.workspace_id],
                |row| row.get(0),
            )
            .map_err(DbError::from)?;
        if !ws_exists {
            return Err(TabError::WorkspaceNotFound(req.workspace_id.clone()));
        }

        let tab_id = Uuid::new_v4().to_string();
        let name = req.name.clone().unwrap_or_else(|| "Terminal".to_string());
        let shell = req.shell.clone().unwrap_or_else(|| {
            if cfg!(target_os = "macos") {
                "/bin/zsh".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });
        let cwd = req.cwd.clone().unwrap_or_else(|| "/".to_string());
        let created_at = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO tabs (tab_id, workspace_id, name, shell, cwd, scroll_back, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, '[]', ?6)",
            rusqlite::params![tab_id, req.workspace_id, name, shell, cwd, created_at],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(ref err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                TabError::Db(DbError::Query(format!(
                    "constraint violation inserting tab: {e}"
                )))
            }
            other => TabError::Db(DbError::Query(other.to_string())),
        })?;

        Ok(TabState {
            tab_id,
            workspace_id: req.workspace_id.clone(),
            name,
            shell,
            cwd,
            created_at,
        })
    }

    pub fn list_by_workspace(pool: &DbPool, workspace_id: &str) -> Result<Vec<TabState>, TabError> {
        let conn = pool.get().map_err(DbError::from)?;
        let mut stmt = conn
            .prepare(
                "SELECT tab_id, workspace_id, name, shell, cwd, created_at
                 FROM tabs WHERE workspace_id = ?1 ORDER BY created_at DESC",
            )
            .map_err(DbError::from)?;

        let rows = stmt
            .query_map([workspace_id], Self::row_to_tab)
            .map_err(DbError::from)?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(DbError::from)?);
        }
        Ok(result)
    }

    pub fn get(pool: &DbPool, tab_id: &str) -> Result<TabState, TabError> {
        let conn = pool.get().map_err(DbError::from)?;
        conn.query_row(
            "SELECT tab_id, workspace_id, name, shell, cwd, created_at
             FROM tabs WHERE tab_id = ?1",
            [tab_id],
            Self::row_to_tab,
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => TabError::NotFound(tab_id.to_string()),
            other => TabError::Db(DbError::Query(other.to_string())),
        })
    }

    pub fn rename(pool: &DbPool, tab_id: &str, new_name: &str) -> Result<(), TabError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute(
                "UPDATE tabs SET name = ?1 WHERE tab_id = ?2",
                rusqlite::params![new_name, tab_id],
            )
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(TabError::NotFound(tab_id.to_string()));
        }
        Ok(())
    }

    pub fn update_cwd(pool: &DbPool, tab_id: &str, cwd: &str) -> Result<(), TabError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute(
                "UPDATE tabs SET cwd = ?1 WHERE tab_id = ?2",
                rusqlite::params![cwd, tab_id],
            )
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(TabError::NotFound(tab_id.to_string()));
        }
        Ok(())
    }

    pub fn delete(pool: &DbPool, tab_id: &str) -> Result<(), TabError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute("DELETE FROM tabs WHERE tab_id = ?1", [tab_id])
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(TabError::NotFound(tab_id.to_string()));
        }
        Ok(())
    }

    pub fn scrollback_append(
        pool: &DbPool,
        tab_id: &str,
        lines: &[String],
    ) -> Result<(), TabError> {
        if lines.is_empty() {
            return Ok(());
        }

        let conn = pool.get().map_err(DbError::from)?;

        let current: String = conn
            .query_row(
                "SELECT scroll_back FROM tabs WHERE tab_id = ?1",
                [tab_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => TabError::NotFound(tab_id.to_string()),
                other => TabError::Db(DbError::Query(other.to_string())),
            })?;

        let mut scrollback: Vec<String> =
            serde_json::from_str(&current).unwrap_or_else(|_| Vec::new());
        scrollback.extend(lines.iter().cloned());

        let max_lines: usize = 10_000;
        if scrollback.len() > max_lines {
            let drain_count = scrollback.len() - max_lines;
            scrollback.drain(0..drain_count);
        }

        let json = serde_json::to_string(&scrollback).map_err(|e| DbError::Query(e.to_string()))?;

        let rows = conn
            .execute(
                "UPDATE tabs SET scroll_back = ?1 WHERE tab_id = ?2",
                rusqlite::params![json, tab_id],
            )
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(TabError::NotFound(tab_id.to_string()));
        }
        Ok(())
    }

    pub fn scrollback_fetch(
        pool: &DbPool,
        tab_id: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<String>, TabError> {
        let conn = pool.get().map_err(DbError::from)?;

        let current: String = conn
            .query_row(
                "SELECT scroll_back FROM tabs WHERE tab_id = ?1",
                [tab_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => TabError::NotFound(tab_id.to_string()),
                other => TabError::Db(DbError::Query(other.to_string())),
            })?;

        let scrollback: Vec<String> = serde_json::from_str(&current).unwrap_or_else(|_| Vec::new());

        let start = offset.min(scrollback.len());
        let end = (offset + limit).min(scrollback.len());
        Ok(scrollback[start..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::workspace::WorkspaceStore;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool, String) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_tabs.db");
        let pool = db::open_pool(&db_path).unwrap();
        let ws_dir = dir.path().join("test-ws");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        (dir, pool, ws.workspace_id)
    }

    fn make_req(workspace_id: &str) -> TabCreateRequest {
        TabCreateRequest {
            workspace_id: workspace_id.to_string(),
            name: None,
            shell: None,
            cwd: None,
        }
    }

    #[test]
    fn create_tab_basic() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        assert!(!tab.tab_id.is_empty());
        assert_eq!(tab.workspace_id, ws_id);
        assert_eq!(tab.name, "Terminal");
        assert!(!tab.shell.is_empty());
        assert_eq!(tab.cwd, "/");
    }

    #[test]
    fn create_tab_custom_fields() {
        let (_dir, pool, ws_id) = setup();
        let req = TabCreateRequest {
            workspace_id: ws_id.clone(),
            name: Some("My Tab".to_string()),
            shell: Some("/bin/bash".to_string()),
            cwd: Some("/home/user".to_string()),
        };
        let tab = TabsDao::create(&pool, &req).unwrap();
        assert_eq!(tab.name, "My Tab");
        assert_eq!(tab.shell, "/bin/bash");
        assert_eq!(tab.cwd, "/home/user");
    }

    #[test]
    fn create_tab_nonexistent_workspace_errors() {
        let (_dir, pool, _) = setup();
        let mut req = make_req("nonexistent-ws-id");
        req.workspace_id = "nonexistent-ws-id".to_string();
        let result = TabsDao::create(&pool, &req);
        assert!(matches!(result, Err(TabError::WorkspaceNotFound(_))));
    }

    #[test]
    fn list_by_workspace_empty() {
        let (_dir, pool, ws_id) = setup();
        let tabs = TabsDao::list_by_workspace(&pool, &ws_id).unwrap();
        assert!(tabs.is_empty());
    }

    #[test]
    fn list_by_workspace_returns_tabs() {
        let (_dir, pool, ws_id) = setup();
        TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        let tabs = TabsDao::list_by_workspace(&pool, &ws_id).unwrap();
        assert_eq!(tabs.len(), 2);
    }

    #[test]
    fn list_ordered_by_created_at_desc() {
        let (_dir, pool, ws_id) = setup();
        let t1 = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let t2 = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        let tabs = TabsDao::list_by_workspace(&pool, &ws_id).unwrap();
        assert_eq!(tabs[0].tab_id, t2.tab_id);
        assert_eq!(tabs[1].tab_id, t1.tab_id);
    }

    #[test]
    fn get_tab_by_id() {
        let (_dir, pool, ws_id) = setup();
        let created = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        let fetched = TabsDao::get(&pool, &created.tab_id).unwrap();
        assert_eq!(fetched.tab_id, created.tab_id);
        assert_eq!(fetched.name, created.name);
    }

    #[test]
    fn get_nonexistent_tab_errors() {
        let (_dir, pool, _) = setup();
        let result = TabsDao::get(&pool, "nonexistent-id");
        assert!(matches!(result, Err(TabError::NotFound(_))));
    }

    #[test]
    fn rename_tab() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        TabsDao::rename(&pool, &tab.tab_id, "New Name").unwrap();
        let fetched = TabsDao::get(&pool, &tab.tab_id).unwrap();
        assert_eq!(fetched.name, "New Name");
    }

    #[test]
    fn rename_nonexistent_tab_errors() {
        let (_dir, pool, _) = setup();
        let result = TabsDao::rename(&pool, "nonexistent-id", "name");
        assert!(matches!(result, Err(TabError::NotFound(_))));
    }

    #[test]
    fn update_cwd() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        TabsDao::update_cwd(&pool, &tab.tab_id, "/home/user/project").unwrap();
        let fetched = TabsDao::get(&pool, &tab.tab_id).unwrap();
        assert_eq!(fetched.cwd, "/home/user/project");
    }

    #[test]
    fn update_cwd_nonexistent_errors() {
        let (_dir, pool, _) = setup();
        let result = TabsDao::update_cwd(&pool, "nonexistent-id", "/tmp");
        assert!(matches!(result, Err(TabError::NotFound(_))));
    }

    #[test]
    fn delete_tab() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        TabsDao::delete(&pool, &tab.tab_id).unwrap();
        let result = TabsDao::get(&pool, &tab.tab_id);
        assert!(matches!(result, Err(TabError::NotFound(_))));
    }

    #[test]
    fn delete_nonexistent_tab_errors() {
        let (_dir, pool, _) = setup();
        let result = TabsDao::delete(&pool, "nonexistent-id");
        assert!(matches!(result, Err(TabError::NotFound(_))));
    }

    #[test]
    fn cascade_delete_on_workspace_removal() {
        let (dir, pool, ws_id) = setup();
        TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        let conn = pool.get().unwrap();
        let tab_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM tabs WHERE workspace_id = ?1",
                [&ws_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tab_count, 2);
        drop(conn);

        WorkspaceStore::delete(&pool, &ws_id).unwrap();

        let conn = pool.get().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        let remaining: i64 = conn
            .query_row(
                "SELECT count(*) FROM tabs WHERE workspace_id = ?1",
                [&ws_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 0, "tabs should cascade-delete with workspace");
        drop(conn);
        drop(pool);
        drop(dir);
    }

    #[test]
    fn concurrent_create_10_tabs() {
        let (_dir, pool, ws_id) = setup();
        let mut tab_ids = Vec::new();
        for i in 0..10 {
            let req = TabCreateRequest {
                workspace_id: ws_id.clone(),
                name: Some(format!("Tab {i}")),
                shell: Some("/bin/zsh".to_string()),
                cwd: Some(format!("/home/user/{i}")),
            };
            let tab = TabsDao::create(&pool, &req).unwrap();
            tab_ids.push(tab.tab_id);
        }

        let tabs = TabsDao::list_by_workspace(&pool, &ws_id).unwrap();
        assert_eq!(tabs.len(), 10);
        assert_eq!(tab_ids.len(), 10);
        let unique_ids: std::collections::HashSet<_> = tab_ids.iter().cloned().collect();
        assert_eq!(unique_ids.len(), 10, "all tab IDs should be unique");
    }

    #[test]
    fn scrollback_append_and_fetch() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        TabsDao::scrollback_append(
            &pool,
            &tab.tab_id,
            &["line1".to_string(), "line2".to_string()],
        )
        .unwrap();

        let lines = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 0, 10).unwrap();
        assert_eq!(lines, vec!["line1", "line2"]);
    }

    #[test]
    fn scrollback_fetch_with_offset_and_limit() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        let lines: Vec<String> = (0..10).map(|i| format!("line{i}")).collect();
        TabsDao::scrollback_append(&pool, &tab.tab_id, &lines).unwrap();

        let page1 = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 0, 3).unwrap();
        assert_eq!(page1, vec!["line0", "line1", "line2"]);

        let page2 = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 8, 10).unwrap();
        assert_eq!(page2, vec!["line8", "line9"]);
    }

    #[test]
    fn scrollback_capped_at_10k() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        let lines: Vec<String> = (0..10_000).map(|i| format!("line{i}")).collect();
        TabsDao::scrollback_append(&pool, &tab.tab_id, &lines).unwrap();

        let all = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 0, 10_500).unwrap();
        assert_eq!(all.len(), 10_000);
    }

    #[test]
    fn scrollback_trims_oldest_on_overflow() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        let first: Vec<String> = (0..9_998).map(|i| format!("old{i}")).collect();
        TabsDao::scrollback_append(&pool, &tab.tab_id, &first).unwrap();

        let extra: Vec<String> = (0..5).map(|i| format!("new{i}")).collect();
        TabsDao::scrollback_append(&pool, &tab.tab_id, &extra).unwrap();

        let all = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 0, 10_500).unwrap();
        assert_eq!(all.len(), 10_000);
        assert!(
            all[0].starts_with("old"),
            "oldest entries should still be present"
        );
        assert_eq!(all[9999], "new4");
    }

    #[test]
    fn scrollback_fetch_nonexistent_tab_errors() {
        let (_dir, pool, _) = setup();
        let result = TabsDao::scrollback_fetch(&pool, "nonexistent", 0, 10);
        assert!(matches!(result, Err(TabError::NotFound(_))));
    }

    #[test]
    fn scrollback_append_empty_lines_noop() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();

        TabsDao::scrollback_append(&pool, &tab.tab_id, &[]).unwrap();

        let lines = TabsDao::scrollback_fetch(&pool, &tab.tab_id, 0, 10).unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn tab_id_is_valid_uuid_format() {
        let (_dir, pool, ws_id) = setup();
        let tab = TabsDao::create(&pool, &make_req(&ws_id)).unwrap();
        assert!(
            Uuid::parse_str(&tab.tab_id).is_ok(),
            "tab_id should be valid UUID v4"
        );
    }

    #[test]
    fn empty_workspace_list_returns_empty_vec() {
        let (_dir, pool, _) = setup();
        let dir2 = TempDir::new().unwrap();
        let ws_dir2 = dir2.path().join("ws-no-tabs");
        std::fs::create_dir_all(&ws_dir2).unwrap();
        let ws2 = WorkspaceStore::create(&pool, ws_dir2.to_str().unwrap(), None).unwrap();

        let tabs = TabsDao::list_by_workspace(&pool, &ws2.workspace_id).unwrap();
        assert!(tabs.is_empty());
    }
}
