//! CliEvent 统一中间表示（IR）+ CliParser trait 契约。
//!
//! **这是 Phase B 两路并行 adapter 的接口契约**（Claude 薄协议 adapter +
//! Codex 厚结构 adapter）。Phase A 锁定此契约并 test-prove · Phase B 两个 agent
//! 各实现一个 `CliParser`，不得改本文件的 enum / trait 签名（改则两路产物不兼容）。
//!
//! IR 源自 SPIKE-07 spec §D，按 Phase A 全 36 样本实测结构画像微调：
//! - `Unrecognized` 是强制兜底（parser 遇不认识的块**不得 panic / 丢弃**）
//! - `raw_ansi` 保留原始带色块供前端可选渲染
//! - Claude 薄协议（纯文本 + exit code）和 Codex 厚结构（session id / role /
//!   hook / tokens）共享同一 IR · 差异吸收在各自 adapter 内

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ErrorKind {
    Auth,
    Network,
    RateLimit,
    Timeout,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Error(ErrorKind),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum CliEvent {
    /// 会话/消息开始（Codex 有显式 role marker · Claude 隐式整段=assistant）
    MessageStart {
        role: Role,
    },
    /// 消息正文增量（ANSI 剥离后纯文本 · raw_ansi 保留原始）
    MessageDelta {
        content: String,
        raw_ansi: Option<String>,
    },
    MessageEnd {
        finish_reason: FinishReason,
    },
    Error {
        kind: ErrorKind,
        message: String,
        recoverable: bool,
    },
    /// Codex 专有：session id 显式输出（Claude 无 · 是统一抽象的关键差异点）
    SessionMeta {
        key: String,
        value: String,
    },
    /// Codex 专有：hook 生命周期（SessionStart/UserPromptSubmit/Stop ...）
    Hook {
        name: String,
        completed: bool,
    },
    /// token 用量 footer（Codex `tokens used` · Claude 无）
    Usage {
        tokens: Option<u64>,
    },
    ToolUseStart {
        tool_name: String,
        tool_id: String,
    },
    ToolUseEnd {
        tool_id: String,
    },
    /// 强制兜底：无法解析的块 · 必须包装不得丢弃（spec §D 设计原则）
    Unrecognized {
        raw: String,
        heuristic: Option<String>,
    },
}

impl CliEvent {
    pub fn is_unrecognized(&self) -> bool {
        matches!(self, CliEvent::Unrecognized { .. })
    }
    /// 事件类型短名（统计/断言用）。
    pub fn kind_tag(&self) -> &'static str {
        match self {
            CliEvent::MessageStart { .. } => "message_start",
            CliEvent::MessageDelta { .. } => "message_delta",
            CliEvent::MessageEnd { .. } => "message_end",
            CliEvent::Error { .. } => "error",
            CliEvent::SessionMeta { .. } => "session_meta",
            CliEvent::Hook { .. } => "hook",
            CliEvent::Usage { .. } => "usage",
            CliEvent::ToolUseStart { .. } => "tool_use_start",
            CliEvent::ToolUseEnd { .. } => "tool_use_end",
            CliEvent::Unrecognized { .. } => "unrecognized",
        }
    }
}

/// 解析输入：解码后的终端字节流 + 退出码（parser 只看这两个）。
#[derive(Debug, Clone)]
pub struct ParseInput<'a> {
    pub raw_output: &'a str,
    pub exit_code: Option<i64>,
}

/// 解析产物：事件序列 + parser 自报的诊断（不参与 gate · 仅 Phase D 分析）。
#[derive(Debug, Clone, Default)]
pub struct ParseResult {
    pub events: Vec<CliEvent>,
}

impl ParseResult {
    pub fn unrecognized_ratio(&self) -> f64 {
        if self.events.is_empty() {
            return 1.0;
        }
        let u = self.events.iter().filter(|e| e.is_unrecognized()).count();
        u as f64 / self.events.len() as f64
    }
}

/// **Phase B 契约**：每个 CLI 一个 adapter 实现本 trait。
/// Phase A 锁定签名 · Phase B agent 不得修改本 trait。
pub trait CliParser {
    /// CLI 标识（"claude" / "codex"）· fixture 按文件名路由到对应 parser。
    fn cli_id(&self) -> &'static str;
    /// 解析一条样本的终端字节流为 CliEvent 序列。
    /// 契约：**不得 panic**；不认识的块包装为 `Unrecognized`；空输出返回空 vec。
    fn parse(&self, input: &ParseInput<'_>) -> ParseResult;
}

/// Phase A 占位 parser：把整段输出包成单个 `Unrecognized`。
/// 作用：让 fixture harness 在 Phase B adapter 落地前即可端到端跑通
/// （证明 cast 解码 + fixture loader + IR 管道连贯）· Phase B 用真 adapter 替换。
pub struct StubParser {
    pub cli: &'static str,
}

impl CliParser for StubParser {
    fn cli_id(&self) -> &'static str {
        self.cli
    }
    fn parse(&self, input: &ParseInput<'_>) -> ParseResult {
        if input.raw_output.is_empty() {
            return ParseResult::default();
        }
        ParseResult {
            events: vec![CliEvent::Unrecognized {
                raw: input.raw_output.chars().take(120).collect(),
                heuristic: Some("STUB · Phase B adapter 未实现".into()),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrecognized_ratio_empty_is_one() {
        assert_eq!(ParseResult::default().unrecognized_ratio(), 1.0);
    }

    #[test]
    fn stub_parser_wraps_as_unrecognized_no_panic() {
        let p = StubParser { cli: "claude" };
        let r = p.parse(&ParseInput {
            raw_output: "hello",
            exit_code: Some(0),
        });
        assert_eq!(r.events.len(), 1);
        assert!(r.events[0].is_unrecognized());
        assert_eq!(r.unrecognized_ratio(), 1.0);
    }

    #[test]
    fn stub_empty_output_yields_no_events() {
        let p = StubParser { cli: "codex" };
        let r = p.parse(&ParseInput {
            raw_output: "",
            exit_code: Some(0),
        });
        assert!(r.events.is_empty());
    }

    #[test]
    fn kind_tag_stable() {
        assert_eq!(
            CliEvent::Error {
                kind: ErrorKind::Auth,
                message: "x".into(),
                recoverable: false
            }
            .kind_tag(),
            "error"
        );
    }
}
