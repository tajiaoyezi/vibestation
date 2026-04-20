use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDraft {
    pub name: String,
    pub root_path: String,
    pub note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRecord {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub note: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListResponse {
    pub items: Vec<WorkspaceRecord>,
    pub total: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkspaceRequest {
    pub workspace_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DeleteWorkspaceResponse {
    pub deleted_workspace_id: String,
    pub remaining: usize,
}
