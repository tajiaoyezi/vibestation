// MOCK · 等 MVP-17 Phase A PR merge 后删 · 改 import "../ExternalTerminalLaunchRequest"

export interface ExternalTerminalLaunchRequest {
  /** 目标终端 ID · 如 "ghostty" */
  terminalId: string;
  /** 源 Pane ID · 用于取 cwd + shell */
  paneId: string;
  /** 覆盖 env（可选）· null = 用默认白名单 */
  overrideEnv?: Record<string, string> | null;
}
