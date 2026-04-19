use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use spike_03_git_benchmark::{BenchmarkSuite, Engine, Mode, Scenario, ScenarioRun};

fn main() -> Result<()> {
    let args = Args::parse()?;
    let suite: BenchmarkSuite =
        serde_json::from_slice(&fs::read(&args.input).with_context(|| args.input.display().to_string())?)?;
    let report = build_report(&suite, &args)?;
    fs::write(&args.output, report)?;
    println!("{}", args.output.display());
    Ok(())
}

fn build_report(suite: &BenchmarkSuite, args: &Args) -> Result<String> {
    let mut by_key: BTreeMap<(Engine, Scenario, Mode), &ScenarioRun> = BTreeMap::new();
    for run in &suite.runs {
        by_key.insert((run.engine, run.scenario, run.mode), run);
    }

    let git2_rows = format_rows(&by_key, Engine::Git2)?;
    let gix_rows = format_rows(&by_key, Engine::Gix)?;
    let comparisons = format_comparisons(&by_key)?;
    let decision = choose_decision(&by_key)?;
    let checks = format_checks(&by_key)?;
    let cold_note = if suite.purge_command.is_some() {
        "本机支持在每个 cold 样本前执行缓存清理命令。".to_string()
    } else {
        "本机无法在当前权限模型下执行系统级 cache purge（`purge` 需要 sudo，且本会话无无密码授权），因此报告里的 cold 数据是“未显式清缓存的首轮样本”近似值；正式选型以 warm P99 为准。".to_string()
    };

    Ok(format!(
        "# SPIKE-03 · git2 vs gix 读路径 benchmark 报告\n\n\
## 环境\n\
- OS：{}\n\
- CPU：{}\n\
- RAM：{}\n\
- Rust toolchain：{}\n\
- git2 版本：0.20.x\n\
- gix 版本：0.70.x\n\
- linux kernel commit：{} · 总 commits：{}\n\n\
## 测量方法\n\
- 数据集：`{}`\n\
- 采样方式：自定义固定迭代 runner 生成 cold/warm 数据；附带 `criterion` warm bench 原始目录用于交叉复核。\n\
- `log -100/-1000/-10000` cold 样本数：{}（每次样本前执行 `{}`，然后重新 open repository 并执行完整场景）\n\
- `log -100/-1000/-10000` warm 样本数：{}（连续执行，不清缓存；每次样本均重新 open repository）\n\
- `全量 count` cold/warm 样本数：{}/{}（只做上限参考，避免把大部分时间浪费在非决策门槛场景）\n\
- 统计项：P50 / P99 / mean / std；时间单位统一为毫秒。\n\
- 场景说明：`log -100/-1000/-10000` 会遍历 commit、读取 commit object，并提取摘要行；`全量 count` 只做可达 commit 全量遍历计数，用作上限参考。\n\
- cold 数据说明：{}\n\n\
## 数据\n\n\
### git2 0.20\n\n\
| 场景 | cold P50 | cold P99 | warm P50 | warm P99 | mean | std |\n\
|---|---:|---:|---:|---:|---:|---:|\n\
{}\n\n\
### gix 0.70\n\n\
| 场景 | cold P50 | cold P99 | warm P50 | warm P99 | mean | std |\n\
|---|---:|---:|---:|---:|---:|---:|\n\
{}\n\n\
### 对比\n\
{}\n\n\
## 结论\n\n\
选择：{} · {}\n\n\
## 验收清单\n\
{}\n",
        args.os,
        args.cpu,
        args.ram,
        args.rustc,
        args.head,
        args.commit_count,
        suite.repo_path,
        suite.cold_samples,
        suite
            .purge_command
            .as_deref()
            .unwrap_or("N/A"),
        suite.warm_samples,
        suite.count_all_cold_samples,
        suite.count_all_warm_samples,
        cold_note,
        git2_rows,
        gix_rows,
        comparisons,
        decision.0,
        decision.1,
        checks
    ))
}

fn format_rows(
    by_key: &BTreeMap<(Engine, Scenario, Mode), &ScenarioRun>,
    engine: Engine,
) -> Result<String> {
    let mut lines = Vec::new();
    for scenario in Scenario::ALL {
        let cold = stats(by_key.get(&(engine, scenario, Mode::Cold)).context("missing cold run")?)?;
        let warm = stats(by_key.get(&(engine, scenario, Mode::Warm)).context("missing warm run")?)?;
        lines.push(format!(
            "| {} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            scenario.label(),
            cold.p50,
            cold.p99,
            warm.p50,
            warm.p99,
            warm.mean,
            warm.std
        ));
    }
    Ok(lines.join("\n"))
}

fn format_comparisons(by_key: &BTreeMap<(Engine, Scenario, Mode), &ScenarioRun>) -> Result<String> {
    let mut lines = Vec::new();
    for scenario in Scenario::ALL {
        let git2 = stats(by_key.get(&(Engine::Git2, scenario, Mode::Warm)).context("missing git2 warm run")?)?;
        let gix = stats(by_key.get(&(Engine::Gix, scenario, Mode::Warm)).context("missing gix warm run")?)?;
        let faster = if git2.p99 <= gix.p99 { "git2" } else { "gix" };
        let slower = if faster == "git2" { gix.p99 / git2.p99 } else { git2.p99 / gix.p99 };
        lines.push(format!(
            "- `{}`：warm P99 git2 = {:.2} ms，gix = {:.2} ms；{} 更快，约 {:.2}x。",
            scenario.label(),
            git2.p99,
            gix.p99,
            faster,
            slower
        ));
    }
    Ok(lines.join("\n"))
}

fn choose_decision(by_key: &BTreeMap<(Engine, Scenario, Mode), &ScenarioRun>) -> Result<(&'static str, String)> {
    let git2_100 = stats(by_key.get(&(Engine::Git2, Scenario::Log100, Mode::Warm)).context("missing git2 -100")?)?;
    let git2_1000 =
        stats(by_key.get(&(Engine::Git2, Scenario::Log1000, Mode::Warm)).context("missing git2 -1000")?)?;
    let git2_10000 =
        stats(by_key.get(&(Engine::Git2, Scenario::Log10000, Mode::Warm)).context("missing git2 -10000")?)?;
    let gix_100 = stats(by_key.get(&(Engine::Gix, Scenario::Log100, Mode::Warm)).context("missing gix -100")?)?;
    let gix_1000 =
        stats(by_key.get(&(Engine::Gix, Scenario::Log1000, Mode::Warm)).context("missing gix -1000")?)?;
    let gix_10000 =
        stats(by_key.get(&(Engine::Gix, Scenario::Log10000, Mode::Warm)).context("missing gix -10000")?)?;

    let git2_pass = git2_100.p99 < 200.0 && git2_1000.p99 < 1_000.0 && git2_10000.p99 < 5_000.0;
    let gix_pass = gix_100.p99 < 200.0 && gix_1000.p99 < 1_000.0 && gix_10000.p99 < 5_000.0;

    if git2_pass {
        return Ok((
            "(A)",
            format!(
                "git2 在 warm 场景下全部通过门槛（-100 {:.2} ms / -1000 {:.2} ms / -10000 {:.2} ms），MVP 可先保持纯 git2。",
                git2_100.p99, git2_1000.p99, git2_10000.p99
            ),
        ));
    }

    if !git2_pass && gix_pass {
        return Ok((
            "(B)",
            format!(
                "git2 未通过门槛，但 gix 通过（-100 {:.2} ms / -1000 {:.2} ms / -10000 {:.2} ms），值得把读路径切到 gix，写路径保留 git2。",
                gix_100.p99, gix_1000.p99, gix_10000.p99
            ),
        ));
    }

    Ok((
        "(C)",
        format!(
            "git2 与 gix 都未完整通过门槛，需触发 R3 并继续做分页/后台索引策略。当前最慢 warm P99：git2 -10000 {:.2} ms，gix -10000 {:.2} ms。",
            git2_10000.p99, gix_10000.p99
        ),
    ))
}

fn format_checks(by_key: &BTreeMap<(Engine, Scenario, Mode), &ScenarioRun>) -> Result<String> {
    let git2_100 = stats(by_key.get(&(Engine::Git2, Scenario::Log100, Mode::Warm)).context("missing git2 -100")?)?;
    let git2_1000 =
        stats(by_key.get(&(Engine::Git2, Scenario::Log1000, Mode::Warm)).context("missing git2 -1000")?)?;
    let git2_10000 =
        stats(by_key.get(&(Engine::Git2, Scenario::Log10000, Mode::Warm)).context("missing git2 -10000")?)?;
    let gix_100 = stats(by_key.get(&(Engine::Gix, Scenario::Log100, Mode::Warm)).context("missing gix -100")?)?;
    let gix_1000 =
        stats(by_key.get(&(Engine::Gix, Scenario::Log1000, Mode::Warm)).context("missing gix -1000")?)?;
    let gix_10000 =
        stats(by_key.get(&(Engine::Gix, Scenario::Log10000, Mode::Warm)).context("missing gix -10000")?)?;

    let variance_ok = [
        git2_100.std / git2_100.mean.max(1.0),
        git2_1000.std / git2_1000.mean.max(1.0),
        git2_10000.std / git2_10000.mean.max(1.0),
        gix_100.std / gix_100.mean.max(1.0),
        gix_1000.std / gix_1000.mean.max(1.0),
        gix_10000.std / gix_10000.mean.max(1.0),
    ]
    .into_iter()
    .all(|ratio| ratio < 0.5);

    Ok(format!(
        "- [{}] git2 log -100 warm P99 < 200ms\n- [{}] git2 log -1000 warm P99 < 1s\n- [{}] git2 log -10000 warm P99 < 5s\n- [x] gix 同场景数据齐全\n- [{}] 方差 < 50%（benchmark 可复现）",
        check(git2_100.p99 < 200.0),
        check(git2_1000.p99 < 1_000.0),
        check(git2_10000.p99 < 5_000.0),
        check(variance_ok),
    ))
}

fn check(passed: bool) -> &'static str {
    if passed { "x" } else { " " }
}

#[derive(Clone, Copy)]
struct Stats {
    p50: f64,
    p99: f64,
    mean: f64,
    std: f64,
}

fn stats(run: &&ScenarioRun) -> Result<Stats> {
    let mut values: Vec<f64> = run.samples.iter().map(|sample| sample.duration_ms).collect();
    if values.is_empty() {
        bail!("empty sample set");
    }

    values.sort_by(|a, b| a.total_cmp(b));
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;

    Ok(Stats {
        p50: percentile(&values, 0.50),
        p99: percentile(&values, 0.99),
        mean,
        std: variance.sqrt(),
    })
}

fn percentile(sorted: &[f64], ratio: f64) -> f64 {
    let index = ((sorted.len() - 1) as f64 * ratio).ceil() as usize;
    sorted[index]
}

struct Args {
    input: PathBuf,
    output: PathBuf,
    os: String,
    cpu: String,
    ram: String,
    rustc: String,
    head: String,
    commit_count: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut input = None;
        let mut output = None;
        let mut os = None;
        let mut cpu = None;
        let mut ram = None;
        let mut rustc = None;
        let mut head = None;
        let mut commit_count = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--input" => input = args.next().map(PathBuf::from),
                "--output" => output = args.next().map(PathBuf::from),
                "--os" => os = args.next(),
                "--cpu" => cpu = args.next(),
                "--ram" => ram = args.next(),
                "--rustc" => rustc = args.next(),
                "--head" => head = args.next(),
                "--commit-count" => commit_count = args.next(),
                other => bail!("unknown argument: {other}"),
            }
        }

        Ok(Self {
            input: input.context("--input is required")?,
            output: output.context("--output is required")?,
            os: os.context("--os is required")?,
            cpu: cpu.context("--cpu is required")?,
            ram: ram.context("--ram is required")?,
            rustc: rustc.context("--rustc is required")?,
            head: head.context("--head is required")?,
            commit_count: commit_count.context("--commit-count is required")?,
        })
    }
}
