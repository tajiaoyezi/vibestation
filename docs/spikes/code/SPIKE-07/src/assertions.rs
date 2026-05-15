//! §F 测试矩阵断言（Phase C · 主 agent · decision-grade 证据逻辑）。
//!
//! 纯函数：输入 `(scenario, events, raw_output)` → 每条 §F 断言 pass/fail。
//! 断言定义**严格对齐 spec §F**（通用 3 条 + 场景特化）· 不向任何 §H 路径凑数。
//!
//! 设计原则：
//! - 矩阵数字是 R1 降级的 decision-grade 输入 · 断言逻辑必须 test-prove
//!   （buggy 断言 = fabricated-equivalent 数字）
//! - "正确率" 定义：单样本所有适用断言全过 → 该样本 PASS；
//!   场景正确率 = PASS 样本数 / 该场景样本数（§F 矩阵每场景 3 样本）
//! - U1（不 panic）由 harness `catch_unwind` 兜底 · 本模块不覆盖

use crate::ir::{CliEvent, ErrorKind, FinishReason, Role};

/// 单条 §F 断言结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct Check {
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

impl Check {
    fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Check {
            name,
            passed: true,
            detail: detail.into(),
        }
    }
    fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Check {
            name,
            passed: false,
            detail: detail.into(),
        }
    }
}

/// 单样本评估：所有适用断言 + 总判定。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SampleAssessment {
    pub checks: Vec<Check>,
    pub all_passed: bool,
}

impl SampleAssessment {
    fn from(checks: Vec<Check>) -> Self {
        let all_passed = checks.iter().all(|c| c.passed);
        SampleAssessment { checks, all_passed }
    }
}

/// 保守 ANSI 剥离（仅用于 long_stream 的 95% 长度分母估算 · spec §F 允许"少量长度差异"）。
/// 非生产 strip（各 adapter 有自己的精确实现）· 这里只需稳定分母。
fn strip_ansi_estimate(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            if c != '\r' {
                out.push(c);
            }
            continue;
        }
        match chars.next() {
            Some('[') => {
                for c2 in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c2) {
                        break;
                    }
                }
            }
            Some(']') => {
                for c2 in chars.by_ref() {
                    if c2 == '\u{7}' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// 单调性（§F 通用 #3）：消息事件必须 start → delta → end 有序。
/// 判定：扫描序列 · 任一 `MessageDelta`/`MessageEnd` 前必须已出现过未闭合的
/// `MessageStart`；`MessageEnd` 闭合当前消息。Codex 多 user/assistant 段
/// （多 start/end 对）合法 · 只要每段不交叉。
pub fn is_monotone(events: &[CliEvent]) -> bool {
    let mut in_message = false;
    for e in events {
        match e {
            // 允许连续 start（上一段未显式 end · 视为隐式闭合）· 不算违规
            CliEvent::MessageStart { .. } => in_message = true,
            CliEvent::MessageDelta { .. } if !in_message => return false, // delta 早于 start
            CliEvent::MessageDelta { .. } => {}
            // 错误终止 MessageEnd（与 Error 事件配对 · §F auth/network「出现在
            // message_start 之前或替代 message_start」）：合法无需前置 start ·
            // 同时闭合任何打开的 message。不修此分支则 12 条 auth/network 样本
            // 全假 FAIL · 把 §H verdict 错误推向 deferred（decision-grade 污染）。
            CliEvent::MessageEnd {
                finish_reason: FinishReason::Error(_),
            } => in_message = false,
            // 正常终止（Stop/Length/ToolCalls/Unknown）必须有前置 start
            CliEvent::MessageEnd { .. } if !in_message => return false,
            CliEvent::MessageEnd { .. } => in_message = false,
            _ => {}
        }
    }
    true
}

fn has_error_kind(events: &[CliEvent], target: &ErrorKind) -> bool {
    events
        .iter()
        .any(|e| matches!(e, CliEvent::Error { kind, .. } if kind == target))
}

fn total_delta_len(events: &[CliEvent]) -> usize {
    events
        .iter()
        .filter_map(|e| match e {
            CliEvent::MessageDelta { content, .. } => Some(content.chars().count()),
            _ => None,
        })
        .sum()
}

/// 通用断言（§F 通用 · U2 非空 + U3 单调）。U1 panic 由 harness 兜底。
fn check_universal(events: &[CliEvent]) -> Vec<Check> {
    let mut v = Vec::new();
    v.push(if events.is_empty() {
        Check::fail("U2_nonempty", "事件序列为空（空输入除外 · 需人工审计）")
    } else {
        Check::ok("U2_nonempty", format!("{} 事件", events.len()))
    });
    v.push(if is_monotone(events) {
        Check::ok("U3_monotone", "start→delta→end 有序")
    } else {
        Check::fail("U3_monotone", "delta/end 早于 start（时序违规）")
    });
    v
}

/// §F 场景特化断言。`raw_output` 用于 long_stream 95% 长度分母。
pub fn assess(scenario: &str, events: &[CliEvent], raw_output: &str) -> SampleAssessment {
    let mut checks = check_universal(events);

    match scenario {
        "happy_path" => {
            let has_assistant = events.iter().any(|e| {
                matches!(
                    e,
                    CliEvent::MessageStart {
                        role: Role::Assistant
                    }
                )
            });
            let has_error = events.iter().any(|e| matches!(e, CliEvent::Error { .. }));
            let has_end = events
                .iter()
                .any(|e| matches!(e, CliEvent::MessageEnd { .. }));
            checks.push(if has_assistant {
                Check::ok("happy_role_assistant", "MessageStart{Assistant} 存在")
            } else {
                Check::fail("happy_role_assistant", "无 assistant role 识别")
            });
            checks.push(if !has_error {
                Check::ok("happy_no_error", "无 Error 事件混入")
            } else {
                Check::fail("happy_no_error", "happy path 混入 Error 事件")
            });
            checks.push(if has_end {
                Check::ok("happy_has_end", "MessageEnd 存在")
            } else {
                Check::fail("happy_has_end", "无 MessageEnd（消息未闭合）")
            });
        }
        "interrupt_residual" => {
            // §F：最后一个事件不是悬空 message_start（必须 end 或 unrecognized 收尾）
            let ok = match events.last() {
                None => false,
                Some(CliEvent::MessageStart { .. }) => false,
                Some(_) => true,
            };
            checks.push(if ok {
                Check::ok(
                    "interrupt_no_dangling_start",
                    "末事件非悬空 MessageStart（end/unrecognized 收尾）",
                )
            } else {
                Check::fail(
                    "interrupt_no_dangling_start",
                    "末事件为悬空 MessageStart（无收尾）",
                )
            });
        }
        "auth_fail" => {
            let auth = events.iter().any(|e| {
                matches!(
                    e,
                    CliEvent::Error {
                        kind: ErrorKind::Auth,
                        recoverable: false,
                        ..
                    }
                )
            });
            let mislabel_net = has_error_kind(events, &ErrorKind::Network);
            checks.push(if auth {
                Check::ok("auth_kind_auth", "Error{Auth, recoverable:false} 存在")
            } else {
                Check::fail("auth_kind_auth", "无 Error{Auth, recoverable:false}")
            });
            checks.push(if !mislabel_net {
                Check::ok("auth_not_network", "未误标 Network")
            } else {
                Check::fail("auth_not_network", "误标为 Network")
            });
        }
        "network_error" => {
            let net = events.iter().any(|e| {
                matches!(
                    e,
                    CliEvent::Error {
                        kind: ErrorKind::Network,
                        recoverable: true,
                        ..
                    }
                )
            });
            let mislabel_timeout = has_error_kind(events, &ErrorKind::Timeout);
            checks.push(if net {
                Check::ok("net_kind_network", "Error{Network, recoverable:true} 存在")
            } else {
                Check::fail("net_kind_network", "无 Error{Network, recoverable:true}")
            });
            checks.push(if !mislabel_timeout {
                Check::ok("net_not_timeout", "未误标 Timeout")
            } else {
                Check::fail("net_not_timeout", "误标为 Timeout")
            });
        }
        "long_stream" => {
            // §F：总 content 长度 ≥ ANSI 剥离后原始输出 95%
            let denom = strip_ansi_estimate(raw_output).chars().count();
            let got = total_delta_len(events);
            let ratio = if denom == 0 {
                0.0
            } else {
                got as f64 / denom as f64
            };
            checks.push(if ratio >= 0.95 {
                Check::ok(
                    "long_content_95pct",
                    format!("content {got}/{denom} = {:.0}% ≥ 95%", ratio * 100.0),
                )
            } else {
                Check::fail(
                    "long_content_95pct",
                    format!("content {got}/{denom} = {:.0}% < 95%", ratio * 100.0),
                )
            });
        }
        "mixed_ansi_json" => {
            // §F：提取的内容可解析为 serde_json::Value 且关键字段存在
            let json_ok = events.iter().any(|e| match e {
                CliEvent::MessageDelta { content, .. } => content
                    .lines()
                    .map(str::trim)
                    .filter(|l| l.starts_with('{') || l.starts_with('['))
                    .any(|l| serde_json::from_str::<serde_json::Value>(l).is_ok()),
                _ => false,
            });
            checks.push(if json_ok {
                Check::ok("mixed_json_parseable", "提取出可解析 JSON")
            } else {
                Check::fail(
                    "mixed_json_parseable",
                    "无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）",
                )
            });
        }
        other => {
            checks.push(Check::fail(
                "unknown_scenario",
                format!("未知场景: {other}"),
            ));
        }
    }

    SampleAssessment::from(checks)
}

// ── §E.11 对比基线（证明 parser 价值 · 非 §F 断言）──────────────────

/// 基线 A "不做 parser"：纯关键字扫描判 "有错/无错"。
/// 返回 true = 判定为错误样本。
pub fn baseline_keyword_is_error(raw_output: &str, exit_code: Option<i64>) -> bool {
    let low = raw_output.to_ascii_lowercase();
    exit_code.is_some_and(|c| c != 0)
        || low.contains("error")
        || low.contains("unauthorized")
        || low.contains("failed")
}

/// 基线 B "简单启发式"：按行首 `Error:` / `WARNING:` 判错。
pub fn baseline_lineprefix_is_error(raw_output: &str) -> bool {
    raw_output
        .lines()
        .map(str::trim_start)
        .any(|l| l.starts_with("Error:") || l.starts_with("ERROR:") || l.starts_with("WARNING:"))
}

/// 场景真值：该场景是否"错误类"（auth/network = 错 · happy/long/mixed = 非错 ·
/// interrupt = 非错严格判 · 用于基线对比 confusion）。
pub fn scenario_is_error(scenario: &str) -> bool {
    matches!(scenario, "auth_fail" | "network_error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CliEvent, ErrorKind, FinishReason, Role};

    fn start(r: Role) -> CliEvent {
        CliEvent::MessageStart { role: r }
    }
    fn delta(s: &str) -> CliEvent {
        CliEvent::MessageDelta {
            content: s.into(),
            raw_ansi: None,
        }
    }
    fn end() -> CliEvent {
        CliEvent::MessageEnd {
            finish_reason: FinishReason::Stop,
        }
    }
    /// 错误终止 end（真实 parser 对 auth/network 发的形态 · 见 claude.rs/codex.rs）。
    fn end_err(k: ErrorKind) -> CliEvent {
        CliEvent::MessageEnd {
            finish_reason: FinishReason::Error(k),
        }
    }
    fn err(k: ErrorKind, recov: bool) -> CliEvent {
        CliEvent::Error {
            kind: k,
            message: "x".into(),
            recoverable: recov,
        }
    }
    fn unrec() -> CliEvent {
        CliEvent::Unrecognized {
            raw: "blob".into(),
            heuristic: None,
        }
    }

    #[test]
    fn monotone_accepts_ordered_sequence() {
        assert!(is_monotone(&[start(Role::Assistant), delta("hi"), end()]));
    }

    #[test]
    fn monotone_accepts_codex_multi_segment() {
        assert!(is_monotone(&[
            start(Role::User),
            delta("q"),
            end(),
            start(Role::Assistant),
            delta("a"),
            end(),
        ]));
    }

    #[test]
    fn monotone_rejects_delta_before_start() {
        assert!(!is_monotone(&[delta("orphan"), start(Role::Assistant)]));
    }

    #[test]
    fn monotone_rejects_end_before_start() {
        assert!(!is_monotone(&[end()]));
    }

    #[test]
    fn happy_path_all_pass_with_clean_assistant_message() {
        let a = assess(
            "happy_path",
            &[start(Role::Assistant), delta("Vibestation 是…"), end()],
            "irrelevant",
        );
        assert!(a.all_passed, "checks: {:?}", a.checks);
    }

    #[test]
    fn happy_path_fails_when_error_mixed_in() {
        let a = assess(
            "happy_path",
            &[start(Role::Assistant), err(ErrorKind::Network, true), end()],
            "x",
        );
        assert!(!a.all_passed);
        assert!(a
            .checks
            .iter()
            .any(|c| c.name == "happy_no_error" && !c.passed));
    }

    #[test]
    fn auth_fail_pass_requires_auth_not_network() {
        let pass = assess(
            "auth_fail",
            &[err(ErrorKind::Auth, false), end_err(ErrorKind::Auth)],
            "x",
        );
        assert!(pass.all_passed, "checks: {:?}", pass.checks);
        let fail = assess("auth_fail", &[err(ErrorKind::Network, true)], "x");
        assert!(!fail.all_passed);
    }

    #[test]
    fn network_error_pass_requires_network_recoverable() {
        let pass = assess(
            "network_error",
            &[err(ErrorKind::Network, true), end_err(ErrorKind::Network)],
            "x",
        );
        assert!(pass.all_passed, "checks: {:?}", pass.checks);
        let fail_kind = assess("network_error", &[err(ErrorKind::Timeout, true)], "x");
        assert!(!fail_kind.all_passed);
    }

    #[test]
    fn interrupt_residual_unrecognized_tail_passes() {
        let a = assess("interrupt_residual", &[unrec()], "x");
        assert!(a.all_passed, "Unrecognized 收尾应通过宽松 interrupt 断言");
    }

    #[test]
    fn interrupt_residual_dangling_start_fails() {
        let a = assess("interrupt_residual", &[start(Role::Assistant)], "x");
        assert!(!a.all_passed, "悬空 start 收尾应 fail");
    }

    #[test]
    fn long_stream_95pct_threshold() {
        // content 完整 → pass
        let raw = "abcdefghij"; // 10 chars no ansi
        let full = assess(
            "long_stream",
            &[start(Role::Assistant), delta("abcdefghij"), end()],
            raw,
        );
        assert!(full.all_passed, "checks: {:?}", full.checks);
        // content 仅 1 字符（Unrecognized 包整段 · delta 为空）→ fail
        let empty = assess("long_stream", &[unrec()], raw);
        assert!(!empty.all_passed);
    }

    #[test]
    fn mixed_ansi_json_requires_parseable_json() {
        let with_json = assess(
            "mixed_ansi_json",
            &[start(Role::Assistant), delta("{\"k\":1}"), end()],
            "x",
        );
        assert!(with_json.all_passed, "checks: {:?}", with_json.checks);
        let plain = assess(
            "mixed_ansi_json",
            &[start(Role::Assistant), delta("just prose"), end()],
            "x",
        );
        assert!(!plain.all_passed, "纯文本无 JSON 应 fail");
    }

    #[test]
    fn baselines_detect_error_keywords() {
        assert!(baseline_keyword_is_error("Invalid API key", Some(1)));
        assert!(!baseline_keyword_is_error(
            "Vibestation 是桌面工作台",
            Some(0)
        ));
        assert!(baseline_lineprefix_is_error("Error: boom"));
        assert!(!baseline_lineprefix_is_error("all good"));
        assert!(scenario_is_error("auth_fail"));
        assert!(!scenario_is_error("happy_path"));
    }
}
