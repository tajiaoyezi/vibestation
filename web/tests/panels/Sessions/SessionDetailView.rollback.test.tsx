import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import type { RollbackPreview } from "../../../src/bindings";

const mockDispatch = vi.fn();
const mockRollbackPreview = vi.fn();
const mockRollbackExecute = vi.fn();
const mockRollbackAbort = vi.fn();

let conflictCb: ((p: { path: string; commitSha: string }) => void) | undefined;
let doneCb: ((p: { sessionId: string; revertShas: string[] }) => void) | undefined;
let abortedCb: ((p: { sessionId: string; headSha: string }) => void) | undefined;

vi.mock("../../../src/stores/layout-context", () => ({
  useLayout: () => ({ dispatch: mockDispatch }),
}));

vi.mock("../../../src/panels/SessionDetail/rollbackApi", () => ({
  rollbackPreview: (...args: unknown[]) => mockRollbackPreview(...args),
  rollbackExecute: (...args: unknown[]) => mockRollbackExecute(...args),
  rollbackAbort: (...args: unknown[]) => mockRollbackAbort(...args),
  onRollbackConflict: async (
    cb: (p: { path: string; commitSha: string }) => void,
  ) => {
    conflictCb = cb;
    return () => {};
  },
  onRollbackDone: async (
    cb: (p: { sessionId: string; revertShas: string[] }) => void,
  ) => {
    doneCb = cb;
    return () => {};
  },
  onRollbackAborted: async (
    cb: (p: { sessionId: string; headSha: string }) => void,
  ) => {
    abortedCb = cb;
    return () => {};
  },
}));

vi.mock("../../../src/panels/SessionDetail/RollbackPreviewModal", () => ({
  RollbackPreviewModal: (props: { onConfirm: (shas: string[]) => void }) => (
    <button onClick={() => props.onConfirm(["sha-1"])}>mock-preview-confirm</button>
  ),
}));

vi.mock("../../../src/panels/SessionDetail/RollbackConfirmDialog", () => ({
  RollbackConfirmDialog: (props: { onExecute: () => void }) => (
    <button onClick={props.onExecute}>mock-execute</button>
  ),
}));

vi.mock("../../../src/panels/SessionDetail/RollbackProgressBanner", () => ({
  RollbackProgressBanner: () => <div data-testid="progress-banner" />,
}));

vi.mock("../../../src/panels/SessionDetail/RollbackConflictView", () => ({
  RollbackConflictView: () => <div data-testid="rollback-conflict-view" />,
}));

const mockSession = {
  id: "sess-1",
  workspaceId: "ws-1",
  cliKind: "claude",
  source: "auto",
  title: "session one",
  startedAt: 1760000000000,
  endedAt: null,
  endReason: null,
  promptCount: 3,
  tokenCount: null,
  eventCount: 0,
  status: "active" as const,
  parserVersion: null,
  strategyVersion: null,
  metadataJson: "{}",
  createdAt: 1760000000000,
  updatedAt: 1760000000000,
};

const mockLink = {
  id: "link-1",
  workspaceId: "ws-1",
  sessionId: "sess-1",
  commitSha: "abc1234",
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

const mockStore = {
  sessionsByWorkspace: { "ws-1": [mockSession] } as Record<string, typeof mockSession[]>,
  lastError: null,
  clearError: vi.fn(),
};

vi.mock("../../../src/stores/sessions-context", () => ({
  useSessions: () => ({
    store: mockStore,
    activeDetail: () => ({ sessionId: "sess-1" }),
    closeDetail: vi.fn(),
    getDetail: vi.fn(async () => ({
      session: mockSession,
      links: [mockLink],
      commitCount: 1,
      avgConfidence: 0.95,
    })),
    unbind: vi.fn(async () => ({})),
    rebind: vi.fn(async () => ({})),
    recalc: vi.fn(async () => ({})),
    end: vi.fn(async () => ({})),
  }),
}));

import { SessionDetailView } from "../../../src/panels/Sessions/SessionDetailView";

describe("SessionDetailView rollback Phase C wiring", () => {
  beforeEach(() => {
    mockDispatch.mockReset();
    mockRollbackPreview.mockReset();
    mockRollbackExecute.mockReset();
    mockRollbackAbort.mockReset();
    conflictCb = undefined;
    doneCb = undefined;
    abortedCb = undefined;
  });

  it("dirty tree error dispatches open-bottom and shows error message", async () => {
    mockRollbackPreview.mockRejectedValueOnce(
      JSON.stringify({
        kind: "dirtyWorkingTree",
        modified: ["a.ts"],
        staged: [],
      }),
    );

    render(() => <SessionDetailView />);

    await fireEvent.click(await screen.findByText("↩ 一键回滚"));
    await waitFor(() => {
      expect(mockDispatch).toHaveBeenCalledWith({ kind: "open-bottom" });
    });
    expect(
      screen.getByText(/已自动打开底部 Git Status 面板/),
    ).toBeInTheDocument();
  });

  it("conflict event shows rollback conflict view", async () => {
    const preview: RollbackPreview = {
      sessionId: "sess-1",
      commits: [],
      totalFilesChanged: 1,
      totalInsertions: 1,
      totalDeletions: 1,
      hasLowConfidence: false,
    };
    mockRollbackPreview.mockResolvedValueOnce(preview);
    mockRollbackExecute.mockResolvedValue({});

    render(() => <SessionDetailView />);

    await fireEvent.click(await screen.findByText("↩ 一键回滚"));
    await fireEvent.click(screen.getByText("mock-preview-confirm"));
    await fireEvent.click(screen.getByText("mock-execute"));
    conflictCb?.({ path: "src/main.rs", commitSha: "abc1234" });

    await waitFor(() => {
      expect(screen.getByTestId("rollback-conflict-view")).toBeInTheDocument();
    });
  });

  it("rollback done event clears conflict view state", async () => {
    const preview: RollbackPreview = {
      sessionId: "sess-1",
      commits: [],
      totalFilesChanged: 1,
      totalInsertions: 1,
      totalDeletions: 1,
      hasLowConfidence: false,
    };
    mockRollbackPreview.mockResolvedValueOnce(preview);
    mockRollbackExecute.mockResolvedValue({});

    render(() => <SessionDetailView />);

    await fireEvent.click(await screen.findByText("↩ 一键回滚"));
    await fireEvent.click(screen.getByText("mock-preview-confirm"));
    await fireEvent.click(screen.getByText("mock-execute"));
    conflictCb?.({ path: "src/main.rs", commitSha: "abc1234" });
    await waitFor(() => {
      expect(screen.getByTestId("rollback-conflict-view")).toBeInTheDocument();
    });

    doneCb?.({ sessionId: "sess-1", revertShas: ["r1"] });
    await waitFor(() => {
      expect(screen.queryByTestId("rollback-conflict-view")).toBeNull();
    });
  });
});
