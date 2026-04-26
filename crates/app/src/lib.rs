//! Vibestation Tauri 启动层
//!
//! MVP-02：workspace CRUD IPC 命令 + greet 保留作为版本自检。
//! 存储：rusqlite via r2d2 连接池 · SPIKE-04.5 B.1-5 全过。

mod fix_path_env;
mod menu;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};
#[allow(unused_imports)]
use vibestation_core::panes;
use vibestation_core::{
    pane_pty, pane_service, telemetry, AppSettings, AppSettingsStore, AppVersionInfo, CommitDetail,
    DiffRequest, DiffResponse, DiffService, GitConfigIdentity, GitLogQueryRequest,
    GitLogQueryResponse, GitLogReader, GitOpsService, GitStatusCollapseRequest,
    GitStatusPanelSettings, GitStatusRequest, GitStatusResponse, GitStatusService,
    GitStatusWatcher, LayoutApplyRequest, LayoutState, LayoutStore, PaneCloseRequest,
    PaneCreateRequest, PaneFocusRequest, PaneInitRequest, PaneListResponse, PanePtyEvent,
    PanePtySpawnRequest, PtyEvent, PtyEventReceiver, PtyManager, PtySpawnRequest,
    SetGitIdentityRequest, SettingsUpdateRequest, SplitRatioUpdateRequest, StageRequest,
    TabCloseRequest, TabCreateRequest, TabListResponse, TabRenameRequest, TabState, TabsDao,
    TelemetryOptInRequest, TelemetryStatus, UnstageRequest, WorkspaceMetadata, WorkspaceStore,
};

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

const GIT_STATUS_CHANGED_EVENT: &str = "git_status:changed";

struct AppState {
    pool: Mutex<Option<DbPool>>,
    git_status: GitStatusWatchManager,
    pty: PtyManager,
    /// MVP-05 Phase B · 独立 PTY manager for pane（独立 sessions HashMap +
    /// 独立 reader/scrollback thread · 避免 tab/pane id collision）
    pane_pty: PtyManager,
}

struct GitStatusSubscription {
    subscribers: usize,
    _watcher: GitStatusWatcher,
}

struct GitStatusWatchManager {
    subscriptions: Mutex<HashMap<String, GitStatusSubscription>>,
}

impl GitStatusWatchManager {
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

        subscriptions.insert(
            workspace_id.clone(),
            GitStatusSubscription {
                subscribers: 1,
                _watcher: spawn_git_status_watcher(app, workspace_id, repo_path)?,
            },
        );
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
///
/// MVP-10 Phase B 副作用：从 DB 读取 telemetry_opt_in · 同步到 telemetry runtime atomic
/// （panic hook + capture_crash_report 用 · §B.4 实时生效）。
#[tauri::command]
fn workspace_init(state: State<'_, AppState>, app: AppHandle) -> Result<String, String> {
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("cannot resolve app_local_data_dir: {e}"))?;
    let db_path = dir.join("vibestation.db");
    let pool = vibestation_core::db::open_pool(&db_path).map_err(|e| e.to_string())?;
    state.pty.set_pool(pool.clone());

    // MVP-10 Phase B · 同步 telemetry runtime opt-in atomic（panic hook 用）
    let opt_in = AppSettingsStore::get_all(&pool).telemetry_opt_in;
    telemetry::set_runtime_opt_in(opt_in);

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
fn default_shell_get(state: State<'_, AppState>) -> Result<String, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let shell = vibestation_core::resolve_default_shell(Some(pool));
    Ok(shell)
}

#[tauri::command]
fn default_shell_set(state: State<'_, AppState>, shell: String) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    vibestation_core::check_shell_exists(&shell).map_err(|e| e.to_string())?;
    AppSettingsStore::set(pool, "default_shell", &shell).map_err(|e| e.to_string())
}

#[tauri::command]
fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    Ok(AppSettingsStore::get_all(pool))
}

#[tauri::command]
fn settings_update(
    state: State<'_, AppState>,
    app: AppHandle,
    req: SettingsUpdateRequest,
) -> Result<AppSettings, String> {
    // MVP-10 §B.4 · 若本次 update 改了 telemetry_opt_in · 同步 runtime atomic
    // （Privacy 面板 toggle 走的是 settings_update 路径 · 不走 telemetry_opt_in_set
    // 必须在此处也同步 · 否则 capture_crash_report 拒绝发送的检查会 stale）
    let opt_in_change = req.telemetry_opt_in;

    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    AppSettingsStore::update(pool, &req).map_err(|e| e.to_string())?;
    drop(guard);
    let updated = {
        let guard = state.pool.lock().map_err(|e| e.to_string())?;
        let pool = guard.as_ref().ok_or("database not initialized")?;
        AppSettingsStore::get_all(pool)
    };

    if let Some(opt_in) = opt_in_change {
        telemetry::set_runtime_opt_in(Some(opt_in));
    }

    let _ = app.emit("settings_changed", &updated);
    Ok(updated)
}

// ─── MVP-10 Phase B · Telemetry IPC（3 commands · ADR-015 accepted） ───

/// 持久化 telemetry opt-in 决策（首次启动对话框 / 设置面板 toggle）
///
/// 等价于 `settings_update({ telemetry_opt_in: req.opt_in })` · 单字段语义化封装
/// 让前端意图清晰 · 同时复用 settings_changed event 通知所有订阅方。
///
/// 副作用：同步 telemetry runtime atomic（§B.4 · opt-out 立即停止发送）。
#[tauri::command]
fn telemetry_opt_in_set(
    state: State<'_, AppState>,
    app: AppHandle,
    req: TelemetryOptInRequest,
) -> Result<TelemetryStatus, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let update = SettingsUpdateRequest {
        telemetry_opt_in: Some(req.opt_in),
        ..Default::default()
    };
    AppSettingsStore::update(pool, &update).map_err(|e| e.to_string())?;
    drop(guard);

    // 同步 runtime atomic（§B.4 实时生效 · panic hook 共享）
    telemetry::set_runtime_opt_in(Some(req.opt_in));

    // 重新拉全量 settings · emit settings_changed 让所有订阅方刷新
    let settings = {
        let guard = state.pool.lock().map_err(|e| e.to_string())?;
        let pool = guard.as_ref().ok_or("database not initialized")?;
        AppSettingsStore::get_all(pool)
    };
    let _ = app.emit("settings_changed", &settings);

    Ok(telemetry::build_status(settings.telemetry_opt_in))
}

/// 读取 telemetry 当前状态（opt-in 值 + endpoint 描述 + SDK 是否已 init + 数据收集摘要）
#[tauri::command]
fn telemetry_status_get(state: State<'_, AppState>) -> Result<TelemetryStatus, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let opt_in = AppSettingsStore::get_all(pool).telemetry_opt_in;
    Ok(telemetry::build_status(opt_in))
}

/// 读取应用版本信息（version + os_type + build_target · Privacy 面板调试展示）
#[tauri::command]
fn app_version_get() -> AppVersionInfo {
    telemetry::build_version_info()
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

// ─── MVP-05 Phase B · Pane PTY IPC（5 commands · 独立命名空间 §H.6 锁 A） ───

#[tauri::command]
fn pane_pty_spawn(state: State<'_, AppState>, req: PanePtySpawnRequest) -> Result<(), String> {
    pane_pty::spawn(&state.pane_pty, req).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_pty_stdin(state: State<'_, AppState>, pane_id: String, data: String) -> Result<(), String> {
    pane_pty::stdin(&state.pane_pty, &pane_id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_pty_resize(
    state: State<'_, AppState>,
    pane_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    pane_pty::resize(&state.pane_pty, &pane_id, cols, rows).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_pty_signal(
    state: State<'_, AppState>,
    pane_id: String,
    signal: String,
) -> Result<(), String> {
    pane_pty::signal(&state.pane_pty, &pane_id, &signal).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_pty_kill(state: State<'_, AppState>, pane_id: String) -> Result<(), String> {
    pane_pty::kill(&state.pane_pty, &pane_id)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ─── MVP-05 Phase B Step 2 · Pane layout IPC（6 commands · §H.3 atomicity） ───

#[tauri::command]
fn pane_init_for_tab(
    state: State<'_, AppState>,
    req: PaneInitRequest,
) -> Result<PaneListResponse, String> {
    let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = pool_guard
        .as_ref()
        .ok_or("workspace pool not initialized")?;
    pane_service::apply_pane_init_for_tab(pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_split(
    state: State<'_, AppState>,
    req: PaneCreateRequest,
) -> Result<PaneListResponse, String> {
    let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = pool_guard
        .as_ref()
        .ok_or("workspace pool not initialized")?;
    pane_service::apply_pane_split(pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_close(
    state: State<'_, AppState>,
    req: PaneCloseRequest,
) -> Result<PaneListResponse, String> {
    let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = pool_guard
        .as_ref()
        .ok_or("workspace pool not initialized")?;
    pane_service::apply_pane_close(pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_focus(
    state: State<'_, AppState>,
    req: PaneFocusRequest,
) -> Result<PaneListResponse, String> {
    let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = pool_guard
        .as_ref()
        .ok_or("workspace pool not initialized")?;
    pane_service::apply_pane_focus(pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_layout_apply(
    state: State<'_, AppState>,
    req: LayoutApplyRequest,
) -> Result<PaneListResponse, String> {
    let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = pool_guard
        .as_ref()
        .ok_or("workspace pool not initialized")?;
    pane_service::apply_layout_preset(pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
fn pane_split_ratio_update(
    state: State<'_, AppState>,
    req: SplitRatioUpdateRequest,
) -> Result<PaneListResponse, String> {
    let pool_guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = pool_guard
        .as_ref()
        .ok_or("workspace pool not initialized")?;
    pane_service::apply_split_ratio_update(pool, &req).map_err(|e| e.to_string())
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

fn emit_pane_pty_events(app: AppHandle, events: PtyEventReceiver) {
    while let Ok(event) = events.recv() {
        let mapped = pane_pty::map_event(event);
        let result = match mapped {
            PanePtyEvent::Stdout(payload) => app.emit("pane_pty_stdout", payload),
            PanePtyEvent::Exited(payload) => app.emit("pane_pty_exited", payload),
        };

        if let Err(error) = result {
            eprintln!("[mvp-05] emit pane PTY event failed: {error}");
        }
    }
}

fn git_status_repo_path(pool: &DbPool, workspace_id: &str) -> Result<PathBuf, String> {
    let workspace = WorkspaceStore::get_by_id(pool, workspace_id).map_err(|e| e.to_string())?;
    Ok(PathBuf::from(workspace.path))
}

fn spawn_git_status_watcher(
    app: AppHandle,
    workspace_id: String,
    repo_path: PathBuf,
) -> Result<GitStatusWatcher, String> {
    let req = GitStatusRequest {
        workspace_id: workspace_id.clone(),
    };
    let mut last_status = GitStatusService::query(&repo_path, &req).ok();

    GitStatusWatcher::spawn(repo_path.clone(), move || {
        match GitStatusService::refresh(&repo_path, &req) {
            Ok(response) => {
                let changed = last_status
                    .as_ref()
                    .is_none_or(|previous| !previous.equivalent(&response));
                if changed {
                    if let Err(error) = app.emit(
                        GIT_STATUS_CHANGED_EVENT,
                        response.to_event(workspace_id.clone()),
                    ) {
                        eprintln!("[mvp-08] emit git status change failed: {error}");
                    }
                    last_status = Some(response);
                }
            }
            Err(error) => {
                eprintln!("[mvp-08] git status fs watch refresh failed: {error}");
            }
        }
    })
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

#[tauri::command]
fn git_ops_stage_files(
    state: State<'_, AppState>,
    req: StageRequest,
) -> Result<vibestation_core::StageResult, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitOpsService::stage_files(&repo_path, &req.file_paths.into_iter().collect::<Vec<_>>())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn git_ops_unstage_files(state: State<'_, AppState>, req: UnstageRequest) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitOpsService::unstage_files(&repo_path, &req.file_paths.into_iter().collect::<Vec<_>>())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn git_ops_commit(
    state: State<'_, AppState>,
    req: vibestation_core::CommitRequest,
) -> Result<vibestation_core::CommitResponse, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitOpsService::commit(&repo_path, &req.message, req.amend).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_ops_read_identity(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<GitConfigIdentity, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace = WorkspaceStore::get_by_id(pool, &workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitOpsService::read_git_identity(&repo_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn git_ops_set_identity(
    state: State<'_, AppState>,
    req: SetGitIdentityRequest,
) -> Result<(), String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let workspace =
        WorkspaceStore::get_by_id(pool, &req.workspace_id).map_err(|e| e.to_string())?;
    let repo_path = std::path::PathBuf::from(&workspace.path);
    GitOpsService::set_git_identity(&repo_path, &req.name, &req.email, &req.scope)
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
fn configure_title_bar<R: tauri::Runtime>(app: &tauri::App<R>) {
    let Some(window) = app.get_webview_window("main") else {
        eprintln!("[mvp-11] main window not found for title bar setup");
        return;
    };

    if let Err(error) = window.set_title_bar_style(tauri::TitleBarStyle::Overlay) {
        eprintln!("[mvp-11] title bar overlay setup failed: {error}");
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_title_bar<R: tauri::Runtime>(_app: &tauri::App<R>) {}

/// 安装 panic hook · 必须在 Tauri builder 启动前调用（一次性 · 进程级）
///
/// MVP-10 §B.4 acceptance：
/// - opt-in == false 或 SDK 未 init → panic 时只走原 hook（stderr 打印）· 0 网络
/// - opt-in == true 且 SDK 已 init → 原 hook + capture_crash_report (脱敏 payload)
fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        original(info);
        let payload = telemetry::capture_panic(&format!("{info}"));
        // capture_crash_report 内部检查 should_send_telemetry · opt-out 时 no-op
        telemetry::capture_crash_report(&payload);
    }));
}

/// 尝试从环境变量初始化 Sentry SDK（按 ADR-015 §决策）
///
/// DSN 来源（优先级）：
/// 1. `VIBESTATION_SENTRY_DSN` 环境变量（CI / 本地未提交配置 / 自托管 endpoint）
/// 2. 缺失 → SDK 不初始化 · capture 全部 no-op
///
/// `environment` 来源 `VIBESTATION_TELEMETRY_ENV` · 缺失默认 "production"。
/// 注意：SDK 初始化和 opt-in 状态独立 · runtime atomic 才决定是否实际发送（§B.4）。
fn try_init_sentry_from_env() {
    let Ok(dsn) = std::env::var("VIBESTATION_SENTRY_DSN") else {
        return;
    };
    if dsn.trim().is_empty() {
        return;
    }
    let environment =
        std::env::var("VIBESTATION_TELEMETRY_ENV").unwrap_or_else(|_| "production".to_string());
    if let Err(error) = telemetry::init_sentry(&dsn, &environment) {
        eprintln!("[mvp-10] sentry init skipped: {error}");
    }
}

/// Tauri 应用主入口 · 被 `src/main.rs` 调用。
///
/// # Panics
/// 若 Tauri 初始化失败（窗口 / 插件加载异常）则 panic · 由 Tauri 默认错误处理上浮。
pub fn run() {
    let _ = fix_path_env::fix();

    // MVP-10 Phase B · panic hook + Sentry SDK 初始化（在 Tauri builder 之前）
    install_panic_hook();
    try_init_sentry_from_env();

    let pty = PtyManager::new();
    let pty_events = pty
        .take_event_receiver()
        .expect("PTY event receiver should be taken exactly once");
    let pane_pty = PtyManager::new();
    let pane_pty_events = pane_pty
        .take_event_receiver()
        .expect("Pane PTY event receiver should be taken exactly once");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            pool: Mutex::new(None),
            git_status: GitStatusWatchManager::new(),
            pty,
            pane_pty,
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
            default_shell_get,
            default_shell_set,
            settings_get,
            settings_update,
            telemetry_opt_in_set,
            telemetry_status_get,
            app_version_get,
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
            pane_pty_spawn,
            pane_pty_stdin,
            pane_pty_resize,
            pane_pty_signal,
            pane_pty_kill,
            pane_init_for_tab,
            pane_split,
            pane_close,
            pane_focus,
            pane_layout_apply,
            pane_split_ratio_update,
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
            git_ops_stage_files,
            git_ops_unstage_files,
            git_ops_commit,
            git_ops_read_identity,
            git_ops_set_identity,
            menu::menu_show_tab,
            menu::menu_show_terminal,
            menu::menu_register_shortcuts,
            menu::menu_item_clicked,
        ])
        .setup(move |app| {
            configure_title_bar(app);

            let handle = app.handle().clone();
            thread::Builder::new()
                .name("vibestation-pty-events".to_string())
                .spawn(move || emit_pty_events(handle, pty_events))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

            let pane_handle = app.handle().clone();
            thread::Builder::new()
                .name("vibestation-pane-pty-events".to_string())
                .spawn(move || emit_pane_pty_events(pane_handle, pane_pty_events))
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

            menu::setup_menu_events(app.handle());
            if let Err(e) = menu::app_menu(app.handle())
                .and_then(|m| app.handle().set_menu(m).map_err(|e| e.to_string()))
            {
                eprintln!("[mvp-11] app menu setup failed: {e}");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri 应用启动失败");
}
