//! MVP-08 Phase E · F.1 + F.2 性能基准
//!
//! F.1: 1k 文件 git2 statuses() < 100ms P99
//! F.2: 1k 文件 IPC 序列化 + 反序列化 < 30ms P99

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use git2::{IndexAddOption, Repository, Signature};
use tempfile::TempDir;
use vibestation_core::{GitStatusRequest, GitStatusService};

fn create_1k_file_repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("Bench", "bench@test.com").unwrap();

    for i in 0..800u32 {
        let file_name = format!("src/committed_{:04}.rs", i);
        let dir_path = dir.path().join(&file_name).parent().unwrap().to_path_buf();
        if !dir_path.exists() {
            std::fs::create_dir_all(&dir_path).unwrap();
        }
        std::fs::write(
            dir.path().join(&file_name),
            format!("// committed {i}\nfn main() {{ println!(\"{i}\"); }}\n"),
        )
        .unwrap();
    }

    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
        .unwrap();
    drop(tree);

    for i in 0..100u32 {
        let file_name = format!("src/staged_{:04}.rs", i);
        std::fs::write(
            dir.path().join(&file_name),
            format!("// staged {i}\nfn staged() {{}}\n"),
        )
        .unwrap();
    }
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    for i in 0..100u32 {
        let file_name = format!("src/modified_{:04}.rs", i);
        std::fs::write(
            dir.path().join(&file_name),
            format!("// modified {i}\nfn modified() {{}} // unstaged change\n"),
        )
        .unwrap();
    }

    for i in 0..100u32 {
        let file_name = format!("src/untracked_{:04}.rs", i);
        std::fs::write(
            dir.path().join(&file_name),
            format!("// untracked {i}\nfn untracked() {{}}\n"),
        )
        .unwrap();
    }

    drop(index);
    drop(repo);

    dir
}

fn bench_git_status_query_1k(c: &mut Criterion) {
    let dir = create_1k_file_repo();
    let req = GitStatusRequest {
        workspace_id: "bench-ws".to_string(),
    };

    let mut group = c.benchmark_group("git_status_query_1k");
    group.sample_size(30);
    group.bench_function("statuses_query", |b| {
        b.iter(|| {
            let response = GitStatusService::query(dir.path(), &req).unwrap();
            criterion::black_box(&response);
        });
    });
    group.finish();

    drop(dir);
}

fn bench_ipc_serialization_1k(c: &mut Criterion) {
    let dir = create_1k_file_repo();
    let req = GitStatusRequest {
        workspace_id: "bench-ws".to_string(),
    };
    let response = GitStatusService::query(dir.path(), &req).unwrap();

    let mut group = c.benchmark_group("git_status_ipc_1k");
    group.sample_size(50);
    group.bench_with_input(
        BenchmarkId::new(
            "serde_json_roundtrip",
            response.staged.len() + response.unstaged.len() + response.untracked.len(),
        ),
        &response,
        |b, resp| {
            b.iter(|| {
                let json = serde_json::to_string(resp).unwrap();
                let decoded: vibestation_core::GitStatusResponse =
                    serde_json::from_str(&json).unwrap();
                criterion::black_box(decoded);
            });
        },
    );
    group.finish();

    drop(dir);
}

criterion_group!(
    benches,
    bench_git_status_query_1k,
    bench_ipc_serialization_1k
);
criterion_main!(benches);
