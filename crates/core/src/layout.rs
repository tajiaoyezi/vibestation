use crate::db::{DbError, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutState {
    pub primary_open: bool,
    pub secondary_open: bool,
    pub bottom_open: bool,
    pub primary_width: u32,
    pub secondary_width: u32,
    pub bottom_height: u32,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            primary_open: true,
            secondary_open: false,
            bottom_open: false,
            primary_width: 236,
            secondary_width: 400,
            bottom_height: 240,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("workspace not found: {0}")]
    NotFound(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct LayoutStore;

impl LayoutStore {
    pub fn save(pool: &DbPool, workspace_id: &str, state: &LayoutState) -> Result<(), LayoutError> {
        let json = serde_json::to_string(state).map_err(|e| DbError::Query(e.to_string()))?;
        let conn = pool.get().map_err(DbError::from)?;
        let rows = conn
            .execute(
                "UPDATE workspaces SET layout_state = ?1 WHERE workspace_id = ?2",
                rusqlite::params![json, workspace_id],
            )
            .map_err(DbError::from)?;
        if rows == 0 {
            return Err(LayoutError::NotFound(workspace_id.to_string()));
        }
        Ok(())
    }

    pub fn load(pool: &DbPool, workspace_id: &str) -> Result<LayoutState, LayoutError> {
        let conn = pool.get().map_err(DbError::from)?;
        let json: Option<String> = conn
            .query_row(
                "SELECT layout_state FROM workspaces WHERE workspace_id = ?1",
                [workspace_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    LayoutError::NotFound(workspace_id.to_string())
                }
                other => LayoutError::Db(DbError::Query(other.to_string())),
            })?;

        match json {
            Some(s) if !s.is_empty() => {
                serde_json::from_str(&s).map_err(|e| LayoutError::Db(DbError::Query(e.to_string())))
            }
            _ => Ok(LayoutState::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::workspace::WorkspaceStore;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_layout.db");
        let pool = db::open_pool(&db_path).unwrap();
        (dir, pool)
    }

    fn create_ws(pool: &DbPool, dir: &TempDir) -> String {
        let ws_dir = dir.path().join("test-ws");
        std::fs::create_dir_all(&ws_dir).unwrap();
        let ws = WorkspaceStore::create(pool, ws_dir.to_str().unwrap(), None).unwrap();
        ws.workspace_id
    }

    #[test]
    fn load_default_when_no_state() {
        let (dir, pool) = setup();
        let id = create_ws(&pool, &dir);
        let state = LayoutStore::load(&pool, &id).unwrap();
        assert!(state.primary_open);
        assert!(!state.secondary_open);
        assert!(!state.bottom_open);
        assert_eq!(state.primary_width, 236);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (dir, pool) = setup();
        let id = create_ws(&pool, &dir);
        let mut state = LayoutState::default();
        state.primary_open = false;
        state.secondary_open = true;
        state.primary_width = 300;
        state.bottom_height = 350;
        LayoutStore::save(&pool, &id, &state).unwrap();

        let loaded = LayoutStore::load(&pool, &id).unwrap();
        assert!(!loaded.primary_open);
        assert!(loaded.secondary_open);
        assert_eq!(loaded.primary_width, 300);
        assert_eq!(loaded.bottom_height, 350);
    }

    #[test]
    fn load_nonexistent_workspace_errors() {
        let (_dir, pool) = setup();
        let result = LayoutStore::load(&pool, "nonexistent-id");
        assert!(matches!(result, Err(LayoutError::NotFound(_))));
    }

    #[test]
    fn save_nonexistent_workspace_errors() {
        let (_dir, pool) = setup();
        let state = LayoutState::default();
        let result = LayoutStore::save(&pool, "nonexistent-id", &state);
        assert!(matches!(result, Err(LayoutError::NotFound(_))));
    }

    #[test]
    fn overwrite_previous_state() {
        let (dir, pool) = setup();
        let id = create_ws(&pool, &dir);
        let mut state = LayoutState::default();
        state.primary_width = 200;
        LayoutStore::save(&pool, &id, &state).unwrap();

        state.primary_width = 400;
        LayoutStore::save(&pool, &id, &state).unwrap();

        let loaded = LayoutStore::load(&pool, &id).unwrap();
        assert_eq!(loaded.primary_width, 400);
    }
}
