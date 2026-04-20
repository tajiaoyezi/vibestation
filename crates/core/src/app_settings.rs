use crate::db::{DbError, DbPool};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("setting not found: {0}")]
    NotFound(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
}

pub struct AppSettingsStore;

impl AppSettingsStore {
    pub fn get(pool: &DbPool, key: &str) -> Result<String, SettingsError> {
        let conn = pool.get().map_err(DbError::from)?;
        let value: String = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => SettingsError::NotFound(key.to_string()),
                other => SettingsError::Db(DbError::Query(other.to_string())),
            })?;
        Ok(value)
    }

    pub fn set(pool: &DbPool, key: &str, value: &str) -> Result<(), SettingsError> {
        let conn = pool.get().map_err(DbError::from)?;
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        )
        .map_err(DbError::from)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    fn setup() -> (TempDir, DbPool) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test_settings.db");
        let pool = db::open_pool(&db_path).unwrap();
        (dir, pool)
    }

    #[test]
    fn get_nonexistent_key_errors() {
        let (_dir, pool) = setup();
        let result = AppSettingsStore::get(&pool, "theme");
        assert!(matches!(result, Err(SettingsError::NotFound(_))));
    }

    #[test]
    fn set_and_get_roundtrip() {
        let (_dir, pool) = setup();
        AppSettingsStore::set(&pool, "theme", "dark").unwrap();
        let val = AppSettingsStore::get(&pool, "theme").unwrap();
        assert_eq!(val, "dark");
    }

    #[test]
    fn set_overwrites_previous() {
        let (_dir, pool) = setup();
        AppSettingsStore::set(&pool, "theme", "dark").unwrap();
        AppSettingsStore::set(&pool, "theme", "light").unwrap();
        let val = AppSettingsStore::get(&pool, "theme").unwrap();
        assert_eq!(val, "light");
    }

    #[test]
    fn multiple_keys_independent() {
        let (_dir, pool) = setup();
        AppSettingsStore::set(&pool, "theme", "auto").unwrap();
        AppSettingsStore::set(&pool, "fontSize", "14").unwrap();
        assert_eq!(AppSettingsStore::get(&pool, "theme").unwrap(), "auto");
        assert_eq!(AppSettingsStore::get(&pool, "fontSize").unwrap(), "14");
    }
}
