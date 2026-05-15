//! Codex CLI adapter（**Phase B dispatch track 2 · 独占本文件**）。
//!
//! Phase A 实测画像（见 README §关键结论溯源）：Codex = **厚结构**
//! - header 段：`OpenAI Codex vX` + `--------` + `key: value` 行
//!   （`workdir:` `model:` `provider:` `approval:` `sandbox:` `reasoning effort:`
//!   `session id: <uuid>`）+ `--------`
//! - `user` role marker 行 + 用户 prompt 文本
//! - `hook: <Name>` / `hook: <Name> Completed`（SessionStart/UserPromptSubmit/Stop ...）
//! - `codex` role marker 行 + assistant 响应文本
//! - `tokens used` + 数字 footer
//! - `auth_fail` exit 1 · `interrupt_residual_1` 339KB/exit 143（真 SIGTERM）
//!   但 take 2/3 退化（corpus 质量 caveat · 见 Phase D）
//!
//! Phase B 任务：把下面 `CodexParser::parse` 从占位替换为真实厚结构解析
//! （session id → SessionMeta · user/codex → MessageStart{role} · hook → Hook ·
//! tokens used → Usage · exit≠0 / 文本模式 → Error · 正文 → MessageDelta）。
//! **契约硬约束**：不得改 `crate::ir` 的 enum/trait 签名；不得 panic；不认识的
//! 块包 `Unrecognized`；只编辑本文件（`src/parser/codex.rs`）。

use crate::ir::{CliEvent, CliParser, ErrorKind, FinishReason, ParseInput, ParseResult, Role};

pub struct CodexParser;

impl CliParser for CodexParser {
    fn cli_id(&self) -> &'static str {
        "codex"
    }

    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        if input.raw_output.is_empty() {
            return ParseResult::default();
        }

        let lines = normalize_lines(input.raw_output);
        let mut events = Vec::new();
        let mut active_role: Option<Role> = None;
        let mut message = MessageBuffer::default();
        let mut error_lines = Vec::new();
        let mut i = 0usize;

        while i < lines.len() {
            let (raw_line, clean_line) = &lines[i];
            let line = clean_line.trim();
            if line.is_empty() {
                if active_role.is_some() {
                    message.push(clean_line, raw_line);
                }
                i += 1;
                continue;
            }

            if line == "--------" {
                i += 1;
                continue;
            }

            if let Some((key, value)) = parse_meta_line(line) {
                flush_message(&mut events, &mut message);
                events.push(CliEvent::SessionMeta { key, value });
                i += 1;
                continue;
            }

            if line == "user" || line == "codex" {
                flush_message(&mut events, &mut message);
                if active_role.is_some() {
                    events.push(CliEvent::MessageEnd {
                        finish_reason: FinishReason::Stop,
                    });
                }
                let role = if line == "user" {
                    Role::User
                } else {
                    Role::Assistant
                };
                events.push(CliEvent::MessageStart { role: role.clone() });
                active_role = Some(role);
                i += 1;
                continue;
            }

            if let Some((name, completed)) = parse_hook_line(line) {
                flush_message(&mut events, &mut message);
                events.push(CliEvent::Hook { name, completed });
                i += 1;
                continue;
            }

            if line == "tokens used" {
                flush_message(&mut events, &mut message);
                if active_role.is_some() {
                    events.push(CliEvent::MessageEnd {
                        finish_reason: FinishReason::Stop,
                    });
                    active_role = None;
                }
                let (tokens, consumed) = parse_usage_after(&lines, i + 1);
                events.push(CliEvent::Usage { tokens });
                i += 1 + consumed;
                continue;
            }

            if !matches!(active_role, Some(Role::Assistant)) && is_error_line(line) {
                flush_message(&mut events, &mut message);
                error_lines.push(line.to_string());
                i += 1;
                continue;
            }

            if active_role.is_some() {
                message.push(clean_line, raw_line);
            } else {
                events.push(CliEvent::Unrecognized {
                    raw: clean_line.to_string(),
                    heuristic: Some(classify_unrecognized(line)),
                });
            }
            i += 1;
        }

        flush_message(&mut events, &mut message);

        if let Some(kind) = classify_error(input.exit_code, input.raw_output, &error_lines) {
            let message = if error_lines.is_empty() {
                format!("codex exited with code {:?}", input.exit_code)
            } else {
                error_lines.join("\n")
            };
            let recoverable = matches!(
                kind,
                ErrorKind::Network | ErrorKind::RateLimit | ErrorKind::Timeout
            );
            events.push(CliEvent::Error {
                kind: kind.clone(),
                message,
                recoverable,
            });
            events.push(CliEvent::MessageEnd {
                finish_reason: FinishReason::Error(kind),
            });
        } else if active_role.is_some() {
            events.push(CliEvent::MessageEnd {
                finish_reason: FinishReason::Stop,
            });
        }

        ParseResult { events }
    }
}

#[derive(Default)]
struct MessageBuffer {
    clean: Vec<String>,
    raw: Vec<String>,
    has_ansi: bool,
}

impl MessageBuffer {
    fn push(&mut self, clean: &str, raw: &str) {
        self.has_ansi |= clean != raw;
        self.clean.push(clean.to_string());
        self.raw.push(raw.to_string());
    }
}

fn flush_message(events: &mut Vec<CliEvent>, message: &mut MessageBuffer) {
    if message.clean.is_empty() {
        return;
    }
    let content = message.clean.join("\n");
    let content = content.trim_end_matches('\n').to_string();
    if !content.trim().is_empty() {
        let raw_ansi = if message.has_ansi {
            Some(message.raw.join("\n"))
        } else {
            None
        };
        events.push(CliEvent::MessageDelta { content, raw_ansi });
    }
    *message = MessageBuffer::default();
}

fn normalize_lines(input: &str) -> Vec<(String, String)> {
    input
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(|raw| (raw.to_string(), strip_ansi(raw)))
        .collect()
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\u{1b}' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('[') => {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
            Some(']') => {
                while let Some(c) = chars.next() {
                    if c == '\u{7}' {
                        break;
                    }
                    if c == '\u{1b}' && matches!(chars.peek(), Some('\\')) {
                        chars.next();
                        break;
                    }
                }
            }
            Some(_) | None => {}
        }
    }
    out
}

fn parse_meta_line(line: &str) -> Option<(String, String)> {
    if line.starts_with("OpenAI Codex ") {
        return Some(("banner".into(), line.to_string()));
    }
    let (key, value) = line.split_once(':')?;
    let key = key.trim().to_ascii_lowercase();
    let value = value.trim().to_string();
    let is_codex_header = matches!(
        key.as_str(),
        "workdir"
            | "model"
            | "provider"
            | "approval"
            | "sandbox"
            | "reasoning effort"
            | "reasoning summaries"
            | "session id"
    );
    if is_codex_header && !value.is_empty() {
        Some((key, value))
    } else {
        None
    }
}

fn parse_hook_line(line: &str) -> Option<(String, bool)> {
    let rest = line.strip_prefix("hook:")?.trim();
    if rest.is_empty() {
        return None;
    }
    if let Some(name) = rest.strip_suffix(" Completed") {
        Some((name.trim().to_string(), true))
    } else {
        Some((rest.to_string(), false))
    }
}

fn parse_usage_after(lines: &[(String, String)], start: usize) -> (Option<u64>, usize) {
    let mut consumed = 0usize;
    for (_, clean) in lines.iter().skip(start) {
        consumed += 1;
        let line = clean.trim();
        if line.is_empty() {
            continue;
        }
        let digits = line.replace(',', "");
        return (digits.parse::<u64>().ok(), consumed);
    }
    (None, consumed)
}

fn is_error_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("error:")
        || lower.contains(" unauthorized")
        || lower.contains("failed to connect")
        || lower.contains("connection refused")
        || lower.contains("turn interrupted")
}

fn classify_error(
    exit_code: Option<i64>,
    raw_output: &str,
    error_lines: &[String],
) -> Option<ErrorKind> {
    let error_text = error_lines.join("\n").to_ascii_lowercase();
    let haystack = if error_text.is_empty() && exit_code.is_some_and(|code| code != 0) {
        raw_output.to_ascii_lowercase()
    } else {
        error_text
    };
    if haystack.is_empty() && exit_code.is_none_or(|code| code == 0) {
        return None;
    }
    if haystack.contains("401 unauthorized")
        || haystack.contains("missing bearer")
        || haystack.contains("api key")
        || haystack.contains("authentication")
    {
        return Some(ErrorKind::Auth);
    }
    if haystack.contains("429") || haystack.contains("rate limit") {
        return Some(ErrorKind::RateLimit);
    }
    if haystack.contains("timeout") || exit_code == Some(124) {
        return Some(ErrorKind::Timeout);
    }
    if haystack.contains("failed to connect")
        || haystack.contains("connection refused")
        || haystack.contains("network")
        || haystack.contains("reconnecting")
        || haystack.contains("websocket")
    {
        return Some(ErrorKind::Network);
    }
    if exit_code.is_some_and(|code| code != 0) || !error_lines.is_empty() {
        return Some(ErrorKind::Unknown);
    }
    None
}

fn classify_unrecognized(line: &str) -> String {
    if line.starts_with('{') && line.ends_with('}') {
        "codex json event · no IR mapping in Phase B".into()
    } else {
        "codex text outside known structural markers".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ErrorKind, FinishReason, Role};

    #[test]
    fn parses_happy_path_thick_structure() {
        let p = CodexParser;
        assert_eq!(p.cli_id(), "codex");
        let r = p.parse(&ParseInput {
            raw_output: "OpenAI Codex v0.121.0 (research preview)\r\n--------\r\nworkdir: /tmp/work\r\nmodel: gpt-5.4\r\nprovider: openai\r\nsession id: 019daaf2-38bd-7123-8f60-977b2645674a\r\n--------\r\nuser\r\n用一句中文说明 Vibestation 是什么。\r\nhook: SessionStart\r\nhook: SessionStart Completed\r\nhook: UserPromptSubmit\r\nhook: UserPromptSubmit Completed\r\ncodex\r\nVibestation 是一款桌面开发工作台。\r\nhook: Stop\r\nhook: Stop Completed\r\ntokens used\r\n22,836\r\n",
            exit_code: Some(0),
        });
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::SessionMeta { key, value }
                if key == "session id" && value == "019daaf2-38bd-7123-8f60-977b2645674a"
        )));
        assert!(r
            .events
            .iter()
            .any(|e| matches!(e, CliEvent::MessageStart { role: Role::User })));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::MessageStart {
                role: Role::Assistant
            }
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::MessageDelta { content, .. } if content.contains("桌面开发工作台")
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::Hook { name, completed: false } if name == "SessionStart"
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::Usage {
                tokens: Some(22836)
            }
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::MessageEnd {
                finish_reason: FinishReason::Stop
            }
        )));
    }

    #[test]
    fn parses_auth_failure_as_nonrecoverable_error() {
        let p = CodexParser;
        let r = p.parse(&ParseInput {
            raw_output: "OpenAI Codex v0.121.0\r\nsession id: auth-1\r\nuser\r\nsay hi only\r\nERROR: unexpected status 401 Unauthorized: Missing bearer or basic authentication in header\r\n",
            exit_code: Some(1),
        });
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::Error {
                kind: ErrorKind::Auth,
                message,
                recoverable: false,
            } if message.contains("401 Unauthorized")
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::MessageEnd {
                finish_reason: FinishReason::Error(ErrorKind::Auth),
            }
        )));
    }

    #[test]
    fn parses_hook_completed_flag() {
        let p = CodexParser;
        let r = p.parse(&ParseInput {
            raw_output: "hook: UserPromptSubmit\r\nhook: UserPromptSubmit Completed\r\n",
            exit_code: Some(0),
        });
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::Hook { name, completed: false } if name == "UserPromptSubmit"
        )));
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::Hook { name, completed: true } if name == "UserPromptSubmit"
        )));
    }

    #[test]
    fn parses_tokens_used_with_commas() {
        let p = CodexParser;
        let r = p.parse(&ParseInput {
            raw_output: "tokens used\r\n26,978\r\n",
            exit_code: Some(0),
        });
        assert_eq!(
            r.events,
            vec![CliEvent::Usage {
                tokens: Some(26978)
            }]
        );
    }

    #[test]
    fn empty_input_returns_default() {
        let p = CodexParser;
        let r = p.parse(&ParseInput {
            raw_output: "",
            exit_code: Some(0),
        });
        assert!(r.events.is_empty());
    }

    #[test]
    fn sandbox_network_access_header_is_not_network_error() {
        let p = CodexParser;
        let r = p.parse(&ParseInput {
            raw_output: "OpenAI Codex v0.121.0\r\nsandbox: workspace-write (network access enabled)\r\nuser\r\nhi\r\ncodex\r\nhello\r\n",
            exit_code: Some(0),
        });
        assert!(!r.events.iter().any(|e| matches!(e, CliEvent::Error { .. })));
    }

    #[test]
    fn assistant_text_may_contain_error_word_without_runtime_error() {
        let p = CodexParser;
        let r = p.parse(&ParseInput {
            raw_output:
                "codex\r\nA grep result may include src/main.rs:12: Error: example text.\r\n",
            exit_code: Some(0),
        });
        assert!(r.events.iter().any(|e| matches!(
            e,
            CliEvent::MessageDelta { content, .. } if content.contains("Error: example text")
        )));
        assert!(!r.events.iter().any(|e| matches!(e, CliEvent::Error { .. })));
    }
}
