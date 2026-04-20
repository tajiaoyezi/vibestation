//! Workspace management — CRUD + git detection
//!
//! MVP-02 spec §A-F: create / list / open / close / delete workspaces,
//! auto-detect git repos, multi-workspace并存, persistence via rusqlite.

use crate::db::{DbError, DbPool};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;
use uuid::Uuid;

/// Workspace metadata · IPC contract source of truth.
///
/// 前端类型由 ts-rs 自动生成到 `web/src/bindings/WorkspaceMetadata.ts`（见
/// `crates/app/build.rs`）· 禁止手写 TypeScript 对偶 interface · 避免 H2 类
/// camelCase / snake_case drift 事故（见 `docs/spikes/SPIKE-08-report.md §A`）。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceMetadata {
    pub workspace_id: String,
    pub name: String,
    pub path: String,
    pub has_git: bool,
    pub repo_root: Option<String>,
    /// Unix timestamp (seconds)· 映射为 TS `number` 而非默认 `bigint`：unix 时间戳
    /// 秒数在可预见未来（~year 285476）都 < 2^53 · 用 `number` 前端 Date/sort 零改动。
    #[ts(type = "number")]
    pub created_at: i64,
    /// 同 `created_at`。
    #[ts(type = "number")]
    pub last_opened: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("workspace already exists at path: {0}")]
    DuplicatePath(String),
    #[error("workspace not found: {0}")]
    NotFound(String),
    #[error("path does not exist or is inaccessible: {0}")]
    InvalidPath(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct WorkspaceStore;

impl WorkspaceStore {
    /// Create a new workspace for the given directory path.
    ///
    /// - Validates the path exists
    /// - Generates a UUID v4 workspace_id
    /// - Auto-detects git repo (up to 5 parent levels)
    /// - Default name = directory name (editable later)
    pub fn create(
        pool: &DbPool,
        dir_path: &str,
        name: Option<&str>,
    ) -> Result<WorkspaceMetadata, WorkspaceError> {
        let canonical = Self::canonicalize_path(dir_path)?;
        let dir_name = Path::new(&canonical)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());

        let workspace_name = name.map(String::from).unwrap_or(dir_name);
        let (has_git, repo_root) = Self::detect_git(&canonical);

        let ws = WorkspaceMetadata {
            workspace_id: Uuid::new_v4().to_string(),
            name: workspace_name,
            path: canonical,
            has_git,
            repo_root,
            created_at: chrono::Utc::now().timestamp(),
            last_opened: chrono::Utc::now().timestamp(),
        };

        let conn = pool.get().map_err(DbError::from)?;
        conn.execute(
            "INSERT INTO workspaces (workspace_id, name, path, has_git, repo_root, created_at, last_opened)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                ws.workspace_id,
                ws.name,
                ws.path,
                ws.has_git as i32,
                ws.repo_root,
                ws.created_at,
                ws.last_opened,
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(
                ref err, _
            ) if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                WorkspaceError::DuplicatePath(ws.path.clone())
            }
            other => WorkspaceError::Db(DbError::Query(other.to_string())),
        })?;

        Ok(ws)
    }

    /// List all workspaces, ordered by last_opened desc (most recent first).
    pub fn list(pool: &DbPool) -> Result<Vec<WorkspaceMetadata>, WorkspaceError> {
        let conn = pool.get().map_err(DbError::from)?;
        let mut stmt = conn
            .prepare(
                "SELECT workspace_id, name, path, has_git, repo_root, created_at, last_opened
                 FROM workspaces ORDER BY last_opened DESC",
            )
            .map_err(DbError::from)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(WorkspaceMetadata {
                    workspace_id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    has_git: row.get::<_, i32>(3)? != 0,
                    repo_root: row.get(4)?,
                    created_at: row.get(5)?,
                    last_opened: row.get(6)?,
                })
            })
            .map_err(DbError::from)?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(DbError::from)?);
        }
        Ok(result)
    }

    /// Get a single workspace by id.
    pub fn get_by_id(pool: &DbPool, id: &str) -> Result<WorkspaceMetadata, WorkspaceError> {
        let conn = pool.get().map_err(DbError::from)?;
        conn.query_row(
            "SELECT workspace_id, name, path, has_git, repo_root, created_at, last_opened
             FROM workspaces WHERE workspace_id = ?1",
            [id],
            |row| {
                Ok(WorkspaceMetadata {
                    workspace_id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    has_git: row.get::<_, i32>(3)? != 0,
                    repo_root: row.get(4)?,
                    created_at: row.get(5)?,
                    last_opened: row.get(6)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => WorkspaceError::NotFound(id.to_string()),
            other => WorkspaceError::Db(DbError::Query(other.to_string())),
        })
    }

    /// Update last_opened timestamp (workspace "open" action).
    pub fn touch(pool: &DbPool, id: &str) -> Result<(), WorkspaceError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute(
                "UPDATE workspaces SET last_opened = ?1 WHERE workspace_id = ?2",
                rusqlite::params![chrono::Utc::now().timestamp(), id],
            )
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(WorkspaceError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Permanently delete a workspace from the database.
    pub fn delete(pool: &DbPool, id: &str) -> Result<(), WorkspaceError> {
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute("DELETE FROM workspaces WHERE workspace_id = ?1", [id])
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(WorkspaceError::NotFound(id.to_string()));
        }
        Ok(())
    }

    /// Check if a workspace exists at the given path (for duplicate detection).
    pub fn exists_at_path(pool: &DbPool, path: &str) -> Result<bool, WorkspaceError> {
        let canonical = Self::canonicalize_path(path)?;
        let conn = pool.get().map_err(DbError::from)?;
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM workspaces WHERE path = ?1",
                [canonical],
                |row| row.get(0),
            )
            .map_err(DbError::from)?;
        Ok(count > 0)
    }

    fn canonicalize_path(path: &str) -> Result<String, WorkspaceError> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(WorkspaceError::InvalidPath(path.to_string()));
        }
        dunce::canonicalize(p)
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|_| WorkspaceError::InvalidPath(path.to_string()))
    }

    /// Detect git repo: check directory for `.git`, then walk up 5 levels.
    fn detect_git(dir: &str) -> (bool, Option<String>) {
        let start = Path::new(dir);
        let mut current = start;

        for _ in 0..=5 {
            if current.join(".git").exists() {
                return (true, Some(current.to_string_lossy().to_string()));
            }
            current = match current.parent() {
                Some(p) => p,
                None => break,
            };
        }

        (false, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_workspace.db");
        let pool = db::open_pool(&db_path).unwrap();
        (dir, pool)
    }

    #[test]
    fn create_workspace_basic() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("my-project");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();

        assert!(!ws.workspace_id.is_empty());
        assert_eq!(ws.name, "my-project");
        assert!(!ws.has_git);
        assert!(ws.repo_root.is_none());
    }

    #[test]
    fn create_workspace_custom_name() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("project-x");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), Some("My Custom Name"))
            .unwrap();

        assert_eq!(ws.name, "My Custom Name");
    }

    #[test]
    fn create_workspace_detects_git() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("git-project");
        std::fs::create_dir_all(ws_dir.join(".git")).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();

        assert!(ws.has_git);
        assert!(ws.repo_root.is_some());
    }

    #[test]
    fn detect_git_parent_directory() {
        let (dir, _pool) = setup();
        let parent = dir.path().join("repo-root");
        let child = parent.join("subdir/deep");
        std::fs::create_dir_all(parent.join(".git")).unwrap();
        std::fs::create_dir_all(&child).unwrap();

        let (has_git, repo_root) = WorkspaceStore::detect_git(child.to_str().unwrap());
        assert!(has_git);
        assert!(repo_root.is_some());
    }

    #[test]
    fn duplicate_path_rejected() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("dup");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let _ = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None);
        let result = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None);
        assert!(matches!(result, Err(WorkspaceError::DuplicatePath(_))));
    }

    #[test]
    fn list_workspaces() {
        let (dir, pool) = setup();
        let d1 = dir.path().join("ws-a");
        let d2 = dir.path().join("ws-b");
        std::fs::create_dir_all(&d1).unwrap();
        std::fs::create_dir_all(&d2).unwrap();

        WorkspaceStore::create(&pool, d1.to_str().unwrap(), None).unwrap();
        WorkspaceStore::create(&pool, d2.to_str().unwrap(), None).unwrap();

        let list = WorkspaceStore::list(&pool).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn touch_updates_last_opened() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("touch-ws");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        let original = ws.last_opened;

        std::thread::sleep(std::time::Duration::from_millis(10));

        WorkspaceStore::touch(&pool, &ws.workspace_id).unwrap();
        let updated = WorkspaceStore::get_by_id(&pool, &ws.workspace_id).unwrap();
        assert!(updated.last_opened >= original);
    }

    #[test]
    fn delete_workspace() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("del-ws");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        WorkspaceStore::delete(&pool, &ws.workspace_id).unwrap();

        let list = WorkspaceStore::list(&pool).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn delete_nonexistent_is_error() {
        let (_dir, pool) = setup();
        let result = WorkspaceStore::delete(&pool, "nonexistent-uuid");
        assert!(matches!(result, Err(WorkspaceError::NotFound(_))));
    }

    #[test]
    fn non_existent_path_rejected() {
        let (dir, pool) = setup();
        let result = WorkspaceStore::create(
            &pool,
            dir.path().join("no-such-dir").to_str().unwrap(),
            None,
        );
        assert!(matches!(result, Err(WorkspaceError::InvalidPath(_))));
    }

    #[test]
    fn exists_at_path() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("exists-check");
        std::fs::create_dir_all(&ws_dir).unwrap();

        assert!(!WorkspaceStore::exists_at_path(&pool, ws_dir.to_str().unwrap()).unwrap());

        WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();

        assert!(WorkspaceStore::exists_at_path(&pool, ws_dir.to_str().unwrap()).unwrap());
    }

    #[test]
    fn utf8_path_supported() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("中文目录");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        assert!(ws.path.contains("中文"));
    }

    #[test]
    fn path_with_spaces_supported() {
        let (dir, pool) = setup();
        let ws_dir = dir.path().join("my project folder");
        std::fs::create_dir_all(&ws_dir).unwrap();

        let ws = WorkspaceStore::create(&pool, ws_dir.to_str().unwrap(), None).unwrap();
        assert!(ws.path.contains("my project folder"));
    }
}
