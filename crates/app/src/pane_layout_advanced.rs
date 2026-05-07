//! MVP-14 · Pane 高级布局 IPC 命令（4 commands）
//!
//! - `pane_layout_apply_advanced` · 应用 DualAi / TripleReview / Quad 预设
//! - `pane_navigate` · 方向键跳相邻 Pane
//! - `pane_maximize` · 临时最大化 toggle
//! - `pane_resize_step` · 键盘 resize 5% step

use tauri::{AppHandle, Emitter, State};
use vibestation_core::{
    pane_service, LayoutApplyAdvancedRequest, LayoutApplyResult,
    PaneMaximizeRequest, PaneMaximizeResult, PaneNavigateRequest, PaneNavigateResult,
    PaneResizeStepRequest, PaneListResponse,
};

use crate::{AppState, DbPool};

fn get_pool(state: &State<'_, AppState>) -> Result<DbPool, String> {
    let guard = state.pool.lock().map_err(|e| e.to_string())?;
    guard.as_ref().cloned().ok_or("database not initialized".to_string())
}

#[tauri::command]
pub fn pane_layout_apply_advanced(
    state: State<'_, AppState>,
    req: LayoutApplyAdvancedRequest,
) -> Result<LayoutApplyResult, String> {
    let pool = get_pool(&state)?;
    pane_service::apply_layout_preset_advanced(&pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pane_navigate(
    state: State<'_, AppState>,
    req: PaneNavigateRequest,
) -> Result<PaneNavigateResult, String> {
    let pool = get_pool(&state)?;
    pane_service::apply_pane_navigate(&pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pane_maximize(
    state: State<'_, AppState>,
    req: PaneMaximizeRequest,
) -> Result<PaneMaximizeResult, String> {
    let pool = get_pool(&state)?;
    pane_service::apply_pane_maximize(&pool, &req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pane_resize_step(
    state: State<'_, AppState>,
    req: PaneResizeStepRequest,
) -> Result<PaneListResponse, String> {
    let pool = get_pool(&state)?;
    pane_service::apply_pane_resize_step(&pool, &req).map_err(|e| e.to_string())
}
