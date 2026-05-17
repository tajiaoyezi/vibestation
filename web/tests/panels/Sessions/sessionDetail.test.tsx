import { beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@solidjs/testing-library";
import type {
  AiSession,
  SessionCommitLink,
  SessionDetailResult,
  SessionError,
} from "../../../src/bindings";
import type { SessionDetailTarget } from "../../../src/stores/sessions-context";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => vi.fn()),
}));

const state = vi.hoisted(() => {
  let _activeDetail: SessionDetailTarget | null = { sessionId: "sess-1" };
  let _lastError: SessionError | null = null;

  const stubSession: AiSession = {
    id: "sess-1",
    workspaceId: "ws-1",
    cliKind: "claude",
    source: "auto",
    title: "Test session",
    startedAt: 1760000000000,
    endedAt: null,
    endReason: null,
    promptCount: 5,
    tokenCount: null,
    eventCount: 3,
    status: "active",
    parserVersion: null,
    strategyVersion: null,
    metadataJson: "{}",
    createdAt: 1760000000000,
    updatedAt: 1760000000000,
  };

  const store = {
    sessionsByWorkspace: {
      "ws-1": [stubSession],
    } as Record<string, AiSession[]>,
    linksByWorkspace: {} as Record<string, SessionCommitLink[]>,
    get lastError() {
      return _lastError;
    },
    applyStartedEvent: vi.fn(),
    applyEndedEvent: vi.fn(),
    applyCommitBoundEvent: vi.fn(),
    applyCommitUnboundEvent: vi.fn(),
    applyLinkUpdatedEvent: vi.fn(),
    applyErrorEvent: vi.fn(),
    clearError: vi.fn(() => {
      _lastError = null;
    }),
    createWorkspaceScopedSelectors: vi.fn(),
  };

  const ctx = {
    store,
    selectorsFor: vi.fn(),
    start: vi.fn(),
    end: vi.fn(async () => ({ session: stubSession })),
    bindCommit: vi.fn(),
    unbind: vi.fn(async () => ({ linkId: "link-1", unlinked: true })),
    list: vi.fn(async () => ({ sessions: [stubSession] })),
    getDetail: vi.fn(),
    rebind: vi.fn(async () => ({
      supersededLinkId: "link-1",
      newLink: {} as SessionCommitLink,
    })),
    recalc: vi.fn(async () => ({ candidates: [] })),
    activeDetail: () => _activeDetail,
    openDetail: vi.fn(),
    closeDetail: vi.fn(() => {
      _activeDetail = null;
    }),
  };

  return {
    ctx,
    setActiveDetail(v: SessionDetailTarget | null) {
      _activeDetail = v;
    },
    setLastError(v: SessionError | null) {
      _lastError = v;
    },
  };
});

vi.mock("../../../src/stores/sessions-context", () => ({
  useSessions: () => state.ctx,
}));

vi.mock("../../../src/stores/layout-context", () => ({
  useLayout: () => ({ dispatch: vi.fn() }),
}));

vi.mock("../../../src/panels/Sessions/sessionDetail.css", () => ({}));

import { SessionDetailView } from "../../../src/panels/Sessions/SessionDetailView";
import { SessionUnbindModal } from "../../../src/panels/Sessions/SessionUnbindModal";
import { SessionDetailHost } from "../../../src/panels/Sessions/SessionDetailHost";

const mockCtx = state.ctx;

const stubSession: AiSession = {
  id: "sess-1",
  workspaceId: "ws-1",
  cliKind: "claude",
  source: "auto",
  title: "Test session",
  startedAt: 1760000000000,
  endedAt: null,
  endReason: null,
  promptCount: 5,
  tokenCount: null,
  eventCount: 3,
  status: "active",
  parserVersion: null,
  strategyVersion: null,
  metadataJson: "{}",
  createdAt: 1760000000000,
  updatedAt: 1760000000000,
};

const stubLink: SessionCommitLink = {
  id: "link-1",
  workspaceId: "ws-1",
  sessionId: "sess-1",
  commitSha: "abc12345deadbeef",
  isPrimary: true,
  linkState: "confirmedManual",
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

const stubPendingLink: SessionCommitLink = {
  ...stubLink,
  id: "link-2",
  commitSha: "def67890aabbccdd",
  linkState: "pending",
  confidence: 0.4,
  autoBound: true,
};

const stubUnlinkedLink: SessionCommitLink = {
  ...stubLink,
  id: "link-3",
  commitSha: "999aaa888bbb7777",
  linkState: "unlinked",
  confidence: 0.8,
};

const stubDetailResult: SessionDetailResult = {
  session: stubSession,
  links: [stubLink, stubPendingLink, stubUnlinkedLink],
  commitCount: 3,
  avgConfidence: 0.72,
};

describe("SessionDetailHost", () => {
  beforeEach(() => {
    cleanup();
    state.setActiveDetail({ sessionId: "sess-1" });
  });

  it("renders nothing when activeDetail is null", () => {
    state.setActiveDetail(null);
    const { container } = render(() => <SessionDetailHost />);
    expect(container.querySelector(".vs-session-detail")).toBeNull();
  });

  it("renders detail view when activeDetail is set", () => {
    vi.mocked(mockCtx.getDetail).mockResolvedValueOnce(stubDetailResult);
    const { container } = render(() => <SessionDetailHost />);
    expect(container.querySelector(".vs-session-detail")).not.toBeNull();
  });
});

describe("SessionDetailView · layout (§D.1)", () => {
  beforeEach(() => {
    cleanup();
    state.setActiveDetail({ sessionId: "sess-1" });
    vi.mocked(mockCtx.getDetail).mockResolvedValue(stubDetailResult);
    vi.mocked(mockCtx.end).mockClear();
    vi.mocked(mockCtx.unbind).mockClear();
    vi.mocked(mockCtx.rebind).mockClear();
    vi.mocked(mockCtx.recalc).mockClear();
    vi.mocked(mockCtx.closeDetail).mockClear();
    state.setLastError(null);
  });

  it("shows loading state initially", () => {
    vi.mocked(mockCtx.getDetail).mockReturnValue(new Promise(() => {}));
    const { getByText } = render(() => <SessionDetailView />);
    expect(getByText("Loading session…")).toBeInTheDocument();
  });

  it("renders session header with title, CLI tag, and status", async () => {
    const { findByText, getByLabelText } = render(() => <SessionDetailView />);
    expect(await findByText("Test session")).toBeInTheDocument();
    expect(await findByText("claude")).toBeInTheDocument();
    const badge = getByLabelText("Status: Active");
    expect(badge).toBeInTheDocument();
    expect(badge.dataset.status).toBe("active");
  });

  it("renders summary strip (commit count + avg confidence + prompts)", async () => {
    const { findByText } = render(() => <SessionDetailView />);
    expect(await findByText("3")).toBeInTheDocument();
    expect(await findByText("72.0%")).toBeInTheDocument();
    expect(await findByText("5")).toBeInTheDocument();
  });

  it("renders time range", async () => {
    const { findByText } = render(() => <SessionDetailView />);
    const started = await findByText(/Started:/);
    expect(started).toBeInTheDocument();
  });

  it("renders commit list with all links", async () => {
    const { findByText, findAllByRole } = render(() => <SessionDetailView />);
    expect(await findByText("Linked commits (3)")).toBeInTheDocument();
    const items = await findAllByRole("listitem");
    expect(items.length).toBe(3);
  });

  it("shows short SHA for each commit", async () => {
    const { findByText } = render(() => <SessionDetailView />);
    expect(await findByText("abc12345")).toBeInTheDocument();
    expect(await findByText("def67890")).toBeInTheDocument();
    expect(await findByText("999aaa88")).toBeInTheDocument();
  });

  it("shows link state badges with data-state attribute", async () => {
    const { container, findByText } = render(() => <SessionDetailView />);
    await findByText("abc12345");
    const states = container.querySelectorAll(".vs-session-commit-state");
    const dataStates = Array.from(states).map((s) =>
      s.getAttribute("data-state"),
    );
    expect(dataStates).toContain("confirmedManual");
    expect(dataStates).toContain("pending");
    expect(dataStates).toContain("unlinked");
  });

  it("shows pending link with warning indicator (⚠)", async () => {
    const { findByText } = render(() => <SessionDetailView />);
    const pending = await findByText(/pending/);
    expect(pending.textContent).toContain("⚠");
  });

  it("hides unbind/rebind buttons for unlinked rows (inactive)", async () => {
    const { container, findByText } = render(() => <SessionDetailView />);
    await findByText("999aaa88");
    const rows = container.querySelectorAll(".vs-session-commit-row");
    const unlinkedRow = Array.from(rows).find((r) =>
      r.textContent?.includes("999aaa88"),
    );
    expect(unlinkedRow).toBeDefined();
    const buttons = unlinkedRow!.querySelectorAll(
      ".vs-session-commit-action-btn",
    );
    expect(buttons.length).toBe(0);
  });

  it("shows unbind/rebind buttons for active links", async () => {
    const { findByLabelText } = render(() => <SessionDetailView />);
    expect(
      await findByLabelText("Unbind commit abc12345"),
    ).toBeInTheDocument();
    expect(
      await findByLabelText("Rebind commit abc12345"),
    ).toBeInTheDocument();
  });

  it("close button calls closeDetail", async () => {
    const { findByLabelText } = render(() => <SessionDetailView />);
    const closeBtn = await findByLabelText("Close session detail");
    fireEvent.click(closeBtn);
    expect(mockCtx.closeDetail).toHaveBeenCalled();
  });

  it("end session button calls ctx.end for active sessions", async () => {
    vi.mocked(mockCtx.end).mockResolvedValueOnce({ session: stubSession });
    const { findByLabelText } = render(() => <SessionDetailView />);
    const endBtn = await findByLabelText("End this session");
    fireEvent.click(endBtn);
    expect(mockCtx.end).toHaveBeenCalledWith(
      expect.objectContaining({
        workspaceId: "ws-1",
        sessionId: "sess-1",
        endReason: "manual_end",
      }),
    );
  });

  it("recalc all button calls ctx.recalc for unique SHAs", async () => {
    vi.mocked(mockCtx.recalc).mockResolvedValue({ candidates: [] });
    const { findByLabelText } = render(() => <SessionDetailView />);
    const btn = await findByLabelText("Recalculate all commit bindings");
    fireEvent.click(btn);
    await vi.waitFor(() => {
      expect(mockCtx.recalc).toHaveBeenCalledTimes(3);
    });
  });
});

describe("SessionDetailView · status variants", () => {
  beforeEach(() => {
    cleanup();
    state.setActiveDetail({ sessionId: "sess-1" });
    state.setLastError(null);
  });

  for (const status of [
    "active",
    "ended",
    "idleCutoff",
    "archived",
  ] as const) {
    it(`renders ${status} badge correctly`, async () => {
      const modifiedResult = {
        ...stubDetailResult,
        session: { ...stubSession, status },
      };
      vi.mocked(mockCtx.getDetail).mockResolvedValueOnce(modifiedResult);
      const { findByLabelText } = render(() => <SessionDetailView />);
      const expected =
        status === "idleCutoff"
          ? "Idle cutoff"
          : status.charAt(0).toUpperCase() + status.slice(1);
      const badge = await findByLabelText(`Status: ${expected}`);
      expect(badge.dataset.status).toBe(status);
    });
  }

  it("hides end-session button for ended sessions", async () => {
    const endedResult = {
      ...stubDetailResult,
      session: { ...stubSession, status: "ended" as const },
    };
    vi.mocked(mockCtx.getDetail).mockResolvedValueOnce(endedResult);
    const { findByText, queryByLabelText } = render(() => (
      <SessionDetailView />
    ));
    await findByText("Test session");
    expect(queryByLabelText("End this session")).toBeNull();
  });
});

describe("SessionDetailView · error bar (§D.2/D.4)", () => {
  beforeEach(() => {
    cleanup();
    state.setActiveDetail({ sessionId: "sess-1" });
    vi.mocked(mockCtx.getDetail).mockResolvedValue(stubDetailResult);
  });

  it("shows error bar when store.lastError is set", async () => {
    state.setLastError({ kind: "sessionNotFound", detail: "no such session" });
    const { findByRole } = render(() => <SessionDetailView />);
    const alert = await findByRole("alert");
    expect(alert.textContent).toContain("sessionNotFound");
    expect(alert.textContent).toContain("no such session");
  });

  it("shows error bar for crossWorkspaceDenied (no detail field)", async () => {
    state.setLastError({ kind: "crossWorkspaceDenied" });
    const { findByRole } = render(() => <SessionDetailView />);
    const alert = await findByRole("alert");
    expect(alert.textContent).toContain("crossWorkspaceDenied");
  });

  it("dismiss button clears the error", async () => {
    state.setLastError({ kind: "dbError", detail: "disk full" });
    const { findByLabelText } = render(() => <SessionDetailView />);
    const btn = await findByLabelText("Dismiss error");
    fireEvent.click(btn);
    expect(mockCtx.store.clearError).toHaveBeenCalled();
  });
});

describe("SessionDetailView · session not found", () => {
  beforeEach(() => {
    cleanup();
    state.setActiveDetail({ sessionId: "sess-1" });
    state.setLastError(null);
  });

  it("shows loading fallback when getDetail never resolves", () => {
    vi.mocked(mockCtx.getDetail).mockReturnValue(new Promise(() => {}));
    const { getByText } = render(() => <SessionDetailView />);
    expect(getByText("Loading session…")).toBeInTheDocument();
  });
});

describe("SessionUnbindModal (§D.3)", () => {
  beforeEach(cleanup);

  it("renders modal with commit sha and session title", () => {
    const { getByText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    expect(getByText(/abc12345/)).toBeInTheDocument();
    expect(getByText(/Test session/)).toBeInTheDocument();
  });

  it("renders strategy and confidence info", () => {
    const { getByText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    expect(getByText(/v1/)).toBeInTheDocument();
    expect(getByText(/95\.0%/)).toBeInTheDocument();
  });

  it("renders risk warning text", () => {
    const { getByText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    expect(getByText(/auditable/)).toBeInTheDocument();
  });

  it("has 3 action buttons: Cancel, Unbind, Unbind & recalc", () => {
    const { getByText, getByLabelText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    expect(getByText("Cancel")).toBeInTheDocument();
    expect(getByLabelText("Unbind commit")).toBeInTheDocument();
    expect(getByLabelText("Unbind and recalculate")).toBeInTheDocument();
  });

  it("Cancel button calls onCancel", () => {
    const onCancel = vi.fn();
    const { getByText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={onCancel}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    fireEvent.click(getByText("Cancel"));
    expect(onCancel).toHaveBeenCalled();
  });

  it("Unbind button calls onUnbind with reason", async () => {
    const onUnbind = vi.fn(async () => {});
    const { getByLabelText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={onUnbind}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    fireEvent.click(getByLabelText("Unbind commit"));
    await vi.waitFor(() => {
      expect(onUnbind).toHaveBeenCalledWith("manual correction");
    });
  });

  it("Unbind & recalc button calls onUnbindAndRecalc", async () => {
    const onRecalc = vi.fn(async () => {});
    const { getByLabelText } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={onRecalc}
      />
    ));
    fireEvent.click(getByLabelText("Unbind and recalculate"));
    await vi.waitFor(() => {
      expect(onRecalc).toHaveBeenCalledWith("manual correction");
    });
  });

  it("has aria-modal and role=dialog for a11y", () => {
    const { container } = render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={vi.fn()}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    const dialog = container.querySelector('[role="dialog"]');
    expect(dialog).not.toBeNull();
    expect(dialog?.getAttribute("aria-modal")).toBe("true");
  });

  it("Esc key calls onCancel (focus trap)", () => {
    const onCancel = vi.fn();
    render(() => (
      <SessionUnbindModal
        link={stubLink}
        sessionTitle="Test session"
        onCancel={onCancel}
        onUnbind={vi.fn()}
        onUnbindAndRecalc={vi.fn()}
      />
    ));
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onCancel).toHaveBeenCalled();
  });
});

describe("SessionDetailView · a11y (§D.4)", () => {
  beforeEach(() => {
    cleanup();
    state.setActiveDetail({ sessionId: "sess-1" });
    vi.mocked(mockCtx.getDetail).mockResolvedValue(stubDetailResult);
    state.setLastError(null);
  });

  it("detail view has role=region and aria-label", async () => {
    const { findByRole } = render(() => <SessionDetailView />);
    const region = await findByRole("region");
    expect(region.getAttribute("aria-label")).toBe("Session detail");
  });

  it("commit list has role=list and aria-label", async () => {
    const { findByRole } = render(() => <SessionDetailView />);
    const list = await findByRole("list");
    expect(list.getAttribute("aria-label")).toBe("Linked commits");
  });

  it("commit rows have aria-label with sha, state, confidence", async () => {
    const { findAllByRole } = render(() => <SessionDetailView />);
    const items = await findAllByRole("listitem");
    const firstLabel = items[0].getAttribute("aria-label") ?? "";
    expect(firstLabel).toContain("abc12345");
    expect(firstLabel).toContain("confirmedManual");
    expect(firstLabel).toContain("95%");
  });

  it("commit rows are keyboard focusable (tabindex=0)", async () => {
    const { findAllByRole } = render(() => <SessionDetailView />);
    const items = await findAllByRole("listitem");
    for (const item of items) {
      expect(item.getAttribute("tabindex")).toBe("0");
    }
  });

  it("all action buttons have aria-labels", async () => {
    const { findByLabelText } = render(() => <SessionDetailView />);
    expect(await findByLabelText("Close session detail")).toBeInTheDocument();
    expect(await findByLabelText("End this session")).toBeInTheDocument();
    expect(
      await findByLabelText("Recalculate all commit bindings"),
    ).toBeInTheDocument();
    expect(await findByLabelText("Unbind commit abc12345")).toBeInTheDocument();
    expect(await findByLabelText("Rebind commit abc12345")).toBeInTheDocument();
  });

  it("status badge has aria-label and role=status", async () => {
    const { findByRole } = render(() => <SessionDetailView />);
    const badge = await findByRole("status");
    expect(badge.getAttribute("aria-label")).toContain("Status:");
  });
});
