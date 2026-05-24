//! Ghostty TOML 配置 parser
//!
//! 路径优先级（spec §Acceptance A）：
//!   1. ~/.config/ghostty/config
//!   2. ~/Library/Application Support/com.mitchellh.ghostty/config（macOS fallback）
//!
//! 两路径都存在时优先前者。
//!
//! schema 可能演进 · 未知字段用 `#[serde(default)]` 跳过 + tracing::warn 记录（spec §已知风险）

use super::ipc::ThemeMode;
use super::{ConfigImportError, ImportSource, ImportedField, RawScanResult};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct GhosttyConfig {
    font_family: Option<String>,
    font_size: Option<f32>,
    theme: Option<String>,
    shell: Option<String>,
}

/// 扫描 Ghostty 配置 · 返回结构化结果
pub fn scan(home: &Path) -> RawScanResult {
    let primary = home.join(".config/ghostty/config");
    let fallback = home.join("Library/Application Support/com.mitchellh.ghostty/config");
    let path = if primary.exists() {
        Some(primary)
    } else if fallback.exists() {
        Some(fallback)
    } else {
        None
    };
    match &path {
        Some(p) => parse_file(p).map_or_else(
            |e| RawScanResult {
                source: ImportSource::Ghostty,
                path: Some(p.clone()),
                path_exists: true,
                detected_fields: Vec::new(),
                errors: vec![e.to_string()],
            },
            |(fields, warnings)| RawScanResult {
                source: ImportSource::Ghostty,
                path: Some(p.clone()),
                path_exists: true,
                detected_fields: fields,
                // Issue #206 Important 1 修复：parser warnings（theme 不支持等非致命 reject）
                // 流到 errors 字段 · 让 UI 显示 "为什么这个字段没出现在 preview"
                errors: warnings,
            },
        ),
        None => RawScanResult {
            source: ImportSource::Ghostty,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        },
    }
}

/// 返回 (fields, warnings) · warnings 是非致命 reject（例 theme 不支持）
fn parse_file(path: &Path) -> Result<(Vec<ImportedField>, Vec<String>), ConfigImportError> {
    let content = std::fs::read_to_string(path)?;
    // 去掉 keybind 行 · 让 toml 能正常解析 font/theme/shell（不受重复 key 影响）
    let sanitized: String = content
        .lines()
        .filter(|line| !line.trim().starts_with("keybind"))
        .collect::<Vec<_>>()
        .join("\n");
    let (toml_fields, warnings) = match toml::from_str::<GhosttyConfig>(&sanitized) {
        Ok(cfg) => config_to_fields(cfg),
        Err(e) => {
            // toml 解析失败（非 keybind 原因）· 降级只取 keybinds
            let err_fields = parse_keybinds_raw(&content);
            if err_fields.is_empty() {
                return Err(ConfigImportError::Toml(e.to_string()));
            }
            // Issue #206 Advisory #5 修复：降级路径下 TOML error 不能丢弃 · 推到
            // warnings 让 UI 知道为什么 font/theme/shell 没出现（之前 silent Ok 用户
            // 困惑 "我配了 font_family 但 preview 里没有"）
            return Ok((
                err_fields,
                vec![format!(
                    "toml parse failed · only keybinds extracted (font/theme/shell unavailable): {e}"
                )],
            ));
        }
    };
    let mut all = toml_fields;
    all.extend(parse_keybinds_raw(&content));
    Ok((all, warnings))
}

fn parse_keybinds_raw(content: &str) -> Vec<ImportedField> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.is_empty() {
                return None;
            }
            let rest = trimmed
                .strip_prefix("keybind")?
                .trim_start()
                .strip_prefix('=')?
                .trim();
            let (key, action) = rest.split_once('=')?;
            Some(ImportedField::KeyBinding {
                key: key.trim().to_string(),
                action: action.trim().to_string(),
            })
        })
        .collect()
}

/// 返回 (detected_fields, parser_warnings) · warnings 是非致命的字段级 reject 原因
/// （例 theme = "tokyo-night" · vibestation 只支持 light/dark/auto · v0.2+ 评估 palette）
fn config_to_fields(cfg: GhosttyConfig) -> (Vec<ImportedField>, Vec<String>) {
    let mut fields = Vec::new();
    let mut warnings = Vec::new();
    if let Some(f) = cfg.font_family {
        fields.push(ImportedField::FontFamily(f));
    }
    if let Some(s) = cfg.font_size {
        fields.push(ImportedField::FontSize(s));
    }
    if let Some(t) = cfg.theme {
        // Issue #206 Important 1 修复：parser 层 reject 非 light/dark/auto · 不进 detected_fields
        // · 改前任意 String 进 ImportedField · 到 apply 才 reject
        match ThemeMode::parse_normalized(&t) {
            Ok(mode) => fields.push(ImportedField::Theme(mode)),
            Err(rejected) => warnings.push(format!(
                "theme '{rejected}' not supported (only light/dark/auto · v0.2+ evaluate palette import)"
            )),
        }
    }
    if let Some(s) = cfg.shell {
        fields.push(ImportedField::Shell(s));
    }
    (fields, warnings)
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
    fn scan_primary_path_exists() {
        let tmp = tempfile::tempdir().unwrap();
        // Issue #206 Important 1 修复后：theme 必须是 light/dark/auto · 用 "dark"
        // 保持原意（3 fields detected + 0 errors）· 另加 reject 测试见下
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
            font_family = "JetBrains Mono"
            font_size = 14
            theme = "dark"
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 3);
        assert!(r.errors.is_empty());
        assert!(
            matches!(r.detected_fields[0], ImportedField::FontFamily(ref f) if f == "JetBrains Mono")
        );
    }

    #[test]
    fn scan_fallback_path_when_primary_absent() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            "Library/Application Support/com.mitchellh.ghostty/config",
            r#"
            font_size = 16
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 1);
    }

    #[test]
    fn scan_both_paths_prefers_primary() {
        let tmp = tempfile::tempdir().unwrap();
        // Issue #206 Important 1: theme 必须是合法 ThemeMode · 用 dark 表 primary · light 表 fallback
        write_fixture(tmp.path(), ".config/ghostty/config", r#"theme = "dark""#);
        write_fixture(
            tmp.path(),
            "Library/Application Support/com.mitchellh.ghostty/config",
            r#"theme = "light""#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(matches!(
            &r.detected_fields[0],
            ImportedField::Theme(ThemeMode::Dark)
        ));
    }

    /// Issue #206 Important 1 修复：parser 层 reject 非 light/dark/auto 的 theme
    /// · 不进 detected_fields · 进 errors（warnings）让 UI 显示拒绝原因
    #[test]
    fn scan_rejects_unsupported_theme_palette_name() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
            font_family = "JetBrains Mono"
            theme = "tokyo-night"
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // font_family 仍 detected · theme 被 reject 进 errors
        assert_eq!(r.detected_fields.len(), 1);
        assert!(matches!(r.detected_fields[0], ImportedField::FontFamily(_)));
        assert!(
            r.errors
                .iter()
                .any(|e| e.contains("tokyo-night") && e.contains("not supported")),
            "errors 应含 'tokyo-night not supported' · 实际={:?}",
            r.errors
        );
    }

    /// Issue #206 Advisory #5 修复：TOML parse fail 但有 keybinds 时 · 降级路径
    /// 必须把 TOML error 推到 errors（warnings）· 不能 silent Ok · 否则用户不知道
    /// 为什么 font/theme/shell 没出现
    #[test]
    fn scan_broken_toml_with_keybinds_reports_warning() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            "this is not valid {{{ toml\nkeybind = ctrl+a=action_a",
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // keybind 仍能 detect
        let kb_count = r
            .detected_fields
            .iter()
            .filter(|f| matches!(f, ImportedField::KeyBinding { .. }))
            .count();
        assert_eq!(
            kb_count, 1,
            "keybind 应被降级路径 detect · 实际={:?}",
            r.detected_fields
        );
        // errors 必须含 TOML fail 说明（之前是 silent Ok · 现在 push warning）
        assert!(
            r.errors
                .iter()
                .any(|e| e.contains("toml") && e.contains("font/theme/shell unavailable")),
            "errors 应含 'toml parse failed · font/theme/shell unavailable' · 实际={:?}",
            r.errors
        );
    }

    /// Issue #206 Important 1 修复：合法 theme 含大小写空格也能解析
    #[test]
    fn scan_accepts_theme_case_insensitive_with_whitespace() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"theme = "  Dark  ""#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 1);
        assert!(matches!(
            r.detected_fields[0],
            ImportedField::Theme(ThemeMode::Dark)
        ));
        assert!(r.errors.is_empty());
    }

    #[test]
    fn scan_broken_toml_graceful_error() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"this is not valid {{{ toml"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(r.detected_fields.is_empty());
        assert_eq!(r.errors.len(), 1);
        assert!(r.errors[0].contains("toml"));
    }

    #[test]
    fn scan_no_file_not_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let r = scan(tmp.path());
        assert!(!r.path_exists);
        assert!(r.detected_fields.is_empty());
        assert!(r.errors.is_empty());
    }

    #[test]
    fn scan_unknown_fields_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
            font_family = "JetBrains Mono"
            future_unknown_field = "should be skipped"
            another_new_field = 42
        "#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert_eq!(r.detected_fields.len(), 1);
        assert!(r.errors.is_empty());
    }

    #[test]
    fn scan_keybinds_multiple_lines() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
keybind = cmd+t=new_tab
keybind = cmd+w=close_tab
keybind = cmd+shift+[=previous_tab
"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // toml parse 因重复 key 降级 · 只取 keybinds
        assert_eq!(r.detected_fields.len(), 3);
        assert!(r.errors.is_empty());
        assert!(
            matches!(&r.detected_fields[0], ImportedField::KeyBinding { key, action } if key == "cmd+t" && action == "new_tab")
        );
    }

    #[test]
    fn scan_keybinds_with_font() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
font_family = "JetBrains Mono"
keybind = cmd+t=new_tab
font_size = 14
keybind = cmd+w=close_tab
"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // toml 先解析 font · 再手动 merge keybinds
        assert_eq!(r.detected_fields.len(), 4);
        assert!(r.errors.is_empty());
        assert!(
            matches!(&r.detected_fields[0], ImportedField::FontFamily(ref f) if f == "JetBrains Mono")
        );
        assert!(
            matches!(&r.detected_fields[2], ImportedField::KeyBinding { key, action } if key == "cmd+t" && action == "new_tab")
        );
    }

    #[test]
    fn scan_keybinds_comment_skip() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
# keybind = cmd+t=should_be_skipped
keybind = cmd+t=new_tab
"#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        // 注释行被跳过 · 只有 1 个有效 keybind
        assert_eq!(r.detected_fields.len(), 1);
        assert!(
            matches!(&r.detected_fields[0], ImportedField::KeyBinding { key, action } if key == "cmd+t" && action == "new_tab")
        );
    }
}
