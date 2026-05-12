// MOCK · 等 MVP-17 Phase A PR merge 后删 · 改 import "../ExternalTerminalInfo"

export interface ExternalTerminalInfo {
  /** 终端唯一标识 · 如 "ghostty" / "iterm" / "terminal_app" / "alacritty" */
  id: string;
  /** 显示名称 · 如 "Ghostty" / "iTerm2" */
  displayName: string;
  /** 是否在系统探测到 */
  detected: boolean;
  /** 排序优先级 · 越小越靠前 */
  priorityHint: number;
}
