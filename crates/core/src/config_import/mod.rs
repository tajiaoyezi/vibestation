//! 配置导入 parser + IPC 模块 · MVP-06
//!
//! - Phase A（PR #80 / #81）：parser 纯逻辑 · 三家解析（Ghostty TOML / iTerm2 plist /
//!   Alacritty TOML+YAML）+ ImportedField + RawScanResult（内部使用 · 含 PathBuf）
//! - Phase B（本 PR）：ts-rs IPC 类型（[`ipc`] 子模块）+ canonical form 算法
//!   ([`keybinding`] 子模块）+ 4 个 Tauri command + 写入 `app_settings`
//!
//! spec 依据：docs/tasks/MVP-06-config-import.md §G IPC + §H 决策锁定

pub mod alacritty;
pub mod ghostty;
pub mod ipc;
pub mod iterm2;
pub mod keybinding;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// 导入源枚举（spec §G.2）· 既给 Phase A parser 内部用 · 也是 IPC 类型
///
/// JSON serialization：`"ghostty" | "iTerm2" | "alacritty"`（serde camelCase enum）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ImportSource {
    Ghostty,
    ITerm2,
    Alacritty,
}

/// 解析后的单个字段（Phase A 内部用 · 不直接 IPC · IPC 用 [`ipc::ImportFieldType`]）
#[derive(Debug, Clone, PartialEq)]
pub enum ImportedField {
    FontFamily(String),
    FontSize(f32),
    /// Issue #206 Important 1 修复：String → ThemeMode enum · parser 层 reject
    /// 非 light/dark/auto 的输入 · 类型层保证只有合法值流到下游
    Theme(ipc::ThemeMode),
    Shell(String),
    KeyBinding {
        key: String,
        action: String,
    },
    AnsiColor {
        index: u8,
        hex: String,
    },
}

/// 单个源扫描 + 解析的 raw 结果（Phase A 内部 · 含 PathBuf · 不进 IPC）
///
/// IPC 表示：[`ipc::ImportScanResult`] · 通过 [`ipc::ImportScanResult::from`] 转换
#[derive(Debug, Clone)]
pub struct RawScanResult {
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

/// 扫描默认路径 · 返回所有检测到的源（Phase A 内部 raw 结果）
///
/// IPC 入口：[`ipc::scan_all_sources_ipc`] · 在此函数基础上做 IPC 转换
pub fn scan_all_sources(home: &Path) -> Vec<RawScanResult> {
    vec![
        ghostty::scan(home),
        iterm2::scan(home),
        alacritty::scan(home),
    ]
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
