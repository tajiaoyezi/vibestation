//! MVP-21 Phase D · git_sync Criterion benchmarks.
//!
//! Uses local bare repositories as remotes so push/pull/fetch measurements stay
//! deterministic and do not depend on external network services.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use git2::{build::CheckoutBuilder, Oid, Repository, RepositoryInitOptions, Signature};
use std::path::Path;
use std::time::Duration;
use tempfile::TempDir;
use vibestation_core::{
    git_fetch, git_pull, git_push, AuthMethod, FetchRequest, GitStatusResponse, NetworkOpError,
    PullRequest, PullStrategy, PushRequest,
};

struct GitSyncFixture {
    _bare_dir: TempDir,
    _work_dir: TempDir,
    _producer_dir: Option<TempDir>,
    work_path: std::path::PathBuf,
}

fn create_fixture_clean() -> GitSyncFixture {
    let bare_dir = tempfile::tempdir().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.bare(true);
    opts.initial_head("main");
    Repository::init_opts(bare_dir.path(), &opts).unwrap();

    let work_dir = tempfile::tempdir().unwrap();
    let work = Repository::clone(bare_dir.path().to_str().unwrap(), work_dir.path()).unwrap();
    configure_repo(&work);
    work.set_head("refs/heads/main").unwrap();
    commit_file(&work, work_dir.path(), "seed.txt", b"seed", "initial");
    push_branch(&work, "main");

    let work_path = work_dir.path().to_path_buf();
    GitSyncFixture {
        _bare_dir: bare_dir,
        _work_dir: work_dir,
        _producer_dir: None,
        work_path,
    }
}

fn create_fixture_with_local_ahead() -> GitSyncFixture {
    let fixture = create_fixture_clean();
    let repo = Repository::open(&fixture.work_path).unwrap();
    commit_1mb_100(&repo, &fixture.work_path, "push");
    fixture
}

fn create_fixture_with_remote_ahead() -> GitSyncFixture {
    let mut fixture = create_fixture_clean();
    let producer_dir = clone_origin_from(&fixture);
    let producer = Repository::open(producer_dir.path()).unwrap();
    commit_1mb_100(&producer, producer_dir.path(), "pull");
    push_branch(&producer, "main");
    fixture._producer_dir = Some(producer_dir);
    fixture
}

fn create_fixture_with_conflict() -> GitSyncFixture {
    let mut fixture = create_fixture_clean();
    let producer_dir = clone_origin_from(&fixture);
    let producer = Repository::open(producer_dir.path()).unwrap();
    commit_file(
        &producer,
        producer_dir.path(),
        "seed.txt",
        b"remote-conflict\n",
        "remote conflict",
    );
    push_branch(&producer, "main");

    let repo = Repository::open(&fixture.work_path).unwrap();
    commit_file(
        &repo,
        &fixture.work_path,
        "seed.txt",
        b"local-conflict\n",
        "local conflict",
    );

    fixture._producer_dir = Some(producer_dir);
    fixture
}

fn create_fixture_with_fetch_refs() -> GitSyncFixture {
    let mut fixture = create_fixture_clean();
    let producer_dir = clone_origin_from(&fixture);
    let producer = Repository::open(producer_dir.path()).unwrap();
    let base = producer.head().unwrap().peel_to_commit().unwrap();

    for branch_idx in 0..10 {
        let branch = format!("bench/ref-{branch_idx:02}");
        producer.branch(&branch, &base, false).unwrap();
        producer.set_head(&format!("refs/heads/{branch}")).unwrap();
        producer
            .checkout_head(Some(CheckoutBuilder::new().force()))
            .unwrap();
        for commit_idx in 0..10 {
            let file = format!("fetch-{branch_idx:02}-{commit_idx:02}.bin");
            let message = format!("fetch {branch_idx:02} {commit_idx:02}");
            commit_file(
                &producer,
                producer_dir.path(),
                &file,
                &payload_10kb(branch_idx * 10 + commit_idx),
                &message,
            );
        }
        push_branch(&producer, &branch);
    }

    fixture._producer_dir = Some(producer_dir);
    fixture
}

fn clone_origin_from(fixture: &GitSyncFixture) -> TempDir {
    let origin_url = Repository::open(&fixture.work_path)
        .unwrap()
        .find_remote("origin")
        .unwrap()
        .url()
        .unwrap()
        .to_string();
    let producer_dir = tempfile::tempdir().unwrap();
    let producer = Repository::clone(&origin_url, producer_dir.path()).unwrap();
    configure_repo(&producer);
    producer_dir
}

fn configure_repo(repo: &Repository) {
    let mut cfg = repo.config().unwrap();
    cfg.set_str("user.name", "Bench").unwrap();
    cfg.set_str("user.email", "bench@example.com").unwrap();
}

fn commit_1mb_100(repo: &Repository, root: &Path, prefix: &str) {
    for idx in 0..100 {
        let file = format!("{prefix}-{idx:03}.bin");
        let message = format!("{prefix} {idx:03}");
        commit_file(repo, root, &file, &payload_10kb(idx), &message);
    }
}

fn payload_10kb(seed: usize) -> Vec<u8> {
    let mut payload = Vec::with_capacity(10 * 1024);
    for idx in 0..(10 * 1024) {
        payload.push(((seed + idx) % 251) as u8);
    }
    payload
}

fn commit_file(repo: &Repository, root: &Path, file: &str, content: &[u8], message: &str) -> Oid {
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

fn push_branch(repo: &Repository, branch: &str) {
    let refspec = format!("refs/heads/{branch}:refs/heads/{branch}");
    let mut remote = repo.find_remote("origin").unwrap();
    remote.push(&[refspec.as_str()], None).unwrap();
    let mut remote = repo.find_remote("origin").unwrap();
    remote.fetch(&[branch], None, None).unwrap();
}

fn workspace_id() -> String {
    "bench-workspace".to_string()
}

fn push_req() -> PushRequest {
    PushRequest {
        workspace_id: workspace_id(),
        remote: "origin".to_string(),
        branch: "main".to_string(),
        force: false,
        expected_remote_oid: None,
        auth_method: Some(AuthMethod::HttpsHelper),
        task_id: Some("bench-push".to_string()),
    }
}

fn pull_req() -> PullRequest {
    PullRequest {
        workspace_id: workspace_id(),
        remote: "origin".to_string(),
        branch: "main".to_string(),
        strategy: PullStrategy::Merge,
        frontend_status_snapshot: Some(GitStatusResponse {
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
        }),
        frontend_status_taken_at: Some(0),
        auth_method: Some(AuthMethod::HttpsHelper),
        task_id: Some("bench-pull".to_string()),
    }
}

fn fetch_req() -> FetchRequest {
    FetchRequest {
        workspace_id: workspace_id(),
        remote: "origin".to_string(),
        prune: false,
        auth_method: Some(AuthMethod::HttpsHelper),
        task_id: Some("bench-fetch".to_string()),
    }
}

fn bench_push_1mb_100commits(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_sync_push_1mb_100commits");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("push", |b| {
        b.iter_batched(
            create_fixture_with_local_ahead,
            |fixture| {
                let result = git_push(&fixture.work_path, push_req()).unwrap();
                black_box(result);
                black_box(fixture);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_pull_ff_1mb_100commits(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_sync_pull_ff_1mb_100commits");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(4));
    group.bench_function("pull_ff", |b| {
        b.iter_batched(
            create_fixture_with_remote_ahead,
            |fixture| {
                let result = git_pull(&fixture.work_path, pull_req()).unwrap();
                black_box(result);
                black_box(fixture);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_pull_conflict_abort(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_sync_pull_conflict_abort");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("pull_conflict_abort", |b| {
        b.iter_batched(
            create_fixture_with_conflict,
            |fixture| {
                let error = git_pull(&fixture.work_path, pull_req()).unwrap_err();
                assert!(matches!(
                    error,
                    NetworkOpError::MergeConflict { aborted: true, .. }
                ));
                let actual = std::fs::read_to_string(fixture.work_path.join("seed.txt")).unwrap();
                let normalized = actual.replace("\r\n", "\n").replace('\r', "\n");
                assert_eq!(normalized, "local-conflict\n");
                black_box(error);
                black_box(fixture);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_fetch_10refs_100commits(c: &mut Criterion) {
    let mut group = c.benchmark_group("git_sync_fetch_10refs_100commits");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(3));
    group.bench_function("fetch", |b| {
        b.iter_batched(
            create_fixture_with_fetch_refs,
            |fixture| {
                let result = git_fetch(&fixture.work_path, fetch_req()).unwrap();
                assert!(result.fetched_refs.len() >= 10);
                black_box(result);
                black_box(fixture);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_push_1mb_100commits,
    bench_pull_ff_1mb_100commits,
    bench_pull_conflict_abort,
    bench_fetch_10refs_100commits
);
criterion_main!(benches);
