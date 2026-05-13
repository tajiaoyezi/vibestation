import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import type { ExternalTerminalInfo } from "../bindings/ExternalTerminalInfo";
import type { ExternalTerminalLaunchRequest } from "../bindings/ExternalTerminalLaunchRequest";
import type { ExternalTerminalLaunchResult } from "../bindings/ExternalTerminalLaunchResult";
import type { EnvPreview } from "../bindings/EnvPreview";

/** MVP-17 Phase A · 外部终端 IPC wrapper · Phase C wiring 全局 signal */

// MVP-17 Phase C · Pop to External dialog 请求 · null = 关闭 · { paneId } = 为该 pane 弹对话框
// 由 PaneContextMenu / 快捷键 ⌘⇧O 设置 · App.tsx 顶层订阅渲染 PopToExternalDialog
export const [popToExternalRequest, setPopToExternalRequest] = createSignal<{
  paneId: string;
} | null>(null);

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
