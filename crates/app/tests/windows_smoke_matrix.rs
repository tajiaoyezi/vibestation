//! task-6.2 · Windows smoke-matrix 汇总断言（跨平台可跑 · 满足 §9 unit-test 强制门槛）。
//!
//! 本测试是 Windows 适配工作流的端到端集成兜底之**机械可验证**部分（GUI critical UX path
//! 走 docs/runtime-evidence/windows-smoke/ 手动归档 · §2.14）。聚合校验 Windows 链路前置条件
//! 不漂移，跨 phase 串起关键不变量：
//!
//! - AC1 / SCEN-6.2.1：tauri.conf bundle.targets 含 nsis + msi（Windows 安装包前置 · task-5.1）。
//! - AC4 / SCEN-6.2.4：三平台矩阵零回归——Unix bundle target 仍在（不为 Windows 牺牲 mac/Linux）。
//! - AC5 / SCEN-6.2.5：Windows smoke checklist 文档存在且非空（防空 smoke）+ ConPTY 自动化覆盖在位。
//!
//! 跨平台可跑：在 Windows / macOS / Ubuntu 都断言相同的配置 / 文档 / 不变量（不 spawn 进程）。

use std::path::{Path, PathBuf};

use serde_json::Value;

/// repo 根（CARGO_MANIFEST_DIR = crates/app → 上溯两级）。
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// 读取并解析 crates/app/tauri.conf.json。
fn load_tauri_conf() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).expect("tauri.conf.json 应为合法 JSON")
}

fn bundle_target_names(conf: &Value) -> Vec<String> {
    conf["bundle"]["targets"]
        .as_array()
        .expect("bundle.targets 应为数组")
        .iter()
        .filter_map(|t| t.as_str().map(str::to_string))
        .collect()
}

/// TEST-6.2.1（SCEN-6.2.1 / AC1）：Windows bundle targets 含 nsis + msi。
#[test]
fn test_6_2_1_tauri_conf_has_windows_bundle_targets() {
    let conf = load_tauri_conf();
    let names = bundle_target_names(&conf);
    assert!(
        names.iter().any(|n| n == "nsis"),
        "Windows 安装包前置：bundle.targets 须含 nsis · 实际 {names:?}"
    );
    assert!(
        names.iter().any(|n| n == "msi"),
        "Windows 安装包前置：bundle.targets 须含 msi · 实际 {names:?}"
    );
}

/// TEST-6.2.2（SCEN-6.2.5 / AC5）：Windows smoke checklist 文档存在且非空（防空 smoke）·
/// 且 ConPTY 进程级自动化覆盖文件在位（spawn/echo/exit 已有自动兜底）。
#[test]
fn test_6_2_2_windows_smoke_checklist_present() {
    let checklist = repo_root()
        .join("docs")
        .join("runtime-evidence")
        .join("windows-smoke")
        .join("README.md");
    let content = std::fs::read_to_string(&checklist).unwrap_or_else(|e| {
        panic!(
            "Windows smoke checklist 必须存在 {}: {e}",
            checklist.display()
        )
    });
    assert!(
        content.trim().len() > 200,
        "smoke checklist 不得为空壳（防空 smoke）· 实际长度 {}",
        content.trim().len()
    );
    // checklist 必须列出 critical UX path 四步锚点（起窗 / Tab / 回显 / Git status 刷新）
    for anchor in ["新建 Tab", "ConPTY", "git --version", "Git status"] {
        assert!(
            content.contains(anchor),
            "smoke checklist 应含 critical UX path 锚点 {anchor:?}"
        );
    }

    // ConPTY 进程级语义已由 task-2.2 集成测试自动化覆盖（spawn/echo/exit/signal）·
    // 该文件在位是 GUI smoke 之外的自动兜底前置。
    let conpty_test = repo_root()
        .join("crates")
        .join("core")
        .join("tests")
        .join("pty_windows_conpty_integration.rs");
    assert!(
        conpty_test.exists(),
        "ConPTY 自动化覆盖文件应在位（task-2.2）· {}",
        conpty_test.display()
    );
}

/// TEST-6.2.3（SCEN-6.2.4 / AC4）：三平台矩阵零回归——Unix bundle target 仍保留
/// （为 Windows 加 nsis/msi 不得删除 mac/Linux 的 dmg/appimage/deb）。
#[test]
fn test_6_2_3_unix_bundle_targets_preserved_zero_regression() {
    let conf = load_tauri_conf();
    let names = bundle_target_names(&conf);
    for expected in ["dmg", "appimage", "deb"] {
        assert!(
            names.iter().any(|n| n == expected),
            "三平台零回归：bundle.targets 须仍含 {expected}（不为 Windows 牺牲 Unix）· 实际 {names:?}"
        );
    }
}

/// TEST-6.2.4（AC2/AC3 前置不变量 · 跨 phase 聚合）：Windows 默认 shell 绝非 Unix 路径
/// （否则 ConPTY spawn 立即失败 · critical UX path 起步即崩）· Git status debounce 不漂移（200ms）。
#[test]
fn test_6_2_4_windows_adaptation_invariants() {
    // 跨 phase 不变量 1（task-1.3 / ADR-003）：当前平台默认 shell 合法。
    let default_shell = vibestation_core::AppSettings::default().default_shell;
    #[cfg(target_os = "windows")]
    {
        assert!(
            !default_shell.starts_with("/bin/"),
            "Windows 默认 shell 绝不应是 Unix 路径（/bin/* 在 Windows 不存在 → ConPTY spawn 立即失败）· 实际={default_shell}"
        );
        assert_eq!(
            default_shell, "cmd.exe",
            "Windows 默认 shell 占位应为 cmd.exe（ADR-003 · 运行期 resolve 探测 pwsh→powershell→cmd）"
        );
    }
    #[cfg(target_os = "macos")]
    assert_eq!(
        default_shell, "/bin/zsh",
        "macOS 默认 shell 零回归须保持 /bin/zsh"
    );
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    assert_eq!(
        default_shell, "/bin/bash",
        "Linux 默认 shell 零回归须保持 /bin/bash"
    );

    // 跨 phase 不变量 2（task-3.4 / PRD §Constraints）：Git status fs_watch debounce = 200ms
    // （critical UX path S4 "200ms 内刷新"的来源常量 · 三平台一致 · 不得漂移）。
    assert_eq!(
        vibestation_core::GIT_STATUS_WATCH_DEBOUNCE,
        std::time::Duration::from_millis(200),
        "Git status watch debounce 须为 200ms（critical UX path S4 性能门）"
    );

    // 跨 phase 不变量 3：tauri.conf 路径解析成功（窗口配置可加载 · 起窗前置）。
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tauri.conf.json")
            .exists(),
        "tauri.conf.json 须存在（起窗前置）"
    );
}
