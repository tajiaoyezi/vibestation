//! Database initialization and migration
//!
//! SPIKE-04.5 结论：rusqlite B.1-5 全过 · A.3 方案(a) MVP 接受 220ms。
//! Schema version 从 MVP-01 的 1 升级到 2（workspace metadata 扩展）。

#![allow(dead_code)] // Used by app crate via workspace::init_db

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("database connection failed: {0}")]
    Connection(String),
    #[error("migration failed at version {version}: {reason}")]
    Migration { version: u32, reason: String },
    #[error("query failed: {0}")]
    Query(String),
}

impl From<r2d2::Error> for DbError {
    fn from(e: r2d2::Error) -> Self {
        DbError::Connection(e.to_string())
    }
}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError::Query(e.to_string())
    }
}

pub type DbPool = Pool<SqliteConnectionManager>;

const CURRENT_SCHEMA_VERSION: u32 = 6;

/// Open or create the database at `db_path`, run migrations, return a connection pool.
pub fn open_pool(db_path: &std::path::Path) -> Result<DbPool, DbError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DbError::Connection(format!("cannot create db parent dir: {e}")))?;
    }

    let manager = SqliteConnectionManager::file(db_path)
        .with_init(|conn| conn.execute_batch("PRAGMA foreign_keys = ON;"));
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(DbError::from)?;

    let conn = pool.get().map_err(DbError::from)?;
    migrate(&conn)?;

    Ok(pool)
}

pub fn migrate(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(DbError::from)?;
    let user_version: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(DbError::from)?;

    if user_version < 1 {
        migrate_v1(conn)?;
    }
    if user_version < 2 {
        migrate_v2(conn)?;
    }
    if user_version < 3 {
        migrate_v3(conn)?;
    }
    if user_version < 4 {
        migrate_v4(conn)?;
    }
    if user_version < 5 {
        migrate_v5(conn)?;
    }
    if user_version < 6 {
        migrate_v6(conn)?;
    }

    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    migrate(conn)
}

fn migrate_v1(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS workspaces (
            workspace_id  TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            path          TEXT NOT NULL UNIQUE,
            created_at    INTEGER NOT NULL,
            last_opened   INTEGER NOT NULL
        );
        PRAGMA user_version = 1;",
    )
    .map_err(|e| DbError::Migration {
        version: 1,
        reason: e.to_string(),
    })?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool, DbError> {
    let sql = format!(
        "SELECT count(*) > 0 FROM pragma_table_info('{}') WHERE name = '{}'",
        table, column
    );
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(|e| DbError::Migration {
            version: 0,
            reason: e.to_string(),
        })
}

fn migrate_v2(conn: &Connection) -> Result<(), DbError> {
    if !column_exists(conn, "workspaces", "has_git")? {
        conn.execute(
            "ALTER TABLE workspaces ADD COLUMN has_git INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| DbError::Migration {
            version: 2,
            reason: e.to_string(),
        })?;
    }
    if !column_exists(conn, "workspaces", "repo_root")? {
        conn.execute("ALTER TABLE workspaces ADD COLUMN repo_root TEXT", [])
            .map_err(|e| DbError::Migration {
                version: 2,
                reason: e.to_string(),
            })?;
    }
    conn.execute_batch("PRAGMA user_version = 2;")
        .map_err(|e| DbError::Migration {
            version: 2,
            reason: e.to_string(),
        })?;
    Ok(())
}

fn migrate_v3(conn: &Connection) -> Result<(), DbError> {
    if !column_exists(conn, "workspaces", "layout_state")? {
        conn.execute("ALTER TABLE workspaces ADD COLUMN layout_state TEXT", [])
            .map_err(|e| DbError::Migration {
                version: 3,
                reason: e.to_string(),
            })?;
    }
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_settings (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );
         PRAGMA user_version = 3;",
    )
    .map_err(|e| DbError::Migration {
        version: 3,
        reason: e.to_string(),
    })?;
    Ok(())
}

fn migrate_v4(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_workspaces_last_opened
            ON workspaces(last_opened DESC);
         PRAGMA user_version = 4;",
    )
    .map_err(|e| DbError::Migration {
        version: 4,
        reason: e.to_string(),
    })?;
    Ok(())
}

fn migrate_v5(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tabs (
             tab_id        TEXT PRIMARY KEY,
             workspace_id  TEXT NOT NULL,
             name          TEXT NOT NULL,
             shell         TEXT NOT NULL,
             cwd           TEXT NOT NULL,
             scroll_back   TEXT NOT NULL DEFAULT '[]',
             created_at    INTEGER NOT NULL,
             FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_tabs_workspace_created
             ON tabs(workspace_id, created_at DESC);
         PRAGMA user_version = 5;",
    )
    .map_err(|e| DbError::Migration {
        version: 5,
        reason: e.to_string(),
    })?;
    Ok(())
}

fn migrate_v6(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS panes (
             pane_id      TEXT PRIMARY KEY,
             tab_id       TEXT NOT NULL,
             shell        TEXT NOT NULL,
             cwd          TEXT NOT NULL,
             scroll_back  TEXT NOT NULL DEFAULT '[]',
             created_at   INTEGER NOT NULL,
             FOREIGN KEY (tab_id) REFERENCES tabs(tab_id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_panes_tab_created
             ON panes(tab_id, created_at DESC);",
    )
    .map_err(|e| DbError::Migration {
        version: 6,
        reason: e.to_string(),
    })?;

    if !column_exists(conn, "tabs", "layout")? {
        conn.execute(
            "ALTER TABLE tabs ADD COLUMN layout TEXT NOT NULL DEFAULT '{\"kind\":\"single\",\"paneId\":\"\"}'",
            [],
        )
        .map_err(|e| DbError::Migration {
            version: 6,
            reason: e.to_string(),
        })?;
    }

    if !column_exists(conn, "tabs", "focused_pane_id")? {
        conn.execute("ALTER TABLE tabs ADD COLUMN focused_pane_id TEXT", [])
            .map_err(|e| DbError::Migration {
                version: 6,
                reason: e.to_string(),
            })?;
    }

    conn.execute_batch("PRAGMA user_version = 6;")
        .map_err(|e| DbError::Migration {
            version: 6,
            reason: e.to_string(),
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_pool() -> (TempDir, DbPool) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let pool = open_pool(&db_path).unwrap();
        (dir, pool)
    }

    #[test]
    fn fresh_db_creates_schema() {
        let (dir, pool) = test_pool();
        let conn = pool.get().unwrap();
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        let table_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='workspaces'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        drop(conn);
        assert!(table_exists);
        drop(pool);
        drop(dir);
    }

    #[test]
    fn v1_migration_creates_base_table() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v1test.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();

        let has_workspaces: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='workspaces'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_workspaces);

        let has_git: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_table_info('workspaces') WHERE name='has_git'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!has_git);

        drop(conn);
        drop(dir);
    }

    #[test]
    fn v2_migration_adds_git_columns() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v2test.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();

        let has_git: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_table_info('workspaces') WHERE name='has_git'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_git);

        let has_repo_root: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_table_info('workspaces') WHERE name='repo_root'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_repo_root);

        drop(conn);
        drop(dir);
    }

    #[test]
    fn v3_migration_adds_layout_and_settings() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v3test.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();

        let has_layout_state: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_table_info('workspaces') WHERE name='layout_state'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_layout_state);

        let has_settings: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='app_settings'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(has_settings);

        drop(conn);
        drop(dir);
    }

    #[test]
    fn idempotent_migration() {
        let (dir, pool) = test_pool();
        let conn = pool.get().unwrap();
        run_migrations(&conn).unwrap();
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);
        drop(conn);
        drop(pool);
        drop(dir);
    }

    #[test]
    fn v4_migration_creates_index() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v4test.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();

        let index_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_workspaces_last_opened'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            index_exists,
            "idx_workspaces_last_opened should exist after v4 migration"
        );

        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 4);
        drop(conn);
        drop(dir);
    }

    #[test]
    fn v4_index_creation_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v4idemtest.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();

        conn.execute("DROP INDEX idx_workspaces_last_opened", [])
            .unwrap();
        let index_dropped: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_workspaces_last_opened'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!index_dropped, "index should be gone after DROP");

        conn.execute(
            "CREATE INDEX idx_workspaces_last_opened ON workspaces(last_opened DESC)",
            [],
        )
        .unwrap();
        let index_recreated: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_workspaces_last_opened'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_recreated, "index should exist after re-CREATE");

        migrate_v4(&conn).unwrap();
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 4, "re-running v4 migration should be idempotent");
        drop(conn);
        drop(dir);
    }

    #[test]
    fn v5_migration_creates_tabs_table() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v5test.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();
        migrate_v5(&conn).unwrap();

        let table_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='tabs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists, "tabs table should exist after v5 migration");

        let index_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_tabs_workspace_created'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            index_exists,
            "idx_tabs_workspace_created should exist after v5 migration"
        );

        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
        drop(conn);
        drop(dir);
    }

    #[test]
    fn v5_tabs_table_has_fk_and_columns() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v5fktest.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();
        migrate_v5(&conn).unwrap();

        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(tabs)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap();
        assert!(cols.contains(&"tab_id".to_string()));
        assert!(cols.contains(&"workspace_id".to_string()));
        assert!(cols.contains(&"name".to_string()));
        assert!(cols.contains(&"shell".to_string()));
        assert!(cols.contains(&"cwd".to_string()));
        assert!(cols.contains(&"scroll_back".to_string()));
        assert!(cols.contains(&"created_at".to_string()));

        let fk_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_foreign_key_list('tabs')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(fk_exists, "tabs should have FK referencing workspaces");

        drop(conn);
        drop(dir);
    }

    #[test]
    fn v5_migration_idempotent() {
        let (dir, pool) = test_pool();
        let conn = pool.get().unwrap();

        run_migrations(&conn).unwrap();
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        conn.execute("DROP INDEX IF EXISTS idx_panes_tab_created", [])
            .unwrap();
        conn.execute("DROP TABLE IF EXISTS panes", []).unwrap();
        conn.execute("DROP INDEX idx_tabs_workspace_created", [])
            .unwrap();
        conn.execute("DROP TABLE tabs", []).unwrap();

        conn.execute(
            "CREATE TABLE tabs (tab_id TEXT PRIMARY KEY, workspace_id TEXT NOT NULL, name TEXT NOT NULL, shell TEXT NOT NULL, cwd TEXT NOT NULL, scroll_back TEXT NOT NULL DEFAULT '[]', created_at INTEGER NOT NULL, FOREIGN KEY (workspace_id) REFERENCES workspaces(workspace_id) ON DELETE CASCADE)",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE INDEX idx_tabs_workspace_created ON tabs(workspace_id, created_at DESC)",
            [],
        )
        .unwrap();

        migrate_v5(&conn).unwrap();

        let table_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='tabs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            table_exists,
            "tabs table should still exist after re-running v5"
        );

        let version2: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(
            version2, 5,
            "user_version should remain 5 after idempotent run"
        );

        drop(conn);
        drop(pool);
        drop(dir);
    }

    #[test]
    fn v6_migration_creates_panes_table_and_tab_columns() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v6test.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();
        migrate_v5(&conn).unwrap();
        migrate_v6(&conn).unwrap();

        let panes_table_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='panes'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(panes_table_exists, "panes table should exist after v6");

        let panes_index_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM sqlite_master WHERE type='index' AND name='idx_panes_tab_created'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            panes_index_exists,
            "idx_panes_tab_created should exist after v6"
        );

        let tab_cols: Vec<String> = conn
            .prepare("PRAGMA table_info(tabs)")
            .and_then(|mut stmt| {
                stmt.query_map([], |row| row.get::<_, String>(1))
                    .map(|rows| rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap();
        assert!(tab_cols.contains(&"layout".to_string()));
        assert!(tab_cols.contains(&"focused_pane_id".to_string()));

        let panes_fk_exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM pragma_foreign_key_list('panes') WHERE \"table\" = 'tabs'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(panes_fk_exists, "panes should have FK referencing tabs");

        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 6);
        drop(conn);
        drop(dir);
    }

    #[test]
    fn v6_migration_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v6idemtest.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();
        migrate_v5(&conn).unwrap();
        migrate_v6(&conn).unwrap();
        migrate_v6(&conn).unwrap();

        let layout_col_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('tabs') WHERE name = 'layout'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let focused_col_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('tabs') WHERE name = 'focused_pane_id'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(layout_col_count, 1, "layout column should not duplicate");
        assert_eq!(
            focused_col_count, 1,
            "focused_pane_id column should not duplicate"
        );

        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 6);
        drop(conn);
        drop(dir);
    }

    #[test]
    fn v5_to_v6_preserves_existing_tabs() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("v5-to-v6.db");
        let conn = Connection::open(&db_path).unwrap();
        migrate_v1(&conn).unwrap();
        migrate_v2(&conn).unwrap();
        migrate_v3(&conn).unwrap();
        migrate_v4(&conn).unwrap();
        migrate_v5(&conn).unwrap();

        conn.execute(
            "INSERT INTO workspaces (workspace_id, name, path, has_git, repo_root, created_at, last_opened)
             VALUES ('w1', 'Workspace', '/tmp/vibestation', 0, NULL, 1, 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tabs (tab_id, workspace_id, name, shell, cwd, scroll_back, created_at)
             VALUES ('t1', 'w1', 'Tab', '/bin/zsh', '/tmp', '[\"line\"]', 2)",
            [],
        )
        .unwrap();

        migrate_v6(&conn).unwrap();

        let row: (
            String,
            String,
            String,
            String,
            String,
            i64,
            String,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT tab_id, workspace_id, name, shell, cwd, created_at, layout, focused_pane_id
                 FROM tabs WHERE tab_id = 't1'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(row.0, "t1");
        assert_eq!(row.1, "w1");
        assert_eq!(row.2, "Tab");
        assert_eq!(row.3, "/bin/zsh");
        assert_eq!(row.4, "/tmp");
        assert_eq!(row.5, 2);
        assert_eq!(row.6, r#"{"kind":"single","paneId":""}"#);
        assert_eq!(row.7, None);

        let scroll_back: String = conn
            .query_row(
                "SELECT scroll_back FROM tabs WHERE tab_id = 't1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(scroll_back, r#"["line"]"#);
        drop(conn);
        drop(dir);
    }

    #[test]
    fn v5_fk_violation_rejected() {
        let (dir, pool) = test_pool();
        let conn = pool.get().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        let result = conn.execute(
            "INSERT INTO tabs (tab_id, workspace_id, name, shell, cwd, created_at) VALUES ('t1', 'nonexistent-ws', 'tab', 'zsh', '/tmp', 0)",
            [],
        );
        assert!(result.is_err(), "FK violation should be rejected");

        drop(conn);
        drop(pool);
        drop(dir);
    }
}
