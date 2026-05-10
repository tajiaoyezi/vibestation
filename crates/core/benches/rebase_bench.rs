//! MVP-16 Phase D · Criterion bench for rebase / merge / cherry-pick / conflict / crash recovery.
//!
//! 验证 spec §A.9 / §B.9 / §C.9 / §D.9 / §F.5 时间预算：
//! - rebase 10/100 commit clean
//! - merge no-ff 50 commit
//! - cherrypick single / range 10
//! - 3-way conflict 50 file backend detection（后端读取时间 · 不含前端 UI）
//! - crash recovery detection（启动时检测）
//!
//! 跑：`cargo bench --bench rebase_bench`
//! 输出：`target/criterion/<bench_name>/report/index.html`

use criterion::{criterion_group, criterion_main, Criterion};
use git2::{build::CheckoutBuilder, Repository, Signature};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use vibestation_core::rebase_ops::{
    cherrypick_start, conflict_status, detect_in_progress, merge_start, rebase_start,
    CherryPickRequest, MergeRequest, MergeStrategy, RebaseStartRequest,
};

// =========== Fixture helpers ===========

fn init_repo(dir: &Path) -> Repository {
    let repo = Repository::init(dir).unwrap();
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Bench").unwrap();
        config.set_str("user.email", "bench@example.com").unwrap();
    }
    repo
}

fn commit_file(
    repo: &Repository,
    path: &Path,
    name: &str,
    content: &str,
    message: &str,
) -> git2::Oid {
    let full = path.join(name);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&full, content).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(name)).unwrap();
    index.write().unwrap();
    let tree_oid = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();
    let sig = Signature::now("Bench", "bench@example.com").unwrap();
    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<_> = parent_commit.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .unwrap()
}

fn create_branch_at_head(repo: &Repository, name: &str) {
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch(name, &head, true).unwrap();
}

fn checkout_branch(repo: &Repository, name: &str) {
    repo.set_head(&format!("refs/heads/{name}")).unwrap();
    let mut co = CheckoutBuilder::new();
    co.force();
    repo.checkout_head(Some(&mut co)).unwrap();
}

/// Setup: main(base+1 commit) + feature(base+N commits) · 不冲突的不同文件 · 适合 clean rebase / merge
fn setup_diverged(feature_commit_count: usize) -> (TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();
    let repo = init_repo(&path);
    let _base = commit_file(&repo, &path, "base.txt", "base\n", "base");
    create_branch_at_head(&repo, "main");
    create_branch_at_head(&repo, "feature");
    checkout_branch(&repo, "main");
    commit_file(&repo, &path, "main.txt", "main\n", "main work");
    checkout_branch(&repo, "feature");
    for i in 0..feature_commit_count {
        let fname = format!("feature-{i}.txt");
        let content = format!("feature content {i}\n");
        commit_file(
            &repo,
            &path,
            &fname,
            &content,
            &format!("feature commit {i}"),
        );
    }
    (dir, path)
}

// =========== Benchmarks ===========

/// §A.9 · rebase 10 commit clean · spec 目标 < 1s（v0.3 范围）
fn bench_rebase_10_commits_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("rebase_10_commits_clean");
    group.sample_size(20);
    group.bench_function("rebase_10_commits_clean", |b| {
        b.iter_with_setup(
            || setup_diverged(10),
            |(_dir, path)| {
                let _ = rebase_start(
                    &path,
                    RebaseStartRequest {
                        workspace_id: "bench".to_string(),
                        branch: "feature".to_string(),
                        onto: "main".to_string(),
                        interactive: false,
                    },
                );
            },
        );
    });
    group.finish();
}

/// §A.9 · rebase 100 commit clean · spec 目标 < 5s
fn bench_rebase_100_commits_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("rebase_100_commits_clean");
    // criterion 默认 sample_size=100 · 100 commit rebase 单次几秒 · 总耗时太长 ·
    // 降到 10 取 P95 即可（spec §A.9 不要求 statistical 99% confidence · 量级验证够）
    group.sample_size(10);
    group.bench_function("rebase_100_commits_clean", |b| {
        b.iter_with_setup(
            || setup_diverged(100),
            |(_dir, path)| {
                let _ = rebase_start(
                    &path,
                    RebaseStartRequest {
                        workspace_id: "bench".to_string(),
                        branch: "feature".to_string(),
                        onto: "main".to_string(),
                        interactive: false,
                    },
                );
            },
        );
    });
    group.finish();
}

/// §B.9 · merge --no-ff with feature 50 commit · spec 目标 < 3s
fn bench_merge_no_ff_50_commits(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_no_ff_50_commits");
    group.sample_size(20);
    group.bench_function("merge_no_ff_50_commits", |b| {
        b.iter_with_setup(
            || {
                let (dir, path) = setup_diverged(50);
                // merge_start 要求 HEAD 在 main · setup_diverged 末尾在 feature · 切回 main
                let repo = Repository::open(&path).unwrap();
                checkout_branch(&repo, "main");
                drop(repo);
                (dir, path)
            },
            |(_dir, path)| {
                let _ = merge_start(
                    &path,
                    MergeRequest {
                        workspace_id: "bench".to_string(),
                        source_branch: "feature".to_string(),
                        strategy: MergeStrategy::NoFastForward,
                        commit_message: Some("merge feature".to_string()),
                    },
                );
            },
        );
    });
    group.finish();
}

/// §C.9 · cherry-pick single commit · spec 目标 < 1s
fn bench_cherrypick_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("cherrypick_single");
    group.sample_size(50);
    group.bench_function("cherrypick_single", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().to_path_buf();
                let repo = init_repo(&path);
                let _base = commit_file(&repo, &path, "base.txt", "base\n", "base");
                create_branch_at_head(&repo, "main");
                create_branch_at_head(&repo, "feature");
                checkout_branch(&repo, "feature");
                let target = commit_file(&repo, &path, "f.txt", "feature\n", "f");
                checkout_branch(&repo, "main");
                (dir, path, target.to_string())
            },
            |(_dir, path, sha)| {
                let _ = cherrypick_start(
                    &path,
                    CherryPickRequest {
                        workspace_id: "bench".to_string(),
                        commit_shas: vec![sha],
                        auto_commit: true,
                    },
                );
            },
        );
    });
    group.finish();
}

/// §C.9 · cherry-pick range 10 commit · spec 目标 < 5s
fn bench_cherrypick_range_10(c: &mut Criterion) {
    let mut group = c.benchmark_group("cherrypick_range_10");
    group.sample_size(20);
    group.bench_function("cherrypick_range_10", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().to_path_buf();
                let repo = init_repo(&path);
                let _base = commit_file(&repo, &path, "base.txt", "base\n", "base");
                create_branch_at_head(&repo, "main");
                create_branch_at_head(&repo, "feature");
                checkout_branch(&repo, "feature");
                let mut shas = Vec::with_capacity(10);
                for i in 0..10 {
                    let oid =
                        commit_file(&repo, &path, &format!("f-{i}.txt"), "x\n", &format!("f{i}"));
                    shas.push(oid.to_string());
                }
                checkout_branch(&repo, "main");
                (dir, path, shas)
            },
            |(_dir, path, shas)| {
                let _ = cherrypick_start(
                    &path,
                    CherryPickRequest {
                        workspace_id: "bench".to_string(),
                        commit_shas: shas,
                        auto_commit: true,
                    },
                );
            },
        );
    });
    group.finish();
}

/// §D.9 · 3-way conflict 50 file 后端检测时间 · spec 目标 < 2s（仅后端 conflict_status 读取）
fn bench_conflict_3way_50_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("conflict_3way_50_files");
    group.sample_size(20);
    group.bench_function("conflict_3way_50_files_status", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().to_path_buf();
                let repo = init_repo(&path);
                // base · 50 files
                for i in 0..50 {
                    commit_file(
                        &repo,
                        &path,
                        &format!("f-{i}.txt"),
                        "base\n",
                        &format!("base {i}"),
                    );
                }
                create_branch_at_head(&repo, "main");
                create_branch_at_head(&repo, "feature");
                // main 修 50 files · 单次 commit
                checkout_branch(&repo, "main");
                for i in 0..50 {
                    let fname = format!("f-{i}.txt");
                    fs::write(path.join(&fname), "main\n").unwrap();
                }
                let mut idx = repo.index().unwrap();
                for i in 0..50 {
                    idx.add_path(Path::new(&format!("f-{i}.txt"))).unwrap();
                }
                idx.write().unwrap();
                let tree_oid = idx.write_tree().unwrap();
                let tree = repo.find_tree(tree_oid).unwrap();
                let sig = Signature::now("Bench", "bench@example.com").unwrap();
                let parent = repo.head().unwrap().peel_to_commit().unwrap();
                repo.commit(Some("HEAD"), &sig, &sig, "main bulk", &tree, &[&parent])
                    .unwrap();
                // feature 修 50 files · 与 main 冲突
                checkout_branch(&repo, "feature");
                for i in 0..50 {
                    let fname = format!("f-{i}.txt");
                    fs::write(path.join(&fname), "feature\n").unwrap();
                }
                let mut idx = repo.index().unwrap();
                for i in 0..50 {
                    idx.add_path(Path::new(&format!("f-{i}.txt"))).unwrap();
                }
                idx.write().unwrap();
                let tree_oid = idx.write_tree().unwrap();
                let tree = repo.find_tree(tree_oid).unwrap();
                let parent = repo.head().unwrap().peel_to_commit().unwrap();
                repo.commit(Some("HEAD"), &sig, &sig, "feature bulk", &tree, &[&parent])
                    .unwrap();
                // 切回 main · merge feature → 全部冲突 · merge_start 返回 Err 但 .git/MERGE_HEAD + index conflict 已写入
                checkout_branch(&repo, "main");
                let _ = merge_start(
                    &path,
                    MergeRequest {
                        workspace_id: "bench".to_string(),
                        source_branch: "feature".to_string(),
                        strategy: MergeStrategy::NoFastForward,
                        commit_message: Some("conflict".to_string()),
                    },
                );
                (dir, path)
            },
            |(_dir, path)| {
                // 仅测后端 conflict_status 读取耗时（不重做 merge）
                let _ = conflict_status(&path);
            },
        );
    });
    group.finish();
}

/// §F.5 · crash recovery detection clean repo · spec 目标 < 200ms
fn bench_crash_recovery_detection_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("crash_recovery_detection_clean");
    group.sample_size(50);
    group.bench_function("crash_recovery_detection_clean", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().to_path_buf();
                let repo = init_repo(&path);
                commit_file(&repo, &path, "base.txt", "base\n", "base");
                (dir, path)
            },
            |(_dir, path)| {
                let _ = detect_in_progress(&path);
            },
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_rebase_10_commits_clean,
    bench_rebase_100_commits_clean,
    bench_merge_no_ff_50_commits,
    bench_cherrypick_single,
    bench_cherrypick_range_10,
    bench_conflict_3way_50_files,
    bench_crash_recovery_detection_clean,
);
criterion_main!(benches);
