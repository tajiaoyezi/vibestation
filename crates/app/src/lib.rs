//! Vibestation Tauri 启动层
//!
//! MVP-02：workspace CRUD IPC 命令 + greet 保留作为版本自检。
//! 存储：rusqlite via r2d2 连接池 · SPIKE-04.5 B.1-5 全过。

use std::sync::Mutex;
use tauri::State;
use vibestation_core::{WorkspaceMetadata, WorkspaceStore};

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

struct AppState {
    pool: Mutex<Option<DbPool>>,
}

#[tauri::command]
fn greet() -> String {
    format!(
        "{} · v{}",
        vibestation_core::greet(),
        vibestation_core::VERSION
    )
}

#[tauri::command]
fn workspace_init(state: State<'_, AppState>, db_dir: String) -> Result<String, String> {
    let db_path = std::path::PathBuf::from(&db_dir).join("vibestation.db");
    let pool = vibestation_core::db::open_pool(&db_path).map_err(|e| e.to_string())?;
    let mut guard = state.pool.lock().map_err(|e| e.to_string())?;
    *guard = Some(pool);
    Ok("ok".to_string())
}

#[tauri::command]
fn workspace_list(state: State<'_, AppState>) -> Result<Vec<WorkspaceMetadata>, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    WorkspaceStore::list(pool).map_err(|e| e.to_string())
}

#[tauri::command]
fn workspace_create(
    state: State<'_, AppState>,
    path: String,
    name: Option<String>,
) -> Result<WorkspaceMetadata, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    WorkspaceStore::create(pool, &path, name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn workspace_open(state: State<'_, AppState>, id: String) -> Result<WorkspaceMetadata, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    WorkspaceStore::touch(pool, &id)?;
    WorkspaceStore::get_by_id(pool, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn workspace_delete(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    WorkspaceStore::delete(pool, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn workspace_exists(state: State<'_, AppState>, path: String) -> Result<bool, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    WorkspaceStore::exists_at_path(pool, &path).map_err(|e| e.to_string())
}

/// Tauri 应用主入口 · 被 `src/main.rs` 调用。
///
/// # Panics
/// 若 Tauri 初始化失败（窗口 / 插件加载异常）则 panic · 由 Tauri 默认错误处理上浮。
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            pool: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            workspace_init,
            workspace_list,
            workspace_create,
            workspace_open,
            workspace_delete,
            workspace_exists,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}