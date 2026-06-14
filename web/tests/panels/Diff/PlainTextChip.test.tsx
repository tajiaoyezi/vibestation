// MVP-15 Phase B · PlainTextChip 单测
//
// 验证 spec §B.6 / §E.5：
// - 已识别 lang · chip 不显示
// - 不识别 lang · chip 显示 "Plain text" · 含 hover tooltip

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@solidjs/testing-library";
import type { AppSettings } from "../../../src/bindings/AppSettings";

const { mockAppSettings, resetMockSettings } = vi.hoisted(() => {
  const defaultFixture = (): AppSettings => ({
    language: "en",
    theme: "dark",
    uiFontFamily: "Inter",
    fontFamily: "JetBrains Mono",
    fontSize: 14,
    defaultShell: "/bin/bash",
    pasteProtection: true,
    telemetryOptIn: null,
    gitUserName: null,
    gitUserEmail: null,
    bgOpacity: 0.85,
    bgBlur: 20,
    windowPaddingX: 2,
    windowPaddingY: 2,
    cursorStyle: "block",
    cursorBlink: false,
    unfocusedPaneOpacity: 0.7,
    ptyPoolEnabled: true,
    ptyPoolSize: 1,
    primaryWidth: 236,
    secondaryWidth: 400,
    bottomHeight: 240,
    externalTermPreferred: null,
    externalTermDontAskAgain: false,
  });
  const mockAppSettings: AppSettings = defaultFixture();
  return {
    mockAppSettings,
    resetMockSettings: () => {
      Object.assign(mockAppSettings, defaultFixture());
    },
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "settings_get") {
      return { ...mockAppSettings };
    }
    return null;
  }),
}));

import { reloadSettings } from "../../../src/stores/settings";
import { PlainTextChip } from "../../../src/panels/Diff/PlainTextChip";

beforeEach(async () => {
  resetMockSettings();
  mockAppSettings.language = "zh-Hans";
  await reloadSettings();
});

afterEach(cleanup);

describe("PlainTextChip", () => {
  it("已识别 lang（.ts）· 不渲染 chip", () => {
    // typescript 后缀 · guessLanguageFromPath 返回 'typescript' · chip 应不出现
    const { container } = render(() => <PlainTextChip filePath="src/foo.ts" />);
    expect(container.querySelector(".vs-diff-plain-text-chip")).toBeNull();
  });

  it("已识别 lang（.rs）· 不渲染 chip", () => {
    // rust 后缀同理
    const { container } = render(() => (
      <PlainTextChip filePath="crates/core/lib.rs" />
    ));
    expect(container.querySelector(".vs-diff-plain-text-chip")).toBeNull();
  });

  it("不识别 lang（自定义后缀）· 渲染 chip 含 '纯文本'", () => {
    // 不在 Tier 1 后缀映射 · chip 显示
    const { container } = render(() => (
      <PlainTextChip filePath="data/blob.unknown_ext" />
    ));
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("纯文本");
  });

  it("无后缀文件（如 Dockerfile）· 渲染 chip 含 '纯文本'", () => {
    // Tier 1 不含 dockerfile · 走纯文本降级
    const { container } = render(() => <PlainTextChip filePath="Dockerfile" />);
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("纯文本");
  });

  it("chip 含 title / aria-label 解释（无障碍 + 鼠标 hover）", () => {
    // 视觉降级时给用户解释 · 不弹 toast 避免烦扰（spec UI 引用 line 217）
    const { container } = render(() => <PlainTextChip filePath="binary.xyz" />);
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip!.getAttribute("title")).toBeTruthy();
    expect(chip!.getAttribute("title")).toContain("语法高亮");
  });

  it("large-file 模式优先显示大文件提示（即使语言可识别）", () => {
    const { container } = render(() => (
      <PlainTextChip
        filePath="src/foo.ts"
        reason="large-file"
        fileSize={55 * 1024 * 1024}
      />
    ));
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("大文件");
    expect(chip!.textContent).toContain("语法高亮已禁用");
  });

  it("large-file 无 fileSize 时也显示禁用提示", () => {
    const { container } = render(() => (
      <PlainTextChip filePath="src/foo.ts" reason="large-file" />
    ));
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("大文件");
    expect(chip!.textContent).toContain("语法高亮已禁用");
  });
});
