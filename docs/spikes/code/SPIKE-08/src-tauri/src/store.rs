use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

use crate::contract::{
    DeleteWorkspaceRequest, DeleteWorkspaceResponse, WorkspaceDraft, WorkspaceListResponse,
    WorkspaceRecord,
};

pub struct WorkspaceStore {
    next_id: AtomicUsize,
    items: Mutex<Vec<WorkspaceRecord>>,
}

impl WorkspaceStore {
    pub fn new() -> Self {
        Self {
            next_id: AtomicUsize::new(1),
            items: Mutex::new(Vec::new()),
        }
    }

    pub fn list(&self) -> WorkspaceListResponse {
        let items = self.items.lock().expect("workspace store lock poisoned").clone();
        let total = items.len();
        WorkspaceListResponse { items, total }
    }

    pub fn create(&self, request: WorkspaceDraft) -> WorkspaceRecord {
        let next_id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let record = WorkspaceRecord {
            id: format!("workspace-{next_id:04}"),
            name: request.name,
            root_path: request.root_path,
            note: request.note,
            created_at: current_timestamp(),
        };

        self.items
            .lock()
            .expect("workspace store lock poisoned")
            .insert(0, record.clone());

        record
    }

    pub fn delete(&self, request: DeleteWorkspaceRequest) -> Result<DeleteWorkspaceResponse, String> {
        let mut items = self.items.lock().expect("workspace store lock poisoned");
        let before = items.len();
        items.retain(|workspace| workspace.id != request.workspace_id);

        if items.len() == before {
            return Err(format!("workspace not found: {}", request.workspace_id));
        }

        Ok(DeleteWorkspaceResponse {
            deleted_workspace_id: request.workspace_id,
            remaining: items.len(),
        })
    }
}

fn current_timestamp() -> String {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock drift");
    format!("unix:{}", elapsed.as_secs())
}
