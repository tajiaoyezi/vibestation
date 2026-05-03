//! MVP-13 Phase D · branch_ops Criterion benchmarks.
//!
//! Measures branch list/create/checkout/delete plus backend switcher query against
//! the Phase D performance gates.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use git2::{Oid, Repository, Signature};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use vibestation_core::{
    branch_checkout, branch_create, branch_delete, branch_list, branch_switcher_query,
    BranchCheckoutRequest, BranchCreateRequest, BranchDeleteRequest, SwitcherQueryRequest,
};

fn create_base_repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    repo.set_head("refs/heads/main").unwrap();
    let mut cfg = repo.config().unwrap();
    cfg.set_str("user.name", "Bench").unwrap();
    cfg.set_str("user.email", "bench@example.com").unwrap();
    commit_file(&repo, dir.path(), "initial.txt", "initial", "initial");
    drop(repo);
    dir
}

fn create_branch_fixture(count: usize) -> TempDir {
    let dir = create_base_repo();
    let repo = Repository::open(dir.path()).unwrap();
    let head = repo.head().unwrap().target().unwrap();
    for idx in 0..count {
        create_branch(&repo, &format!("feat/bench-{idx:04}"), head);
    }
    drop(repo);
    dir
}

fn create_checkout_fixture() -> TempDir {
    let dir = create_base_repo();
    let repo = Repository::open(dir.path()).unwrap();
    let head = repo.head().unwrap().target().unwrap();
    create_branch(&repo, "feat/checkout-target", head);
    drop(repo);
    dir
}

fn create_delete_fixture() -> TempDir {
    let dir = create_base_repo();
    let repo = Repository::open(dir.path()).unwrap();
    let head = repo.head().unwrap().target().unwrap();
    create_branch(&repo, "feat/delete-target", head);
    drop(repo);
    dir
}

fn commit_file(repo: &Repository, root: &Path, file: &str, content: &str, message: &str) -> Oid {
    std::fs::write(root.join(file), content).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(file)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = Signature::now("Bench", "bench@example.com").unwrap();
    let parents = repo
        .head()
        .ok()
        .and_then(|head| head.peel_to_commit().ok())
        .into_iter()
        .collect::<Vec<_>>();
    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .unwrap()
}

fn create_branch(repo: &Repository, name: &str, target: Oid) {
    let commit = repo.find_commit(target).unwrap();
    repo.branch(name, &commit, false).unwrap();
}

fn workspace_id() -> String {
    "bench-workspace".to_string()
}

fn bench_branch_list_10(c: &mut Criterion) {
    let dir = create_branch_fixture(10);
    let mut group = c.benchmark_group("branch_list_10");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("list", |b| {
        b.iter(|| {
            let response = branch_list(dir.path()).unwrap();
            black_box(response);
        });
    });
    group.finish();
    drop(dir);
}

fn bench_branch_list_1000(c: &mut Criterion) {
    let dir = create_branch_fixture(1000);
    let mut group = c.benchmark_group("branch_list_1000");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("list", |b| {
        b.iter(|| {
            let response = branch_list(dir.path()).unwrap();
            black_box(response);
        });
    });
    group.finish();
    drop(dir);
}

fn bench_branch_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_create");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("create", |b| {
        b.iter_batched(
            create_base_repo,
            |dir| {
                branch_create(
                    dir.path(),
                    BranchCreateRequest {
                        workspace_id: workspace_id(),
                        name: "feat/new-branch".to_string(),
                        from_ref: None,
                        checkout: false,
                    },
                )
                .unwrap();
                black_box(dir);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_branch_checkout_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_checkout_clean");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("checkout", |b| {
        b.iter_batched(
            create_checkout_fixture,
            |dir| {
                let result = branch_checkout(
                    dir.path(),
                    BranchCheckoutRequest {
                        workspace_id: workspace_id(),
                        name: "feat/checkout-target".to_string(),
                        force: false,
                    },
                )
                .unwrap();
                black_box(result);
                black_box(dir);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_branch_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_delete");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("delete", |b| {
        b.iter_batched(
            create_delete_fixture,
            |dir| {
                branch_delete(
                    dir.path(),
                    BranchDeleteRequest {
                        workspace_id: workspace_id(),
                        name: "feat/delete-target".to_string(),
                        force: false,
                    },
                )
                .unwrap();
                black_box(dir);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_fuzzy_filter_100(c: &mut Criterion) {
    let dir = create_branch_fixture(100);
    let mut group = c.benchmark_group("fuzzy_filter_100");
    group.sample_size(30);
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("query", |b| {
        b.iter(|| {
            let result = branch_switcher_query(
                dir.path(),
                SwitcherQueryRequest {
                    workspace_id: workspace_id(),
                    query: "fb".to_string(),
                    limit: 25,
                },
            )
            .unwrap();
            black_box(result);
        });
    });
    group.finish();
    drop(dir);
}

criterion_group!(
    benches,
    bench_branch_list_10,
    bench_branch_list_1000,
    bench_branch_create,
    bench_branch_checkout_clean,
    bench_branch_delete,
    bench_fuzzy_filter_100
);
criterion_main!(benches);
