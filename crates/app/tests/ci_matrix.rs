// crates/app/tests/ci_matrix.rs
// SCEN-5.2.1 / AC1 · 断言 rust-build job 含 windows-latest 矩阵

use std::path::PathBuf;

/// 定位 repo 根的 .github/workflows/ci.yml
/// CARGO_MANIFEST_DIR = crates/app → 上溯两级到 repo 根
fn read_ci_workflow() -> String {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let path = repo_root.join(".github").join("workflows").join("ci.yml");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn test_5_2_1_rust_build_has_windows_matrix() {
    // TEST-5.2.1 · 字符串断言兜底（不强依赖 serde_yaml）：
    // matrix 化后 ci.yml 必含 "windows-latest" 与矩阵占位 "${{ matrix.os }}"
    let yaml = read_ci_workflow();
    assert!(
        yaml.contains("windows-latest"),
        "ci.yml rust-build job must include windows-latest in matrix"
    );
    assert!(
        yaml.contains("matrix.os"),
        "ci.yml must use ${{{{ matrix.os }}}} runs-on for cross-platform leg"
    );
}

#[test]
fn test_5_2_2_workflow_dispatch_trigger_preserved() {
    // TEST-5.2.2 · ADR-021 零回归：触发仍是 workflow_dispatch（不恢复 push/pull_request）
    let yaml = read_ci_workflow();
    assert!(
        yaml.contains("workflow_dispatch"),
        "ci.yml must keep workflow_dispatch trigger (ADR-021)"
    );
}
