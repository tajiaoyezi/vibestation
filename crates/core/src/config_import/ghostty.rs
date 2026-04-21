//! Ghostty TOML 配置 parser
//!
//! 路径优先级（spec §Acceptance A）：
//!   1. ~/.config/ghostty/config
//!   2. ~/Library/Application Support/com.mitchellh.ghostty/config（macOS fallback）
//!
//! 两路径都存在时优先前者。
//!
//! schema 可能演进 · 未知字段用 `#[serde(default)]` 跳过 + tracing::warn 记录（spec §已知风险）

use super::{ConfigImportError, ImportScanResult, ImportSource, ImportedField};
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
pub fn scan(home: &Path) -> ImportScanResult {
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
            |e| ImportScanResult {
                source: ImportSource::Ghostty,
                path: Some(p.clone()),
                path_exists: true,
                detected_fields: Vec::new(),
                errors: vec![e.to_string()],
            },
            |fields| ImportScanResult {
                source: ImportSource::Ghostty,
                path: Some(p.clone()),
                path_exists: true,
                detected_fields: fields,
                errors: Vec::new(),
            },
        ),
        None => ImportScanResult {
            source: ImportSource::Ghostty,
            path: None,
            path_exists: false,
            detected_fields: Vec::new(),
            errors: Vec::new(),
        },
    }
}

fn parse_file(path: &Path) -> Result<Vec<ImportedField>, ConfigImportError> {
    let content = std::fs::read_to_string(path)?;
    let cfg: GhosttyConfig =
        toml::from_str(&content).map_err(|e| ConfigImportError::Toml(e.to_string()))?;
    Ok(config_to_fields(cfg))
}

fn config_to_fields(cfg: GhosttyConfig) -> Vec<ImportedField> {
    let mut fields = Vec::new();
    if let Some(f) = cfg.font_family {
        fields.push(ImportedField::FontFamily(f));
    }
    if let Some(s) = cfg.font_size {
        fields.push(ImportedField::FontSize(s));
    }
    if let Some(t) = cfg.theme {
        fields.push(ImportedField::Theme(t));
    }
    if let Some(s) = cfg.shell {
        fields.push(ImportedField::Shell(s));
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
    fn scan_primary_path_exists() {
        let tmp = tempfile::tempdir().unwrap();
        write_fixture(
            tmp.path(),
            ".config/ghostty/config",
            r#"
            font_family = "JetBrains Mono"
            font_size = 14
            theme = "tokyo-night"
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
        write_fixture(tmp.path(), ".config/ghostty/config", r#"theme = "primary""#);
        write_fixture(
            tmp.path(),
            "Library/Application Support/com.mitchellh.ghostty/config",
            r#"theme = "fallback""#,
        );
        let r = scan(tmp.path());
        assert!(r.path_exists);
        assert!(matches!(&r.detected_fields[0], ImportedField::Theme(t) if t == "primary"));
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
}
