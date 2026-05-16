import { createMemo, type Accessor } from "solid-js";
import { createStore } from "solid-js/store";
import type {
  PaneLink,
  PaneLinkStatus,
  PaneLinkedEvent,
  PaneLinkError,
  PaneLinkErrorEvent,
} from "../bindings";

/**
 * Phase C placeholder · PaneBuildFailedEvent is not yet in bindings (Codex domain).
 * This local type mirrors spec §K.2 shape · will be replaced by real binding when
 * Phase C merges.
 */
export type PaneBuildFailedEvent = {
  workspaceId: string;
  linkId: string;
  parentPaneId: string;
  childPaneId: string;
  commandRunId: string;
  exitCode: number | null;
  rawExcerpt: string;
  parsedIssues: Array<{
    severity: "error" | "warning" | "info";
    file: string | null;
    line: number | null;
    column: number | null;
    message: string;
  }>;
  parserConfidence: number;
  fallbackMode: "structured" | "rawText";
  occurredAt: number;
};

export interface PaneFailureCallout {
  workspaceId: string;
  linkId: string;
  parentPaneId: string;
  childPaneId: string;
  commandRunId: string;
  failureHash: string;
  exitCode: number | null;
  rawExcerpt: string;
  parserConfidence: number;
  fallbackMode: PaneBuildFailedEvent["fallbackMode"];
  parsedIssuesCount: number;
  occurredAt: number;
}

export const FAILURE_BACKLOG_CAP = 5;

export interface PaneLinksStore {
  linksByWorkspace: Record<string, PaneLink[]>;
  failureCalloutsByWorkspace: Record<string, PaneFailureCallout[]>;
  lastError: PaneLinkError | null;
  applyLinkedEvent: (event: PaneLinkedEvent) => void;
  applyBuildFailedEvent: (event: PaneBuildFailedEvent) => void;
  applyErrorEvent: (event: PaneLinkErrorEvent) => void;
  clearError: () => void;
  markChildStale: (workspaceId: string, childPaneId: string) => void;
  dismissCallout: (workspaceId: string, commandRunId: string) => void;
  createWorkspaceScopedSelectors: (
    workspaceId: Accessor<string | null | undefined>,
  ) => {
    links: Accessor<PaneLink[]>;
    failureCallouts: Accessor<PaneFailureCallout[]>;
    activeLinks: Accessor<PaneLink[]>;
    linkForPane: (paneId: string) => PaneLink | undefined;
    parentLinkForChild: (childPaneId: string) => PaneLink | undefined;
  };
}

function statusToEnabled(status: PaneLinkStatus): boolean {
  return status === "enabled";
}

export function createPaneLinksStore(): PaneLinksStore {
  const [linksByWorkspace, setLinksByWorkspace] = createStore<
    Record<string, PaneLink[]>
  >({});
  const [failureCalloutsByWorkspace, setFailureCalloutsByWorkspace] =
    createStore<Record<string, PaneFailureCallout[]>>({});
  const [lastError, setLastError] = createStore<{
    value: PaneLinkError | null;
  }>({ value: null });

  const applyLinkedEvent = (event: PaneLinkedEvent) => {
    if (!event.workspaceId) return;

    setLinksByWorkspace(event.workspaceId, (prev = []) => {
      if (event.status === "removed") {
        return prev.filter((link) => link.id !== event.linkId);
      }

      const next = [...prev];
      const existingIndex = next.findIndex((link) => link.id === event.linkId);

      const updated: PaneLink = {
        id: event.linkId,
        workspaceId: event.workspaceId,
        parentPaneId: event.parentPaneId,
        childPaneId: event.childPaneId,
        linkKind: event.linkKind,
        enabled: statusToEnabled(event.status),
        fallbackMode: "structured",
        createdBy: "user",
        createdAt: event.updatedAt,
        updatedAt: event.updatedAt,
        lastTriggeredAt: 0,
      };

      if (existingIndex >= 0) {
        const existing = next[existingIndex];
        updated.createdAt = existing.createdAt;
        updated.createdBy = existing.createdBy;
        updated.fallbackMode = existing.fallbackMode;
        updated.lastTriggeredAt = existing.lastTriggeredAt;
        next[existingIndex] = updated;
        return next;
      }

      return [...next, updated];
    });
  };

  const applyBuildFailedEvent = (event: PaneBuildFailedEvent) => {
    if (!event.workspaceId) return;

    const failureHash = buildFailureHash(event);
    const dedupeKey = `${event.commandRunId}:${failureHash}`;

    setFailureCalloutsByWorkspace(event.workspaceId, (prev = []) => {
      const hasDuplicate = prev.some(
        (callout) =>
          `${callout.commandRunId}:${callout.failureHash}` === dedupeKey,
      );
      if (hasDuplicate) return prev;

      const nextCallout: PaneFailureCallout = {
        workspaceId: event.workspaceId,
        linkId: event.linkId,
        parentPaneId: event.parentPaneId,
        childPaneId: event.childPaneId,
        commandRunId: event.commandRunId,
        failureHash,
        exitCode: event.exitCode,
        rawExcerpt: event.rawExcerpt,
        parserConfidence: event.parserConfidence,
        fallbackMode: event.fallbackMode,
        parsedIssuesCount: event.parsedIssues.length,
        occurredAt: event.occurredAt,
      };

      const next = [...prev, nextCallout];
      if (next.length <= FAILURE_BACKLOG_CAP) return next;
      return next.slice(next.length - FAILURE_BACKLOG_CAP);
    });
  };

  const applyErrorEvent = (event: PaneLinkErrorEvent) => {
    setLastError("value", event.error);
  };

  const clearError = () => {
    setLastError("value", null);
  };

  const markChildStale = (workspaceId: string, childPaneId: string) => {
    if (!workspaceId || !childPaneId) return;

    setLinksByWorkspace(workspaceId, (prev = []) =>
      prev.map((link) =>
        link.childPaneId === childPaneId && link.enabled
          ? { ...link, enabled: false }
          : link,
      ),
    );
  };

  const dismissCallout = (workspaceId: string, commandRunId: string) => {
    if (!workspaceId || !commandRunId) return;

    setFailureCalloutsByWorkspace(workspaceId, (prev = []) =>
      prev.filter((callout) => callout.commandRunId !== commandRunId),
    );
  };

  const createWorkspaceScopedSelectors = (
    workspaceId: Accessor<string | null | undefined>,
  ) => {
    const links = createMemo<PaneLink[]>(() => {
      const id = workspaceId();
      if (!id) return [];
      return linksByWorkspace[id] ?? [];
    });

    const failureCallouts = createMemo<PaneFailureCallout[]>(() => {
      const id = workspaceId();
      if (!id) return [];
      return failureCalloutsByWorkspace[id] ?? [];
    });

    const activeLinks = createMemo<PaneLink[]>(() =>
      links().filter((link) => link.enabled),
    );

    const linkForPane = (paneId: string): PaneLink | undefined =>
      links().find(
        (link) =>
          (link.parentPaneId === paneId || link.childPaneId === paneId) &&
          link.enabled,
      );

    const parentLinkForChild = (childPaneId: string): PaneLink | undefined =>
      links().find((link) => link.childPaneId === childPaneId && link.enabled);

    return {
      links,
      failureCallouts,
      activeLinks,
      linkForPane,
      parentLinkForChild,
    };
  };

  return {
    linksByWorkspace,
    failureCalloutsByWorkspace,
    get lastError() {
      return lastError.value;
    },
    applyLinkedEvent,
    applyBuildFailedEvent,
    applyErrorEvent,
    clearError,
    markChildStale,
    dismissCallout,
    createWorkspaceScopedSelectors,
  };
}

export function buildFailureHash(event: PaneBuildFailedEvent): string {
  const issuesDigest = event.parsedIssues
    .map(
      (issue) =>
        `${issue.severity}:${issue.file ?? ""}:${issue.line ?? ""}:${issue.column ?? ""}:${issue.message}`,
    )
    .join("|");

  return `${event.childPaneId}::${event.exitCode ?? "none"}::${event.fallbackMode}::${event.rawExcerpt}::${issuesDigest}`;
}
