/**
 * PaneSplitView · MVP-14 Phase B
 *
 * 递归渲染 [`LayoutNode`] 树：
 * - `Single` → 渲染 [`PaneTerminal`]
 * - `Split` → 渲染两个子 [`PaneSplitView`] · 中间夹一个 [`PaneSplitter`] 分隔条
 *
 * §H 布局模型：MVP-14 支持最多 5 层嵌套（backend `MAX_LAYOUT_SPLIT_DEPTH = 5`）。
 *
 * BUG-001 fix（2026-05-23 session 34 · 第 6 路修复方案 · plain getter pattern）：
 * - 不用 `const split = props.layout` capture（非 reactive · 内层递归收 stale）
 * - 不用 `createMemo` 派生字段（嵌套 createMemo 触发 SolidJS reactive owner cleanup race
 *   `null is not an object 'node.owned[i]'`）
 * - 不用嵌套 `<Show>` render prop（同样触发 owner cleanup race）
 * - 改用 plain getter function `() => props.layout?.field`· 配合 optional chaining +
 *   nullable fallback · 配合 JSX ternary `{cond && <X />}` 替代嵌套 Show
 * - JSX 内 prop binding 自动 reactive · plain getter 不创建额外 reactive scope ·
 *   避免 owner.owned 数组复杂度增长
 */
import { Show, type Component, type JSX } from "solid-js";
import type { LayoutNode, PaneState, SplitDir } from "../../bindings";
import { PaneSplitter } from "./PaneSplitter";
import { PaneTerminal, type PaneTerminalApi } from "./PaneTerminal";
import { DetachedPlaceholder } from "./DetachedPlaceholder";
import { detachedPanes } from "../../lib/pane-detach";

type SplitLayout = Extract<LayoutNode, { kind: "split" }>;

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
  // BUG-001 fix · plain getter（不用 createMemo · 不增加 owner.owned 复杂度）+
  // optional chaining guard 防 SolidJS unmount race 中 props.layout 短暂 undefined。
  const paneId = (): string | undefined =>
    props.layout?.kind === "single" ? props.layout.paneId : undefined;
  const pane = (): PaneState | null => {
    const id = paneId();
    return id ? findPane(props.panes, id) : null;
  };
  // MVP-17 Phase C wiring · 当 pane 已 detach 到独立 WebviewWindow · 原位置渲染 placeholder
  // backend `pane_detach_state_changed` 事件驱动 `detachedPanes` signal · 跨 worktree 共享
  const detachedLabel = (): string | null => {
    const id = paneId();
    return id ? (detachedPanes().get(id) ?? null) : null;
  };

  return (
    <Show when={pane()} fallback={<div class="vs-pane-missing" />}>
      {(p) => (
        <Show
          when={!detachedLabel()}
          fallback={
            <DetachedPlaceholder
              paneId={paneId() ?? ""}
              windowLabel={detachedLabel() ?? ""}
            />
          }
        >
          <PaneTerminal
            paneId={paneId() ?? ""}
            shell={p().shell}
            cwd={p().cwd}
            workspaceId={props.workspaceId}
            siblingPanes={props.panes.filter((sp) => sp.paneId !== paneId())}
            active={props.active}
            focused={props.focusedPaneId === paneId()}
            maximized={props.maximizedPaneId === paneId()}
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
  // BUG-001 fix · plain getter（不用 createMemo · 不增加 owner.owned 复杂度）+
  // optional chaining guard 防 SolidJS unmount race。每次 JSX 访问 evaluate · reactive。
  const splitNode = (): SplitLayout | undefined =>
    props.layout?.kind === "split" ? props.layout : undefined;
  const direction = (): SplitDir => splitNode()?.direction ?? "horizontal";
  const ratio = (): number => splitNode()?.ratio ?? 0.5;
  const first = (): LayoutNode | undefined => splitNode()?.first;
  const second = (): LayoutNode | undefined => splitNode()?.second;
  const parentPaneId = (): string => {
    const node = splitNode();
    return node ? firstPaneInSubtree(node.first) : "";
  };
  const styleProp = (): JSX.CSSProperties => ({
    "--vs-pane-ratio": String(ratio()),
  });

  return (
    <div
      class={`vs-pane-split vs-pane-split-${direction()}`}
      style={styleProp()}
    >
      <div class="vs-pane-split-first">
        {first() && (
          <PaneSplitView
            layout={first()!}
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
        )}
      </div>
      <PaneSplitter
        direction={direction()}
        ratio={ratio()}
        parentPaneId={parentPaneId()}
        onDragEnd={props.onSplitterDragEnd}
      />
      <div class="vs-pane-split-second">
        {second() && (
          <PaneSplitView
            layout={second()!}
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
        )}
      </div>
    </div>
  );
};
