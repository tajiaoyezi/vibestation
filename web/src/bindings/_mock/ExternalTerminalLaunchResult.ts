// MOCK · 等 MVP-17 Phase A PR merge 后删 · 改 import "../ExternalTerminalLaunchResult"

export interface ExternalTerminalLaunchResult {
  /** 是否成功启动 */
  success: boolean;
  /** 失败原因（success=true 时为 null） */
  failedReason?: string | null;
}
