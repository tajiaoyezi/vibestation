//! Vibestation Tauri 启动层
//!
//! MVP-02：workspace CRUD IPC 命令 + greet 保留作为版本自检。
//! 存储：rusqlite via r2d2 连接池 · SPIKE-04.5 B.1-5 全过。

mod fix_path_env;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use vibestation_core::{
    AppSettingsStore, CommitDetail, DiffRequest, DiffResponse, DiffService, GitLogQueryRequest,
    GitLogQueryResponse, GitLogReader, GitStatusCollapseRequest, GitStatusPanelSettings,
    GitStatusRequest, GitStatusResponse, GitStatusService, LayoutState, LayoutStore, PtyEvent,
    PtyEventReceiver, PtyManager, PtySpawnRequest, TabCloseRequest, TabCreateRequest,
    TabListResponse, TabRenameRequest, TabState, TabsDao, WorkspaceMetadata, WorkspaceStore,
};

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

const GIT_STATUS_UPDATED_EVENT: &str = "git_status_updated";
const GIT_STATUS_AUTO_REFRESH_INTERVAL: Duration = Duration::from_secs(2);

struct AppState {
    pool: Mutex<Option<DbPool>>,
    git_status: GitStatusAutoRefreshManager,
    pty: PtyManager,
}

struct GitStatusSubscription {
    subscribers: usize,
    stop: Arc<AtomicBool>,
}

struct GitStatusAutoRefreshManager {
    subscriptions: Mutex<HashMap<String, GitStatusSubscription>>,
}

impl GitStatusAutoRefreshManager {
    fn new() -> Self {
        Self {
            subscriptions: Mutex::new(HashMap::new()),
        }
    }

    fn subscribe(
        &self,
        app: AppHandle,
        workspace_id: String,
        repo_path: PathBuf,
    ) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = subscriptions.get_mut(&workspace_id) {
            existing.subscribers += 1;
            return Ok(());
        }

        let stop = Arc::new(AtomicBool::new(false));
        subscriptions.insert(
            workspace_id.clone(),
            GitStatusSubscription {
                subscribers: 1,
                stop: stop.clone(),
            },
        );
        drop(subscriptions);

        spawn_git_status_auto_refresh(app, workspace_id, repo_path, stop)?;
        Ok(())
    }

    fn unsubscribe(&self, workspace_id: &str) -> Result<(), String> {
        let mut subscriptions = self.subscriptions.lock().map_err(|e| e.to_string())?;
        let Some(existing) = subscriptions.get_mut(workspace_id) else {
            return Ok(());
        };

        if existing.subscribers > 1 {
            existing.subscribers -= 1;
            return Ok(());
        }

        existing.stop.store(true, Ordering::Relaxed);
        subscriptions.remove(workspace_id);
        Ok(())
    }
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
    state.pty.set_pool(pool.clone());
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

#[tauri::command]
fn tab_pty_spawn(state: State<'_, AppState>, req: PtySpawnRequest) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let persisted = TabsDao::get(pool, &req.tab_id).map_err(|e| e.to_string())?;
    drop(guard);

    let spawn_req = PtySpawnRequest {
        shell: if req.shell.is_empty() {
            persisted.shell
        } else {
            req.shell
        },
        cwd: if req.cwd.is_empty() {
            persisted.cwd
        } else {
            req.cwd
        },
        ..req
    };

    state.pty.spawn(spawn_req).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_pty_stdin(state: State<'_, AppState>, tab_id: String, data: String) -> Result<(), String> {
    state.pty.stdin(&tab_id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_pty_resize(
    state: State<'_, AppState>,
    tab_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state
        .pty
        .resize(&tab_id, cols, rows)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_pty_signal(
    state: State<'_, AppState>,
    tab_id: String,
    signal: String,
) -> Result<(), String> {
    state
        .pty
        .signal(&tab_id, &signal)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn tab_pty_kill(state: State<'_, AppState>, tab_id: String) -> Result<(), String> {
    state
        .pty
        .kill(&tab_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn emit_pty_events(app: AppHandle, events: PtyEventReceiver) {
    while let Ok(event) = events.recv() {
        let result = match event {
            PtyEvent::Stdout(payload) => app.emit("tab_pty_stdout", payload),
            PtyEvent::Exited(payload) => app.emit("tab_pty_exited", payload),
        };

        if let Err(error) = result {
            eprintln!("[mvp-04] emit PTY event failed: {error}");
        }
    }
}

fn git_status_repo_path(pool: &DbPool, workspace_id: &str) -> Result<PathBuf, String> {
    let workspace = WorkspaceStore::get_by_id(pool, workspace_id).map_err(|e| e.to_string())?;
    Ok(PathBuf::from(workspace.path))
}

fn wait_for_stop(stop: &AtomicBool, duration: Duration) -> bool {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        if stop.load(Ordering::Relaxed) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    stop.load(Ordering::Relaxed)
}

fn spawn_git_status_auto_refresh(
    app: AppHandle,
    workspace_id: String,
    repo_path: PathBuf,
    stop: Arc<AtomicBool>,
) -> Result<(), String> {
    thread::Builder::new()
        .name(format!("vibestation-git-status-{workspace_id}"))
        .spawn(move || {
            let req = GitStatusRequest {
                workspace_id: workspace_id.clone(),
            };
            let mut last_status = GitStatusService::query(&repo_path, &req).ok();

            loop {
                if wait_for_stop(stop.as_ref(), GIT_STATUS_AUTO_REFRESH_INTERVAL) {
                    break;
                }

                match GitStatusService::refresh(&repo_path, &req) {
                    Ok(response) => {
                        let changed = last_status
                            .as_ref()
                            .is_none_or(|previous| !previous.equivalent(&response));
                        if changed {
                            if let Err(error) = app.emit(
                                GIT_STATUS_UPDATED_EVENT,
                                response.to_event(workspace_id.clone()),
                            ) {
                                eprintln!("[mvp-08] emit git status update failed: {error}");
                            }
                            last_status = Some(response);
                        }
                    }
                    Err(error) => {
                        eprintln!("[mvp-08] git status auto-refresh failed: {error}");
                    }
                }
            }
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn git_log_query(
    state: State<'_, AppState>,
    req: GitLogQueryRequest,
) -> Result<GitLogQueryResponse, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitLogReader::query(&repo_path, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_log_commit_detail(
    state: State<'_, AppState>,
    workspace_id: String,
    sha: String,
) -> Result<CommitDetail, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace = WorkspaceStore::get_by_id(pool, &workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitLogReader::commit_detail(&repo_path, &sha).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_log_cache_clear() -> Result<(), String> {
    GitLogReader::cache_clear().map_err(|e| e.to_string())
}

#[tauri::command]
fn diff_compute(state: State<'_, AppState>, req: DiffRequest) -> Result<DiffResponse, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    DiffService::compute(&repo_path, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn diff_get_settings(state: State<'_, AppState>, workspace_id: String) -> Result<String, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    DiffService::get_view_mode(pool, &workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn diff_set_view_mode(
    state: State<'_, AppState>,
    workspace_id: String,
    view_mode: String,
) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    DiffService::set_view_mode(pool, &workspace_id, &view_mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_status_query(
    state: State<'_, AppState>,
    req: GitStatusRequest,
) -> Result<GitStatusResponse, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitStatusService::query(&repo_path, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_status_refresh(
    state: State<'_, AppState>,
    req: GitStatusRequest,
) -> Result<GitStatusResponse, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitStatusService::refresh(&repo_path, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_status_get_settings(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<GitStatusPanelSettings, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    GitStatusService::get_panel_settings(pool, &workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_status_set_group_collapsed(
    state: State<'_, AppState>,
    req: GitStatusCollapseRequest,
) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    GitStatusService::set_group_collapsed(pool, &req.workspace_id, req.group, req.collapsed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn git_status_subscribe(
    state: State<'_, AppState>,
    app: AppHandle,
    workspace_id: String,
) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let repo_path = git_status_repo_path(pool, &workspace_id)?;
    drop(guard);

    GitStatusService::subscribe(&workspace_id);
    state
        .git_status
        .subscribe(app, workspace_id, repo_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn git_status_unsubscribe(state: State<'_, AppState>, workspace_id: String) -> Result<(), String> {
    GitStatusService::unsubscribe(&workspace_id);
    state.git_status.unsubscribe(&workspace_id)?;
    Ok(())
}

/// Tauri 应用主入口 · 被 `src/main.rs` 调用。
///
/// # Panics
/// 若 Tauri 初始化失败（窗口 / 插件加载异常）则 panic · 由 Tauri 默认错误处理上浮。
pub fn run() {
    let _ = fix_path_env::fix();
    let pty = PtyManager::new();
    let pty_events = pty
        .take_event_receiver()
        .expect("PTY event receiver should be taken exactly once");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            pool: Mutex::new(None),
            git_status: GitStatusAutoRefreshManager::new(),
            pty,
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
            tab_pty_spawn,
            tab_pty_stdin,
            tab_pty_resize,
            tab_pty_signal,
            tab_pty_kill,
            git_log_query,
            git_log_commit_detail,
            git_log_cache_clear,
            diff_compute,
            diff_get_settings,
            diff_set_view_mode,
            git_status_query,
            git_status_refresh,
            git_status_get_settings,
            git_status_set_group_collapsed,
            git_status_subscribe,
            git_status_unsubscribe,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            thread::Builder::new()
                .name("vibestation-pty-events".to_string())
                .spawn(move || emit_pty_events(handle, pty_events))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
