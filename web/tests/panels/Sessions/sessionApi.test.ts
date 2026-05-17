import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  SessionBindCommitRequest,
  SessionBindCommitResult,
  SessionCommitBoundEvent,
  SessionCommitUnboundEvent,
  SessionDetailRequest,
  SessionDetailResult,
  SessionEndRequest,
  SessionEndResult,
  SessionEndedEvent,
  SessionErrorEvent,
  SessionLinkUpdatedEvent,
  SessionListRequest,
  SessionListResult,
  SessionRebindRequest,
  SessionRebindResult,
  SessionRecalcRequest,
  SessionRecalcResult,
  SessionStartRequest,
  SessionStartResult,
  SessionStartedEvent,
  SessionUnbindRequest,
  SessionUnbindResult,
} from "../../../src/bindings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
} from "../../../src/panels/Sessions/sessionApi";

const stubSession = {
  id: "sess-1",
  workspaceId: "ws-1",
  cliKind: "claude",
  source: "auto",
  title: "test session",
  startedAt: 1760000000000,
  endedAt: null,
  endReason: null,
  promptCount: 0,
  tokenCount: null,
  eventCount: 0,
  status: "active" as const,
  parserVersion: null,
  strategyVersion: null,
  metadataJson: "{}",
  createdAt: 1760000000000,
  updatedAt: 1760000000000,
};

const stubLink = {
  id: "link-1",
  workspaceId: "ws-1",
  sessionId: "sess-1",
  commitSha: "abc123",
  isPrimary: true,
  linkState: "confirmedManual" as const,
  autoBound: false,
  confidence: 0.95,
  confidenceReason: "manual",
  strategyVersion: "v1",
  sourceEventId: null,
  linkedAt: 1760000000000,
  unlinkedAt: null,
  unlinkedReason: null,
  supersededByLinkId: null,
  createdBy: "user",
  reviewedBy: null,
  createdAt: 1760000000000,
  updatedAt: 1760000000000,
};

describe("sessionApi · invoke wrappers", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("wraps session_start", async () => {
    const result: SessionStartResult = {
      session: stubSession,
      alreadyActive: false,
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionStartRequest = {
      workspaceId: "ws-1",
      cliKind: "claude",
      source: "auto",
      paneId: null,
      title: null,
      startedAt: null,
    };

    await expect(sessionStart(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_start", { req });
  });

  it("wraps session_end", async () => {
    const result: SessionEndResult = { session: stubSession };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionEndRequest = {
      workspaceId: "ws-1",
      sessionId: "sess-1",
      endReason: "manual_end",
    };

    await expect(sessionEnd(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_end", { req });
  });

  it("wraps session_bind_commit", async () => {
    const result: SessionBindCommitResult = {
      link: stubLink,
      requiresConfirmation: false,
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionBindCommitRequest = {
      workspaceId: "ws-1",
      commitSha: "abc123",
      sessionId: "sess-1",
      mode: "manual",
      reason: null,
    };

    await expect(sessionBindCommit(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_bind_commit", { req });
  });

  it("wraps session_unbind", async () => {
    const result: SessionUnbindResult = { linkId: "link-1", unlinked: true };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionUnbindRequest = {
      workspaceId: "ws-1",
      linkId: "link-1",
      reason: null,
    };

    await expect(sessionUnbind(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_unbind", { req });
  });

  it("wraps session_list", async () => {
    const result: SessionListResult = { sessions: [stubSession] };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionListRequest = { workspaceId: "ws-1" };

    await expect(sessionList(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_list", { req });
  });

  it("wraps session_get_detail", async () => {
    const result: SessionDetailResult = {
      session: stubSession,
      links: [stubLink],
      commitCount: 1,
      avgConfidence: 0.95,
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionDetailRequest = {
      workspaceId: "ws-1",
      sessionId: "sess-1",
    };

    await expect(sessionGetDetail(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_get_detail", { req });
  });

  it("wraps session_rebind", async () => {
    const result: SessionRebindResult = {
      supersededLinkId: "link-1",
      newLink: stubLink,
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionRebindRequest = {
      workspaceId: "ws-1",
      linkId: "link-1",
      targetSessionId: "sess-2",
      reason: null,
    };

    await expect(sessionRebind(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_rebind", { req });
  });

  it("wraps session_recalc", async () => {
    const result: SessionRecalcResult = { candidates: [stubLink] };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: SessionRecalcRequest = {
      workspaceId: "ws-1",
      commitSha: "abc123",
    };

    await expect(sessionRecalc(req)).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("session_recalc", { req });
  });
});

describe("sessionApi · event listeners", () => {
  beforeEach(() => {
    vi.mocked(listen).mockReset();
  });

  it("onSessionStarted listens to session:started", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onSessionStarted(cb);

    expect(listen).toHaveBeenCalledWith("session:started", expect.any(Function));
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: SessionStartedEvent = {
      workspaceId: "ws-1",
      session: stubSession,
    };
    wrapper({ payload } as never);
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("onSessionEnded listens to session:ended", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onSessionEnded(cb);

    expect(listen).toHaveBeenCalledWith("session:ended", expect.any(Function));
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: SessionEndedEvent = {
      workspaceId: "ws-1",
      sessionId: "sess-1",
      endReason: "manual_end",
    };
    wrapper({ payload } as never);
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("onSessionCommitBound listens to session:commit-bound", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onSessionCommitBound(cb);

    expect(listen).toHaveBeenCalledWith(
      "session:commit-bound",
      expect.any(Function),
    );
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: SessionCommitBoundEvent = {
      workspaceId: "ws-1",
      link: stubLink,
    };
    wrapper({ payload } as never);
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("onSessionCommitUnbound listens to session:commit-unbound", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onSessionCommitUnbound(cb);

    expect(listen).toHaveBeenCalledWith(
      "session:commit-unbound",
      expect.any(Function),
    );
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: SessionCommitUnboundEvent = {
      workspaceId: "ws-1",
      linkId: "link-1",
      commitSha: "abc123",
    };
    wrapper({ payload } as never);
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("onSessionLinkUpdated listens to session:link-updated", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onSessionLinkUpdated(cb);

    expect(listen).toHaveBeenCalledWith(
      "session:link-updated",
      expect.any(Function),
    );
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: SessionLinkUpdatedEvent = {
      workspaceId: "ws-1",
      link: stubLink,
    };
    wrapper({ payload } as never);
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("onSessionError listens to session:error", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onSessionError(cb);

    expect(listen).toHaveBeenCalledWith("session:error", expect.any(Function));
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: SessionErrorEvent = {
      workspaceId: "ws-1",
      error: { kind: "sessionNotFound", detail: "no such session" },
    };
    wrapper({ payload } as never);
    expect(cb).toHaveBeenCalledWith(payload);
  });
});
