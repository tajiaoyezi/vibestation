// MVP-14 Phase C §a11y · PaneSplitter 键盘 resize 测试
//
// 覆盖：
// - tabindex=0 + role=separator + aria-* 完整
// - ArrowRight (horizontal) 增加 ratio +0.01 · ArrowLeft 减小 0.01
// - Shift+Arrow 5% step
// - Home / End 跳到 RATIO_MIN / RATIO_MAX
// - Enter / Space 复位 0.5
// - 不在拖拽中才响应（拖拽中 keydown 被忽略）

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import { PaneSplitter } from "../../../src/panels/Terminal/PaneSplitter";

const fireKey = (el: Element, key: string, opts: KeyboardEventInit = {}) => {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, ...opts });
  el.dispatchEvent(event);
};

describe("PaneSplitter · keyboard resize", () => {
  beforeEach(() => cleanup());

  it("ARIA · tabindex=0 + role=separator + aria-orientation/value/min/max + aria-label", () => {
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={() => {}}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    expect(splitter.getAttribute("tabindex")).toBe("0");
    // horizontal direction = vertical splitter (separates left/right) · aria-orientation = vertical
    expect(splitter.getAttribute("aria-orientation")).toBe("vertical");
    expect(splitter.getAttribute("aria-valuenow")).toBe("50");
    expect(splitter.getAttribute("aria-valuemin")).toBe("10");
    expect(splitter.getAttribute("aria-valuemax")).toBe("90");
    expect(splitter.getAttribute("aria-label")).toContain("方向键");
  });

  it("ArrowRight (horizontal splitter) → ratio +0.01", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "ArrowRight");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", expect.closeTo(0.51, 5));
  });

  it("ArrowLeft (horizontal splitter) → ratio -0.01", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "ArrowLeft");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", expect.closeTo(0.49, 5));
  });

  it("Shift+ArrowRight → +0.05 step", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "ArrowRight", { shiftKey: true });
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", expect.closeTo(0.55, 5));
  });

  it("ArrowDown (vertical splitter) → +0.01 (轴向匹配)", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="vertical"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "ArrowDown");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", expect.closeTo(0.51, 5));
  });

  it("Home → RATIO_MIN (0.1)", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "Home");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", 0.1);
  });

  it("End → RATIO_MAX (0.9)", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "End");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", 0.9);
  });

  it("Enter → 0.5 复位 (从 0.7 调回中间)", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.7}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "Enter");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", 0.5);
  });

  it("Space → 0.5 复位 · 与 Enter 行为一致", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="vertical"
        ratio={0.3}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, " ");
    expect(onDragEnd).toHaveBeenCalledWith("pane-a", 0.5);
  });

  it("ArrowLeft 已在 RATIO_MIN → clamp · 不触发 onDragEnd（Δ < 0.001）", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.1}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "ArrowLeft");
    // ratio=0.1 已是 RATIO_MIN · 再减仍 clamp 到 0.1 · Δ=0 不触发回调
    expect(onDragEnd).not.toHaveBeenCalled();
  });

  it("非映射 key (a / Tab / Esc) 不触发 onDragEnd", () => {
    const onDragEnd = vi.fn();
    const { container } = render(() => (
      <PaneSplitter
        direction="horizontal"
        ratio={0.5}
        parentPaneId="pane-a"
        onDragEnd={onDragEnd}
      />
    ));
    const splitter = container.querySelector("[role='separator']")!;
    fireKey(splitter, "a");
    fireKey(splitter, "Tab");
    fireKey(splitter, "Escape");
    expect(onDragEnd).not.toHaveBeenCalled();
  });
});
