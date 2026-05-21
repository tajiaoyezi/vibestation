//! MVP-18 · §F.3 fixture contract smoke tests.
//!
//! 验证 `crates/core/tests/fixtures/pane_link/*.txt` 实际文件形态可被
//! `sanitize_ai_prompt` + `parser_bridge::normalize_issues` raw-fallback
//! 路径稳定 consume，且 secret / OSC52 等敏感数据全部 strip。
//!
//! 单元测试（`sanitize.rs` / `parser_bridge.rs` 内部 `#[cfg(test)]`）继续覆盖
//! 精确字段行为；本文件给 §F.3 fixture **文件契约**加一层合约 smoke：
//! 消费方按 `<fixture-name>.txt` 路径读取后，sanitize 输出仍然安全。

use std::fs;
use std::path::PathBuf;

use vibestation_core::parser_bridge::{
    normalize_issues, ParserFallbackMode, UntrustedParserOutput,
};
use vibestation_core::sanitize::{sanitize_ai_prompt, SanitizeCtx, MAX_AI_PROMPT_BYTES};

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
#[test]
fn secret_fixture_redacts_all_token_shapes() {
    let raw = read_fixture("pane_failure_secret.txt");
    let result = sanitize_ai_prompt(&raw, &SanitizeCtx::default())
        .expect("secret fixture must sanitize successfully");

    assert!(
        result.redaction_count >= 6,
        "§E.2: secret fixture must trigger redactions for at least 6 distinct shapes \
         (bearer / api_key / aws_secret / github_token / url-creds × 2 / bearer dup); \
         got {}",
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

/// §E.1 + §C.6：parser_bridge raw-fallback path 对所有 §F.3 fixture 不崩 + 输出 sanitized。
#[test]
fn parser_bridge_raw_fallback_consumes_all_fixtures() {
    for name in FAILURE_FIXTURES {
        let raw = read_fixture(name);
        // ParserUnavailable 强制走 raw fallback path（与 ParserCrash / unsupported-cli 同形态）。
        let parser_out = UntrustedParserOutput::ParserUnavailable {
            cli_kind: Some("test".to_string()),
            message: "forced raw fallback for fixture contract smoke".to_string(),
            raw_text: raw.clone(),
        };

        let result = normalize_issues(&parser_out, 20);

        assert_eq!(
            result.fallback_mode,
            ParserFallbackMode::RawText,
            "{name}: ParserUnavailable must yield RawText fallback"
        );
        assert!(
            result.parsed_issues.is_empty(),
            "{name}: raw fallback must not emit parsed issues"
        );
        assert!(
            !result.raw_excerpt.contains("\u{1b}]52;c;"),
            "{name}: raw_excerpt must not carry OSC52 sequences after sanitization"
        );
        assert!(
            !result.raw_excerpt.as_bytes().contains(&0x1b),
            "{name}: raw_excerpt must not carry ESC bytes after sanitization"
        );
    }
}

/// §E.2：secret fixture 走 parser_bridge raw fallback 时仍要 redact secret。
#[test]
fn parser_bridge_raw_fallback_redacts_secret_fixture() {
    let raw = read_fixture("pane_failure_secret.txt");
    let parser_out = UntrustedParserOutput::ParserUnavailable {
        cli_kind: Some("test".to_string()),
        message: "raw fallback".to_string(),
        raw_text: raw,
    };

    let result = normalize_issues(&parser_out, 20);

    assert!(
        result.redaction_count >= 6,
        "§E.2: raw fallback path must propagate redactions (got {})",
        result.redaction_count
    );
    assert!(
        !result.raw_excerpt.contains("sk-FAKE"),
        "§E.2: raw_excerpt must not leak sk-FAKE token shapes"
    );
}
