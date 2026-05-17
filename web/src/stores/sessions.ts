import { createMemo, type Accessor } from "solid-js";
import { createStore } from "solid-js/store";
import type {
  AiSession,
  LinkState,
  SessionCommitBoundEvent,
  SessionCommitLink,
  SessionCommitUnboundEvent,
  SessionEndedEvent,
  SessionError,
  SessionErrorEvent,
  SessionLinkUpdatedEvent,
  SessionStartedEvent,
} from "../bindings";

/**
 * MVP-19 Phase C/D/E shared session store (= W2-A.0-role single source of truth
 * consumed in parallel by Phase C Git Log badge and Phase D detail view).
 *
 * #353 design-debt lesson applied: the canonical `SessionCommitLink` binding
 * already carries the full `linkState: LinkState` lifecycle
 * (pending | confirmedAuto | confirmedManual | unlinked | superseded | stale).
 * We model the store on `linkState` as the single source of truth and do NOT
 * introduce a lossy derived boolean — every downstream predicate
 * (`isLinkActive` / `isLinkConfirmed`) derives from `linkState`, so a soft-
 * unlinked / superseded / stale link stays distinguishable for the audit-facing
 * detail view (spec §H5/§E5.5/§D.2). The store is event-driven: the 6 backend
 * `session:*` events (W2-A.0 contract) are the only writers; components issue
 * commands through `sessionApi` and the store reacts to the resulting events.
 */
export type SessionCommitLinkView = SessionCommitLink;

/** §D.2 · a link contributes a primary Git Log badge iff active & primary. */
export function isLinkActive(link: SessionCommitLinkView): boolean {
  return (
    link.linkState !== "unlinked" &&
    link.linkState !== "superseded" &&
    link.linkState !== "stale"
  );
}

/** §E3.5/§D.2 · confirmed (auto or manual) drives the non-weakened badge style. */
export function isLinkConfirmed(link: SessionCommitLinkView): boolean {
  return (
    link.linkState === "confirmedAuto" || link.linkState === "confirmedManual"
  );
}

export interface SessionsStore {
  sessionsByWorkspace: Record<string, AiSession[]>;
  linksByWorkspace: Record<string, SessionCommitLinkView[]>;
  lastError: SessionError | null;
  applyStartedEvent: (event: SessionStartedEvent) => void;
  applyEndedEvent: (event: SessionEndedEvent) => void;
  applyCommitBoundEvent: (event: SessionCommitBoundEvent) => void;
  applyCommitUnboundEvent: (event: SessionCommitUnboundEvent) => void;
  applyLinkUpdatedEvent: (event: SessionLinkUpdatedEvent) => void;
  applyErrorEvent: (event: SessionErrorEvent) => void;
  clearError: () => void;
  createWorkspaceScopedSelectors: (
    workspaceId: Accessor<string | null | undefined>,
  ) => {
    sessions: Accessor<AiSession[]>;
    sessionById: (id: string) => AiSession | undefined;
    linksForCommit: (commitSha: string) => SessionCommitLinkView[];
    primaryLinkForCommit: (
      commitSha: string,
    ) => SessionCommitLinkView | undefined;
    linksForSession: (sessionId: string) => SessionCommitLinkView[];
  };
}

function upsertSession(prev: AiSession[], next: AiSession): AiSession[] {
  const i = prev.findIndex((s) => s.id === next.id);
  if (i < 0) return [...prev, next];
  const copy = [...prev];
  copy[i] = next;
  return copy;
}

function upsertLink(
  prev: SessionCommitLinkView[],
  next: SessionCommitLinkView,
): SessionCommitLinkView[] {
  const i = prev.findIndex((l) => l.id === next.id);
  if (i < 0) return [...prev, next];
  const copy = [...prev];
  copy[i] = next;
  return copy;
}

export function createSessionsStore(): SessionsStore {
  const [sessionsByWorkspace, setSessionsByWorkspace] = createStore<
    Record<string, AiSession[]>
  >({});
  const [linksByWorkspace, setLinksByWorkspace] = createStore<
    Record<string, SessionCommitLinkView[]>
  >({});
  const [lastError, setLastError] = createStore<{
    value: SessionError | null;
  }>({ value: null });

  const applyStartedEvent = (event: SessionStartedEvent) => {
    if (!event.workspaceId) return;
    setSessionsByWorkspace(event.workspaceId, (prev = []) =>
      upsertSession(prev, event.session),
    );
  };

  const applyEndedEvent = (event: SessionEndedEvent) => {
    if (!event.workspaceId) return;
    setSessionsByWorkspace(event.workspaceId, (prev = []) =>
      prev.map((s) =>
        s.id === event.sessionId
          ? {
              ...s,
              // end_reason "idle_cutoff" → IdleCutoff status; else Ended.
              status:
                event.endReason === "idle_cutoff" ? "idleCutoff" : "ended",
              endReason: event.endReason ?? s.endReason,
            }
          : s,
      ),
    );
  };

  const applyCommitBoundEvent = (event: SessionCommitBoundEvent) => {
    if (!event.workspaceId) return;
    setLinksByWorkspace(event.workspaceId, (prev = []) =>
      upsertLink(prev, event.link),
    );
  };

  const applyCommitUnboundEvent = (event: SessionCommitUnboundEvent) => {
    if (!event.workspaceId) return;
    // §H5 soft unbind: keep the row, flip lifecycle to `unlinked` so the
    // audit-facing detail view stays able to show the broken association
    // (never silent-drop). Event carries only id+sha; patch state only.
    setLinksByWorkspace(event.workspaceId, (prev = []) =>
      prev.map((l) =>
        l.id === event.linkId
          ? { ...l, linkState: "unlinked" as LinkState }
          : l,
      ),
    );
  };

  const applyLinkUpdatedEvent = (event: SessionLinkUpdatedEvent) => {
    if (!event.workspaceId) return;
    setLinksByWorkspace(event.workspaceId, (prev = []) =>
      upsertLink(prev, event.link),
    );
  };

  const applyErrorEvent = (event: SessionErrorEvent) => {
    setLastError("value", event.error);
  };

  const clearError = () => setLastError("value", null);

  const createWorkspaceScopedSelectors = (
    workspaceId: Accessor<string | null | undefined>,
  ) => {
    const sessions = createMemo<AiSession[]>(() => {
      const id = workspaceId();
      if (!id) return [];
      return sessionsByWorkspace[id] ?? [];
    });

    const links = createMemo<SessionCommitLinkView[]>(() => {
      const id = workspaceId();
      if (!id) return [];
      return linksByWorkspace[id] ?? [];
    });

    const sessionById = (id: string): AiSession | undefined =>
      sessions().find((s) => s.id === id);

    const linksForCommit = (commitSha: string): SessionCommitLinkView[] =>
      links().filter((l) => l.commitSha === commitSha);

    const primaryLinkForCommit = (
      commitSha: string,
    ): SessionCommitLinkView | undefined =>
      links().find(
        (l) => l.commitSha === commitSha && l.isPrimary && isLinkActive(l),
      );

    const linksForSession = (sessionId: string): SessionCommitLinkView[] =>
      links().filter((l) => l.sessionId === sessionId);

    return {
      sessions,
      sessionById,
      linksForCommit,
      primaryLinkForCommit,
      linksForSession,
    };
  };

  return {
    sessionsByWorkspace,
    linksByWorkspace,
    get lastError() {
      return lastError.value;
    },
    applyStartedEvent,
    applyEndedEvent,
    applyCommitBoundEvent,
    applyCommitUnboundEvent,
    applyLinkUpdatedEvent,
    applyErrorEvent,
    clearError,
    createWorkspaceScopedSelectors,
  };
}
