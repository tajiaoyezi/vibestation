use std::io;

use tauri::{Manager, State};

mod ipc;
mod pty_manager;
mod session;

use ipc::{
    ArtifactReadRequest, ArtifactWriteRequest, DrainSessionRequest, ProcessStats, ResizeSessionRequest,
    RuntimeConfig, SessionSummary, SpawnSessionRequest, WriteSessionRequest,
};
use pty_manager::PtyManager;

fn to_string(error: io::Error) -> String {
    error.to_string()
}

fn env_or(name: &str, fallback: &str) -> Option<String> {
    std::env::var(name).ok().or_else(|| std::env::var(fallback).ok())
}

#[tauri::command]
fn spawn_session(state: State<'_, PtyManager>, request: SpawnSessionRequest) -> Result<SessionSummary, String> {
    state.spawn_session(request).map_err(to_string)
}

#[tauri::command]
fn write_session(state: State<'_, PtyManager>, request: WriteSessionRequest) -> Result<(), String> {
    state.write_session(request).map_err(to_string)
}

#[tauri::command]
fn resize_session(state: State<'_, PtyManager>, request: ResizeSessionRequest) -> Result<(), String> {
    state.resize_session(request).map_err(to_string)
}

#[tauri::command]
fn drain_session(
    state: State<'_, PtyManager>,
    request: DrainSessionRequest,
) -> Result<ipc::DrainResponse, String> {
    state
        .drain_session(
            &request.session_id,
            request.max_chunks.unwrap_or(128),
            request.max_bytes.unwrap_or(1024 * 1024),
        )
        .map_err(to_string)
}

#[tauri::command]
fn close_session(state: State<'_, PtyManager>, session_id: String) -> Result<(), String> {
    state.close_session(&session_id).map_err(to_string)
}

#[tauri::command]
fn close_all_sessions(state: State<'_, PtyManager>) {
    state.close_all_sessions();
}

#[tauri::command]
fn session_snapshot(state: State<'_, PtyManager>, session_id: String) -> Result<SessionSummary, String> {
    state.session_snapshot(&session_id).map_err(to_string)
}

#[tauri::command]
fn manager_snapshot(state: State<'_, PtyManager>) -> Vec<SessionSummary> {
    state.manager_snapshot()
}

#[tauri::command]
fn write_artifact(state: State<'_, PtyManager>, request: ArtifactWriteRequest) -> Result<(), String> {
    state.write_artifact(request).map_err(to_string)
}

#[tauri::command]
fn read_artifact(state: State<'_, PtyManager>, request: ArtifactReadRequest) -> Result<String, String> {
    state.read_artifact(request).map_err(to_string)
}

#[tauri::command]
fn sample_process_stats(state: State<'_, PtyManager>) -> Result<ProcessStats, String> {
    state.sample_process_stats().map_err(to_string)
}

#[tauri::command]
fn runtime_config(state: State<'_, PtyManager>) -> RuntimeConfig {
    RuntimeConfig {
        scenario: env_or("SPIKE055_SCENARIO", "SPIKE05_SCENARIO"),
        output_dir: env_or("SPIKE055_OUTPUT_DIR", "SPIKE05_OUTPUT_DIR"),
        run_label: env_or("SPIKE055_RUN_LABEL", "SPIKE05_RUN_LABEL"),
        close_on_complete: env_or("SPIKE055_CLOSE_ON_COMPLETE", "SPIKE05_CLOSE_ON_COMPLETE")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
            .unwrap_or(false),
        strategy: state.strategy_name().to_string(),
    }
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PtyManager::new())
        .invoke_handler(tauri::generate_handler![
            spawn_session,
            write_session,
            resize_session,
            drain_session,
            close_session,
            close_all_sessions,
            session_snapshot,
            manager_snapshot,
            write_artifact,
            read_artifact,
            sample_process_stats,
            runtime_config,
            exit_app,
        ])
        .setup(|app| {
            let strategy = app.state::<PtyManager>().strategy_name().to_string();
            eprintln!(
                "[SPIKE-05.5] app pid={} scenario={:?} strategy={}",
                std::process::id(),
                env_or("SPIKE055_SCENARIO", "SPIKE05_SCENARIO"),
                strategy
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
