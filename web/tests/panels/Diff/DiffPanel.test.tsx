import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import type { AppSettings } from "../../../src/bindings/AppSettings";
import type { DiffResponse } from "../../../src/bindings";

const { mockAppSettings, resetMockSettings } = vi.hoisted(() => {
  const defaultFixture = (): AppSettings => ({
    language: "en",
    theme: "dark",
    uiFontFamily: "Inter",
    fontFamily: "JetBrains Mono",
    fontSize: 14,
    defaultShell: "/bin/bash",
    pasteProtection: true,
    telemetryOptIn: null,
    gitUserName: null,
    gitUserEmail: null,
    bgOpacity: 0.85,
    bgBlur: 20,
    windowPaddingX: 2,
    windowPaddingY: 2,
    cursorStyle: "block",
    cursorBlink: false,
    unfocusedPaneOpacity: 0.7,
    ptyPoolEnabled: true,
    ptyPoolSize: 1,
    primaryWidth: 236,
    secondaryWidth: 400,
    bottomHeight: 240,
    externalTermPreferred: null,
    externalTermDontAskAgain: false,
  });
  const mockAppSettings: AppSettings = defaultFixture();
  return {
    mockAppSettings,
    resetMockSettings: () => {
      Object.assign(mockAppSettings, defaultFixture());
    },
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "settings_get") {
      return { ...mockAppSettings };
    }
    return null;
  }),
}));

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
  DiffLineContent: (props: {
    disableHighlight?: boolean;
    fileSize?: number;
  }) => {
    diffLineHarness.props.push(props);
    return <span data-testid="mock-diff-line">line</span>;
  },
}));

import { reloadSettings } from "../../../src/stores/settings";
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
  beforeEach(async () => {
    cleanup();
    resetMockSettings();
    mockAppSettings.language = "zh-Hans";
    await reloadSettings();
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
    expect(screen.queryByText(/大文件.*语法高亮已禁用/i)).not.toBeNull();
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
