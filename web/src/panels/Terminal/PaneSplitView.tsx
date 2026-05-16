/**
 * PaneSplitView · MVP-14 Phase B
 *
 * 递归渲染 [`LayoutNode`] 树：
 * - `Single` → 渲染 [`PaneTerminal`]
 * - `Split` → 渲染两个子 [`PaneSplitView`] · 中间夹一个 [`PaneSplitter`] 分隔条
 *
 * §H 布局模型：MVP-14 支持最多 5 层嵌套（backend `MAX_LAYOUT_SPLIT_DEPTH = 5`）。
 * 前端用 `createMemo` 派生 split node 属性 · 避免 sibling pane body 因无关 ratio
 * 更新而重渲染。
 */
import { createMemo, Show, type Component } from "solid-js";
import type { LayoutNode, PaneState, SplitDir } from "../../bindings";
import { PaneSplitter } from "./PaneSplitter";
import { PaneTerminal, type PaneTerminalApi } from "./PaneTerminal";
import { DetachedPlaceholder } from "./DetachedPlaceholder";
import { detachedPanes } from "../../lib/pane-detach";

type PaneSplitViewProps = {
  layout: LayoutNode;
  panes: PaneState[];
  /** MVP-18 Wave 2 · scope pane-link selectors + create requests to this workspace. */
  workspaceId: string;
  active: boolean;
  focusedPaneId: string | null;
  /**
   * MVP-14 Phase C · 当前 tab 临时最大化的 pane id · null 时正常 split layout。
   * 透传给 [`PaneTerminal`] 设 `data-is-maximized` · CSS 选中后撑满 host · §E.1。
   */
  maximizedPaneId?: string | null;
  onPaneClick: (paneId: string) => void;
  onSplitterDragEnd?: (parentPaneId: string, ratio: number) => void;
  onPaneError?: (paneId: string, message: string) => void;
  onPaneExit?: (paneId: string, exitCode: number | null) => void;
  onRegisterPaneApi?: (paneId: string, api: PaneTerminalApi) => void;
  onUnregisterPaneApi?: (paneId: string) => void;
  onPaneSplit?: (direction: SplitDir, paneId: string) => void;
  onPaneClose?: (paneId: string) => void;
  onPanePasteRequest?: (paneId: string, text: string) => void;
};

const findPane = (panes: PaneState[], paneId: string): PaneState | null =>
  panes.find((p) => p.paneId === paneId) ?? null;

/**
 * 找到 split 节点 first 子树最深处的 pane_id · 用作拖拽 splitter 时的 parent_pane_id key。
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
  const pane = createMemo(() => findPane(props.panes, paneId));
  // MVP-17 Phase C wiring · 当 pane 已 detach 到独立 WebviewWindow · 原位置渲染 placeholder
  // backend `pane_detach_state_changed` 事件驱动 `detachedPanes` signal · 跨 worktree 共享
  const detachedLabel = createMemo(() => detachedPanes().get(paneId) ?? null);

  return (
    <Show when={pane()} fallback={<div class="vs-pane-missing" />}>
      {(p) => (
        <Show
          when={!detachedLabel()}
          fallback={
            <DetachedPlaceholder
              paneId={paneId}
              windowLabel={detachedLabel() ?? ""}
            />
          }
        >
          <PaneTerminal
            paneId={paneId}
            shell={p().shell}
            cwd={p().cwd}
            workspaceId={props.workspaceId}
            siblingPanes={props.panes.filter((sp) => sp.paneId !== paneId)}
            active={props.active}
            focused={props.focusedPaneId === paneId}
            maximized={props.maximizedPaneId === paneId}
            onClick={props.onPaneClick}
            onExit={props.onPaneExit}
            onError={props.onPaneError}
            onRegisterApi={props.onRegisterPaneApi}
            onUnregisterApi={props.onUnregisterPaneApi}
            onSplit={props.onPaneSplit}
            onClose={props.onPaneClose}
            onPasteRequest={props.onPanePasteRequest}
          />
        </Show>
      )}
    </Show>
  );
};

const RenderSplit: Component<PaneSplitViewProps> = (props) => {
  if (props.layout.kind !== "split") return null;
  const split = props.layout;

  const direction = createMemo(() => split.direction);
  const ratio = createMemo(() => split.ratio);
  const styleMemo = createMemo(() => ({
    "--vs-pane-ratio": String(ratio()),
  }));
  const parentPaneId = firstPaneInSubtree(split.first);

  return (
    <div
      class={`vs-pane-split vs-pane-split-${direction()}`}
      style={styleMemo()}
    >
      <div class="vs-pane-split-first">
        <PaneSplitView
          layout={split.first}
          panes={props.panes}
          workspaceId={props.workspaceId}
          active={props.active}
          focusedPaneId={props.focusedPaneId}
          maximizedPaneId={props.maximizedPaneId}
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
        direction={direction()}
        ratio={ratio()}
        parentPaneId={parentPaneId}
        onDragEnd={props.onSplitterDragEnd}
      />
      <div class="vs-pane-split-second">
        <PaneSplitView
          layout={split.second}
          panes={props.panes}
          workspaceId={props.workspaceId}
          active={props.active}
          focusedPaneId={props.focusedPaneId}
          maximizedPaneId={props.maximizedPaneId}
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
