use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};
use spike_03_git_benchmark::{BenchmarkSuite, Engine, Mode, Scenario, ScenarioRun, measure_once};

fn main() -> Result<()> {
    let args = Args::parse()?;
    fs::create_dir_all(
        args.output
            .parent()
            .context("output path must have a parent directory")?,
    )?;

    let mut runs = Vec::new();

    for engine in Engine::ALL {
        for scenario in Scenario::ALL {
            let cold_samples = if scenario == Scenario::CountAll {
                args.count_all_cold_samples
            } else {
                args.cold_samples
            };
            let warm_samples = if scenario == Scenario::CountAll {
                args.count_all_warm_samples
            } else {
                args.warm_samples
            };

            eprintln!(
                "[measure] engine={} scenario={} cold_samples={} warm_samples={}",
                engine.as_str(),
                scenario.label(),
                cold_samples,
                warm_samples
            );

            runs.push(run_mode(
                &args.repo,
                engine,
                scenario,
                Mode::Cold,
                cold_samples,
                args.purge_command.as_deref(),
            )?);
            runs.push(run_mode(
                &args.repo,
                engine,
                scenario,
                Mode::Warm,
                warm_samples,
                None,
            )?);
        }
    }

    let suite = BenchmarkSuite {
        repo_path: args.repo.display().to_string(),
        cold_samples: args.cold_samples,
        warm_samples: args.warm_samples,
        count_all_cold_samples: args.count_all_cold_samples,
        count_all_warm_samples: args.count_all_warm_samples,
        purge_command: args.purge_command.clone(),
        runs,
    };

    fs::write(&args.output, serde_json::to_vec_pretty(&suite)?)?;
    println!("{}", args.output.display());
    Ok(())
}

fn run_mode(
    repo: &PathBuf,
    engine: Engine,
    scenario: Scenario,
    mode: Mode,
    samples: usize,
    purge_command: Option<&str>,
) -> Result<ScenarioRun> {
    let mut collected = Vec::with_capacity(samples);

    for sample_index in 0..samples {
        if mode == Mode::Cold {
            run_purge(purge_command)?;
        }

        eprintln!(
            "[sample] engine={} scenario={} mode={} sample={}/{}",
            engine.as_str(),
            scenario.label(),
            mode.as_str(),
            sample_index + 1,
            samples
        );
        let sample = measure_once(repo, engine, scenario).with_context(|| {
            format!(
                "measurement failed for engine={} scenario={} mode={} sample={}",
                engine.as_str(),
                scenario.label(),
                mode.as_str(),
                sample_index
            )
        })?;
        collected.push(sample);
    }

    Ok(ScenarioRun {
        engine,
        scenario,
        mode,
        samples: collected,
    })
}

fn run_purge(purge_command: Option<&str>) -> Result<()> {
    let Some(command) = purge_command else {
        return Ok(());
    };

    let status = Command::new("/bin/zsh")
        .arg("-lc")
        .arg(command)
        .status()
        .with_context(|| format!("failed to launch purge command: {command}"))?;

    if !status.success() {
        bail!("purge command exited with status {status}");
    }

    Ok(())
}

struct Args {
    repo: PathBuf,
    output: PathBuf,
    cold_samples: usize,
    warm_samples: usize,
    count_all_cold_samples: usize,
    count_all_warm_samples: usize,
    purge_command: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut repo = None;
        let mut output = None;
        let mut cold_samples = 7usize;
        let mut warm_samples = 30usize;
        let mut count_all_cold_samples = 1usize;
        let mut count_all_warm_samples = 3usize;
        let mut purge_command = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--repo" => repo = args.next().map(PathBuf::from),
                "--output" => output = args.next().map(PathBuf::from),
                "--cold-samples" => {
                    cold_samples = args
                        .next()
                        .context("--cold-samples requires a value")?
                        .parse()?
                }
                "--warm-samples" => {
                    warm_samples = args
                        .next()
                        .context("--warm-samples requires a value")?
                        .parse()?
                }
                "--count-all-cold-samples" => {
                    count_all_cold_samples = args
                        .next()
                        .context("--count-all-cold-samples requires a value")?
                        .parse()?
                }
                "--count-all-warm-samples" => {
                    count_all_warm_samples = args
                        .next()
                        .context("--count-all-warm-samples requires a value")?
                        .parse()?
                }
                "--purge-command" => purge_command = args.next(),
                other => bail!("unknown argument: {other}"),
            }
        }

        Ok(Self {
            repo: repo.context("--repo is required")?,
            output: output.context("--output is required")?,
            cold_samples,
            warm_samples,
            count_all_cold_samples,
            count_all_warm_samples,
            purge_command,
        })
    }
}
