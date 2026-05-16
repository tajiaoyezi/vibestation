//! Codex `exec --json` 结构化模式 adapter（SPIKE-07.5 新写）。
//!
//! 路由：顶层 `type`。实测结构（raw `/tmp/spike075-raw` · 已脱敏 corpus）：
//!
//! - `thread.started` → `SessionMeta{thread_id}`
//! - `turn.started` → 无事件（turn 边界）
//! - `item.*` item.type==`command_execution` → `ToolUseStart`/`ToolUseEnd`
//!   （agentic tool-use · SPIKE-07 IR 未显式覆盖此 item · 映射 ToolUse* · Phase D 分析点）
//! - `item.completed` item.type==`agent_message` → `MessageStart{Assistant}`+`MessageDelta`
//! - `turn.completed` → `Usage{output_tokens}` +（若本 turn 有 message）`MessageEnd{Stop}`
//! - `error` → `Error{Network, recoverable:true}`（"Reconnecting… tls handshake
//!   eof" 等瞬态 · 无 MessageEnd · turn 收尾）
//! - 坏行 / 未知 → `Unrecognized`（不 panic · spec §D 兜底契约）
//!
//! ⚠ codex auth/network 退化（recording-summary #3 / spec §E fail#2）：codex 用
//! ChatGPT OAuth backend · 无视 `OPENAI_API_KEY`/`OPENAI_BASE_URL` env · 无法用
//! env 注入构造 codex 错误态 → 6 个 codex auth/network 样本是 corpus 构造退化
//! （非 parser 缺陷）· §H 判定须显式标注（同 SPIKE-07 Phase D 退化纪律）。

use crate::ir::{CliEvent, CliParser, ErrorKind, FinishReason, ParseInput, ParseResult, Role};
use crate::jsonl::{parse_lines, JsonLine};
use serde_json::Value;

pub struct CodexParser;

fn map_event(out: &mut Vec<CliEvent>, msg_open: &mut bool, v: &Value) {
    let ty = v.get("type").and_then(Value::as_str).unwrap_or("");
    match ty {
        "thread.started" => {
            if let Some(tid) = v.get("thread_id").and_then(Value::as_str) {
                out.push(CliEvent::SessionMeta {
                    key: "thread_id".into(),
                    value: tid.to_string(),
                });
            }
        }
        "turn.started" => {}
        "turn.completed" => {
            let tokens = v
                .get("usage")
                .and_then(|u| u.get("output_tokens"))
                .and_then(Value::as_u64);
            out.push(CliEvent::Usage { tokens });
            if *msg_open {
                out.push(CliEvent::MessageEnd {
                    finish_reason: FinishReason::Stop,
                });
                *msg_open = false;
            }
        }
        "item.started" | "item.completed" => {
            let item = v.get("item").cloned().unwrap_or(Value::Null);
            let itype = item.get("type").and_then(Value::as_str).unwrap_or("");
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("item")
                .to_string();
            match (ty, itype) {
                ("item.started", "command_execution") => {
                    out.push(CliEvent::ToolUseStart {
                        tool_name: "command_execution".into(),
                        tool_id: id,
                    });
                }
                ("item.completed", "command_execution") => {
                    out.push(CliEvent::ToolUseEnd { tool_id: id });
                }
                ("item.completed", "agent_message") => {
                    if let Some(t) = item.get("text").and_then(Value::as_str) {
                        if !t.is_empty() {
                            out.push(CliEvent::MessageStart {
                                role: Role::Assistant,
                            });
                            out.push(CliEvent::MessageDelta {
                                content: t.to_string(),
                                raw_ansi: None,
                            });
                            *msg_open = true;
                        }
                    }
                }
                ("item.started", "agent_message") => {} // 等 completed 才出文本
                _ => out.push(CliEvent::Unrecognized {
                    raw: v.to_string().chars().take(160).collect(),
                    heuristic: Some(format!("item:{itype}")),
                }),
            }
        }
        "error" => {
            let m = v
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("codex error")
                .to_string();
            out.push(CliEvent::Error {
                kind: ErrorKind::Network,
                message: m,
                recoverable: true,
            });
        }
        other => out.push(CliEvent::Unrecognized {
            raw: v.to_string().chars().take(160).collect(),
            heuristic: Some(format!("type:{other}")),
        }),
    }
}

impl CliParser for CodexParser {
    fn cli_id(&self) -> &'static str {
        "codex"
    }
    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        let mut events = Vec::new();
        let mut msg_open = false;
        for line in parse_lines(input.raw_output) {
            match line {
                JsonLine::Parsed(v) => map_event(&mut events, &mut msg_open, &v),
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

    fn parse(s: &str) -> Vec<CliEvent> {
        CodexParser
            .parse(&ParseInput {
                raw_output: s,
                exit_code: None,
            })
            .events
    }

    #[test]
    fn no_panic_on_empty_and_garbage() {
        assert!(parse("").is_empty());
        let g = parse("xxx\n{bad");
        assert_eq!(g.len(), 2);
        assert!(g.iter().all(CliEvent::is_unrecognized));
    }

    #[test]
    fn happy_path_thread_message_usage_end() {
        let s = concat!(
            r#"{"type":"thread.started","thread_id":"T1"}"#,
            "\n",
            r#"{"type":"turn.started"}"#,
            "\n",
            r#"{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"答案"}}"#,
            "\n",
            r#"{"type":"turn.completed","usage":{"output_tokens":86}}"#,
            "\n"
        );
        let e = parse(s);
        assert!(matches!(&e[0], CliEvent::SessionMeta { key, .. } if key == "thread_id"));
        assert!(matches!(
            e[1],
            CliEvent::MessageStart {
                role: Role::Assistant
            }
        ));
        assert!(matches!(&e[2], CliEvent::MessageDelta { content, .. } if content == "答案"));
        assert!(matches!(e[3], CliEvent::Usage { tokens: Some(86) }));
        assert!(matches!(
            e[4],
            CliEvent::MessageEnd {
                finish_reason: FinishReason::Stop
            }
        ));
    }

    #[test]
    fn command_execution_maps_to_tooluse_pair() {
        let s = concat!(
            r#"{"type":"thread.started","thread_id":"T"}"#,
            "\n",
            r#"{"type":"item.started","item":{"id":"item_1","type":"command_execution","command":"ls"}}"#,
            "\n",
            r#"{"type":"item.completed","item":{"id":"item_1","type":"command_execution","exit_code":0}}"#,
            "\n"
        );
        let e = parse(s);
        assert!(
            matches!(&e[1], CliEvent::ToolUseStart { tool_name, tool_id } if tool_name == "command_execution" && tool_id == "item_1")
        );
        assert!(matches!(&e[2], CliEvent::ToolUseEnd { tool_id } if tool_id == "item_1"));
    }

    #[test]
    fn error_event_maps_network_recoverable_no_end() {
        let s = concat!(
            r#"{"type":"thread.started","thread_id":"T"}"#,
            "\n",
            r#"{"type":"error","message":"Reconnecting... 2/5 (stream disconnected before completion: tls handshake eof)"}"#,
            "\n"
        );
        let e = parse(s);
        assert!(matches!(
            &e[1],
            CliEvent::Error {
                kind: ErrorKind::Network,
                recoverable: true,
                ..
            }
        ));
        // error 不产 MessageEnd（turn 收尾负责）
        assert!(!e.iter().any(|x| matches!(x, CliEvent::MessageEnd { .. })));
    }

    #[test]
    fn turn_completed_without_message_no_dangling_end() {
        // 只跑命令无 agent_message · turn.completed 不应产 MessageEnd（防非法时序）
        let s = concat!(
            r#"{"type":"thread.started","thread_id":"T"}"#,
            "\n",
            r#"{"type":"item.completed","item":{"id":"i","type":"command_execution"}}"#,
            "\n",
            r#"{"type":"turn.completed","usage":{"output_tokens":3}}"#,
            "\n"
        );
        let e = parse(s);
        assert!(e
            .iter()
            .any(|x| matches!(x, CliEvent::Usage { tokens: Some(3) })));
        assert!(!e.iter().any(|x| matches!(x, CliEvent::MessageEnd { .. })));
    }
}
