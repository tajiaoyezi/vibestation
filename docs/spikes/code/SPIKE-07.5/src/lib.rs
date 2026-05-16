//! SPIKE-07.5 结构化模式 parser 验证原型（路径 A · R1 重判前置）。
//!
//! 复用 SPIKE-07（字节级一致 · sha256 校验）：
//! - `ir.rs`         CliEvent IR + CliParser trait（锁定契约）
//! - `assertions.rs` §F 断言（format-agnostic · is_monotone error-path 修复已含）
//!
//! 新写（SPIKE-07.5 特有 · 结构化协议非 asciinema TUI）：
//! - `jsonl.rs`        逐行 JSON loader（SPIKE-07 cast.rs 不可复用）
//! - `fixture.rs`      corpus 发现 + parser 无关 ground truth
//! - `parser/claude`   claude stream-json adapter
//! - `parser/codex`    codex exec --json adapter
//! - `bin/matrix`      §F 矩阵 harness（复用 SPIKE-07 聚合逻辑 · corpus 换结构化）
//!
//! 归档级原型 · 用完归档 · 不重写生产 parser（沿用 SPIKE-07 spec §C Don't.5）。

pub mod assertions;
pub mod fixture;
pub mod ir;
pub mod jsonl;

pub mod parser {
    pub mod claude;
    pub mod codex;

    use crate::ir::{CliParser, StubParser};

    /// 按 cli 标识路由到结构化 adapter（Phase 2 锁定）。
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
