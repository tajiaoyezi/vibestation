import { type Component, Show } from "solid-js";
import type { WorkspaceMetadata } from "../App";
import { SidebarLeftIcon } from "./Icons";

interface TopBarProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  primaryOpen: () => boolean;
  onTogglePrimary: () => void;
  onMouseDown?: (e: MouseEvent) => void;
}

export const TopBar: Component<TopBarProps> = (props) => {
  return (
    <header
      class="vs-top-bar"
      data-tauri-drag-region
      onMouseDown={props.onMouseDown}
    >
      <button
        type="button"
        class={`vs-top-bar-toggle${props.primaryOpen() ? "" : " vs-top-bar-toggle-off"}`}
        aria-label="Toggle primary sidebar"
        aria-pressed={props.primaryOpen()}
        title="Toggle Primary Sidebar (⌘B)"
        onClick={props.onTogglePrimary}
      >
        <SidebarLeftIcon />
      </button>
      <Show when={props.activeWorkspace()}>
        {(ws) => (
          <div class="vs-top-bar-meta">
            <span class="vs-top-bar-name">{ws().name}</span>
            <span class="vs-top-bar-path" title={ws().path}>
              {ws().path}
            </span>
          </div>
        )}
      </Show>
    </header>
  );
};
