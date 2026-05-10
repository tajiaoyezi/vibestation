// MVP-14 Phase C · usePaneNavigation + usePaneMaximizeToggle hook 集成测试
//
// 在 jsdom 下挂 hook · 模拟 keydown 事件 · 验证：
// - §D.1 ⌘⌥Arrow 触发 onNavigate · ⌘ 单独 / ⌘⇧ Arrow 不触发
// - §D.4 找不到相邻 pane 时调 onNoOpFlash
// - §D.6 navigation hook 每次重新查 DOM rect（暴露给 caller 的 getActiveTabHost）
// - §E ⌘Enter 触发 onToggle · Esc 在 maximized 时触发 onExit
// - shouldSuppress true 时全部不触发

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import { createComponent, createSignal } from "solid-js";
import {
  usePaneNavigation,
  usePaneMaximizeToggle,
} from "../../../src/panels/Terminal/usePaneNavigation";

const fireWindowKey = (key: string, opts: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, ...opts });
  window.dispatchEvent(event);
};

const setMacPlatform = () => {
  Object.defineProperty(window.navigator, "platform", {
    value: "MacIntel",
    configurable: true,
  });
};

const setLinuxPlatform = () => {
  Object.defineProperty(window.navigator, "platform", {
    value: "Linux x86_64",
    configurable: true,
  });
};

describe("usePaneNavigation", () => {
  let originalPlatform: PropertyDescriptor | undefined;

  beforeEach(() => {
    cleanup();
    originalPlatform = Object.getOwnPropertyDescriptor(
      window.navigator,
      "platform",
    );
    setMacPlatform();

    // 在 jsdom 下挂 2 个 stub pane DOM · 让 collectPaneRectsFromDom 拿到 rect
    const host = document.createElement("div");
    host.setAttribute("data-pane-tab-host", "tab-1");
    host.style.cssText =
      "position:absolute;left:0;top:0;width:1000px;height:1000px;";
    const a = document.createElement("div");
    a.setAttribute("data-pane-id", "pane-a");
    Object.defineProperty(a, "getBoundingClientRect", {
      value: () => ({
        left: 0,
        top: 0,
        right: 500,
        bottom: 1000,
        width: 500,
        height: 1000,
        x: 0,
        y: 0,
        toJSON: () => ({}),
      }),
    });
    const b = document.createElement("div");
    b.setAttribute("data-pane-id", "pane-b");
    Object.defineProperty(b, "getBoundingClientRect", {
      value: () => ({
        left: 500,
        top: 0,
        right: 1000,
        bottom: 1000,
        width: 500,
        height: 1000,
        x: 500,
        y: 0,
        toJSON: () => ({}),
      }),
    });
    host.appendChild(a);
    host.appendChild(b);
    document.body.appendChild(host);
  });

  afterEach(() => {
    document.body.innerHTML = "";
    if (originalPlatform) {
      Object.defineProperty(window.navigator, "platform", originalPlatform);
    }
  });

  const mountNav = (overrides: {
    onNavigate?: (id: string) => void;
    onNoOpFlash?: () => void;
    isPaneMode?: () => boolean;
    getFocusedPaneId?: () => string | null;
    shouldSuppress?: () => boolean;
  }) => {
    const Wrapper = () => {
      usePaneNavigation({
        isPaneMode: overrides.isPaneMode ?? (() => true),
        getFocusedPaneId: overrides.getFocusedPaneId ?? (() => "pane-a"),
        getActiveTabHost: () =>
          document.querySelector('[data-pane-tab-host="tab-1"]'),
        onNavigate: overrides.onNavigate ?? (() => {}),
        onNoOpFlash: overrides.onNoOpFlash ?? (() => {}),
        shouldSuppress: overrides.shouldSuppress,
      });
      return <div />;
    };
    return render(() => createComponent(Wrapper, {}));
  };

  it("⌘⌥ ArrowRight from pane-a → onNavigate('pane-b')", () => {
    const onNavigate = vi.fn();
    mountNav({ onNavigate });
    fireWindowKey("ArrowRight", { metaKey: true, altKey: true });
    expect(onNavigate).toHaveBeenCalledWith("pane-b");
  });

  it("⌘ 单独 (无 Alt) ArrowRight → 不触发 onNavigate", () => {
    const onNavigate = vi.fn();
    mountNav({ onNavigate });
    fireWindowKey("ArrowRight", { metaKey: true });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("⌘⌥⇧ ArrowRight (Shift 加进来) → 不触发 onNavigate", () => {
    const onNavigate = vi.fn();
    mountNav({ onNavigate });
    fireWindowKey("ArrowRight", {
      metaKey: true,
      altKey: true,
      shiftKey: true,
    });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("⌘⌥ ArrowLeft from pane-a → onNoOpFlash (左侧无相邻 pane)", () => {
    const onNoOpFlash = vi.fn();
    mountNav({ onNoOpFlash });
    fireWindowKey("ArrowLeft", { metaKey: true, altKey: true });
    expect(onNoOpFlash).toHaveBeenCalled();
  });

  it("isPaneMode false (legacy single tab) → 不触发", () => {
    const onNavigate = vi.fn();
    const onNoOpFlash = vi.fn();
    mountNav({
      onNavigate,
      onNoOpFlash,
      isPaneMode: () => false,
    });
    fireWindowKey("ArrowRight", { metaKey: true, altKey: true });
    expect(onNavigate).not.toHaveBeenCalled();
    expect(onNoOpFlash).not.toHaveBeenCalled();
  });

  it("shouldSuppress true (paste pending) → 不触发", () => {
    const onNavigate = vi.fn();
    mountNav({
      onNavigate,
      shouldSuppress: () => true,
    });
    fireWindowKey("ArrowRight", { metaKey: true, altKey: true });
    expect(onNavigate).not.toHaveBeenCalled();
  });

  it("Linux Ctrl+Alt ArrowRight → onNavigate (跨平台 modifier)", () => {
    setLinuxPlatform();
    const onNavigate = vi.fn();
    mountNav({ onNavigate });
    fireWindowKey("ArrowRight", { ctrlKey: true, altKey: true });
    expect(onNavigate).toHaveBeenCalledWith("pane-b");
  });

  it("getFocusedPaneId null → 不触发 (caller 没拿到 focused)", () => {
    const onNavigate = vi.fn();
    mountNav({ onNavigate, getFocusedPaneId: () => null });
    fireWindowKey("ArrowRight", { metaKey: true, altKey: true });
    expect(onNavigate).not.toHaveBeenCalled();
  });
});

describe("usePaneMaximizeToggle", () => {
  let originalPlatform: PropertyDescriptor | undefined;

  beforeEach(() => {
    cleanup();
    originalPlatform = Object.getOwnPropertyDescriptor(
      window.navigator,
      "platform",
    );
    setMacPlatform();
  });

  afterEach(() => {
    if (originalPlatform) {
      Object.defineProperty(window.navigator, "platform", originalPlatform);
    }
  });

  const mountMax = (overrides: {
    onToggle?: () => void;
    onExit?: () => void;
    isMaximized?: () => boolean;
    isPaneMode?: () => boolean;
    hasFocusedPane?: () => boolean;
    shouldSuppress?: () => boolean;
  }) => {
    const Wrapper = () => {
      usePaneMaximizeToggle({
        isPaneMode: overrides.isPaneMode ?? (() => true),
        hasFocusedPane: overrides.hasFocusedPane ?? (() => true),
        isMaximized: overrides.isMaximized ?? (() => false),
        onToggle: overrides.onToggle ?? (() => {}),
        onExit: overrides.onExit ?? (() => {}),
        shouldSuppress: overrides.shouldSuppress,
      });
      return <div />;
    };
    return render(() => createComponent(Wrapper, {}));
  };

  it("⌘Enter → onToggle (macOS)", () => {
    const onToggle = vi.fn();
    mountMax({ onToggle });
    fireWindowKey("Enter", { metaKey: true });
    expect(onToggle).toHaveBeenCalled();
  });

  it("Linux Ctrl+Enter → onToggle", () => {
    setLinuxPlatform();
    const onToggle = vi.fn();
    mountMax({ onToggle });
    fireWindowKey("Enter", { ctrlKey: true });
    expect(onToggle).toHaveBeenCalled();
  });

  it("Esc 在 maximized 状态 → onExit", () => {
    const onExit = vi.fn();
    mountMax({ onExit, isMaximized: () => true });
    fireWindowKey("Escape");
    expect(onExit).toHaveBeenCalled();
  });

  it("Esc 在非 maximized 状态 → 不拦截 (let other handlers handle)", () => {
    const onExit = vi.fn();
    mountMax({ onExit, isMaximized: () => false });
    fireWindowKey("Escape");
    expect(onExit).not.toHaveBeenCalled();
  });

  it("isPaneMode false → ⌘Enter 不触发", () => {
    const onToggle = vi.fn();
    mountMax({ onToggle, isPaneMode: () => false });
    fireWindowKey("Enter", { metaKey: true });
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("hasFocusedPane false → ⌘Enter 不触发", () => {
    const onToggle = vi.fn();
    mountMax({ onToggle, hasFocusedPane: () => false });
    fireWindowKey("Enter", { metaKey: true });
    expect(onToggle).not.toHaveBeenCalled();
  });

  it("shouldSuppress true → 全部不触发", () => {
    const onToggle = vi.fn();
    const onExit = vi.fn();
    mountMax({
      onToggle,
      onExit,
      isMaximized: () => true,
      shouldSuppress: () => true,
    });
    fireWindowKey("Enter", { metaKey: true });
    fireWindowKey("Escape");
    expect(onToggle).not.toHaveBeenCalled();
    expect(onExit).not.toHaveBeenCalled();
  });

  it("⌘⇧Enter (Shift 加进来) → 不触发", () => {
    const onToggle = vi.fn();
    mountMax({ onToggle });
    fireWindowKey("Enter", { metaKey: true, shiftKey: true });
    expect(onToggle).not.toHaveBeenCalled();
  });
});
