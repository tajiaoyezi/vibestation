import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  FileStatusEvent,
  GitStatusCollapseRequest,
  GitStatusPanelSettings,
  GitStatusRequest,
  GitStatusResponse,
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
