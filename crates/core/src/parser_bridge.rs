//! MVP-18 parser bridge.
//!
//! The bridge normalizes already-parsed, untrusted CLI output into the stable
//! `ParsedIssue` shape. It intentionally does not depend on SPIKE-07 prototype
//! crates or re-implement a CLI transcript parser.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;
use ts_rs::TS;

use crate::sanitize::{sanitize_ai_prompt, SanitizeCtx, SanitizeError};

pub const DEFAULT_ISSUE_LIMIT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum ParsedIssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ParsedIssue {
    pub severity: ParsedIssueSeverity,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParserFallbackMode {
    Structured,
    RawText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, Error)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PaneLinkError {
    #[error("parser unavailable: {message}")]
    ParserUnavailable { message: String },
    #[error("parser timeout: {message}")]
    ParserTimeout { message: String },
    #[error("prompt sanitization failed: {message}")]
    PromptSanitizationFailed { message: String },
    #[error("unsupported CLI kind: {message}")]
    UnsupportedCliKind { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UntrustedParserOutput {
    Structured {
        cli_kind: String,
        events: Vec<UntrustedParserEvent>,
        raw_text: String,
    },
    ParserUnavailable {
        cli_kind: Option<String>,
        message: String,
        raw_text: String,
    },
    ParserTimeout {
        cli_kind: Option<String>,
        message: String,
        raw_text: String,
    },
    UnsupportedCliKind {
        cli_kind: String,
        raw_text: String,
    },
    ParserCrash {
        cli_kind: Option<String>,
        message: String,
        raw_text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UntrustedParserEvent {
    Issue {
        severity: ParsedIssueSeverity,
        file: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
        message: String,
    },
    Text {
        text: String,
    },
    Error {
        message: String,
    },
    Finish {
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizeResult {
    pub parsed_issues: Vec<ParsedIssue>,
    pub raw_excerpt: String,
    pub fallback_mode: ParserFallbackMode,
    pub truncated_count: usize,
    pub redaction_count: usize,
    pub error: Option<PaneLinkError>,
}

pub fn normalize_issues(raw_parser_out: &UntrustedParserOutput, limit: usize) -> NormalizeResult {
    let ctx = SanitizeCtx::default();

    match raw_parser_out {
        UntrustedParserOutput::Structured {
            cli_kind,
            events,
            raw_text,
        } => {
            if !is_supported_cli_kind(cli_kind) {
                return raw_fallback(
                    raw_text,
                    PaneLinkError::UnsupportedCliKind {
                        message: cli_kind.clone(),
                    },
                    &ctx,
                );
            }

            normalize_structured(events, raw_text, limit, &ctx)
        }
        UntrustedParserOutput::ParserUnavailable {
            message, raw_text, ..
        }
        | UntrustedParserOutput::ParserCrash {
            message, raw_text, ..
        } => raw_fallback(
            raw_text,
            PaneLinkError::ParserUnavailable {
                message: message.clone(),
            },
            &ctx,
        ),
        UntrustedParserOutput::ParserTimeout {
            message, raw_text, ..
        } => raw_fallback(
            raw_text,
            PaneLinkError::ParserTimeout {
                message: message.clone(),
            },
            &ctx,
        ),
        UntrustedParserOutput::UnsupportedCliKind { cli_kind, raw_text } => raw_fallback(
            raw_text,
            PaneLinkError::UnsupportedCliKind {
                message: cli_kind.clone(),
            },
            &ctx,
        ),
    }
}

fn normalize_structured(
    events: &[UntrustedParserEvent],
    raw_text: &str,
    limit: usize,
    ctx: &SanitizeCtx,
) -> NormalizeResult {
    let mut redaction_count = 0;
    let raw_excerpt = match sanitize_text(raw_text, ctx, &mut redaction_count) {
        Ok(text) => text,
        Err(error) => return sanitization_fallback(raw_text, error, ctx),
    };

    let mut issues = Vec::new();
    for event in events {
        let UntrustedParserEvent::Issue {
            severity,
            file,
            line,
            column,
            message,
        } = event
        else {
            continue;
        };

        let message = match sanitize_text(message, ctx, &mut redaction_count) {
            Ok(text) => text,
            Err(error) => return sanitization_fallback(raw_text, error, ctx),
        };
        let file = match file {
            Some(value) => match sanitize_text(value, ctx, &mut redaction_count) {
                Ok(text) => Some(text),
                Err(error) => return sanitization_fallback(raw_text, error, ctx),
            },
            None => None,
        };

        issues.push(ParsedIssue {
            severity: *severity,
            file,
            line: *line,
            column: *column,
            message,
        });
    }

    let mut seen = HashSet::new();
    issues.retain(|issue| seen.insert((issue.severity, issue.file.clone(), issue.line)));

    let effective_limit = effective_issue_limit(limit);
    let truncated_count = issues.len().saturating_sub(effective_limit);
    issues.truncate(effective_limit);

    NormalizeResult {
        parsed_issues: issues,
        raw_excerpt,
        fallback_mode: ParserFallbackMode::Structured,
        truncated_count,
        redaction_count,
        error: None,
    }
}

fn raw_fallback(raw_text: &str, error: PaneLinkError, ctx: &SanitizeCtx) -> NormalizeResult {
    let mut redaction_count = 0;
    let raw_excerpt = match sanitize_text(raw_text, ctx, &mut redaction_count) {
        Ok(text) => text,
        Err(sanitize_error) => {
            return sanitization_fallback(raw_text, sanitize_error, ctx);
        }
    };

    NormalizeResult {
        parsed_issues: Vec::new(),
        raw_excerpt,
        fallback_mode: ParserFallbackMode::RawText,
        truncated_count: 0,
        redaction_count,
        error: Some(error),
    }
}

fn sanitization_fallback(
    raw_text: &str,
    error: SanitizeError,
    ctx: &SanitizeCtx,
) -> NormalizeResult {
    let mut redaction_count = 0;
    let raw_excerpt = match sanitize_ai_prompt(raw_text, ctx) {
        Ok(out) => {
            redaction_count += out.redaction_count;
            out.text
        }
        Err(_) => String::new(),
    };

    NormalizeResult {
        parsed_issues: Vec::new(),
        raw_excerpt,
        fallback_mode: ParserFallbackMode::RawText,
        truncated_count: 0,
        redaction_count,
        error: Some(PaneLinkError::PromptSanitizationFailed {
            message: error.to_string(),
        }),
    }
}

fn sanitize_text(
    raw: &str,
    ctx: &SanitizeCtx,
    redaction_count: &mut usize,
) -> Result<String, SanitizeError> {
    let sanitized = sanitize_ai_prompt(raw, ctx)?;
    *redaction_count += sanitized.redaction_count;
    Ok(sanitized.text)
}

fn effective_issue_limit(limit: usize) -> usize {
    if limit == 0 {
        DEFAULT_ISSUE_LIMIT
    } else {
        limit.min(DEFAULT_ISSUE_LIMIT)
    }
}

fn is_supported_cli_kind(cli_kind: &str) -> bool {
    matches!(
        cli_kind.to_ascii_lowercase().as_str(),
        "claude"
            | "codex"
            | "compiler"
            | "rustc"
            | "cargo"
            | "vitest"
            | "pytest"
            | "go"
            | "go test"
            | "npm"
            | "pnpm"
            | "tsc"
            | "gcc"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(MVP-18 A3): switch inline samples to crates/core/tests/fixtures/pane_link once fixtures land.

    fn issue(
        severity: ParsedIssueSeverity,
        file: &str,
        line: u32,
        message: &str,
    ) -> UntrustedParserEvent {
        UntrustedParserEvent::Issue {
            severity,
            file: Some(file.to_string()),
            line: Some(line),
            column: Some(7),
            message: message.to_string(),
        }
    }

    #[test]
    fn normalizes_structured_issues_with_sanitized_messages() {
        let input = UntrustedParserOutput::Structured {
            cli_kind: "codex".to_string(),
            events: vec![issue(
                ParsedIssueSeverity::Error,
                "src/lib.rs",
                42,
                "\u{1b}[31mcannot find value\u{1b}[0m OPENAI_API_KEY=sk-test1234567890",
            )],
            raw_text: "raw error".to_string(),
        };

        let result = normalize_issues(&input, DEFAULT_ISSUE_LIMIT);

        assert_eq!(result.fallback_mode, ParserFallbackMode::Structured);
        assert_eq!(result.parsed_issues.len(), 1);
        assert_eq!(
            result.parsed_issues[0].message,
            "cannot find value OPENAI_API_KEY=<REDACTED>"
        );
        assert_eq!(result.redaction_count, 1);
        assert_eq!(result.error, None);
    }

    #[test]
    fn caps_issues_at_twenty_after_deduplicating_by_severity_file_line() {
        let mut events = Vec::new();
        for line in 1..=25 {
            events.push(issue(
                ParsedIssueSeverity::Error,
                "src/lib.rs",
                line,
                &format!("error {line}"),
            ));
        }
        events.push(issue(
            ParsedIssueSeverity::Error,
            "src/lib.rs",
            1,
            "duplicate should be ignored",
        ));
        let input = UntrustedParserOutput::Structured {
            cli_kind: "codex".to_string(),
            events,
            raw_text: "raw".to_string(),
        };

        let result = normalize_issues(&input, 100);

        assert_eq!(result.parsed_issues.len(), DEFAULT_ISSUE_LIMIT);
        assert_eq!(result.truncated_count, 5);
        assert_eq!(result.parsed_issues[0].message, "error 1");
    }

    #[test]
    fn parser_timeout_falls_back_to_sanitized_raw_text() {
        let input = UntrustedParserOutput::ParserTimeout {
            cli_kind: Some("codex".to_string()),
            message: "2s deadline exceeded".to_string(),
            raw_text: "\u{1b}[31mraw failure\u{1b}[0m token=ghp_1234567890abcdef".to_string(),
        };

        let result = normalize_issues(&input, DEFAULT_ISSUE_LIMIT);

        assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
        assert_eq!(result.parsed_issues, Vec::new());
        assert_eq!(result.raw_excerpt, "raw failure token=<REDACTED>");
        assert_eq!(
            result.error,
            Some(PaneLinkError::ParserTimeout {
                message: "2s deadline exceeded".to_string()
            })
        );
    }

    #[test]
    fn unsupported_cli_kind_falls_back_without_panicking() {
        let input = UntrustedParserOutput::UnsupportedCliKind {
            cli_kind: "unknown-cli".to_string(),
            raw_text: "plain output".to_string(),
        };

        let result = normalize_issues(&input, DEFAULT_ISSUE_LIMIT);

        assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
        assert_eq!(
            result.error,
            Some(PaneLinkError::UnsupportedCliKind {
                message: "unknown-cli".to_string()
            })
        );
        assert_eq!(result.raw_excerpt, "plain output");
    }

    #[test]
    fn parser_crash_falls_back_as_unavailable() {
        let input = UntrustedParserOutput::ParserCrash {
            cli_kind: Some("codex".to_string()),
            message: "panic while parsing".to_string(),
            raw_text: "raw crash text".to_string(),
        };

        let result = normalize_issues(&input, DEFAULT_ISSUE_LIMIT);

        assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
        assert_eq!(
            result.error,
            Some(PaneLinkError::ParserUnavailable {
                message: "panic while parsing".to_string()
            })
        );
    }

    #[test]
    fn sanitization_failure_returns_prompt_sanitization_error() {
        let input = UntrustedParserOutput::Structured {
            cli_kind: "codex".to_string(),
            events: vec![issue(
                ParsedIssueSeverity::Error,
                "src/lib.rs",
                1,
                "bad\0message",
            )],
            raw_text: "raw".to_string(),
        };

        let result = normalize_issues(&input, DEFAULT_ISSUE_LIMIT);

        assert_eq!(result.fallback_mode, ParserFallbackMode::RawText);
        assert_eq!(result.parsed_issues, Vec::new());
        assert_eq!(
            result.error,
            Some(PaneLinkError::PromptSanitizationFailed {
                message: "prompt contains a NUL byte".to_string()
            })
        );
    }
}
