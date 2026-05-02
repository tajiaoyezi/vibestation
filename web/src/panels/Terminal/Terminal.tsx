import { ask } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  batch,
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
  TabReorderRequest,
  TabState,
  WorkspaceMetadata,
} from "../../bindings";
import { PaneSplitView } from "./PaneSplitView";
import { paneSnapshots } from "./PaneTerminal";
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
  copy?: () => void; // legacy TerminalPane 提供 · pane mode 不一定有
  selectAll?: () => void; // 同上
  serialize?: () => string; // pane mode PaneTerminal 提供 · MVP-05 layout 切换 reload 修复用
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

  /**
   * MVP-05 视觉细节 · 点击 pane 外（sidebar / tab 区 / 空白）时隐藏 focused pane 的蓝色
   * outline。focusedPaneId 仍保留在 store · 方便切回 webview / re-click pane 时恢复。
   * 由全局 mousedown listener 切换 · 在 pane 内点击 → false（显示）· pane 外 → true。
   */
  const [paneFocusSuppressed, setPaneFocusSuppressed] = createSignal(false);

  onMount(() => {
    const onDown = (e: MouseEvent) => {
      const target = e.target as Element | null;
      const insidePane = !!target?.closest?.(
        ".vs-pane-terminal, .vs-pane-actions",
      );
      setPaneFocusSuppressed(!insidePane);
    };
    document.addEventListener("mousedown", onDown, true);
    onCleanup(() => document.removeEventListener("mousedown", onDown, true));
  });

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

  // activeRenderer 之前用于顶部 row 显示 "webgl renderer" 等 debug 信息 ·
  // workspace meta row 整段移到全局 TopBar 后 · 该信息暂不在 UI 显示 ·
  // runtimeByTabId 仍然记录 renderer · 保留 createMemo 以便将来 settings 面板复用。
  void runtimeByTabId; // 防 typecheck 报 unused

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

  /**
   * 统一 paste guard 入口 · 所有 paste 路径（menu paste / pane mode cmd+V / legacy
   * TerminalPane DOM paste）走此 helper · 单点决策 multiline + skipPasteConfirm。
   * - paneId 提供时 · paste 走 paneApis.get(paneId) （pane mode 精确目标）
   * - paneId 未提供时 · paste 走 getActivePaneApi(tabId) (legacy / menu fallback)
   * 防 multiline 内容 bypass guard 直接注入 shell（Codex review #208 round 6 / round 7
   * self-review finding · 修 cmd+V bypass 与 menu paste 行为不一致）。
   */
  const requestPaste = (tabId: string, text: string, paneId: string | null) => {
    if (!text) return;
    const tab = findTab(tabId);
    if (!tab) return;
    const workspaceId = tab.workspaceId;
    const skip = skipPasteConfirmByWorkspace()[workspaceId] ?? false;
    const needsConfirm = /[\r\n]/.test(text) && !skip;
    if (needsConfirm) {
      setPendingPaste({ tabId, text, workspaceId });
    } else if (paneId) {
      paneApis.get(paneId)?.paste(text);
    } else {
      getActivePaneApi(tabId)?.paste(text);
    }
  };

  /**
   * 解析当前 active pane API · 处理 pane mode / legacy fallback 双键 schema：
   * - pane mode（panesByTabId[tabId] 存在）· paneApis 以 paneId 为 key · 用 focusedPaneId 查
   * - legacy fallback（TerminalPane 渲染）· paneApis 以 tabId 为 key · 直接查 tabId
   * 所有 reader（focusActivePane / confirmPaste / menu actions clear/copy/paste/select_all）
   * 必须走此 helper · 不能直接 paneApis.get(tabId) · 否则 pane mode 下 silent no-op（Codex
   * review #208 round 5 finding · grep paneApis.get 找出 6 处违反）。
   */
  const getActivePaneApi = (tabId: string): PaneApi | undefined => {
    const list = panesByTabId()[tabId];
    if (list?.focusedPaneId) {
      return paneApis.get(list.focusedPaneId);
    }
    return paneApis.get(tabId);
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

      // 让所有 tabs 都进 pane mode · 跟 createTab 路径一致 · 否则首个 tab 走 legacy
      // <TerminalPane> + .vs-terminal-host（有 line-soft 内边框）· 而后续新 tab 走
      // <PaneTerminal> + .vs-pane-terminal-host（无 border）· 视觉不一致。
      // pane_init_for_tab 是 idempotent · 已 init 的 tab 直接返回当前 layout。
      //
      // BUG-001 round 2 fix · 先收齐所有 paneList · batch 写入 · 避免 tab 先 render
      // 走 fallback <TerminalPane>（panesByTabId 还没设）· 触发 stale <Show> 警告。
      const collectedPaneLists: Array<[string, PaneListResponse]> = [];
      for (const tab of tabs) {
        try {
          const paneList = await invoke<PaneListResponse>("pane_init_for_tab", {
            req: {
              tabId: tab.tabId,
              shell: tab.shell,
              cwd: tab.cwd,
            } satisfies PaneInitRequest,
          });
          collectedPaneLists.push([tab.tabId, paneList]);
        } catch (paneError) {
          // 失败时不阻塞 · tab 退化到 legacy TerminalPane（仍 work · 仅视觉差异）
          // eslint-disable-next-line no-console
          console.warn(
            `[mvp-05] pane_init_for_tab failed for ${tab.tabId}:`,
            paneError,
          );
        }
      }
      batch(() => {
        for (const [tabId, paneList] of collectedPaneLists) {
          setPaneListForTab(tabId, paneList);
        }
        syncWorkspaceTabs(workspaceId, tabs);
        setPasteConfirmSkip(workspaceId, false);
      });
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

      // 关键：用 batch 把 4 个 signal update 合一帧 render · 避免 tab 先 render 走
      // <TerminalPane> fallback（panesByTabId 还没设）· 然后切到 <PaneSplitView> 重 mount。
      // 双 mount 期间 idle pty stdout 事件触发 stale <Show> accessor 警告。
      // 新 tab 加到末尾（不是开头）· 跟 iTerm / Warp / 浏览器约定一致 · 避免新建时
      // 把当前 tab "推走"造成视觉错位。
      batch(() => {
        if (paneList) {
          setPaneListForTab(tab.tabId, paneList);
        }
        updateWorkspaceTabs(workspace.workspaceId, (tabs) => [...tabs, tab]);
        setWorkspaceActiveTab(workspace.workspaceId, tab.tabId);
        upsertRuntime(tab.tabId, (runtime) => ({
          ...runtime,
          phase: "idle",
          spawnError: null,
          exitCode: null,
        }));
      });
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
      // MVP-05 reload 修复 · split 前 snapshot 所有现有 pane · 让重 mount 后 PaneTerminal
      // 通过 paneSnapshots 恢复 xterm 显示。新 pane 的 paneId 不在快照里 · cold spawn 走默认。
      const currentList = panesByTabId()[tabId];
      if (currentList) {
        for (const pane of currentList.panes) {
          const api = paneApis.get(pane.paneId);
          const snapshot = api?.serialize?.();
          if (snapshot) paneSnapshots.set(pane.paneId, snapshot);
        }
      }
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
      // MVP-05 reload 修复 · close 前 snapshot 保留 panes（被关 paneId 跳过）· 让重 mount
      // 后通过 paneSnapshots 恢复 xterm 显示。
      for (const pane of list.panes) {
        if (pane.paneId === paneId) continue;
        const api = paneApis.get(pane.paneId);
        const snapshot = api?.serialize?.();
        if (snapshot) paneSnapshots.set(pane.paneId, snapshot);
      }
      // 先 commit backend layout close · 成功后再 kill PTY · 顺序逆转后即使 pane_close
      // 失败（pool 未 init / DB 错 / stale paneId）· 用户的 shell 进程仍存活 · 可重试 ·
      // 不会从 "可恢复的 close 失败" 变成 "不可恢复的 session 丢失"（Codex review #208 finding）。
      const response = await invoke<PaneListResponse>("pane_close", {
        req: {
          paneId,
        } satisfies PaneCloseRequest,
      });
      setPaneListForTab(tabId, response);
      // pane_close 已 commit · 现在显式 kill PTY · 因为 PaneTerminal.onCleanup 默认不再 kill。
      // kill 失败不静默吞 · 提示用户存在 PTY leak（pane 已从 layout 移除 · 进程仍在 backend
      // pty_pool · 直到 closeWorkspaceTabs 或应用退出才清理）。
      try {
        await invoke("pane_pty_kill", { paneId });
      } catch (killErr) {
        const msg = errorMessage(killErr);
        if (!/not\s*found|already.*exited/i.test(msg)) {
          showToast(`Pane PTY leak warning: ${msg}`);
        }
      }
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
    // PaneTerminal.onCleanup 默认不再 kill PTY · backend pane_layout_apply 对 solo / aiAndRunner
    // 等 preset 会 DELETE 多余 panes · 前端必须 diff pre/post panes 显式 kill 被删 PTY · 否则
    // 进程留在 pty_pool 没人控制 · resource leak（Codex review #208 finding）。
    // 同时对**保留** panes serialize 快照 · 让 layout 切换后 PaneTerminal 重 mount 时通过
    // paneSnapshots 恢复 xterm 显示 · 否则保留 pane 仍会出现"空白 → 等待 stdout"（round 3 finding）。
    const preList = panesByTabId()[tabId];
    const prePaneIds = new Set(preList?.panes.map((p) => p.paneId) ?? []);
    if (preList) {
      for (const pane of preList.panes) {
        const api = paneApis.get(pane.paneId);
        const snapshot = api?.serialize?.();
        if (snapshot) paneSnapshots.set(pane.paneId, snapshot);
      }
    }
    const response = await invoke<PaneListResponse>("pane_layout_apply", {
      req: {
        tabId,
        preset,
        confirmed: true,
      } satisfies LayoutApplyRequest,
    });
    setPaneListForTab(tabId, response);
    // 比较前后 panes · 显式 kill 被预设删除的 PTY · 失败（已 exited）静默跳过 · 真错误 toast 警告 leak。
    // 同时清掉被删 paneId 的 snapshot · 防内存泄漏 + 防 paneId 复用时 stale snapshot 误用。
    const postPaneIds = new Set(response.panes.map((p) => p.paneId));
    for (const id of prePaneIds) {
      if (!postPaneIds.has(id)) {
        paneSnapshots.delete(id);
        try {
          await invoke("pane_pty_kill", { paneId: id });
        } catch (killErr) {
          const msg = errorMessage(killErr);
          if (!/not\s*found|already.*exited/i.test(msg)) {
            showToast(`Pane PTY leak warning: ${msg}`);
          }
        }
      }
    }
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

  /**
   * MVP-20 follow-up · 拖拽 reorder · 调 backend `tab_reorder` IPC 持久化 ·
   * 用返回的完整 tabs 列表 sync 前端 store · 保证前后端顺序一致。
   * 失败时不动 store · 让用户看到原顺序 + toast。
   *
   * **关键** · 必须保留旧 TabState 对象引用 reorder · 否则 SolidJS keyed `<For>`
   * 看到新引用会 unmount + remount tab DOM · 触发 enter animation 重跑（width 0→240）·
   * 期间 indicator scaleX 拿到中间值 · 视觉上"蓝条漂移 / 闪一下"。
   */
  const reorderTab = async (tabId: string, newIndex: number) => {
    const tab = findTab(tabId);
    if (!tab) return;

    try {
      const reordered = await invoke<TabState[]>("tab_reorder", {
        req: {
          tabId,
          newIndex,
        } satisfies TabReorderRequest,
      });
      // 保留旧 TabState 引用 reorder · 避免 <For> reference 变化触发 remount + 动画重跑
      setTabsByWorkspace((prev) => {
        const oldTabs = prev[tab.workspaceId] ?? [];
        const oldMap = new Map(oldTabs.map((t) => [t.tabId, t]));
        const reorderedRefs = reordered.map((t) => oldMap.get(t.tabId) ?? t);
        return {
          ...prev,
          [tab.workspaceId]: reorderedRefs,
        };
      });
    } catch (error) {
      showToast(`Tab 排序失败：${errorMessage(error)}`);
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
      // 先 commit backend tab_close · 成功后再 kill 所有 PTY（含 legacy tab PTY + panes
      // PTY）· 即使 tab_close 失败 · 用户 shell 进程仍存活 · 可重试。与 handlePaneClose
      // 顺序一致（Codex review #208 round 3 finding · 之前漏了 killTabPty 这行）。
      // pre 阶段 snapshot pane id 列表 · 防 setPaneListForTab 后 store 被清空拿不到。
      const paneList = panesByTabId()[tabId];
      const panePaneIds = paneList?.panes.map((p) => p.paneId) ?? [];
      await invoke("tab_close", {
        req: {
          tabId,
        } satisfies TabCloseRequest,
      });

      // tab_close 已 commit · kill PTY · 失败 toast 警告 leak。
      // pane mode tab 没有 legacy tab_pty session · 跳过 killTabPty 省一次冗余 IPC（Codex
      // round 5 finding · 另 killTabPty 内部已 swallow "tab not found" · 当前不 toast 但
      // pane mode 显式 skip 更干净）。
      if (!paneList) {
        try {
          await killTabPty(tabId);
        } catch (killErr) {
          showToast(`Tab PTY leak warning: ${errorMessage(killErr)}`);
        }
      }
      for (const pid of panePaneIds) {
        paneSnapshots.delete(pid);
        try {
          await invoke("pane_pty_kill", { paneId: pid });
        } catch (killErr) {
          const msg = errorMessage(killErr);
          if (!/not\s*found|already.*exited/i.test(msg)) {
            showToast(`Pane PTY leak warning: ${msg}`);
          }
        }
      }

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
    // workspace cleanup（切走 workspace · 不调 tab_close 保留 DB 状态）· 仅 kill in-memory PTY
    // 进程。kill 失败不静默吞 · toast 警告 leak（一致于 closeTab / handlePaneClose · Codex review）。
    for (const tab of tabs) {
      const paneList = panesByTabId()[tab.tabId];
      if (paneList) {
        for (const pane of paneList.panes) {
          paneSnapshots.delete(pane.paneId);
          try {
            await invoke("pane_pty_kill", { paneId: pane.paneId });
          } catch (killErr) {
            const msg = errorMessage(killErr);
            if (!/not\s*found|already.*exited/i.test(msg)) {
              showToast(`Pane PTY leak warning: ${msg}`);
            }
          }
        }
      }
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

    getActivePaneApi(tabId)?.focus();
  };

  const confirmPaste = (rememberForWorkspace: boolean) => {
    const pending = pendingPaste();
    if (!pending) {
      return;
    }

    if (rememberForWorkspace) {
      setPasteConfirmSkip(pending.workspaceId, true);
    }

    const paneApi = getActivePaneApi(pending.tabId);
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
            getActivePaneApi(tabId)?.clear();
          }
          break;
        case "copy":
          if (tabId) {
            getActivePaneApi(tabId)?.copy?.();
          }
          break;
        case "paste":
          if (tabId) {
            const localTabId = tabId;
            void navigator.clipboard.readText().then((text) => {
              // 统一走 requestPaste · 与 cmd+V / legacy DOM paste 共享 multiline + skip 决策
              requestPaste(localTabId, text ?? "", null);
            });
          }
          break;
        case "select_all":
          if (tabId) {
            getActivePaneApi(tabId)?.selectAll?.();
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
      {/* workspace name + path 已移到全局 TopBar（不受 sidebar 收缩影响） ·
          renderer status 是开发期 debug 信息 · v0.1 alpha 暂不在 UI 显示 ·
          activeRenderer() signal 仍然 set · 后续如需可挪到 settings panel。 */}

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
        onCloseRequested={(tabId) => {
          // BUG-001 follow-up · leave 动画开始时立即切 active 到 sibling tab ·
          // 让底部蓝条 indicator 平滑滑过去 · 不再等 240ms 后"瞬间跳"。
          // 真正 unmount 仍由 onClose（240ms 后）调 closeTab。
          const workspaceId = activeWorkspaceId();
          if (!workspaceId) return;
          if (activeTabByWorkspace()[workspaceId] !== tabId) return;
          const tabs = tabsByWorkspace()[workspaceId] ?? [];
          const sibling = pickSiblingTabId(tabs, tabId);
          setWorkspaceActiveTab(workspaceId, sibling);
        }}
        onRename={(tabId, name) => {
          setPendingRenameTabId(null);
          return renameTab(tabId, name);
        }}
        onReorder={(tabId, newIndex) => {
          void reorderTab(tabId, newIndex);
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
                        class={`vs-pane-tab-host ${tabActive() ? "is-active" : "is-hidden"} ${paneFocusSuppressed() ? "is-focus-suppressed" : ""}`}
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
                          onPaneSplit={(direction, paneId) => {
                            void handlePaneSplit(direction, paneId);
                          }}
                          onPaneClose={(paneId) => {
                            void handlePaneClose(paneId);
                          }}
                          onPanePasteRequest={(paneId, text) => {
                            // pane mode cmd+V 触发 · 走统一 requestPaste · multiline 弹
                            // PasteConfirmDialog · 与 menu paste / legacy paste 行为一致
                            // (Codex round 6 / round 7 self-review finding)。
                            requestPaste(tabId, text, paneId);
                          }}
                          onRegisterPaneApi={(paneId, api) => {
                            // pane mode · paneApis 用 paneId 作 key（与 legacy fallback
                            // 用 tabId 不冲突：同一 tab 同时只走一种 mode）。这里漏接
                            // 会让 handlePaneSplit / handlePaneClose 的 serialize() 永远拿
                            // 不到 PaneTerminalApi · paneSnapshots 永远空 · SerializeAddon
                            // 形同虚设（Codex review #208 finding）。
                            paneApis.set(paneId, api);
                          }}
                          onUnregisterPaneApi={(paneId) => {
                            paneApis.delete(paneId);
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
