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
  let unlistenPush: UnlistenFn | undefined;
  let unlistenFetch: UnlistenFn | undefined;
  let unlistenDone: UnlistenFn | undefined;

  /** 在 FIFO 头部插入/更新 entry · 维持 MAX_ENTRIES 上限。 */
  const upsert = (taskId: string, mutator: (e: OutputEntry) => OutputEntry) => {
    setEntries((prev) => {
      const idx = prev.findIndex((e) => e.id === taskId);
      if (idx >= 0) {
        const next = [...prev];
        next[idx] = mutator(prev[idx]);
        return next;
      }
      // 新建 · 时间戳作为 startedAt · kind 占位（operation-done 会纠正）
      const fresh: OutputEntry = mutator({
        id: taskId,
        kind: "fetch",
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

  onMount(async () => {
    unlistenPush = await listen<PushProgressEvent>(
      GIT_PUSH_PROGRESS_EVENT,
      (event) => {
        const p = event.payload;
        upsert(p.taskId, (e) => ({
          ...e,
          kind: "push",
          workspaceId: p.workspaceId,
          stage: p.stage,
          objectsDone: p.objectsDone,
          objectsTotal: p.objectsTotal,
          bytesDone: p.bytesDone,
          bytesTotal: p.bytesTotal,
        }));
      },
    );

    unlistenFetch = await listen<FetchProgressEvent>(
      GIT_FETCH_PROGRESS_EVENT,
      (event) => {
        const p = event.payload;
        upsert(p.taskId, (e) => ({
          ...e,
          kind: "fetch",
          workspaceId: p.workspaceId,
          stage: p.stage,
          objectsDone: p.receivedObjects,
          objectsTotal: p.totalObjects,
          bytesDone: p.receivedBytes,
          bytesTotal: e.bytesTotal,
        }));
      },
    );

    unlistenDone = await listen<OperationDoneEvent>(
      GIT_OPERATION_DONE_EVENT,
      (event) => {
        const p = event.payload;
        const kind = kindFromOperation(p.operation);
        const outcome: OutputEntryOutcome =
          p.outcome === "success"
            ? "success"
            : p.outcome === "cancelled"
              ? "cancelled"
              : "error";
        upsert(p.taskId, (e) => ({
          ...e,
          kind: kind ?? e.kind,
          workspaceId: p.workspaceId || e.workspaceId,
          outcome,
          endedAt: Date.now(),
          stage: e.stage,
          error: outcome === "error" && p.error ? String(p.error) : null,
        }));
      },
    );
  });

  onCleanup(() => {
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
