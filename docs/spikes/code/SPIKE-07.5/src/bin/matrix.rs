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
    /// 口径："locked-§F"（默认 · assertions.rs byte-identical）或
    /// "recalibrated-carve-out-b"（SPIKE075_RECAL=1 · Arbiter approved）
    scoring_mode: String,
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

/// carve-out (b) 重校准（Arbiter "你直接执行" 2026-05-16 approved · ADR-018）：
/// `mixed_json_parseable` 锁定 §F 用**行首**启发式（行 trim 后 starts_with {/[）·
/// 对模型把 JSON **内联** ANSI 同行 / 包 markdown fence 的合法输出漏抽（#33）。
/// 重校准 = 对 parser 抽出的 `MessageDelta` content 做**子串** JSON 可恢复扫描
/// （任一 `{`/`[` 起始处能解析出一个 JSON value 即算可恢复）· 语义上更贴 §F
/// 本意"内容里 JSON 可恢复"。**统一作用于全部 6 个 mixed 样本**（非特判 #33 ·
/// 自审四问边界适用性）。`assertions.rs` **保持 byte-identical 不改**（§B 完整性）·
/// 仅 `SPIKE075_RECAL=1` 时本函数覆盖 mixed 断言 · 双口径并报。
fn mixed_json_recoverable(events: &[CliEvent]) -> bool {
    events.iter().any(|e| {
        let CliEvent::MessageDelta { content, .. } = e else {
            return false;
        };
        content.char_indices().any(|(i, c)| {
            if c != '{' && c != '[' {
                return false;
            }
            // 从该处起解析一个 JSON value（容忍尾随文本 · StreamDeserializer）
            let mut it =
                serde_json::Deserializer::from_str(&content[i..]).into_iter::<serde_json::Value>();
            matches!(it.next(), Some(Ok(_)))
        })
    })
}

/// 用重校准结果覆盖 mixed_ansi_json 的 `mixed_json_parseable` check · 重算 all_passed。
fn apply_recal_mixed(a: &mut SampleAssessment, events: &[CliEvent]) {
    let recovered = mixed_json_recoverable(events);
    for c in a.checks.iter_mut() {
        if c.name == "mixed_json_parseable" {
            c.passed = recovered;
            c.detail = if recovered {
                "重校准(carve-out b · 子串扫描): content 内 JSON 可恢复".into()
            } else {
                "重校准(carve-out b): content 内仍无可恢复 JSON".into()
            };
        }
    }
    a.all_passed = a.checks.iter().all(|c| c.passed);
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

    // carve-out (b) 重校准口径开关（Arbiter "你直接执行" approved · 默认 off = 锁定 §F）
    let mixed_recal = std::env::var("SPIKE075_RECAL").is_ok();

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
        let mut assessment = assess(&s.scenario, &events, &s.reference_text);
        // carve-out (b) Arbiter-approved 重校准（仅 SPIKE075_RECAL=1 · 仅 mixed）
        if mixed_recal && s.scenario == "mixed_ansi_json" {
            apply_recal_mixed(&mut assessment, &events);
        }
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
        scoring_mode: if mixed_recal {
            "recalibrated-carve-out-b (Arbiter approved 2026-05-16 · assertions.rs 仍 byte-identical)"
                .into()
        } else {
            "locked-§F (assertions.rs byte-identical · §B 完整性)".into()
        },
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
    println!("评分口径：**{}**\n", r.scoring_mode);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn delta(s: &str) -> CliEvent {
        CliEvent::MessageDelta {
            content: s.into(),
            raw_ansi: None,
        }
    }

    #[test]
    fn recal_recovers_inline_json_after_ansi_same_line() {
        // #33 实测形态：JSON 内联在 ANSI 同行 + markdown fence
        let e = vec![delta(
            "```text\n\\033[1m粗体\\033[0m [{\"x\":1},{\"y\":2}]\n```",
        )];
        assert!(mixed_json_recoverable(&e));
    }

    #[test]
    fn recal_recovers_line_start_json_too() {
        // 锁定 §F 能过的（行首 JSON）重校准也必须过（统一更宽 · 不回退）
        let e = vec![delta("preface\n{\"lang\":\"zh\",\"ok\":true}\n")];
        assert!(mixed_json_recoverable(&e));
    }

    #[test]
    fn recal_rejects_pure_prose_no_json() {
        let e = vec![delta("这是一段没有任何 JSON 的纯文本说明。")];
        assert!(!mixed_json_recoverable(&e));
    }

    #[test]
    fn recal_ignores_lone_brace_not_valid_json() {
        // 单个 { 不构成可解析 JSON value · 不得误判可恢复
        let e = vec![delta("函数体 { 缩进 } 不是 JSON")];
        assert!(!mixed_json_recoverable(&e));
    }
}
