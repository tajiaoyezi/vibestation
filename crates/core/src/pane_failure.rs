//! MVP-18 Phase C failure feedback wire.

use crate::pane_links::{PaneLink, PaneLinkError, PaneTriggerEvent};
use crate::parser_bridge::{
    normalize_issues, ParsedIssue, ParserBridgeError, ParserFallbackMode, UntrustedParserEvent,
    UntrustedParserOutput, DEFAULT_ISSUE_LIMIT,
};
use crate::sanitize::{sanitize_ai_prompt, SanitizeCtx};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum PaneFailureFallbackMode {
    Structured,
    RawText,
}

impl From<ParserFallbackMode> for PaneFailureFallbackMode {
    fn from(value: ParserFallbackMode) -> Self {
        match value {
            ParserFallbackMode::Structured => PaneFailureFallbackMode::Structured,
            ParserFallbackMode::RawText => PaneFailureFallbackMode::RawText,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaneFailureTriggerReason {
    ExitCode,
    Signal,
    WatchRerun,
    ManualRerun,
}

impl PaneFailureTriggerReason {
    fn as_event_reason(&self) -> &'static str {
        match self {
            PaneFailureTriggerReason::ExitCode => "exitCode",
            PaneFailureTriggerReason::Signal => "signal",
            PaneFailureTriggerReason::WatchRerun => "watchRerun",
            PaneFailureTriggerReason::ManualRerun => "manualRerun",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneFailurePreviewRequest {
    pub workspace_id: String,
    pub child_pane_id: String,
    pub command_run_id: String,
    pub exit_code: Option<i32>,
    pub command: String,
    pub cwd: String,
    pub cli_kind: String,
    pub raw_output: String,
    pub parsed_issues: Vec<ParsedIssue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneFailurePreviewResult {
    pub prompt_fragment: String,
    pub raw_excerpt: String,
    pub parsed_issues: Vec<ParsedIssue>,
    pub parser_confidence: f32,
    pub fallback_mode: PaneFailureFallbackMode,
    pub truncated_count: usize,
    pub redaction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneBuildFailedEvent {
    pub workspace_id: String,
    pub link_id: String,
    pub parent_pane_id: String,
    pub child_pane_id: String,
    pub command_run_id: String,
    pub exit_code: Option<i32>,
    pub raw_excerpt: String,
    pub parsed_issues: Vec<ParsedIssue>,
    pub parser_confidence: f32,
    pub fallback_mode: PaneFailureFallbackMode,
    #[ts(type = "number")]
    pub occurred_at: i64,
    pub truncated_count: usize,
    pub redaction_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneFailureSource {
    pub workspace_id: String,
    pub child_pane_id: String,
    pub command_run_id: String,
    pub reason: PaneFailureTriggerReason,
    pub exit_code: Option<i32>,
    pub command: String,
    pub cwd: String,
    pub cli_kind: String,
    pub raw_output: String,
    pub parsed_issues: Vec<ParsedIssue>,
    pub occurred_at: i64,
}

pub fn preview_failure_prompt(
    request: &PaneFailurePreviewRequest,
) -> Result<PaneFailurePreviewResult, PaneLinkError> {
    let source = PaneFailureSource {
        workspace_id: request.workspace_id.clone(),
        child_pane_id: request.child_pane_id.clone(),
        command_run_id: request.command_run_id.clone(),
        reason: PaneFailureTriggerReason::ExitCode,
        exit_code: request.exit_code,
        command: request.command.clone(),
        cwd: request.cwd.clone(),
        cli_kind: request.cli_kind.clone(),
        raw_output: request.raw_output.clone(),
        parsed_issues: request.parsed_issues.clone(),
        occurred_at: 0,
    };
    let normalized = normalize_failure_source(&source);
    if let Some(error @ ParserBridgeError::PromptSanitizationFailed { .. }) =
        normalized.error.clone()
    {
        return Err(map_parser_bridge_error(error));
    }

    let mut redaction_count = normalized.redaction_count;
    let prompt_fragment =
        build_prompt_fragment(&source, &normalized).map_err(map_parser_bridge_error)?;
    redaction_count += prompt_fragment.redaction_count;

    Ok(PaneFailurePreviewResult {
        prompt_fragment: prompt_fragment.text,
        raw_excerpt: normalized.raw_excerpt,
        parsed_issues: normalized.parsed_issues,
        parser_confidence: parser_confidence(normalized.fallback_mode),
        fallback_mode: normalized.fallback_mode.into(),
        truncated_count: normalized.truncated_count,
        redaction_count,
    })
}

pub fn build_failure_events(
    link: &PaneLink,
    source: &PaneFailureSource,
) -> (PaneTriggerEvent, PaneBuildFailedEvent) {
    let normalized = normalize_failure_source(source);
    let trigger = PaneTriggerEvent {
        workspace_id: source.workspace_id.clone(),
        child_pane_id: source.child_pane_id.clone(),
        command_run_id: source.command_run_id.clone(),
        reason: source.reason.as_event_reason().to_string(),
        exit_code: source.exit_code,
        command: source.command.clone(),
        occurred_at: source.occurred_at,
    };
    let build_failed = PaneBuildFailedEvent {
        workspace_id: source.workspace_id.clone(),
        link_id: link.id.clone(),
        parent_pane_id: link.parent_pane_id.clone(),
        child_pane_id: source.child_pane_id.clone(),
        command_run_id: source.command_run_id.clone(),
        exit_code: source.exit_code,
        raw_excerpt: normalized.raw_excerpt,
        parsed_issues: normalized.parsed_issues,
        parser_confidence: parser_confidence(normalized.fallback_mode),
        fallback_mode: normalized.fallback_mode.into(),
        occurred_at: source.occurred_at,
        truncated_count: normalized.truncated_count,
        redaction_count: normalized.redaction_count,
    };

    (trigger, build_failed)
}

pub fn map_parser_bridge_error(error: ParserBridgeError) -> PaneLinkError {
    match error {
        ParserBridgeError::ParserUnavailable { message } => {
            PaneLinkError::ParserUnavailable(message)
        }
        ParserBridgeError::ParserTimeout { message } => PaneLinkError::ParserTimeout(message),
        ParserBridgeError::PromptSanitizationFailed { message } => {
            PaneLinkError::PromptSanitizationFailed(message)
        }
        ParserBridgeError::UnsupportedCliKind { message } => {
            PaneLinkError::UnsupportedCliKind(message)
        }
    }
}

fn normalize_failure_source(source: &PaneFailureSource) -> crate::parser_bridge::NormalizeResult {
    let parser_output = if source.parsed_issues.is_empty() {
        UntrustedParserOutput::ParserUnavailable {
            cli_kind: Some(source.cli_kind.clone()),
            message: "structured parser output unavailable for pane failure pipeline".to_string(),
            raw_text: source.raw_output.clone(),
        }
    } else {
        UntrustedParserOutput::Structured {
            cli_kind: source.cli_kind.clone(),
            events: source
                .parsed_issues
                .iter()
                .map(|issue| UntrustedParserEvent::Issue {
                    severity: issue.severity,
                    file: issue.file.clone(),
                    line: issue.line,
                    column: issue.column,
                    message: issue.message.clone(),
                })
                .collect(),
            raw_text: source.raw_output.clone(),
        }
    };

    let mut normalized = normalize_issues(&parser_output, DEFAULT_ISSUE_LIMIT);
    apply_workspace_sanitization(&mut normalized, &source.cwd);
    normalized
}

fn apply_workspace_sanitization(
    normalized: &mut crate::parser_bridge::NormalizeResult,
    workspace_root: &str,
) {
    let ctx = SanitizeCtx::new(workspace_root);
    if let Ok(raw_excerpt) = sanitize_ai_prompt(&normalized.raw_excerpt, &ctx) {
        normalized.raw_excerpt = raw_excerpt.text;
    }
    for issue in &mut normalized.parsed_issues {
        if let Some(file) = &issue.file {
            if let Ok(sanitized) = sanitize_ai_prompt(file, &ctx) {
                issue.file = Some(sanitized.text);
            }
        }
    }
}

fn build_prompt_fragment(
    source: &PaneFailureSource,
    normalized: &crate::parser_bridge::NormalizeResult,
) -> Result<crate::sanitize::SanitizedPrompt, ParserBridgeError> {
    let ctx = SanitizeCtx::new(&source.cwd);
    let mut redaction_count = 0;
    let source_pane = sanitize_prompt_field(&source.child_pane_id, &ctx, &mut redaction_count)?;
    let command_run = sanitize_prompt_field(&source.command_run_id, &ctx, &mut redaction_count)?;
    let command = sanitize_prompt_field(&source.command, &ctx, &mut redaction_count)?;
    let cwd = sanitize_prompt_field(&source.cwd, &ctx, &mut redaction_count)?;
    let issue_lines = if normalized.parsed_issues.is_empty() {
        "No structured issues were available; use the raw excerpt.".to_string()
    } else {
        normalized
            .parsed_issues
            .iter()
            .map(format_issue)
            .collect::<Vec<_>>()
            .join("\n")
    };
    let fragment = format!(
        "Pane failure context\nSource pane: {}\nCommand run: {}\nCommand: {}\nCwd: {}\nExit code: {}\nParser mode: {:?}\nIssues:\n{}\nRaw excerpt:\n{}",
        source_pane,
        command_run,
        command,
        cwd,
        source
            .exit_code
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        PaneFailureFallbackMode::from(normalized.fallback_mode),
        issue_lines,
        normalized.raw_excerpt,
    );
    let mut sanitized = sanitize_ai_prompt(&fragment, &ctx).map_err(|error| {
        ParserBridgeError::PromptSanitizationFailed {
            message: error.to_string(),
        }
    })?;
    sanitized.redaction_count = redaction_count;
    Ok(sanitized)
}

fn format_issue(issue: &ParsedIssue) -> String {
    let location = match (&issue.file, issue.line, issue.column) {
        (Some(file), Some(line), Some(column)) => format!("{file}:{line}:{column}"),
        (Some(file), Some(line), None) => format!("{file}:{line}"),
        (Some(file), None, _) => file.clone(),
        (None, _, _) => "unknown location".to_string(),
    };
    format!("- {:?} {location}: {}", issue.severity, issue.message)
}

fn sanitize_prompt_field(
    raw: &str,
    ctx: &SanitizeCtx,
    redaction_count: &mut usize,
) -> Result<String, ParserBridgeError> {
    let sanitized = sanitize_ai_prompt(raw, ctx).map_err(|error| {
        ParserBridgeError::PromptSanitizationFailed {
            message: error.to_string(),
        }
    })?;
    *redaction_count += sanitized.redaction_count;
    Ok(sanitized.text)
}

fn parser_confidence(fallback_mode: ParserFallbackMode) -> f32 {
    match fallback_mode {
        ParserFallbackMode::Structured => 1.0,
        ParserFallbackMode::RawText => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pane_links::{PaneLink, PaneLinkError, PaneLinkKind};
    use crate::parser_bridge::{
        ParsedIssue, ParsedIssueSeverity, ParserBridgeError, DEFAULT_ISSUE_LIMIT,
    };

    fn link() -> PaneLink {
        PaneLink {
            id: "link-1".to_string(),
            workspace_id: "workspace-a".to_string(),
            parent_pane_id: "pane-ai".to_string(),
            child_pane_id: "pane-runner".to_string(),
            link_kind: PaneLinkKind::FailureFeedback,
            enabled: true,
            fallback_mode: "structured".to_string(),
            created_by: "user".to_string(),
            created_at: 1_760_000_000_000,
            updated_at: 1_760_000_000_000,
            last_triggered_at: None,
        }
    }

    #[test]
    fn preview_prompt_sanitizes_raw_text_and_preserves_issues() {
        let request = PaneFailurePreviewRequest {
            workspace_id: "workspace-a".to_string(),
            child_pane_id: "pane-runner".to_string(),
            command_run_id: "run-42".to_string(),
            exit_code: Some(101),
            command: "cargo test".to_string(),
            cwd: "/Users/leaf/project".to_string(),
            cli_kind: "cargo".to_string(),
            raw_output: "\u{1b}[31merror[E0425]\u{1b}[0m token=ghp_1234567890abcdef".to_string(),
            parsed_issues: vec![ParsedIssue {
                severity: ParsedIssueSeverity::Error,
                file: Some("/Users/leaf/project/src/lib.rs".to_string()),
                line: Some(42),
                column: Some(13),
                message: "cannot find value `foo`".to_string(),
            }],
        };

        let result = preview_failure_prompt(&request).expect("preview should be built");

        assert_eq!(result.fallback_mode, PaneFailureFallbackMode::Structured);
        assert_eq!(result.parsed_issues.len(), 1);
        assert_eq!(
            result.parsed_issues[0].file.as_deref(),
            Some("./src/lib.rs")
        );
        assert!(result.prompt_fragment.contains("cargo test"));
        assert!(result.prompt_fragment.contains("cannot find value `foo`"));
        assert!(!result.raw_excerpt.contains("ghp_"));
        assert_eq!(result.redaction_count, 1);
    }

    #[test]
    fn parser_bridge_errors_map_to_canonical_pane_link_errors() {
        assert_eq!(
            map_parser_bridge_error(ParserBridgeError::ParserUnavailable {
                message: "missing parser".to_string()
            }),
            PaneLinkError::ParserUnavailable("missing parser".to_string())
        );
        assert_eq!(
            map_parser_bridge_error(ParserBridgeError::ParserTimeout {
                message: "deadline".to_string()
            }),
            PaneLinkError::ParserTimeout("deadline".to_string())
        );
        assert_eq!(
            map_parser_bridge_error(ParserBridgeError::PromptSanitizationFailed {
                message: "prompt contains a NUL byte".to_string()
            }),
            PaneLinkError::PromptSanitizationFailed("prompt contains a NUL byte".to_string())
        );
        assert_eq!(
            map_parser_bridge_error(ParserBridgeError::UnsupportedCliKind {
                message: "fish".to_string()
            }),
            PaneLinkError::UnsupportedCliKind("fish".to_string())
        );
    }

    #[test]
    fn build_failure_events_caps_issues_and_shapes_trigger_and_build_failed_payloads() {
        let mut parsed_issues = Vec::new();
        for line in 1..=25 {
            parsed_issues.push(ParsedIssue {
                severity: ParsedIssueSeverity::Error,
                file: Some("src/lib.rs".to_string()),
                line: Some(line),
                column: Some(7),
                message: format!("error {line}"),
            });
        }
        let source = PaneFailureSource {
            workspace_id: "workspace-a".to_string(),
            child_pane_id: "pane-runner".to_string(),
            command_run_id: "run-42".to_string(),
            reason: PaneFailureTriggerReason::ExitCode,
            exit_code: Some(101),
            command: "cargo test".to_string(),
            cwd: "/Users/leaf/project".to_string(),
            cli_kind: "cargo".to_string(),
            raw_output: "raw compiler output".to_string(),
            parsed_issues,
            occurred_at: 1_760_000_000_200,
        };

        let (trigger, build_failed) = build_failure_events(&link(), &source);

        assert_eq!(trigger.workspace_id, "workspace-a");
        assert_eq!(trigger.child_pane_id, "pane-runner");
        assert_eq!(trigger.command_run_id, "run-42");
        assert_eq!(trigger.exit_code, Some(101));
        assert_eq!(trigger.command, "cargo test");
        assert_eq!(build_failed.link_id, "link-1");
        assert_eq!(build_failed.parent_pane_id, "pane-ai");
        assert_eq!(build_failed.parsed_issues.len(), DEFAULT_ISSUE_LIMIT);
        assert_eq!(build_failed.truncated_count, 5);
        assert_eq!(
            build_failed.fallback_mode,
            PaneFailureFallbackMode::Structured
        );
        assert_eq!(build_failed.parser_confidence, 1.0);
    }

    #[test]
    fn parser_unavailable_keeps_raw_fallback_event_without_dropping_source() {
        let source = PaneFailureSource {
            workspace_id: "workspace-a".to_string(),
            child_pane_id: "pane-runner".to_string(),
            command_run_id: "run-raw".to_string(),
            reason: PaneFailureTriggerReason::ExitCode,
            exit_code: Some(1),
            command: "npm test".to_string(),
            cwd: "/Users/leaf/project".to_string(),
            cli_kind: "npm".to_string(),
            raw_output: "\u{1b}[31mfailed\u{1b}[0m".to_string(),
            parsed_issues: Vec::new(),
            occurred_at: 1_760_000_000_300,
        };

        let (_trigger, build_failed) = build_failure_events(&link(), &source);

        assert_eq!(build_failed.command_run_id, "run-raw");
        assert_eq!(build_failed.child_pane_id, "pane-runner");
        assert_eq!(build_failed.raw_excerpt, "failed");
        assert!(build_failed.parsed_issues.is_empty());
        assert_eq!(build_failed.fallback_mode, PaneFailureFallbackMode::RawText);
        assert_eq!(build_failed.parser_confidence, 0.0);
    }
}
