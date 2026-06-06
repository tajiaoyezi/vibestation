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
    /// MVP-20 · 是否启用 PTY 预热池（新 tab 启动加速）
    pub pty_pool_enabled: bool,
    /// MVP-20 · PTY 预热池容量（推荐 1-3 · 实际取值由 UI 限制）
    #[ts(type = "number")]
    pub pty_pool_size: u32,
    /// 全局 sidebar 宽度（像素）· 跨 workspace 共享 · 类似 VSCode/Cursor 设计 ·
    /// 区别于 per-workspace LayoutState 的 open/close。
    #[ts(type = "number")]
    pub primary_width: u32,
    #[ts(type = "number")]
    pub secondary_width: u32,
    #[ts(type = "number")]
    pub bottom_height: u32,
    /// MVP-17 · 用户选择的默认外部终端 ID · None = 每次问
    pub external_term_preferred: Option<String>,
    /// MVP-17 · "Don't ask again" 状态 · true 时跳过 PopToExternalDialog
    pub external_term_dont_ask_again: bool,
}

/// 平台默认 shell（task-1.3 · ADR-003）。
///
/// macOS → `/bin/zsh` · Windows → `cmd.exe`（占位 · 永远保底 · Phase 2 task-2.1
///   `resolve_default_shell` 探测链运行期细化为 pwsh→powershell→cmd）· Linux/其他 → `/bin/bash`。
///
/// 收敛 `impl Default` 与 `get_all` fallback 两处字面值，避免漂移（AC2）。
fn default_shell_for_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "/bin/zsh"
    }
    #[cfg(target_os = "windows")]
    {
        "cmd.exe" // 占位 · 与 ADR-003 探测链对齐 · Phase 2 运行期 resolve 细化
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "/bin/bash"
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_family: "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, ui-monospace, Liberation Mono, Sarasa Term SC, PingFang SC, Hiragino Sans GB, Microsoft YaHei, Noto Sans CJK SC, WenQuanYi Micro Hei, monospace".to_string(),
            font_size: 14,
            default_shell: default_shell_for_platform().to_string(),
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
            pty_pool_enabled: true,
            pty_pool_size: 1,
            // 全局 sidebar 默认值 · 与原 DEFAULT_LAYOUT (web/src/stores/layout.ts) 一致
            primary_width: 236,
            secondary_width: 400,
            bottom_height: 240,
            external_term_preferred: None,
            external_term_dont_ask_again: false,
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
    pub pty_pool_enabled: Option<bool>,
    pub pty_pool_size: Option<u32>,
    pub primary_width: Option<u32>,
    pub secondary_width: Option<u32>,
    pub bottom_height: Option<u32>,
    /// 外层 Some = 请求包含该字段 · 内层 None = 清空偏好（每次询问）
    pub external_term_preferred: Option<Option<String>>,
    pub external_term_dont_ask_again: Option<bool>,
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
        let theme = get_parsed(pool, "theme", "dark");
        let font_family = get_parsed(pool, "font_family", "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, ui-monospace, Liberation Mono, Sarasa Term SC, PingFang SC, Hiragino Sans GB, Microsoft YaHei, Noto Sans CJK SC, WenQuanYi Micro Hei, monospace");
        let font_size: u32 = get_parsed(pool, "font_size", "14");
        let default_shell = get_parsed(pool, "default_shell", default_shell_for_platform());
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
        let pty_pool_enabled: bool = get_parsed(pool, "pty_pool_enabled", "true");
        let pty_pool_size: u32 = get_parsed(pool, "pty_pool_size", "1");
        let primary_width: u32 = get_parsed(pool, "primary_width", "236");
        let secondary_width: u32 = get_parsed(pool, "secondary_width", "400");
        let bottom_height: u32 = get_parsed(pool, "bottom_height", "240");
        let external_term_preferred = get_optional_string(pool, "external_term_preferred");
        let external_term_dont_ask_again: bool =
            get_parsed(pool, "external_term_dont_ask_again", "false");

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
            pty_pool_enabled,
            pty_pool_size,
            primary_width,
            secondary_width,
            bottom_height,
            external_term_preferred,
            external_term_dont_ask_again,
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
        if let Some(v) = req.pty_pool_enabled {
            Self::set(pool, "pty_pool_enabled", &v.to_string())?;
        }
        if let Some(v) = req.pty_pool_size {
            Self::set(pool, "pty_pool_size", &v.to_string())?;
        }
        if let Some(v) = req.primary_width {
            Self::set(pool, "primary_width", &v.to_string())?;
        }
        if let Some(v) = req.secondary_width {
            Self::set(pool, "secondary_width", &v.to_string())?;
        }
        if let Some(v) = req.bottom_height {
            Self::set(pool, "bottom_height", &v.to_string())?;
        }
        if let Some(ref opt) = req.external_term_preferred {
            Self::set(
                pool,
                "external_term_preferred",
                opt.as_deref().unwrap_or(""),
            )?;
        }
        if let Some(v) = req.external_term_dont_ask_again {
            Self::set(pool, "external_term_dont_ask_again", &v.to_string())?;
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
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.font_family, "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, ui-monospace, Liberation Mono, Sarasa Term SC, PingFang SC, Hiragino Sans GB, Microsoft YaHei, Noto Sans CJK SC, WenQuanYi Micro Hei, monospace");
        assert_eq!(settings.font_size, 14);
        assert!((settings.bg_opacity - 0.85).abs() < f32::EPSILON);
        assert_eq!(settings.bg_blur, 20);
        assert_eq!(settings.window_padding_x, 2);
        assert_eq!(settings.window_padding_y, 2);
        assert_eq!(settings.cursor_style, "block");
        assert!(!settings.cursor_blink);
        assert!((settings.unfocused_pane_opacity - 0.7).abs() < f32::EPSILON);
        // MVP-20 · PTY 预热池默认开 · 容量 1
        assert!(settings.pty_pool_enabled);
        assert_eq!(settings.pty_pool_size, 1);
        // MVP-17 Phase E.4 · 外部终端偏好默认
        assert!(settings.external_term_preferred.is_none());
        assert!(!settings.external_term_dont_ask_again);
    }

    #[test]
    fn app_settings_language_default_is_en() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn app_settings_language_get_all_defaults_to_en_when_empty() {
        let (_dir, pool) = setup();
        let settings = AppSettingsStore::get_all(&pool);
        assert_eq!(settings.language, "en");
    }

    #[test]
    fn app_settings_language_persists_across_get_all() {
        let (_dir, pool) = setup();
        let req = SettingsUpdateRequest {
            language: Some("zh-Hans".to_string()),
            ..Default::default()
        };

        AppSettingsStore::update(&pool, &req).expect("language update succeeds");

        let settings = AppSettingsStore::get_all(&pool);
        assert_eq!(settings.language, "zh-Hans");
    }

    #[test]
    fn app_settings_language_persists_across_pool_reopen() {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("persist_language.db");

        {
            let pool = db::open_pool(&db_path).unwrap();
            let req = SettingsUpdateRequest {
                language: Some("zh-Hans".to_string()),
                ..Default::default()
            };
            AppSettingsStore::update(&pool, &req).expect("language update succeeds");
        }

        let pool = db::open_pool(&db_path).unwrap();
        let settings = AppSettingsStore::get_all(&pool);
        assert_eq!(settings.language, "zh-Hans");
    }

    #[test]
    fn app_settings_language_invalid_value_falls_back_to_en() {
        let (_dir, pool) = setup();

        for invalid in ["fr", "", "zh"] {
            AppSettingsStore::set(&pool, "language", invalid).expect("seed invalid language");
            let settings = AppSettingsStore::get_all(&pool);
            assert_eq!(settings.language, "en");
        }
    }

    #[test]
    fn pty_pool_settings_roundtrip() {
        // MVP-20 · 验证 pty_pool_* 字段持久化 · 关闭 + 容量改 3
        let (_dir, pool) = setup();
        let req = SettingsUpdateRequest {
            pty_pool_enabled: Some(false),
            pty_pool_size: Some(3),
            ..Default::default()
        };
        AppSettingsStore::update(&pool, &req).unwrap();

        let s = AppSettingsStore::get_all(&pool);
        assert!(!s.pty_pool_enabled);
        assert_eq!(s.pty_pool_size, 3);
    }

    #[test]
    fn external_term_settings_defaults_and_roundtrip() {
        let (_dir, pool) = setup();
        let s = AppSettingsStore::get_all(&pool);
        assert!(s.external_term_preferred.is_none());
        assert!(!s.external_term_dont_ask_again);

        AppSettingsStore::update(
            &pool,
            &SettingsUpdateRequest {
                external_term_preferred: Some(Some("ghostty".into())),
                external_term_dont_ask_again: Some(true),
                ..Default::default()
            },
        )
        .unwrap();
        let s2 = AppSettingsStore::get_all(&pool);
        assert_eq!(s2.external_term_preferred.as_deref(), Some("ghostty"));
        assert!(s2.external_term_dont_ask_again);

        AppSettingsStore::update(
            &pool,
            &SettingsUpdateRequest {
                external_term_preferred: Some(None),
                ..Default::default()
            },
        )
        .unwrap();
        let s3 = AppSettingsStore::get_all(&pool);
        assert!(s3.external_term_preferred.is_none());
        assert!(s3.external_term_dont_ask_again);
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

    // ─── task-1.3 · 跨平台 default_shell 默认值（SCEN-1.3.1~1.3.4 / AC1~AC5） ───

    /// 当前平台期望的默认 shell（测试断言基准）。
    /// macOS → /bin/zsh · Windows → cmd.exe（占位 · ADR-003）· Linux/其他 → /bin/bash。
    fn expected_default_shell() -> &'static str {
        #[cfg(target_os = "macos")]
        {
            "/bin/zsh"
        }
        #[cfg(target_os = "windows")]
        {
            "cmd.exe"
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            "/bin/bash"
        }
    }

    // SCEN-1.3.1 / AC1 — AppSettings::default().default_shell 三平台各自正确
    #[test]
    fn test_1_3_1_default_shell_per_platform() {
        assert_eq!(
            AppSettings::default().default_shell,
            expected_default_shell(),
            "default() 的 default_shell 应为当前平台默认 shell"
        );
    }

    // SCEN-1.3.2 / AC2 — get_all（DB 无记录）fallback 与 default() 一致
    #[test]
    fn test_1_3_2_get_all_fallback_matches_default() {
        let (_dir, pool) = setup();
        let from_get_all = AppSettingsStore::get_all(&pool).default_shell;
        assert_eq!(
            from_get_all,
            AppSettings::default().default_shell,
            "get_all 的 default_shell fallback 必须与 impl Default 一致（无字面值漂移）"
        );
        assert_eq!(from_get_all, expected_default_shell());
    }

    // SCEN-1.3.3 / AC3 — Windows 默认占位为 cmd.exe（绝不回落 Unix 路径）
    #[cfg(target_os = "windows")]
    #[test]
    fn test_1_3_3_windows_default_is_cmd() {
        let shell = AppSettings::default().default_shell;
        assert_eq!(
            shell, "cmd.exe",
            "Windows 默认 shell 占位应为 cmd.exe（ADR-003）"
        );
        assert!(
            !shell.starts_with("/bin/"),
            "Windows 默认 shell 绝不应是 Unix 路径（/bin/bash 在 Windows 不存在 → PTY spawn 立即失败），实际={shell}"
        );
    }

    // SCEN-1.3.4 / AC4+AC5 — Unix 默认值字节级零回归
    #[cfg(unix)]
    #[test]
    fn test_1_3_4_unix_default_unchanged() {
        let shell = AppSettings::default().default_shell;
        #[cfg(target_os = "macos")]
        assert_eq!(
            shell, "/bin/zsh",
            "macOS 默认 shell 必须保持 /bin/zsh（零回归）"
        );
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            shell, "/bin/bash",
            "Linux 默认 shell 必须保持 /bin/bash（零回归）"
        );
    }
}
