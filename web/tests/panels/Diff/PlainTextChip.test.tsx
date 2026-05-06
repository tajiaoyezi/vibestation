// MVP-15 Phase B · PlainTextChip 单测
//
// 验证 spec §B.6 / §E.5：
// - 已识别 lang · chip 不显示
// - 不识别 lang · chip 显示 "Plain text" · 含 hover tooltip

import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import { PlainTextChip } from "../../../src/panels/Diff/PlainTextChip";

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

  it("不识别 lang（自定义后缀）· 渲染 chip 含 'Plain text'", () => {
    // 不在 Tier 1 后缀映射 · chip 显示
    const { container } = render(() => (
      <PlainTextChip filePath="data/blob.unknown_ext" />
    ));
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("Plain text");
  });

  it("无后缀文件（如 Dockerfile）· 渲染 chip 含 'Plain text'", () => {
    // Tier 1 不含 dockerfile · 走纯文本降级
    const { container } = render(() => <PlainTextChip filePath="Dockerfile" />);
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip).not.toBeNull();
    expect(chip!.textContent).toContain("Plain text");
  });

  it("chip 含 title / aria-label 解释（无障碍 + 鼠标 hover）", () => {
    // 视觉降级时给用户解释 · 不弹 toast 避免烦扰（spec UI 引用 line 217）
    const { container } = render(() => <PlainTextChip filePath="binary.xyz" />);
    const chip = container.querySelector(".vs-diff-plain-text-chip");
    expect(chip!.getAttribute("title")).toBeTruthy();
    expect(chip!.getAttribute("title")).toContain("语法高亮");
  });
});
