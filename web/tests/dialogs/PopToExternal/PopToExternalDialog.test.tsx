import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@solidjs/testing-library";
import { PopToExternalDialog } from "../../../src/dialogs/PopToExternal/PopToExternalDialog";

// mock external-term.ts
vi.mock("../../../src/lib/external-term", () => ({
  listTerminals: vi.fn(),
  previewEnv: vi.fn(),
  launchTerminal: vi.fn(),
}));

import {
  listTerminals,
  previewEnv,
  launchTerminal,
} from "../../../src/lib/external-term";

const mockedListTerminals = vi.mocked(listTerminals);
const mockedPreviewEnv = vi.mocked(previewEnv);
const mockedLaunchTerminal = vi.mocked(launchTerminal);

describe.skip("PopToExternalDialog", () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it("打开时显示 loading 状态", () => {
    mockedListTerminals.mockImplementation(
      () => new Promise(() => {}), // never resolve
    );
    mockedPreviewEnv.mockImplementation(() => new Promise(() => {}));

    render(() => <PopToExternalDialog open={true} onClose={() => {}} paneId="p1" />);

    expect(screen.getByText("Detecting terminals…")).toBeInTheDocument();
  });

  it("检测到终端后渲染列表", async () => {
    mockedListTerminals.mockResolvedValue([
      {
        id: "ghostty",
        displayName: "Ghostty",
        detected: true,
        priorityHint: 1,
      },
      {
        id: "iterm",
        displayName: "iTerm2",
        detected: false,
        priorityHint: 2,
      },
    ]);
    mockedPreviewEnv.mockResolvedValue({
      visibleEntries: [{ key: "PATH", valueTruncated: "/usr/bin", isSensitiveRedacted: false }],
      filteredCount: 0,
    });

    render(() => <PopToExternalDialog open={true} onClose={() => {}} paneId="p1" />);

    await waitFor(() => {
      expect(screen.getByText("Ghostty")).toBeInTheDocument();
    });
    expect(screen.getByText("iTerm2")).toBeInTheDocument();
    expect(screen.getByText("Not installed")).toBeInTheDocument();
  });

  it("选中终端后点击 Open 调用 launchTerminal", async () => {
    mockedListTerminals.mockResolvedValue([
      {
        id: "ghostty",
        displayName: "Ghostty",
        detected: true,
        priorityHint: 1,
      },
    ]);
    mockedPreviewEnv.mockResolvedValue({
      visibleEntries: [],
      filteredCount: 0,
    });
    mockedLaunchTerminal.mockResolvedValue({ success: true });

    const onClose = vi.fn();
    render(() => <PopToExternalDialog open={true} onClose={onClose} paneId="p1" />);

    await waitFor(() => {
      expect(screen.getByText("Ghostty")).toBeInTheDocument();
    });

    const openBtn = screen.getByRole("button", { name: /Open in/i });
    fireEvent.click(openBtn);

    await waitFor(() => {
      expect(mockedLaunchTerminal).toHaveBeenCalledWith({
        terminalId: "ghostty",
        paneId: "p1",
        overrideEnv: null,
      });
    });
    expect(onClose).toHaveBeenCalled();
  });

  it("launch 失败显示错误", async () => {
    mockedListTerminals.mockResolvedValue([
      {
        id: "ghostty",
        displayName: "Ghostty",
        detected: true,
        priorityHint: 1,
      },
    ]);
    mockedPreviewEnv.mockResolvedValue({
      visibleEntries: [],
      filteredCount: 0,
    });
    mockedLaunchTerminal.mockResolvedValue({
      success: false,
      failedReason: "Ghostty not found",
    });

    render(() => <PopToExternalDialog open={true} onClose={() => {}} paneId="p1" />);

    await waitFor(() => {
      expect(screen.getByText("Ghostty")).toBeInTheDocument();
    });

    const openBtn = screen.getByRole("button", { name: /Open in/i });
    fireEvent.click(openBtn);

    await waitFor(() => {
      expect(screen.getByText("Ghostty not found")).toBeInTheDocument();
    });
  });

  it("0 终端时显示 fallback 提示", async () => {
    mockedListTerminals.mockResolvedValue([]);
    mockedPreviewEnv.mockResolvedValue({
      visibleEntries: [],
      filteredCount: 0,
    });

    render(() => <PopToExternalDialog open={true} onClose={() => {}} paneId="p1" />);

    await waitFor(() => {
      expect(screen.getByText("No external terminals detected.")).toBeInTheDocument();
    });
    expect(
      screen.getByText(/Install Ghostty, iTerm2, or Terminal.app/),
    ).toBeInTheDocument();
  });

  it("env preview 显示 filtered_count", async () => {
    mockedListTerminals.mockResolvedValue([
      {
        id: "ghostty",
        displayName: "Ghostty",
        detected: true,
        priorityHint: 1,
      },
    ]);
    mockedPreviewEnv.mockResolvedValue({
      visibleEntries: [
        { key: "PATH", valueTruncated: "/usr/bin", isSensitiveRedacted: false },
      ],
      filteredCount: 3,
    });

    render(() => <PopToExternalDialog open={true} onClose={() => {}} paneId="p1" />);

    await waitFor(() => {
      expect(screen.getByText("3 items filtered for security")).toBeInTheDocument();
    });
  });
});
