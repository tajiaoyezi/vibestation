// MOCK · 等 MVP-17 Phase B PR merge 后删 · 改 import "../PaneDetachOpenResult"

export interface PaneDetachOpenResult {
  /** 新窗口 label · 如 "pane-detach-<uuid>" */
  windowLabel: string;
  /** 初始窗口位置大小 */
  initialBounds: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}
