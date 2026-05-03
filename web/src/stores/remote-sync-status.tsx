import {
  createContext,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
  useContext,
  type ParentComponent,
} from "solid-js";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  BranchInfo,
  BranchListRequest,
  BranchListResponse,
  OperationDoneEvent,
  WorkspaceMetadata,
} from "../bindings";

export type RemoteSyncDirection = "ahead" | "behind";

export interface RemoteSyncSnapshot {
  workspaceId: string;
  branchName: string | null;
  upstream: string | null;
  ahead: number;
  behind: number;
  loading: boolean;
  error: string | null;
}

export interface RemoteSyncHighlightRequest {
  id: number;
  workspaceId: string;
  direction: RemoteSyncDirection;
  count: number;
  branchName: string | null;
  upstream: string | null;
}

interface BranchChangedPayload {
  workspaceId: string;
  branches: BranchInfo[];
  head: string | null;
}

interface RemoteSyncContextValue {
  current: () => RemoteSyncSnapshot;
  refresh: (workspaceId: string) => Promise<void>;
  requestGitLogHighlight: (direction: RemoteSyncDirection) => void;
  highlightRequest: () => RemoteSyncHighlightRequest | null;
}

const GIT_BRANCH_CHANGED_EVENT = "git:branch-changed";
const GIT_OPERATION_DONE_EVENT = "git:operation-done";

const emptySnapshot: RemoteSyncSnapshot = {
  workspaceId: "",
  branchName: null,
  upstream: null,
  ahead: 0,
  behind: 0,
  loading: false,
  error: null,
};

const RemoteSyncStatusContext = createContext<RemoteSyncContextValue>();

export const RemoteSyncStatusProvider: ParentComponent<{
  activeWorkspace: () => WorkspaceMetadata | null;
}> = (props) => {
  const [states, setStates] = createStore<Record<string, RemoteSyncSnapshot>>(
    {},
  );
  const [highlightRequest, setHighlightRequest] =
    createSignal<RemoteSyncHighlightRequest | null>(null);
  let nextHighlightId = 1;
  let unlistenBranchChanged: UnlistenFn | undefined;
  let unlistenOperationDone: UnlistenFn | undefined;

  const current = createMemo<RemoteSyncSnapshot>(() => {
    const workspace = props.activeWorkspace();
    if (!workspace?.hasGit) {
      return emptySnapshot;
    }
    return (
      states[workspace.workspaceId] ?? {
        ...emptySnapshot,
        workspaceId: workspace.workspaceId,
      }
    );
  });

  const refresh = async (workspaceId: string) => {
    if (!workspaceId) return;
    setStates(workspaceId, (prev = { ...emptySnapshot, workspaceId }) => ({
      ...prev,
      workspaceId,
      loading: true,
      error: null,
    }));

    const req: BranchListRequest = { workspaceId };
    try {
      const response = await invoke<BranchListResponse>("branch_list", { req });
      setStates(workspaceId, snapshotFromBranchList(workspaceId, response));
    } catch (error) {
      setStates(workspaceId, (prev = { ...emptySnapshot, workspaceId }) => ({
        ...prev,
        workspaceId,
        loading: false,
        error: stringifyUnknown(error),
      }));
    }
  };

  const applyBranchChanged = (payload: BranchChangedPayload) => {
    setStates(
      payload.workspaceId,
      snapshotFromBranches(payload.workspaceId, payload.branches, payload.head),
    );
  };

  const requestGitLogHighlight = (direction: RemoteSyncDirection) => {
    const snapshot = current();
    const count = direction === "ahead" ? snapshot.ahead : snapshot.behind;
    if (!snapshot.workspaceId || count <= 0) {
      return;
    }
    setHighlightRequest({
      id: nextHighlightId++,
      workspaceId: snapshot.workspaceId,
      direction,
      count,
      branchName: snapshot.branchName,
      upstream: snapshot.upstream,
    });
  };

  createEffect(() => {
    const workspace = props.activeWorkspace();
    if (!workspace?.hasGit) {
      return;
    }
    void refresh(workspace.workspaceId);
  });

  onMount(() => {
    void listen<BranchChangedPayload>(GIT_BRANCH_CHANGED_EVENT, (event) => {
      applyBranchChanged(event.payload);
    }).then((unlisten) => {
      unlistenBranchChanged = unlisten;
    });

    void listen<OperationDoneEvent>(GIT_OPERATION_DONE_EVENT, (event) => {
      if (event.payload.outcome === "success") {
        void refresh(event.payload.workspaceId);
      }
    }).then((unlisten) => {
      unlistenOperationDone = unlisten;
    });
  });

  onCleanup(() => {
    unlistenBranchChanged?.();
    unlistenOperationDone?.();
  });

  return (
    <RemoteSyncStatusContext.Provider
      value={{
        current,
        refresh,
        requestGitLogHighlight,
        highlightRequest,
      }}
    >
      {props.children}
    </RemoteSyncStatusContext.Provider>
  );
};

export function useRemoteSyncStatus() {
  const ctx = useContext(RemoteSyncStatusContext);
  if (!ctx) {
    throw new Error(
      "useRemoteSyncStatus must be used within RemoteSyncStatusProvider",
    );
  }
  return ctx;
}

function snapshotFromBranchList(
  workspaceId: string,
  response: BranchListResponse,
): RemoteSyncSnapshot {
  return snapshotFromBranches(
    workspaceId,
    response.branches,
    response.headName,
  );
}

function snapshotFromBranches(
  workspaceId: string,
  branches: BranchInfo[],
  headName: string | null,
): RemoteSyncSnapshot {
  const currentBranch = branches.find(
    (branch) => branch.kind === "local" && branch.name === headName,
  );
  return {
    workspaceId,
    branchName: headName,
    upstream: currentBranch?.upstream ?? null,
    ahead: currentBranch?.ahead ?? 0,
    behind: currentBranch?.behind ?? 0,
    loading: false,
    error: null,
  };
}

function stringifyUnknown(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}
