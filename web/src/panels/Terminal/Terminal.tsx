import { ask } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import type {
  PtySpawnRequest,
  TabCloseRequest,
  TabCreateRequest,
  TabListResponse,
  TabRenameRequest,
  TabState,
  WorkspaceMetadata,
} from "../../bindings";
import { PasteConfirmDialog } from "./PasteConfirmDialog";
import { TabBar } from "./TabBar";
import { TerminalPane } from "./TerminalPane";
import {
  DEFAULT_PTY_COLS,
  DEFAULT_PTY_ROWS,
  pickAdjacentTabId,
  pickSiblingTabId,
  useKeybindings,
  type RendererKind,
  type ShortcutAction,
  type TabRuntimeState,
} from "./hooks";

type TerminalToast = {
  kind: "error" | "info";
  message: string;
};

type PendingPaste = {
  tabId: string;
  text: string;
  workspaceId: string;
};

type PaneApi = {
  focus: () => void;
  paste: (text: string) => void;
  clear: () => void;
  copy: () => void;
  selectAll: () => void;
};

type TerminalProps = {
  activeWorkspace: () => WorkspaceMetadata | null;
  onCloseWorkspaceView: (workspaceId: string) => void;
  workspaces: () => WorkspaceMetadata[];
};

const DEFAULT_RUNTIME_STATE: TabRuntimeState = {
  phase: "idle",
  exitCode: null,
  spawnError: null,
  renderer: null,
  cols: DEFAULT_PTY_COLS,
  rows: DEFAULT_PTY_ROWS,
};

const errorMessage = (error: unknown): string =>
  error instanceof Error ? error.message : String(error);

export const Terminal: Component<TerminalProps> = (props) => {
  const [tabsByWorkspace, setTabsByWorkspace] = createSignal<
    Record<string, TabState[]>
  >({});
  const [activeTabByWorkspace, setActiveTabByWorkspace] = createSignal<
    Record<string, string | null>
  >({});
  const [runtimeByTabId, setRuntimeByTabId] = createSignal<
    Record<string, TabRuntimeState>
  >({});
  const [skipPasteConfirmByWorkspace, setSkipPasteConfirmByWorkspace] =
    createSignal<Record<string, boolean>>({});
  const [pendingPaste, setPendingPaste] = createSignal<PendingPaste | null>(
    null,
  );
  const [toast, setToast] = createSignal<TerminalToast | null>(null);
  const [pendingRenameTabId, setPendingRenameTabId] = createSignal<
    string | null
  >(null);

  const paneApis = new Map<string, PaneApi>();
  const loadingWorkspaces = new Set<string>();
  const newlyCreatedTabIds = new Set<string>();
  const removingWorkspaces = new Set<string>();
  let toastTimer: ReturnType<typeof setTimeout> | undefined;

  const activeWorkspaceId = () => props.activeWorkspace()?.workspaceId ?? null;

  const currentTabs = createMemo(() => {
    const workspaceId = activeWorkspaceId();
    return workspaceId ? (tabsByWorkspace()[workspaceId] ?? []) : [];
  });

  const currentActiveTabId = createMemo(() => {
    const workspaceId = activeWorkspaceId();
    if (!workspaceId) {
      return null;
    }

    const explicit = activeTabByWorkspace()[workspaceId] ?? null;
    if (explicit) {
      return explicit;
    }

    return currentTabs()[0]?.tabId ?? null;
  });

  const allTabs = createMemo(() => Object.values(tabsByWorkspace()).flat());

  const activeRenderer = createMemo(() => {
    const tabId = currentActiveTabId();
    if (!tabId) {
      return null;
    }

    return runtimeByTabId()[tabId]?.renderer ?? null;
  });

  const showToast = (
    message: string,
    kind: TerminalToast["kind"] = "error",
  ) => {
    setToast({ kind, message });
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    toastTimer = setTimeout(() => setToast(null), 4200);
  };

  const updateWorkspaceTabs = (
    workspaceId: string,
    updater: (tabs: TabState[]) => TabState[],
  ) => {
    setTabsByWorkspace((prev) => ({
      ...prev,
      [workspaceId]: updater(prev[workspaceId] ?? []),
    }));
  };

  const removeWorkspaceTabs = (workspaceId: string) => {
    setTabsByWorkspace((prev) => {
      const next = { ...prev };
      delete next[workspaceId];
      return next;
    });
  };

  const setWorkspaceActiveTab = (workspaceId: string, tabId: string | null) => {
    setActiveTabByWorkspace((prev) => ({
      ...prev,
      [workspaceId]: tabId,
    }));
  };

  const removeWorkspaceActiveTab = (workspaceId: string) => {
    setActiveTabByWorkspace((prev) => {
      const next = { ...prev };
      delete next[workspaceId];
      return next;
    });
  };

  const upsertRuntime = (
    tabId: string,
    updater: (runtime: TabRuntimeState) => TabRuntimeState,
  ) => {
    setRuntimeByTabId((prev) => ({
      ...prev,
      [tabId]: updater(prev[tabId] ?? DEFAULT_RUNTIME_STATE),
    }));
  };

  const removeRuntime = (tabId: string) => {
    setRuntimeByTabId((prev) => {
      const next = { ...prev };
      delete next[tabId];
      return next;
    });
  };

  const setPasteConfirmSkip = (workspaceId: string, value: boolean) => {
    setSkipPasteConfirmByWorkspace((prev) => ({
      ...prev,
      [workspaceId]: value,
    }));
  };

  const removePasteConfirmSkip = (workspaceId: string) => {
    setSkipPasteConfirmByWorkspace((prev) => {
      const next = { ...prev };
      delete next[workspaceId];
      return next;
    });
  };

  const findTab = (tabId: string): TabState | null => {
    for (const tabs of Object.values(tabsByWorkspace())) {
      const match = tabs.find((tab) => tab.tabId === tabId);
      if (match) {
        return match;
      }
    }

    return null;
  };

  const syncWorkspaceTabs = (workspaceId: string, tabs: TabState[]) => {
    setTabsByWorkspace((prev) => ({
      ...prev,
      [workspaceId]: tabs,
    }));

    setActiveTabByWorkspace((prev) => ({
      ...prev,
      [workspaceId]:
        prev[workspaceId] && tabs.some((tab) => tab.tabId === prev[workspaceId])
          ? prev[workspaceId]
          : (tabs[0]?.tabId ?? null),
    }));

    for (const tab of tabs) {
      upsertRuntime(tab.tabId, (runtime) => runtime);
    }
  };

  const killTabPty = async (tabId: string) => {
    try {
      await invoke("tab_pty_kill", { tabId });
    } catch (error) {
      if (!errorMessage(error).includes("tab not found")) {
        throw error;
      }
    }
  };

  const dropWorkspaceState = (workspaceId: string) => {
    const tabs = tabsByWorkspace()[workspaceId] ?? [];
    for (const tab of tabs) {
      paneApis.delete(tab.tabId);
      newlyCreatedTabIds.delete(tab.tabId);
      removeRuntime(tab.tabId);
    }

    removeWorkspaceTabs(workspaceId);
    removeWorkspaceActiveTab(workspaceId);
    removePasteConfirmSkip(workspaceId);
  };

  const ensureWorkspaceReady = async (workspace: WorkspaceMetadata) => {
    const workspaceId = workspace.workspaceId;
    const existingTabs = tabsByWorkspace()[workspaceId] ?? [];
    if (existingTabs.length > 0) {
      if (!activeTabByWorkspace()[workspaceId]) {
        setWorkspaceActiveTab(workspaceId, existingTabs[0]?.tabId ?? null);
      }
      return;
    }

    if (loadingWorkspaces.has(workspaceId)) {
      return;
    }

    loadingWorkspaces.add(workspaceId);

    try {
      const response = await invoke<TabListResponse>("tab_list", {
        workspaceId,
      });
      for (const tab of response.tabs) {
        newlyCreatedTabIds.delete(tab.tabId);
      }

      const tabs =
        response.tabs.length > 0
          ? response.tabs
          : [
              await invoke<TabState>("tab_create", {
                req: {
                  workspaceId,
                  name: null,
                  shell: null,
                  cwd: workspace.path,
                } satisfies TabCreateRequest,
              }),
            ];

      if (response.tabs.length === 0 && tabs[0]) {
        newlyCreatedTabIds.add(tabs[0].tabId);
      }
      syncWorkspaceTabs(workspaceId, tabs);
      setPasteConfirmSkip(workspaceId, false);
    } catch (error) {
      showToast(errorMessage(error));
    } finally {
      loadingWorkspaces.delete(workspaceId);
    }
  };

  const startTab = async (tab: TabState, cols: number, rows: number) => {
    upsertRuntime(tab.tabId, (runtime) => ({
      ...runtime,
      phase: "starting",
      spawnError: null,
      exitCode: null,
      cols,
      rows,
    }));

    try {
      await invoke("tab_pty_spawn", {
        req: {
          tabId: tab.tabId,
          shell: tab.shell,
          cwd: tab.cwd,
          cols,
          rows,
        } satisfies PtySpawnRequest,
      });

      upsertRuntime(tab.tabId, (runtime) => ({
        ...runtime,
        phase: "starting",
        spawnError: null,
        exitCode: null,
        cols,
        rows,
      }));
    } catch (error) {
      const message = errorMessage(error);
      upsertRuntime(tab.tabId, (runtime) => ({
        ...runtime,
        phase: "error",
        spawnError: message,
        exitCode: null,
        cols,
        rows,
      }));
      showToast(`无法启动 shell：${message}`);
    }
  };

  const createTab = async (workspace: WorkspaceMetadata) => {
    try {
      const tab = await invoke<TabState>("tab_create", {
        req: {
          workspaceId: workspace.workspaceId,
          name: null,
          shell: null,
          cwd: workspace.path,
        } satisfies TabCreateRequest,
      });

      newlyCreatedTabIds.add(tab.tabId);
      updateWorkspaceTabs(workspace.workspaceId, (tabs) => [tab, ...tabs]);
      setWorkspaceActiveTab(workspace.workspaceId, tab.tabId);
      upsertRuntime(tab.tabId, (runtime) => ({
        ...runtime,
        phase: "idle",
        spawnError: null,
        exitCode: null,
      }));
    } catch (error) {
      showToast(errorMessage(error));
    }
  };

  const renameTab = async (tabId: string, name: string) => {
    const tab = findTab(tabId);
    if (!tab) {
      return;
    }

    try {
      const updated = await invoke<TabState>("tab_rename", {
        req: {
          tabId,
          name,
        } satisfies TabRenameRequest,
      });

      updateWorkspaceTabs(tab.workspaceId, (tabs) =>
        tabs.map((item) => (item.tabId === tabId ? updated : item)),
      );
    } catch (error) {
      showToast(errorMessage(error));
    }
  };

  const closeTab = async (tabId: string) => {
    const tab = findTab(tabId);
    if (!tab) {
      return;
    }

    const tabs = tabsByWorkspace()[tab.workspaceId] ?? [];
    const isLastTab = tabs.length === 1;

    if (isLastTab) {
      const confirmed = await ask("关闭 workspace？", {
        title: "Vibestation",
        kind: "warning",
      });
      if (!confirmed) {
        return;
      }
    }

    try {
      await killTabPty(tabId);
      await invoke("tab_close", {
        req: {
          tabId,
        } satisfies TabCloseRequest,
      });

      newlyCreatedTabIds.delete(tabId);
      removeRuntime(tabId);
      paneApis.delete(tabId);

      if (isLastTab) {
        dropWorkspaceState(tab.workspaceId);
        props.onCloseWorkspaceView(tab.workspaceId);
        return;
      }

      updateWorkspaceTabs(tab.workspaceId, (items) =>
        items.filter((item) => item.tabId !== tabId),
      );

      if (activeTabByWorkspace()[tab.workspaceId] === tabId) {
        setWorkspaceActiveTab(tab.workspaceId, pickSiblingTabId(tabs, tabId));
      }
    } catch (error) {
      showToast(errorMessage(error));
    }
  };

  const closeWorkspaceTabs = async (workspaceId: string) => {
    const tabs = tabsByWorkspace()[workspaceId] ?? [];
    for (const tab of tabs) {
      try {
        await killTabPty(tab.tabId);
      } catch (error) {
        showToast(errorMessage(error));
      }
    }

    dropWorkspaceState(workspaceId);
  };

  const handleResize = async (tabId: string, cols: number, rows: number) => {
    const previous = runtimeByTabId()[tabId] ?? DEFAULT_RUNTIME_STATE;
    if (previous.cols === cols && previous.rows === rows) {
      return;
    }

    upsertRuntime(tabId, (runtime) => ({
      ...runtime,
      cols,
      rows,
    }));

    if (previous.phase !== "running") {
      return;
    }

    try {
      await invoke("tab_pty_resize", { tabId, cols, rows });
    } catch (error) {
      showToast(errorMessage(error));
    }
  };

  const handleExit = (tabId: string, exitCode: number | null) => {
    upsertRuntime(tabId, (runtime) => ({
      ...runtime,
      phase: "exited",
      exitCode,
      spawnError: null,
    }));
  };

  const handleRendererChange = (tabId: string, renderer: RendererKind) => {
    upsertRuntime(tabId, (runtime) => ({
      ...runtime,
      renderer,
    }));
  };

  const handleStdout = (tabId: string) => {
    upsertRuntime(tabId, (runtime) =>
      runtime.phase === "starting"
        ? {
            ...runtime,
            phase: "running",
          }
        : runtime,
    );
  };

  const focusActivePane = () => {
    const tabId = currentActiveTabId();
    if (!tabId) {
      return;
    }

    paneApis.get(tabId)?.focus();
  };

  const confirmPaste = (rememberForWorkspace: boolean) => {
    const pending = pendingPaste();
    if (!pending) {
      return;
    }

    if (rememberForWorkspace) {
      setPasteConfirmSkip(pending.workspaceId, true);
    }

    const paneApi = paneApis.get(pending.tabId);
    if (paneApi) {
      paneApi.paste(pending.text);
      paneApi.focus();
    } else {
      void invoke("tab_pty_stdin", {
        tabId: pending.tabId,
        data: pending.text,
      }).catch((error) => showToast(errorMessage(error)));
    }

    setPendingPaste(null);
  };

  const handleShortcutAction = (action: ShortcutAction) => {
    const workspace = props.activeWorkspace();
    if (!workspace) {
      return;
    }

    const tabs = currentTabs();
    switch (action.kind) {
      case "new-tab":
        void createTab(workspace);
        return;
      case "close-tab":
        if (currentActiveTabId()) {
          void closeTab(currentActiveTabId() as string);
        }
        return;
      case "previous-tab":
        setWorkspaceActiveTab(
          workspace.workspaceId,
          pickAdjacentTabId(tabs, currentActiveTabId(), -1),
        );
        return;
      case "next-tab":
        setWorkspaceActiveTab(
          workspace.workspaceId,
          pickAdjacentTabId(tabs, currentActiveTabId(), 1),
        );
        return;
      case "jump-tab":
        if (tabs[action.index]) {
          setWorkspaceActiveTab(
            workspace.workspaceId,
            tabs[action.index].tabId,
          );
        }
        return;
    }
  };

  useKeybindings(
    () => props.activeWorkspace() !== null && pendingPaste() === null,
    (action) => handleShortcutAction(action),
  );

  createEffect(() => {
    const workspace = props.activeWorkspace();
    if (!workspace) {
      return;
    }

    void ensureWorkspaceReady(workspace);
  });

  createEffect(() => {
    const knownWorkspaceIds = new Set(
      props.workspaces().map((workspace) => workspace.workspaceId),
    );

    for (const workspaceId of Object.keys(tabsByWorkspace())) {
      if (
        knownWorkspaceIds.has(workspaceId) ||
        removingWorkspaces.has(workspaceId)
      ) {
        continue;
      }

      removingWorkspaces.add(workspaceId);
      void closeWorkspaceTabs(workspaceId).finally(() => {
        removingWorkspaces.delete(workspaceId);
      });
    }
  });

  createEffect(() => {
    currentActiveTabId();
    queueMicrotask(() => focusActivePane());
  });

  let unlistenMenu: UnlistenFn | undefined;

  onMount(async () => {
    unlistenMenu = await listen<{ action: string }>("menu:action", (event) => {
      const workspace = props.activeWorkspace();
      if (!workspace) {
        return;
      }

      const tabId = currentActiveTabId();
      const tabs = currentTabs();

      switch (event.payload.action) {
        case "close_tab":
          if (tabId) {
            void closeTab(tabId);
          }
          break;
        case "close_other_tabs": {
          if (!tabId) {
            break;
          }
          // 串行 await · 防止并发 ask() 弹多对话框 / IPC tab_close 后端竞态（round 2 fix WARN-2）
          (async () => {
            for (const t of tabs) {
              if (t.tabId !== tabId) {
                await closeTab(t.tabId);
              }
            }
          })().catch(() => {});
          break;
        }
        case "close_tabs_to_right": {
          if (!tabId) {
            break;
          }
          const idx = tabs.findIndex((t) => t.tabId === tabId);
          if (idx >= 0) {
            // 同 close_other_tabs · 串行避免并发竞态
            (async () => {
              for (let i = idx + 1; i < tabs.length; i++) {
                await closeTab(tabs[i].tabId);
              }
            })().catch(() => {});
          }
          break;
        }
        case "rename_tab":
          if (tabId) {
            setPendingRenameTabId(tabId);
          }
          break;
        case "duplicate_tab": {
          if (!tabId) {
            break;
          }
          const src = findTab(tabId);
          if (src) {
            void invoke<TabState>("tab_create", {
              req: {
                workspaceId: src.workspaceId,
                name: `${src.name} (copy)`,
                shell: src.shell,
                cwd: src.cwd,
              } satisfies TabCreateRequest,
            })
              .then((newTab) => {
                updateWorkspaceTabs(src.workspaceId, (prev) => [
                  ...prev,
                  newTab,
                ]);
                setWorkspaceActiveTab(src.workspaceId, newTab.tabId);
              })
              .catch((err) => showToast(errorMessage(err)));
          }
          break;
        }
        case "new_tab":
          void createTab(workspace);
          break;
        case "split_horizontal":
        case "split_vertical":
          showToast(
            `Split ${event.payload.action === "split_horizontal" ? "horizontal" : "vertical"} · 即将推出`,
            "info",
          );
          break;
        case "clear_terminal":
          if (tabId) {
            paneApis.get(tabId)?.clear();
          }
          break;
        case "copy":
          if (tabId) {
            paneApis.get(tabId)?.copy();
          }
          break;
        case "paste":
          if (tabId) {
            void navigator.clipboard.readText().then((text) => {
              if (text) {
                paneApis.get(tabId)?.paste(text);
              }
            });
          }
          break;
        case "select_all":
          if (tabId) {
            paneApis.get(tabId)?.selectAll();
          }
          break;
      }
    });
  });

  onCleanup(() => {
    if (toastTimer) {
      clearTimeout(toastTimer);
    }
    unlistenMenu?.();
  });

  return (
    <div class="vs-terminal-shell">
      <div class="vs-terminal-workspace-row">
        <div class="vs-terminal-workspace-meta">
          <span class="vs-terminal-workspace-name">
            {props.activeWorkspace()?.name}
          </span>
          <span class="vs-terminal-workspace-path">
            {props.activeWorkspace()?.path}
          </span>
        </div>
        <div class="vs-terminal-session-meta">
          {activeRenderer()
            ? `${activeRenderer()} renderer`
            : "renderer pending"}
        </div>
      </div>

      <TabBar
        tabs={currentTabs()}
        activeTabId={currentActiveTabId()}
        pendingRenameTabId={pendingRenameTabId() ?? undefined}
        onCreate={() => {
          const workspace = props.activeWorkspace();
          if (workspace) {
            void createTab(workspace);
          }
        }}
        onClose={(tabId) => {
          void closeTab(tabId);
        }}
        onRename={(tabId, name) => {
          setPendingRenameTabId(null);
          return renameTab(tabId, name);
        }}
        onSelect={(tabId) => {
          const workspaceId = activeWorkspaceId();
          if (workspaceId) {
            setWorkspaceActiveTab(workspaceId, tabId);
          }
        }}
      />

      <div class="vs-terminal-stage">
        <For each={allTabs()}>
          {(tab) => (
            <TerminalPane
              active={
                tab.workspaceId === activeWorkspaceId() &&
                tab.tabId === currentActiveTabId()
              }
              isNewlyCreated={newlyCreatedTabIds.has(tab.tabId)}
              pasteGuardDisabled={
                skipPasteConfirmByWorkspace()[tab.workspaceId] ?? false
              }
              runtime={runtimeByTabId()[tab.tabId] ?? DEFAULT_RUNTIME_STATE}
              tab={tab}
              onExit={handleExit}
              onPasteRequest={(tabId, text) =>
                setPendingPaste({
                  tabId,
                  text,
                  workspaceId: tab.workspaceId,
                })
              }
              onRegisterApi={(tabId, api) => {
                paneApis.set(tabId, api);
              }}
              onRendererChange={handleRendererChange}
              onResize={handleResize}
              onStart={startTab}
              onStdinError={showToast}
              onStdout={handleStdout}
              onUnregisterApi={(tabId) => {
                paneApis.delete(tabId);
              }}
            />
          )}
        </For>
      </div>

      <Show when={toast()}>
        {(value) => (
          <div
            class={`vs-terminal-toast ${value().kind === "error" ? "is-error" : ""}`}
            role="alert"
          >
            {value().message}
          </div>
        )}
      </Show>

      <Show when={pendingPaste()}>
        {(value) => (
          <PasteConfirmDialog
            text={value().text}
            onCancel={() => {
              setPendingPaste(null);
              focusActivePane();
            }}
            onConfirm={confirmPaste}
          />
        )}
      </Show>
    </div>
  );
};
