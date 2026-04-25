#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn test_clipboard(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    let clipboard = app.clipboard();
    clipboard.write_text("你好 Vibestation 测试".to_string())
        .map_err(|e| format!("write failed: {}", e))?;
    let text = clipboard.read_text()
        .map_err(|e| format!("read failed: {}", e))?
        .unwrap_or_default();
    Ok(text)
}

#[tauri::command]
fn test_fs() -> Result<String, String> {
    use std::path::PathBuf;
    let home = dirs::home_dir().ok_or("no home dir")?;
    let path = home.join("spike-test.txt");
    std::fs::write(&path, b"\xe6\xb5\x8b\xe8\xaf\x95\xe5\x86\x85\xe5\xae\xb9\xe4\xb8\xad\xe6\x96\x87")
        .map_err(|e| format!("write failed: {}", e))?;
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("read failed: {}", e))?;
    let _ = std::fs::remove_file(&path);
    Ok(content)
}

#[tauri::command]
fn test_dialog() -> Result<String, String> {
    Ok("dialog plugin loaded".to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![test_clipboard, test_fs, test_dialog])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
