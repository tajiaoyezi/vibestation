// SPIKE-02 冷启动埋点 + Plugin smoke test
// - 保留 SPIKE-01 的冷启动测量（Instant::now() → setup callback 差值）
// - 加载 3 个 plugin：clipboard-manager / fs / opener
// - Note：updater plugin 需要 Apple Developer Program 签名 key · 归 SPIKE-06 一起做
//   本 Spike 验证 updater 以外的 §3.1.1 其余判据

use std::time::Instant;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let boot_start = Instant::now();
    eprintln!("[SPIKE-02] boot_start t=0ms");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |_app| {
            let elapsed_ms = boot_start.elapsed().as_millis();
            eprintln!("[SPIKE-02] window_ready t={}ms", elapsed_ms);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
