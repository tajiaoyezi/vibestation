/**
 * PaneSplitView · MVP-05 Phase C
 *
 * 递归渲染 [`LayoutNode`] 树：
 * - `Single` → 渲染 [`PaneTerminal`]
 * - `Split` → 渲染两个子 [`PaneSplitView`] · 中间夹一个 [`PaneSplitter`] 分隔条
 *
 * §H 布局模型：MVP-05 限制单层嵌套（深度 ≤ 2 · 4 panes max · 由 backend `validate_mvp_05`
 * 强制 · 前端不重复校验 · 但用 layout depth 显示 toast 提示）。
 *
 * Phase C scaffolding · 暂未与 [`Terminal.tsx`] 集成 · 集成留 PR #145。
 */
import { Show, type Component } from "solid-js";
import type { LayoutNode, PaneState, SplitDir } from "../../bindings";
import { PaneSplitter } from "./PaneSplitter";
import { PaneTerminal, type PaneTerminalApi } from "./PaneTerminal";

type PaneSplitViewProps = {
  layout: LayoutNode;
  panes: PaneState[];
  active: boolean;
  focusedPaneId: string | null;
  onPaneClick: (paneId: string) => void;
  onSplitterDragEnd?: (parentPaneId: string, ratio: number) => void;
  onPaneError?: (paneId: string, message: string) => void;
  onPaneExit?: (paneId: string, exitCode: number | null) => void;
  onRegisterPaneApi?: (paneId: string, api: PaneTerminalApi) => void;
  onUnregisterPaneApi?: (paneId: string) => void;
  // MVP-05 visible toolbar · 按钮触发 split / close · wire 到 Terminal.tsx
  onPaneSplit?: (direction: SplitDir, paneId: string) => void;
  onPaneClose?: (paneId: string) => void;
  // cmd+V paste guard 透传 · 与 menu paste 路径共享 setPendingPaste 流程
  onPanePasteRequest?: (paneId: string, text: string) => void;
};

const findPane = (panes: PaneState[], paneId: string): PaneState | null =>
  panes.find((p) => p.paneId === paneId) ?? null;

/**
 * 找到 split 节点 first 子树最深处的 pane_id · 用作拖拽 splitter 时的 parent_pane_id key。
 * 与 `pane_service.update_split_ratio` 的语义保持一致（first 子树包含 parent_pane_id 的 Split
 * 节点是目标）。
 */
const firstPaneInSubtree = (layout: LayoutNode): string => {
  if (layout.kind === "single") return layout.paneId;
  return firstPaneInSubtree(layout.first);
};

export const PaneSplitView: Component<PaneSplitViewProps> = (props) => {
  return (
    <Show
      when={props.layout.kind === "single"}
      fallback={<RenderSplit {...props} />}
    >
      <RenderSingle {...props} />
    </Show>
  );
};

const RenderSingle: Component<PaneSplitViewProps> = (props) => {
  if (props.layout.kind !== "single") return null;
  const paneId = props.layout.paneId;
  const pane = () => findPane(props.panes, paneId);

  return (
    // fallback 用空 placeholder · 防 layout 切换瞬间 SolidJS 在旧 RenderSingle 子树 dispose
    // 完成前重 evaluate pane() = null · 闪现 "Pane 缺失" 字样几 ms。
    <Show when={pane()} fallback={<div class="vs-pane-missing" />}>
      {(p) => (
        <PaneTerminal
          paneId={paneId}
          shell={p().shell}
          cwd={p().cwd}
          active={props.active}
          focused={props.focusedPaneId === paneId}
          onClick={props.onPaneClick}
          onExit={props.onPaneExit}
          onError={props.onPaneError}
          onRegisterApi={props.onRegisterPaneApi}
          onUnregisterApi={props.onUnregisterPaneApi}
          onSplit={props.onPaneSplit}
          onClose={props.onPaneClose}
          onPasteRequest={props.onPanePasteRequest}
        />
      )}
    </Show>
  );
};

const RenderSplit: Component<PaneSplitViewProps> = (props) => {
  if (props.layout.kind !== "split") return null;
  const split = props.layout;
  const parentPaneId = firstPaneInSubtree(split.first);

  return (
    <div
      class={`vs-pane-split vs-pane-split-${split.direction}`}
      style={{
        "--vs-pane-ratio": String(split.ratio),
      }}
    >
      <div class="vs-pane-split-first">
        <PaneSplitView
          layout={split.first}
          panes={props.panes}
          active={props.active}
          focusedPaneId={props.focusedPaneId}
          onPaneClick={props.onPaneClick}
          onSplitterDragEnd={props.onSplitterDragEnd}
          onPaneError={props.onPaneError}
          onPaneExit={props.onPaneExit}
          onRegisterPaneApi={props.onRegisterPaneApi}
          onUnregisterPaneApi={props.onUnregisterPaneApi}
          onPaneSplit={props.onPaneSplit}
          onPaneClose={props.onPaneClose}
          onPanePasteRequest={props.onPanePasteRequest}
        />
      </div>
      <PaneSplitter
        direction={split.direction}
        ratio={split.ratio}
        parentPaneId={parentPaneId}
        onDragEnd={props.onSplitterDragEnd}
      />
      <div class="vs-pane-split-second">
        <PaneSplitView
          layout={split.second}
          panes={props.panes}
          active={props.active}
          focusedPaneId={props.focusedPaneId}
          onPaneClick={props.onPaneClick}
          onSplitterDragEnd={props.onSplitterDragEnd}
          onPaneError={props.onPaneError}
          onPaneExit={props.onPaneExit}
          onRegisterPaneApi={props.onRegisterPaneApi}
          onUnregisterPaneApi={props.onUnregisterPaneApi}
          onPaneSplit={props.onPaneSplit}
          onPaneClose={props.onPaneClose}
          onPanePasteRequest={props.onPanePasteRequest}
        />
      </div>
    </div>
  );
};
