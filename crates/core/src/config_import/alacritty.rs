//! Alacritty 配置 parser · TOML 0.14+ 优先 · YAML 0.13- fallback
//!
//! 路径优先级（spec §Acceptance C · task-3.2 加 Windows 分支）：
//!   - 非 Windows（macOS/Linux）：
//!     1. ~/.config/alacritty/alacritty.toml（0.14+）
//!     2. ~/.config/alacritty/alacritty.yml（0.13- 已 deprecated 但仍有用户）
//!   - Windows：
//!     1. %APPDATA%/alacritty/alacritty.toml
//!     2. %APPDATA%/alacritty/alacritty.yml
//!     3. ~/.config/alacritty/alacritty.toml（WSL fallback）
//!     4. ~/.config/alacritty/alacritty.yml（WSL fallback）
//!
//! 字段：font.normal.family / font.size / colors / key_bindings
//! key_bindings action 映射延后到 Phase B（如 Alacritty SpawnNewInstance → 无映射 warn）

use super::{ConfigImportError, ImportSource, ImportedField, RawScanResult};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AlacrittyConfig {
    font: Option<AlacrittyFont>,
    keyboard: Option<AlacrittyKeyboard>,
    #[serde(rename = "key_bindings")]
    key_bindings: Option<Vec<AlacrittyBinding>>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AlacrittyKeyboard {
    bindings: Vec<AlacrittyBinding>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AlacrittyBinding {
    key: Option<String>,
    mods: Option<String>,
    action: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AlacrittyFont {
    normal: Option<AlacrittyFontFamily>,
    size: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AlacrittyFontFamily {
    family: Option<String>,
}

/// 构造候选配置路径（按优先级）· 每项 `(path, is_yaml)` · 平台分支 + 可注入 `appdata`。
///
/// - 非 Windows：`~/.config/alacritty/alacritty.toml` → `.yml`
/// - Windows：`%APPDATA%/alacritty/alacritty.toml` → `.yml` → WSL `~/.config/...` toml → yml
///
/// `appdata` 仅在 Windows 分支使用（None 时跳过 `%APPDATA%` 候选）。
#[cfg_attr(not(windows), allow(unused_variables))]
fn candidates_for(home: &Path, appdata: Option<PathBuf>) -> Vec<(PathBuf, bool)> {
    let dotconfig = vec![
        (home.join(".config/alacritty/alacritty.toml"), false),
        (home.join(".config/alacritty/alacritty.yml"), true),
    ];
    #[cfg(windows)]
    {
        let mut v = Vec::with_capacity(4);
        if let Some(appdata) = appdata {
            v.push((appdata.join("alacritty/alacritty.toml"), false));
            v.push((appdata.join("alacritty/alacritty.yml"), true));
        }
        v.extend(dotconfig); // WSL fallback
        v
    }
    #[cfg(not(windows))]
    {
        dotconfig
    }
}

/// 扫描 Alacritty 配置（读环境变量后委托 [`scan_with_appdata`]）
pub fn scan(home: &Path) -> RawScanResult {
    #[cfg(windows)]
    let appdata = std::env::var("APPDATA").ok().map(PathBuf::from);
    #[cfg(not(windows))]
    let appdata = None;
    scan_with_appdata(home, appdata)
}

/// scan 的可注入实现：`appdata` 显式传入（Windows）· 测试不依赖真实 `%APPDATA%`。
fn scan_with_appdata(home: &Path, appdata: Option<PathBuf>) -> RawScanResult {
    let (path, is_yaml) = candidates_for(home, appdata)
        .into_iter()
        .find(|(p, _)| p.exists())
        .map_or((None, false), |(p, y)| (Some(p), y));
    match &path {
        Some(p) => {
            let result = if is_yaml {
                parse_yaml(p)
            } else {
                parse_toml(p)
            };
            match result {
                Ok(fields) => RawScanResult {
                    source: ImportSource::Alacritty,
                    path: Some(p.clone()),
                    path_exists: true,
                    detected_fields: fields,
                    errors: Vec::new(),
                },
                Err(e) => RawScanResult {
                    source: ImportSource::Alacritty,
                    path: Some(p.clone()),
                    path_exists: true,
                    detected_fields: Vec::new(),
                    errors: vec![e.to_string()],
                },
            }
        }
        None => RawScanResult {
            source: ImportSource::Alacritty,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        },
    }
}

fn parse_toml(path: &Path) -> Result<Vec<ImportedField>, ConfigImportError> {
    let content = std::fs::read_to_string(path)?;
    let cfg: AlacrittyConfig =
        toml::from_str(&content).map_err(|e| ConfigImportError::Toml(e.to_string()))?;
    Ok(config_to_fields(cfg))
}

fn parse_yaml(path: &Path) -> Result<Vec<ImportedField>, ConfigImportError> {
    let content = std::fs::read_to_string(path)?;
    let cfg: AlacrittyConfig =
        serde_yaml::from_str(&content).map_err(|e| ConfigImportError::Yaml(e.to_string()))?;
    Ok(config_to_fields(cfg))
}

fn config_to_fields(cfg: AlacrittyConfig) -> Vec<ImportedField> {
    let mut fields = Vec::new();
    if let Some(font) = cfg.font {
        if let Some(normal) = font.normal {
            if let Some(family) = normal.family {
                fields.push(ImportedField::FontFamily(family));
            }
        }
        if let Some(size) = font.size {
            fields.push(ImportedField::FontSize(size));
        }
    }

    let kb_from_toml = cfg.keyboard.map(|k| k.bindings).unwrap_or_default();
    let kb_from_yaml = cfg.key_bindings.unwrap_or_default();
    let all_bindings: Vec<_> = kb_from_toml.into_iter().chain(kb_from_yaml).collect();

    for binding in all_bindings {
        if let Some(key) = binding.key {
            // review round 5 fix: filter 空 action（含 None 或空字符串）·
            // 避免 ("Cmd+N", "") 这种无意义 binding 静默写 imported_keybindings
            let action = match binding.action {
                Some(a) if !a.trim().is_empty() => a,
                _ => continue, // 无 action · 跳过此 binding
            };
            let combined_key = match binding.mods {
                Some(mods) if !mods.is_empty() => format!("{}+{}", mods, key),
                _ => key,
            };
            fields.push(ImportedField::KeyBinding {
                key: combined_key,
                action,
            });
        }
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fixture(dir: &Path, rel_path: &str, content: &str) -> std::path::PathBuf {
        let full = dir.join(rel_path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(&full, content).unwrap();
        full
    }

    #[test]
    fn scan_toml_preferred_over_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.toml",
            r#"
            [font.normal]
            family = "JetBrains Mono"
            [font]
            size = 13.0
        "#,
        );
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.yml",
            r#"
font:
  normal:
    family: Fira Code
  size: 11.0
"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 2);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::FontFamily(f) if f == "JetBrains Mono")
        );
    }

    #[test]
    fn scan_yaml_fallback_when_toml_absent() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.yml",
            r#"
font:
  normal:
    family: Fira Code
  size: 11.0
"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 2);
        assert!(matches!(&r.detected_fields[0], ImportedField::FontFamily(f) if f == "Fira Code"));
    }

    #[test]
    fn scan_broken_toml_graceful() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.toml",
            "not valid {{{",
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(r.detected_fields.is_empty());
        assert_eq!(r.errors.len(), 1);
        assert!(r.errors[0].contains("toml"));
    }

    #[test]
    fn scan_missing_fields_partial_result() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.toml",
            r#"
            [font]
            size = 12.0
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 1); // 只有 size · family 缺
        assert!(r.errors.is_empty());
    }

    #[test]
    fn scan_toml_keyboard_bindings() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.toml",
            r#"
            [[keyboard.bindings]]
            key = "N"
            mods = "Command"
            action = "CreateNewWindow"

            [[keyboard.bindings]]
            key = "T"
            mods = "Command"
            action = "SpawnNewTab"
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 2);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::KeyBinding { key, action } if key == "Command+N" && action == "CreateNewWindow")
        );
    }

    #[test]
    fn scan_yaml_key_bindings_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.yml",
            r#"
key_bindings:
  - { key: N, mods: Command, action: CreateNewWindow }
  - { key: T, mods: Command, action: SpawnNewTab }
"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 2);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::KeyBinding { key, action } if key == "Command+N" && action == "CreateNewWindow")
        );
    }

    #[test]
    fn scan_bindings_no_mods() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.toml",
            r#"
            [[keyboard.bindings]]
            key = "F11"
            action = "ToggleFullscreen"
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 1);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::KeyBinding { key, action } if key == "F11" && action == "ToggleFullscreen")
        );
    }

    #[test]
    fn scan_bindings_mixed_mods() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/alacritty/alacritty.toml",
            r#"
            [[keyboard.bindings]]
            key = "T"
            mods = "Command|Shift"
            action = "SpawnNewTab"
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 1);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::KeyBinding { key, action } if key == "Command|Shift+T" && action == "SpawnNewTab")
        );
    }

    // ─── task-3.2 · Windows 路径分支 ──────────────────────────────────────

    /// TEST-3.2.1（AC1）：Windows 上 `%APPDATA%/alacritty/alacritty.toml`（含 `[font]`）
    /// 命中 · `path_exists=true` + font 字段被 detect。
    #[cfg(windows)]
    #[test]
    fn test_3_2_1_windows_appdata_toml_detects() {
        let appdata = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        write_fixture(
            appdata.path(),
            "alacritty/alacritty.toml",
            r#"
            [font.normal]
            family = "JetBrains Mono"
            [font]
            size = 13.0
        "#,
        );
        let r = scan_with_appdata(home.path(), Some(appdata.path().to_path_buf()));
        assert!(r.path_exists, "应命中 %APPDATA%/alacritty/alacritty.toml");
        assert_eq!(r.detected_fields.len(), 2);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::FontFamily(f) if f == "JetBrains Mono")
        );
    }

    /// TEST-3.2.1b（AC1）：Windows 上 `%APPDATA%/alacritty/alacritty.yml` fallback 命中。
    #[cfg(windows)]
    #[test]
    fn test_3_2_1b_windows_appdata_yml_fallback() {
        let appdata = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        write_fixture(
            appdata.path(),
            "alacritty/alacritty.yml",
            "font:\n  normal:\n    family: Fira Code\n  size: 11.0\n",
        );
        let r = scan_with_appdata(home.path(), Some(appdata.path().to_path_buf()));
        assert!(r.path_exists, "应命中 %APPDATA%/alacritty/alacritty.yml");
        assert_eq!(r.detected_fields.len(), 2);
        assert!(matches!(&r.detected_fields[0], ImportedField::FontFamily(f) if f == "Fira Code"));
    }

    /// TEST-3.2.2（AC2）：Windows 上 `%APPDATA%` 无配置而 `~/.config/alacritty/alacritty.toml`
    /// （WSL 风格）存在时 · fallback 仍命中。
    #[cfg(windows)]
    #[test]
    fn test_3_2_2_windows_dotconfig_fallback() {
        let appdata = tempfile::tempdir().unwrap(); // 空
        let home = tempfile::tempdir().unwrap();
        write_fixture(
            home.path(),
            ".config/alacritty/alacritty.toml",
            r#"
            [font]
            size = 12.0
        "#,
        );
        let r = scan_with_appdata(home.path(), Some(appdata.path().to_path_buf()));
        assert!(r.path_exists, "WSL fallback ~/.config/alacritty 应命中");
        assert_eq!(r.detected_fields.len(), 1);
    }

    /// AC1 路径构造单测（Windows · 顺序：%APPDATA% toml → %APPDATA% yml → WSL toml → WSL yml）。
    #[cfg(windows)]
    #[test]
    fn test_3_2_1_windows_candidates_order() {
        let home = Path::new("C:\\Users\\alice");
        let appdata = PathBuf::from("C:\\Users\\alice\\AppData\\Roaming");
        let c = candidates_for(home, Some(appdata.clone()));
        assert_eq!(c[0].0, appdata.join("alacritty/alacritty.toml"));
        assert!(!c[0].1, "[0] 是 toml · is_yaml=false");
        assert_eq!(c[1].0, appdata.join("alacritty/alacritty.yml"));
        assert!(c[1].1, "[1] 是 yml · is_yaml=true");
        assert_eq!(c[2].0, home.join(".config/alacritty/alacritty.toml"));
        assert_eq!(c[3].0, home.join(".config/alacritty/alacritty.yml"));
    }
}
