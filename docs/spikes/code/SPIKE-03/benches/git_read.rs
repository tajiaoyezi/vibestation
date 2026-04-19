use std::env;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use spike_03_git_benchmark::{Engine, Scenario, run_scenario};

fn git_read_benchmark(c: &mut Criterion) {
    let repo = repo_path();
    let mut group = c.benchmark_group("git-read");
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(2));
    group.measurement_time(Duration::from_secs(8));

    for scenario in [Scenario::Log100, Scenario::Log1000, Scenario::Log10000] {
        for engine in Engine::ALL {
            group.bench_with_input(
                BenchmarkId::new(engine.as_str(), scenario.label()),
                &scenario,
                |b, &scenario| {
                    b.iter(|| {
                        let outcome =
                            run_scenario(&repo, engine, scenario).expect("benchmark run should succeed");
                        criterion::black_box(outcome);
                    });
                },
            );
        }
    }

    group.finish();
}

fn repo_path() -> PathBuf {
    env::var("SPIKE03_REPO")
        .map(PathBuf::from)
        .expect("SPIKE03_REPO must point to the linux kernel repository")
}

criterion_group!(benches, git_read_benchmark);
criterion_main!(benches);
