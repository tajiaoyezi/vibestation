import { invoke } from "@tauri-apps/api/core";
import type { ExternalTerminalInfo } from "../bindings/_mock/ExternalTerminalInfo";
import type { ExternalTerminalLaunchRequest } from "../bindings/_mock/ExternalTerminalLaunchRequest";
import type { ExternalTerminalLaunchResult } from "../bindings/_mock/ExternalTerminalLaunchResult";
import type { EnvPreview } from "../bindings/_mock/EnvPreview";

/**
 * MVP-17 Phase A · 外部终端 IPC wrapper
 *
 * Phase A/B merge 后改 import 路径：
 *   "../bindings/_mock/ExternalTerminalInfo" → "../bindings/ExternalTerminalInfo"
 */

export async function listTerminals(): Promise<ExternalTerminalInfo[]> {
  return invoke<ExternalTerminalInfo[]>("external_term_list");
}

export async function previewEnv(paneId: string): Promise<EnvPreview> {
  return invoke<EnvPreview>("external_term_preview_env", { paneId });
}

export async function launchTerminal(
  request: ExternalTerminalLaunchRequest,
): Promise<ExternalTerminalLaunchResult> {
  return invoke<ExternalTerminalLaunchResult>("external_term_launch", {
    request,
  });
}
