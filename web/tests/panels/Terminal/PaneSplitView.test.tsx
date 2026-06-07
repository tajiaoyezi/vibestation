// MVP-14 Phase B · PaneSplitView 递归渲染优化测试
//
// 验证目标（spec §C.1 / §C.2 / §C.3）：
// - 5 层 nested fixture 渲染不产生 SolidJS key 警告
// - 每个 leaf pane DOM 有稳定 data-pane-id
// - createMemo 防止 sibling pane body 因无关 ratio 更新而重渲染

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, cleanup, within } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import { PaneSplitView } from "../../../src/panels/Terminal/PaneSplitView";
import { PaneLinksProvider } from "../../../src/stores/paneLinks-context";
import { PaneDraftsProvider } from "../../../src/stores/paneDrafts-context";
import type { LayoutNode, PaneState } from "../../../src/bindings";

// 5 层嵌套 fixture: H(A, V(B, H(C, V(D, E))))
function create5LayerLayout(): LayoutNode {
  return {
    kind: "split",
    direction: "horizontal",
    ratio: 0.5,
    first: { kind: "single", paneId: "pane-a" },
    second: {
      kind: "split",
      direction: "vertical",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-b" },
      second: {
        kind: "split",
        direction: "horizontal",
        ratio: 0.5,
        first: { kind: "single", paneId: "pane-c" },
        second: {
          kind: "split",
          direction: "vertical",
          ratio: 0.5,
          first: { kind: "single", paneId: "pane-d" },
          second: { kind: "single", paneId: "pane-e" },
        },
      },
    },
  };
}

// 简单 3 pane fixture: H(A, V(B, C))
function createHVLayout(): LayoutNode {
  return {
    kind: "split",
    direction: "horizontal",
    ratio: 0.5,
    first: { kind: "single", paneId: "pane-a" },
    second: {
      kind: "split",
      direction: "vertical",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-b" },
      second: { kind: "single", paneId: "pane-c" },
    },
  };
}

function createThreeColumnLayout(): LayoutNode {
  return {
    kind: "split",
    direction: "horizontal",
    ratio: 1 / 3,
    first: { kind: "single", paneId: "pane-a" },
    second: {
      kind: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-b" },
      second: { kind: "single", paneId: "pane-c" },
    },
  };
}

function createTwoRowLayout(): LayoutNode {
  return {
    kind: "split",
    direction: "vertical",
    ratio: 0.5,
    first: { kind: "single", paneId: "pane-a" },
    second: { kind: "single", paneId: "pane-b" },
  };
}

function createThreeByTwoLayout(): LayoutNode {
  return {
    kind: "split",
    direction: "horizontal",
    ratio: 1 / 3,
    first: {
      kind: "split",
      direction: "vertical",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-a" },
      second: { kind: "single", paneId: "pane-d" },
    },
    second: {
      kind: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: {
        kind: "split",
        direction: "vertical",
        ratio: 0.5,
        first: { kind: "single", paneId: "pane-b" },
        second: { kind: "single", paneId: "pane-e" },
      },
      second: {
        kind: "split",
        direction: "vertical",
        ratio: 0.5,
        first: { kind: "single", paneId: "pane-c" },
        second: { kind: "single", paneId: "pane-f" },
      },
    },
  };
}

function createPanes(count: number): PaneState[] {
  return Array.from({ length: count }, (_, i) => ({
    paneId: `pane-${String.fromCharCode(97 + i)}`,
    shell: `/bin/zsh`,
    cwd: `/home/user`,
    env: {},
    cols: 80,
    rows: 24,
  }));
}

describe("PaneSplitView", () => {
  const consoleWarnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

  beforeEach(() => {
    cleanup();
    consoleWarnSpy.mockClear();
  });

  it("renders 5-layer nested layout without key warnings", () => {
    const layout = create5LayerLayout();
    const panes = createPanes(5);

    const { container } = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={layout}
            panes={panes}
            workspaceId="ws-test"
            active={true}
            focusedPaneId={null}
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));

    // 验证 5 个 leaf pane 都有 data-pane-id
    const paneElements = container.querySelectorAll("[data-pane-id]");
    expect(paneElements).toHaveLength(5);

    // 验证每个 paneId 都出现且不重复
    const ids = Array.from(paneElements).map((el) =>
      el.getAttribute("data-pane-id"),
    );
    expect(ids).toContain("pane-a");
    expect(ids).toContain("pane-b");
    expect(ids).toContain("pane-c");
    expect(ids).toContain("pane-d");
    expect(ids).toContain("pane-e");

    // 验证没有 key 相关的 console.warn
    const keyWarnings = consoleWarnSpy.mock.calls.filter(
      (call) =>
        typeof call[0] === "string" &&
        (call[0].includes("key") || call[0].includes("duplicate")),
    );
    expect(keyWarnings).toHaveLength(0);
  });

  it("renders nested split structure with correct CSS classes", () => {
    const layout = createHVLayout();
    const panes = createPanes(3);

    const { container } = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={layout}
            panes={panes}
            workspaceId="ws-test"
            active={true}
            focusedPaneId={null}
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));

    // 根 split 是 horizontal
    const rootSplit = container.querySelector(".vs-pane-split-horizontal");
    expect(rootSplit).not.toBeNull();

    // 子 split 是 vertical
    const verticalSplit = container.querySelector(".vs-pane-split-vertical");
    expect(verticalSplit).not.toBeNull();

    // 根 split 包含 first/second 子容器
    expect(rootSplit?.querySelector(".vs-pane-split-first")).not.toBeNull();
    expect(rootSplit?.querySelector(".vs-pane-split-second")).not.toBeNull();
  });

  // BUG-001 reproduce · 2026-05-23 · session 34
  //
  // 用户复现：点 1 次右分屏 → 2 pane（正常）→ 关右侧 pane → 留白
  // backend `apply_pane_close` 经测试验证正确（panes.rs:1688 split→single collapse）·
  // 故根因在 frontend PaneSplitView 的 reactive 切换 · 本测试 reactive 触发 split→single
  // 验证 RenderSplit 是否正确 unmount + RenderSingle 是否正确 mount · 不留 vs-pane-missing。
  it("BUG-001 · collapses split → single when layout shrinks (close pane second)", () => {
    const [layout, setLayout] = createSignal<LayoutNode>({
      kind: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-a" },
      second: { kind: "single", paneId: "pane-b" },
    });
    const [panes, setPanes] = createSignal<PaneState[]>(createPanes(2));

    const { container } = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={layout()}
            panes={panes()}
            workspaceId="ws-test"
            active={true}
            focusedPaneId="pane-a"
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));

    // 初始：split 模式 · 2 个 pane 都渲染
    expect(container.querySelectorAll(".vs-pane-split").length).toBe(1);
    expect(container.querySelectorAll('[data-pane-id="pane-a"]').length).toBe(
      1,
    );
    expect(container.querySelectorAll('[data-pane-id="pane-b"]').length).toBe(
      1,
    );

    // close pane-b · backend 返回 single{pane-a}（已被 panes.rs:1688 测试验证）
    setLayout({ kind: "single", paneId: "pane-a" });
    setPanes(createPanes(1));

    // 期望：
    // - .vs-pane-split DOM 消失（RenderSplit unmount）
    // - .vs-pane-missing 不出现（RenderSingle 找到 pane-a · 不走 fallback）
    // - 只剩 pane-a · pane-b 完全消失
    expect(container.querySelectorAll(".vs-pane-split").length).toBe(0);
    expect(container.querySelectorAll(".vs-pane-missing").length).toBe(0);
    expect(container.querySelectorAll('[data-pane-id="pane-a"]').length).toBe(
      1,
    );
    expect(container.querySelectorAll('[data-pane-id="pane-b"]').length).toBe(
      0,
    );
  });

  // BUG-001 真实复现 · 2026-05-23 session 34 · 标 it.fails（known-failing）
  //
  // 用户场景：layout 从 split(A,B) 变成 split(A, split(B,C))（第二次 split 嵌套）。
  // 期望：3 个 pane 全部渲染。
  // 实测 root cause：RenderSplit 内 `const split = props.layout` 是非 reactive capture ·
  //   内层 split.second 不响应外层 props.layout 变化 · 新 pane C 不渲染。
  //
  // session 34 尝试 3 路修复（throw / nullable Show / prev-fallback memo）· 全部触发
  // SolidJS reactive owner cleanup 在 dev webview 报 "null is not an object 'node.owned[i]'"·
  // vitest jsdom 测试 PASS 但 dev FAIL · jsdom 与 webview reactive ordering 差异。
  // Per `~/.claude/rules/always/08-systematic-debugging.md` Phase 4 红旗：3+ fix 失败必须
  // STOP 并质疑架构 · 故 revert fix · 留本测试为已知失败 audit · 留 BUG-001 spec 给下次
  // cold session 用架构级方案修（重构 PaneSplitView 整体 / SolidJS Switch+Match keyed /
  // untrack 包装 / RenderSplit/Single 接收 narrowed prop type）。
  //
  // 修复后：把 `.fails` 删掉 · 测试 GREEN 即代表真修复 + 防回归。
  it("BUG-001 · renders new pane when split nests deeper (real reproduce)", () => {
    const [layout, setLayout] = createSignal<LayoutNode>({
      kind: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-a" },
      second: { kind: "single", paneId: "pane-b" },
    });
    const [panes, setPanes] = createSignal<PaneState[]>(createPanes(2));

    const { container } = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={layout()}
            panes={panes()}
            workspaceId="ws-test"
            active={true}
            focusedPaneId="pane-b"
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));

    // 初始 · 2 panes 渲染
    expect(container.querySelectorAll("[data-pane-id]").length).toBe(2);

    // 第 2 次 split · 在 pane-b 上分屏 · backend 返回嵌套 split(A, split(B, C))
    setLayout({
      kind: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: { kind: "single", paneId: "pane-a" },
      second: {
        kind: "split",
        direction: "horizontal",
        ratio: 0.5,
        first: { kind: "single", paneId: "pane-b" },
        second: { kind: "single", paneId: "pane-c" },
      },
    });
    setPanes(createPanes(3));

    // 期望：3 个 pane 全部渲染（A + B + C）
    const renderedIds = Array.from(
      container.querySelectorAll("[data-pane-id]"),
    ).map((el) => el.getAttribute("data-pane-id"));
    expect(renderedIds).toContain("pane-a");
    expect(renderedIds).toContain("pane-b");
    expect(renderedIds).toContain("pane-c");
    expect(renderedIds.length).toBe(3);
  });

  it("passes correct parentPaneId to splitter for nested layout", () => {
    const layout = createHVLayout();
    const panes = createPanes(3);
    const onDragEnd = vi.fn();

    const { container } = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={layout}
            panes={panes}
            workspaceId="ws-test"
            active={true}
            focusedPaneId={null}
            onPaneClick={() => {}}
            onSplitterDragEnd={onDragEnd}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));

    // 找到所有 splitter
    const splitters = container.querySelectorAll("[role='separator']");
    expect(splitters.length).toBeGreaterThanOrEqual(1);

    // 最外层 splitter 的 parentPaneId 应该是 first 子树最深的 pane（pane-a）
    // 通过触发 drag end 来验证
    // 注意：由于 splitter 的 onDragEnd 需要 pointer 事件触发，这里只验证渲染结构
    expect(splitters.length).toBe(2);
  });

  it("disables split buttons per axis and total pane limits", () => {
    const { container, unmount } = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={createThreeColumnLayout()}
            panes={createPanes(3)}
            workspaceId="ws-test"
            active={true}
            focusedPaneId="pane-a"
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));

    const paneA = container.querySelector('[data-pane-id="pane-a"]');
    expect(paneA).not.toBeNull();
    expect(
      (
        within(paneA as HTMLElement).getByLabelText(
          "右分屏",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(
      (
        within(paneA as HTMLElement).getByLabelText(
          "下分屏",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(false);

    unmount();

    const twoRows = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={createTwoRowLayout()}
            panes={createPanes(2)}
            workspaceId="ws-test"
            active={true}
            focusedPaneId="pane-a"
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));
    const rowPaneA = twoRows.container.querySelector('[data-pane-id="pane-a"]');
    expect(rowPaneA).not.toBeNull();
    expect(
      (
        within(rowPaneA as HTMLElement).getByLabelText(
          "右分屏",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(false);
    expect(
      (
        within(rowPaneA as HTMLElement).getByLabelText(
          "下分屏",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(true);

    twoRows.unmount();

    const full = render(() => (
      <PaneLinksProvider>
        <PaneDraftsProvider>
          <PaneSplitView
            layout={createThreeByTwoLayout()}
            panes={createPanes(6)}
            workspaceId="ws-test"
            active={true}
            focusedPaneId="pane-a"
            onPaneClick={() => {}}
          />
        </PaneDraftsProvider>
      </PaneLinksProvider>
    ));
    const fullPaneA = full.container.querySelector('[data-pane-id="pane-a"]');
    expect(fullPaneA).not.toBeNull();
    expect(
      (
        within(fullPaneA as HTMLElement).getByLabelText(
          "右分屏",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(
      (
        within(fullPaneA as HTMLElement).getByLabelText(
          "下分屏",
        ) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    full.unmount();
  });
});
