/**
 * usePaneShortcuts · MVP-05 Phase C §A
 *
 * 3 个快捷键：
 * - `⌘\`（Cmd+Backslash）→ 右分屏（horizontal split）
 * - `⌘⇧\`（Cmd+Shift+Backslash）→ 下分屏（vertical split）
 * - `⌘⌃W`（Cmd+Ctrl+W）→ 关当前 Pane（仅剩 1 个 → 关整个 Tab · 由 caller 决策）
 *
 * 用法：
 * ```ts
 * usePaneShortcuts({
 *   getFocusedPaneId: () => currentTabFocusedPaneId(),
 *   onSplit: (direction) => invoke('pane_split', ...),
 *   onClose: (paneId) => invoke('pane_close', { req: { paneId } }),
 * });
 * ```
 */
import { onCleanup, onMount } from "solid-js";
import type { SplitDir } from "../../bindings";

type PaneShortcutOptions = {
  getFocusedPaneId: () => string | null;
  onSplit: (direction: SplitDir, focusedPaneId: string) => void;
  onClose: (focusedPaneId: string) => void;
  /**
   * 阻断快捷键的条件 · 默认 false（始终响应）。
   * 例：传入 `() => isAnyDialogOpen()` 让 dialog 打开时不响应分屏快捷键。
   */
  shouldSuppress?: () => boolean;
};

const isMac =
  typeof navigator !== "undefined" &&
  navigator.platform.toUpperCase().includes("MAC");

const matchesMod = (event: KeyboardEvent): boolean =>
  isMac ? event.metaKey : event.ctrlKey;

export const usePaneShortcuts = (options: PaneShortcutOptions): void => {
  const handler = (event: KeyboardEvent) => {
    if (options.shouldSuppress?.()) return;

    // ⌘\ 右分屏
    if (
      matchesMod(event) &&
      !event.shiftKey &&
      !event.altKey &&
      event.key === "\\"
    ) {
      const focused = options.getFocusedPaneId();
      if (focused) {
        event.preventDefault();
        options.onSplit("horizontal", focused);
      }
      return;
    }

    // ⌘⇧\ 下分屏
    if (
      matchesMod(event) &&
      event.shiftKey &&
      !event.altKey &&
      event.key === "|"
    ) {
      const focused = options.getFocusedPaneId();
      if (focused) {
        event.preventDefault();
        options.onSplit("vertical", focused);
      }
      return;
    }

    // ⌘⌃W 关当前 Pane（macOS：metaKey + ctrlKey · 其他平台保持 ctrl + alt + w 作为后备）
    const isMacCloseCombo =
      event.metaKey && event.ctrlKey && !event.shiftKey && event.key === "w";
    const isOtherCloseCombo =
      !isMac &&
      event.ctrlKey &&
      event.altKey &&
      !event.shiftKey &&
      event.key === "w";
    if (isMacCloseCombo || isOtherCloseCombo) {
      const focused = options.getFocusedPaneId();
      if (focused) {
        event.preventDefault();
        options.onClose(focused);
      }
    }
  };

  onMount(() => {
    window.addEventListener("keydown", handler, { capture: true });
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handler, { capture: true });
  });
};
