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

const CURRENT_SCHEMA_VERSION: u32 = 2;

/// Open or create the database at `db_path`, run migrations, return a connection pool.
pub fn open_pool(db_path: &std::path::Path) -> Result<DbPool, DbError> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DbError::Connection(format!("cannot create db parent dir: {e}")))?;
    }

    let manager = SqliteConnectionManager::file(db_path);
    let pool = Pool::builder()
        .max_size(4)
        .build(manager)
        .map_err(DbError::from)?;

    let conn = pool.get().map_err(DbError::from)?;
    run_migrations(&conn)?;

    Ok(pool)
}

fn run_migrations(conn: &Connection) -> Result<(), DbError> {
    let user_version: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(DbError::from)?;

    if user_version < 1 {
        migrate_v1(conn)?;
    }
    if user_version < 2 {
        migrate_v2(conn)?;
    }

    Ok(())
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

fn migrate_v2(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        "ALTER TABLE workspaces ADD COLUMN has_git INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE workspaces ADD COLUMN repo_root TEXT;
        PRAGMA user_version = 2;",
    )
    .map_err(|e| DbError::Migration {
        version: 2,
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
}
