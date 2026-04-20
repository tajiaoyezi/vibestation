import { invoke } from "@tauri-apps/api/core";
import type {
  DeleteWorkspaceRequest,
  DeleteWorkspaceResponse,
  WorkspaceDraft,
  WorkspaceListResponse,
  WorkspaceRecord,
} from "./bindings";

export type { WorkspaceDraft, WorkspaceListResponse, WorkspaceRecord };

type MockStore = {
  nextId: number;
  workspaces: WorkspaceRecord[];
};

const mockStore: MockStore = {
  nextId: 1,
  workspaces: [],
};

function isTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function nowIso() {
  return new Date().toISOString();
}

function createMockRecord(request: WorkspaceDraft): WorkspaceRecord {
  const id = `workspace-${String(mockStore.nextId).padStart(4, "0")}`;
  mockStore.nextId += 1;

  return {
    id,
    name: request.name,
    rootPath: request.rootPath,
    note: request.note,
    createdAt: nowIso(),
  };
}

async function browserInvoke<T>(command: string, args?: Record<string, unknown>) {
  switch (command) {
    case "list_workspaces": {
      const response: WorkspaceListResponse = {
        items: [...mockStore.workspaces],
        total: mockStore.workspaces.length,
      };
      return response as T;
    }

    case "create_workspace": {
      const request = args?.request as WorkspaceDraft | undefined;
      if (!request) {
        throw new Error("missing request payload");
      }
      const created = createMockRecord(request);
      mockStore.workspaces = [created, ...mockStore.workspaces];
      return created as T;
    }

    case "delete_workspace": {
      const request = args?.request as DeleteWorkspaceRequest | undefined;
      if (!request) {
        throw new Error("missing delete request payload");
      }

      const nextItems = mockStore.workspaces.filter(
        (workspace) => workspace.id !== request.workspaceId,
      );

      if (nextItems.length === mockStore.workspaces.length) {
        throw new Error(`workspace not found: ${request.workspaceId}`);
      }

      mockStore.workspaces = nextItems;
      const response: DeleteWorkspaceResponse = {
        deletedWorkspaceId: request.workspaceId,
        remaining: nextItems.length,
      };
      return response as T;
    }

    default:
      throw new Error(`unsupported mock invoke: ${command}`);
  }
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>) {
  if (isTauriRuntime()) {
    return invoke<T>(command, args);
  }
  return browserInvoke<T>(command, args);
}

export async function listWorkspaces() {
  return invokeCommand<WorkspaceListResponse>("list_workspaces");
}

export async function createWorkspace(request: WorkspaceDraft) {
  return invokeCommand<WorkspaceRecord>("create_workspace", { request });
}

export async function deleteWorkspace(request: DeleteWorkspaceRequest) {
  return invokeCommand<DeleteWorkspaceResponse>("delete_workspace", { request });
}
