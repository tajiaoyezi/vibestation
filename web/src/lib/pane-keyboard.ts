/**
 * MVP-14 Phase C · Pane 键盘导航 · 几何相邻算法（pure function）
 *
 * 给定一组 leaf pane 的 bounding box 和当前 focused pane，找方向键期望跳到的相邻 pane。
 *
 * §D.2 重叠投影最大：以 focused leaf bbox 为锚，向目标方向找跨方向轴上重叠最大的 pane。
 * §D.3 不跨非相邻 splitter：跨方向轴上没有重叠的 pane 直接被过滤掉，
 *   tie-break 用方向轴中心距离最小者；都为 0 时再用 paneId 字典序保证确定性。
 *
 * 该函数纯函数 · 不依赖 DOM · 只接受 rect 列表 · 方便 vitest 单测。
 */
import { isMacPlatform } from "./platform";

export type PaneDirection = "left" | "right" | "up" | "down";

export interface PaneRect {
  paneId: string;
  /**
   * Bounding box · 单位任意（px / normalized 0..1 / 整数 grid 坐标都行）。
   * left < right · top < bottom。
   */
  rect: {
    left: number;
    top: number;
    right: number;
    bottom: number;
  };
}

/**
 * 浮点容差 · 防 0.49999 vs 0.5 这种 sub-pixel 边界判定误差。1px 单位下 1.0
 * 容差不会让真正不相邻的 pane 错配（pane 之间至少有 1px splitter）。
 */
const EPS = 1.0;

/**
 * 在指定方向上找几何相邻的 leaf pane id；找不到时返回 null（D.4 caller 走 no-op）。
 *
 * 复杂度：O(n)，n 为 pane 数量。MVP-14 上限 5 层 nested = 16 leaf · O(n) 完全够用。
 */
export function findGeometricNeighbor(
  panes: ReadonlyArray<PaneRect>,
  focusedPaneId: string,
  direction: PaneDirection,
): string | null {
  const focused = panes.find((p) => p.paneId === focusedPaneId);
  if (!focused) return null;

  // §D.2 第一步 · 方向过滤：只保留在目标方向上的 pane
  const candidates = panes.filter((p) => {
    if (p.paneId === focusedPaneId) return false;
    switch (direction) {
      case "left":
        return p.rect.right <= focused.rect.left + EPS;
      case "right":
        return p.rect.left >= focused.rect.right - EPS;
      case "up":
        return p.rect.bottom <= focused.rect.top + EPS;
      case "down":
        return p.rect.top >= focused.rect.bottom - EPS;
    }
  });

  if (candidates.length === 0) return null;

  // §D.3 第二步 · 跨方向轴重叠投影 + 中心距离 + paneId tie-break
  const horizontal = direction === "left" || direction === "right";

  const score = (c: PaneRect): { overlap: number; centerDist: number } => {
    if (horizontal) {
      // left/right 方向 · 跨轴是 vertical · 计算 vertical overlap
      const overlap = Math.max(
        0,
        Math.min(focused.rect.bottom, c.rect.bottom) -
          Math.max(focused.rect.top, c.rect.top),
      );
      const focusedCenter = (focused.rect.top + focused.rect.bottom) / 2;
      const cCenter = (c.rect.top + c.rect.bottom) / 2;
      return { overlap, centerDist: Math.abs(focusedCenter - cCenter) };
    } else {
      // up/down 方向 · 跨轴是 horizontal · 计算 horizontal overlap
      const overlap = Math.max(
        0,
        Math.min(focused.rect.right, c.rect.right) -
          Math.max(focused.rect.left, c.rect.left),
      );
      const focusedCenter = (focused.rect.left + focused.rect.right) / 2;
      const cCenter = (c.rect.left + c.rect.right) / 2;
      return { overlap, centerDist: Math.abs(focusedCenter - cCenter) };
    }
  };

  // 过滤跨轴 overlap = 0 的 pane（§D.3 不跨非相邻 splitter · 必须 overlap > 0）
  const overlapping = candidates
    .map((c) => ({ pane: c, ...score(c) }))
    .filter((entry) => entry.overlap > 0);

  if (overlapping.length === 0) return null;

  // 排序：overlap desc · centerDist asc · paneId asc（确定性 tie-break）
  overlapping.sort((a, b) => {
    if (a.overlap !== b.overlap) return b.overlap - a.overlap;
    if (a.centerDist !== b.centerDist) return a.centerDist - b.centerDist;
    return a.pane.paneId.localeCompare(b.pane.paneId);
  });

  return overlapping[0].pane.paneId;
}

/**
 * 从 DOM 收集所有 `[data-pane-id]` leaf 的 bounding box。
 *
 * 调用时机：keydown 处理器内部 · 用 `getBoundingClientRect` 拿当前 viewport rect ·
 * §D.6 tab 切换后必须重新调用 · 不要缓存跨 tab 的 rect（旧 tab DOM 已 hidden）。
 *
 * scope 默认 document · 测试 / 嵌套场景可传入 ancestor 限制查询范围（避免拿到旁路 tab 的 pane）。
 */
export function collectPaneRectsFromDom(
  scope: ParentNode = document,
): PaneRect[] {
  const nodes = scope.querySelectorAll<HTMLElement>("[data-pane-id]");
  const result: PaneRect[] = [];
  nodes.forEach((node) => {
    const paneId = node.getAttribute("data-pane-id");
    if (!paneId) return;
    const rect = node.getBoundingClientRect();
    // 0×0 rect = display:none 隐藏的 maximized 状态 sibling 或 hidden tab · 跳过
    if (rect.width <= 0 || rect.height <= 0) return;
    result.push({
      paneId,
      rect: {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
      },
    });
  });
  return result;
}

/**
 * §D.1 跨平台 modifier 匹配：
 * - macOS：`⌘⌥` (metaKey + altKey)
 * - 其他：`Ctrl+Alt` (ctrlKey + altKey)
 *
 * 只在 keydown 处理器命中时返回 true · caller 据此 preventDefault 防 PTY 吞 escape。
 */
export function matchesPaneNavigateModifier(event: KeyboardEvent): boolean {
  if (event.shiftKey) return false;
  const isMac = isMacPlatform();
  return isMac
    ? event.metaKey && event.altKey && !event.ctrlKey
    : event.ctrlKey && event.altKey && !event.metaKey;
}

/**
 * 把 KeyboardEvent.key 翻译成 PaneDirection · 不命中 return null（caller 不响应）。
 */
export function arrowKeyToDirection(key: string): PaneDirection | null {
  switch (key) {
    case "ArrowLeft":
      return "left";
    case "ArrowRight":
      return "right";
    case "ArrowUp":
      return "up";
    case "ArrowDown":
      return "down";
    default:
      return null;
  }
}
