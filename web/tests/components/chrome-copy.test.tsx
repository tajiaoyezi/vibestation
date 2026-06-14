import { render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppSettings } from "../../src/bindings/AppSettings";

const { mockAppSettings, resetMockSettings, mockWindow } = vi.hoisted(() => {
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
    mockWindow: {
      close: vi.fn(async () => undefined),
      isMaximized: vi.fn(async () => false),
      minimize: vi.fn(async () => undefined),
      onResized: vi.fn(async () => vi.fn()),
      setTheme: vi.fn(async () => undefined),
      toggleMaximize: vi.fn(async () => undefined),
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

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => mockWindow,
}));

vi.mock("../../src/lib/platform", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("../../src/lib/platform")>();
  return {
    ...actual,
    detectPlatform: () => "windows",
  };
});

vi.mock("../../src/panels/Terminal", () => ({
  Terminal: () => <div data-testid="terminal-panel" />,
}));

vi.mock("../../src/panels/Diff", () => ({
  DiffPanel: () => <div data-testid="diff-panel" />,
}));

vi.mock("../../src/panels/GitLog", () => ({
  GitLogPanel: () => <div data-testid="git-log-panel" />,
}));

import { reloadSettings } from "../../src/stores/settings";
import { LayoutProvider } from "../../src/stores/layout-context";
import { PrimarySidebar } from "../../src/components/PrimarySidebar";
import { ActivityStrip } from "../../src/components/ActivityStrip";
import { TopBar } from "../../src/components/TopBar";
import { BottomPanel } from "../../src/components/BottomPanel";
import { MainContent } from "../../src/components/MainContent";
import { SecondarySidebar } from "../../src/components/SecondarySidebar";
import type { WorkspaceMetadata } from "../../src/bindings/WorkspaceMetadata";

const renderWithLayout = (view: () => unknown) =>
  render(() => (
    <LayoutProvider activeWorkspaceId={() => null} dbReady={() => false}>
      {view()}
    </LayoutProvider>
  ));

beforeEach(async () => {
  resetMockSettings();
  mockAppSettings.language = "zh-Hans";
  await reloadSettings();
});

describe("FEAT-02 workspace chrome copy", () => {
  it("TEST-FEAT-02.6: renders Primary Sidebar labels in zh-Hans", () => {
    render(() => (
      <PrimarySidebar
        workspaces={() => []}
        activeWorkspace={() => null}
        onOpen={vi.fn()}
        onCreate={vi.fn()}
        onDelete={vi.fn()}
        onOpenImport={vi.fn()}
        loading={() => false}
        layout={() => ({ primaryOpen: true, primaryWidth: 236 })}
        onResizeStart={vi.fn()}
        onResizeReset={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("complementary", { name: "主侧边栏" }),
    ).toBeInTheDocument();
    expect(screen.getByText("工作区")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "创建工作区" }),
    ).toBeInTheDocument();
    expect(screen.getByText("还没有工作区。")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "从其他终端导入设置" }),
    ).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders activity strip and bottom panel labels in zh-Hans", () => {
    renderWithLayout(() => (
      <>
        <ActivityStrip />
        <BottomPanel
          layout={() => ({ bottomOpen: true, bottomHeight: 240 })}
          onResizeStart={vi.fn()}
          onResizeReset={vi.fn()}
          activeWorkspace={() => null as WorkspaceMetadata | null}
          onOpenDiff={vi.fn()}
        />
      </>
    ));

    expect(
      screen.getByRole("toolbar", { name: "面板切换" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Git 日志" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Git 状态" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "底部面板" }),
    ).toBeInTheDocument();
    expect(screen.getByText("输出")).toBeInTheDocument();
    expect(screen.getByText("差异")).toBeInTheDocument();
    expect(screen.getByLabelText("调整底部面板大小")).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders TopBar controls in zh-Hans", () => {
    render(() => (
      <TopBar
        activeWorkspace={() => null}
        primaryOpen={() => true}
        onTogglePrimary={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("button", { name: "切换主侧边栏" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "最小化" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "最大化" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "关闭" })).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders main content labels in zh-Hans", () => {
    const workspace: WorkspaceMetadata = {
      workspaceId: "ws-1",
      name: "Repo",
      path: "C:\\repo",
      repoRoot: "C:\\repo",
      hasGit: true,
      createdAt: 1,
      lastOpened: 1,
    };

    render(() => (
      <MainContent
        activeWorkspace={() => null}
        activeDiff={() => null}
        onCloseDiff={vi.fn()}
        onCloseWorkspaceView={vi.fn()}
        workspaces={() => []}
      />
    ));

    expect(screen.getByRole("main", { name: "主内容区" })).toBeInTheDocument();
    expect(screen.getByText("选择或创建工作区以开始")).toBeInTheDocument();

    render(() => (
      <MainContent
        activeWorkspace={() => workspace}
        activeDiff={() => ({
          workspaceId: workspace.workspaceId,
          source: "worktree",
          filePath: "README.md",
        })}
        onCloseDiff={vi.fn()}
        onCloseWorkspaceView={vi.fn()}
        workspaces={() => [workspace]}
      />
    ));

    expect(screen.getByText("差异")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "返回终端" }),
    ).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders Secondary Sidebar labels in zh-Hans", () => {
    render(() => (
      <SecondarySidebar
        layout={() => ({ secondaryOpen: true, secondaryWidth: 400 })}
        onResizeStart={vi.fn()}
        onResizeReset={vi.fn()}
        activeWorkspace={() => null}
        onOpenDiff={vi.fn()}
        onOpenGitStatus={vi.fn()}
      />
    ));

    expect(
      screen.getByRole("complementary", { name: "副侧边栏" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText("调整副侧边栏大小")).toBeInTheDocument();
  });
});
