// crates/app/tests/bundle_config.rs
// SCEN-5.1.1 / AC1 · 解析 tauri.conf.json 断言 bundle.targets

use serde_json::Value;
use std::path::PathBuf;

/// 读取 crates/app/tauri.conf.json 并解析为 serde_json::Value
fn load_tauri_conf() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).expect("tauri.conf.json must be valid JSON")
}

#[test]
fn test_5_1_1_bundle_targets_include_windows() {
    // TEST-5.1.1
    let conf = load_tauri_conf();
    let targets = conf["bundle"]["targets"]
        .as_array()
        .expect("bundle.targets must be an array");
    let names: Vec<&str> = targets.iter().filter_map(Value::as_str).collect();
    assert!(
        names.contains(&"nsis"),
        "bundle.targets must include nsis, got {names:?}"
    );
    assert!(
        names.contains(&"msi"),
        "bundle.targets must include msi, got {names:?}"
    );
}

#[test]
fn test_5_1_2_bundle_targets_preserve_unix() {
    // TEST-5.1.2 · 零回归：不得删除 mac/Linux targets
    let conf = load_tauri_conf();
    let names: Vec<String> = conf["bundle"]["targets"]
        .as_array()
        .expect("array")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    for expected in ["dmg", "appimage", "deb"] {
        assert!(
            names.iter().any(|n| n == expected),
            "bundle.targets must still include {expected} (no Unix regression), got {names:?}"
        );
    }
}
