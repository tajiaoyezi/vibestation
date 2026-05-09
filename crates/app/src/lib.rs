//! Vibestation Tauri 启动层
//!
//! MVP-02：workspace CRUD IPC 命令 + greet 保留作为版本自检。
//! 存储：rusqlite via r2d2 连接池 · SPIKE-04.5 B.1-5 全过。

mod fix_path_env;
mod menu;
mod pane_layout_advanced;

use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};
#[allow(unused_imports)]
use vibestation_core::panes;
use crate::pane_layout_advanced::{
    pane_layout_apply_advanced, pane_maximize, pane_navigate, pane_resize_step,
};
use vibestation_core::pty_pool::{PoolConfig, PtyPool, SpawnResult, TakeResult};
use vibestation_core::{
    branch_checkout as core_branch_checkout, branch_create as core_branch_create,
    branch_delete as core_branch_delete, branch_list as core_branch_list,
    branch_switcher_query as core_branch_switcher_query, git_auth_provide as core_git_auth_provide,
    git_fetch_with_events as core_git_fetch, git_merge_abort as core_git_merge_abort,
    git_pull_with_events as core_git_pull, git_push_with_events as core_git_push,
    git_remote_list as core_git_remote_list, pane_pty, pane_service, telemetry, AppSettings,
    AppSettingsStore, AppVersionInfo, AuthRequest, BranchCheckoutRequest, BranchCreateRequest,
    BranchDeleteRequest, BranchError, BranchInfo, BranchListRequest, BranchListResponse,
    BranchSwitchResult, CherryPickRequest, CherryPickStatus, CommitDetail,
    ConflictResolveFileRequest, ConflictedFile, CrashRecoveryState, DiffRequest, DiffResponse,
    DiffService, FetchProgressEvent, FetchRequest, FetchResult, GitConfigIdentity,
    GitLogQueryRequest, GitLogQueryResponse, GitLogReader, GitOpsService, GitStatusCollapseRequest,
    GitStatusPanelSettings, GitStatusRequest, GitStatusResponse, GitStatusService,
    GitStatusWatcher, GitSyncEventHandlers, LayoutApplyAdvancedRequest, LayoutApplyRequest,
    LayoutApplyResult, LayoutState, LayoutStore, MergeRequest, MergeStatus, NetworkOpError,
    OperationDoneEvent, PaneCloseRequest, PaneCreateRequest, PaneFocusRequest, PaneInitRequest,
    PaneListResponse, PaneMaximizeRequest, PaneMaximizeResult, PaneNavigateRequest,
    PaneNavigateResult, PanePtyEvent, PanePtySpawnRequest, PaneResizeStepRequest, PtyEvent,
    PtyEventReceiver, PtyManager, PtySpawnRequest, PullRequest, PullResult, PushProgressEvent,
    PushRequest, PushResult, RebaseControlRequest, RebaseInteractivePlan, RebaseOpError,
    RebaseStartRequest, RebaseStatus, RemoteListRequest, RemoteListResponse, SetGitIdentityRequest,
    SettingsUpdateRequest, SplitRatioUpdateRequest, StageRequest, SwitcherQueryRequest,
    SwitcherSearchResult, TabCloseRequest, TabCreateRequest, TabListResponse, TabRenameRequest,
    TabReorderRequest, TabState, TabsDao, TelemetryOptInRequest, TelemetryStatus, UnstageRequest,
    WorkspaceMetadata, WorkspaceStore,
};

pub type DbPool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;

const GIT_STATUS_CHANGED_EVENT: &str = "git_status:changed";
const BRANCH_CHANGED_EVENT: &str = "git:branch-changed";
const GIT_PUSH_PROGRESS_EVENT: &str = "git:push-progress";
const GIT_FETCH_PROGRESS_EVENT: &str = "git:fetch-progress";
const GIT_OPERATION_DONE_EVENT: &str = "git:operation-done";
const GIT_REBASE_PROGRESS_EVENT: &str = "git:rebase-progress";
const GIT_CONFLICT_DETECTED_EVENT: &str = "git:conflict-detected";
/// MVP-16 Phase C · app 启动 / workspace 切换时检测到 in-progress rebase / merge / cherrypick 时
/// emit 此 event · payload 为 `RebaseCrashRecoveryEvent` · 前端据此渲染 recovery banner。
const GIT_CRASH_RECOVERY_EVENT: &str = "git:crash-recovery-detected";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BranchChangedEvent {
    workspace_id: String,
    branches: Vec<BranchInfo>,
    head: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RebaseProgressEvent {
    workspace_id: String,
    current_step: u32,
    total_steps: u32,
    current_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConflictDetectedEvent {
    workspace_id: String,
    operation: String,
    conflicting_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RebaseOperationDoneEvent {
    workspace_id: String,
    operation: String,
    success: bool,
    message: String,
}

/// MVP-16 Phase C · 启动 / workspace 切换检测到上次未完成操作时 emit。
/// 字段对齐 `vibestation_core::CrashRecoveryState` · 加 `workspace_id` 用于多 workspace 路由。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RebaseCrashRecoveryEvent {
    workspace_id: String,
    /// "rebase" / "merge" / "cherrypick" · None 不会被 emit（only emit 当 in_progress=true）
    operation: Option<String>,
    /// 当前分支 · detached HEAD 时为 None
    branch: Option<String>,
    current_step: u32,
    total_steps: u32,
}

struct AppState {
    pool: Mutex<Option<DbPool>>,
    git_status: GitStatusWatchManager,
    pty: Arc<PtyManager>,
    /// MVP-05 Phase B · 独立 PTY manager for pane（独立 sessions HashMap +
    /// 独立 reader/scrollback thread · 避免 tab/pane id collision）
    pane_pty: Arc<PtyManager>,
    /// MVP-20 · PTY 预热池（与 pty 配套 · 共享同一 PtyManager 的 reader thread）
    pty_pool: Arc<PtyPool>,
    /// MVP-20 · PTY 预热池（与 pane_pty 配套 · 独立池）
    pane_pty_pool: Arc<PtyPool>,
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
    apply_rebase_state_migration(&pool)?;
    state.pty.set_pool(pool.clone());

    // MVP-10 Phase B · 同步 telemetry runtime opt-in atomic（panic hook 用）+
    // 拉一份 settings 准备 emit（workspace_init 是 frontend 启动后第一个能确保 pool
    // 已 init 的时机 · 必须在此 emit settings_changed · 让 frontend store 用真实 DB
    // 值而不是 DEFAULTS · 否则 settings_get race 导致 telemetry modal 即使 DB 有
    // telemetry_opt_in=true 也每次启动重弹）。
    let settings = AppSettingsStore::get_all(&pool);
    telemetry::set_runtime_opt_in(settings.telemetry_opt_in);

    // MVP-20 · 用 DB settings 初始化 PTY 预热池配置 · 触发首次 refill
    // workspace_init 是 frontend 启动后第一个能确保 settings 已 hydrate 的时机 ·
    // 必须在此 apply config + refill_async 让 idle PTY 在用户点 + 之前预备好。
    let pool_config = PoolConfig {
        enabled: settings.pty_pool_enabled,
        target_size: settings.pty_pool_size as u8,
    };
    let default_shell_path = PathBuf::from(&settings.default_shell);
    let home_path = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"));
    state
        .pty_pool
        .apply_config_change(pool_config, default_shell_path.clone());
    state
        .pane_pty_pool
        .apply_config_change(pool_config, default_shell_path.clone());
    if pool_config.enabled {
        state
            .pty_pool
            .refill_async(default_shell_path.clone(), home_path.clone());
        state
            .pane_pty_pool
            .refill_async(default_shell_path, home_path);
    }

    let crash_pool = pool.clone();
    let mut guard = state.pool.lock().map_err(|e| e.to_string())?;
    *guard = Some(pool);
    drop(guard);

    emit_rebase_crash_recovery(&app, &crash_pool);
    let _ = app.emit("settings_changed", &settings);
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
    // 切 / 点击 workspace 时重 detect git 状态 · cover 用户在 non-git workspace 内
    // 跑 `git init` 后 sidebar 不识别的设计 gap（2026-05-04 · v0.1 GA quick win · A 方案）。
    // detect_git 是一次 path walk · ~1-3 ms · 性能可忽略。
    WorkspaceStore::refresh_git(pool, &id).map_err(|e| e.to_string())?;
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

/// 列出系统认证的所有 shell · 供 Settings UI 动态填 dropdown · 防 hardcoded 选项
/// 在用户机器上不存在导致的 "PTY 启动失败" 体验问题（fix24）。
#[tauri::command]
fn available_shells_list() -> Vec<vibestation_core::ShellInfo> {
    vibestation_core::list_available_shells()
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
    // MVP-20 · 检测 PTY 预热池相关字段改动 · 待 update 后调 apply_config_change
    let pool_config_changed = req.pty_pool_enabled.is_some() || req.pty_pool_size.is_some();
    let shell_change = req.default_shell.clone();

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

    // MVP-20 · 同步 PTY 预热池配置（settings 改动实时生效 · spec A7/A8）
    if pool_config_changed {
        let new_config = PoolConfig {
            enabled: updated.pty_pool_enabled,
            target_size: updated.pty_pool_size as u8,
        };
        let shell_path = PathBuf::from(&updated.default_shell);
        state
            .pty_pool
            .apply_config_change(new_config, shell_path.clone());
        state
            .pane_pty_pool
            .apply_config_change(new_config, shell_path);
    }
    // MVP-20 · default shell 变更立即 kill 旧 idle + 预热新 shell（spec A3）
    if let Some(new_shell) = shell_change {
        let shell_path = PathBuf::from(&new_shell);
        state
            .pty_pool
            .handle_default_shell_change(shell_path.clone());
        state.pane_pty_pool.handle_default_shell_change(shell_path);
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

// ─── MVP-06 Phase B · Config Import IPC（4 commands · spec §G.1） ───────

/// 扫描默认路径 · 返回 3 个源（Ghostty / iTerm2 / Alacritty）的扫描结果
///
/// 不读 DB · 纯文件解析 · 失败 graceful（errors 字段记录字段级错误 · 整体不崩）
#[tauri::command]
fn config_import_scan() -> Vec<vibestation_core::ImportScanResult> {
    let home = home_dir_or_root();
    vibestation_core::config_import_scan(&home)
}

/// 构建预览 · 跨源合并 + 全局冲突检测
///
/// `selected_sources` 为空时所有源参与冲突检测（默认）
#[tauri::command]
fn config_import_preview(
    selected_sources: Vec<vibestation_core::ImportSource>,
) -> vibestation_core::ImportPreview {
    let home = home_dir_or_root();
    vibestation_core::config_import_build_preview(&home, &selected_sources)
}

/// 检测冲突（UI 切换勾选时调）
#[tauri::command]
fn config_import_detect_conflicts(
    fields: Vec<vibestation_core::ImportFieldType>,
) -> Vec<vibestation_core::KeyBindingConflict> {
    vibestation_core::config_import_detect_conflicts(&fields)
}

/// 应用导入 · 写 app_settings · emit settings_changed
///
/// graceful：单字段写失败不阻止其他（errors 字段记录）
#[tauri::command]
fn config_import_apply(
    state: State<'_, AppState>,
    app: AppHandle,
    req: vibestation_core::ImportApplyRequest,
) -> Result<vibestation_core::ImportApplyResult, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    let result = vibestation_core::config_import_apply(pool, &req);

    // 写完后拉一份 settings · emit · 让前端 settings store 立即刷新
    // （主题 / 字体 / shell 切换走和 settings_update 一致的 event 通道）
    let updated = AppSettingsStore::get_all(pool);
    drop(guard);

    let _ = app.emit("settings_changed", &updated);
    Ok(result)
}

/// 计算用户 home 目录 · 失败回退根目录（让扫描看到 path_exists=false 而不是 panic）
fn home_dir_or_root() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn apply_rebase_state_migration(pool: &DbPool) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute_batch(include_str!("../migrations/0042_rebase_state.sql"))
        .map_err(|e| e.to_string())
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
fn tab_reorder(
    state: State<'_, AppState>,
    req: TabReorderRequest,
) -> Result<Vec<TabState>, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    let pool = guard.as_ref().ok_or("database not initialized")?;
    TabsDao::reorder(pool, &req).map_err(|e| e.to_string())
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
fn tab_pty_spawn(state: State<'_, AppState>, req: PtySpawnRequest) -> Result<SpawnResult, String> {
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

    // MVP-20 · 优先 PTY 预热池命中 · 命中则 idle PTY 已 rename + cd 注入完成
    // 不命中（pool disable / shell 不匹配 / pool 空）→ 降级 cold spawn
    // BUG-001 fix · 返回 SpawnResult{warm} 让前端进入 ANSI clear filter 模式
    match state.pty_pool.take(&spawn_req) {
        TakeResult::Warm(_) => {
            // 触发后台 refill 补回 idle · 不阻塞当前 IPC return
            state.pty_pool.refill_async(
                PathBuf::from(&spawn_req.shell),
                PathBuf::from(&spawn_req.cwd),
            );
            Ok(SpawnResult { warm: true })
        }
        TakeResult::Cold => {
            state.pty.spawn(spawn_req).map_err(|e| e.to_string())?;
            Ok(SpawnResult { warm: false })
        }
    }
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
fn pane_pty_spawn(
    state: State<'_, AppState>,
    req: PanePtySpawnRequest,
) -> Result<SpawnResult, String> {
    // MVP-20 · 优先 pane PTY 预热池命中 · pool.take 接受 PtySpawnRequest
    // pane_pty::spawn 内部就是把 pane_id 当 tab_id · 这里直接构造 PtySpawnRequest
    // BUG-001 fix · 返回 SpawnResult{warm} 让前端进入 ANSI clear filter 模式
    let spawn_req = PtySpawnRequest {
        tab_id: req.pane_id.clone(),
        shell: req.shell.clone(),
        cwd: req.cwd.clone(),
        cols: req.cols,
        rows: req.rows,
    };
    match state.pane_pty_pool.take(&spawn_req) {
        TakeResult::Warm(_) => {
            state.pane_pty_pool.refill_async(
                PathBuf::from(&spawn_req.shell),
                PathBuf::from(&spawn_req.cwd),
            );
            Ok(SpawnResult { warm: true })
        }
        TakeResult::Cold => {
            pane_pty::spawn(&state.pane_pty, req).map_err(|e| e.to_string())?;
            Ok(SpawnResult { warm: false })
        }
    }
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

#[tauri::command]
fn branch_list(
    state: State<'_, AppState>,
    req: BranchListRequest,
) -> Result<BranchListResponse, BranchError> {
    let repo_path = branch_repo_path(&state, &req.workspace_id)?;
    core_branch_list(&repo_path)
}

#[tauri::command]
fn branch_create(
    state: State<'_, AppState>,
    app: AppHandle,
    req: BranchCreateRequest,
) -> Result<(), BranchError> {
    let repo_path = branch_repo_path(&state, &req.workspace_id)?;
    let result = core_branch_create(&repo_path, req.clone());
    match result {
        Ok(()) => emit_branch_changed(&app, &req.workspace_id, &repo_path),
        Err(error) => {
            if req.checkout && branch_exists(&repo_path, &req.name) {
                let _ = emit_branch_changed(&app, &req.workspace_id, &repo_path);
            }
            Err(error)
        }
    }
}

#[tauri::command]
fn branch_checkout(
    state: State<'_, AppState>,
    app: AppHandle,
    req: BranchCheckoutRequest,
) -> Result<BranchSwitchResult, BranchError> {
    let repo_path = branch_repo_path(&state, &req.workspace_id)?;
    let result = core_branch_checkout(&repo_path, req.clone())?;
    emit_branch_changed(&app, &req.workspace_id, &repo_path)?;
    Ok(result)
}

#[tauri::command]
fn branch_delete(
    state: State<'_, AppState>,
    app: AppHandle,
    req: BranchDeleteRequest,
) -> Result<(), BranchError> {
    let repo_path = branch_repo_path(&state, &req.workspace_id)?;
    core_branch_delete(&repo_path, req.clone())?;
    emit_branch_changed(&app, &req.workspace_id, &repo_path)
}

#[tauri::command]
fn branch_switcher_query(
    state: State<'_, AppState>,
    req: SwitcherQueryRequest,
) -> Result<SwitcherSearchResult, BranchError> {
    let repo_path = branch_repo_path(&state, &req.workspace_id)?;
    core_branch_switcher_query(&repo_path, req)
}

fn branch_repo_path(
    state: &State<'_, AppState>,
    workspace_id: &str,
) -> Result<PathBuf, BranchError> {
    let guard = state
        .pool
        .lock()
        .map_err(|e| branch_state_error(e.to_string()))?;
    let pool = guard
        .as_ref()
        .ok_or_else(|| branch_state_error("database not initialized"))?;
    let workspace = WorkspaceStore::get_by_id(pool, workspace_id)
        .map_err(|e| branch_state_error(e.to_string()))?;
    Ok(PathBuf::from(workspace.path))
}

fn emit_branch_changed(
    app: &AppHandle,
    workspace_id: &str,
    repo_path: &Path,
) -> Result<(), BranchError> {
    let response = core_branch_list(repo_path)?;
    app.emit(
        BRANCH_CHANGED_EVENT,
        BranchChangedEvent {
            workspace_id: workspace_id.to_string(),
            branches: response.branches,
            head: response.head_name,
        },
    )
    .map_err(|e| branch_state_error(e.to_string()))
}

fn branch_exists(repo_path: &Path, name: &str) -> bool {
    core_branch_list(repo_path).is_ok_and(|response| {
        response
            .branches
            .iter()
            .any(|branch| branch.name == name && branch.kind == vibestation_core::BranchKind::Local)
    })
}

fn branch_state_error(message: impl Into<String>) -> BranchError {
    BranchError::Git2Error {
        class: "AppState".to_string(),
        code: -1,
        message: message.into(),
    }
}

#[tauri::command]
fn git_remote_list(
    state: State<'_, AppState>,
    req: RemoteListRequest,
) -> Result<RemoteListResponse, NetworkOpError> {
    let repo_path = git_sync_repo_path(&state, &req.workspace_id)?;
    core_git_remote_list(&repo_path)
}

#[tauri::command]
fn git_push(
    state: State<'_, AppState>,
    app: AppHandle,
    req: PushRequest,
) -> Result<PushResult, NetworkOpError> {
    let repo_path = git_sync_repo_path(&state, &req.workspace_id)?;
    core_git_push(&repo_path, req, git_sync_event_handlers(app))
}

#[tauri::command]
fn git_pull(
    state: State<'_, AppState>,
    app: AppHandle,
    req: PullRequest,
) -> Result<PullResult, NetworkOpError> {
    let repo_path = git_sync_repo_path(&state, &req.workspace_id)?;
    core_git_pull(&repo_path, req, git_sync_event_handlers(app))
}

#[tauri::command]
fn git_fetch(
    state: State<'_, AppState>,
    app: AppHandle,
    req: FetchRequest,
) -> Result<FetchResult, NetworkOpError> {
    let repo_path = git_sync_repo_path(&state, &req.workspace_id)?;
    core_git_fetch(&repo_path, req, git_sync_event_handlers(app))
}

#[tauri::command]
fn git_auth_provide(state: State<'_, AppState>, req: AuthRequest) -> Result<(), NetworkOpError> {
    let repo_path = git_sync_repo_path(&state, &req.workspace_id)?;
    core_git_auth_provide(&repo_path, req)
}

#[tauri::command]
fn git_merge_abort(state: State<'_, AppState>, workspace_id: String) -> Result<(), NetworkOpError> {
    let repo_path = git_sync_repo_path(&state, &workspace_id)?;
    core_git_merge_abort(&repo_path)
}

fn git_sync_repo_path(
    state: &State<'_, AppState>,
    workspace_id: &str,
) -> Result<PathBuf, NetworkOpError> {
    let guard = state
        .pool
        .lock()
        .map_err(|error| NetworkOpError::Git2Error {
            class: "AppState".to_string(),
            code: -1,
            message: error.to_string(),
        })?;
    let pool = guard.as_ref().ok_or_else(|| NetworkOpError::Git2Error {
        class: "AppState".to_string(),
        code: -1,
        message: "database not initialized".to_string(),
    })?;
    let workspace = WorkspaceStore::get_by_id(pool, workspace_id).map_err(|error| {
        NetworkOpError::Git2Error {
            class: "AppState".to_string(),
            code: -1,
            message: error.to_string(),
        }
    })?;
    Ok(PathBuf::from(workspace.path))
}

fn git_sync_event_handlers(app: AppHandle) -> GitSyncEventHandlers {
    let push_app = app.clone();
    let fetch_app = app.clone();
    let done_app = app;
    GitSyncEventHandlers {
        push_progress: Some(Arc::new(move |event: PushProgressEvent| {
            push_app.emit(GIT_PUSH_PROGRESS_EVENT, event).is_ok()
        })),
        fetch_progress: Some(Arc::new(move |event: FetchProgressEvent| {
            fetch_app.emit(GIT_FETCH_PROGRESS_EVENT, event).is_ok()
        })),
        operation_done: Some(Arc::new(move |event: OperationDoneEvent| {
            if let Err(error) = done_app.emit(GIT_OPERATION_DONE_EVENT, event) {
                eprintln!("[mvp-21] emit git operation done failed: {error}");
            }
        })),
    }
}

#[tauri::command]
fn rebase_start(
    state: State<'_, AppState>,
    app: AppHandle,
    req: RebaseStartRequest,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    let workspace_id = req.workspace_id.clone();
    let result = vibestation_core::rebase_start(&repo_path, req)?;
    emit_rebase_status(&app, &workspace_id, &result);
    if !result.in_progress {
        emit_rebase_done(&app, &workspace_id, "rebase", true, "rebase completed");
    }
    Ok(result)
}

#[tauri::command]
fn rebase_continue(
    state: State<'_, AppState>,
    app: AppHandle,
    req: RebaseControlRequest,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    let workspace_id = req.workspace_id.clone();
    let result = vibestation_core::rebase_continue(&repo_path, req)?;
    emit_rebase_status(&app, &workspace_id, &result);
    if !result.in_progress {
        emit_rebase_done(&app, &workspace_id, "rebase", true, "rebase completed");
    }
    Ok(result)
}

#[tauri::command]
fn rebase_abort(
    state: State<'_, AppState>,
    app: AppHandle,
    req: RebaseControlRequest,
) -> Result<(), RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    vibestation_core::rebase_abort(&repo_path, req.clone())?;
    emit_rebase_done(&app, &req.workspace_id, "rebase", false, "rebase aborted");
    Ok(())
}

#[tauri::command]
fn rebase_skip(
    state: State<'_, AppState>,
    app: AppHandle,
    req: RebaseControlRequest,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    let workspace_id = req.workspace_id.clone();
    let result = vibestation_core::rebase_skip(&repo_path, req)?;
    emit_rebase_status(&app, &workspace_id, &result);
    Ok(result)
}

#[tauri::command]
fn rebase_interactive_plan(
    state: State<'_, AppState>,
    workspace_id: String,
    branch: String,
    onto: String,
) -> Result<RebaseInteractivePlan, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    vibestation_core::rebase_interactive_plan(&repo_path, &branch, &onto)
}

#[tauri::command]
fn rebase_interactive_apply(
    state: State<'_, AppState>,
    app: AppHandle,
    workspace_id: String,
    plan: RebaseInteractivePlan,
) -> Result<RebaseStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    let result = vibestation_core::rebase_interactive_apply(&repo_path, plan)?;
    emit_rebase_status(&app, &workspace_id, &result);
    Ok(result)
}

#[tauri::command]
fn merge_start(
    state: State<'_, AppState>,
    app: AppHandle,
    req: MergeRequest,
) -> Result<MergeStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    let workspace_id = req.workspace_id.clone();
    let result = vibestation_core::merge_start(&repo_path, req)?;
    if result.outcome == "conflict" {
        emit_rebase_conflict(&app, &workspace_id, "merge", &result.conflicting_files);
    } else {
        emit_rebase_done(&app, &workspace_id, "merge", true, &result.outcome);
    }
    Ok(result)
}

#[tauri::command]
fn merge_abort(
    state: State<'_, AppState>,
    app: AppHandle,
    workspace_id: String,
) -> Result<(), RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    vibestation_core::rebase_merge_abort(&repo_path)?;
    emit_rebase_done(&app, &workspace_id, "merge", false, "merge aborted");
    Ok(())
}

#[tauri::command]
fn cherrypick_start(
    state: State<'_, AppState>,
    app: AppHandle,
    req: CherryPickRequest,
) -> Result<CherryPickStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    let workspace_id = req.workspace_id.clone();
    let result = vibestation_core::cherrypick_start(&repo_path, req)?;
    if result.in_progress && !result.conflicting_files.is_empty() {
        emit_rebase_conflict(&app, &workspace_id, "cherrypick", &result.conflicting_files);
    }
    if !result.in_progress {
        emit_rebase_done(
            &app,
            &workspace_id,
            "cherrypick",
            true,
            "cherrypick completed",
        );
    }
    Ok(result)
}

#[tauri::command]
fn cherrypick_continue(
    state: State<'_, AppState>,
    app: AppHandle,
    workspace_id: String,
) -> Result<CherryPickStatus, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    let result = vibestation_core::cherrypick_continue(&repo_path)?;
    if !result.in_progress {
        emit_rebase_done(
            &app,
            &workspace_id,
            "cherrypick",
            true,
            "cherrypick completed",
        );
    }
    Ok(result)
}

#[tauri::command]
fn cherrypick_abort(
    state: State<'_, AppState>,
    app: AppHandle,
    workspace_id: String,
) -> Result<(), RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    vibestation_core::cherrypick_abort(&repo_path)?;
    emit_rebase_done(
        &app,
        &workspace_id,
        "cherrypick",
        false,
        "cherrypick aborted",
    );
    Ok(())
}

#[tauri::command]
fn conflict_resolve_file(
    state: State<'_, AppState>,
    req: ConflictResolveFileRequest,
) -> Result<(), RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &req.workspace_id)?;
    vibestation_core::conflict_resolve_file(&repo_path, req)
}

#[tauri::command]
fn conflict_status(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Vec<ConflictedFile>, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    vibestation_core::conflict_status(&repo_path)
}

/// MVP-16 Phase C · 主动按 workspace 查询是否有 in-progress 操作。
///
/// 用途：(1) workspace 切换时 frontend 检查目标 workspace 是否有未完成 rebase / merge /
/// cherrypick · 立即渲染 recovery banner（启动时 emit 是 best-effort · 之后切换需主动查）。
/// (2) 用户手动刷新或在 banner 上点 "重新检测" 时调用。
///
/// 与启动 emit 用同一 `vibestation_core::detect_in_progress` · 输出语义一致。
#[tauri::command]
fn rebase_detect_in_progress(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<CrashRecoveryState, RebaseOpError> {
    let repo_path = rebase_repo_path(&state, &workspace_id)?;
    vibestation_core::detect_in_progress(&repo_path)
}

fn rebase_repo_path(
    state: &State<'_, AppState>,
    workspace_id: &str,
) -> Result<PathBuf, RebaseOpError> {
    let guard = state
        .pool
        .lock()
        .map_err(|error| RebaseOpError::Git2Error {
            class: "AppState".to_string(),
            code: -1,
            message: error.to_string(),
        })?;
    let pool = guard.as_ref().ok_or_else(|| RebaseOpError::Git2Error {
        class: "AppState".to_string(),
        code: -1,
        message: "database not initialized".to_string(),
    })?;
    let workspace = WorkspaceStore::get_by_id(pool, workspace_id).map_err(|error| {
        RebaseOpError::Git2Error {
            class: "AppState".to_string(),
            code: -1,
            message: error.to_string(),
        }
    })?;
    Ok(PathBuf::from(workspace.path))
}

fn emit_rebase_status(app: &AppHandle, workspace_id: &str, status: &RebaseStatus) {
    let _ = app.emit(
        GIT_REBASE_PROGRESS_EVENT,
        RebaseProgressEvent {
            workspace_id: workspace_id.to_string(),
            current_step: status.current_step,
            total_steps: status.total_steps,
            current_commit: None,
        },
    );
    if status.in_progress && !status.conflicting_files.is_empty() {
        emit_rebase_conflict(
            app,
            workspace_id,
            status.operation.as_deref().unwrap_or("rebase"),
            &status.conflicting_files,
        );
    }
}

fn emit_rebase_conflict(app: &AppHandle, workspace_id: &str, operation: &str, files: &[String]) {
    let _ = app.emit(
        GIT_CONFLICT_DETECTED_EVENT,
        ConflictDetectedEvent {
            workspace_id: workspace_id.to_string(),
            operation: operation.to_string(),
            conflicting_files: files.to_vec(),
        },
    );
}

fn emit_rebase_done(
    app: &AppHandle,
    workspace_id: &str,
    operation: &str,
    success: bool,
    message: &str,
) {
    let _ = app.emit(
        GIT_OPERATION_DONE_EVENT,
        RebaseOperationDoneEvent {
            workspace_id: workspace_id.to_string(),
            operation: operation.to_string(),
            success,
            message: message.to_string(),
        },
    );
}

/// MVP-16 Phase C · 启动时扫所有 workspace · 任一存在 in-progress 操作（.git/rebase-merge /
/// .git/CHERRY_PICK_HEAD / .git/MERGE_HEAD / OperationState 文件）即 emit 一条
/// `git:crash-recovery-detected` event · 前端据此渲染 recovery banner。
///
/// 不复用 `git:operation-done` · 避免和正常 abort/completion 路径在 success=false 处歧义。
fn emit_rebase_crash_recovery(app: &AppHandle, pool: &DbPool) {
    let Ok(workspaces) = WorkspaceStore::list(pool) else {
        return;
    };
    for workspace in workspaces {
        let repo_path = PathBuf::from(&workspace.path);
        let Ok(state) = vibestation_core::detect_in_progress(&repo_path) else {
            continue;
        };
        if !state.in_progress {
            continue;
        }
        let _ = app.emit(
            GIT_CRASH_RECOVERY_EVENT,
            RebaseCrashRecoveryEvent {
                workspace_id: workspace.workspace_id.clone(),
                operation: state.operation,
                branch: state.branch,
                current_step: state.current_step,
                total_steps: state.total_steps,
            },
        );
    }
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

    let pty = Arc::new(PtyManager::new());
    let pty_events = pty
        .take_event_receiver()
        .expect("PTY event receiver should be taken exactly once");
    let pane_pty = Arc::new(PtyManager::new());
    let pane_pty_events = pane_pty
        .take_event_receiver()
        .expect("Pane PTY event receiver should be taken exactly once");

    // MVP-20 · 创建 PtyPool（与各自 PtyManager 配套 · 持 Weak 引用防循环）
    // 默认 PoolConfig（enabled=true · size=1）· workspace_init 后用 DB settings 同步
    let pty_pool = Arc::new(PtyPool::new(Arc::clone(&pty), PoolConfig::default()));
    let pane_pty_pool = Arc::new(PtyPool::new(Arc::clone(&pane_pty), PoolConfig::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            pool: Mutex::new(None),
            git_status: GitStatusWatchManager::new(),
            pty,
            pane_pty,
            pty_pool,
            pane_pty_pool,
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
            default_shell_get,
            default_shell_set,
            available_shells_list,
            settings_get,
            settings_update,
            telemetry_opt_in_set,
            telemetry_status_get,
            app_version_get,
            config_import_scan,
            config_import_preview,
            config_import_detect_conflicts,
            config_import_apply,
            tab_list,
            tab_create,
            tab_close,
            tab_rename,
            tab_reorder,
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
            pane_layout_apply_advanced,
            pane_split_ratio_update,
            pane_navigate,
            pane_maximize,
            pane_resize_step,
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
            branch_list,
            branch_create,
            branch_checkout,
            branch_delete,
            branch_switcher_query,
            git_push,
            git_pull,
            git_fetch,
            git_remote_list,
            git_auth_provide,
            git_merge_abort,
            rebase_start,
            rebase_continue,
            rebase_abort,
            rebase_skip,
            rebase_interactive_plan,
            rebase_interactive_apply,
            merge_start,
            merge_abort,
            cherrypick_start,
            cherrypick_continue,
            cherrypick_abort,
            conflict_resolve_file,
            conflict_status,
            rebase_detect_in_progress,
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
