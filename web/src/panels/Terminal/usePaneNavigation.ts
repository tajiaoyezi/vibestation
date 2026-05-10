/**
 * MVP-14 Phase C · ⌘⌥ / Ctrl+Alt + 方向键 几何相邻导航 hook
 *
 * §D.1 跨平台 modifier · §D.2 重叠投影最大相邻算法（lib/pane-keyboard）·
 * §D.4 no-op 时给 caller 闪动回调（不发 toast · 150ms 视觉确认）·
 * §D.6 tab 切换后下一次 keydown 自动重新查 DOM rect · 不复用旧缓存。
 *
 * 该 hook 只关心 keyboard event → neighbor pane id 决策；caller 拿到
 * neighborId 自己决定走 IPC pane_focus 还是 frontend-only state。
 */
import { onCleanup, onMount } from "solid-js";
import {
  arrowKeyToDirection,
  collectPaneRectsFromDom,
  findGeometricNeighbor,
  matchesPaneNavigateModifier,
} from "../../lib/pane-keyboard";

export type PaneNavigationOptions = {
  /** active tab 是否处于 pane mode（panesByTabId 有记录）· 否则不响应 */
  isPaneMode: () => boolean;
  /** 当前 active tab 的 focused pane id · null 时不触发 */
  getFocusedPaneId: () => string | null;
  /** active tab 容器 DOM scope · 多 tab 时只查 active tab 的 pane · 防 hidden tab 干扰 */
  getActiveTabHost: () => ParentNode | null;
  /** 找到相邻 pane · caller 调 IPC pane_focus 切焦点 */
  onNavigate: (neighborPaneId: string) => void;
  /** §D.4 找不到相邻 · caller 触发 150ms 闪动反馈 */
  onNoOpFlash: () => void;
  /** suppress 条件 · 例：dialog open / pendingPaste · 默认 false */
  shouldSuppress?: () => boolean;
};

export const usePaneNavigation = (options: PaneNavigationOptions): void => {
  const handler = (event: KeyboardEvent) => {
    if (options.shouldSuppress?.()) return;
    if (!matchesPaneNavigateModifier(event)) return;
    const direction = arrowKeyToDirection(event.key);
    if (!direction) return;
    if (!options.isPaneMode()) return;

    // §D.1 必须 preventDefault · 否则 PTY 收到 ESC[1;5C 等 escape sequence
    // 导致 shell 误读光标导航键。stopPropagation 防止全局其他 keydown handler
    // 重复处理。
    event.preventDefault();
    event.stopPropagation();

    const focused = options.getFocusedPaneId();
    if (!focused) return;

    // §D.6 每次重新查 DOM rect · 不缓存 · tab 切换后 hidden tab 的 rect 是
    // 0×0（is-hidden display:none）· collectPaneRectsFromDom 已过滤 0×0。
    const scope = options.getActiveTabHost() ?? document;
    const rects = collectPaneRectsFromDom(scope);
    const neighbor = findGeometricNeighbor(rects, focused, direction);

    if (neighbor) {
      options.onNavigate(neighbor);
    } else {
      options.onNoOpFlash();
    }
  };

  onMount(() => {
    window.addEventListener("keydown", handler, { capture: true });
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handler, { capture: true });
  });
};

/**
 * MVP-14 Phase C · ⌘Enter / Ctrl+Enter 临时最大化 hook
 *
 * §E.1 进入：focused pane 撑满 · 其他 pane visibility hidden 不 unmount。
 * §E.2 退出：再按一次 / Esc 恢复 · LayoutNode 字段不动（max 状态是 frontend-only）。
 * §E.5 workspace switch / tab close 由 caller 在 toggle 之外清理。
 *
 * 平台 modifier：macOS metaKey · Linux ctrlKey · Shift / Alt 必须没按。
 */
export type PaneMaximizeToggleOptions = {
  isPaneMode: () => boolean;
  hasFocusedPane: () => boolean;
  /** 切换最大化状态 · caller 维护 maximizedPaneIdByTab signal */
  onToggle: () => void;
  /** Esc 退出 · 仅当当前处于 maximized 才响应 */
  onExit: () => void;
  /** 当前是否处于 maximized 模式 · 决定 Esc 是否拦截 */
  isMaximized: () => boolean;
  shouldSuppress?: () => boolean;
};

const isMacPlatform = (): boolean =>
  typeof navigator !== "undefined" &&
  navigator.platform.toUpperCase().includes("MAC");

const matchesMaximizeToggle = (event: KeyboardEvent): boolean => {
  if (event.key !== "Enter") return false;
  if (event.shiftKey || event.altKey) return false;
  return isMacPlatform()
    ? event.metaKey && !event.ctrlKey
    : event.ctrlKey && !event.metaKey;
};

export const usePaneMaximizeToggle = (
  options: PaneMaximizeToggleOptions,
): void => {
  const handler = (event: KeyboardEvent) => {
    if (options.shouldSuppress?.()) return;

    // ⌘Enter / Ctrl+Enter toggle
    if (matchesMaximizeToggle(event)) {
      if (!options.isPaneMode()) return;
      if (!options.hasFocusedPane()) return;
      event.preventDefault();
      event.stopPropagation();
      options.onToggle();
      return;
    }

    // Esc 退出 maximized · 不在 maximized 时不拦截（保留 cancel paste / dialog 等用途）
    if (
      event.key === "Escape" &&
      !event.shiftKey &&
      !event.altKey &&
      !event.metaKey &&
      !event.ctrlKey &&
      options.isMaximized()
    ) {
      event.preventDefault();
      event.stopPropagation();
      options.onExit();
    }
  };

  onMount(() => {
    window.addEventListener("keydown", handler, { capture: true });
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handler, { capture: true });
  });
};
