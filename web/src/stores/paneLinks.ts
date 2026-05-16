import { createMemo, type Accessor } from "solid-js";
import { createStore } from "solid-js/store";
import type {
  PaneBuildFailedEvent,
  PaneLink,
  PaneLinkedEvent,
} from "@/panels/Terminal/paneLinkContract";

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
  applyLinkedEvent: (event: PaneLinkedEvent) => void;
  applyBuildFailedEvent: (event: PaneBuildFailedEvent) => void;
  markChildStale: (workspaceId: string, childPaneId: string) => void;
  createWorkspaceScopedSelectors: (
    workspaceId: Accessor<string | null | undefined>,
  ) => {
    links: Accessor<PaneLink[]>;
    failureCallouts: Accessor<PaneFailureCallout[]>;
  };
}

export function createPaneLinksStore(): PaneLinksStore {
  const [linksByWorkspace, setLinksByWorkspace] = createStore<
    Record<string, PaneLink[]>
  >({});
  const [failureCalloutsByWorkspace, setFailureCalloutsByWorkspace] =
    createStore<Record<string, PaneFailureCallout[]>>({});

  const applyLinkedEvent = (event: PaneLinkedEvent) => {
    if (!event.workspaceId) {
      return;
    }

    setLinksByWorkspace(event.workspaceId, (prev = []) => {
      if (event.status === "removed") {
        return prev.filter((link) => link.linkId !== event.linkId);
      }

      const next = [...prev];
      const existingIndex = next.findIndex(
        (link) => link.linkId === event.linkId,
      );

      const updated: PaneLink = {
        workspaceId: event.workspaceId,
        linkId: event.linkId,
        parentPaneId: event.parentPaneId,
        childPaneId: event.childPaneId,
        linkKind: event.linkKind,
        status: event.status,
        updatedAt: event.updatedAt,
      };

      if (existingIndex >= 0) {
        next[existingIndex] = updated;
        return next;
      }

      return [...next, updated];
    });
  };

  const applyBuildFailedEvent = (event: PaneBuildFailedEvent) => {
    if (!event.workspaceId) {
      return;
    }

    const failureHash = buildFailureHash(event);
    const dedupeKey = `${event.commandRunId}:${failureHash}`;

    setFailureCalloutsByWorkspace(event.workspaceId, (prev = []) => {
      const hasDuplicate = prev.some(
        (callout) =>
          `${callout.commandRunId}:${callout.failureHash}` === dedupeKey,
      );
      if (hasDuplicate) {
        return prev;
      }

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
      if (next.length <= FAILURE_BACKLOG_CAP) {
        return next;
      }
      return next.slice(next.length - FAILURE_BACKLOG_CAP);
    });
  };

  const markChildStale = (workspaceId: string, childPaneId: string) => {
    if (!workspaceId || !childPaneId) {
      return;
    }

    setLinksByWorkspace(workspaceId, (prev = []) =>
      prev.map((link) =>
        link.childPaneId === childPaneId && link.status !== "removed"
          ? { ...link, status: "stale" }
          : link,
      ),
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

    return {
      links,
      failureCallouts,
    };
  };

  return {
    linksByWorkspace,
    failureCalloutsByWorkspace,
    applyLinkedEvent,
    applyBuildFailedEvent,
    markChildStale,
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
