//! SPIKE-07 CLI 输出协议 parser 验证原型。
//!
//! Phase A 地基（本次）：cast v3 解码 + fixture loader + CliEvent IR 契约 +
//! CliParser trait（Phase B 两路 adapter 接口）+ StubParser（端到端跑通占位）。
//!
//! Phase B（dispatch 2 路）：`parser::claude` + `parser::codex` 实现 `CliParser`。
//! Phase C/D/E/F（主 agent + Arbiter）：断言 / 准确率 / §H 三路径 / ADR-017。
//!
//! 归档级原型 · 不重写为生产 parser（SPIKE-07 spec §C Don't.5）。

pub mod assertions;
pub mod cast;
pub mod fixture;
pub mod ir;

/// Phase B 两路 adapter（**文件域硬隔离**：track 1 只碰 `parser/claude.rs`，
/// track 2 只碰 `parser/codex.rs`；本 `for_cli` 路由 Phase A 已锁 · Phase B 不改 lib.rs）。
pub mod parser {
    pub mod claude;
    pub mod codex;

    use crate::ir::{CliParser, StubParser};

    /// 按 cli 标识路由到对应 adapter（Phase A 锁定 · Phase B 各 agent 只改自己的
    /// module 文件 · `for_cli` 已指向真实类型 `ClaudeParser`/`CodexParser`，其
    /// `parse` impl 由 Phase B 各 track 填充 · 无需触碰本文件）。
    pub fn for_cli(cli: &str) -> Box<dyn CliParser> {
        match cli {
            "claude" => Box::new(claude::ClaudeParser),
            "codex" => Box::new(codex::CodexParser),
            other => Box::new(StubParser {
                cli: Box::leak(other.to_string().into_boxed_str()),
            }),
        }
    }
}
