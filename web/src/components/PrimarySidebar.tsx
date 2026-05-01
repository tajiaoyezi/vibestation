import { type Component, Show, For } from "solid-js";
import type { WorkspaceMetadata } from "../App";

interface PrimarySidebarProps {
  workspaces: () => WorkspaceMetadata[];
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpen: (id: string) => void;
  onCreate: () => void;
  onDelete: (id: string) => void;
  loading: () => boolean;
  layout: () => { primaryOpen: boolean; primaryWidth: number };
  onResizeStart: (e: MouseEvent) => void;
  onResizeReset: () => void;
}

// /Users/leaf/Foo/Bar → ~/Foo/Bar · /home/leaf/Foo → ~/Foo · 其他保持原样
function prettyPath(p: string): string {
  return p.replace(/^(\/Users\/[^/]+|\/home\/[^/]+|\/root)(?=\/|$)/, "~");
}

export const PrimarySidebar: Component<PrimarySidebarProps> = (props) => {
  return (
    <div
      class={`vs-primary-sidebar${props.layout().primaryOpen ? "" : " vs-panel-hidden"}`}
      role="complementary"
      aria-label="Primary sidebar"
      style={{
        width: props.layout().primaryOpen
          ? `${props.layout().primaryWidth}px`
          : "0",
      }}
    >
      <div class="vs-panel-head">
        <span class="vs-panel-title">Workspaces</span>
        <div class="vs-panel-actions">
          <button
            type="button"
            class="vs-icon-btn"
            aria-label="Create workspace"
            onClick={props.onCreate}
            disabled={props.loading()}
          >
            +
          </button>
        </div>
      </div>
      <div class="vs-panel-body">
        <ul class="vs-ws-list" role="listbox" aria-label="Workspace list">
          <For each={props.workspaces()}>
            {(ws) => (
              <li
                role="option"
                aria-selected={
                  props.activeWorkspace()?.workspaceId === ws.workspaceId
                }
                classList={{
                  "vs-ws-item": true,
                  "vs-ws-item-active":
                    props.activeWorkspace()?.workspaceId === ws.workspaceId,
                }}
                onClick={() => props.onOpen(ws.workspaceId)}
              >
                <div class="vs-ws-row-main">
                  <span class="vs-ws-name">{ws.name}</span>
                  <Show when={ws.hasGit}>
                    <span class="vs-git-badge" aria-label="Git repository">
                      Git
                    </span>
                  </Show>
                  <button
                    type="button"
                    class="vs-ws-delete"
                    aria-label={`Delete ${ws.name}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onDelete(ws.workspaceId);
                    }}
                  >
                    ×
                  </button>
                </div>
                <span class="vs-ws-path" title={ws.path}>
                  {prettyPath(ws.path)}
                </span>
              </li>
            )}
          </For>
        </ul>
        <Show when={props.workspaces().length === 0}>
          <p class="vs-empty-hint">No workspaces yet.</p>
        </Show>
      </div>
      <div
        class="vs-resize-handle vs-resize-handle-e"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize primary sidebar"
        onMouseDown={props.onResizeStart}
        onDblClick={props.onResizeReset}
      />
    </div>
  );
};
