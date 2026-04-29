use crate::db::{DbError, DbPool};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub font_family: String,
    pub font_size: u32,
    pub default_shell: String,
    pub paste_protection: bool,
    pub telemetry_opt_in: Option<bool>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    #[ts(type = "number")]
    pub bg_opacity: f32,
    #[ts(type = "number")]
    pub bg_blur: u32,
    #[ts(type = "number")]
    pub window_padding_x: u32,
    #[ts(type = "number")]
    pub window_padding_y: u32,
    pub cursor_style: String,
    pub cursor_blink: bool,
    #[ts(type = "number")]
    pub unfocused_pane_opacity: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            font_family: "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, ui-monospace, Liberation Mono, monospace".to_string(),
            font_size: 14,
            default_shell: if cfg!(target_os = "macos") {
                "/bin/zsh".to_string()
            } else {
                "/bin/bash".to_string()
            },
            paste_protection: true,
            telemetry_opt_in: None,
            git_user_name: None,
            git_user_email: None,
            bg_opacity: 0.85,
            bg_blur: 20,
            window_padding_x: 2,
            window_padding_y: 2,
            cursor_style: "block".to_string(),
            cursor_blink: false,
            unfocused_pane_opacity: 0.7,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdateRequest {
    pub theme: Option<String>,
    pub font_family: Option<String>,
    pub font_size: Option<u32>,
    pub default_shell: Option<String>,
    pub paste_protection: Option<bool>,
    pub telemetry_opt_in: Option<bool>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    pub bg_opacity: Option<f32>,
    pub bg_blur: Option<u32>,
    pub window_padding_x: Option<u32>,
    pub window_padding_y: Option<u32>,
    pub cursor_style: Option<String>,
    pub cursor_blink: Option<bool>,
    pub unfocused_pane_opacity: Option<f32>,
}

fn get_parsed<T: std::str::FromStr>(pool: &DbPool, key: &str, default: &str) -> T
where
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let raw = AppSettingsStore::get(pool, key).unwrap_or_else(|_| default.to_string());
    raw.parse().unwrap_or_else(|_| default.parse().unwrap())
}

fn get_optional_string(pool: &DbPool, key: &str) -> Option<String> {
    match AppSettingsStore::get(pool, key) {
        Ok(v) if v.is_empty() => None,
        Ok(v) => Some(v),
        Err(_) => None,
    }
}

fn get_optional_bool(pool: &DbPool, key: &str) -> Option<bool> {
    match AppSettingsStore::get(pool, key) {
        Ok(v) if v.is_empty() => None,
        Ok(v) => Some(v == "true"),
        Err(_) => None,
    }
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

    pub fn get_all(pool: &DbPool) -> AppSettings {
        let theme = get_parsed(pool, "theme", "auto");
        let font_family = get_parsed(pool, "font_family", "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, ui-monospace, Liberation Mono, monospace");
        let font_size: u32 = get_parsed(pool, "font_size", "14");
        let default_shell = get_parsed(
            pool,
            "default_shell",
            if cfg!(target_os = "macos") { "/bin/zsh" } else { "/bin/bash" },
        );
        let paste_protection: bool = get_parsed(pool, "paste_protection", "true");
        let telemetry_opt_in = get_optional_bool(pool, "telemetry_opt_in");
        let git_user_name = get_optional_string(pool, "git_user_name");
        let git_user_email = get_optional_string(pool, "git_user_email");
        let bg_opacity: f32 = get_parsed(pool, "bg_opacity", "0.85");
        let bg_blur: u32 = get_parsed(pool, "bg_blur", "20");
        let window_padding_x: u32 = get_parsed(pool, "window_padding_x", "2");
        let window_padding_y: u32 = get_parsed(pool, "window_padding_y", "2");
        let cursor_style = get_parsed(pool, "cursor_style", "block");
        let cursor_blink: bool = get_parsed(pool, "cursor_blink", "false");
        let unfocused_pane_opacity: f32 = get_parsed(pool, "unfocused_pane_opacity", "0.7");

        AppSettings {
            theme,
            font_family,
            font_size,
            default_shell,
            paste_protection,
            telemetry_opt_in,
            git_user_name,
            git_user_email,
            bg_opacity,
            bg_blur,
            window_padding_x,
            window_padding_y,
            cursor_style,
            cursor_blink,
            unfocused_pane_opacity,
        }
    }

    pub fn update(pool: &DbPool, req: &SettingsUpdateRequest) -> Result<(), SettingsError> {
        if let Some(ref v) = req.theme {
            Self::set(pool, "theme", v)?;
        }
        if let Some(ref v) = req.font_family {
            Self::set(pool, "font_family", v)?;
        }
        if let Some(v) = req.font_size {
            Self::set(pool, "font_size", &v.to_string())?;
        }
        if let Some(ref v) = req.default_shell {
            Self::set(pool, "default_shell", v)?;
        }
        if let Some(v) = req.paste_protection {
            Self::set(pool, "paste_protection", &v.to_string())?;
        }
        if let Some(v) = req.telemetry_opt_in {
            Self::set(pool, "telemetry_opt_in", &v.to_string())?;
        }
        if let Some(ref v) = req.git_user_name {
            Self::set(pool, "git_user_name", v)?;
        }
        if let Some(ref v) = req.git_user_email {
            Self::set(pool, "git_user_email", v)?;
        }
        if let Some(v) = req.bg_opacity {
            Self::set(pool, "bg_opacity", &v.to_string())?;
        }
        if let Some(v) = req.bg_blur {
            Self::set(pool, "bg_blur", &v.to_string())?;
        }
        if let Some(v) = req.window_padding_x {
            Self::set(pool, "window_padding_x", &v.to_string())?;
        }
        if let Some(v) = req.window_padding_y {
            Self::set(pool, "window_padding_y", &v.to_string())?;
        }
        if let Some(ref v) = req.cursor_style {
            Self::set(pool, "cursor_style", v)?;
        }
        if let Some(v) = req.cursor_blink {
            Self::set(pool, "cursor_blink", &v.to_string())?;
        }
        if let Some(v) = req.unfocused_pane_opacity {
            Self::set(pool, "unfocused_pane_opacity", &v.to_string())?;
        }
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

    #[test]
    fn get_all_returns_defaults_when_empty() {
        let (_dir, pool) = setup();
        let settings = AppSettingsStore::get_all(&pool);
        assert_eq!(settings.theme, "auto");
        assert_eq!(settings.font_family, "JetBrains Mono");
        assert_eq!(settings.font_size, 14);
        assert!((settings.bg_opacity - 0.85).abs() < f32::EPSILON);
        assert_eq!(settings.bg_blur, 20);
        assert_eq!(settings.window_padding_x, 2);
        assert_eq!(settings.window_padding_y, 2);
        assert_eq!(settings.cursor_style, "block");
        assert!(!settings.cursor_blink);
        assert!((settings.unfocused_pane_opacity - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn update_writes_only_provided_fields() {
        let (_dir, pool) = setup();
        AppSettingsStore::set(&pool, "theme", "light").unwrap();
        AppSettingsStore::set(&pool, "bg_opacity", "0.5").unwrap();

        let req = SettingsUpdateRequest {
            bg_opacity: Some(0.9),
            ..Default::default()
        };
        AppSettingsStore::update(&pool, &req).unwrap();

        assert_eq!(AppSettingsStore::get(&pool, "theme").unwrap(), "light");
        let val = AppSettingsStore::get(&pool, "bg_opacity").unwrap();
        assert!((val.parse::<f32>().unwrap() - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn settings_persist_across_pool_reopen() {
        // MVP-11 Phase 4 §D.6 · 模拟应用重启 · pool drop 后重新打开同一 db
        // 必须读回先前写入的 7 字段（不依赖 in-memory · 走真实 sqlite 文件）
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("persist_settings.db");

        {
            let pool = db::open_pool(&db_path).unwrap();
            let req = SettingsUpdateRequest {
                theme: Some("light".into()),
                font_size: Some(16),
                bg_opacity: Some(0.5),
                bg_blur: Some(40),
                window_padding_x: Some(8),
                window_padding_y: Some(6),
                cursor_style: Some("bar".into()),
                cursor_blink: Some(true),
                unfocused_pane_opacity: Some(0.3),
                ..Default::default()
            };
            AppSettingsStore::update(&pool, &req).unwrap();
        } // pool drop · sqlite file handle close

        let pool2 = db::open_pool(&db_path).unwrap();
        let s = AppSettingsStore::get_all(&pool2);
        assert_eq!(s.theme, "light");
        assert_eq!(s.font_size, 16);
        assert!((s.bg_opacity - 0.5).abs() < f32::EPSILON);
        assert_eq!(s.bg_blur, 40);
        assert_eq!(s.window_padding_x, 8);
        assert_eq!(s.window_padding_y, 6);
        assert_eq!(s.cursor_style, "bar");
        assert!(s.cursor_blink);
        assert!((s.unfocused_pane_opacity - 0.3).abs() < f32::EPSILON);
    }
}
