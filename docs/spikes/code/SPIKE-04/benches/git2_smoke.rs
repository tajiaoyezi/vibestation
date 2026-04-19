use criterion::{criterion_group, criterion_main, Criterion};

fn git2_smoke_bench(c: &mut Criterion) {
    c.bench_function("git2_init_add_commit", |b| {
        b.iter(|| {
            let tmp = tempfile::tempdir().unwrap();
            let dir = tmp.path();

            let repo = git2::Repository::init(dir).unwrap();
            let sig = git2::Signature::new("SPIKE-04", "spike@vibestation.test", &git2::Time::new(1_700_000_000, 0)).unwrap();

            let readme_path = dir.join("README.md");
            std::fs::write(&readme_path, "你好 · SPIKE-04 测试\n").unwrap();

            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("README.md")).unwrap();
            index.write().unwrap();

            let tree_id = index.write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();

            let commit_id = repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "test: 中文 commit 测试 🎉",
                &tree,
                &[],
            ).unwrap();

            criterion::black_box(commit_id);
        });
    });
}

criterion_group!(name = git2_smoke; config = Criterion::default().sample_size(20); targets = git2_smoke_bench);
criterion_main!(git2_smoke);