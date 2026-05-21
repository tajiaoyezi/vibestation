//! MVP-18 · §F.3 fixture contract smoke tests.
//!
//! 验证 `crates/core/tests/fixtures/pane_link/*.txt` 实际文件形态可被
//! `sanitize_ai_prompt` + `parser_bridge::normalize_issues` raw-fallback
//! 路径稳定 consume，且 secret / OSC52 等敏感数据全部 strip。
//!
//! 单元测试（`sanitize.rs` / `parser_bridge.rs` 内部 `#[cfg(test)]`）继续覆盖
//! 精确字段行为；本文件给 §F.3 fixture **文件契约**加一层合约 smoke：
//! 消费方按 `<fixture-name>.txt` 路径读取后，sanitize 输出仍然安全。
//!
//! 互补关系（L1 cross-link · PR #410 review）：
//! - §F.1 typed Rust fixture（`fixture_failure(...)` / `fixture_ai_pane(...)` 等）：
//!   见 `pane_link_integration.rs` · 测 DAO / validate_link_request 行为。
//! - §F.3 raw text fixture（`pane_failure_*.txt`）：本文件 · 测 sanitize /
//!   parser_bridge raw-fallback pipeline 对真实失败输出文本的鲁棒性。

use std::fs;
use std::path::PathBuf;

use vibestation_core::parser_bridge::{
    normalize_issues, ParserFallbackMode, UntrustedParserOutput,
};
use vibestation_core::sanitize::{sanitize_ai_prompt, SanitizeCtx, MAX_AI_PROMPT_BYTES};

/// §F.3 fixture 目录的绝对路径。
///
/// 用 `CARGO_MANIFEST_DIR`（crate root · 编译期常量）而非 CWD-relative path：
/// cargo test 默认 CWD 是 crate root，但自动化脚本 / IDE / 外部 worktree
/// 可能从其他目录调起 `cargo test` · `env!` 保证 fixture 解析始终从 crate root 出发。
fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("pane_link")
}

fn read_fixture(name: &str) -> String {
    let path = fixture_dir().join(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture {name} must exist at {path:?}: {e}"))
}

/// §F.3 fixture catalog（§I.1 contract）· 实际消费方 path 硬编码。
const FAILURE_FIXTURES: &[&str] = &[
    "pane_failure_rustc.txt",
    "pane_failure_vitest.txt",
    "pane_failure_pytest.txt",
    "pane_failure_ansi_json.txt",
    "pane_failure_secret.txt",
    "pane_failure_osc52.txt",
];

/// §E.1 / §E.2 / §F.3：所有 fixture 文件存在且非空。
#[test]
fn all_failure_fixtures_present_and_non_empty() {
    for name in FAILURE_FIXTURES {
        let content = read_fixture(name);
        assert!(!content.is_empty(), "§F.3 fixture {name} must be non-empty");
        assert!(
            !content.as_bytes().contains(&0),
            "§F.3 fixture {name} must not contain NUL bytes (sanitize would reject)"
        );
    }
}

/// §E.1：sanitize_ai_prompt 对所有 §F.3 fixture 必须返回 Ok（不崩 / 不被 NUL 阻断）。
#[test]
fn sanitize_consumes_all_fixtures_without_error() {
    let ctx = SanitizeCtx::default();
    for name in FAILURE_FIXTURES {
        let raw = read_fixture(name);
        let result = sanitize_ai_prompt(&raw, &ctx)
            .unwrap_or_else(|e| panic!("sanitize must succeed on {name}: {e}"));

        assert!(
            !result.text.is_empty(),
            "{name}: sanitized text must remain non-empty (fixture has meaningful content)"
        );
        assert!(
            result.text.len() <= MAX_AI_PROMPT_BYTES,
            "{name}: sanitized text must respect MAX_AI_PROMPT_BYTES"
        );
    }
}

/// §E.1：OSC52 fixture sanitized 后不含 `]52;c;` 控制序列残留。
#[test]
fn osc52_fixture_strips_control_sequence() {
    let raw = read_fixture("pane_failure_osc52.txt");

    assert!(
        raw.contains("\u{1b}]52;c;"),
        "fixture precondition: raw OSC52 bytes must be present in source"
    );

    let sanitized = sanitize_ai_prompt(&raw, &SanitizeCtx::default())
        .expect("OSC52 fixture must sanitize successfully");

    assert!(
        !sanitized.text.contains("]52;c;"),
        "§E.1: OSC52 control sequence must be stripped from sanitized output"
    );
    assert!(
        !sanitized.text.as_bytes().contains(&0x1b),
        "§E.1: no ESC bytes may remain after sanitization"
    );
}

/// §E.2：secret fixture sanitized 后无可识别 secret 残留，且 redaction_count 计入。
///
/// `pane_failure_secret.txt` 锁死 7 处独立 redaction（§F.3 README 契约 · 文件不可变）：
/// 1. `Authorization: Bearer sk-FAKE0000...` → standalone `sk-` 形态
/// 2. `OPENAI_API_KEY=sk-FAKEabcdef...` → key 含 `api_key`
/// 3. `AWS_SECRET_ACCESS_KEY=FAKEwJal...` → key 含 `secret`
/// 4. `GITHUB_TOKEN=ghp_FAKE...` → key 含 `token` + standalone `ghp_` 形态（同 token 内重复命中按 1 计 · sanitize 内部 dedup）
/// 5. `https://ci-bot:FAKEPASSWORD123@github.example.test/...` → URL credentials
/// 6. `postgres://app:FAKEDBPASS@db.example.test:...` → URL credentials
/// 7. `(redact this line's bearer token too: sk-FAKEzzz...)` → 末尾 standalone `sk-`
///
/// 严格契约 `== 7`：若 sanitize 规则变动让任一种 token shape 漏检 · 测试立即 fail
/// 暴露 silent regression。若 fixture 内容真要演进 · 同步改本测试 + §F.3 README + spec。
///
/// `banned` 数组与 fixture 文件内容紧耦合 by design（fixture 是 §F.3 spec 契约 ·
/// 不应随便改）· 任何 fixture 字面值改动必须同步本测试。
#[test]
fn secret_fixture_redacts_all_token_shapes() {
    let raw = read_fixture("pane_failure_secret.txt");
    let result = sanitize_ai_prompt(&raw, &SanitizeCtx::default())
        .expect("secret fixture must sanitize successfully");

    assert_eq!(
        result.redaction_count, 7,
        "§E.2: secret fixture must trigger exactly 7 redactions \
         (bearer / api_key / aws_secret / github_token / url-creds × 2 / trailing bearer); \
         got {} · 阈值不匹配 = sanitize 规则变动或 fixture 内容漂移 · 修测试前先排查根因",
        result.redaction_count
    );

    let banned = [
        "sk-FAKE0000000000000000000000000000000000000000",
        "sk-FAKEabcdefghijklmnopqrstuvwxyz0123456789ABCD",
        "sk-FAKEzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        "FAKEwJalrXUtnFEMIK7MDENGbPxRfiCYEXAMPLEKEY",
        "ghp_FAKE0000000000000000000000000000000000",
        "FAKEPASSWORD123",
        "FAKEDBPASS",
    ];
    for needle in banned {
        assert!(
            !result.text.contains(needle),
            "§E.2: secret token {needle} must be redacted from sanitized output"
        );
    }
    assert!(
        result.text.contains("<REDACTED>"),
        "§E.2: sanitized output must mark redactions with <REDACTED>"
    );
}

/// `UntrustedParserOutput` 中走 `raw_fallback` 的 4 个 variant 构造器列表。
/// 用于参数化覆盖全部 fallback 入口（M2 · PR #410 review）· 避免只测一个 path
/// 给 silent regression 留缝（例如某 variant 未来错走非 raw_fallback 分支）。
///
/// `Structured` 不在本列表 · 它走结构化解析路径 · 由 unit test 单独覆盖。
fn raw_fallback_variants(raw: String) -> Vec<(&'static str, UntrustedParserOutput)> {
    vec![
        (
            "ParserUnavailable",
            UntrustedParserOutput::ParserUnavailable {
                cli_kind: Some("test".to_string()),
                message: "parser unavailable".to_string(),
                raw_text: raw.clone(),
            },
        ),
        (
            "ParserTimeout",
            UntrustedParserOutput::ParserTimeout {
                cli_kind: Some("test".to_string()),
                message: "2s deadline exceeded".to_string(),
                raw_text: raw.clone(),
            },
        ),
        (
            "ParserCrash",
            UntrustedParserOutput::ParserCrash {
                cli_kind: Some("test".to_string()),
                message: "parser panicked".to_string(),
                raw_text: raw.clone(),
            },
        ),
        (
            "UnsupportedCliKind",
            UntrustedParserOutput::UnsupportedCliKind {
                cli_kind: "unknown-cli".to_string(),
                raw_text: raw,
            },
        ),
    ]
}

/// §E.1 + §C.6：parser_bridge raw-fallback path 对所有 §F.3 fixture × 4 variant 不崩 + 输出 sanitized。
///
/// M2（PR #410 review）：测试覆盖全部 4 个走 `raw_fallback` 的 `UntrustedParserOutput` variant
/// （ParserUnavailable / ParserTimeout / ParserCrash / UnsupportedCliKind）·
/// 共 6 fixture × 4 variant = 24 个矩阵点 · 全过才算契约覆盖。
#[test]
fn parser_bridge_raw_fallback_consumes_all_fixtures_all_variants() {
    for name in FAILURE_FIXTURES {
        let raw = read_fixture(name);
        for (variant_name, parser_out) in raw_fallback_variants(raw.clone()) {
            let result = normalize_issues(&parser_out, 20);
            let ctx = format!("{name} × {variant_name}");

            assert_eq!(
                result.fallback_mode,
                ParserFallbackMode::RawText,
                "{ctx}: variant must yield RawText fallback"
            );
            assert!(
                result.parsed_issues.is_empty(),
                "{ctx}: raw fallback must not emit parsed issues"
            );
            assert!(
                !result.raw_excerpt.contains("\u{1b}]52;c;"),
                "{ctx}: raw_excerpt must not carry OSC52 sequences after sanitization"
            );
            assert!(
                !result.raw_excerpt.as_bytes().contains(&0x1b),
                "{ctx}: raw_excerpt must not carry ESC bytes after sanitization"
            );
        }
    }
}

/// §E.2：secret fixture 走 parser_bridge raw fallback 时仍要 redact secret · 4 variant 全覆盖。
///
/// 严格契约 `== 7`（与 `secret_fixture_redacts_all_token_shapes` 对齐 · 同源 fixture · 同 7 处 redaction）。
#[test]
fn parser_bridge_raw_fallback_redacts_secret_fixture_all_variants() {
    let raw = read_fixture("pane_failure_secret.txt");
    for (variant_name, parser_out) in raw_fallback_variants(raw.clone()) {
        let result = normalize_issues(&parser_out, 20);

        assert_eq!(
            result.redaction_count, 7,
            "§E.2 ({variant_name}): raw fallback must propagate exactly 7 redactions (got {}) · \
             阈值不匹配 = sanitize 规则变动或 fixture 内容漂移",
            result.redaction_count
        );
        assert!(
            !result.raw_excerpt.contains("sk-FAKE"),
            "§E.2 ({variant_name}): raw_excerpt must not leak sk-FAKE token shapes"
        );
    }
}
