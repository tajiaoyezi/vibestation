// MVP-19 Phase C · SessionBadge unit tests
//
// TDD: tests written before implementation. Covers §D.2 (a)-(g), HC-4 a11y,
// and click/keyboard navigation to A.0 openDetail.

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render } from "@solidjs/testing-library";
import type {
  AiSession,
  LinkState,
  SessionCommitLink,
} from "../../../src/bindings";

// ── Hoist mocks before any module imports ──────────────────────────────────

const mockOpenDetail = vi.hoisted(() => vi.fn());
const mockSelectorsFor = vi.hoisted(() => vi.fn());

vi.mock("../../../src/stores/sessions-context", () => ({
  useSessions: () => ({
    selectorsFor: mockSelectorsFor,
    openDetail: mockOpenDetail,
    // other SessionsContextValue fields as no-ops (not needed by SessionBadge)
    store: {},
    start: vi.fn(),
    end: vi.fn(),
    bindCommit: vi.fn(),
    unbind: vi.fn(),
    list: vi.fn(),
    getDetail: vi.fn(),
    rebind: vi.fn(),
    recalc: vi.fn(),
    activeDetail: () => null,
    closeDetail: vi.fn(),
  }),
}));

// Import component after mock setup
import { SessionBadge } from "../../../src/panels/GitLog/SessionBadge";

// ── Test fixtures ─────────────────────────────────────────────────────────────

function mkSession(id: string, title: string): AiSession {
  return {
    id,
    workspaceId: "ws-1",
    cliKind: "claude",
    source: "auto",
    title,
    startedAt: 1_700_000_000_000,
    endedAt: null,
    endReason: null,
    promptCount: 2,
    tokenCount: null,
    eventCount: 5,
    status: "active",
    parserVersion: null,
    strategyVersion: null,
    metadataJson: "{}",
    createdAt: 1_700_000_000_000,
    updatedAt: 1_700_000_000_000,
  };
}

function mkLink(
  id: string,
  sessionId: string,
  commitSha: string,
  linkState: LinkState = "confirmedManual",
  isPrimary = true,
): SessionCommitLink {
  return {
    id,
    workspaceId: "ws-1",
    sessionId,
    commitSha,
    isPrimary,
    linkState,
    autoBound: true,
    confidence: 0.8,
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

function setupSelectors(opts: {
  primaryLink?: SessionCommitLink;
  allLinks?: SessionCommitLink[];
  session?: AiSession;
}) {
  const { primaryLink, session } = opts;
  const allLinks = opts.allLinks ?? (primaryLink ? [primaryLink] : []);

  mockSelectorsFor.mockReturnValue({
    sessions: () => [],
    sessionById: (_id: string) => session,
    linksForCommit: (_sha: string) => allLinks,
    primaryLinkForCommit: (_sha: string) => primaryLink,
    linksForSession: (_sid: string) => [],
  });
}

afterEach(() => {
  cleanup();
  mockOpenDetail.mockReset();
  mockSelectorsFor.mockReset();
});

// ── §D.2 (g): no link → no badge ──────────────────────────────────────────────

describe("§D.2 (g): no primary link → no badge", () => {
  it("renders nothing when no primary link exists for the commit", () => {
    setupSelectors({});
    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));
    expect(container.querySelector(".vs-session-badge")).toBeNull();
  });
});

// ── §D.2 (a)/(c): confirmed badge ────────────────────────────────────────────

describe("§D.2 (a)/(c): confirmed badge", () => {
  it("renders confirmed style for confirmedManual link with session title", () => {
    const sess = mkSession("s1", "fix auth bug");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    expect(badge).not.toBeNull();
    expect(badge!.classList.contains("vs-session-badge--confirmed")).toBe(true);
    expect(badge!.textContent).toContain("fix auth bug");
  });

  it("renders confirmed style for confirmedAuto link", () => {
    const sess = mkSession("s1", "refactor db");
    const link = mkLink("l1", "s1", "abc123", "confirmedAuto");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    expect(badge!.classList.contains("vs-session-badge--confirmed")).toBe(true);
  });
});

// ── §D.2 (c): pending / weakened style + icon ────────────────────────────────

describe("§D.2 (c): pending badge", () => {
  it("renders pending style and icon when link linkState is pending", () => {
    const sess = mkSession("s1", "work in progress");
    const link = mkLink("l1", "s1", "abc123", "pending");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    expect(badge!.classList.contains("vs-session-badge--pending")).toBe(true);
    // HC-4: non-colour state indicator must be present for pending
    const icon = container.querySelector(".vs-session-badge__icon");
    expect(icon).not.toBeNull();
  });
});

// ── §D.2 (f): stale style ────────────────────────────────────────────────────

describe("§D.2 (f): stale badge (session not found)", () => {
  it("renders stale style and icon when sessionById returns undefined", () => {
    const link = mkLink("l1", "s-missing", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: undefined });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    expect(badge!.classList.contains("vs-session-badge--stale")).toBe(true);
    // HC-4: non-colour indicator for stale
    const icon = container.querySelector(".vs-session-badge__icon");
    expect(icon).not.toBeNull();
  });
});

// ── §D.2 (b): +N secondary marker ────────────────────────────────────────────

describe("§D.2 (b): +N secondary marker", () => {
  it("shows +2 when there are three links (1 primary + 2 others)", () => {
    const sess = mkSession("s1", "my session");
    const primary = mkLink("l1", "s1", "abc123", "confirmedManual", true);
    const sec1 = mkLink("l2", "s2", "abc123", "pending", false);
    const sec2 = mkLink("l3", "s3", "abc123", "pending", false);
    setupSelectors({
      primaryLink: primary,
      allLinks: [primary, sec1, sec2],
      session: sess,
    });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const secondary = container.querySelector(".vs-session-badge__secondary");
    expect(secondary).not.toBeNull();
    expect(secondary!.textContent).toContain("+2");
  });

  it("shows +1 when there are two links (1 primary + 1 other)", () => {
    const sess = mkSession("s1", "my session");
    const primary = mkLink("l1", "s1", "abc123", "confirmedManual", true);
    const sec = mkLink("l2", "s2", "abc123", "pending", false);
    setupSelectors({
      primaryLink: primary,
      allLinks: [primary, sec],
      session: sess,
    });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const secondary = container.querySelector(".vs-session-badge__secondary");
    expect(secondary!.textContent).toContain("+1");
  });

  it("does not show secondary marker for a single link", () => {
    const sess = mkSession("s1", "solo session");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const secondary = container.querySelector(".vs-session-badge__secondary");
    expect(secondary).toBeNull();
  });
});

// ── §D.2 (d): hover tooltip ──────────────────────────────────────────────────

describe("§D.2 (d): tooltip content", () => {
  it("title attribute contains session title, confidence, and source", () => {
    const sess = mkSession("s1", "refactor auth");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    const title = badge!.getAttribute("title");
    expect(title).toContain("refactor auth");
    expect(title).toContain("80%"); // confidence 0.8 → 80%
    expect(title).toContain("time-window"); // confidenceReason
  });
});

// ── §D.2 (e): click → openDetail ─────────────────────────────────────────────

describe("§D.2 (e): click → openDetail", () => {
  it("calls openDetail(sessionId, commitSha) on badge click", () => {
    const sess = mkSession("s1", "my session");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    fireEvent.click(container.querySelector(".vs-session-badge")!);

    expect(mockOpenDetail).toHaveBeenCalledOnce();
    expect(mockOpenDetail).toHaveBeenCalledWith("s1", "abc123");
  });
});

// ── HC-4: a11y ────────────────────────────────────────────────────────────────

describe("HC-4: a11y", () => {
  it("badge has aria-label with session title, status, and confidence", () => {
    const sess = mkSession("s1", "debug session");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    const ariaLabel = badge!.getAttribute("aria-label");
    expect(ariaLabel).toContain("debug session");
    expect(ariaLabel).toContain("confirmed");
    expect(ariaLabel).toContain("80%");
  });

  it("keyboard Enter triggers openDetail", () => {
    const sess = mkSession("s1", "my session");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    fireEvent.keyDown(container.querySelector(".vs-session-badge")!, {
      key: "Enter",
    });

    expect(mockOpenDetail).toHaveBeenCalledOnce();
    expect(mockOpenDetail).toHaveBeenCalledWith("s1", "abc123");
  });

  it("keyboard Space triggers openDetail", () => {
    const sess = mkSession("s1", "my session");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    fireEvent.keyDown(container.querySelector(".vs-session-badge")!, {
      key: " ",
    });

    expect(mockOpenDetail).toHaveBeenCalledOnce();
    expect(mockOpenDetail).toHaveBeenCalledWith("s1", "abc123");
  });

  it("+N secondary marker has aria-label describing additional links", () => {
    const sess = mkSession("s1", "my session");
    const primary = mkLink("l1", "s1", "abc123", "confirmedManual", true);
    const sec = mkLink("l2", "s2", "abc123", "pending", false);
    setupSelectors({
      primaryLink: primary,
      allLinks: [primary, sec],
      session: sess,
    });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const secondary = container.querySelector(".vs-session-badge__secondary");
    expect(secondary!.getAttribute("aria-label")).toContain("1 more");
  });

  it("badge is a <button> element (keyboard reachable via Tab)", () => {
    const sess = mkSession("s1", "my session");
    const link = mkLink("l1", "s1", "abc123", "confirmedManual");
    setupSelectors({ primaryLink: link, session: sess });

    const { container } = render(() => (
      <SessionBadge commitSha="abc123" workspaceId={() => "ws-1"} />
    ));

    const badge = container.querySelector(".vs-session-badge");
    expect(badge!.tagName.toLowerCase()).toBe("button");
  });
});
