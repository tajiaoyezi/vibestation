import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { render, cleanup, waitFor, screen } from "@solidjs/testing-library";
import type { DiffResponse } from "../../../src/bindings";

const diffApiMocks = vi.hoisted(() => ({
  computeDiff: vi.fn(),
  getDiffViewMode: vi.fn(),
  setDiffViewMode: vi.fn().mockResolvedValue(undefined),
}));

const diffLineHarness = vi.hoisted(() => ({
  props: [] as Array<{
    disableHighlight?: boolean;
    fileSize?: number;
  }>,
}));

vi.mock("../../../src/panels/Diff/diffApi", () => ({
  computeDiff: diffApiMocks.computeDiff,
  getDiffViewMode: diffApiMocks.getDiffViewMode,
  setDiffViewMode: diffApiMocks.setDiffViewMode,
}));

vi.mock("../../../src/panels/Diff/DiffLine", () => ({
  DiffLineContent: (props: { disableHighlight?: boolean; fileSize?: number }) => {
    diffLineHarness.props.push(props);
    return <span data-testid="mock-diff-line">line</span>;
  },
}));

import { DiffPanel } from "../../../src/panels/Diff/DiffPanel";

function buildDiffResponse(
  oldSizeBytes: number | null,
  newSizeBytes: number | null,
): DiffResponse {
  return {
    hunks: [
      {
        oldStart: 1,
        newStart: 1,
        lines: [
          {
            oldLineNum: 1,
            newLineNum: 1,
            lineType: "context",
            content: "const x = 1;",
          },
        ],
      },
    ],
    binary: false,
    truncated: false,
    truncatedReason: null,
    oldSizeBytes,
    newSizeBytes,
    lineCount: 1,
  };
}

describe("DiffPanel · Phase C file-size gate", () => {
  beforeEach(() => {
    cleanup();
    diffLineHarness.props.length = 0;
    diffApiMocks.computeDiff.mockReset();
    diffApiMocks.getDiffViewMode.mockReset();
    diffApiMocks.setDiffViewMode.mockClear();
    diffApiMocks.getDiffViewMode.mockResolvedValue("split");
  });

  afterEach(cleanup);

  it(">= 50MB 时透传 disableHighlight=true 并显示 Large file chip", async () => {
    diffApiMocks.computeDiff.mockResolvedValue(
      buildDiffResponse(30 * 1024 * 1024, 25 * 1024 * 1024),
    );

    render(() => (
      <DiffPanel
        workspaceId="ws-1"
        source="working-tree"
        filePath="src/app.ts"
      />
    ));

    await waitFor(() => {
      expect(diffLineHarness.props.length).toBeGreaterThan(0);
    });

    expect(diffLineHarness.props[0].disableHighlight).toBe(true);
    expect(diffLineHarness.props[0].fileSize).toBe(55 * 1024 * 1024);
    expect(
      screen.queryByText(/Large file.*语法高亮已禁用/i),
    ).not.toBeNull();
  });

  it("< 50MB 时 disableHighlight=false", async () => {
    diffApiMocks.computeDiff.mockResolvedValue(
      buildDiffResponse(1024 * 1024, 1024 * 1024),
    );

    render(() => (
      <DiffPanel
        workspaceId="ws-2"
        source="working-tree"
        filePath="src/app.ts"
      />
    ));

    await waitFor(() => {
      expect(diffLineHarness.props.length).toBeGreaterThan(0);
    });

    expect(diffLineHarness.props[0].disableHighlight).toBe(false);
    expect(screen.queryByText(/语法高亮已禁用/i)).toBeNull();
  });
});
