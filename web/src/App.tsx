import {
  createSignal,
  createEffect,
  onMount,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit, type UnlistenFn } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import {
  readText as readClipboardText,
  writeText as writeClipboardText,
} from "@tauri-apps/plugin-clipboard-manager";
import "./styles.css";

import { LayoutProvider, useLayout } from "./stores/layout-context";
import {
  RemoteSyncStatusProvider,
  useRemoteSyncStatus,
  type RemoteSyncDirection,
} from "./stores/remote-sync-status";
import { PaneLinksProvider } from "./stores/paneLinks-context";
import { PaneDraftsProvider } from "./stores/paneDrafts-context";
import { SessionsProvider } from "./stores/sessions-context";
import { BottomPanelTabsProvider } from "./stores/bottom-panel-tabs";
import { SessionDetailHost } from "./panels/Sessions/SessionDetailHost";
import { ThemeProvider } from "./stores/theme";
import { useSettings, reloadSettings } from "./stores/settings";
import { PrimarySidebar } from "./components/PrimarySidebar";
import { SecondarySidebar } from "./components/SecondarySidebar";
import { BottomPanel } from "./components/BottomPanel";
import { ActivityStrip } from "./components/ActivityStrip";
import { MainContent } from "./components/MainContent";
import type { DiffTarget } from "./components/MainContent";
import { ThemeSwitch } from "./components/ThemeSwitch";
import { GearIcon } from "./components/Icons";
import { TopBar } from "./components/TopBar";
import { SettingsPanel } from "./panels/Settings";
import { TelemetryOptInModal } from "./dialogs/TelemetryOptIn/TelemetryOptInModal";
import { ConfigImportDialog } from "./dialogs/ConfigImport";
import { BranchSwitcher } from "./dialogs/BranchSwitcher/BranchSwitcher";
import { MergeDialog } from "./dialogs/MergeDialog";
import { PopToExternalDialog } from "./dialogs/PopToExternal/PopToExternalDialog";
import {
  popToExternalRequest,
  setPopToExternalRequest,
} from "./lib/external-term";
import { initPaneDetachStateListener } from "./lib/pane-detach";
import { t, normalizeLanguage } from "./i18n";
import { formatShortcut } from "./lib/format-shortcut";
import { detectPlatform } from "./lib/platform";
import {
  ConflictBanner,
  type ConflictOperation,
} from "./components/ConflictBanner";
import { ThreeWayDiffView } from "./panels/Diff/3way";
import "./styles/mvp16.css";
// MVP-16 Phase C · crash recovery 状态机纯函数（解耦 SolidJS · 可单测）
import {
  type CrashRecoveryPayload,
  type RecoveryDetectResult,
  type RecoveryUiState,
  detectResultToRecoveryState,
  payloadToRecoveryState,
  reduceRecoveries,
} from "./lib/crash-recovery";
// MVP-20 Phase D · rollback crash recovery 状态机纯函数（解耦 SolidJS · 可单测）
import {
  type RollbackRecoveryPayload,
  type RollbackRecoveryUiState,
  payloadToRollbackRecovery,
  reduceRollbackRecoveries,
} from "./lib/rollback-recovery";
import { RollbackRecoveryBanner } from "./panels/SessionDetail/RollbackRecoveryBanner";
import {
  rollbackAbort,
  rollbackExecute,
} from "./panels/SessionDetail/rollbackApi";

// IPC contract types · 由 `crates/app/build.rs` 从 Rust `#[derive(TS)]` 自动生成。
// 禁止手写对偶 interface（SPIKE-08 §A rollout · 防 H2 类 drift）。
import type {
  BranchListResponse,
  MergeStatus,
  RebaseControlRequest,
  WorkspaceMetadata,
} from "./bindings";
export type { WorkspaceMetadata };

type IpcState =
  | { kind: "pending" }
  | { kind: "ok"; message: string }
  | { kind: "error"; message: string };

type View = { kind: "welcome" } | { kind: "workspace"; ws: WorkspaceMetadata };

type RebaseProgressPayload = {
  workspaceId: string;
  currentStep: number;
  totalSteps: number;
  currentCommit: string | null;
};

type ConflictDetectedPayload = {
  workspaceId: string;
  operation: string;
  conflictingFiles: string[];
};

type RebaseOperationDonePayload = {
  workspaceId: string;
  operation: string;
  success: boolean;
  message: string;
};

type ConflictUiState = {
  workspaceId: string;
  operation: ConflictOperation;
  files: string[];
  resolvedFiles: string[];
  currentStep: number;
  totalSteps: number;
  currentCommit: string | null;
};

const IpcIndicator: Component<{ state: IpcState }> = (props) => {
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const chromeLabel = (key: string) => t(key, language());
  const statusLabel = () => {
    switch (props.state.kind) {
      case "pending":
        return chromeLabel("chrome.status.ipcConnecting");
      case "ok":
        return `${chromeLabel("chrome.status.ipcOk")} ${props.state.message}`;
      case "error":
        return `${chromeLabel("chrome.status.ipcError")} ${props.state.message}`;
    }
  };
  const className = () =>
    props.state.kind === "error" ? "vs-diag-error" : "vs-diag-ok";
  return <span class={className()}>{statusLabel()}</span>;
};

const RemoteSyncStatusItem: Component<{
  onOpenGitLog: (direction: RemoteSyncDirection) => void;
}> = (props) => {
  const { settings } = useSettings();
  const remoteSync = useRemoteSyncStatus();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const snapshot = () => remoteSync.current();
  const hasRemoteDelta = () => snapshot().ahead > 0 || snapshot().behind > 0;
  const remoteAheadTitle = () =>
    `${label("chrome.status.remoteAheadTitlePrefix")} ${snapshot().ahead} ${label("chrome.status.remoteAheadTitleSuffix")}`;
  const remoteAheadAria = () =>
    `${label("chrome.status.remoteAheadAriaPrefix")} ${snapshot().ahead} ${label("chrome.status.remoteAheadAriaSuffix")}`;
  const remoteBehindTitle = () =>
    `${label("chrome.status.remoteBehindTitlePrefix")} ${snapshot().behind} ${label("chrome.status.remoteBehindTitleSuffix")}`;
  const remoteBehindAria = () =>
    `${label("chrome.status.remoteBehindAriaPrefix")} ${snapshot().behind} ${label("chrome.status.remoteBehindAriaSuffix")}`;

  return (
    <Show when={hasRemoteDelta()}>
      <span class="vs-status-item vs-status-remote">
        <span class="vs-status-key">{label("chrome.status.remote")}</span>
        <Show when={snapshot().ahead > 0}>
          <button
            type="button"
            class="vs-status-remote-count is-ahead"
            title={remoteAheadTitle()}
            aria-label={remoteAheadAria()}
            onClick={() => props.onOpenGitLog("ahead")}
          >
            ↑{snapshot().ahead}
          </button>
        </Show>
        <Show when={snapshot().behind > 0}>
          <button
            type="button"
            class="vs-status-remote-count is-behind"
            title={remoteBehindTitle()}
            aria-label={remoteBehindAria()}
            onClick={() => props.onOpenGitLog("behind")}
          >
            ↓{snapshot().behind}
          </button>
        </Show>
      </span>
    </Show>
  );
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
  // MVP-06 · 配置导入对话框（首次启动 / Settings 头部触发）
  const [importVisible, setImportVisible] = createSignal(false);
  const [branchSwitcherOpen, setBranchSwitcherOpen] = createSignal(false);
  const [globalMergeOpen, setGlobalMergeOpen] = createSignal(false);
  const [globalMergeBranch, setGlobalMergeBranch] = createSignal("current");
  const [conflict, setConflict] = createSignal<ConflictUiState | null>(null);
  const [conflictBusy, setConflictBusy] = createSignal(false);
  const [conflictError, setConflictError] = createSignal<string | null>(null);
  // MVP-16 Phase C · per-workspace crash recovery banner 状态 · workspace 切换不丢
  // key = workspaceId · null/missing 表示该 workspace 无未完成操作
  const [recoveries, setRecoveries] = createSignal<
    Record<string, RecoveryUiState>
  >({});
  const [recoveryBusy, setRecoveryBusy] = createSignal(false);
  const [recoveryError, setRecoveryError] = createSignal<string | null>(null);
  // MVP-20 Phase D · per-session rollback crash recovery banner 状态
  // key = sessionId（rollback 是 session 维度 · 区别 MVP-16 的 workspace 维度）
  const [rollbackRecoveries, setRollbackRecoveries] = createSignal<
    Record<string, RollbackRecoveryUiState>
  >({});
  const [rollbackRecoveryBusy, setRollbackRecoveryBusy] = createSignal(false);
  const [rollbackRecoveryError, setRollbackRecoveryError] = createSignal<
    string | null
  >(null);
  const remoteSync = useRemoteSyncStatus();
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  const activeWorkspace = (): WorkspaceMetadata | null => {
    const v = props.currentView();
    return v.kind === "workspace" ? v.ws : null;
  };

  const handleTitleBarMouseDown = (event: MouseEvent) => {
    if (event.button !== 0 || event.detail !== 1) {
      return;
    }
    // 跳过交互元素 · 防 startDragging 接管鼠标后吞掉 button onClick
    // （TopBar 收起按钮放在 header 上 · sidebar 收起后是唯一恢复路径 · 不能丢点击）
    const target = event.target as HTMLElement | null;
    if (
      target?.closest(
        "button, a, input, textarea, select, [role=button], [data-no-drag]",
      )
    ) {
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

  const openGitStatusPanel = () => {
    if (!layout().bottomOpen) {
      dispatch({ kind: "toggle-bottom" });
    }
  };

  const openGitLogHighlight = (direction: RemoteSyncDirection) => {
    if (!layout().secondaryOpen) {
      dispatch({ kind: "toggle-secondary" });
    }
    remoteSync.requestGitLogHighlight(direction);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.defaultPrevented) return;
    if (e.target instanceof Element && e.target.closest(".vs-terminal-shell")) {
      return;
    }

    const mod = e.metaKey || e.ctrlKey;
    if (!mod) return;
    switch (e.key) {
      case "b":
      case "B":
        e.preventDefault();
        if (activeWorkspace()?.hasGit) {
          setBranchSwitcherOpen(true);
        }
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

  // Windows app-menu 快捷键 fallback（PR #452 起 Windows 关原生 menu · 见 lib.rs cfg gate）。
  // 原生 menu 同时承载键盘 accelerator · 关掉后这些动作在 Windows 失去键盘来源（右键菜单仍可达）。
  // 这里前端补发 menu:action 事件复用既有 wiring（App.tsx + Terminal.tsx 的 listen("menu:action")）·
  // 触发与原生菜单完全相同的逻辑 · 零 handler 重复。
  //
  // 为何用 Ctrl+Shift 而非裸 Ctrl：Ctrl+T/W/D 撞 shell readline（transpose / kill-word /
  // EOF 直接关 shell）· 故采用 Windows Terminal / VS Code 约定（不撞 readline）：
  //   Ctrl+Shift+T → new_tab · Ctrl+Shift+W → close_tab · Ctrl+, → preferences。
  //
  // 用 capture 阶段注册（先于 xterm 的 bubble-phase handler）+ stopPropagation 阻止键落到终端 ·
  // 故无视终端聚焦也能触发（不同于 handleKeyDown · 后者在 .vs-terminal-shell 内 early-return）。
  const isWindows = detectPlatform() === "windows";
  const winMenuShortcutHandler = (e: KeyboardEvent) => {
    let action: string | null = null;
    // e.code 布局无关（不受键盘布局 / IME 影响）
    if (
      e.ctrlKey &&
      e.shiftKey &&
      !e.altKey &&
      !e.metaKey &&
      e.code === "KeyT"
    ) {
      action = "new_tab";
    } else if (
      e.ctrlKey &&
      e.shiftKey &&
      !e.altKey &&
      !e.metaKey &&
      e.code === "KeyW"
    ) {
      action = "close_tab";
    } else if (
      e.ctrlKey &&
      !e.shiftKey &&
      !e.altKey &&
      !e.metaKey &&
      e.code === "Comma"
    ) {
      action = "preferences";
    }
    if (!action) return;
    e.preventDefault();
    e.stopPropagation();
    void emit("menu:action", { action });
  };

  const normalizeConflictOperation = (operation: string): ConflictOperation => {
    switch (operation) {
      case "merge":
        return "merge";
      case "cherrypick":
        return "cherrypick";
      default:
        return "rebase";
    }
  };

  const allConflictFilesResolved = () => {
    const current = conflict();
    if (!current || current.files.length === 0) {
      return false;
    }
    return current.files.every((file) => current.resolvedFiles.includes(file));
  };

  const currentConflictFile = () => {
    const current = conflict();
    if (!current) {
      return null;
    }
    return (
      current.files.find((file) => !current.resolvedFiles.includes(file)) ??
      current.files[0] ??
      null
    );
  };

  const markConflictFileResolved = (filePath: string) => {
    setConflict((current) => {
      if (!current || current.resolvedFiles.includes(filePath)) {
        return current;
      }
      return {
        ...current,
        resolvedFiles: [...current.resolvedFiles, filePath],
      };
    });
  };

  const openGlobalMergeDialog = async () => {
    const ws = activeWorkspace();
    if (!ws?.hasGit) {
      return;
    }
    setGlobalMergeBranch("current");
    try {
      const req = { workspaceId: ws.workspaceId };
      const response = await invoke<BranchListResponse>("branch_list", { req });
      setGlobalMergeBranch(response.headName ?? "current");
    } catch {
      setGlobalMergeBranch("current");
    }
    setGlobalMergeOpen(true);
  };

  const handleConflictContinue = async () => {
    const current = conflict();
    if (!current || !allConflictFilesResolved()) {
      return;
    }
    setConflictBusy(true);
    setConflictError(null);
    try {
      if (current.operation === "rebase") {
        const req: RebaseControlRequest = {
          workspaceId: current.workspaceId,
          action: "continue",
        };
        await invoke("rebase_continue", { req });
      } else if (current.operation === "cherrypick") {
        await invoke("cherrypick_continue", {
          workspaceId: current.workspaceId,
        });
      } else {
        throw new Error("merge conflict continue 尚未由 Phase A 暴露 IPC");
      }
    } catch (err) {
      setConflictError(err instanceof Error ? err.message : String(err));
    } finally {
      setConflictBusy(false);
    }
  };

  const handleConflictAbort = async () => {
    const current = conflict();
    if (!current) {
      return;
    }
    setConflictBusy(true);
    setConflictError(null);
    try {
      if (current.operation === "rebase") {
        const req: RebaseControlRequest = {
          workspaceId: current.workspaceId,
          action: "abort",
        };
        await invoke("rebase_abort", { req });
      } else if (current.operation === "cherrypick") {
        await invoke("cherrypick_abort", {
          workspaceId: current.workspaceId,
        });
      } else {
        await invoke("merge_abort", { workspaceId: current.workspaceId });
      }
      setConflict(null);
    } catch (err) {
      setConflictError(err instanceof Error ? err.message : String(err));
    } finally {
      setConflictBusy(false);
    }
  };

  const handleConflictSkip = async () => {
    const current = conflict();
    if (!current || current.operation === "merge") {
      return;
    }
    setConflictBusy(true);
    setConflictError(null);
    const req: RebaseControlRequest = {
      workspaceId: current.workspaceId,
      action: "skip",
    };
    try {
      await invoke("rebase_skip", { req });
    } catch (err) {
      setConflictError(err instanceof Error ? err.message : String(err));
    } finally {
      setConflictBusy(false);
    }
  };

  // MVP-16 Phase C · 当前 active workspace 的 recovery 状态（响应式 · 切换 workspace 时切换）
  const activeRecovery = (): RecoveryUiState | null => {
    const ws = activeWorkspace();
    if (!ws) return null;
    return recoveries()[ws.workspaceId] ?? null;
  };

  // MVP-16 Phase C · 写入 / 清除单 workspace recovery 状态（不影响其他 workspace · 不可变 patch）
  // 委托 lib/crash-recovery::reduceRecoveries · 单测在那边
  const upsertRecovery = (
    workspaceId: string,
    next: RecoveryUiState | null,
  ) => {
    setRecoveries((prev) => reduceRecoveries(prev, workspaceId, next));
  };

  // MVP-16 Phase C · 收到 crash recovery payload（来源：启动 emit / 切 workspace IPC 主动查）
  const ingestRecoveryPayload = (payload: CrashRecoveryPayload) => {
    upsertRecovery(payload.workspaceId, payloadToRecoveryState(payload));
  };

  // MVP-16 Phase C · workspace 切换或 dev 启动后主动查 in-progress 状态
  // 用 IPC 的原因：启动 emit 是 best-effort（在 workspace_init 完成时一次性扫）·
  // 之后切 workspace 需要 frontend 主动询问目标 workspace。
  const refreshRecoveryForWorkspace = async (workspaceId: string) => {
    try {
      // CrashRecoveryState 字段对齐 ts-rs binding · 不手写 interface
      const result = await invoke<RecoveryDetectResult>(
        "rebase_detect_in_progress",
        { workspaceId },
      );
      upsertRecovery(
        workspaceId,
        detectResultToRecoveryState(workspaceId, result),
      );
    } catch (err) {
      // 检测失败不影响主流程 · 控制台留痕（不弹 toast 避免烦扰）
      // eslint-disable-next-line no-console
      console.warn(
        `[mvp-16] crash recovery detect failed for ${workspaceId}:`,
        err instanceof Error ? err.message : String(err),
      );
    }
  };

  // MVP-16 Phase C · Continue：调对应 IPC 让 backend 把状态机推进到下一 step。
  // 若推进后仍有 conflict · 现有 `git:conflict-detected` event 会渲染 active banner（接管）·
  // 若完成 · `git:operation-done` event 会清 conflict + 这里清 recovery。
  const handleRecoveryContinue = async () => {
    const current = activeRecovery();
    if (!current) return;
    setRecoveryBusy(true);
    setRecoveryError(null);
    try {
      if (current.operation === "rebase") {
        const req: RebaseControlRequest = {
          workspaceId: current.workspaceId,
          action: "continue",
        };
        await invoke("rebase_continue", { req });
      } else if (current.operation === "cherrypick") {
        await invoke("cherrypick_continue", {
          workspaceId: current.workspaceId,
        });
      } else {
        // merge crash recovery：git2 `MERGE_HEAD` 存在但还未 commit
        // backend 没有 merge_continue · 用户应自行 commit / abort · 给清晰提示
        throw new Error(
          "merge 中断恢复需手动在 Status 面板 commit · 或选 Abort",
        );
      }
    } catch (err) {
      setRecoveryError(err instanceof Error ? err.message : String(err));
    } finally {
      setRecoveryBusy(false);
    }
  };

  // MVP-16 Phase C · Abort：清 .git 状态 + remove operation_state file · 回原 HEAD。
  // 复用 backend 已有 `*_abort` IPC（同 active conflict abort 路径 · 语义一致）。
  const handleRecoveryAbort = async () => {
    const current = activeRecovery();
    if (!current) return;
    setRecoveryBusy(true);
    setRecoveryError(null);
    try {
      if (current.operation === "rebase") {
        const req: RebaseControlRequest = {
          workspaceId: current.workspaceId,
          action: "abort",
        };
        await invoke("rebase_abort", { req });
      } else if (current.operation === "cherrypick") {
        await invoke("cherrypick_abort", {
          workspaceId: current.workspaceId,
        });
      } else {
        await invoke("merge_abort", { workspaceId: current.workspaceId });
      }
      // backend 成功后会 emit `git:operation-done` success=true · 但本地立即清也无伤
      upsertRecovery(current.workspaceId, null);
    } catch (err) {
      setRecoveryError(err instanceof Error ? err.message : String(err));
    } finally {
      setRecoveryBusy(false);
    }
  };

  // MVP-16 Phase C · View status：dismiss banner + 确保 Secondary Sidebar 打开（Git Log）。
  // 不导航到具体 commit · 不高亮 rebase 起点（v0.4+ polish · 需 Git Log scroll-to-oid 能力）。
  const handleRecoveryViewStatus = () => {
    const current = activeRecovery();
    if (!current) return;
    upsertRecovery(current.workspaceId, null);
    if (!layout().secondaryOpen) {
      dispatch({ kind: "toggle-secondary" });
    }
  };

  // ── MVP-20 Phase D · rollback crash recovery banner（spec §H.9 · §I）──
  // rollback 是 session 维度（区别 MVP-16 workspace 维度）· 单 banner 显示
  // 当前一条（多并发崩溃罕见 · 处理一条后下一条浮现 · 镜像 MVP-16 单 banner）。
  const activeRollbackRecovery = (): RollbackRecoveryUiState | null => {
    const entries = Object.values(rollbackRecoveries());
    return entries[0] ?? null;
  };

  const upsertRollbackRecovery = (
    sessionId: string,
    next: RollbackRecoveryUiState | null,
  ) => {
    setRollbackRecoveries((prev) =>
      reduceRollbackRecoveries(prev, sessionId, next),
    );
  };

  // 收到 `git:rollback-crash-recovery-detected`（启动 emit）· 委托纯状态机
  const ingestRollbackRecovery = (payload: RollbackRecoveryPayload) => {
    upsertRollbackRecovery(
      payload.sessionId,
      payloadToRollbackRecovery(payload),
    );
  };

  // Abort：始终安全（后端 cleanup_state + 反向 revert 已完成项 → 回 rollback
  // 前 HEAD）· 复用既有 rollback_abort IPC（同 SessionDetail abort 路径）。
  const handleRollbackRecoveryAbort = async () => {
    const current = activeRollbackRecovery();
    if (!current) return;
    setRollbackRecoveryBusy(true);
    setRollbackRecoveryError(null);
    try {
      await rollbackAbort(current.sessionId);
      upsertRollbackRecovery(current.sessionId, null);
    } catch (err) {
      setRollbackRecoveryError(
        err instanceof Error ? err.message : String(err),
      );
    } finally {
      setRollbackRecoveryBusy(false);
    }
  };

  // Resume：仅 conflict_paused 可用（canResume 守护 · 后端 resume 路径）。
  // 失败（如冲突未解决）如实暴露错误 · 引导用户去 Session 详情或中止
  // （不静默吞 · 不假装成功 · CLAUDE.md 错误处理）。
  const handleRollbackRecoveryResume = async () => {
    const current = activeRollbackRecovery();
    if (!current || !current.canResume) return;
    setRollbackRecoveryBusy(true);
    setRollbackRecoveryError(null);
    try {
      await rollbackExecute(current.sessionId, []);
      upsertRollbackRecovery(current.sessionId, null);
    } catch (err) {
      setRollbackRecoveryError(
        err instanceof Error ? err.message : String(err),
      );
    } finally {
      setRollbackRecoveryBusy(false);
    }
  };

  // Dismiss：本地清 banner（延后处理 · 用户可在 Session 详情继续）·
  // 不动 .git / DB 状态 · 下次启动若仍未完成会再次检测 emit。
  const handleRollbackRecoveryDismiss = () => {
    const current = activeRollbackRecovery();
    if (!current) return;
    upsertRollbackRecovery(current.sessionId, null);
  };

  const handleGlobalMergeResult = (status: MergeStatus) => {
    setGlobalMergeOpen(false);
    if (status.outcome === "conflict" && status.conflictingFiles.length > 0) {
      const ws = activeWorkspace();
      if (!ws) {
        return;
      }
      setConflict({
        workspaceId: ws.workspaceId,
        operation: "merge",
        files: status.conflictingFiles,
        resolvedFiles: [],
        currentStep: 1,
        totalSteps: status.conflictingFiles.length,
        currentCommit: null,
      });
    }
  };

  let unlistenMenu: UnlistenFn | undefined;
  let unlistenRebaseProgress: UnlistenFn | undefined;
  let unlistenConflictDetected: UnlistenFn | undefined;
  let unlistenRebaseDone: UnlistenFn | undefined;
  // MVP-16 Phase C · 启动检测 emit · per-workspace 一条 event
  let unlistenCrashRecovery: UnlistenFn | undefined;
  // MVP-20 Phase D · 启动检测 emit · per-session 一条 rollback recovery event
  let unlistenRollbackCrashRecovery: UnlistenFn | undefined;

  onMount(async () => {
    document.addEventListener("keydown", handleKeyDown);
    // Windows menu accelerator 前端 fallback · capture 阶段 · 仅 Windows 注册
    if (isWindows) {
      document.addEventListener("keydown", winMenuShortcutHandler, true);
    }
    unlistenMenu = await listen<{ action: string }>("menu:action", (event) => {
      switch (event.payload.action) {
        case "preferences":
          setSettingsVisible((v) => !v);
          break;
        case "merge":
          void openGlobalMergeDialog();
          break;
      }
    });
    unlistenRebaseProgress = await listen<RebaseProgressPayload>(
      "git:rebase-progress",
      (event) => {
        const ws = activeWorkspace();
        if (!ws || event.payload.workspaceId !== ws.workspaceId) {
          return;
        }
        setConflict((current) =>
          current
            ? {
                ...current,
                currentStep: event.payload.currentStep,
                totalSteps: event.payload.totalSteps,
                currentCommit: event.payload.currentCommit,
              }
            : current,
        );
      },
    );
    unlistenConflictDetected = await listen<ConflictDetectedPayload>(
      "git:conflict-detected",
      (event) => {
        const ws = activeWorkspace();
        if (!ws || event.payload.workspaceId !== ws.workspaceId) {
          return;
        }
        setConflictError(null);
        setConflict({
          workspaceId: event.payload.workspaceId,
          operation: normalizeConflictOperation(event.payload.operation),
          files: event.payload.conflictingFiles,
          resolvedFiles: [],
          currentStep: conflict()?.currentStep ?? 1,
          totalSteps:
            conflict()?.totalSteps ?? event.payload.conflictingFiles.length,
          currentCommit: null,
        });
      },
    );
    unlistenRebaseDone = await listen<RebaseOperationDonePayload>(
      "git:operation-done",
      (event) => {
        // MVP-16 Phase C · operation-done 不止跟当前 active workspace 有关 ·
        // 后台多 workspace 的 abort/continue 完成都会 emit · 故清 recovery 时按 payload.workspaceId
        // 路由（不 gate active workspace）· 而 active conflict banner 仍仅当前 workspace 有效。
        if (event.payload.success) {
          upsertRecovery(event.payload.workspaceId, null);
        }
        const ws = activeWorkspace();
        if (!ws || event.payload.workspaceId !== ws.workspaceId) {
          return;
        }
        if (event.payload.success) {
          setConflict(null);
          setConflictError(null);
          setRecoveryError(null);
        }
      },
    );
    // MVP-16 Phase C · 启动 emit `git:crash-recovery-detected` · 每 workspace 一条
    // backend `emit_rebase_crash_recovery` 在 `workspace_init` 调一次 · 之后切 workspace
    // 走 `rebase_detect_in_progress` IPC 主动查（见 createEffect below）。
    unlistenCrashRecovery = await listen<CrashRecoveryPayload>(
      "git:crash-recovery-detected",
      (event) => {
        ingestRecoveryPayload(event.payload);
      },
    );
    // MVP-20 Phase D · 启动 emit `git:rollback-crash-recovery-detected` ·
    // backend `emit_rollback_crash_recovery` 在 `workspace_init` 扫一次
    // （REVERT_HEAD + DB in_progress/conflict_paused）· 每条 session 一个 event。
    unlistenRollbackCrashRecovery = await listen<RollbackRecoveryPayload>(
      "git:rollback-crash-recovery-detected",
      (event) => {
        ingestRollbackRecovery(event.payload);
      },
    );
  });

  // MVP-16 Phase C · workspace 切换时主动查目标 workspace 是否有未完成操作
  // 触发：activeWorkspace() 变化（switch / open）· 不依赖启动 emit
  // 副作用幂等：如果状态没变 setRecoveries 不写新值（upsertRecovery 已处理）
  createEffect(() => {
    const ws = activeWorkspace();
    if (!ws) return;
    void refreshRecoveryForWorkspace(ws.workspaceId);
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyDown);
    if (isWindows) {
      document.removeEventListener("keydown", winMenuShortcutHandler, true);
    }
    unlistenMenu?.();
    unlistenRebaseProgress?.();
    unlistenConflictDetected?.();
    unlistenRebaseDone?.();
    unlistenCrashRecovery?.();
    unlistenRollbackCrashRecovery?.();
  });

  return (
    <div class="vs-shell">
      <TopBar
        activeWorkspace={activeWorkspace}
        primaryOpen={() => layout().primaryOpen}
        onTogglePrimary={() => dispatch({ kind: "toggle-primary" })}
        onMouseDown={handleTitleBarMouseDown}
      />

      <Show when={conflict()}>
        {(current) => (
          <ConflictBanner
            variant="active"
            operation={current().operation}
            source={current().currentCommit}
            target={activeWorkspace()?.repoRoot ?? activeWorkspace()?.name}
            currentStep={current().currentStep}
            totalSteps={current().totalSteps}
            filePath={currentConflictFile()}
            allResolved={allConflictFilesResolved()}
            allowSkip={current().operation !== "merge"}
            busy={conflictBusy()}
            onContinue={handleConflictContinue}
            onAbort={handleConflictAbort}
            onSkip={handleConflictSkip}
          />
        )}
      </Show>

      <Show when={conflictError()}>
        {(message) => (
          <div class="vs-conflict-error" role="alert">
            {message()}
          </div>
        )}
      </Show>

      {/* MVP-16 Phase C · crash recovery banner · 仅当 active conflict 不存在时显示
       * 避免双 banner 抢屏（active conflict 比 recovery 优先级高 · 因为它代表当前用户正在解决的冲突）
       * `allResolved=true` · `allowSkip=false`（recovery 不出 Skip 按钮 · 用户应先 Continue 让操作推进）
       * Continue 触发 `*_continue` IPC · 后续 conflict / done 由现有 event 链路接管
       */}
      <Show when={!conflict() && activeRecovery()}>
        {(current) => (
          <ConflictBanner
            variant="recovery"
            operation={current().operation}
            source={current().branch}
            target={activeWorkspace()?.repoRoot ?? activeWorkspace()?.name}
            currentStep={current().currentStep}
            totalSteps={current().totalSteps}
            allResolved={true}
            allowSkip={false}
            busy={recoveryBusy()}
            onContinue={handleRecoveryContinue}
            onAbort={handleRecoveryAbort}
            onViewStatus={handleRecoveryViewStatus}
          />
        )}
      </Show>

      <Show when={recoveryError()}>
        {(message) => (
          <div class="vs-conflict-error" role="alert">
            {message()}
          </div>
        )}
      </Show>

      {/* MVP-20 Phase D · rollback crash recovery banner（spec §H.9 · §I）
       * 与 MVP-16 recovery banner 独立（rollback session 维度 · rebase
       * workspace 维度 · 正交 · 罕见同时出现）· Abort 始终安全为主操作。
       */}
      <Show when={activeRollbackRecovery()}>
        {(current) => (
          <RollbackRecoveryBanner
            state={current()}
            busy={rollbackRecoveryBusy()}
            error={rollbackRecoveryError()}
            onAbort={handleRollbackRecoveryAbort}
            onResume={handleRollbackRecoveryResume}
            onDismiss={handleRollbackRecoveryDismiss}
          />
        )}
      </Show>

      <div class="vs-main-grid">
        <PrimarySidebar
          workspaces={props.workspaces}
          activeWorkspace={activeWorkspace}
          onOpen={props.onOpen}
          onCreate={props.onCreate}
          onDelete={props.onDeleteConfirm}
          onOpenImport={() => setImportVisible(true)}
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
          onOpenGitStatus={openGitStatusPanel}
        />

        <ActivityStrip />
      </div>

      <Show when={conflict()}>
        {(current) => (
          <div class="vs-conflict-diff-overlay">
            <ThreeWayDiffView
              workspaceId={current().workspaceId}
              initialFilePath={currentConflictFile()}
              onResolvedFile={markConflictFileResolved}
              onError={setConflictError}
            />
          </div>
        )}
      </Show>

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

      <footer
        class="vs-status-bar"
        aria-label={label("chrome.status.statusBar")}
      >
        <div class="vs-status-group">
          <IpcIndicator state={props.ipc()} />
          <RemoteSyncStatusItem onOpenGitLog={openGitLogHighlight} />
        </div>
        <div class="vs-status-group">
          <button
            type="button"
            class="vs-status-merge-btn"
            disabled={!activeWorkspace()?.hasGit}
            onClick={() => void openGlobalMergeDialog()}
          >
            {label("chrome.status.merge")}
          </button>
          <button
            type="button"
            class="vs-status-icon-btn"
            aria-label={label("chrome.status.openSettings")}
            title={`${label("chrome.status.settingsTitle")} (${formatShortcut("⌘,", "Ctrl+,")})`}
            onClick={() => setSettingsVisible(true)}
          >
            <GearIcon />
          </button>
          <ThemeSwitch />
          <span class="vs-status-item">
            <span class="vs-status-val">
              v{props.version()} · {label("chrome.status.alpha")}
            </span>
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
            aria-label={label("chrome.status.dismissError")}
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
        onOpenImport={() => setImportVisible(true)}
      />

      <Show when={importVisible()}>
        <ConfigImportDialog
          onClose={() => setImportVisible(false)}
          onApplied={() => {
            // settings store 已通过 settings_changed event 自动 reload
            // 此处仅做用户感知反馈 · 不做额外刷新
          }}
        />
      </Show>

      <BranchSwitcher
        activeWorkspace={activeWorkspace}
        open={branchSwitcherOpen}
        onClose={() => setBranchSwitcherOpen(false)}
      />

      <Show when={globalMergeOpen() && activeWorkspace()}>
        {(workspace) => (
          <MergeDialog
            workspaceId={workspace().workspaceId}
            currentBranch={globalMergeBranch()}
            onCancel={() => setGlobalMergeOpen(false)}
            onMerged={handleGlobalMergeResult}
            onOpenGitStatus={openGitStatusPanel}
            onError={setConflictError}
          />
        )}
      </Show>

      {/* MVP-17 Phase C · Pop to External Dialog 顶层渲染 · 由 popToExternalRequest signal 驱动 */}
      <Show when={popToExternalRequest()}>
        {(request) => (
          <PopToExternalDialog
            open={true}
            paneId={request().paneId}
            onClose={() => setPopToExternalRequest(null)}
          />
        )}
      </Show>
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
      // pool init 完成 · 主动重新拉 settings · 防 module-load 期 settings_get race
      // 拿到 default（telemetry_opt_in null）导致 modal 反复弹（session 19 实测）
      await reloadSettings();
      await refreshWorkspaces();
    } catch (err) {
      setIpc({
        kind: "error",
        message: `db init: ${err instanceof Error ? err.message : String(err)}`,
      });
    }

    // 全局 cmd/ctrl+C/V/A/X 处理 · 应用没 App Menu Edit submenu accelerator ·
    // macOS WKWebView 默认不响应 cmd+C 在 input/textarea/普通 selection 上 ·
    // 走 tauri-plugin-clipboard-manager IPC 调系统 NSPasteboard。
    // xterm focus 跳过 · 让 PaneTerminal attachCustomKeyEventHandler 自处理。
    document.addEventListener("keydown", handleClipboardKey);

    // MVP-17 Phase C wiring · 订阅 backend pane_detach_state_changed 事件
    // best-effort · 失败不阻塞 UI · listener 自动重连由 Tauri 处理
    initPaneDetachStateListener()
      .then((unlisten) => {
        onCleanup(() => unlisten());
      })
      .catch((err) => {
        console.warn("[mvp-17] initPaneDetachStateListener failed:", err);
      });
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleClipboardKey);
  });

  function handleClipboardKey(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (!mod || e.shiftKey || e.altKey) return;
    const key = e.key.toLowerCase();
    if (!["c", "v", "a", "x"].includes(key)) return;

    const target = e.target as Element | null;
    // xterm 内 · PaneTerminal attachCustomKeyEventHandler 自处理
    if (target?.closest?.(".xterm")) return;

    if (
      target instanceof HTMLInputElement ||
      target instanceof HTMLTextAreaElement
    ) {
      const start = target.selectionStart ?? 0;
      const end = target.selectionEnd ?? 0;
      const hasSel = start !== end;
      if (key === "c" && hasSel) {
        e.preventDefault();
        const text = target.value.substring(start, end);
        void writeClipboardText(text).catch((err) =>
          console.warn("[clipboard] copy failed", err),
        );
        return;
      }
      if (key === "x" && hasSel) {
        e.preventDefault();
        const text = target.value.substring(start, end);
        void writeClipboardText(text).catch((err) =>
          console.warn("[clipboard] cut failed", err),
        );
        target.value =
          target.value.substring(0, start) + target.value.substring(end);
        target.setSelectionRange(start, start);
        target.dispatchEvent(new Event("input", { bubbles: true }));
        return;
      }
      if (key === "v") {
        e.preventDefault();
        void readClipboardText()
          .then((text) => {
            if (!text) return;
            const s = target.selectionStart ?? 0;
            const eEnd = target.selectionEnd ?? 0;
            target.value =
              target.value.substring(0, s) +
              text +
              target.value.substring(eEnd);
            const caret = s + text.length;
            target.setSelectionRange(caret, caret);
            target.dispatchEvent(new Event("input", { bubbles: true }));
          })
          .catch((err) => console.warn("[clipboard] paste failed", err));
        return;
      }
      if (key === "a") {
        e.preventDefault();
        target.select();
        return;
      }
      return;
    }

    // 普通 selection · 仅 copy / select-all 适用（普通 div 不接 paste / cut）
    if (key === "c") {
      const sel = window.getSelection()?.toString() ?? "";
      if (sel) {
        e.preventDefault();
        void writeClipboardText(sel).catch((err) =>
          console.warn("[clipboard] copy failed", err),
        );
      }
    }
  }

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
      <LayoutProvider activeWorkspaceId={activeWorkspaceId} dbReady={dbReady}>
        <RemoteSyncStatusProvider
          activeWorkspace={() => {
            const view = currentView();
            return view.kind === "workspace" ? view.ws : null;
          }}
        >
          <PaneLinksProvider>
            <PaneDraftsProvider>
              <SessionsProvider>
                <BottomPanelTabsProvider>
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
                  <Show when={dbReady() && !telemetryDecided()}>
                    <TelemetryOptInModal />
                  </Show>
                  {/* MVP-19 §D · Phase D fills SessionDetailHost (renders on
                      activeDetail() from sessions-context · A.0.1 mounts host). */}
                  <SessionDetailHost />
                </BottomPanelTabsProvider>
              </SessionsProvider>
            </PaneDraftsProvider>
          </PaneLinksProvider>
        </RemoteSyncStatusProvider>
      </LayoutProvider>
    </ThemeProvider>
  );
};

export { App };
