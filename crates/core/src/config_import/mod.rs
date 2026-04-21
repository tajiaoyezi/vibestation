//! 配置导入 parser 模块 · MVP-06 Phase A
//!
//! Scope：只做 parse 纯逻辑 · 不含 IPC / UI / ts-rs derive
//! Phase B（后续 · MVP-04 Phase B-F done 后）再做 IPC command 暴露 + UI 接入
//!
//! spec 依据：docs/tasks/MVP-06-config-import.md §Scope + §H.1 解析库选型

pub mod alacritty;
pub mod ghostty;
pub mod iterm2;

use std::path::{Path, PathBuf};

/// 导入源枚举（Phase A 内部用 · Phase B 会 ts-rs derive 给 IPC）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportSource {
    Ghostty,
    ITerm2,
    Alacritty,
}

/// 解析后的单个字段（Phase A 内部用）
#[derive(Debug, Clone, PartialEq)]
pub enum ImportedField {
    FontFamily(String),
    FontSize(f32),
    Theme(String),
    Shell(String),
    KeyBinding { key: String, action: String },
    AnsiColor { index: u8, hex: String },
}

/// 单个源扫描 + 解析结果
#[derive(Debug, Clone)]
pub struct ImportScanResult {
    pub source: ImportSource,
    pub path: Option<PathBuf>, // 实际找到的路径（None 若未检测到）
    pub path_exists: bool,
    pub detected_fields: Vec<ImportedField>,
    pub errors: Vec<String>, // graceful · 字段级错误
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigImportError {
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml parse error: {0}")]
    Toml(String),
    #[error("plist parse error: {0}")]
    Plist(String),
    #[error("yaml parse error: {0}")]
    Yaml(String),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// 扫描默认路径 · 返回所有检测到的源
///
/// Phase A：只返回结构化结果 · 不做 UI / 不写 DB
pub fn scan_all_sources(home: &Path) -> Vec<ImportScanResult> {
    let results = vec![
        ghostty::scan(home),
        iterm2::scan(home),
        alacritty::scan(home),
    ];
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_returns_3_sources() {
        let tmp = tempfile::tempdir().unwrap();
        let results = scan_all_sources(tmp.path());
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].source, ImportSource::Ghostty);
        assert_eq!(results[1].source, ImportSource::ITerm2);
        assert_eq!(results[2].source, ImportSource::Alacritty);
    }

    #[test]
    fn scan_empty_home_all_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let results = scan_all_sources(tmp.path());
        for r in &results {
            assert!(!r.path_exists);
            assert!(r.detected_fields.is_empty());
        }
    }
}
