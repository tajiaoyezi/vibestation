import { createSignal, onMount, Show, For, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-dialog";
import { appDataDir } from "@tauri-apps/api/path";
import "./styles.css";

interface WorkspaceMetadata {
  workspace_id: string;
  name: string;
  path: string;
  has_git: boolean;
  repo_root: string | null;
  created_at: number;
  last_opened: number;
}

type IpcState =
  | { kind: "pending" }
  | { kind: "ok"; message: string }
  | { kind: "error"; message: string };

type View = { kind: "welcome" } | { kind: "workspace"; ws: WorkspaceMetadata };

export const App: Component = () => {
  const [version, setVersion] = createSignal<string>("…");
  const [ipc, setIpc] = createSignal<IpcState>({ kind: "pending" });
  const [workspaces, setWorkspaces] = createSignal<WorkspaceMetadata[]>([]);
  const [currentView, setCurrentView] = createSignal<View>({
    kind: "welcome",
  });
  const [dbReady, setDbReady] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      setVersion(await getVersion());
    } catch {
      setVersion("unknown");
    }

    try {
      const msg = await invoke<string>("greet");
      setIpc({ kind: "ok", message: msg });
    } catch (err) {
      setIpc({
        kind: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }

    try {
      const dir = await appDataDir();
      await invoke("workspace_init", { dbDir: dir });
      setDbReady(true);
      await refreshWorkspaces();
    } catch (err) {
      setIpc({
        kind: "error",
        message: `db init: ${err instanceof Error ? err.message : String(err)}`,
      });
    }
  });

  async function refreshWorkspaces() {
    try {
      const list = await invoke<WorkspaceMetadata[]>("workspace_list");
      setWorkspaces(list);
      if (list.length > 0 && currentView().kind === "welcome") {
        setCurrentView({ kind: "workspace", ws: list[0] });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  const handleCreateWorkspace = async () => {
    setError(null);
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected) return;

      const dirPath = typeof selected === "string" ? selected : selected;

      const exists = await invoke<boolean>("workspace_exists", {
        path: dirPath,
      });
      if (exists) {
        setError("该目录已有 workspace，请直接打开它");
        await refreshWorkspaces();
        return;
      }

      setLoading(true);
      const ws = await invoke<WorkspaceMetadata>("workspace_create", {
        path: dirPath,
        name: null,
      });
      setWorkspaces((prev) => [ws, ...prev]);
      setCurrentView({ kind: "workspace", ws });
      setLoading(false);
    } catch (err) {
      setLoading(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleOpenWorkspace = async (id: string) => {
    try {
      const ws = await invoke<WorkspaceMetadata>("workspace_open", { id });
      setWorkspaces((prev) =>
        prev.map((w) => (w.workspace_id === id ? ws : w)),
      );
      setCurrentView({ kind: "workspace", ws });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDeleteWorkspace = async (id: string) => {
    try {
      await invoke("workspace_delete", { id });
      setWorkspaces((prev) => prev.filter((w) => w.workspace_id !== id));
      setDeleteConfirm(null);
      const view = currentView();
      if (view.kind === "workspace" && view.ws.workspace_id === id) {
        const remaining = workspaces().filter((w) => w.workspace_id !== id);
        setCurrentView(
          remaining.length > 0
            ? { kind: "workspace", ws: remaining[0] }
            : { kind: "welcome" },
        );
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const activeWorkspace = (): WorkspaceMetadata | null => {
    const v = currentView();
    return v.kind === "workspace" ? v.ws : null;
  };

  return (
    <main class="vs-root">
      <div class="vs-sidebar">
        <div class="vs-sidebar-header">
          <VibestationMarkSmall />
          <span class="vs-sidebar-title">Workspaces</span>
          <button
            type="button"
            class="vs-btn-icon"
            aria-label="Create workspace"
            onClick={handleCreateWorkspace}
            disabled={loading()}
          >
            +
          </button>
        </div>
        <ul class="vs-ws-list" role="listbox" aria-label="Workspace list">
          <For each={workspaces()}>
            {(ws) => (
              <li
                role="option"
                classList={{
                  "vs-ws-item": true,
                  "vs-ws-item-active":
                    activeWorkspace()?.workspace_id === ws.workspace_id,
                }}
                onClick={() => handleOpenWorkspace(ws.workspace_id)}
              >
                <span class="vs-ws-name">{ws.name}</span>
                <Show when={ws.has_git}>
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
                    setDeleteConfirm(ws.workspace_id);
                  }}
                >
                  ×
                </button>
              </li>
            )}
          </For>
        </ul>
        <Show when={workspaces().length === 0 && dbReady()}>
          <p class="vs-empty-hint">No workspaces yet.</p>
        </Show>
      </div>

      <section class="vs-main">
        <Show
          when={activeWorkspace() !== null}
          fallback={
            <div class="vs-welcome">
              <VibestationMark />
              <div class="vs-title-block">
                <h1 id="vs-welcome-title" class="vs-title">
                  Vibestation
                </h1>
                <p class="vs-tagline">多 Tab 终端 + JetBrains 级 Git 工作台</p>
                <span class="vs-version" aria-label={`Version ${version()}`}>
                  <span class="vs-version-dot" aria-hidden="true" />v{version()}{" "}
                  · alpha
                </span>
              </div>
              <button
                type="button"
                class="vs-cta"
                aria-label="Create first workspace"
                onClick={handleCreateWorkspace}
                disabled={loading()}
              >
                Create first workspace
              </button>
            </div>
          }
        >
          <div class="vs-workspace-view">
            <h2 class="vs-ws-heading">{activeWorkspace()?.name}</h2>
            <p class="vs-ws-path" title={activeWorkspace()?.path ?? ""}>
              {activeWorkspace()?.path}
            </p>
            <Show when={activeWorkspace()?.has_git}>
              <p class="vs-ws-git-info">
                <span class="vs-git-badge">Git</span>
                <span class="vs-ws-repo-root">
                  {activeWorkspace()?.repo_root}
                </span>
              </p>
            </Show>
            <p class="vs-ws-placeholder">
              Tool Windows + Tab 管理由 MVP-03/04 接管
            </p>
          </div>
        </Show>
      </section>

      <Show when={error()}>
        <div class="vs-error-bar" role="alert">
          {error()}
          <button
            type="button"
            class="vs-error-dismiss"
            onClick={() => setError(null)}
            aria-label="Dismiss error"
          >
            ×
          </button>
        </div>
      </Show>

      <Show when={deleteConfirm() !== null}>
        <div class="vs-modal-overlay" role="dialog" aria-modal="true">
          <div class="vs-modal">
            <h3>Delete workspace?</h3>
            <p>文件不会删，仅从 Vibestation 移除。</p>
            <div class="vs-modal-actions">
              <button
                type="button"
                class="vs-btn-danger"
                onClick={() => handleDeleteWorkspace(deleteConfirm()!)}
              >
                Delete
              </button>
              <button
                type="button"
                class="vs-btn-secondary"
                onClick={() => setDeleteConfirm(null)}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      </Show>

      <footer class="vs-diag" aria-label="Runtime diagnostics">
        <IpcIndicator state={ipc()} />
      </footer>
    </main>
  );
};

const VibestationMark: Component = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 64 64"
    width="64"
    height="64"
    role="img"
    aria-label="Vibestation mark"
  >
    <defs>
      <linearGradient id="m-grad" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="oklch(0.72 0.16 240)" />
        <stop offset="100%" stop-color="oklch(0.58 0.2 260)" />
      </linearGradient>
    </defs>
    <rect x="4" y="4" width="56" height="56" rx="14" fill="url(#m-grad)" />
    <g
      fill="none"
      stroke="white"
      stroke-width="3.2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M20 24 L28 32 L20 40" />
      <line x1="32" y1="42" x2="46" y2="42" />
    </g>
    <circle cx="46" cy="22" r="2.5" fill="white" opacity="0.9" />
  </svg>
);

const VibestationMarkSmall: Component = () => (
  <svg
    xmlns="http://www.w3.org/2002/svg"
    viewBox="0 0 64 64"
    width="20"
    height="20"
    role="img"
    aria-label="Vibestation mark"
  >
    <rect x="4" y="4" width="56" height="56" rx="14" fill="url(#m-grad)" />
    <g
      fill="none"
      stroke="white"
      stroke-width="3.2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M20 24 L28 32 L20 40" />
      <line x1="32" y1="42" x2="46" y2="42" />
    </g>
  </svg>
);

interface IpcIndicatorProps {
  state: IpcState;
}

const IpcIndicator: Component<IpcIndicatorProps> = (props) => {
  const label = () => {
    switch (props.state.kind) {
      case "pending":
        return "ipc: connecting…";
      case "ok":
        return `ipc: ${props.state.message}`;
      case "error":
        return `ipc error: ${props.state.message}`;
    }
  };

  const className = () =>
    props.state.kind === "error" ? "vs-diag-error" : "vs-diag-ok";

  return (
    <span class={className()} data-testid="ipc-indicator">
      {label()}
    </span>
  );
};
