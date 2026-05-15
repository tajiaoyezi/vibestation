//! Claude CLI adapter（**Phase B dispatch track 1 · 独占本文件**）。
//!
//! Phase A 实测画像（见 README §关键结论溯源）：Claude = **薄协议**
//! - `happy_path` 196–238B：纯 assistant 文本 + exit 0 · 无 role marker / 无 session id / 无 hook
//! - `auth_fail` 127B："Invalid API key · Fix external API key" + exit 1
//! - `network_error` / `interrupt_residual` / `long_stream` / `mixed_ansi_json`：
//!   32–300KB 终端 TUI 重绘流（exit 0 · corpus 质量 caveat · 见 Phase D）
//!
//! Phase B 任务：把下面 `ClaudeParser::parse` 从占位实现替换为真实薄协议解析
//! （ANSI 剥离 → 整段 stdout 视作 assistant message · exit≠0 + 文本模式 → Error）。
//! **契约硬约束**：不得改 `crate::ir` 的 enum/trait 签名；不得 panic；不认识的
//! 块包 `Unrecognized`；只编辑本文件（`src/parser/claude.rs`）。

use crate::ir::{CliEvent, CliParser, ParseInput, ParseResult};

pub struct ClaudeParser;

impl CliParser for ClaudeParser {
    fn cli_id(&self) -> &'static str {
        "claude"
    }

    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        // ── Phase A 占位（StubParser 等价）· Phase B 替换为真实薄协议解析 ──
        if input.raw_output.is_empty() {
            return ParseResult::default();
        }
        ParseResult {
            events: vec![CliEvent::Unrecognized {
                raw: input.raw_output.chars().take(120).collect(),
                heuristic: Some("STUB · Phase B Claude adapter 未实现".into()),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_no_panic_contract() {
        let p = ClaudeParser;
        assert_eq!(p.cli_id(), "claude");
        let r = p.parse(&ParseInput {
            raw_output: "Invalid API key · Fix external API key",
            exit_code: Some(1),
        });
        assert_eq!(r.events.len(), 1);
        assert!(!r.events.is_empty());
    }
}
