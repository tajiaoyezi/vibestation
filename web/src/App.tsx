import {
  createSignal,
  createEffect,
  onMount,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import "./styles.css";

import { LayoutProvider, useLayout } from "./stores/layout-context";
import { ThemeProvider } from "./stores/theme";
import { useSettings } from "./stores/settings";
import { PrimarySidebar } from "./components/PrimarySidebar";
import { SecondarySidebar } from "./components/SecondarySidebar";
import { BottomPanel } from "./components/BottomPanel";
import { ActivityStrip } from "./components/ActivityStrip";
import { MainContent } from "./components/MainContent";
import type { DiffTarget } from "./components/MainContent";
import { ThemeSwitch } from "./components/ThemeSwitch";
import { SettingsPanel } from "./panels/Settings";
import { TelemetryOptInModal } from "./dialogs/TelemetryOptIn/TelemetryOptInModal";

// IPC contract types · 由 `crates/app/build.rs` 从 Rust `#[derive(TS)]` 自动生成。
// 禁止手写对偶 interface（SPIKE-08 §A rollout · 防 H2 类 drift）。
import type { WorkspaceMetadata } from "./bindings";
export type { WorkspaceMetadata };

type IpcState =
  | { kind: "pending" }
  | { kind: "ok"; message: string }
  | { kind: "error"; message: string };

type View = { kind: "welcome" } | { kind: "workspace"; ws: WorkspaceMetadata };

const IpcIndicator: Component<{ state: IpcState }> = (props) => {
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
  return <span class={className()}>{label()}</span>;
};

const LayoutShell: Component<{
  workspaces: () => WorkspaceMetadata[];
  currentView: () => View;
  activeDiff: () => DiffTarget | null;
  ipc: () => IpcState;
  version: () => string;
  dbReady: () => boolean;
  loading: () => boolean;
  deleteConfirm: () => string | null;
  error: () => string | null;
  onOpen: (id: string) => void;
  onCreate: () => void;
  onDeleteConfirm: (id: string) => void;
  onDeleteExecute: () => void;
  onDeleteCancel: () => void;
  onDismissError: () => void;
  onOpenDiff: (target: DiffTarget) => void;
  onCloseDiff: () => void;
  onCloseWorkspaceView: (workspaceId: string) => void;
}> = (props) => {
  const { layout, dispatch, loadForWorkspace } = useLayout();
  const [settingsVisible, setSettingsVisible] = createSignal(false);

  const activeWorkspace = (): WorkspaceMetadata | null => {
    const v = props.currentView();
    return v.kind === "workspace" ? v.ws : null;
  };

  const handleTitleBarMouseDown = (event: MouseEvent) => {
    if (event.button !== 0 || event.detail !== 1) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    void getCurrentWindow().startDragging();
  };

  createEffect(() => {
    const ws = activeWorkspace();
    if (ws) {
      loadForWorkspace(ws.workspaceId);
    }
  });

  const handlePrimaryResizeStart = (e: MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = layout().primaryWidth;
    const onMove = (ev: MouseEvent) => {
      const delta = ev.clientX - startX;
      dispatch({ kind: "resize-primary", width: startWidth + delta });
    };
    const onUp = () => {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  };

  const handleSecondaryResizeStart = (e: MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = layout().secondaryWidth;
    const onMove = (ev: MouseEvent) => {
      const delta = startX - ev.clientX;
      dispatch({ kind: "resize-secondary", width: startWidth + delta });
    };
    const onUp = () => {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  };

  const handleBottomResizeStart = (e: MouseEvent) => {
    e.preventDefault();
    const startY = e.clientY;
    const startHeight = layout().bottomHeight;
    const onMove = (ev: MouseEvent) => {
      const delta = startY - ev.clientY;
      dispatch({ kind: "resize-bottom", height: startHeight + delta });
    };
    const onUp = () => {
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", onUp);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    document.body.style.cursor = "row-resize";
    document.body.style.userSelect = "none";
    document.addEventListener("mousemove", onMove);
    document.addEventListener("mouseup", onUp);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.defaultPrevented) return;
    if (e.target instanceof Element && e.target.closest(".vs-terminal-shell")) {
      return;
    }

    const mod = e.metaKey || e.ctrlKey;
    if (!mod) return;
    switch (e.key) {
      case "1":
        e.preventDefault();
        dispatch({ kind: "toggle-primary" });
        break;
      case "2":
        e.preventDefault();
        dispatch({ kind: "toggle-secondary" });
        break;
      case "j":
      case "J":
        e.preventDefault();
        dispatch({ kind: "toggle-bottom" });
        break;
      // ⌘, 由 Menu Accelerator 处理（menu:action "preferences"）· 删除 keydown 重复触发（round 2 fix INFO-2）
    }
  };

  let unlistenMenu: UnlistenFn | undefined;

  onMount(async () => {
    document.addEventListener("keydown", handleKeyDown);
    unlistenMenu = await listen<{ action: string }>("menu:action", (event) => {
      switch (event.payload.action) {
        case "preferences":
          setSettingsVisible((v) => !v);
          break;
      }
    });
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyDown);
    unlistenMenu?.();
  });

  return (
    <div class="vs-shell">
      <div
        class="title-bar-drag"
        data-tauri-drag-region
        aria-hidden="true"
        onMouseDown={handleTitleBarMouseDown}
      />

      <div class="vs-main-grid">
        <PrimarySidebar
          workspaces={props.workspaces}
          activeWorkspace={activeWorkspace}
          onOpen={props.onOpen}
          onCreate={props.onCreate}
          onDelete={props.onDeleteConfirm}
          loading={props.loading}
          layout={() => ({
            primaryOpen: layout().primaryOpen,
            primaryWidth: layout().primaryWidth,
          })}
          onResizeStart={handlePrimaryResizeStart}
          onResizeReset={() => dispatch({ kind: "reset-primary" })}
        />

        <MainContent
          activeWorkspace={activeWorkspace}
          activeDiff={props.activeDiff}
          onCloseDiff={props.onCloseDiff}
          onCloseWorkspaceView={props.onCloseWorkspaceView}
          workspaces={props.workspaces}
        />

        <SecondarySidebar
          layout={() => ({
            secondaryOpen: layout().secondaryOpen,
            secondaryWidth: layout().secondaryWidth,
          })}
          onResizeStart={handleSecondaryResizeStart}
          onResizeReset={() => dispatch({ kind: "reset-secondary" })}
          activeWorkspace={activeWorkspace}
          onOpenDiff={props.onOpenDiff}
        />

        <ActivityStrip />
      </div>

      <BottomPanel
        layout={() => ({
          bottomOpen: layout().bottomOpen,
          bottomHeight: layout().bottomHeight,
        })}
        onResizeStart={handleBottomResizeStart}
        onResizeReset={() => dispatch({ kind: "reset-bottom" })}
        activeWorkspace={activeWorkspace}
        onOpenDiff={props.onOpenDiff}
      />

      <footer class="vs-status-bar" aria-label="Status bar">
        <div class="vs-status-group">
          <IpcIndicator state={props.ipc()} />
        </div>
        <div class="vs-status-group">
          <ThemeSwitch />
          <span class="vs-status-item">
            <span class="vs-status-val">v{props.version()} · alpha</span>
          </span>
        </div>
      </footer>

      <Show when={props.error()}>
        <div class="vs-error-bar" role="alert">
          {props.error()}
          <button
            type="button"
            class="vs-error-dismiss"
            onClick={props.onDismissError}
            aria-label="Dismiss error"
          >
            ×
          </button>
        </div>
      </Show>

      <Show when={props.deleteConfirm() !== null}>
        <div class="vs-modal-overlay" role="dialog" aria-modal="true">
          <div class="vs-modal">
            <h3>Delete workspace?</h3>
            <p>文件不会删，仅从 Vibestation 移除。</p>
            <div class="vs-modal-actions">
              <button
                type="button"
                class="vs-btn-danger"
                onClick={props.onDeleteExecute}
              >
                Delete
              </button>
              <button
                type="button"
                class="vs-btn-secondary"
                onClick={props.onDeleteCancel}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      </Show>

      <SettingsPanel
        visible={settingsVisible()}
        onClose={() => setSettingsVisible(false)}
      />
    </div>
  );
};

const App: Component = () => {
  const [version, setVersion] = createSignal<string>("…");
  const [ipc, setIpc] = createSignal<IpcState>({ kind: "pending" });
  const [workspaces, setWorkspaces] = createSignal<WorkspaceMetadata[]>([]);
  const [currentView, setCurrentView] = createSignal<View>({ kind: "welcome" });
  const [activeDiff, setActiveDiff] = createSignal<DiffTarget | null>(null);
  const [dbReady, setDbReady] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [deleteConfirm, setDeleteConfirm] = createSignal<string | null>(null);

  // MVP-10 §B.1 · Telemetry opt-in 首次启动检查
  // settings.telemetryOptIn === null → 首次启动 · 阻塞 WelcomePage 渲染（modal 全屏覆盖）
  const { settings } = useSettings();
  const telemetryDecided = (): boolean => settings.telemetryOptIn !== null;

  const activeWorkspaceId = (): string | null => {
    const v = currentView();
    return v.kind === "workspace" ? v.ws.workspaceId : null;
  };

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
      await invoke("workspace_init");
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
      setActiveDiff(null);
      setLoading(false);
    } catch (err) {
      setLoading(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleOpenWorkspace = async (id: string) => {
    try {
      const ws = await invoke<WorkspaceMetadata>("workspace_open", { id });
      setWorkspaces((prev) => prev.map((w) => (w.workspaceId === id ? ws : w)));
      setCurrentView({ kind: "workspace", ws });
      setActiveDiff(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleDeleteWorkspace = async () => {
    const id = deleteConfirm();
    if (!id) return;
    try {
      await invoke("workspace_delete", { id });
      setWorkspaces((prev) => prev.filter((w) => w.workspaceId !== id));
      setDeleteConfirm(null);
      const view = currentView();
      if (view.kind === "workspace" && view.ws.workspaceId === id) {
        const remaining = workspaces().filter((w) => w.workspaceId !== id);
        setCurrentView(
          remaining.length > 0
            ? { kind: "workspace", ws: remaining[0] }
            : { kind: "welcome" },
        );
        setActiveDiff(null);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const handleCloseWorkspaceView = (workspaceId: string) => {
    const view = currentView();
    if (view.kind === "workspace" && view.ws.workspaceId === workspaceId) {
      setCurrentView({ kind: "welcome" });
      setActiveDiff(null);
    }
  };

  const handleOpenDiff = (target: DiffTarget) => {
    const workspace = workspaces().find(
      (item) => item.workspaceId === target.workspaceId,
    );
    if (workspace) {
      setCurrentView({ kind: "workspace", ws: workspace });
    }
    setActiveDiff(target);
  };

  return (
    <ThemeProvider>
      <LayoutProvider activeWorkspaceId={activeWorkspaceId}>
        <LayoutShell
          workspaces={workspaces}
          currentView={currentView}
          activeDiff={activeDiff}
          ipc={ipc}
          version={version}
          dbReady={dbReady}
          loading={loading}
          deleteConfirm={deleteConfirm}
          error={error}
          onOpen={handleOpenWorkspace}
          onCreate={handleCreateWorkspace}
          onDeleteConfirm={(id) => setDeleteConfirm(id)}
          onDeleteExecute={handleDeleteWorkspace}
          onDeleteCancel={() => setDeleteConfirm(null)}
          onDismissError={() => setError(null)}
          onOpenDiff={handleOpenDiff}
          onCloseDiff={() => setActiveDiff(null)}
          onCloseWorkspaceView={handleCloseWorkspaceView}
        />
        <Show when={!telemetryDecided()}>
          <TelemetryOptInModal />
        </Show>
      </LayoutProvider>
    </ThemeProvider>
  );
};

export { App };
