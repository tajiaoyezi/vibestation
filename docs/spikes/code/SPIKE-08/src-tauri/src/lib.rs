mod contract;
mod store;

use tauri::State;

use contract::{
    DeleteWorkspaceRequest, DeleteWorkspaceResponse, WorkspaceDraft, WorkspaceListResponse,
    WorkspaceRecord,
};
use store::WorkspaceStore;

#[tauri::command]
fn list_workspaces(store: State<'_, WorkspaceStore>) -> WorkspaceListResponse {
    store.list()
}

#[tauri::command]
fn create_workspace(store: State<'_, WorkspaceStore>, request: WorkspaceDraft) -> WorkspaceRecord {
    store.create(request)
}

#[tauri::command]
fn delete_workspace(
    store: State<'_, WorkspaceStore>,
    request: DeleteWorkspaceRequest,
) -> Result<DeleteWorkspaceResponse, String> {
    store.delete(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .manage(WorkspaceStore::new())
        .invoke_handler(tauri::generate_handler![list_workspaces, create_workspace, delete_workspace]);

    #[cfg(feature = "e2e-testing")]
    let builder = builder.plugin(tauri_plugin_playwright::init());

    builder
        .run(tauri::generate_context!())
        .expect("error while running spike-08 app");
}
