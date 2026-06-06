import { type Component, Show, For } from "solid-js";
import type { WorkspaceMetadata } from "../App";
import { t, normalizeLanguage } from "../i18n";
import { BranchTree } from "../panels/BranchTree/BranchTree";
import { useSettings } from "../stores/settings";

interface PrimarySidebarProps {
  workspaces: () => WorkspaceMetadata[];
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpen: (id: string) => void;
  onCreate: () => void;
  onDelete: (id: string) => void;
  /** MVP-06 · 触发配置导入对话框（欢迎页 / 空状态显示） */
  onOpenImport?: () => void;
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
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  return (
    <div
      class={`vs-primary-sidebar${props.layout().primaryOpen ? "" : " vs-panel-hidden"}`}
      role="complementary"
      aria-label={label("chrome.sidebars.primary")}
      style={{
        width: props.layout().primaryOpen
          ? `${props.layout().primaryWidth}px`
          : "0",
      }}
    >
      <div class="vs-panel-head">
        <span class="vs-panel-title">
          {label("chrome.sidebars.workspaces")}
        </span>
        <div class="vs-panel-actions">
          <button
            type="button"
            class="vs-icon-btn"
            aria-label={label("chrome.sidebars.createWorkspace")}
            onClick={props.onCreate}
            disabled={props.loading()}
          >
            +
          </button>
        </div>
      </div>
      <div class="vs-panel-body vs-primary-panel-body">
        <div class="vs-ws-section">
          <ul
            class="vs-ws-list"
            role="listbox"
            aria-label={label("chrome.sidebars.workspaceList")}
          >
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
                      <span
                        class="vs-git-badge"
                        aria-label={label("chrome.sidebars.gitRepository")}
                      >
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
            <p class="vs-empty-hint">
              {label("chrome.sidebars.noWorkspacesYet")}
            </p>
            <Show when={props.onOpenImport}>
              <button
                type="button"
                class="vs-import-link"
                onClick={() => props.onOpenImport?.()}
              >
                {label("chrome.sidebars.importSettings")}
              </button>
            </Show>
          </Show>
        </div>
        <BranchTree activeWorkspace={props.activeWorkspace} />
      </div>
      <div
        class="vs-resize-handle vs-resize-handle-e"
        role="separator"
        aria-orientation="vertical"
        aria-label={label("chrome.sidebars.resizePrimary")}
        onMouseDown={props.onResizeStart}
        onDblClick={props.onResizeReset}
      />
    </div>
  );
};
