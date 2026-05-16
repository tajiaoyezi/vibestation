//! `.structured.jsonl` 行分隔 JSON loader（SPIKE-07.5 新写 · SPIKE-07 cast.rs 不可复用）。
//!
//! ## 设计依据（decision-grade · 实测根因）
//!
//! recording-summary.md finding #5 更正（2026-05-16）：raw `/tmp/spike075-raw`
//! 实测 36/36 文件 936/936 行**严格一行一合法 JSON · 零多行事件 · 零 EOF 残尾**。
//! 曾误诊"多行续行 · 须流式累积"实为 redact.py v1 转义破坏 artifact · 已 v2 根治。
//! ∴ loader = **逐行 `serde_json::from_str`**（非流式累积）· 简单即正确（KISS）。
//!
//! 容错纪律（同 SPIKE-07）：单行解析失败 → 包 `JsonLine::Bad`（不 panic ·
//! 不吞后续）· 让 parser 决定降级为 `Unrecognized`。raw 实测 0 坏行 ·
//! 此分支是 SIGTERM 真截断尾的防御性兜底（spec §C.1）。

use serde_json::Value;

/// 一物理行的解析结果。
#[derive(Debug, Clone, PartialEq)]
pub enum JsonLine {
    /// 成功解析为 JSON 事件。
    Parsed(Value),
    /// 解析失败（真截断尾 / 损坏）· 保留原文供 parser 包 `Unrecognized`。
    Bad(String),
}

/// 逐行解析 `.structured.jsonl` 文本。空行跳过 · 失败行保留为 `Bad`。
/// **契约**：不 panic · 一坏行不影响后续行 · 顺序保持。
pub fn parse_lines(text: &str) -> Vec<JsonLine> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| match serde_json::from_str::<Value>(l) {
            Ok(v) => JsonLine::Parsed(v),
            Err(_) => JsonLine::Bad(l.to_string()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_no_lines() {
        assert!(parse_lines("").is_empty());
        assert!(parse_lines("\n\n  \n").is_empty());
    }

    #[test]
    fn parses_each_valid_line() {
        let r = parse_lines("{\"type\":\"a\"}\n{\"type\":\"b\"}\n");
        assert_eq!(r.len(), 2);
        assert!(matches!(&r[0], JsonLine::Parsed(v) if v["type"] == "a"));
        assert!(matches!(&r[1], JsonLine::Parsed(v) if v["type"] == "b"));
    }

    #[test]
    fn blank_lines_skipped_not_counted() {
        let r = parse_lines("{\"x\":1}\n\n   \n{\"y\":2}\n");
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn bad_line_isolated_does_not_swallow_following() {
        // 关键容错：坏行后续行仍须解析（防 naive 累积器吞后续 bug）。
        let r = parse_lines("{\"ok\":1}\n{not json\n{\"ok\":2}\n");
        assert_eq!(r.len(), 3);
        assert!(matches!(r[0], JsonLine::Parsed(_)));
        assert!(matches!(r[1], JsonLine::Bad(_)));
        assert!(matches!(&r[2], JsonLine::Parsed(v) if v["ok"] == 2));
    }

    #[test]
    fn trailing_partial_line_becomes_bad_not_panic() {
        // SIGTERM 真截断尾防御性兜底（raw 实测 0 此情形）。
        let r = parse_lines("{\"a\":1}\n{\"trunc\":");
        assert_eq!(r.len(), 2);
        assert!(matches!(r[1], JsonLine::Bad(_)));
    }
}
