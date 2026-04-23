import { invoke } from "@tauri-apps/api/core";
import type { DiffRequest, DiffResponse } from "../../bindings";

export async function computeDiff(req: DiffRequest): Promise<DiffResponse> {
  return invoke<DiffResponse>("diff_compute", { req });
}

export async function getDiffViewMode(
  workspaceId: string,
): Promise<"split" | "unified"> {
  const result = await invoke<unknown>("diff_get_settings", { workspaceId });

  if (typeof result === "string") {
    if (result === "split" || result === "unified") {
      return result;
    }
  }

  if (typeof result === "object" && result !== null && "viewMode" in result) {
    const mode = (result as Record<string, unknown>).viewMode;
    if (mode === "split" || mode === "unified") {
      return mode as "split" | "unified";
    }
  }

  return "split";
}
