// MOCK · 等 MVP-17 Phase B PR merge 后删 · 改 import "../PaneDetachStateEvent"

export interface PaneDetachStateEvent {
  /** 受影响 Pane ID */
  paneId: string;
  /** 状态变化动作 */
  action: "detached" | "attached";
  /** 窗口 label（detached 时提供） */
  windowLabel?: string | null;
}
