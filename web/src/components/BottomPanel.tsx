import { type Component, Show } from "solid-js";
import type { WorkspaceMetadata } from "../bindings";
import { t, normalizeLanguage } from "../i18n";
import { GitStatusPanel } from "../panels/GitStatus";
import { OutputPanel } from "../panels/Output/OutputPanel";
import { useSettings } from "../stores/settings";
import { useBottomPanelTabs } from "../stores/bottom-panel-tabs";
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
  const { activeTab, setActiveTab } = useBottomPanelTabs();

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
      <div
        class="vs-bp-head"
        role="tablist"
        aria-label={label("chrome.bottom.panel")}
      >
        <button
          type="button"
          class={`vs-bp-tab${activeTab() === "status" ? " vs-bp-tab-on" : ""}`}
          role="tab"
          aria-selected={activeTab() === "status"}
          onClick={() => setActiveTab("status")}
        >
          {label("chrome.bottom.gitStatus")}
        </button>
        <button
          type="button"
          class={`vs-bp-tab${activeTab() === "output" ? " vs-bp-tab-on" : ""}`}
          role="tab"
          aria-selected={activeTab() === "output"}
          onClick={() => setActiveTab("output")}
        >
          {label("chrome.bottom.output")}
        </button>
        <div class="vs-bp-right">
          <span class="vs-kbd-tip">{formatShortcut("⌘J", "Ctrl+J")}</span>
        </div>
      </div>
      <div class="vs-bp-body">
        <Show when={activeTab() === "status"}>
          <GitStatusPanel
            activeWorkspace={props.activeWorkspace}
            onOpenDiff={props.onOpenDiff}
          />
        </Show>
        <Show when={activeTab() === "output"}>
          <OutputPanel />
        </Show>
      </div>
    </div>
  );
};
