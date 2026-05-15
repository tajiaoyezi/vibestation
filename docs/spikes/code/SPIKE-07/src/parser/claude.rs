//! Claude CLI adapter（**Phase B dispatch track 1 · 独占本文件**）。
//!
//! Phase A 实测画像（见 README §关键结论溯源）：Claude = **薄协议**
//! - `happy_path` 196–238B：纯 assistant 文本 + exit 0 · 无 role marker / 无 session id / 无 hook
//! - `auth_fail` 127B："Invalid API key · Fix external API key" + exit 1
//! - `network_error` / `interrupt_residual` / `long_stream` / `mixed_ansi_json`：
//!   32–300KB 终端 TUI 重绘流（exit 0 · corpus 质量 caveat · 见 Phase D）
//!
//! Phase B 实现策略：
//! 1. ANSI/OSC/CSI 剥离 → 纯文本
//! 2. 错误优先判定：Auth 模式 → Error{Auth} · 网络模式 → Error{Network}
//! 3. happy path：小输出 + exit 0 → MessageStart/MessageDelta/MessageEnd{Stop}
//! 4. 大 TUI 重绘流（>5KB）→ Unrecognized（corpus 质量问题 · Phase D 分析）
//!
//! **契约硬约束**：不得改 `crate::ir` 的 enum/trait 签名；不得 panic；不认识的
//! 块包 `Unrecognized`；只编辑本文件（`src/parser/claude.rs`）。

use crate::ir::{CliEvent, CliParser, ErrorKind, FinishReason, ParseInput, ParseResult, Role};

pub struct ClaudeParser;

/// Strip ANSI/VT control sequences from terminal output.
///
/// Handles:
/// - CSI: `ESC [ ... final_byte` (final_byte in 0x40–0x7e)
/// - OSC: `ESC ] ... BEL(\x07)` or `ESC ] ... ST(\x1b\\)`
/// - 2-char escapes: `ESC >`, `ESC <`, `ESC =`, `ESC (`, `ESC )`
/// - Strips bare `\r` (keeps `\n`)
fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\x1b' => match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    // CSI: consume until final byte in [0x40, 0x7e]
                    for c2 in chars.by_ref() {
                        if ('\x40'..='\x7e').contains(&c2) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    // OSC: consume until BEL or ST (ESC \)
                    let mut prev = '\0';
                    for c2 in chars.by_ref() {
                        if c2 == '\x07' {
                            break;
                        }
                        if prev == '\x1b' && c2 == '\\' {
                            break;
                        }
                        prev = c2;
                    }
                }
                Some('>') | Some('<') | Some('=') | Some('(') | Some(')') => {
                    chars.next();
                }
                _ => {
                    // Unknown escape: skip only the ESC byte
                }
            },
            '\r' => {}
            other => result.push(other),
        }
    }
    result
}

/// Return the first non-empty trimmed line from text.
fn first_nonempty_line(text: &str) -> String {
    text.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or(text)
        .trim()
        .to_string()
}

/// Extract a brief network error message.
/// Prefers "Execution error" as the user-visible line; falls back to first 128 chars.
fn network_error_summary(text: &str) -> String {
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("Execution error") {
            return t.to_string();
        }
    }
    text.chars().take(128).collect()
}

impl CliParser for ClaudeParser {
    fn cli_id(&self) -> &'static str {
        "claude"
    }

    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        if input.raw_output.is_empty() {
            return ParseResult::default();
        }

        let raw = input.raw_output;
        let stripped = strip_ansi(raw);
        let text = stripped.trim();

        // 1. Auth error: known error patterns (always exit 1 in corpus)
        if text.contains("Invalid API key") || text.contains("Fix external API key") {
            let msg = first_nonempty_line(text);
            return ParseResult {
                events: vec![
                    CliEvent::Error {
                        kind: ErrorKind::Auth,
                        message: msg,
                        recoverable: false,
                    },
                    CliEvent::MessageEnd {
                        finish_reason: FinishReason::Error(ErrorKind::Auth),
                    },
                ],
            };
        }

        // 2. Network error: connection/request failures in logs (exit 0 in corpus)
        if text.contains("Connection error") || text.contains("Request was aborted") {
            let msg = network_error_summary(text);
            return ParseResult {
                events: vec![
                    CliEvent::Error {
                        kind: ErrorKind::Network,
                        message: msg,
                        recoverable: true,
                    },
                    CliEvent::MessageEnd {
                        finish_reason: FinishReason::Error(ErrorKind::Network),
                    },
                ],
            };
        }

        // 3. Other non-zero exit codes not matched above
        if input.exit_code.map(|c| c != 0).unwrap_or(false) {
            let msg: String = text.chars().take(256).collect();
            return ParseResult {
                events: vec![
                    CliEvent::Error {
                        kind: ErrorKind::Unknown,
                        message: msg,
                        recoverable: false,
                    },
                    CliEvent::MessageEnd {
                        finish_reason: FinishReason::Error(ErrorKind::Unknown),
                    },
                ],
            };
        }

        // 4. Large TUI stream (interrupt_residual / long_stream / mixed_ansi_json)
        // Threshold: 5KB raw output → TUI redraw corpus (Phase D 会做质量分析)
        if raw.len() > 5_000 {
            return ParseResult {
                events: vec![CliEvent::Unrecognized {
                    raw: raw.chars().take(120).collect(),
                    heuristic: Some("TUI 重绘 / 大流式输出".into()),
                }],
            };
        }

        // 5. Happy path: small exit-0 output with readable assistant text
        if !text.is_empty() {
            return ParseResult {
                events: vec![
                    CliEvent::MessageStart {
                        role: Role::Assistant,
                    },
                    CliEvent::MessageDelta {
                        content: text.to_string(),
                        raw_ansi: Some(raw.to_string()),
                    },
                    CliEvent::MessageEnd {
                        finish_reason: FinishReason::Stop,
                    },
                ],
            };
        }

        // 6. Fallback (e.g. output is purely ANSI with no readable text)
        ParseResult {
            events: vec![CliEvent::Unrecognized {
                raw: raw.chars().take(120).collect(),
                heuristic: Some("无法解析：无可读内容".into()),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CliParser, ErrorKind, FinishReason, ParseInput};

    // Inline sample: auth_fail corpus pattern (· = U+00B7)
    const AUTH_FAIL_RAW: &str = "Invalid API key \u{00b7} Fix external API key\r\n\
         \x1b[?1006l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[>4m\x1b[<u\x1b[?1004l\x1b[?25h\
         \x1b]9;4;0;\x07\x1b[?25h";

    // Inline sample: happy_path corpus pattern
    const HAPPY_RAW: &str = "Vibestation 是面向 Claude CLI 用户的桌面应用。\r\n\
         \x1b[?1006l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[>4m\x1b[<u\x1b[?1004l\x1b[?25h\
         \x1b]9;4;0;\x07\x1b[?25h";

    #[test]
    fn auth_fail_produces_auth_error_not_recoverable() {
        let p = ClaudeParser;
        let r = p.parse(&ParseInput {
            raw_output: AUTH_FAIL_RAW,
            exit_code: Some(1),
        });
        // Must contain Error{Auth, recoverable: false}
        let has_auth_err = r.events.iter().any(|e| {
            matches!(
                e,
                CliEvent::Error {
                    kind: ErrorKind::Auth,
                    recoverable: false,
                    ..
                }
            )
        });
        assert!(has_auth_err, "auth_fail must produce Error{{Auth}}");
        // Must contain MessageEnd{Error(Auth)}
        let has_end = r.events.iter().any(|e| {
            matches!(
                e,
                CliEvent::MessageEnd {
                    finish_reason: FinishReason::Error(ErrorKind::Auth),
                }
            )
        });
        assert!(has_end, "auth_fail must produce MessageEnd{{Error(Auth)}}");
        assert!(!r.events.is_empty());
    }

    #[test]
    fn happy_path_produces_message_start_delta_end() {
        let p = ClaudeParser;
        let r = p.parse(&ParseInput {
            raw_output: HAPPY_RAW,
            exit_code: Some(0),
        });
        // MessageStart{Assistant}
        let has_start = r.events.iter().any(|e| {
            matches!(
                e,
                CliEvent::MessageStart {
                    role: Role::Assistant
                }
            )
        });
        assert!(
            has_start,
            "happy_path must produce MessageStart{{Assistant}}"
        );
        // MessageDelta with non-empty content
        let has_delta = r.events.iter().any(|e| {
            if let CliEvent::MessageDelta { content, .. } = e {
                !content.is_empty()
            } else {
                false
            }
        });
        assert!(has_delta, "happy_path must produce non-empty MessageDelta");
        // MessageEnd{Stop}
        let has_stop = r.events.iter().any(|e| {
            matches!(
                e,
                CliEvent::MessageEnd {
                    finish_reason: FinishReason::Stop
                }
            )
        });
        assert!(has_stop, "happy_path must produce MessageEnd{{Stop}}");
    }

    #[test]
    fn empty_input_yields_no_events() {
        let p = ClaudeParser;
        let r = p.parse(&ParseInput {
            raw_output: "",
            exit_code: Some(0),
        });
        assert!(r.events.is_empty(), "empty input must yield no events");
    }

    #[test]
    fn network_error_produces_network_error_recoverable() {
        let p = ClaudeParser;
        let raw = "\x1b[?25hExecution error\r\n--- debug log ---\r\n\
                   [ERROR] API error (attempt 1/11): undefined Connection error.\r\n";
        let r = p.parse(&ParseInput {
            raw_output: raw,
            exit_code: Some(0),
        });
        let has_net = r.events.iter().any(|e| {
            matches!(
                e,
                CliEvent::Error {
                    kind: ErrorKind::Network,
                    recoverable: true,
                    ..
                }
            )
        });
        assert!(
            has_net,
            "network_error must produce Error{{Network, recoverable:true}}"
        );
    }

    #[test]
    fn large_tui_produces_unrecognized_not_empty() {
        let p = ClaudeParser;
        let large = "x".repeat(6_000);
        let r = p.parse(&ParseInput {
            raw_output: &large,
            exit_code: Some(0),
        });
        assert!(
            !r.events.is_empty(),
            "large TUI must not yield empty events"
        );
        assert!(
            r.events.iter().any(|e| e.is_unrecognized()),
            "large TUI must yield Unrecognized"
        );
    }

    #[test]
    fn no_panic_on_edge_cases() {
        let p = ClaudeParser;
        for (raw, exit_code) in &[
            ("\x1b", Some(0i64)),
            ("\x1b[", Some(0)),
            ("\x1b]", Some(0)),
            ("\x1b]unterminated", Some(0)),
            ("\x00\x01\x02", Some(0)),
            ("only ansi \x1b[?25h", None),
        ] {
            let _ = p.parse(&ParseInput {
                raw_output: raw,
                exit_code: *exit_code,
            });
        }
    }
}
