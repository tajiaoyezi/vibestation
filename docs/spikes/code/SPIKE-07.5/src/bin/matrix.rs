//! Phase 3 · §F 测试矩阵 harness（SPIKE-07.5 结构化模式 · decision-grade）。
//!
//! 对 36 条 SPIKE-07.5 结构化 corpus 逐条：jsonl 逐行解析 → 路由 adapter →
//! `catch_unwind` parse → `assertions::assess`（**复用 SPIKE-07 §F · 字节级一致**）
//! 按 §F 逐断言判定 → 汇总场景/CLI/整体正确率 + §E.11 基线对比。
//!
//! 关键 decision-grade 纪律：
//! - `assess` 的 `raw_output` 参数传 **`sample.reference_text`**（协议真值 ·
//!   parser 无关）· 非 jsonl 信封 → long_stream 95% 分母有意义且非循环论证
//! - §E.11 基线扫 `reference_text`（人类可见内容）· 非 jsonl（信封必含
//!   "error"/"type" → 基线恒判错 · 不公平）
//! - codex auth/network 6 退化样本（OAuth backend 无视 env · recording-summary
//!   #3 / spec §E fail#2）标 `degenerate` · 报告同时给"含/不含退化"两口径
//!   （同 SPIKE-07 Phase D 退化 corpus 纪律 · §H 显式处理）
//!
//! 复现：`cargo run --bin matrix`（在 `docs/spikes/code/SPIKE-07.5/`）。
//! 设 env `SPIKE075_JSON=<path>` 另写机器可读 JSON（§E.7 无 hardcode 路径）。

use spike_07_5_parser::assertions::{
    assess, baseline_keyword_is_error, baseline_lineprefix_is_error, scenario_is_error,
    SampleAssessment,
};
use spike_07_5_parser::fixture::{load_corpus, SCENARIOS};
use spike_07_5_parser::ir::{CliEvent, ParseInput};
use spike_07_5_parser::parser::for_cli;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(serde::Serialize)]
struct SampleRow {
    cli: String,
    scenario: String,
    take: u32,
    events: usize,
    unrecognized_ratio: f64,
    panicked: bool,
    parser_emits_error: bool,
    scenario_truth_error: bool,
    degenerate: bool,
    assessment: SampleAssessment,
    sample_pass: bool,
}

#[derive(serde::Serialize)]
struct ScenarioStat {
    pass: usize,
    total: usize,
    accuracy: f64,
    mean_unrecognized: f64,
}

#[derive(serde::Serialize)]
struct CliStat {
    pass: usize,
    total: usize,
    accuracy: f64,
}

#[derive(serde::Serialize)]
struct BaselineStat {
    parser_errdetect_accuracy: f64,
    keyword_baseline_accuracy: f64,
    lineprefix_baseline_accuracy: f64,
}

#[derive(serde::Serialize)]
struct MatrixReport {
    total_samples: usize,
    panics: usize,
    overall_pass: usize,
    overall_accuracy: f64,
    nondegenerate_pass: usize,
    nondegenerate_total: usize,
    nondegenerate_accuracy: f64,
    per_scenario: BTreeMap<String, ScenarioStat>,
    per_cli: BTreeMap<String, CliStat>,
    baseline: BaselineStat,
    rows: Vec<SampleRow>,
}

fn is_degenerate(cli: &str, scenario: &str) -> bool {
    // codex 用 ChatGPT OAuth backend · 无视 OPENAI_API_KEY/BASE_URL env
    // → auth/network 错误态无法用 env 注入构造（corpus 构造退化 · 非 parser 缺陷）
    cli == "codex" && (scenario == "auth_fail" || scenario == "network_error")
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../raw/SPIKE-07.5/corpus");
    let corpus = match load_corpus(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("载入 corpus 失败 ({}): {e}", dir.display());
            std::process::exit(1);
        }
    };

    let mut rows: Vec<SampleRow> = Vec::with_capacity(corpus.len());
    let mut panics = 0usize;

    for s in &corpus {
        let parser = for_cli(&s.cli);
        let input = ParseInput {
            raw_output: &s.raw_text,
            exit_code: s.exit_code,
        };
        let parsed =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse(&input)));
        let (events, unrec, panicked) = match parsed {
            Ok(r) => {
                let u = r.unrecognized_ratio();
                (r.events, u, false)
            }
            Err(_) => {
                panics += 1;
                (Vec::new(), 1.0, true)
            }
        };
        // decision-grade：long_stream 95% 分母 = 协议真值 reference_text（parser 无关）
        let assessment = assess(&s.scenario, &events, &s.reference_text);
        let parser_emits_error = events.iter().any(|e| matches!(e, CliEvent::Error { .. }));
        let scenario_truth_error = scenario_is_error(&s.scenario);
        let sample_pass = !panicked && assessment.all_passed;
        rows.push(SampleRow {
            cli: s.cli.clone(),
            scenario: s.scenario.clone(),
            take: s.take,
            events: events.len(),
            unrecognized_ratio: unrec,
            panicked,
            parser_emits_error,
            scenario_truth_error,
            degenerate: is_degenerate(&s.cli, &s.scenario),
            assessment,
            sample_pass,
        });
    }

    let total = rows.len();
    let overall_pass = rows.iter().filter(|r| r.sample_pass).count();
    let nd: Vec<&SampleRow> = rows.iter().filter(|r| !r.degenerate).collect();
    let nd_pass = nd.iter().filter(|r| r.sample_pass).count();

    let mut per_scenario: BTreeMap<String, ScenarioStat> = BTreeMap::new();
    for scen in SCENARIOS {
        let sr: Vec<&SampleRow> = rows.iter().filter(|r| r.scenario == scen).collect();
        let pass = sr.iter().filter(|r| r.sample_pass).count();
        let mean_unrec = if sr.is_empty() {
            0.0
        } else {
            sr.iter().map(|r| r.unrecognized_ratio).sum::<f64>() / sr.len() as f64
        };
        per_scenario.insert(
            scen.to_string(),
            ScenarioStat {
                pass,
                total: sr.len(),
                accuracy: pct(pass, sr.len()),
                mean_unrecognized: mean_unrec,
            },
        );
    }

    let mut per_cli: BTreeMap<String, CliStat> = BTreeMap::new();
    for cli in ["claude", "codex"] {
        let cr: Vec<&SampleRow> = rows.iter().filter(|r| r.cli == cli).collect();
        let pass = cr.iter().filter(|r| r.sample_pass).count();
        per_cli.insert(
            cli.to_string(),
            CliStat {
                pass,
                total: cr.len(),
                accuracy: pct(pass, cr.len()),
            },
        );
    }

    // §E.11 基线：扫 reference_text（人类可见内容）· 非 jsonl 信封（公平性）
    let corpus_inputs: Vec<(&str, Option<i64>, bool, bool)> = rows
        .iter()
        .zip(corpus.iter())
        .map(|(r, f)| {
            (
                f.reference_text.as_str(),
                f.exit_code,
                r.parser_emits_error,
                r.scenario_truth_error,
            )
        })
        .collect();
    let parser_correct = corpus_inputs
        .iter()
        .filter(|(_, _, p, truth)| p == truth)
        .count();
    let kw_correct = corpus_inputs
        .iter()
        .filter(|(out, ec, _, truth)| &baseline_keyword_is_error(out, *ec) == truth)
        .count();
    let lp_correct = corpus_inputs
        .iter()
        .filter(|(out, _, _, truth)| &baseline_lineprefix_is_error(out) == truth)
        .count();
    let baseline = BaselineStat {
        parser_errdetect_accuracy: pct(parser_correct, total),
        keyword_baseline_accuracy: pct(kw_correct, total),
        lineprefix_baseline_accuracy: pct(lp_correct, total),
    };

    let report = MatrixReport {
        total_samples: total,
        panics,
        overall_pass,
        overall_accuracy: pct(overall_pass, total),
        nondegenerate_pass: nd_pass,
        nondegenerate_total: nd.len(),
        nondegenerate_accuracy: pct(nd_pass, nd.len()),
        per_scenario,
        per_cli,
        baseline,
        rows,
    };

    print_markdown(&report);

    if let Ok(path) = std::env::var("SPIKE075_JSON") {
        match serde_json::to_string_pretty(&report) {
            Ok(j) => {
                if let Err(e) = std::fs::write(&path, j) {
                    eprintln!("写 JSON 失败 {path}: {e}");
                } else {
                    eprintln!("JSON 已写: {path}");
                }
            }
            Err(e) => eprintln!("JSON 序列化失败: {e}"),
        }
    }
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 / d as f64 * 100.0
    }
}

fn print_markdown(r: &MatrixReport) {
    println!("# SPIKE-07.5 Phase 3 · §F 测试矩阵实测结果（结构化模式）\n");
    println!(
        "样本总数 **{}** · panic **{}** · 整体 PASS **{}/{}** = **{:.1}%**",
        r.total_samples, r.panics, r.overall_pass, r.total_samples, r.overall_accuracy
    );
    println!(
        "不含 codex auth/network 退化样本：PASS **{}/{}** = **{:.1}%**（§H 主口径）\n",
        r.nondegenerate_pass, r.nondegenerate_total, r.nondegenerate_accuracy
    );

    println!("## 场景级正确率（§F 矩阵 · single-source = SPIKE-07 §H · 本表 informative）\n");
    println!("| 场景 | PASS/total | 正确率 | 平均 Unrecognized |");
    println!("| --- | --- | --- | --- |");
    for (scen, s) in &r.per_scenario {
        println!(
            "| {} | {}/{} | {:.0}% | {:.0}% |",
            scen,
            s.pass,
            s.total,
            s.accuracy,
            s.mean_unrecognized * 100.0
        );
    }

    println!("\n## CLI 级正确率\n");
    println!("| CLI | PASS/total | 正确率 |");
    println!("| --- | --- | --- |");
    for (cli, c) in &r.per_cli {
        println!("| {} | {}/{} | {:.0}% |", cli, c.pass, c.total, c.accuracy);
    }

    println!("\n## §E.11 基线对比（error-detection · parser vs 廉价启发式 · 扫 reference_text）\n");
    println!("| 方法 | 准确率 |");
    println!("| --- | --- |");
    println!(
        "| Parser（结构化 Error 事件） | {:.0}% |",
        r.baseline.parser_errdetect_accuracy
    );
    println!(
        "| 基线 A 关键字扫描 | {:.0}% |",
        r.baseline.keyword_baseline_accuracy
    );
    println!(
        "| 基线 B 行首启发式 | {:.0}% |",
        r.baseline.lineprefix_baseline_accuracy
    );
    let delta = r.baseline.parser_errdetect_accuracy
        - r.baseline
            .keyword_baseline_accuracy
            .max(r.baseline.lineprefix_baseline_accuracy);
    println!(
        "\n> Parser − 最优基线 = **{:+.0}pp**（§E.11：parser 须显著优于基线 +20pp 才值复杂度）",
        delta
    );

    println!("\n## 逐样本矩阵（36 条 · 每条 §F 断言）\n");
    println!("| # | CLI | 场景 | take | events | unrec | panic | 退化 | 断言 | 样本 |");
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for (i, row) in r.rows.iter().enumerate() {
        let passed = row.assessment.checks.iter().filter(|c| c.passed).count();
        let totalc = row.assessment.checks.len();
        println!(
            "| {} | {} | {} | {} | {} | {:.0}% | {} | {} | {}/{} | {} |",
            i + 1,
            row.cli,
            row.scenario,
            row.take,
            row.events,
            row.unrecognized_ratio * 100.0,
            if row.panicked { "YES" } else { "—" },
            if row.degenerate { "⚠退化" } else { "—" },
            passed,
            totalc,
            if row.sample_pass {
                "✅PASS"
            } else {
                "❌FAIL"
            },
        );
    }

    println!("\n## 失败断言明细\n");
    for (i, row) in r.rows.iter().enumerate() {
        if row.sample_pass {
            continue;
        }
        println!(
            "**#{} {}/{}/{}**{}{}",
            i + 1,
            row.cli,
            row.scenario,
            row.take,
            if row.panicked { " · ⚠ PANIC" } else { "" },
            if row.degenerate {
                " · ⚠退化(corpus 构造限制 · 非 parser)"
            } else {
                ""
            }
        );
        for c in row.assessment.checks.iter().filter(|c| !c.passed) {
            println!("- ❌ `{}` — {}", c.name, c.detail);
        }
    }
}
