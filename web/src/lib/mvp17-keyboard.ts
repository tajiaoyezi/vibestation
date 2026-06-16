/**
 * MVP-17 Phase C · 全局键盘快捷键
 *
 * ⌘⇧O (macOS) / Ctrl⇧O (Linux) · Pop to External
 * ⌘⇧D (macOS) / Ctrl⇧D (Linux) · Detach Pane
 */

import { isMacPlatform } from "./platform";

export interface Mvp17KeyboardHandler {
  onPopToExternal?: () => void;
  onDetachPane?: () => void;
}

/**
 * 判断当前 keydown 是否匹配 Pop to External 快捷键
 */
export function matchesPopToExternalShortcut(event: KeyboardEvent): boolean {
  if (!event.shiftKey) return false;
  if (event.key !== "o" && event.key !== "O") return false;

  const isMac = isMacPlatform();
  return isMac
    ? event.metaKey && !event.ctrlKey && !event.altKey
    : event.ctrlKey && !event.metaKey && !event.altKey;
}

/**
 * 判断当前 keydown 是否匹配 Detach Pane 快捷键
 */
export function matchesDetachPaneShortcut(event: KeyboardEvent): boolean {
  if (!event.shiftKey) return false;
  if (event.key !== "d" && event.key !== "D") return false;

  const isMac = isMacPlatform();
  return isMac
    ? event.metaKey && !event.ctrlKey && !event.altKey
    : event.ctrlKey && !event.metaKey && !event.altKey;
}

/**
 * 注册 MVP-17 快捷键监听器
 *
 * 返回 cleanup 函数（App.tsx onCleanup 调用）
 */
export function registerMvp17KeyboardShortcuts(
  handler: Mvp17KeyboardHandler,
): () => void {
  const onKeyDown = (event: KeyboardEvent): void => {
    if (matchesPopToExternalShortcut(event)) {
      event.preventDefault();
      handler.onPopToExternal?.();
      return;
    }
    if (matchesDetachPaneShortcut(event)) {
      event.preventDefault();
      handler.onDetachPane?.();
      return;
    }
  };

  document.addEventListener("keydown", onKeyDown);
  return () => document.removeEventListener("keydown", onKeyDown);
}
