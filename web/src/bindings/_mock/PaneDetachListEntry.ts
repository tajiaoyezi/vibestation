// MOCK · 等 MVP-17 Phase B PR merge 后删 · 改 import "../PaneDetachListEntry"

export interface PaneDetachListEntry {
  /** 窗口 label */
  windowLabel: string;
  /** 关联 Pane ID */
  paneId: string;
  /** 当前窗口位置大小 */
  bounds: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}
