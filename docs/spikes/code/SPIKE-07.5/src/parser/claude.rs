//! Claude `stream-json` 结构化模式 adapter（SPIKE-07.5 新写 · SPIKE-07 TUI adapter 不复用）。
//!
//! 路由：`type` → `subtype`。实测结构（raw `/tmp/spike075-raw` · 已脱敏 corpus）：
//! - `system/init`        → `SessionMeta{session_id}`（结构化模式 claude **显式发**
//!   session_id · SPIKE-07 TUI 模式不发 · 这是 §H 统一抽象关键差异点本质改善）
//! - `system/hook_started`/`hook_response` → `Hook{name, completed}`
//! - `system/api_retry` · `rate_limit_event` → `Unrecognized`（诊断 · 非 Error ·
//!   最终错误在 `result` · 兜底不丢弃 spec §D）
//! - `assistant` → `message.content[].text` → `MessageStart{Assistant}`+`MessageDelta`
//! - `result is_error=false` → `MessageEnd{Stop}`
//! - `result is_error=true` → `Error{kind}`+`MessageEnd{Error(kind)}`
//!   kind：`api_error_status==401`/auth 关键字 ⇒ Auth(不可恢复)；
//!   连接/网络关键字 ⇒ Network(可恢复)；否则 Unknown
//! - 坏行（jsonl Bad）/未知 type → `Unrecognized`（不 panic · spec §D 契约）

use crate::ir::{CliEvent, CliParser, ErrorKind, ParseInput, ParseResult, Role};
use crate::jsonl::{parse_lines, JsonLine};
use serde_json::Value;

pub struct ClaudeParser;

fn classify_error(api_status: Option<i64>, msg: &str) -> ErrorKind {
    let low = msg.to_ascii_lowercase();
    if api_status == Some(401)
        || api_status == Some(403)
        || low.contains("invalid api key")
        || low.contains("unauthorized")
        || low.contains("authentication")
        || low.contains("forbidden")
    {
        return ErrorKind::Auth;
    }
    if low.contains("connectionrefused")
        || low.contains("unable to connect")
        || low.contains("connection")
        || low.contains("network")
        || low.contains("dns")
        || low.contains("econnrefused")
    {
        return ErrorKind::Network;
    }
    ErrorKind::Unknown
}

fn push_assistant(out: &mut Vec<CliEvent>, ev: &Value) {
    let texts: Vec<String> = ev
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter(|c| c.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|c| c.get("text").and_then(Value::as_str))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    // 仅当有实际文本才开 message（避免空 assistant 造成悬空 MessageStart）。
    if texts.iter().any(|t| !t.is_empty()) {
        out.push(CliEvent::MessageStart {
            role: Role::Assistant,
        });
        for t in texts {
            if !t.is_empty() {
                out.push(CliEvent::MessageDelta {
                    content: t,
                    raw_ansi: None,
                });
            }
        }
    }
}

fn map_event(out: &mut Vec<CliEvent>, v: &Value) {
    let ty = v.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "system" => {
            let sub = v.get("subtype").and_then(Value::as_str).unwrap_or("");
            match sub {
                "init" => {
                    if let Some(sid) = v.get("session_id").and_then(Value::as_str) {
                        out.push(CliEvent::SessionMeta {
                            key: "session_id".into(),
                            value: sid.to_string(),
                        });
                    }
                }
                "hook_started" | "hook_response" => {
                    let name = v
                        .get("hook_name")
                        .and_then(Value::as_str)
                        .unwrap_or("hook")
                        .to_string();
                    out.push(CliEvent::Hook {
                        name,
                        completed: sub == "hook_response",
                    });
                }
                other => out.push(CliEvent::Unrecognized {
                    raw: v.to_string().chars().take(160).collect(),
                    heuristic: Some(format!("system:{other}")),
                }),
            }
        }
        "rate_limit_event" => out.push(CliEvent::Unrecognized {
            raw: v.to_string().chars().take(160).collect(),
            heuristic: Some("rate_limit_event".into()),
        }),
        "assistant" => push_assistant(out, v),
        "result" => {
            let is_err = v.get("is_error").and_then(Value::as_bool).unwrap_or(false);
            if !is_err {
                out.push(CliEvent::MessageEnd {
                    finish_reason: crate::ir::FinishReason::Stop,
                });
            } else {
                let api_status = v.get("api_error_status").and_then(Value::as_i64);
                let msg = v
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error")
                    .to_string();
                let kind = classify_error(api_status, &msg);
                let recoverable = matches!(kind, ErrorKind::Network | ErrorKind::RateLimit);
                out.push(CliEvent::Error {
                    kind: kind.clone(),
                    message: msg,
                    recoverable,
                });
                out.push(CliEvent::MessageEnd {
                    finish_reason: crate::ir::FinishReason::Error(kind),
                });
            }
        }
        other => out.push(CliEvent::Unrecognized {
            raw: v.to_string().chars().take(160).collect(),
            heuristic: Some(format!("type:{other}")),
        }),
    }
}

impl CliParser for ClaudeParser {
    fn cli_id(&self) -> &'static str {
        "claude"
    }
    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        let mut events = Vec::new();
        for line in parse_lines(input.raw_output) {
            match line {
                JsonLine::Parsed(v) => map_event(&mut events, &v),
                JsonLine::Bad(s) => events.push(CliEvent::Unrecognized {
                    raw: s.chars().take(160).collect(),
                    heuristic: Some("non-json line (truncated/corrupt)".into()),
                }),
            }
        }
        ParseResult { events }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::FinishReason;

    fn parse(s: &str) -> Vec<CliEvent> {
        ClaudeParser
            .parse(&ParseInput {
                raw_output: s,
                exit_code: None,
            })
            .events
    }

    #[test]
    fn no_panic_on_empty_and_garbage() {
        assert!(parse("").is_empty());
        let g = parse("not json at all\n{also bad");
        assert_eq!(g.len(), 2);
        assert!(g.iter().all(CliEvent::is_unrecognized));
    }

    #[test]
    fn happy_path_emits_assistant_and_stop_end() {
        let s = concat!(
            r#"{"type":"system","subtype":"init","session_id":"S1"}"#,
            "\n",
            r#"{"type":"system","subtype":"hook_started","hook_name":"SessionStart"}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"答案"}]}}"#,
            "\n",
            r#"{"type":"result","subtype":"success","is_error":false,"result":"答案"}"#,
            "\n"
        );
        let e = parse(s);
        assert!(matches!(e[0], CliEvent::SessionMeta { .. }));
        assert!(matches!(
            e[1],
            CliEvent::Hook {
                completed: false,
                ..
            }
        ));
        assert!(matches!(
            e[2],
            CliEvent::MessageStart {
                role: Role::Assistant
            }
        ));
        assert!(matches!(&e[3], CliEvent::MessageDelta { content, .. } if content == "答案"));
        assert!(matches!(
            e[4],
            CliEvent::MessageEnd {
                finish_reason: FinishReason::Stop
            }
        ));
    }

    #[test]
    fn auth_fail_maps_401_to_auth_unrecoverable() {
        let s = concat!(
            r#"{"type":"system","subtype":"hook_response","hook_name":"Stop"}"#,
            "\n",
            r#"{"type":"result","subtype":"success","is_error":true,"api_error_status":401,"result":"Invalid API key · Fix external API key"}"#,
            "\n"
        );
        let e = parse(s);
        assert!(matches!(
            e.iter().find(|x| matches!(x, CliEvent::Error { .. })),
            Some(CliEvent::Error {
                kind: ErrorKind::Auth,
                recoverable: false,
                ..
            })
        ));
        assert!(matches!(
            e.last(),
            Some(CliEvent::MessageEnd {
                finish_reason: FinishReason::Error(ErrorKind::Auth)
            })
        ));
    }

    #[test]
    fn network_error_maps_connectionrefused_to_network_recoverable() {
        let s = concat!(
            r#"{"type":"system","subtype":"api_retry","attempt":1}"#,
            "\n",
            r#"{"type":"result","subtype":"success","is_error":true,"api_error_status":null,"result":"API Error: Unable to connect to API (ConnectionRefused)"}"#,
            "\n"
        );
        let e = parse(s);
        // api_retry → Unrecognized（诊断 · 非 Error）
        assert!(matches!(e[0], CliEvent::Unrecognized { .. }));
        assert!(e.iter().any(|x| matches!(
            x,
            CliEvent::Error {
                kind: ErrorKind::Network,
                recoverable: true,
                ..
            }
        )));
        assert!(matches!(
            e.last(),
            Some(CliEvent::MessageEnd {
                finish_reason: FinishReason::Error(ErrorKind::Network)
            })
        ));
    }

    #[test]
    fn empty_assistant_does_not_emit_dangling_start() {
        let s = r#"{"type":"assistant","message":{"content":[]}}"#;
        assert!(parse(s).is_empty());
    }
}
