// Task 4.2 · shortcut-display
//
// 覆盖 spec §6 Acceptance：
// - AC1 SCEN-4.2.1 formatShortcut mac/非 mac 双路径 + isMacPlatform 平台判定
// - AC4 SCEN-4.2.4 mac 文案逐字零回归（mac 路径返回原 ⌘ 符号）
// isMacPlatform 消费 Task 4.1 detectPlatform（读 navigator.platform），用
// Object.defineProperty mock 平台（沿用 mvp17-keyboard.test.ts 既有模式）。

import { describe, it, expect, beforeEach } from "vitest";
import { formatShortcut, isMacPlatform } from "../../src/lib/format-shortcut";

const mockPlatform = (value: string): void => {
  Object.defineProperty(navigator, "platform", {
    value,
    configurable: true,
  });
};

describe("isMacPlatform · 平台判定（消费 detectPlatform）", () => {
  it("TEST-4.2.1: mac 返回 true", () => {
    mockPlatform("MacIntel");
    expect(isMacPlatform()).toBe(true);
  });

  it("TEST-4.2.1: Windows 返回 false", () => {
    mockPlatform("Win32");
    expect(isMacPlatform()).toBe(false);
  });

  it("TEST-4.2.1: Linux 返回 false", () => {
    mockPlatform("Linux x86_64");
    expect(isMacPlatform()).toBe(false);
  });
});

describe("formatShortcut · 平台感知快捷键显示", () => {
  describe("mac 路径（逐字零回归）", () => {
    beforeEach(() => mockPlatform("MacIntel"));

    it("TEST-4.2.4: 单键 ⌘B 不变", () => {
      expect(formatShortcut("⌘B", "Ctrl+B")).toBe("⌘B");
    });

    it("TEST-4.2.4: 组合键 ⌘⇧O 不变", () => {
      expect(formatShortcut("⌘⇧O", "Ctrl+Shift+O")).toBe("⌘⇧O");
    });

    it("TEST-4.2.4: ⌘⌃W / ⌘↵ / ⌘\\ / ⌘⇧\\ / ⌘, 不变", () => {
      expect(formatShortcut("⌘⌃W", "Ctrl+Shift+W")).toBe("⌘⌃W");
      expect(formatShortcut("⌘↵", "Ctrl+↵")).toBe("⌘↵");
      expect(formatShortcut("⌘\\", "Ctrl+\\")).toBe("⌘\\");
      expect(formatShortcut("⌘⇧\\", "Ctrl+Shift+\\")).toBe("⌘⇧\\");
      expect(formatShortcut("⌘,", "Ctrl+,")).toBe("⌘,");
    });
  });

  describe("非 mac 路径（Windows / Linux 文案）", () => {
    it("TEST-4.2.1: Windows 单键返回 Ctrl+B", () => {
      mockPlatform("Win32");
      expect(formatShortcut("⌘B", "Ctrl+B")).toBe("Ctrl+B");
    });

    it("TEST-4.2.1: Windows 组合键 Ctrl+Shift+O / Ctrl+Shift+W", () => {
      mockPlatform("Win32");
      expect(formatShortcut("⌘⇧O", "Ctrl+Shift+O")).toBe("Ctrl+Shift+O");
      expect(formatShortcut("⌘⌃W", "Ctrl+Shift+W")).toBe("Ctrl+Shift+W");
    });

    it("TEST-4.2.1: Linux 同走非 mac 文案", () => {
      mockPlatform("Linux x86_64");
      expect(formatShortcut("⌘2", "Ctrl+2")).toBe("Ctrl+2");
      expect(formatShortcut("⌘J", "Ctrl+J")).toBe("Ctrl+J");
    });
  });
});
