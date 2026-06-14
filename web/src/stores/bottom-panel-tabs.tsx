/**
 * Bottom Panel tab 状态 + Output 日志 store · 纯前端（不持久化 · session 内存）。
 *
 * 独立于 `layout.ts`（LayoutState 是 ts-rs 自动生成 binding · 禁止手写扩展）。
 * 镜像 `remote-sync-status.tsx` / `paneLinks-context.tsx` 的 createContext idiom。
 *
 * **为何 Output 日志放这里（常驻 Provider）而非 OutputPanel 内**：
 * OutputPanel 被 `<Show when={activeTab()==="output"}>` 条件渲染 · tab 关闭时
 * unmount → listener 注销 → 错过事件 → 历史丢失。Provider 常驻挂载保证事件
 * 始终被订阅，OutputPanel 只负责渲染。
 */
import {
  createContext,
  useContext,
  createSignal,
  onCleanup,
  onMount,
  type Accessor,
  type ParentComponent,
} from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  FetchProgressEvent,
  NetworkOpError,
  OperationDoneEvent,
  PushProgressEvent,
} from "../bindings";

export type BottomPanelTab = "status" | "output";

type OutputEntryKind = "push" | "pull" | "fetch";
type OutputEntryOutcome = "running" | "success" | "error" | "cancelled";

export interface OutputEntry {
  id: string; // taskId
  kind: OutputEntryKind;
  workspaceId: string;
  outcome: OutputEntryOutcome;
  startedAt: number;
  endedAt: number | null;
  stage: string | null;
  objectsDone: number;
  objectsTotal: number;
  bytesDone: number;
  bytesTotal: number;
  error: string | null;
}

const MAX_ENTRIES = 50;
const GIT_PUSH_PROGRESS_EVENT = "git:push-progress";
const GIT_FETCH_PROGRESS_EVENT = "git:fetch-progress";
const GIT_OPERATION_DONE_EVENT = "git:operation-done";

function kindFromOperation(op: string): OutputEntryKind | null {
  if (op === "push") return "push";
  if (op === "pull") return "pull";
  if (op === "fetch") return "fetch";
  return null;
}

/** 把 NetworkOpError（11 变体判别联合）渲染成可读字符串 · 避免 [object Object] */
function formatNetworkError(e: NetworkOpError): string {
  switch (e.kind) {
    case "authFailed":
      return `auth failed: ${e.detail}`;
    case "networkUnreachable":
      return `network unreachable: ${e.detail}`;
    case "remoteNotFound":
      return `remote not found: ${e.remote}`;
    case "nonFastForward":
      return `non-fast-forward (${e.remoteBranch} · local ahead ${e.localAhead}, remote ahead ${e.remoteAhead})`;
    case "mergeConflict":
      return `merge conflict (${e.files.length} files · aborted=${e.aborted})`;
    case "aborted":
      return `aborted: ${e.reason}`;
    case "dirtyWorkingTree":
      return `dirty working tree (modified=${e.modified.length}, staged=${e.staged.length}, untracked=${e.untracked.length})`;
    case "rejectedByRemote":
      return `rejected by remote: ${e.detail}`;
    case "staleLease":
      return `stale lease (expected=${e.expected}, actual=${e.actual})`;
    case "sslError":
      return `ssl error: ${e.detail}`;
    case "git2Error":
      return `git2 error (class=${e.class}, code=${e.code}): ${e.message}`;
    default: {
      // 兜底 · 防止 NetworkOpError 新增变体时崩溃
      const exhaustive: never = e;
      return String(exhaustive);
    }
  }
}

interface BottomPanelTabsContextValue {
  activeTab: () => BottomPanelTab;
  setActiveTab: (tab: BottomPanelTab) => void;
  // Output log（常驻订阅 · tab 关闭也不丢事件）
  outputEntries: Accessor<OutputEntry[]>;
  clearOutputEntries: () => void;
}

const BottomPanelTabsContext = createContext<BottomPanelTabsContextValue>();

export const BottomPanelTabsProvider: ParentComponent = (props) => {
  const [activeTab, setActiveTab] = createSignal<BottomPanelTab>("status");
  const [entries, setEntries] = createSignal<OutputEntry[]>([]);

  // mounted 守卫 · 防止 listen Promise resolve 前组件 unmount 导致 unlisten 丢失
  // （Provider 常驻挂载正常不会触发，但 hot-reload / 测试场景下需要防御 · 对齐 App.tsx 现有 idiom）
  let mounted = true;
  let unlistenPush: UnlistenFn | undefined;
  let unlistenFetch: UnlistenFn | undefined;
  let unlistenDone: UnlistenFn | undefined;

  /** 在 FIFO 头部插入/更新 entry · 维持 MAX_ENTRIES 上限。 */
  const upsert = (
    taskId: string,
    fallbackKind: OutputEntryKind,
    mutator: (e: OutputEntry) => OutputEntry,
  ) => {
    setEntries((prev) => {
      const idx = prev.findIndex((e) => e.id === taskId);
      if (idx >= 0) {
        const next = [...prev];
        next[idx] = mutator(prev[idx]);
        return next;
      }
      // 新建 · 时间戳作为 startedAt · kind 用 fallbackKind（progress 事件已知操作类型）
      const fresh: OutputEntry = mutator({
        id: taskId,
        kind: fallbackKind,
        workspaceId: "",
        outcome: "running",
        startedAt: Date.now(),
        endedAt: null,
        stage: null,
        objectsDone: 0,
        objectsTotal: 0,
        bytesDone: 0,
        bytesTotal: 0,
        error: null,
      });
      return [fresh, ...prev].slice(0, MAX_ENTRIES);
    });
  };

  onMount(() => {
    void listen<PushProgressEvent>(GIT_PUSH_PROGRESS_EVENT, (event) => {
      const p = event.payload;
      upsert(p.taskId, "push", (e) => ({
        ...e,
        kind: "push",
        workspaceId: p.workspaceId,
        stage: p.stage,
        objectsDone: p.objectsDone,
        objectsTotal: p.objectsTotal,
        bytesDone: p.bytesDone,
        bytesTotal: p.bytesTotal,
      }));
    }).then((u) => {
      if (mounted) unlistenPush = u;
      else u();
    });

    void listen<FetchProgressEvent>(GIT_FETCH_PROGRESS_EVENT, (event) => {
      const p = event.payload;
      upsert(p.taskId, "fetch", (e) => ({
        ...e,
        kind: "fetch",
        workspaceId: p.workspaceId,
        stage: p.stage,
        objectsDone: p.receivedObjects,
        objectsTotal: p.totalObjects,
        bytesDone: p.receivedBytes,
        bytesTotal: e.bytesTotal,
      }));
    }).then((u) => {
      if (mounted) unlistenFetch = u;
      else u();
    });

    void listen<OperationDoneEvent>(GIT_OPERATION_DONE_EVENT, (event) => {
      const p = event.payload;
      const kind = kindFromOperation(p.operation);
      const outcome: OutputEntryOutcome =
        p.outcome === "success"
          ? "success"
          : p.outcome === "cancelled"
            ? "cancelled"
            : "error";
      upsert(p.taskId, kind ?? "fetch", (e) => ({
        ...e,
        kind: kind ?? e.kind,
        workspaceId: p.workspaceId || e.workspaceId,
        outcome,
        endedAt: Date.now(),
        stage: e.stage,
        error:
          outcome === "error" && p.error ? formatNetworkError(p.error) : null,
      }));
    }).then((u) => {
      if (mounted) unlistenDone = u;
      else u();
    });
  });

  onCleanup(() => {
    mounted = false;
    unlistenPush?.();
    unlistenFetch?.();
    unlistenDone?.();
  });

  const clearOutputEntries = () => setEntries([]);

  return (
    <BottomPanelTabsContext.Provider
      value={{
        activeTab,
        setActiveTab,
        outputEntries: entries,
        clearOutputEntries,
      }}
    >
      {props.children}
    </BottomPanelTabsContext.Provider>
  );
};

export function useBottomPanelTabs(): BottomPanelTabsContextValue {
  const ctx = useContext(BottomPanelTabsContext);
  if (!ctx) {
    throw new Error(
      "useBottomPanelTabs must be used within BottomPanelTabsProvider",
    );
  }
  return ctx;
}
