// MVP-15 Phase B · 主题切换 reactive RED
// 验证 theme-store 提供 SolidJS signal · DOM attribute 与 signal 双向同步

import { describe, it, expect, beforeEach } from "vitest";
import {
  setShikiTheme,
  useShikiTheme,
} from "../../../src/utils/shiki/theme-store";

describe("Shiki theme store", () => {
  beforeEach(() => {
    // 重置 DOM attribute · 防测试间污染
    document.documentElement.removeAttribute("data-shiki-theme");
    setShikiTheme("light");
  });

  it("setShikiTheme 应同时更新 DOM attribute 和 signal", () => {
    // 设 dark · 双向同步
    setShikiTheme("dark");
    expect(document.documentElement.getAttribute("data-shiki-theme")).toBe(
      "dark",
    );
    expect(useShikiTheme()).toBe("dark");
  });

  it("反向切回 light 也同步", () => {
    // 默认 light → dark → light · 验证可逆
    setShikiTheme("dark");
    setShikiTheme("light");
    expect(document.documentElement.getAttribute("data-shiki-theme")).toBe(
      "light",
    );
    expect(useShikiTheme()).toBe("light");
  });

  it("useShikiTheme 多次读取应返回最新值（reactive 基础）", () => {
    // SolidJS signal getter · 多次读取同一 tick 应一致
    setShikiTheme("dark");
    expect(useShikiTheme()).toBe("dark");
    expect(useShikiTheme()).toBe("dark");
  });
});
