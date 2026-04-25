//! MVP-08 Phase E · A.2 + E.3 + F.4 + F.5 性能基准
//!
//! F.4: 1k 行 diff similar 计算 < 200ms P99
//! F.5: 10k 行 diff similar 计算 < 1s P99
//! E.3: 100k 行硬 stop 验证（truncatedReason）
//! A.2 辅助：DiffService::compute 端到端（含 git2/gix IO）

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use tempfile::TempDir;
use vibestation_core::{DiffRequest, DiffService};

fn generate_text_lines(count: usize) -> String {
    (0..count)
        .map(|i| {
            format!(
                "line {}: this is text content with some variation {}",
                i,
                i % 7
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_short_lines(count: usize) -> String {
    (0..count)
        .map(|i| format!("{i}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn modify_text_lines(text: &str, change_rate: usize) -> String {
    text.lines()
        .enumerate()
        .map(|(i, line)| {
            if i % change_rate == 0 {
                format!("line {}: modified content here {}", i, i % 13)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

use git2::{IndexAddOption, Repository, Signature};

fn create_repo_with_committed_file(content: &str, file_name: &str) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("Bench", "bench@test.com").unwrap();

    let file_path = dir.path().join(file_name);
    if let Some(parent) = file_path.parent() {
        if parent != dir.path() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
    std::fs::write(&file_path, content).unwrap();

    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();
    drop(tree);
    drop(repo);

    dir
}

fn bench_diff_similar_1k(c: &mut Criterion) {
    let old_text = generate_text_lines(1000);
    let new_text = modify_text_lines(&old_text, 5);

    let dir = create_repo_with_committed_file(&old_text, "bench_file.rs");
    std::fs::write(dir.path().join("bench_file.rs"), &new_text).unwrap();

    let req = DiffRequest {
        workspace_id: "bench-ws".to_string(),
        source: "unstaged".to_string(),
        file_path: "bench_file.rs".to_string(),
        allow_large_file: false,
    };

    let mut group = c.benchmark_group("diff_compute");
    group.sample_size(20);
    group.bench_with_input(
        BenchmarkId::new("similar_1k_lines", 1000),
        &req,
        |b, req| {
            b.iter(|| {
                let response = DiffService::compute(dir.path(), req).unwrap();
                criterion::black_box(&response);
            });
        },
    );
    group.finish();

    drop(dir);
}

fn bench_diff_similar_10k(c: &mut Criterion) {
    let old_text = generate_text_lines(10_000);
    let new_text = modify_text_lines(&old_text, 5);

    let dir = create_repo_with_committed_file(&old_text, "bench_10k.rs");
    std::fs::write(dir.path().join("bench_10k.rs"), &new_text).unwrap();

    let req = DiffRequest {
        workspace_id: "bench-ws".to_string(),
        source: "unstaged".to_string(),
        file_path: "bench_10k.rs".to_string(),
        allow_large_file: false,
    };

    let mut group = c.benchmark_group("diff_compute");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(15));
    group.bench_with_input(
        BenchmarkId::new("similar_10k_lines", 10_000),
        &req,
        |b, req| {
            b.iter(|| {
                let response = DiffService::compute(dir.path(), req).unwrap();
                criterion::black_box(&response);
            });
        },
    );
    group.finish();

    drop(dir);
}

fn bench_diff_pure_similar(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_pure_similar");
    for size in [1_000, 10_000] {
        let old_text = generate_text_lines(size);
        let new_text = modify_text_lines(&old_text, 5);
        group.bench_with_input(BenchmarkId::new("lines", size), &size, |b, _| {
            b.iter(|| {
                let diff = similar::TextDiff::from_lines(&old_text, &new_text);
                let hunks = diff.grouped_ops(3);
                criterion::black_box(hunks.len());
            });
        });
    }
    group.finish();
}

fn bench_diff_large_file_truncation(c: &mut Criterion) {
    let large_text = generate_short_lines(100_001);
    let dir = create_repo_with_committed_file(&large_text, "huge_file.rs");

    let req = DiffRequest {
        workspace_id: "bench-ws".to_string(),
        source: "staged".to_string(),
        file_path: "huge_file.rs".to_string(),
        allow_large_file: false,
    };

    let mut group = c.benchmark_group("diff_truncation");
    group.sample_size(10);
    group.bench_function("100k_lines_reject", |b| {
        b.iter(|| {
            let response = DiffService::compute(dir.path(), &req).unwrap();
            assert!(response.truncated, "expected truncated for 100k+ lines");
            assert_eq!(
                response.truncated_reason.as_deref(),
                Some("too_many_lines"),
                "expected too_many_lines reason, got {:?}",
                response.truncated_reason
            );
            criterion::black_box(&response);
        });
    });
    group.finish();

    drop(dir);
}

criterion_group!(
    benches,
    bench_diff_similar_1k,
    bench_diff_similar_10k,
    bench_diff_pure_similar,
    bench_diff_large_file_truncation
);
criterion_main!(benches);
