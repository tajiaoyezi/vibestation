//! MVP-18 §F「parser bridge unit」· §C.3/C.5/C.6 + §E.1/E.2 fixture-driven integration tests.
//!
//! 集成测试层（`crates/core/tests/` binary · 非 src）。
//! 只 import A1/A2 冻结公共 API：
//! - `vibestation_core::parser_bridge::*`（normalize_issues / NormalizeResult / ParsedIssue / etc.）
//! - `vibestation_core::sanitize::{sanitize_ai_prompt, SanitizeCtx}`
//!
//! 不改任何 src · 不碰 tests/fixtures/pane_link/*（#348 只读）。
//!
//! 覆盖范围：
//! - §F「parser bridge unit」行：rustc/vitest/pytest fixture → normalize_issues
//! - §C.3 parsed_issues 归一化（file/line/column/message）
//! - §C.5 >20 issue → ≤20 + truncated_count > 0
//! - §C.6 unsupported/crash/timeout/unavailable → raw fallback 不 panic · error 来源保留
//! - §E.1 ansi_json/osc52 fixture → sanitize_ai_prompt → ANSI/OSC52 strip 干净
//! - §E.2 secret fixture → sanitize_ai_prompt → redaction_count > 0
//! - §I.2 fixture 缺失 skip with reason（不伪造样本）

use std::path::PathBuf;

use vibestation_core::parser_bridge::{
    normalize_issues, NormalizeResult, ParsedIssueSeverity, ParserBridgeError, ParserFallbackMode,
    UntrustedParserEvent, UntrustedParserOutput, DEFAULT_ISSUE_LIMIT,
};
use vibestation_core::sanitize::{sanitize_ai_prompt, SanitizeCtx};

// ── Fixture helpers ──────────────────────────────────────────────────────────

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("pane_link")
}

/// §I.2：fixture 缺失时返回 None（不伪造样本）。
fn read_fixture(name: &str) -> Option<String> {
    std::fs::read_to_string(fixture_dir().join(name)).ok()
}

/// 构造 `UntrustedParserEvent::Issue` helper。
fn make_issue(
    severity: ParsedIssueSeverity,
    file: &str,
    line: u32,
    column: u32,
    message: &str,
) -> UntrustedParserEvent {
    UntrustedParserEvent::Issue {
        severity,
        file: Some(file.to_string()),
        line: Some(line),
        column: Some(column),
        message: message.to_string(),
    }
}

// ── §C.3 · rustc fixture → normalize_issues → ParsedIssue ───────────────────

/// §C.3 · §F parser bridge unit：rustc 6 error fixture → normalize → 6 ParsedIssue。
/// 验证 file/line/column/message 字段完整（§F.3 required fields：file+line+column+code+message）。
#[test]
fn rustc_fixture_normalizes_to_six_issues() {
    let raw_text = match read_fixture("pane_failure_rustc.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_rustc.txt not found (§I.2)");
            return;
        }
    };

    // 6 rustc error events corresponding to the fixture（E0425/E0308/E0599/E0061/E0277/E0382）
    let events = vec![
        make_issue(
            ParsedIssueSeverity::Error,
            "/workspace/vibestation/crates/core/src/runner.rs",
            42,
            13,
            "cannot find function `spawn_watch_process` in this scope",
        ),
        make_issue(
            ParsedIssueSeverity::Error,
            "/workspace/vibestation/crates/core/src/runner.rs",
            67,
            29,
            "mismatched types",
        ),
        make_issue(
            ParsedIssueSeverity::Error,
            "/workspace/vibestation/crates/core/src/runner.rs",
            103,
            9,
            "no method named `send_output` found for struct `PaneOutput` in the current scope",
        ),
        make_issue(
            ParsedIssueSeverity::Error,
            "/workspace/vibestation/crates/core/src/runner.rs",
            134,
            21,
            "this function takes 2 arguments but 1 argument was supplied",
        ),
        make_issue(
            ParsedIssueSeverity::Error,
            "/workspace/vibestation/crates/core/src/runner.rs",
            156,
            28,
            "the trait bound `Vec<ParsedIssue>: serde::Serialize` is not satisfied",
        ),
        make_issue(
            ParsedIssueSeverity::Error,
            "/workspace/vibestation/crates/core/src/runner.rs",
            178,
            14,
            "borrow of moved value: `child_pane_id`",
        ),
    ];

    let output = UntrustedParserOutput::Structured {
        cli_kind: "rustc".to_string(),
        events,
        raw_text,
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(
        result.fallback_mode,
        ParserFallbackMode::Structured,
        "rustc structured path must not fall back"
    );
    assert_eq!(
        result.error, None,
        "rustc fixture must not produce an error"
    );
    assert_eq!(
        result.parsed_issues.len(),
        6,
        "rustc fixture must produce exactly 6 ParsedIssue"
    );
    assert_eq!(result.truncated_count, 0, "6 issues must not be truncated");

    // §F.3 required fields: file / line / column / message
    for issue in &result.parsed_issues {
        assert!(
            issue.file.is_some(),
            "each rustc issue must have a file field"
        );
        assert!(
            issue.line.is_some(),
            "each rustc issue must have a line field"
        );
        assert!(
            issue.column.is_some(),
            "each rustc issue must have a column field"
        );
        assert!(
            !issue.message.is_empty(),
            "each rustc issue must have a message"
        );
        assert_eq!(
            issue.severity,
            ParsedIssueSeverity::Error,
            "all rustc fixture issues must be Error severity"
        );
    }

    // first issue: file + line spot-check
    assert_eq!(result.parsed_issues[0].line, Some(42));
    assert_eq!(result.parsed_issues[0].column, Some(13));
}

// ── §C.3 · vitest fixture → normalize_issues ────────────────────────────────

/// §C.3 · §F parser bridge unit：vitest 2 failure fixture → normalize → ≥2 ParsedIssue。
#[test]
fn vitest_fixture_normalizes_to_two_issues() {
    let raw_text = match read_fixture("pane_failure_vitest.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_vitest.txt not found (§I.2)");
            return;
        }
    };

    let events = vec![
        make_issue(
            ParsedIssueSeverity::Error,
            "tests/panels/Terminal/pane-linking/store.test.ts",
            88,
            42,
            "AssertionError: expected 'workspace-a' to be 'workspace-b'",
        ),
        make_issue(
            ParsedIssueSeverity::Error,
            "tests/panels/Terminal/pane-linking/store.test.ts",
            135,
            54,
            "AssertionError: expected length 2 to be 1",
        ),
    ];

    let output = UntrustedParserOutput::Structured {
        cli_kind: "vitest".to_string(),
        events,
        raw_text,
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(result.fallback_mode, ParserFallbackMode::Structured);
    assert_eq!(result.error, None);
    assert_eq!(
        result.parsed_issues.len(),
        2,
        "vitest fixture must produce 2 ParsedIssue"
    );

    // file / line spot-check
    assert!(result.parsed_issues[0]
        .file
        .as_deref()
        .unwrap_or("")
        .contains("store.test.ts"));
    assert_eq!(result.parsed_issues[0].line, Some(88));
}

// ── §C.3 · pytest fixture → normalize_issues ────────────────────────────────

/// §C.3 · §F parser bridge unit：pytest 1 failure fixture → normalize → 1 ParsedIssue。
#[test]
fn pytest_fixture_normalizes_to_one_issue() {
    let raw_text = match read_fixture("pane_failure_pytest.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_pytest.txt not found (§I.2)");
            return;
        }
    };

    let events = vec![make_issue(
        ParsedIssueSeverity::Error,
        "tests/test_linkgraph.py",
        57,
        1,
        "linkgraph.CrossWorkspaceError: child pane belongs to workspace 'ws-b', expected 'ws-a'",
    )];

    let output = UntrustedParserOutput::Structured {
        cli_kind: "pytest".to_string(),
        events,
        raw_text,
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(result.fallback_mode, ParserFallbackMode::Structured);
    assert_eq!(result.error, None);
    assert_eq!(
        result.parsed_issues.len(),
        1,
        "pytest fixture must produce exactly 1 ParsedIssue"
    );
    assert!(result.parsed_issues[0]
        .message
        .contains("CrossWorkspaceError"));
    assert_eq!(result.parsed_issues[0].line, Some(57));
}

// ── §C.5 · >20 issue → ≤20 + truncated_count ────────────────────────────────

/// §C.5：25 distinct issues → ≤20 result + truncated_count == 5。
#[test]
fn over_twenty_issues_are_capped_and_counted() {
    let events: Vec<UntrustedParserEvent> = (1u32..=25)
        .map(|line| {
            make_issue(
                ParsedIssueSeverity::Error,
                "/workspace/project/src/lib.rs",
                line,
                1,
                &format!("error at line {line}"),
            )
        })
        .collect();

    let output = UntrustedParserOutput::Structured {
        cli_kind: "rustc".to_string(),
        events,
        raw_text: "raw text for 25 issues".to_string(),
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(
        result.parsed_issues.len(),
        DEFAULT_ISSUE_LIMIT,
        "§C.5: result must be capped at DEFAULT_ISSUE_LIMIT (20)"
    );
    assert_eq!(
        result.truncated_count, 5,
        "§C.5: truncated_count must be 25 - 20 = 5"
    );
    assert_eq!(result.fallback_mode, ParserFallbackMode::Structured);
}

/// §C.5 with rustc fixture raw_text: 25 issues, fallback_mode still Structured.
#[test]
fn truncation_preserves_structured_fallback_mode() {
    let events: Vec<UntrustedParserEvent> = (1u32..=25)
        .map(|line| make_issue(ParsedIssueSeverity::Error, "src/lib.rs", line, 1, "e"))
        .collect();
    let output = UntrustedParserOutput::Structured {
        cli_kind: "cargo".to_string(),
        events,
        raw_text: String::new(),
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);
    assert_eq!(result.fallback_mode, ParserFallbackMode::Structured);
    assert!(result.truncated_count > 0);
}

// ── §C.6 · unsupported/crash/timeout/unavailable → raw fallback ─────────────

/// §C.6：unsupported CLI kind → RawText fallback + UnsupportedCliKind error。
#[test]
fn unsupported_cli_kind_produces_raw_fallback_no_panic() {
    let raw_text =
        read_fixture("pane_failure_rustc.txt").unwrap_or_else(|| "raw fallback text".to_string());

    let output = UntrustedParserOutput::UnsupportedCliKind {
        cli_kind: "unknown-custom-linter".to_string(),
        raw_text,
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(
        result.fallback_mode,
        ParserFallbackMode::RawText,
        "§C.6: unsupported cli kind must produce RawText fallback"
    );
    assert_eq!(
        result.parsed_issues,
        Vec::new(),
        "§C.6: no parsed issues on fallback"
    );
    assert!(result.error.is_some(), "§C.6: error field must be Some");
    assert!(
        matches!(
            result.error,
            Some(ParserBridgeError::UnsupportedCliKind { .. })
        ),
        "§C.6: error kind must be UnsupportedCliKind"
    );
}

/// §C.6：parser timeout → RawText fallback + ParserTimeout error · error source preserved。
#[test]
fn parser_timeout_produces_raw_fallback_with_source() {
    let raw_text =
        read_fixture("pane_failure_vitest.txt").unwrap_or_else(|| "vitest raw".to_string());

    let output = UntrustedParserOutput::ParserTimeout {
        cli_kind: Some("vitest".to_string()),
        message: "2s deadline exceeded".to_string(),
        raw_text,
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
    assert_eq!(result.parsed_issues, Vec::new());
    assert_eq!(
        result.error,
        Some(ParserBridgeError::ParserTimeout {
            message: "2s deadline exceeded".to_string()
        }),
        "§C.6: error source must be preserved exactly"
    );
}

/// §C.6：parser crash → RawText fallback + ParserUnavailable error · no panic。
#[test]
fn parser_crash_produces_raw_fallback_no_panic() {
    let raw_text =
        read_fixture("pane_failure_pytest.txt").unwrap_or_else(|| "pytest raw".to_string());

    let output = UntrustedParserOutput::ParserCrash {
        cli_kind: Some("pytest".to_string()),
        message: "thread panicked in parser worker".to_string(),
        raw_text,
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
    assert_eq!(result.parsed_issues, Vec::new());
    assert!(
        matches!(
            result.error,
            Some(ParserBridgeError::ParserUnavailable { .. })
        ),
        "§C.6: ParserCrash must map to ParserUnavailable error"
    );
}

/// §C.6：parser unavailable → RawText fallback · structured_output_also_unsupported。
#[test]
fn structured_unsupported_cli_kind_in_structured_variant_falls_back() {
    // Structured with unsupported cli_kind triggers UnsupportedCliKind path in normalize_issues
    let output = UntrustedParserOutput::Structured {
        cli_kind: "proprietary-linter-xyz".to_string(),
        events: vec![make_issue(
            ParsedIssueSeverity::Error,
            "src/main.rs",
            1,
            1,
            "some issue",
        )],
        raw_text: "raw".to_string(),
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
    assert!(matches!(
        result.error,
        Some(ParserBridgeError::UnsupportedCliKind { .. })
    ));
}

// ── §E.1 · ansi_json fixture → sanitize → ANSI/OSC52 stripped ───────────────

/// §E.1：pane_failure_ansi_json.txt（真 ANSI ESC 字节）→ sanitize_ai_prompt → 无 ESC 字节。
#[test]
fn ansi_json_fixture_strips_ansi_bytes() {
    let raw = match read_fixture("pane_failure_ansi_json.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_ansi_json.txt not found (§I.2)");
            return;
        }
    };

    // Fixture must contain ESC bytes (0x1b) to be a meaningful test
    assert!(
        raw.as_bytes().contains(&0x1b),
        "fixture must contain ESC bytes to test stripping"
    );

    let ctx = SanitizeCtx::default();
    let result = sanitize_ai_prompt(&raw, &ctx).expect("sanitize must not error on ansi_json");

    assert!(
        !result.text.as_bytes().contains(&0x1b),
        "§E.1: sanitized output must not contain ESC (0x1b) bytes"
    );
    // Meaningful content still present after stripping
    assert!(
        result.text.contains("error") || result.text.contains("compiler-message"),
        "§E.1: meaningful content must survive ANSI stripping"
    );
}

/// §E.1：pane_failure_osc52.txt（真 OSC52 ESC + BEL 字节）→ sanitize → OSC52 strip 干净。
#[test]
fn osc52_fixture_strips_osc52_sequences() {
    let raw = match read_fixture("pane_failure_osc52.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_osc52.txt not found (§I.2)");
            return;
        }
    };

    // Fixture must contain ESC bytes to be meaningful
    assert!(
        raw.as_bytes().contains(&0x1b),
        "fixture must contain ESC bytes for OSC52 sequences"
    );

    let ctx = SanitizeCtx::default();
    let result = sanitize_ai_prompt(&raw, &ctx).expect("sanitize must not error on osc52");

    assert!(
        !result.text.as_bytes().contains(&0x1b),
        "§E.1: sanitized output must not contain ESC bytes after OSC52 stripping"
    );
    // OSC52 base64 payload must not leak into the output
    // The fixture OSC52 payloads decode to workspace paths and a fake secret
    assert!(
        !result.text.contains("L3dvcmtzcGFjZS9wcm9qZWN0"), // base64 payload
        "§E.1: OSC52 base64 payload must not appear in sanitized output"
    );
    // Normal text lines survive
    assert!(
        result.text.contains("error") || result.text.contains("linker"),
        "§E.1: normal error text must survive OSC52 stripping"
    );
}

/// §E.1：normalize_issues raw_excerpt path also strips ANSI from raw_text。
#[test]
fn normalize_issues_raw_excerpt_strips_ansi() {
    let raw = match read_fixture("pane_failure_ansi_json.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_ansi_json.txt not found (§I.2)");
            return;
        }
    };

    // Use ParserTimeout to go through raw_fallback path which sanitizes raw_text
    let output = UntrustedParserOutput::ParserTimeout {
        cli_kind: Some("cargo".to_string()),
        message: "timeout".to_string(),
        raw_text: raw,
    };
    let result: NormalizeResult = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert!(
        !result.raw_excerpt.as_bytes().contains(&0x1b),
        "§E.1: raw_excerpt must not contain ESC bytes"
    );
}

// ── §E.2 · secret fixture → sanitize → redaction_count > 0 ─────────────────

/// §E.2：pane_failure_secret.txt → sanitize_ai_prompt → redaction_count > 0 + <REDACTED> present。
#[test]
fn secret_fixture_redacts_secrets() {
    let raw = match read_fixture("pane_failure_secret.txt") {
        Some(s) => s,
        None => {
            eprintln!("SKIP: pane_failure_secret.txt not found (§I.2)");
            return;
        }
    };

    let ctx = SanitizeCtx::default();
    let result = sanitize_ai_prompt(&raw, &ctx).expect("sanitize must not error on secret fixture");

    assert!(
        result.redaction_count > 0,
        "§E.2: secret fixture must produce at least one redaction, got 0"
    );
    assert!(
        result.text.contains("<REDACTED>"),
        "§E.2: sanitized text must contain <REDACTED> marker"
    );
    // Original secret patterns must not appear verbatim
    // (fixture uses OPENAI_API_KEY= / AWS_SECRET_ACCESS_KEY= sensitive key patterns)
    // After redaction, those keys' values should be gone
    for sensitive_key in &["OPENAI_API_KEY", "AWS_SECRET_ACCESS_KEY"] {
        if raw.contains(sensitive_key) {
            // If the key assignment appears, the VALUE must have been redacted
            // (key itself is preserved, value replaced with <REDACTED>)
            let key_pos = result.text.find(sensitive_key);
            if let Some(pos) = key_pos {
                let after_key = &result.text[pos..];
                assert!(
                    after_key.contains("<REDACTED>"),
                    "§E.2: {sensitive_key} value must be redacted in output"
                );
            }
        }
    }
}

/// §E.2：normalize_issues structured path also redacts secrets in issue messages。
#[test]
fn normalize_issues_redacts_secrets_in_issue_messages() {
    // An issue message with a fake OpenAI key pattern
    let events = vec![UntrustedParserEvent::Issue {
        severity: ParsedIssueSeverity::Error,
        file: Some("src/lib.rs".to_string()),
        line: Some(1),
        column: Some(1),
        message: "token=ghp_FAKETOKEN1234567890abcdef leaked in logs".to_string(),
    }];
    let output = UntrustedParserOutput::Structured {
        cli_kind: "cargo".to_string(),
        events,
        raw_text: "raw".to_string(),
    };
    let result = normalize_issues(&output, DEFAULT_ISSUE_LIMIT);

    assert_eq!(result.fallback_mode, ParserFallbackMode::Structured);
    assert_eq!(result.parsed_issues.len(), 1);
    // token= is a sensitive key → value should be redacted OR ghp_ token redacted as standalone
    let msg = &result.parsed_issues[0].message;
    // Either the assignment redaction or the standalone token redaction must fire
    assert!(
        result.redaction_count > 0 || msg.contains("<REDACTED>"),
        "§E.2: secret in issue message must be redacted, got: {msg}"
    );
}
