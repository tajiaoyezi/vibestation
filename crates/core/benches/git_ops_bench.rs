use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;

fn bench_stage_single_file(c: &mut Criterion) {
    c.bench_function("stage_single_file", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::TempDir::new().unwrap();
                let path = dir.path().to_path_buf();
                let repo = git2::Repository::init(&path).unwrap();
                let mut cfg = repo.config().unwrap();
                cfg.set_str("user.name", "Bench").unwrap();
                cfg.set_str("user.email", "bench@example.com").unwrap();
                fs::write(path.join("base.txt"), "base").unwrap();
                vibestation_core::GitOpsService::stage_files(&path, &["base.txt".to_string()])
                    .unwrap();
                vibestation_core::GitOpsService::commit(&path, "initial", false).unwrap();
                fs::write(path.join("target.txt"), "content").unwrap();
                (dir, path)
            },
            |(_dir, path)| {
                let _ = vibestation_core::GitOpsService::stage_files(
                    &path,
                    &["target.txt".to_string()],
                );
            },
        );
    });
}

fn bench_commit_typical(c: &mut Criterion) {
    c.bench_function("commit_typical", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::TempDir::new().unwrap();
                let path = dir.path().to_path_buf();
                let repo = git2::Repository::init(&path).unwrap();
                let mut cfg = repo.config().unwrap();
                cfg.set_str("user.name", "Bench").unwrap();
                cfg.set_str("user.email", "bench@example.com").unwrap();
                fs::write(path.join("base.txt"), "base").unwrap();
                vibestation_core::GitOpsService::stage_files(&path, &["base.txt".to_string()])
                    .unwrap();
                vibestation_core::GitOpsService::commit(&path, "initial", false).unwrap();
                fs::write(path.join("target.txt"), "changed").unwrap();
                vibestation_core::GitOpsService::stage_files(&path, &["target.txt".to_string()])
                    .unwrap();
                (dir, path)
            },
            |(_dir, path)| {
                let _ = vibestation_core::GitOpsService::commit(&path, "bench commit", false);
            },
        );
    });
}

fn bench_stage_all_1000_files(c: &mut Criterion) {
    c.bench_function("stage_all_1000_files", |b| {
        b.iter_with_setup(
            || {
                let dir = tempfile::TempDir::new().unwrap();
                let path = dir.path().to_path_buf();
                let repo = git2::Repository::init(&path).unwrap();
                let mut cfg = repo.config().unwrap();
                cfg.set_str("user.name", "Bench").unwrap();
                cfg.set_str("user.email", "bench@example.com").unwrap();
                fs::write(path.join("base.txt"), "base").unwrap();
                vibestation_core::GitOpsService::stage_files(&path, &["base.txt".to_string()])
                    .unwrap();
                vibestation_core::GitOpsService::commit(&path, "initial", false).unwrap();

                let mut files = Vec::with_capacity(1000);
                for i in 0..1000 {
                    let name = format!("file_{:04}.txt", i);
                    fs::write(path.join(&name), format!("content {}", i)).unwrap();
                    files.push(name);
                }
                (dir, path, files)
            },
            |(_dir, path, files)| {
                let _ = vibestation_core::GitOpsService::stage_files(&path, &files);
            },
        );
    });
}

criterion_group!(
    benches,
    bench_stage_single_file,
    bench_commit_typical,
    bench_stage_all_1000_files
);
criterion_main!(benches);
