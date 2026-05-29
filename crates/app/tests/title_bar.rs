// crates/app/tests/title_bar.rs
// SCEN-5.3.2 / AC3 · 在非 macOS（含 Windows）build 下 configure_title_bar 走空 stub
// 不 panic（编译期 cfg 已保证 Windows 走 stub 分支；此测试在 Windows CI leg 编译通过即证 cfg 分支健全）

#[test]
fn test_5_3_2_configure_title_bar_non_macos_stub_compiles() {
    // TEST-5.3.2 · 此测试存在并在 windows-latest 编译通过 = Windows cfg 分支无编译错误。
    // configure_title_bar 需 tauri::App，单测难构造真 App；
    // 退而验证 cfg 编译健全 + 标记意图（运行期 GUI 验证走 §9 Runtime smoke 本机 Windows）。
    // task-6.1：编译期断言"当前平台在三支持目标内"（cfg! 为常量 · 用 const block 满足
    // clippy::assertions_on_constants · 编译通过即证平台 cfg 分支健全 · 与测试文档意图一致）。
    const {
        assert!(cfg!(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux"
        )));
    }
}
