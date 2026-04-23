import { invoke } from "@tauri-apps/api/core";
import type {
  GitStatusCollapseRequest,
  GitStatusPanelSettings,
  GitStatusRequest,
  GitStatusResponse,
} from "../../bindings";

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
