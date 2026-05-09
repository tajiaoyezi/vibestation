// MVP-14 Phase B · SmartLayoutMenu 5 preset 测试
//
// 验证目标（spec §B.1 / §B.5）：
// - 渲染 5 个 preset 按钮
// - computeWillClose 覆盖 dualAi / tripleReview / quad
// - aiAndRunner 在 pane 不足时 disabled

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, cleanup, fireEvent } from "@solidjs/testing-library";
import {
  SmartLayoutMenu,
  type SmartLayoutPreset,
} from "../../../src/panels/Terminal/SmartLayoutMenu";
import type { PaneState, LayoutNode } from "../../../src/bindings";

function createPanes(count: number): PaneState[] {
  return Array.from({ length: count }, (_, i) => ({
    paneId: `pane-${i + 1}`,
    shell: `/bin/zsh`,
    cwd: `/home/user`,
    env: {},
    cols: 80,
    rows: 24,
  }));
}

const soloLayout: LayoutNode = {
  kind: "single",
  paneId: "pane-1",
};

describe("SmartLayoutMenu", () => {
  beforeEach(() => {
    cleanup();
  });

  it("renders 5 preset buttons", () => {
    const panes = createPanes(2);
    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-1"
        onApply={async () => {}}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    expect(buttons).toHaveLength(5);

    const names = Array.from(buttons).map(
      (btn) => btn.querySelector(".vs-smart-layout-preset-name")?.textContent,
    );
    expect(names).toContain("Solo");
    expect(names).toContain("AI + Runner");
    expect(names).toContain("Dual AI");
    expect(names).toContain("Triple Review");
    expect(names).toContain("Quad");
  });

  it("dualAi is always enabled regardless of pane count", () => {
    const panes = createPanes(1);
    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-1"
        onApply={async () => {}}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    // dualAi 是第 3 个按钮（索引 2）
    const dualAiBtn = buttons[2];
    expect(dualAiBtn).toBeDefined();
    expect(dualAiBtn.hasAttribute("disabled")).toBe(false);
  });

  it("aiAndRunner disabled when panes.length < 2", () => {
    const panes = createPanes(1);
    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-1"
        onApply={async () => {}}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    // aiAndRunner 是第 2 个按钮（索引 1）
    const aiRunnerBtn = buttons[1];
    expect(aiRunnerBtn).toBeDefined();
    expect(aiRunnerBtn.hasAttribute("disabled")).toBe(true);
  });

  it("tripleReview computeWillClose keeps first 3 panes", () => {
    const panes = createPanes(5);
    const onApply = vi.fn(async (_preset: SmartLayoutPreset) => {});

    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-1"
        onApply={onApply}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    // tripleReview 是第 4 个按钮（索引 3）
    const tripleBtn = buttons[3];
    expect(tripleBtn).toBeDefined();

    fireEvent.click(tripleBtn);

    // 选中后应该显示预览：关闭 2 个（pane-4, pane-5）
    const previewHeader = container.querySelector(
      ".vs-smart-layout-preview-header",
    );
    expect(previewHeader?.textContent).toContain("将关闭 2 个 Pane");
  });

  it("quad computeWillClose keeps first 4 panes", () => {
    const panes = createPanes(5);
    const onApply = vi.fn(async (_preset: SmartLayoutPreset) => {});

    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-1"
        onApply={onApply}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    // quad 是第 5 个按钮（索引 4）
    const quadBtn = buttons[4];
    expect(quadBtn).toBeDefined();

    fireEvent.click(quadBtn);

    // 选中后应该显示预览：关闭 1 个（pane-5）
    const previewHeader = container.querySelector(
      ".vs-smart-layout-preview-header",
    );
    expect(previewHeader?.textContent).toContain("将关闭 1 个 Pane");
  });

  it("solo keeps focused pane and closes others", () => {
    const panes = createPanes(3);
    const onApply = vi.fn(async (_preset: SmartLayoutPreset) => {});

    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-2"
        onApply={onApply}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    const soloBtn = buttons[0];
    expect(soloBtn).toBeDefined();

    fireEvent.click(soloBtn);

    // focusedPaneId = pane-2 · 预览应显示关闭 2 个（pane-1, pane-3）
    const previewHeader = container.querySelector(
      ".vs-smart-layout-preview-header",
    );
    expect(previewHeader?.textContent).toContain("将关闭 2 个 Pane");
  });

  it("calls onApply with selected preset when confirm clicked", async () => {
    const panes = createPanes(2);
    const onApply = vi.fn(async (_preset: SmartLayoutPreset) => {});

    const { container } = render(() => (
      <SmartLayoutMenu
        open={true}
        panes={panes}
        layout={soloLayout}
        focusedPaneId="pane-1"
        onApply={onApply}
        onClose={() => {}}
      />
    ));

    const buttons = container.querySelectorAll(".vs-smart-layout-preset");
    fireEvent.click(buttons[1]); // AI + Runner

    const applyBtn = container.querySelector(".vs-smart-layout-btn-apply");
    expect(applyBtn).toBeDefined();
    fireEvent.click(applyBtn!);

    // 等待 async onApply
    await new Promise((resolve) => setTimeout(resolve, 10));

    expect(onApply).toHaveBeenCalledWith("aiAndRunner");
  });
});
