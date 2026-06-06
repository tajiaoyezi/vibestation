import { render, screen } from "@solidjs/testing-library";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AppSettings } from "../../../src/bindings/AppSettings";

const { mockAppSettings, resetMockSettings } = vi.hoisted(() => {
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
  return {
    mockAppSettings,
    resetMockSettings: () => {
      Object.assign(mockAppSettings, defaultFixture());
    },
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string, args?: { req?: Partial<AppSettings> }) => {
    if (cmd === "settings_get") {
      return { ...mockAppSettings };
    }
    if (cmd === "settings_update") {
      Object.assign(mockAppSettings, args?.req ?? {});
      return { ...mockAppSettings };
    }
    if (cmd === "available_shells_list") {
      return [];
    }
    if (cmd === "external_term_list") {
      return [];
    }
    if (cmd === "telemetry_status_get") {
      return {
        optIn: mockAppSettings.telemetryOptIn,
        endpointHost: "Not configured",
        dataCollectionSummary: "",
        initialized: false,
      };
    }
    return null;
  }),
}));

import { reloadSettings } from "../../../src/stores/settings";
import { SettingsPanel } from "../../../src/panels/Settings/SettingsPanel";

beforeEach(async () => {
  resetMockSettings();
  mockAppSettings.language = "zh-Hans";
  await reloadSettings();
});

describe("FEAT-02 Settings chrome copy", () => {
  it("TEST-FEAT-02.6: renders Settings shell and group labels in zh-Hans", () => {
    render(() => (
      <SettingsPanel visible={true} onClose={vi.fn()} onOpenImport={vi.fn()} />
    ));

    expect(
      screen.getByRole("dialog", { name: "偏好设置" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "偏好设置" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "导入..." })).toBeInTheDocument();
    expect(screen.getByLabelText("关闭设置")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "外观" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "终端" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "外部终端" }),
    ).toBeInTheDocument();
  });

  it("TEST-FEAT-02.6: renders remaining Settings controls in zh-Hans", async () => {
    render(() => (
      <SettingsPanel visible={true} onClose={vi.fn()} onOpenImport={vi.fn()} />
    ));

    expect(screen.getByLabelText("默认 Shell")).toBeInTheDocument();
    expect(
      screen.getByRole("switch", { name: "粘贴保护" }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/未聚焦窗格不透明度/)).toBeInTheDocument();
    expect(
      screen.getByRole("switch", { name: /PTY 预热池/ }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/池大小/)).toBeInTheDocument();

    screen.getByRole("button", { name: "外部终端" }).click();
    expect(screen.getByText("首选终端")).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "每次询问" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("switch", { name: /不要再次询问/ }),
    ).toBeInTheDocument();

    expect(screen.getByLabelText("用户名")).toBeInTheDocument();
    expect(screen.getByLabelText("用户邮箱")).toBeInTheDocument();

    expect(screen.getByText("遥测")).toBeInTheDocument();
    expect(screen.getByText("未决定")).toBeInTheDocument();
    expect(screen.getByText("收集端点")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "查看收集内容" }),
    ).toBeInTheDocument();
  });
});
