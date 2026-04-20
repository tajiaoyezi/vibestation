//! Vibestation Tauri 启动层
//!
//! MVP-02：workspace CRUD IPC 命令 + greet 保留作为版本自检。
//! 存储：rusqlite via r2d2 连接池 · SPIKE-04.5 B.1-5 全过。

use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use vibestation_core::{
    AppSettingsStore, LayoutState, LayoutStore, TabCloseRequest, TabCreateRequest, TabListResponse,
    TabRenameRequest, TabState, TabsDao, WorkspaceMetadata, WorkspaceStore,
};

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

/// Initialize the workspace database connection pool.
///
/// **安全**：DB 路径由 backend 自取 `app_local_data_dir()` · 不接受 frontend 传参。
/// 防止恶意 frontend 代码通过 path traversal 写入任意目录（H1 修复 · 主 agent review · session 10）。
#[tauri::command]
fn workspace_init(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("cannot resolve app_local_data_dir: {e}"))?;
    let db_path = dir.join("vibestation.db");
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
    WorkspaceStore::touch(pool, &id).map_err(|e| e.to_string())?;
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

#[tauri::command]
fn layout_save(
    state: State<'_, AppState>,
    workspace_id: String,
    layout_state: LayoutState,
) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    LayoutStore::save(pool, &workspace_id, &layout_state).map_err(|e| e.to_string())
}

#[tauri::command]
fn layout_load(state: State<'_, AppState>, workspace_id: String) -> Result<LayoutState, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    LayoutStore::load(pool, &workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn theme_get(state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    match AppSettingsStore::get(pool, "theme") {
        Ok(v) => Ok(v),
        Err(vibestation_core::app_settings::SettingsError::NotFound(_)) => Ok("auto".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn theme_set(state: State<'_, AppState>, theme: String) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    AppSettingsStore::set(pool, "theme", &theme).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_list(state: State<'_, AppState>, workspace_id: String) -> Result<TabListResponse, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let tabs = TabsDao::list_by_workspace(pool, &workspace_id).map_err(|e| e.to_string())?;
    Ok(TabListResponse { tabs })
}

#[tauri::command]
fn tab_create(state: State<'_, AppState>, req: TabCreateRequest) -> Result<TabState, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    TabsDao::create(pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_close(state: State<'_, AppState>, req: TabCloseRequest) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    TabsDao::delete(pool, &req.tab_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_rename(state: State<'_, AppState>, req: TabRenameRequest) -> Result<TabState, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    TabsDao::rename(pool, &req.tab_id, &req.name).map_err(|e| e.to_string())?;
    TabsDao::get(pool, &req.tab_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_scrollback_fetch(
    state: State<'_, AppState>,
    tab_id: String,
    offset: u32,
    limit: u32,
) -> Result<Vec<String>, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    TabsDao::scrollback_fetch(pool, &tab_id, offset as usize, limit as usize)
        .map_err(|e| e.to_string())
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
            layout_save,
            layout_load,
            theme_get,
            theme_set,
            tab_list,
            tab_create,
            tab_close,
            tab_rename,
            tab_scrollback_fetch,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
