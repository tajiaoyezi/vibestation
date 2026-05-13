import { invoke } from "@tauri-apps/api/core";
import type { ExternalTerminalInfo } from "../bindings/ExternalTerminalInfo";
import type { ExternalTerminalLaunchRequest } from "../bindings/ExternalTerminalLaunchRequest";
import type { ExternalTerminalLaunchResult } from "../bindings/ExternalTerminalLaunchResult";
import type { EnvPreview } from "../bindings/EnvPreview";

/** MVP-17 Phase A · 外部终端 IPC wrapper */

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
