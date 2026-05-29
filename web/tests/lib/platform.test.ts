// Task 4.1 · platform-windows-class
//
// 覆盖 spec §6 Acceptance：
// - AC1 SCEN-4.1.1 Windows 发 platform-windows + data-platform="windows"
// - AC2 SCEN-4.1.2 mac/Linux 零回归 + 新增 data-platform
// - AC3 SCEN-4.1.3 userAgentData 补充信号 + unknown 不发 class / 不设属性
// detectPlatform 是纯函数（接收参数），可直接断言，避免依赖全局 jsdom 状态。

import { describe, it, expect } from "vitest";
import { detectPlatform, applyPlatformClass } from "../../src/lib/platform";

describe("detectPlatform · 纯函数平台判定", () => {
  it("TEST-4.1.1: Win32 / Windows 大小写差异均返回 windows", () => {
    expect(detectPlatform("Win32")).toBe("windows");
    expect(detectPlatform("Windows")).toBe("windows");
    expect(detectPlatform("WIN32")).toBe("windows");
  });

  it("TEST-4.1.2: mac / Linux 零回归", () => {
    expect(detectPlatform("MacIntel")).toBe("macos");
    expect(detectPlatform("Linux x86_64")).toBe("linux");
  });

  it("TEST-4.1.3: userAgentData 补充信号 + unknown 兜底", () => {
    // navigator.platform 为空，靠 userAgentData 兜底
    expect(detectPlatform("", "Windows")).toBe("windows");
    expect(detectPlatform("", "macOS")).toBe("macos");
    expect(detectPlatform("", "Linux")).toBe("linux");
    // 未知形态
    expect(detectPlatform("FreeBSD", undefined)).toBe("unknown");
    expect(detectPlatform("", undefined)).toBe("unknown");
  });
});

describe("applyPlatformClass · DOM 副作用", () => {
  const makeRoot = (): HTMLElement => document.createElement("html");

  it("TEST-4.1.1: windows 发 platform-windows class + data-platform=windows", () => {
    const root = makeRoot();
    applyPlatformClass(root, "windows");
    expect(root.classList.contains("platform-windows")).toBe(true);
    expect(root.getAttribute("data-platform")).toBe("windows");
  });

  it("TEST-4.1.2: macos / linux class 零回归 + 新增 data-platform", () => {
    const macRoot = makeRoot();
    applyPlatformClass(macRoot, "macos");
    expect(macRoot.classList.contains("platform-macos")).toBe(true);
    expect(macRoot.getAttribute("data-platform")).toBe("macos");

    const linuxRoot = makeRoot();
    applyPlatformClass(linuxRoot, "linux");
    expect(linuxRoot.classList.contains("platform-linux")).toBe(true);
    expect(linuxRoot.getAttribute("data-platform")).toBe("linux");
  });

  it("TEST-4.1.3: unknown 不发任何 class / 不设属性", () => {
    const root = makeRoot();
    applyPlatformClass(root, "unknown");
    expect(root.classList.length).toBe(0);
    expect(root.getAttribute("data-platform")).toBe(null);
  });
});
