//! Vibestation Tauri 启动层
//!
//! Phase A（MVP-01）：启动 Tauri 窗口 · 暴露 `greet` IPC 命令作为 workspace 联通
//! 性自检 · 不含崩溃恢复 / Tool Windows / Tab 等后续 MVP 功能。

#[tauri::command]
fn greet() -> String {
    format!(
        "{} · v{}",
        vibestation_core::greet(),
        vibestation_core::VERSION
    )
}

/// Tauri 应用主入口 · 被 `src/main.rs` 调用。
///
/// # Panics
/// 若 Tauri 初始化失败（窗口 / 插件加载异常）则 panic · 由 Tauri 默认错误处理上浮。
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
