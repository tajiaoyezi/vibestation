import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  CommitRequest,
  CommitResponse,
  FileStatusEvent,
  GitConfigIdentity,
  GitStatusCollapseRequest,
  GitStatusPanelSettings,
  GitStatusRequest,
  GitStatusResponse,
  SetGitIdentityRequest,
  StageRequest,
  StageResult,
  UnstageRequest,
} from "../../bindings";

export const GIT_STATUS_CHANGED_EVENT = "git_status:changed";

export async function queryStatus(
  req: GitStatusRequest,
): Promise<GitStatusResponse> {
  return invoke<GitStatusResponse>("git_status_query", { req });
}

export async function refreshStatus(
  req: GitStatusRequest,
): Promise<GitStatusResponse> {
  return invoke<GitStatusResponse>("git_status_refresh", { req });
}

export async function getPanelSettings(
  workspaceId: string,
): Promise<GitStatusPanelSettings> {
  return invoke<GitStatusPanelSettings>("git_status_get_settings", {
    workspaceId,
  });
}

export async function setGroupCollapsed(
  req: GitStatusCollapseRequest,
): Promise<void> {
  return invoke("git_status_set_group_collapsed", { req });
}

export async function subscribeGitStatus(
  workspaceId: string,
  callback: (event: FileStatusEvent) => void,
): Promise<UnlistenFn> {
  const unlisten = await listen<FileStatusEvent>(
    GIT_STATUS_CHANGED_EVENT,
    (event) => callback(event.payload),
  );

  try {
    await invoke("git_status_subscribe", { workspaceId });
  } catch (err) {
    unlisten();
    throw err;
  }

  return unlisten;
}

export async function unsubscribeGitStatus(workspaceId: string): Promise<void> {
  return invoke("git_status_unsubscribe", { workspaceId });
}

// ── Git Ops (MVP-09 Phase A) ──

export async function stageFiles(req: StageRequest): Promise<StageResult> {
  return invoke<StageResult>("git_ops_stage_files", { req });
}

export async function unstageFiles(req: UnstageRequest): Promise<void> {
  return invoke("git_ops_unstage_files", { req });
}

export async function commit(req: CommitRequest): Promise<CommitResponse> {
  return invoke<CommitResponse>("git_ops_commit", { req });
}

export async function readGitIdentity(
  workspaceId: string,
): Promise<GitConfigIdentity> {
  return invoke<GitConfigIdentity>("git_ops_read_identity", { workspaceId });
}

export async function setGitIdentity(
  req: SetGitIdentityRequest,
): Promise<void> {
  return invoke("git_ops_set_identity", { req });
}
