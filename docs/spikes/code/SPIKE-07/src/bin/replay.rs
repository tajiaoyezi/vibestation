//! Phase C harness 骨架：load corpus → route to CLI parser → 跑 parse → 汇总。
//!
//! Phase A：用 `parser::for_cli`（StubParser）端到端跑通 36 样本 · 证明
//! cast 解码 + fixture loader + IR + parser trait 管道连贯（无 panic）。
//! Phase B：`parser::for_cli` 换真 adapter 后本 harness 不改即出真实事件。
//! Phase C：在此基础上加 §F 测试矩阵逐条断言（主 agent 收 · 不在 Phase A）。
//!
//! 复现：`cargo run --bin replay`

use spike_07_parser::fixture::load_corpus;
use spike_07_parser::ir::ParseInput;
use spike_07_parser::parser::for_cli;
use std::collections::BTreeMap;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../raw/SPIKE-06");
    let corpus = match load_corpus(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("载入失败: {e}");
            std::process::exit(1);
        }
    };

    println!("# SPIKE-07 replay harness · {} 样本", corpus.len());
    println!("# Phase A = StubParser（全 Unrecognized · 仅证明管道连贯）\n");

    let mut total_events = 0usize;
    let mut total_unrec = 0usize;
    let mut panics = 0usize; // catch_unwind 兜底计数（契约要求 parser 不 panic）

    for f in &corpus {
        let parser = for_cli(&f.cli);
        let input = ParseInput {
            raw_output: &f.decoded.output,
            exit_code: f.decoded.exit_code,
        };
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse(&input)));
        let res = match result {
            Ok(r) => r,
            Err(_) => {
                panics += 1;
                eprintln!(
                    "⚠ PANIC · {}/{}/{}（违反 CliParser 不得 panic 契约）",
                    f.cli, f.scenario, f.take
                );
                continue;
            }
        };
        let mut hist: BTreeMap<&str, usize> = BTreeMap::new();
        for e in &res.events {
            *hist.entry(e.kind_tag()).or_insert(0) += 1;
        }
        total_events += res.events.len();
        total_unrec += res.events.iter().filter(|e| e.is_unrecognized()).count();
        println!(
            "{:7}{:20}{:>2}  events={:>3} unrec={:.0}%  {:?}",
            f.cli,
            f.scenario,
            f.take,
            res.events.len(),
            res.unrecognized_ratio() * 100.0,
            hist
        );
    }

    println!(
        "\n## 汇总  events={total_events}  unrecognized={total_unrec} ({:.0}%)  panics={panics}",
        if total_events > 0 {
            total_unrec as f64 / total_events as f64 * 100.0
        } else {
            0.0
        }
    );
    if panics > 0 {
        eprintln!("FAIL · {panics} 样本触发 panic · 违反 §G fail signal #3");
        std::process::exit(2);
    }
    println!("✓ 管道连贯 · 0 panic（Phase A 验收点）");
}
