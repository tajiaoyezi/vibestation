//! SPIKE-07 CLI 输出协议 parser 验证原型。
//!
//! Phase A 地基（本次）：cast v3 解码 + fixture loader + CliEvent IR 契约 +
//! CliParser trait（Phase B 两路 adapter 接口）+ StubParser（端到端跑通占位）。
//!
//! Phase B（dispatch 2 路）：`parser::claude` + `parser::codex` 实现 `CliParser`。
//! Phase C/D/E/F（主 agent + Arbiter）：断言 / 准确率 / §H 三路径 / ADR-017。
//!
//! 归档级原型 · 不重写为生产 parser（SPIKE-07 spec §C Don't.5）。

pub mod cast;
pub mod fixture;
pub mod ir;

/// Phase B adapter 落地位置（占位 module · Phase B agent 各填一个）。
pub mod parser {
    use crate::ir::{CliParser, StubParser};

    /// 按 cli 标识路由到对应 adapter。
    /// Phase A：claude/codex 均返回 StubParser（端到端跑通）。
    /// Phase B：替换为 `claude::ClaudeParser` / `codex::CodexParser`。
    pub fn for_cli(cli: &str) -> Box<dyn CliParser> {
        match cli {
            "claude" => Box::new(StubParser { cli: "claude" }),
            "codex" => Box::new(StubParser { cli: "codex" }),
            other => Box::new(StubParser {
                cli: Box::leak(other.to_string().into_boxed_str()),
            }),
        }
    }
}
