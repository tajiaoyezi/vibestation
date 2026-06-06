import { render, screen, waitFor } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppSettings } from "../../src/bindings/AppSettings";
import type { BranchInfo } from "../../src/bindings/BranchInfo";
import type { GitLogEntry } from "../../src/bindings/GitLogEntry";
import type { RemoteInfo } from "../../src/bindings/RemoteInfo";
import type { WorkspaceMetadata } from "../../src/bindings/WorkspaceMetadata";

const {
  mockAppSettings,
  resetMockSettings,
  branchListMode,
  setBranchListMode,
} = vi.hoisted(() => {
  const defaultFixture = (): AppSettings => ({
    language: "en",
    theme: "dark",
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
  const branchListMode = { current: "empty" as "empty" | "loading" | "error" };
  return {
    mockAppSettings,
    resetMockSettings: () => {
      Object.assign(mockAppSettings, defaultFixture());
      branchListMode.current = "empty";
    },
    branchListMode,
    setBranchListMode: (mode: "empty" | "loading" | "error") => {
      branchListMode.current = mode;
    },
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "settings_get") {
      return { ...mockAppSettings };
    }
    if (cmd === "branch_list") {
      if (branchListMode.current === "loading") {
        return new Promise(() => undefined);
      }
      if (branchListMode.current === "error") {
        throw new Error("branch list failed");
      }
      return { branches: [], headName: null };
    }
    if (cmd === "telemetry_opt_in_set") {
      return null;
    }
    return null;
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => vi.fn()),
}));

vi.mock("../../src/lib/external-term", () => ({
  listTerminals: vi.fn(),
  previewEnv: vi.fn(),
  launchTerminal: vi.fn(),
}));

import { reloadSettings } from "../../src/stores/settings";
import { TelemetryOptInModal } from "../../src/dialogs/TelemetryOptIn/TelemetryOptInModal";
import { PopToExternalDialog } from "../../src/dialogs/PopToExternal/PopToExternalDialog";
import { BranchSwitcher } from "../../src/dialogs/BranchSwitcher/BranchSwitcher";
import { CreateBranchDialog } from "../../src/dialogs/CreateBranchDialog/CreateBranchDialog";
import { CherryPickDialog } from "../../src/dialogs/CherryPickDialog/CherryPickDialog";
import { MergeDialog } from "../../src/dialogs/MergeDialog/MergeDialog";
import { RemoteSelector } from "../../src/dialogs/RemoteSelector/RemoteSelector";
import { listTerminals, previewEnv } from "../../src/lib/external-term";

const mockedListTerminals = vi.mocked(listTerminals);
const mockedPreviewEnv = vi.mocked(previewEnv);

const gitWorkspace: WorkspaceMetadata = {
  workspaceId: "ws-1",
  name: "Repo",
  path: "C:\\repo",
  repoRoot: "C:\\repo",
  hasGit: true,
  createdAt: 1,
  lastOpened: 1,
};

const mainBranch: BranchInfo = {
  name: "main",
  fullRef: "refs/heads/main",
  kind: "local",
  upstream: null,
  ahead: 0,
  behind: 0,
  headCommit: "abc123",
};

const commit: GitLogEntry = {
  shortSha: "abc1234",
  message: "keep commit messages raw",
  authorName: "Leaf",
  authoredDate: 1,
  relativeTime: "now",
  branchLabels: [],
  tagLabels: [],
};

const originRemote: RemoteInfo = {
  name: "origin",
  url: "https://example.test/repo.git",
  fetchUrl: "https://example.test/repo.git",
};

beforeEach(async () => {
  vi.clearAllMocks();
  mockedListTerminals.mockReset();
  mockedPreviewEnv.mockReset();
  resetMockSettings();
  mockAppSettings.language = "zh-Hans";
  await reloadSettings();
});

describe("FEAT-02 dialog chrome copy", () => {
  it("TEST-FEAT-02.6: renders telemetry opt-in chrome in zh-Hans", () => {
    render(() => <TelemetryOptInModal />);

    expect(
      screen.getByRole("heading", { name: "帮助改进 Vibestation" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "拒绝" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "接受" })).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders pop-to-external empty and error chrome in zh-Hans", async () => {
    mockedListTerminals.mockResolvedValue([]);
    mockedPreviewEnv.mockResolvedValue({
      visibleEntries: [],
      filteredCount: 0,
    });

    render(() => (
      <PopToExternalDialog open={true} onClose={vi.fn()} paneId="pane-1" />
    ));

    expect(
      screen.getByRole("heading", { name: "在外部终端打开" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("关闭对话框")).toBeInTheDocument();
    expect(screen.getByText("正在检测终端...")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText("未检测到外部终端。")).toBeInTheDocument();
    });
  });

  it("TEST-FEAT-02.6: renders pop-to-external option chrome in zh-Hans", async () => {
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

    render(() => (
      <PopToExternalDialog open={true} onClose={vi.fn()} paneId="pane-1" />
    ));

    await waitFor(() => {
      expect(screen.getByText("Ghostty")).toBeInTheDocument();
    });
    expect(screen.getByText("不要再次询问")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "取消" })).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders branch switcher chrome in zh-Hans", async () => {
    setBranchListMode("empty");

    render(() => (
      <BranchSwitcher
        activeWorkspace={() => gitWorkspace}
        open={() => true}
        onClose={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("heading", { name: "切换分支" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText("没有匹配的分支")).toBeInTheDocument();
    });
  });

  it("TEST-FEAT-02.6: renders branch switcher loading and retry chrome in zh-Hans", async () => {
    setBranchListMode("loading");

    render(() => (
      <BranchSwitcher
        activeWorkspace={() => gitWorkspace}
        open={() => true}
        onClose={vi.fn()}
      />
    ));

    expect(screen.getByText("正在加载分支...")).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders create branch chrome in default English", async () => {
    mockAppSettings.language = "en";
    await reloadSettings();

    render(() => (
      <CreateBranchDialog
        branches={[mainBranch]}
        onCreate={vi.fn(async () => false)}
        onCancel={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("heading", { name: "Create branch" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Name")).toBeInTheDocument();
    expect(screen.getByText("From")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Confirm" })).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders cherry-pick chrome in zh-Hans", () => {
    render(() => (
      <CherryPickDialog
        workspaceId="ws-1"
        commits={[commit]}
        onCancel={vi.fn()}
      />
    ));

    expect(
      screen.getByLabelText("关闭 cherry-pick 对话框"),
    ).toBeInTheDocument();
    expect(screen.getByText("逐个自动提交")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "取消" })).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders merge dialog chrome in default English", async () => {
    mockAppSettings.language = "en";
    await reloadSettings();

    render(() => (
      <MergeDialog
        workspaceId="ws-1"
        currentBranch="main"
        initialSource="feature"
        onCancel={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("heading", { name: "Merge feature into main" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("Close merge dialog")).toBeInTheDocument();
    expect(screen.getByText("source branch")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Search branch")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Merge" })).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders remote selector chrome in default English", async () => {
    mockAppSettings.language = "en";
    await reloadSettings();

    render(() => (
      <RemoteSelector
        operation="push"
        branch="main"
        remotes={[originRemote]}
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("heading", { name: "Select push remote" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Continue" }),
    ).toBeInTheDocument();
  });
});
