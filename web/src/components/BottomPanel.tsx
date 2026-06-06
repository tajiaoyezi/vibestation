import { type Component } from "solid-js";
import type { WorkspaceMetadata } from "../bindings";
import { t, normalizeLanguage } from "../i18n";
import { GitStatusPanel } from "../panels/GitStatus";
import { useSettings } from "../stores/settings";
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
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  return (
    <div
      class={`vs-bottom-panel${props.layout().bottomOpen ? "" : " vs-panel-hidden"}`}
      role="region"
      aria-label={label("chrome.bottom.panel")}
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
        aria-label={label("chrome.bottom.resizePanel")}
        onMouseDown={props.onResizeStart}
        onDblClick={props.onResizeReset}
      />
      <div class="vs-bp-head">
        <span class="vs-bp-tab vs-bp-tab-on">
          {label("chrome.bottom.gitStatus")}
        </span>
        <span class="vs-bp-tab">{label("chrome.bottom.output")}</span>
        <span class="vs-bp-tab">{label("chrome.bottom.diff")}</span>
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
