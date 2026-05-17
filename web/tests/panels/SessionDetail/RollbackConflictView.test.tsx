import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import type { ConflictedFile } from "../../../src/bindings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("../../../src/panels/Diff/3way", () => ({
  ThreeWayDiffView: (props: {
    onResolvedFile?: (filePath: string) => void;
  }) => (
    <div data-testid="three-way-diff-mock">
      <button onClick={() => props.onResolvedFile?.("src/main.rs")}>
        mark-resolved
      </button>
    </div>
  ),
}));

import { invoke } from "@tauri-apps/api/core";
import { RollbackConflictView } from "../../../src/panels/SessionDetail/RollbackConflictView";

describe("RollbackConflictView", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("continue stays disabled while conflicts remain unresolved", async () => {
    const files: ConflictedFile[] = [
      { path: "src/main.rs", hunks: [], resolved: false },
    ];
    vi.mocked(invoke).mockResolvedValueOnce(files);

    render(() => (
      <RollbackConflictView
        sessionId="session-42"
        workspaceId="ws-1"
        includeShas={["abc"]}
        progress={{ done: 1, total: 3 }}
        initialConflictFile="src/main.rs"
        onResume={vi.fn(async () => {})}
        onAbort={vi.fn(async () => {})}
        onCompleted={vi.fn()}
      />
    ));

    await waitFor(() => {
      expect(screen.getByTestId("three-way-diff-mock")).toBeInTheDocument();
    });
    expect(screen.getByText("Continue")).toBeDisabled();
  });

  it("calls onAbort when abort button clicked", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([]);
    const onAbort = vi.fn(async () => {});

    render(() => (
      <RollbackConflictView
        sessionId="session-42"
        workspaceId="ws-1"
        includeShas={["abc"]}
        progress={{ done: 1, total: 3 }}
        initialConflictFile="src/main.rs"
        onResume={vi.fn(async () => {})}
        onAbort={onAbort}
        onCompleted={vi.fn()}
      />
    ));

    await fireEvent.click(screen.getByText("Abort"));
    await fireEvent.click(screen.getAllByText("Abort")[1]);
    expect(onAbort).toHaveBeenCalledTimes(1);
  });
});
