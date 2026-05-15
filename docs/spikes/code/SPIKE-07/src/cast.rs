//! asciinema v3 cast 解码器。
//!
//! SPIKE-06 corpus 的每条 `.redacted.cast` 是 asciinema **v3** 格式：
//! - 第 1 行：header JSON `{"version":3,"term":{...},"timestamp":...,...}`
//! - 后续每行：事件数组 `[interval, code, data]`
//!   - `"o"` = 输出（stdout）· 拼接成终端字节流
//!   - `"x"` = 退出 · data 是退出码字符串（如 "0" / "1" / "143"=SIGTERM）
//!   - 其余（`"i"` 输入 / `"r"` resize / `"m"` marker）解析但不进字节流
//!
//! 这是 SPIKE-07 全链路的输入层：parser 只看 `DecodedCast::output`（重组后的
//! 终端字节流）+ `exit_code`，不直接碰 cast 结构。

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CastHeader {
    pub version: u32,
    #[serde(default)]
    pub term: Option<serde_json::Value>,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub command: Option<String>,
}

/// 解码后的一条样本：终端输出字节流 + 退出码 + 事件类型计数。
#[derive(Debug, Clone)]
pub struct DecodedCast {
    pub header: CastHeader,
    /// 所有 `"o"` 事件 data 按出现顺序拼接（含 ANSI / \r\n / unicode · 未剥离）。
    pub output: String,
    /// `"x"` 事件携带的退出码（None = 样本未记录退出 · 视为异常）。
    pub exit_code: Option<i64>,
    /// `"o"` 事件个数（survey 用 · 反映流式分片粒度）。
    pub o_events: usize,
    /// 事件类型直方图（"o"/"x"/"i"/"r"/... → count）。
    pub event_types: Vec<(String, usize)>,
}

#[derive(Debug)]
pub enum CastError {
    Empty,
    BadHeader(String),
}

impl std::fmt::Display for CastError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CastError::Empty => write!(f, "cast 文件为空"),
            CastError::BadHeader(e) => write!(f, "cast header 解析失败: {e}"),
        }
    }
}
impl std::error::Error for CastError {}

/// 解码一段 asciinema v3 cast 文本。
///
/// 容错策略（spike 实测要求）：单行事件解析失败**不 panic**，跳过并计入
/// `parse_errors`（返回值第二项），保证 36 样本全跑完拿到全景数据。
pub fn decode(text: &str) -> Result<(DecodedCast, usize), CastError> {
    let mut lines = text.lines();
    let header_line = lines.next().ok_or(CastError::Empty)?;
    let header: CastHeader =
        serde_json::from_str(header_line).map_err(|e| CastError::BadHeader(e.to_string()))?;

    let mut output = String::new();
    let mut exit_code = None;
    let mut o_events = 0usize;
    let mut parse_errors = 0usize;
    let mut hist: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // 事件 = [interval(number), code(string), data(string)]
        let ev: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };
        let arr = match ev.as_array() {
            Some(a) if a.len() >= 3 => a,
            _ => {
                parse_errors += 1;
                continue;
            }
        };
        let code = arr[1].as_str().unwrap_or("?");
        *hist.entry(code.to_string()).or_insert(0) += 1;
        match code {
            "o" => {
                if let Some(s) = arr[2].as_str() {
                    output.push_str(s);
                    o_events += 1;
                }
            }
            "x" => {
                // 退出码可能是字符串 "0" 或数字 0
                exit_code = arr[2]
                    .as_str()
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .or_else(|| arr[2].as_i64());
            }
            _ => {}
        }
    }

    Ok((
        DecodedCast {
            header,
            output,
            exit_code,
            o_events,
            event_types: hist.into_iter().collect(),
        },
        parse_errors,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_v3_header_and_concats_o_events() {
        let cast = r#"{"version":3,"term":{"cols":120,"rows":40},"timestamp":1776689337}
[15.0, "o", "hello "]
[0.1, "o", "world\r\n"]
[0.0, "x", "0"]"#;
        let (d, errs) = decode(cast).expect("decode ok");
        assert_eq!(d.header.version, 3);
        assert_eq!(d.output, "hello world\r\n");
        assert_eq!(d.exit_code, Some(0));
        assert_eq!(d.o_events, 2);
        assert_eq!(errs, 0);
    }

    #[test]
    fn captures_sigterm_exit_143() {
        let cast = "{\"version\":3}\n[0.0, \"o\", \"x\"]\n[1.0, \"x\", \"143\"]";
        let (d, _) = decode(cast).unwrap();
        assert_eq!(d.exit_code, Some(143));
    }

    #[test]
    fn malformed_event_line_skipped_not_panicked() {
        let cast = "{\"version\":3}\n[broken json\n[0.0, \"o\", \"ok\"]";
        let (d, errs) = decode(cast).unwrap();
        assert_eq!(d.output, "ok");
        assert_eq!(errs, 1);
    }

    #[test]
    fn empty_input_errors_cleanly() {
        assert!(matches!(decode(""), Err(CastError::Empty)));
    }

    #[test]
    fn unicode_escape_unescaped_by_serde() {
        //  = ESC · serde_json 负责反转义
        let cast = "{\"version\":3}\n[0.0, \"o\", \"\\u001b[31mred\\u001b[0m\"]";
        let (d, _) = decode(cast).unwrap();
        assert_eq!(d.output, "\u{1b}[31mred\u{1b}[0m");
    }
}
