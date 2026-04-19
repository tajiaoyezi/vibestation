// SPIKE-01 冷启动埋点：从 `run()` 进入到窗口 ready 的毫秒差值
// 打到 stderr 便于 shell 脚本抓取；生产代码不会保留这段埋点

use std::time::Instant;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let boot_start = Instant::now();
    eprintln!("[SPIKE-01] boot_start t=0ms");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |_app| {
            let elapsed_ms = boot_start.elapsed().as_millis();
            eprintln!("[SPIKE-01] window_ready t={}ms", elapsed_ms);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
