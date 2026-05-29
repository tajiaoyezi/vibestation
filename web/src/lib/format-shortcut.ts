// Task 4.2 · shortcut-display
//
// 平台感知快捷键显示助手。复用 Task 4.1 detectPlatform 作 single source
// （避免重复实现平台检测）。仅改显示文案，不碰任何键盘事件处理 helper。

import { detectPlatform } from "./platform";

/** 当前是否 mac（消费 Task 4.1 的 detectPlatform，保持 single source）。 */
export function isMacPlatform(): boolean {
  return detectPlatform() === "macos";
}

/**
 * 平台感知快捷键显示。
 * @param mac   macOS 符号文案，如 "⌘B" / "⌘⇧O" / "⌘↵"
 * @param other 非 mac（Windows/Linux）文案，如 "Ctrl+B" / "Ctrl+Shift+O" / "Ctrl+Enter"
 * @returns isMacPlatform() ? mac : other
 *
 * 例：formatShortcut("⌘B", "Ctrl+B")
 *     formatShortcut("⌘⇧O", "Ctrl+Shift+O")
 *     formatShortcut("⌘↵", "Ctrl+Enter")
 */
export function formatShortcut(mac: string, other: string): string {
  return isMacPlatform() ? mac : other;
}
