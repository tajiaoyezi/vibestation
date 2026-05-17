import { describe, expect, it } from "vitest";
import { createRoot } from "solid-js";
import type {
  AiSession,
  LinkState,
  SessionCommitLink,
} from "../../src/bindings";
import {
  createSessionsStore,
  isLinkActive,
  isLinkConfirmed,
} from "../../src/stores/sessions";

function session(id: string, workspaceId: string): AiSession {
  return {
    id,
    workspaceId,
    cliKind: "claude",
    source: "auto",
    title: "fix bug",
    startedAt: 1_700_000_000_000,
    endedAt: null,
    endReason: null,
    promptCount: 2,
    tokenCount: 1500,
    eventCount: 7,
    status: "active",
    parserVersion: null,
    strategyVersion: null,
    metadataJson: "{}",
    createdAt: 1_700_000_000_000,
    updatedAt: 1_700_000_000_000,
  };
}

function link(
  id: string,
  workspaceId: string,
  sessionId: string,
  commitSha: string,
  linkState: LinkState = "pending",
  isPrimary = true,
): SessionCommitLink {
  return {
    id,
    workspaceId,
    sessionId,
    commitSha,
    isPrimary,
    linkState,
    autoBound: true,
    confidence: 0.5,
    confidenceReason: "time-window",
    strategyVersion: "v1",
    sourceEventId: null,
    linkedAt: 1_700_000_000_000,
    unlinkedAt: null,
    unlinkedReason: null,
    supersededByLinkId: null,
    createdBy: "system",
    reviewedBy: null,
    createdAt: 1_700_000_000_000,
    updatedAt: 1_700_000_000_000,
  };
}

describe("createSessionsStore · event application", () => {
  it("applyStartedEvent upserts a session (insert then update by id)", () => {
    const store = createSessionsStore();
    store.applyStartedEvent({
      workspaceId: "w1",
      session: session("s1", "w1"),
    });
    expect(store.sessionsByWorkspace["w1"]).toHaveLength(1);
    store.applyStartedEvent({
      workspaceId: "w1",
      session: { ...session("s1", "w1"), title: "renamed" },
    });
    expect(store.sessionsByWorkspace["w1"]).toHaveLength(1);
    expect(store.sessionsByWorkspace["w1"][0].title).toBe("renamed");
  });

  it("applyEndedEvent flips status (idle_cutoff → idleCutoff, else ended) + endReason", () => {
    const store = createSessionsStore();
    store.applyStartedEvent({
      workspaceId: "w1",
      session: session("s1", "w1"),
    });
    store.applyStartedEvent({
      workspaceId: "w1",
      session: session("s2", "w1"),
    });
    store.applyEndedEvent({
      workspaceId: "w1",
      sessionId: "s1",
      endReason: "idle_cutoff",
    });
    store.applyEndedEvent({
      workspaceId: "w1",
      sessionId: "s2",
      endReason: "manual_end",
    });
    const byId = (id: string) =>
      store.sessionsByWorkspace["w1"].find((s) => s.id === id)!;
    expect(byId("s1").status).toBe("idleCutoff");
    expect(byId("s1").endReason).toBe("idle_cutoff");
    expect(byId("s2").status).toBe("ended");
  });

  it("applyCommitBoundEvent adds a link; applyLinkUpdatedEvent preserves full LinkState (#353)", () => {
    const store = createSessionsStore();
    store.applyCommitBoundEvent({
      workspaceId: "w1",
      link: link("l1", "w1", "s1", "sha-a", "pending"),
    });
    expect(store.linksByWorkspace["w1"]).toHaveLength(1);
    expect(store.linksByWorkspace["w1"][0].linkState).toBe("pending");
    // pending → confirmedManual: full lifecycle preserved, NOT collapsed to bool
    store.applyLinkUpdatedEvent({
      workspaceId: "w1",
      link: link("l1", "w1", "s1", "sha-a", "confirmedManual"),
    });
    expect(store.linksByWorkspace["w1"]).toHaveLength(1);
    expect(store.linksByWorkspace["w1"][0].linkState).toBe("confirmedManual");
  });

  it("applyCommitUnboundEvent soft-unlinks (row kept, linkState→unlinked · §H5 audit)", () => {
    const store = createSessionsStore();
    store.applyCommitBoundEvent({
      workspaceId: "w1",
      link: link("l1", "w1", "s1", "sha-a", "confirmedAuto"),
    });
    store.applyCommitUnboundEvent({
      workspaceId: "w1",
      linkId: "l1",
      commitSha: "sha-a",
    });
    // row preserved (not silent-dropped), lifecycle flipped to unlinked
    expect(store.linksByWorkspace["w1"]).toHaveLength(1);
    expect(store.linksByWorkspace["w1"][0].linkState).toBe("unlinked");
  });

  it("applyErrorEvent / clearError", () => {
    const store = createSessionsStore();
    store.applyErrorEvent({
      workspaceId: "w1",
      error: { kind: "linkNotFound", detail: "l9" } as never,
    });
    expect(store.lastError).not.toBeNull();
    store.clearError();
    expect(store.lastError).toBeNull();
  });

  it("ignores events with empty workspaceId", () => {
    const store = createSessionsStore();
    store.applyStartedEvent({ workspaceId: "", session: session("s1", "") });
    expect(store.sessionsByWorkspace[""]).toBeUndefined();
  });
});

describe("createSessionsStore · selectors (workspace-scoped)", () => {
  it("sessions / sessionById / linksForCommit / primaryLinkForCommit / linksForSession", () => {
    createRoot((dispose) => {
      const store = createSessionsStore();
      store.applyStartedEvent({
        workspaceId: "w1",
        session: session("s1", "w1"),
      });
      store.applyStartedEvent({
        workspaceId: "w2",
        session: session("s9", "w2"),
      });
      store.applyCommitBoundEvent({
        workspaceId: "w1",
        link: link("l1", "w1", "s1", "sha-a", "confirmedAuto", true),
      });
      store.applyCommitBoundEvent({
        workspaceId: "w1",
        link: link("l2", "w1", "s1", "sha-a", "pending", false),
      });

      const sel = store.createWorkspaceScopedSelectors(() => "w1");
      expect(sel.sessions()).toHaveLength(1);
      expect(sel.sessionById("s1")?.id).toBe("s1");
      expect(sel.sessionById("s9")).toBeUndefined(); // workspace-scoped
      expect(sel.linksForCommit("sha-a")).toHaveLength(2);
      expect(sel.primaryLinkForCommit("sha-a")?.id).toBe("l1"); // primary+active
      expect(sel.linksForSession("s1")).toHaveLength(2);

      const empty = store.createWorkspaceScopedSelectors(() => null);
      expect(empty.sessions()).toHaveLength(0);
      dispose();
    });
  });

  it("primaryLinkForCommit ignores unlinked/superseded/stale primary links", () => {
    createRoot((dispose) => {
      const store = createSessionsStore();
      store.applyCommitBoundEvent({
        workspaceId: "w1",
        link: link("l1", "w1", "s1", "sha-a", "unlinked", true),
      });
      const sel = store.createWorkspaceScopedSelectors(() => "w1");
      expect(sel.primaryLinkForCommit("sha-a")).toBeUndefined();
      dispose();
    });
  });
});

describe("link lifecycle predicates (#353 · derive from linkState SSoT)", () => {
  it("isLinkActive true only for pending/confirmed*; false for unlinked/superseded/stale", () => {
    const mk = (st: LinkState) => link("l", "w", "s", "c", st);
    expect(isLinkActive(mk("pending"))).toBe(true);
    expect(isLinkActive(mk("confirmedAuto"))).toBe(true);
    expect(isLinkActive(mk("confirmedManual"))).toBe(true);
    expect(isLinkActive(mk("unlinked"))).toBe(false);
    expect(isLinkActive(mk("superseded"))).toBe(false);
    expect(isLinkActive(mk("stale"))).toBe(false);
  });

  it("isLinkConfirmed true only for confirmedAuto/confirmedManual", () => {
    const mk = (st: LinkState) => link("l", "w", "s", "c", st);
    expect(isLinkConfirmed(mk("confirmedAuto"))).toBe(true);
    expect(isLinkConfirmed(mk("confirmedManual"))).toBe(true);
    expect(isLinkConfirmed(mk("pending"))).toBe(false);
    expect(isLinkConfirmed(mk("stale"))).toBe(false);
  });
});
