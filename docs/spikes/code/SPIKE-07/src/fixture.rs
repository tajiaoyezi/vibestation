//! Fixture loader：读 `docs/spikes/raw/SPIKE-06/` 36 条 `*.redacted.cast`。
//!
//! - glob 过滤 `*.redacted.cast`（跳过 `claude-version-*.txt` 等非样本）
//! - 文件名 `{cli}_{scenario}_{take}.redacted.cast` → 三维索引
//!   - cli = 首 `_` 前（`claude` / `codex`）
//!   - take = 末 `_` 后（数字）
//!   - scenario = 中间（`happy_path` / `interrupt_residual` / `auth_fail` /
//!     `network_error` / `long_stream` / `mixed_ansi_json`）
//! - 同名 `.redaction.json` sidecar → 脱敏 metadata（redacted_fields 数）
//!
//! 注册表按 `(cli, scenario)` 可查（spec §E.1 要求）。

use crate::cast::{decode, DecodedCast};
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const SCENARIOS: [&str; 6] = [
    "happy_path",
    "interrupt_residual",
    "auth_fail",
    "network_error",
    "long_stream",
    "mixed_ansi_json",
];

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RedactionMeta {
    #[serde(default)]
    pub redacted_fields: Vec<serde_json::Value>,
    #[serde(default)]
    pub source_sha256: Option<String>,
    #[serde(default)]
    pub redacted_sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub cli: String,
    pub scenario: String,
    pub take: u32,
    pub path: PathBuf,
    pub decoded: DecodedCast,
    pub cast_parse_errors: usize,
    pub redaction: RedactionMeta,
}

impl Fixture {
    /// 是否结构==原始（redacted_sha256 == source_sha256 · 即未实际脱敏）。
    pub fn structurally_unchanged(&self) -> bool {
        match (
            &self.redaction.source_sha256,
            &self.redaction.redacted_sha256,
        ) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(String),
    BadFilename(String),
    Decode(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO: {e}"),
            LoadError::BadFilename(e) => write!(f, "文件名不合约定: {e}"),
            LoadError::Decode(e) => write!(f, "cast 解码: {e}"),
        }
    }
}

/// 从 `<cli>_<scenario>_<take>` stem 解析三维键。
pub fn parse_stem(stem: &str) -> Result<(String, String, u32), LoadError> {
    let (head, take_s) = stem
        .rsplit_once('_')
        .ok_or_else(|| LoadError::BadFilename(format!("无 take 后缀: {stem}")))?;
    let take: u32 = take_s
        .parse()
        .map_err(|_| LoadError::BadFilename(format!("take 非数字: {stem}")))?;
    let (cli, scenario) = head
        .split_once('_')
        .ok_or_else(|| LoadError::BadFilename(format!("无 cli/scenario 分隔: {stem}")))?;
    Ok((cli.to_string(), scenario.to_string(), take))
}

/// 加载 corpus 目录全部 `*.redacted.cast`。返回按文件名排序的 Fixture 列表。
pub fn load_corpus(dir: &Path) -> Result<Vec<Fixture>, LoadError> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| LoadError::Io(e.to_string()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.to_string_lossy().ends_with(".redacted.cast"))
        .collect();
    entries.sort();

    let mut out = Vec::with_capacity(entries.len());
    for path in entries {
        let stem = path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.trim_end_matches(".redacted.cast"))
            .ok_or_else(|| LoadError::BadFilename(format!("{path:?}")))?;
        let (cli, scenario, take) = parse_stem(stem)?;
        let text = std::fs::read_to_string(&path).map_err(|e| LoadError::Io(e.to_string()))?;
        let (decoded, cast_errs) =
            decode(&text).map_err(|e| LoadError::Decode(format!("{}: {e}", path.display())))?;
        let rj = path.with_file_name(format!("{stem}.redaction.json"));
        let redaction = std::fs::read_to_string(&rj)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        out.push(Fixture {
            cli,
            scenario,
            take,
            path,
            decoded,
            cast_parse_errors: cast_errs,
            redaction,
        });
    }
    Ok(out)
}

/// 按 (cli, scenario) 过滤（spec §E.1：`cli=claude, scenario=auth_fail` 可查）。
pub fn query<'a>(corpus: &'a [Fixture], cli: &str, scenario: &str) -> Vec<&'a Fixture> {
    corpus
        .iter()
        .filter(|f| f.cli == cli && f.scenario == scenario)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stem_claude_happy() {
        assert_eq!(
            parse_stem("claude_happy_path_1").unwrap(),
            ("claude".into(), "happy_path".into(), 1)
        );
    }

    #[test]
    fn parse_stem_codex_mixed_ansi_json() {
        assert_eq!(
            parse_stem("codex_mixed_ansi_json_3").unwrap(),
            ("codex".into(), "mixed_ansi_json".into(), 3)
        );
    }

    #[test]
    fn parse_stem_rejects_no_take() {
        assert!(parse_stem("claude").is_err());
    }

    /// 真实 corpus 集成 test：必须正好 36 条 · 2 CLI × 6 场景 × 3 take。
    /// 路径相对 crate（docs/spikes/code/SPIKE-07/）→ ../../raw/SPIKE-06。
    #[test]
    fn loads_real_corpus_36_complete_matrix() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../raw/SPIKE-06")
            .canonicalize()
            .expect("corpus dir 存在");
        let corpus = load_corpus(&dir).expect("load ok");
        assert_eq!(corpus.len(), 36, "应正好 36 条 .redacted.cast");
        for cli in ["claude", "codex"] {
            for scen in SCENARIOS {
                let m = query(&corpus, cli, scen);
                assert_eq!(m.len(), 3, "{cli}/{scen} 应 3 take · 实 {}", m.len());
            }
        }
        // 每条都成功解码出 header（version 3）
        assert!(corpus.iter().all(|f| f.decoded.header.version == 3));
    }
}
