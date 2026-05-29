// Task 4.2 · shortcut-display · RED stub（待 GREEN 实现）
//
// 平台感知快捷键显示助手。复用 Task 4.1 detectPlatform 作 single source。

import { detectPlatform } from "./platform";

/** 当前是否 mac（消费 Task 4.1 的 detectPlatform，保持 single source）。 */
export function isMacPlatform(): boolean {
  // RED stub · 未实现
  void detectPlatform;
  return false;
}

/**
 * 平台感知快捷键显示。
 * @param mac   macOS 符号文案，如 "⌘B"
 * @param other 非 mac（Windows/Linux）文案，如 "Ctrl+B"
 */
export function formatShortcut(mac: string, _other: string): string {
  // RED stub · 未实现（恒返回 mac）
  return mac;
}
