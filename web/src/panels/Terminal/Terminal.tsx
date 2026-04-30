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
  LayoutApplyRequest,
  PaneCloseRequest,
  PaneCreateRequest,
  PaneFocusRequest,
  PaneInitRequest,
  PaneListResponse,
  PtySpawnRequest,
  SplitDir,
  TabCloseRequest,
  TabCreateRequest,
  TabListResponse,
  TabRenameRequest,
  TabState,
  WorkspaceMetadata,
} from "../../bindings";
import { PaneSplitView } from "./PaneSplitView";
import { PasteConfirmDialog } from "./PasteConfirmDialog";
import { SmartLayoutMenu, type SmartLayoutPreset } from "./SmartLayoutMenu";
import { TabBar } from "./TabBar";
import { TerminalPane } from "./TerminalPane";
import { usePaneShortcuts } from "./usePaneShortcuts";
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
  // 右键命中的 tab id · TabBar onContextMenu 立刻 set · menu:action listener 优先用此
  // 值而非 currentActiveTabId · 防止右键非 active tab 时操作误作用到 active tab。
  const [contextMenuTabId, setContextMenuTabId] = createSignal<string | null>(
    null,
  );
  /**
   * MVP-05 Phase C · Pane mode state per tab。tab 在 panesByTabId 里 → 渲染 PaneSplitView
   * 用 pane_pty_*；不在则走 legacy TerminalPane（tab_pty_*）。新 tab 创建时调
   * `pane_init_for_tab` 自动入 pane mode；旧 tab 不强制迁移（保数据）。
   */
  const [panesByTabId, setPanesByTabId] = createSignal<
    Record<string, PaneListResponse>
  >({});
  /**
   * MVP-05 Phase C §C · Smart Layouts 命令面板开关 · ⌘⇧P 触发。
   */
  const [smartLayoutOpen, setSmartLayoutOpen] = createSignal(false);

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

  // <For> 按 element identity 比较 · tab object 任何字段变（如 name rename）会创建新对象
  // 触发 unmount + mount · PaneSplitView/PaneTerminal 整体重建 · onCleanup 调 pane_pty_kill
  // 终端内容丢失。改用稳定的 tabId 数组作为 <For> 的 each · 只在 tab 增删时 reconcile ·
  // rename 不影响 tabId 集合 · 不 reconcile · 现有 component 保留 · 只是 reactive props 更新。
  const allTabIds = createMemo(() => allTabs().map((tab) => tab.tabId), [], {
    equals: (a, b) => a.length === b.length && a.every((v, i) => v === b[i]),
  });

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

  /**
   * MVP-05 Phase C · 把 PaneListResponse 写入 panesByTabId（split / close / focus IPC 的统一入口）。
   * Caller 调 IPC 后传 response · 这里更新本地状态 · 触发 PaneSplitView 重渲染。
   */
  const setPaneListForTab = (tabId: string, response: PaneListResponse) => {
    setPanesByTabId((prev) => ({
      ...prev,
      [tabId]: response,
    }));
  };

  /**
   * MVP-05 Phase C · 当前 active tab 是否处于 pane mode（panesByTabId 有数据）。
   */
  const activePaneList = createMemo<PaneListResponse | null>(() => {
    const tabId = currentActiveTabId();
    if (!tabId) return null;
    return panesByTabId()[tabId] ?? null;
  });

  /**
   * 当前 active tab 的 focused_pane_id · null = tab 不在 pane mode 或无聚焦。
   * 由 backend PaneListResponse.focusedPaneId 直接给（PR #143 加 ·session 19 Track A 接通）。
   */
  const activeFocusedPaneId = createMemo<string | null>(
    () => activePaneList()?.focusedPaneId ?? null,
  );

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

      // 让所有 tabs 都进 pane mode · 跟 createTab 路径一致 · 否则首个 tab 走 legacy
      // <TerminalPane> + .vs-terminal-host（有 line-soft 内边框）· 而后续新 tab 走
      // <PaneTerminal> + .vs-pane-terminal-host（无 border）· 视觉不一致。
      // pane_init_for_tab 是 idempotent · 已 init 的 tab 直接返回当前 layout。
      for (const tab of tabs) {
        try {
          const paneList = await invoke<PaneListResponse>("pane_init_for_tab", {
            req: {
              tabId: tab.tabId,
              shell: tab.shell,
              cwd: tab.cwd,
            } satisfies PaneInitRequest,
          });
          setPaneListForTab(tab.tabId, paneList);
        } catch (paneError) {
          // 失败时不阻塞 · tab 退化到 legacy TerminalPane（仍 work · 仅视觉差异）
          // eslint-disable-next-line no-console
          console.warn(
            `[mvp-05] pane_init_for_tab failed for ${tab.tabId}:`,
            paneError,
          );
        }
      }
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
      // MVP-20 · invoke 返回 SpawnResult{warm} · 但 legacy TerminalPane 路径暂不消费
      // warm 字段（buffer 逻辑只在主路径 PaneTerminal.tsx）· 这里仅匹配返回类型
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

      // 先 await pane_init_for_tab 拿到 paneList · 再一次性 update store ·
      // 避免 tab 先 render 走 fallback <TerminalPane>（"Launching..." 一闪即逝）·
      // 又切到 <PaneSplitView> 重新 mount 的双跳。
      let paneList: PaneListResponse | null = null;
      try {
        paneList = await invoke<PaneListResponse>("pane_init_for_tab", {
          req: {
            tabId: tab.tabId,
            shell: tab.shell,
            cwd: tab.cwd,
          } satisfies PaneInitRequest,
        });
      } catch (paneError) {
        showToast(
          `Pane 初始化失败：${errorMessage(paneError)} · tab 退化到单 PTY 模式`,
          "info",
        );
      }

      updateWorkspaceTabs(workspace.workspaceId, (tabs) => [tab, ...tabs]);
      setWorkspaceActiveTab(workspace.workspaceId, tab.tabId);
      upsertRuntime(tab.tabId, (runtime) => ({
        ...runtime,
        phase: "idle",
        spawnError: null,
        exitCode: null,
      }));
      if (paneList) {
        setPaneListForTab(tab.tabId, paneList);
      }
    } catch (error) {
      showToast(errorMessage(error));
    }
  };

  /**
   * MVP-05 Phase C Track A · ⌘\ ⌘⇧\ 触发 split。仅当 active tab 在 pane mode 时生效。
   */
  const handlePaneSplit = async (
    direction: SplitDir,
    focusedPaneId: string,
  ) => {
    const tabId = currentActiveTabId();
    if (!tabId) return;
    const tab = findTab(tabId);
    if (!tab) return;
    // MVP-05 Phase C §F.4 instrumentation · keydown → DOM commit P99 < 150ms 目标
    const t0 = performance.now();
    try {
      const response = await invoke<PaneListResponse>("pane_split", {
        req: {
          tabId,
          parentPaneId: focusedPaneId,
          direction,
          shell: tab.shell,
        } satisfies PaneCreateRequest,
      });
      setPaneListForTab(tabId, response);
      // 等下一帧 · 让 SolidJS 把新 pane DOM commit · 更接近"渲染完成"语义
      requestAnimationFrame(() => {
        const dt = performance.now() - t0;
        // eslint-disable-next-line no-console
        console.info(
          `[mvp-05][F.4] pane_split ${direction} → DOM commit: ${dt.toFixed(1)}ms`,
        );
      });
    } catch (error) {
      const msg = errorMessage(error);
      // §A spec：超单层上限时 backend InvalidLayout · 给 toast 提示
      if (
        msg.includes("invalid pane layout") ||
        msg.includes("InvalidLayout")
      ) {
        showToast("Pane 已达单层上限 · v0.2 将支持任意嵌套");
      } else {
        showToast(`Pane 分屏失败：${msg}`);
      }
    }
  };

  /**
   * MVP-05 Phase C Track A · ⌘⌃W 关当前 pane。若仅剩 1 个 pane → 关整个 tab（spec §A）。
   */
  const handlePaneClose = async (paneId: string) => {
    const tabId = currentActiveTabId();
    if (!tabId) return;
    const list = panesByTabId()[tabId];
    if (!list) return;
    if (list.panes.length <= 1) {
      // 仅 1 个 pane → 关整个 tab（spec §A）
      void closeTab(tabId);
      return;
    }
    // MVP-05 Phase C §F.5 instrumentation · keydown → 重排 DOM commit P99 < 100ms 目标
    const t0 = performance.now();
    try {
      const response = await invoke<PaneListResponse>("pane_close", {
        req: {
          paneId,
        } satisfies PaneCloseRequest,
      });
      setPaneListForTab(tabId, response);
      requestAnimationFrame(() => {
        const dt = performance.now() - t0;
        // eslint-disable-next-line no-console
        console.info(
          `[mvp-05][F.5] pane_close → 重排 DOM commit: ${dt.toFixed(1)}ms`,
        );
      });
    } catch (error) {
      showToast(`Pane 关闭失败：${errorMessage(error)}`);
    }
  };

  /**
   * MVP-05 Phase C Track A · 点击 pane 切焦点 · 同步 backend tabs.focused_pane_id。
   */
  const handlePaneFocus = async (paneId: string) => {
    const tabId = currentActiveTabId();
    if (!tabId) return;
    const list = panesByTabId()[tabId];
    if (!list || list.focusedPaneId === paneId) return;
    try {
      const response = await invoke<PaneListResponse>("pane_focus", {
        req: {
          tabId,
          focusedPaneId: paneId,
        } satisfies PaneFocusRequest,
      });
      setPaneListForTab(tabId, response);
    } catch (error) {
      showToast(`Pane 聚焦失败：${errorMessage(error)}`);
    }
  };

  // MVP-05 Phase C Track A · 注册 ⌘\ ⌘⇧\ ⌘⌃W 快捷键（仅 pane mode 生效）
  usePaneShortcuts({
    getFocusedPaneId: () => activeFocusedPaneId(),
    onSplit: (direction, focusedPaneId) => {
      void handlePaneSplit(direction, focusedPaneId);
    },
    onClose: (focusedPaneId) => {
      void handlePaneClose(focusedPaneId);
    },
    shouldSuppress: () => pendingPaste() !== null,
  });

  /**
   * MVP-05 Phase C §C · Smart Layouts 应用 · 调 pane_layout_apply IPC。
   * preset 直接传给 backend（"solo" / "aiAndRunner" 是 backend pane_service.rs 直接 match 的字符串）·
   * onApply 抛 Error 由 SmartLayoutMenu 内部 alert 显示 · 不向上传播。
   */
  const handleSmartLayoutApply = async (preset: SmartLayoutPreset) => {
    const tabId = currentActiveTabId();
    if (!tabId) {
      throw new Error("没有 active tab");
    }
    // MVP-05 Phase C §F.6 instrumentation · 命令面板确认 → 最终布局 DOM commit P99 < 200ms 目标
    const t0 = performance.now();
    const response = await invoke<PaneListResponse>("pane_layout_apply", {
      req: {
        tabId,
        preset,
        confirmed: true,
      } satisfies LayoutApplyRequest,
    });
    setPaneListForTab(tabId, response);
    requestAnimationFrame(() => {
      const dt = performance.now() - t0;
      // eslint-disable-next-line no-console
      console.info(
        `[mvp-05][F.6] pane_layout_apply ${preset} → DOM commit: ${dt.toFixed(1)}ms`,
      );
    });
  };

  /**
   * MVP-05 Phase C §C · ⌘⇧P 快捷键打开 Smart Layouts 命令面板。
   * 仅 pane mode 生效（active tab 在 panesByTabId）· pendingPaste 时不触发。
   */
  onMount(() => {
    const isMac =
      typeof navigator !== "undefined" &&
      navigator.platform.toUpperCase().includes("MAC");
    const handler = (event: KeyboardEvent) => {
      if (pendingPaste()) return;
      const mod = isMac ? event.metaKey : event.ctrlKey;
      if (mod && event.shiftKey && event.key.toLowerCase() === "p") {
        event.preventDefault();
        if (activePaneList()) {
          setSmartLayoutOpen(true);
        } else {
          showToast("Smart Layouts 仅支持 pane 模式 tab · 请新建 tab", "info");
        }
      }
    };
    window.addEventListener("keydown", handler, { capture: true });
    onCleanup(() =>
      window.removeEventListener("keydown", handler, { capture: true }),
    );
  });

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

      // tab context menu (menu_show_tab) 操作目标是右键命中的 tab · 不是当前 active。
      // 非 tab 类操作（preferences / new_tab）contextTab 也可能被设但用不到 · 处理完
      // 都清。fallback 到 active 是为了 keyboard shortcut 触发同 action 时仍能工作。
      const ctxTab = contextMenuTabId();
      const tabId = ctxTab ?? currentActiveTabId();
      const tabs = currentTabs();
      // 处理完一次菜单事件就清 · 不影响下次右键 / 快捷键路径
      setContextMenuTabId(null);

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
        onContextMenuTab={(tabId) => setContextMenuTabId(tabId)}
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
        <For each={allTabIds()}>
          {(tabId) => {
            // tab 是 reactive memo · rename 时新 tab object 流过来 · 但 component 不重 mount ·
            // tab() 返回新值 · 下游 props 自动 reactive update · 不 unmount/remount。
            const tab = createMemo(() =>
              allTabs().find((t) => t.tabId === tabId),
            );
            const tabActive = () =>
              tab()?.workspaceId === activeWorkspaceId() &&
              tabId === currentActiveTabId();
            const paneList = () => panesByTabId()[tabId];
            // MVP-05 Phase C Track A · 双路渲染：tab 在 pane mode → PaneSplitView · 否则 legacy TerminalPane
            return (
              <Show when={tab()}>
                {(currentTab) => (
                  <Show
                    when={paneList()}
                    fallback={
                      <TerminalPane
                        active={tabActive()}
                        isNewlyCreated={newlyCreatedTabIds.has(tabId)}
                        pasteGuardDisabled={
                          skipPasteConfirmByWorkspace()[
                            currentTab().workspaceId
                          ] ?? false
                        }
                        runtime={
                          runtimeByTabId()[tabId] ?? DEFAULT_RUNTIME_STATE
                        }
                        tab={currentTab()}
                        onExit={handleExit}
                        onPasteRequest={(tid, text) =>
                          setPendingPaste({
                            tabId: tid,
                            text,
                            workspaceId: currentTab().workspaceId,
                          })
                        }
                        onRegisterApi={(tid, api) => {
                          paneApis.set(tid, api);
                        }}
                        onRendererChange={handleRendererChange}
                        onResize={handleResize}
                        onStart={startTab}
                        onStdinError={showToast}
                        onStdout={handleStdout}
                        onUnregisterApi={(tid) => {
                          paneApis.delete(tid);
                        }}
                      />
                    }
                  >
                    {(list) => (
                      <div
                        class={`vs-pane-tab-host ${tabActive() ? "is-active" : "is-hidden"}`}
                        aria-hidden={!tabActive()}
                      >
                        <PaneSplitView
                          layout={list().layout}
                          panes={list().panes}
                          active={tabActive()}
                          focusedPaneId={list().focusedPaneId}
                          onPaneClick={(paneId) => {
                            void handlePaneFocus(paneId);
                          }}
                          onPaneError={(paneId, message) => {
                            showToast(`Pane ${paneId.slice(0, 8)}: ${message}`);
                          }}
                        />
                      </div>
                    )}
                  </Show>
                )}
              </Show>
            );
          }}
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

      <Show when={smartLayoutOpen() && activePaneList()}>
        {(list) => (
          <SmartLayoutMenu
            open={true}
            panes={list().panes}
            layout={list().layout}
            focusedPaneId={list().focusedPaneId}
            onApply={handleSmartLayoutApply}
            onClose={() => setSmartLayoutOpen(false)}
          />
        )}
      </Show>
    </div>
  );
};
