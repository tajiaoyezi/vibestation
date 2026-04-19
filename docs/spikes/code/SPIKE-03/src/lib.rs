use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Engine {
    Git2,
    Gix,
}

impl Engine {
    pub const ALL: [Engine; 2] = [Engine::Git2, Engine::Gix];

    pub fn as_str(self) -> &'static str {
        match self {
            Engine::Git2 => "git2",
            Engine::Gix => "gix",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Scenario {
    Log100,
    Log1000,
    Log10000,
    CountAll,
}

impl Scenario {
    pub const ALL: [Scenario; 4] = [
        Scenario::Log100,
        Scenario::Log1000,
        Scenario::Log10000,
        Scenario::CountAll,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Scenario::Log100 => "log -100",
            Scenario::Log1000 => "log -1000",
            Scenario::Log10000 => "log -10000",
            Scenario::CountAll => "count-all",
        }
    }

    pub fn log_limit(self) -> Option<usize> {
        match self {
            Scenario::Log100 => Some(100),
            Scenario::Log1000 => Some(1_000),
            Scenario::Log10000 => Some(10_000),
            Scenario::CountAll => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Cold,
    Warm,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Cold => "cold",
            Mode::Warm => "warm",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub duration_ms: f64,
    pub visited: usize,
    pub checksum: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioRun {
    pub engine: Engine,
    pub scenario: Scenario,
    pub mode: Mode,
    pub samples: Vec<Sample>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub repo_path: String,
    pub cold_samples: usize,
    pub warm_samples: usize,
    pub count_all_cold_samples: usize,
    pub count_all_warm_samples: usize,
    pub purge_command: Option<String>,
    pub runs: Vec<ScenarioRun>,
}

#[derive(Clone, Copy, Debug)]
pub struct BenchOutcome {
    pub visited: usize,
    pub checksum: u64,
}

pub fn measure_once(repo_path: &Path, engine: Engine, scenario: Scenario) -> Result<Sample> {
    let started_at = Instant::now();
    let outcome = run_scenario(repo_path, engine, scenario)?;
    Ok(Sample {
        duration_ms: duration_to_ms(started_at.elapsed()),
        visited: outcome.visited,
        checksum: outcome.checksum,
    })
}

pub fn run_scenario(repo_path: &Path, engine: Engine, scenario: Scenario) -> Result<BenchOutcome> {
    match (engine, scenario.log_limit()) {
        (Engine::Git2, Some(limit)) => git2_log(repo_path, limit),
        (Engine::Git2, None) => git2_count_all(repo_path),
        (Engine::Gix, Some(limit)) => gix_log(repo_path, limit),
        (Engine::Gix, None) => gix_count_all(repo_path),
    }
}

fn git2_log(repo_path: &Path, limit: usize) -> Result<BenchOutcome> {
    let repo = git2::Repository::open(repo_path)
        .with_context(|| format!("failed to open repo with git2: {}", repo_path.display()))?;
    let head = repo.head()?.peel_to_commit()?;
    let mut walk = repo.revwalk()?;
    walk.set_sorting(git2::Sort::TIME)?;
    walk.push(head.id())?;

    let mut visited = 0usize;
    let mut checksum = 0u64;

    for oid in walk.take(limit) {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let summary_len = commit.summary_bytes().map_or(0usize, |summary| summary.len());
        visited += 1;
        checksum = checksum
            .wrapping_mul(31)
            .wrapping_add(summary_len as u64)
            .wrapping_add(oid.as_bytes()[0] as u64);
    }

    Ok(BenchOutcome { visited, checksum })
}

fn git2_count_all(repo_path: &Path) -> Result<BenchOutcome> {
    let repo = git2::Repository::open(repo_path)
        .with_context(|| format!("failed to open repo with git2: {}", repo_path.display()))?;
    let head = repo.head()?.peel_to_commit()?;
    let mut walk = repo.revwalk()?;
    walk.set_sorting(git2::Sort::TIME)?;
    walk.push(head.id())?;

    let mut visited = 0usize;
    let mut checksum = 0u64;

    for oid in walk {
        let oid = oid?;
        visited += 1;
        checksum = checksum
            .wrapping_mul(131)
            .wrapping_add(oid.as_bytes()[0] as u64);
    }

    Ok(BenchOutcome { visited, checksum })
}

fn gix_log(repo_path: &Path, limit: usize) -> Result<BenchOutcome> {
    let repo = gix::open(repo_path)
        .with_context(|| format!("failed to open repo with gix: {}", repo_path.display()))?;
    let head = repo.head_id()?;
    let mut walk = repo.rev_walk([head.detach()]).all()?;

    let mut visited = 0usize;
    let mut checksum = 0u64;

    while visited < limit {
        let Some(info) = walk.next() else {
            break;
        };

        let info = info?;
        let commit = info.object()?;
        let summary = commit.message()?.summary();
        visited += 1;
        checksum = checksum
            .wrapping_mul(31)
            .wrapping_add(summary.len() as u64)
            .wrapping_add(info.id().to_string().as_bytes()[0] as u64);
    }

    Ok(BenchOutcome { visited, checksum })
}

fn gix_count_all(repo_path: &Path) -> Result<BenchOutcome> {
    let repo = gix::open(repo_path)
        .with_context(|| format!("failed to open repo with gix: {}", repo_path.display()))?;
    let head = repo.head_id()?;
    let mut walk = repo.rev_walk([head.detach()]).all()?;

    let mut visited = 0usize;
    let mut checksum = 0u64;

    for info in &mut walk {
        let info = info?;
        visited += 1;
        checksum = checksum
            .wrapping_mul(131)
            .wrapping_add(info.id().to_string().as_bytes()[0] as u64);
    }

    Ok(BenchOutcome { visited, checksum })
}

fn duration_to_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
