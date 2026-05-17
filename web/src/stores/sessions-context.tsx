import {
  createContext,
  createSignal,
  onCleanup,
  onMount,
  useContext,
  type Accessor,
  type ParentComponent,
} from "solid-js";
import { type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SessionBindCommitRequest,
  SessionBindCommitResult,
  SessionDetailRequest,
  SessionDetailResult,
  SessionEndRequest,
  SessionEndResult,
  SessionListRequest,
  SessionListResult,
  SessionRebindRequest,
  SessionRebindResult,
  SessionRecalcRequest,
  SessionRecalcResult,
  SessionStartRequest,
  SessionStartResult,
  SessionUnbindRequest,
  SessionUnbindResult,
} from "../bindings";
import {
  onSessionCommitBound,
  onSessionCommitUnbound,
  onSessionEnded,
  onSessionError,
  onSessionLinkUpdated,
  onSessionStarted,
  sessionBindCommit,
  sessionEnd,
  sessionGetDetail,
  sessionList,
  sessionRebind,
  sessionRecalc,
  sessionStart,
  sessionUnbind,
} from "../panels/Sessions/sessionApi";
import { createSessionsStore, type SessionsStore } from "./sessions";

/**
 * §D.2 click → §D.1 detail navigation seam — the single shared contract between
 * Phase C (Git Log badge sets it) and Phase D (detail view renders from it).
 * Kept here (not in the data store) so both phases consume one navigation API
 * instead of inventing divergent routing (§2.16 shared-shape discipline).
 */
export interface SessionDetailTarget {
  sessionId: string;
  /** Optional commit sha to scroll/anchor to inside the detail view (§D.1). */
  commitAnchor?: string;
}

export type SessionsContextValue = {
  store: SessionsStore;
  selectorsFor: (
    workspaceId: Accessor<string | null | undefined>,
  ) => ReturnType<SessionsStore["createWorkspaceScopedSelectors"]>;
  // sessionApi command wrappers (thin pass-through · mirror paneLinks-context)
  start: (req: SessionStartRequest) => Promise<SessionStartResult>;
  end: (req: SessionEndRequest) => Promise<SessionEndResult>;
  bindCommit: (
    req: SessionBindCommitRequest,
  ) => Promise<SessionBindCommitResult>;
  unbind: (req: SessionUnbindRequest) => Promise<SessionUnbindResult>;
  list: (req: SessionListRequest) => Promise<SessionListResult>;
  getDetail: (req: SessionDetailRequest) => Promise<SessionDetailResult>;
  rebind: (req: SessionRebindRequest) => Promise<SessionRebindResult>;
  recalc: (req: SessionRecalcRequest) => Promise<SessionRecalcResult>;
  // §D detail navigation seam (shared C→D contract)
  activeDetail: Accessor<SessionDetailTarget | null>;
  openDetail: (sessionId: string, commitAnchor?: string) => void;
  closeDetail: () => void;
};

const SessionsContext = createContext<SessionsContextValue>();

export const SessionsProvider: ParentComponent = (props) => {
  const store = createSessionsStore();
  const [activeDetail, setActiveDetail] =
    createSignal<SessionDetailTarget | null>(null);
  let unlisteners: UnlistenFn[] = [];
  let disposed = false;

  onMount(() => {
    void Promise.all([
      onSessionStarted((e) => store.applyStartedEvent(e)),
      onSessionEnded((e) => store.applyEndedEvent(e)),
      onSessionCommitBound((e) => store.applyCommitBoundEvent(e)),
      onSessionCommitUnbound((e) => store.applyCommitUnboundEvent(e)),
      onSessionLinkUpdated((e) => store.applyLinkUpdatedEvent(e)),
      onSessionError((e) => store.applyErrorEvent(e)),
    ])
      .then((nextUnlisteners) => {
        if (disposed) {
          nextUnlisteners.forEach((unlisten) => unlisten());
          return;
        }
        unlisteners = nextUnlisteners;
      })
      .catch((error) => {
        console.warn("[mvp-19] session event subscription failed:", error);
      });
  });

  onCleanup(() => {
    disposed = true;
    unlisteners.forEach((unlisten) => unlisten());
    unlisteners = [];
  });

  const value: SessionsContextValue = {
    store,
    selectorsFor: (workspaceId) =>
      store.createWorkspaceScopedSelectors(workspaceId),
    start: sessionStart,
    end: sessionEnd,
    bindCommit: sessionBindCommit,
    unbind: sessionUnbind,
    list: sessionList,
    getDetail: sessionGetDetail,
    rebind: sessionRebind,
    recalc: sessionRecalc,
    activeDetail,
    openDetail: (sessionId, commitAnchor) =>
      setActiveDetail({ sessionId, commitAnchor }),
    closeDetail: () => setActiveDetail(null),
  };

  return (
    <SessionsContext.Provider value={value}>
      {props.children}
    </SessionsContext.Provider>
  );
};

export function useSessions(): SessionsContextValue {
  const ctx = useContext(SessionsContext);
  if (!ctx) {
    throw new Error("useSessions must be used within SessionsProvider");
  }
  return ctx;
}
