//! Phase A 交付物：全 36 样本结构画像（Rust 实测 · 替代 python reconnaissance）。
//!
//! 输出进 `docs/spikes/raw/SPIKE-07/phase-a-survey.txt`（raw 数据 · 进 git）。
//! 喂 Phase D 统一抽象分析：Claude 薄协议 vs Codex 厚结构的定量画像。
//!
//! 复现：`cargo run --bin survey`（CWD = crate root · 自动定位 ../../raw/SPIKE-06）

use spike_07_parser::fixture::{load_corpus, SCENARIOS};
use std::path::Path;

/// 极简 ANSI/OSC 剥离（仅 survey 估算 ansi 占比用 · 非 parser）。
fn strip_ansi(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == 0x1b {
            i += 1;
            if i >= b.len() {
                break;
            }
            match b[i] {
                b'[' => {
                    i += 1;
                    while i < b.len() && !(0x40..=0x7e).contains(&b[i]) {
                        i += 1;
                    }
                    i += 1;
                }
                b']' => {
                    i += 1;
                    while i < b.len() && b[i] != 0x07 && b[i] != 0x1b {
                        i += 1;
                    }
                    i += 1;
                }
                _ => i += 1,
            }
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out.replace('\r', "")
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../raw/SPIKE-06");
    let corpus = match load_corpus(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("载入 corpus 失败: {e}");
            std::process::exit(1);
        }
    };

    println!("# SPIKE-07 Phase A · 全 corpus 结构画像（cargo run --bin survey）");
    println!("# corpus = {}", dir.display());
    println!("# 样本数 = {} （期望 36）\n", corpus.len());
    println!(
        "{:7}{:20}{:>3}{:>9}{:>9}{:>5}{:>5}{:>6}{:>6}  structural markers",
        "cli", "scenario", "tk", "rawB", "cleanB", "#o", "exit", "ansi%", "redF"
    );
    println!("{}", "-".repeat(120));

    for f in &corpus {
        let raw = &f.decoded.output;
        let clean = strip_ansi(raw);
        let ansi_pct = if raw.is_empty() {
            0
        } else {
            (100 - clean.len() * 100 / raw.len().max(1)) as i32
        };
        // 结构标记：用相对结构化的判据（非朴素 substring · Phase D 会细化）
        let has_codex_role = clean
            .lines()
            .any(|l| l.trim() == "codex" || l.trim() == "user");
        let markers = [
            ("session-id", clean.contains("session id:")),
            ("role-line", has_codex_role),
            ("hook", clean.contains("hook:")),
            ("tokens-used", clean.contains("tokens used")),
            ("apikey-err", clean.contains("API key")),
            (
                "exit-nonzero",
                f.decoded.exit_code.map(|c| c != 0).unwrap_or(false),
            ),
        ]
        .iter()
        .filter(|(_, v)| *v)
        .map(|(k, _)| *k)
        .collect::<Vec<_>>()
        .join(",");

        println!(
            "{:7}{:20}{:>3}{:>9}{:>9}{:>5}{:>5}{:>6}{:>6}  {}",
            f.cli,
            f.scenario,
            f.take,
            raw.len(),
            clean.len(),
            f.decoded.o_events,
            f.decoded
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "?".into()),
            ansi_pct,
            f.redaction.redacted_fields.len(),
            markers
        );
    }

    // 矩阵完整性 + 跨 take 方差摘要（Phase D 关键证据：corpus 质量）
    println!("\n## 矩阵完整性 + 跨 take rawB 方差");
    for cli in ["claude", "codex"] {
        for scen in SCENARIOS {
            let m: Vec<_> = corpus
                .iter()
                .filter(|f| f.cli == cli && f.scenario == scen)
                .collect();
            let sizes: Vec<usize> = m.iter().map(|f| f.decoded.output.len()).collect();
            let (mn, mx) = (
                sizes.iter().min().copied().unwrap_or(0),
                sizes.iter().max().copied().unwrap_or(0),
            );
            let spread = if mn > 0 { mx as f64 / mn as f64 } else { 0.0 };
            println!(
                "  {cli:7}/{scen:20} take={} rawB[min={mn} max={mx} spread={spread:.1}x]",
                m.len()
            );
        }
    }
}
