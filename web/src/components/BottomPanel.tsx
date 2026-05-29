import { type Component } from "solid-js";
import type { WorkspaceMetadata } from "../bindings";
import { GitStatusPanel } from "../panels/GitStatus";
import type { DiffTarget } from "./MainContent";
import { formatShortcut } from "../lib/format-shortcut";

interface BottomPanelProps {
  layout: () => { bottomOpen: boolean; bottomHeight: number };
  onResizeStart: (e: MouseEvent) => void;
  onResizeReset: () => void;
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpenDiff: (target: DiffTarget) => void;
}

export const BottomPanel: Component<BottomPanelProps> = (props) => {
  return (
    <div
      class={`vs-bottom-panel${props.layout().bottomOpen ? "" : " vs-panel-hidden"}`}
      role="region"
      aria-label="Bottom panel"
      style={{
        height: props.layout().bottomOpen
          ? `${props.layout().bottomHeight}px`
          : "0",
      }}
    >
      <div
        class="vs-resize-handle vs-resize-handle-n"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Resize bottom panel"
        onMouseDown={props.onResizeStart}
        onDblClick={props.onResizeReset}
      />
      <div class="vs-bp-head">
        <span class="vs-bp-tab vs-bp-tab-on">Git Status</span>
        <span class="vs-bp-tab">Output</span>
        <span class="vs-bp-tab">Diff</span>
        <div class="vs-bp-right">
          <span class="vs-kbd-tip">{formatShortcut("⌘J", "Ctrl+J")}</span>
        </div>
      </div>
      <div class="vs-bp-body">
        <GitStatusPanel
          activeWorkspace={props.activeWorkspace}
          onOpenDiff={props.onOpenDiff}
        />
      </div>
    </div>
  );
};
