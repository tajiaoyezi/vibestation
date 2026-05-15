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

use crate::ir::{CliEvent, CliParser, ParseInput, ParseResult};

pub struct CodexParser;

impl CliParser for CodexParser {
    fn cli_id(&self) -> &'static str {
        "codex"
    }

    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        // ── Phase A 占位（StubParser 等价）· Phase B 替换为真实厚结构解析 ──
        if input.raw_output.is_empty() {
            return ParseResult::default();
        }
        ParseResult {
            events: vec![CliEvent::Unrecognized {
                raw: input.raw_output.chars().take(120).collect(),
                heuristic: Some("STUB · Phase B Codex adapter 未实现".into()),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_no_panic_contract() {
        let p = CodexParser;
        assert_eq!(p.cli_id(), "codex");
        let r = p.parse(&ParseInput {
            raw_output: "OpenAI Codex v0.121.0\nsession id: abc\nuser\nhi\ncodex\nhello",
            exit_code: Some(0),
        });
        assert!(!r.events.is_empty());
    }
}
