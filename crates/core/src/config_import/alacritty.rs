//! Alacritty 配置 parser · TOML 0.14+ 优先 · YAML 0.13- fallback
//!
//! 路径优先级（spec §Acceptance C）：
//!   1. ~/.config/alacritty/alacritty.toml（0.14+）
//!   2. ~/.config/alacritty/alacritty.yml（0.13- 已 deprecated 但仍有用户）
//!
//! 字段：font.normal.family / font.size / colors / key_bindings
//! key_bindings action 映射延后到 Phase B（如 Alacritty SpawnNewInstance → 无映射 warn）

use super::{ConfigImportError, ImportScanResult, ImportSource, ImportedField};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AlacrittyConfig {
    font: Option<AlacrittyFont>,
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

pub fn scan(home: &Path) -> ImportScanResult {
    let toml_path = home.join(".config/alacritty/alacritty.toml");
    let yaml_path = home.join(".config/alacritty/alacritty.yml");
    let (path, is_yaml) = if toml_path.exists() {
        (Some(toml_path), false)
    } else if yaml_path.exists() {
        (Some(yaml_path), true)
    } else {
        (None, false)
    };
    match &path {
        Some(p) => {
            let result = if is_yaml {
                parse_yaml(p)
            } else {
                parse_toml(p)
            };
            match result {
                Ok(fields) => ImportScanResult {
                    source: ImportSource::Alacritty,
                    path: Some(p.clone()),
                    path_exists: true,
                    detected_fields: fields,
                    errors: Vec::new(),
                },
                Err(e) => ImportScanResult {
                    source: ImportSource::Alacritty,
                    path: Some(p.clone()),
                    path_exists: true,
                    detected_fields: Vec::new(),
                    errors: vec![e.to_string()],
                },
            }
        }
        None => ImportScanResult {
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
}
