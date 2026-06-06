import { type Component } from "solid-js";
import { GitLogPanel } from "../panels/GitLog";
import type { WorkspaceMetadata } from "../bindings";
import { t, normalizeLanguage } from "../i18n";
import { useSettings } from "../stores/settings";
import type { DiffTarget } from "./MainContent";

interface SecondarySidebarProps {
  layout: () => { secondaryOpen: boolean; secondaryWidth: number };
  onResizeStart: (e: MouseEvent) => void;
  onResizeReset: () => void;
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpenDiff: (target: DiffTarget) => void;
  onOpenGitStatus: () => void;
}

export const SecondarySidebar: Component<SecondarySidebarProps> = (props) => {
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  return (
    <div
      class={`vs-secondary-sidebar${props.layout().secondaryOpen ? "" : " vs-panel-hidden"}`}
      role="complementary"
      aria-label={label("chrome.sidebars.secondary")}
      style={{
        width: props.layout().secondaryOpen
          ? `${props.layout().secondaryWidth}px`
          : "0",
      }}
    >
      <div class="vs-panel-body">
        <GitLogPanel
          activeWorkspace={props.activeWorkspace}
          onOpenDiff={props.onOpenDiff}
          onOpenGitStatus={props.onOpenGitStatus}
        />
      </div>
      <div
        class="vs-resize-handle vs-resize-handle-w"
        role="separator"
        aria-orientation="vertical"
        aria-label={label("chrome.sidebars.resizeSecondary")}
        onMouseDown={props.onResizeStart}
        onDblClick={props.onResizeReset}
      />
    </div>
  );
};
