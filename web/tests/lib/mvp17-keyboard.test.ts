import { describe, it, expect } from "vitest";
import {
  matchesPopToExternalShortcut,
  matchesDetachPaneShortcut,
} from "../../src/lib/mvp17-keyboard";

describe.skip("mvp17-keyboard.ts · 快捷键匹配", () => {
  it("⌘⇧O 匹配 Pop to External", () => {
    const event = new KeyboardEvent("keydown", {
      key: "o",
      metaKey: true,
      shiftKey: true,
    });
    expect(matchesPopToExternalShortcut(event)).toBe(true);
  });

  it("⌘⇧D 匹配 Detach Pane", () => {
    const event = new KeyboardEvent("keydown", {
      key: "d",
      metaKey: true,
      shiftKey: true,
    });
    expect(matchesDetachPaneShortcut(event)).toBe(true);
  });

  it("纯 ⌘O 不匹配（缺少 shift）", () => {
    const event = new KeyboardEvent("keydown", {
      key: "o",
      metaKey: true,
      shiftKey: false,
    });
    expect(matchesPopToExternalShortcut(event)).toBe(false);
  });

  it("Ctrl⇧O 不匹配（macOS 模式）", () => {
    // navigator.platform mock 为 mac
    Object.defineProperty(navigator, "platform", {
      value: "MacIntel",
      configurable: true,
    });
    const event = new KeyboardEvent("keydown", {
      key: "o",
      ctrlKey: true,
      shiftKey: true,
    });
    expect(matchesPopToExternalShortcut(event)).toBe(false);
  });

  it("Ctrl⇧O 匹配（Linux 模式）", () => {
    Object.defineProperty(navigator, "platform", {
      value: "Linux x86_64",
      configurable: true,
    });
    const event = new KeyboardEvent("keydown", {
      key: "o",
      ctrlKey: true,
      shiftKey: true,
    });
    expect(matchesPopToExternalShortcut(event)).toBe(true);
  });
});
