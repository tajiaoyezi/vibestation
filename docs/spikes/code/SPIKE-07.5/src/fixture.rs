//! 结构化 corpus 发现 + parser 无关 ground truth 抽取（SPIKE-07.5 新写）。
//!
//! 文件名约定：`<cli>_<scenario>_<take>.structured.jsonl`
//!  - cli ∈ {claude, codex}（首段 · 第一个 `_` 前）
//!  - take ∈ {1,2,3}（末段 · 最后一个 `_` 后）
//!  - scenario = 中间（含下划线 · 如 `happy_path` / `long_stream`）
//!
//! `reference_text`：**不经 parser** 直接从协议事件抽 assistant 可见文本 ·
//! 作 §F long_stream 95% 的分母（parser MessageDelta 总长 vs 协议真值）·
//! 避免"拿 parser 输出当自己分母"的循环论证（decision-grade 纪律）。

use crate::jsonl::{parse_lines, JsonLine};
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;

/// §F 六场景（与 assertions.rs scenario 串一致 · matrix 按此遍历）。
pub const SCENARIOS: [&str; 6] = [
    "happy_path",
    "long_stream",
    "mixed_ansi_json",
    "network_error",
    "auth_fail",
    "interrupt_residual",
];

#[derive(Debug, Clone)]
pub struct Sample {
    pub cli: String,
    pub scenario: String,
    pub take: u32,
    /// 整份 `.structured.jsonl` 原文（传给 parser）。
    pub raw_text: String,
    /// 协议真值 assistant 文本（parser 无关 · long_stream 分母）。
    pub reference_text: String,
    /// 协议派生 exit 语义（parser 无关 · §E.11 基线对比用）。
    pub exit_code: Option<i64>,
}

/// 从文件名解析 `(cli, scenario, take)`。非法名返回 `None`。
pub fn parse_name(stem: &str) -> Option<(String, String, u32)> {
    let s = stem.strip_suffix(".structured.jsonl").unwrap_or(stem);
    let first = s.find('_')?;
    let last = s.rfind('_')?;
    if last <= first {
        return None;
    }
    let cli = &s[..first];
    let scenario = &s[first + 1..last];
    let take: u32 = s[last + 1..].parse().ok()?;
    if cli.is_empty() || scenario.is_empty() {
        return None;
    }
    Some((cli.to_string(), scenario.to_string(), take))
}

/// parser 无关地抽 assistant 可见文本（claude assistant.content[].text /
/// codex item.completed agent_message .text）· degenerate 时 fallback result.result。
pub fn reference_text(cli: &str, events: &[Value]) -> String {
    let mut out = String::new();
    match cli {
        "claude" => {
            for e in events {
                if e.get("type").and_then(Value::as_str) == Some("assistant") {
                    if let Some(content) = e
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(Value::as_array)
                    {
                        for c in content {
                            if c.get("type").and_then(Value::as_str) == Some("text") {
                                if let Some(t) = c.get("text").and_then(Value::as_str) {
                                    out.push_str(t);
                                }
                            }
                        }
                    }
                }
            }
            if out.is_empty() {
                // degenerate（无 assistant 文本）· fallback 非错 result.result
                for e in events {
                    if e.get("type").and_then(Value::as_str) == Some("result")
                        && e.get("is_error").and_then(Value::as_bool) == Some(false)
                    {
                        if let Some(r) = e.get("result").and_then(Value::as_str) {
                            out.push_str(r);
                        }
                    }
                }
            }
        }
        "codex" => {
            for e in events {
                if e.get("type").and_then(Value::as_str) == Some("item.completed") {
                    if let Some(item) = e.get("item") {
                        if item.get("type").and_then(Value::as_str) == Some("agent_message") {
                            if let Some(t) = item.get("text").and_then(Value::as_str) {
                                out.push_str(t);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    out
}

/// 协议派生 exit 语义（parser 无关 · §E.11 基线）：
/// claude → 末个 result.is_error=true ⇒ Some(1) 否则 Some(0)；
/// codex → 出现任一 `error` 事件 ⇒ Some(1) 否则 Some(0)。
pub fn derive_exit(cli: &str, events: &[Value]) -> Option<i64> {
    match cli {
        "claude" => {
            let mut code = Some(0);
            for e in events {
                if e.get("type").and_then(Value::as_str) == Some("result") {
                    code = Some(
                        if e.get("is_error").and_then(Value::as_bool) == Some(true) {
                            1
                        } else {
                            0
                        },
                    );
                }
            }
            code
        }
        "codex" => {
            let has_err = events
                .iter()
                .any(|e| e.get("type").and_then(Value::as_str) == Some("error"));
            Some(if has_err { 1 } else { 0 })
        }
        _ => None,
    }
}

fn values_of(text: &str) -> Vec<Value> {
    parse_lines(text)
        .into_iter()
        .filter_map(|l| match l {
            JsonLine::Parsed(v) => Some(v),
            JsonLine::Bad(_) => None,
        })
        .collect()
}

/// 载入 corpus 目录下全部 `*.structured.jsonl`（按文件名排序 · 确定性）。
pub fn load_corpus(dir: &Path) -> io::Result<Vec<Sample>> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".structured.jsonl"))
        })
        .collect();
    entries.sort();

    let mut out = Vec::with_capacity(entries.len());
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let Some((cli, scenario, take)) = parse_name(name) else {
            continue;
        };
        let raw_text = fs::read_to_string(&p)?;
        let events = values_of(&raw_text);
        let reference_text = reference_text(&cli, &events);
        let exit_code = derive_exit(&cli, &events);
        out.push(Sample {
            cli,
            scenario,
            take,
            raw_text,
            reference_text,
            exit_code,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_name_handles_underscore_scenarios() {
        assert_eq!(
            parse_name("claude_happy_path_1.structured.jsonl"),
            Some(("claude".into(), "happy_path".into(), 1))
        );
        assert_eq!(
            parse_name("codex_mixed_ansi_json_3.structured.jsonl"),
            Some(("codex".into(), "mixed_ansi_json".into(), 3))
        );
        assert_eq!(
            parse_name("claude_interrupt_residual_2.structured.jsonl"),
            Some(("claude".into(), "interrupt_residual".into(), 2))
        );
    }

    #[test]
    fn parse_name_rejects_malformed() {
        assert_eq!(parse_name("noseparators"), None);
        assert_eq!(parse_name("claude_only"), None); // last==first
        assert_eq!(parse_name("claude_happy_x.structured.jsonl"), None); // take 非数字
    }

    #[test]
    fn reference_text_claude_concats_assistant_text() {
        let evs = vec![
            json!({"type":"system","subtype":"init"}),
            json!({"type":"assistant","message":{"content":[
                {"type":"text","text":"Hello "},{"type":"text","text":"world"}]}}),
            json!({"type":"result","subtype":"success","is_error":false,"result":"Hello world"}),
        ];
        assert_eq!(reference_text("claude", &evs), "Hello world");
    }

    #[test]
    fn reference_text_claude_fallback_to_result_when_no_assistant() {
        let evs = vec![
            json!({"type":"result","subtype":"success","is_error":false,"result":"only here"}),
        ];
        assert_eq!(reference_text("claude", &evs), "only here");
    }

    #[test]
    fn reference_text_codex_concats_agent_message() {
        let evs = vec![
            json!({"type":"thread.started","thread_id":"x"}),
            json!({"type":"item.completed","item":{"type":"agent_message","text":"answer A"}}),
            json!({"type":"item.completed","item":{"type":"command_execution","command":"ls"}}),
        ];
        assert_eq!(reference_text("codex", &evs), "answer A");
    }

    #[test]
    fn derive_exit_claude_uses_last_result_is_error() {
        let ok = vec![json!({"type":"result","is_error":false})];
        assert_eq!(derive_exit("claude", &ok), Some(0));
        let err = vec![json!({"type":"result","is_error":true,"api_error_status":401})];
        assert_eq!(derive_exit("claude", &err), Some(1));
    }

    #[test]
    fn derive_exit_codex_uses_error_event_presence() {
        let ok = vec![json!({"type":"turn.completed"})];
        assert_eq!(derive_exit("codex", &ok), Some(0));
        let err = vec![json!({"type":"error","message":"tls eof"})];
        assert_eq!(derive_exit("codex", &err), Some(1));
    }
}
