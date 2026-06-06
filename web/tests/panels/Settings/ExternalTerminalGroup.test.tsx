import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@solidjs/testing-library";
import type { AppSettings } from "../../../src/bindings/AppSettings";

const { mockAppSettings, resetMockSettings } = vi.hoisted(() => {
  const defaultFixture = (): AppSettings => ({
    language: "en",
    theme: "dark",
    fontFamily: "monospace",
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
  invoke: vi.fn(
    async (cmd: string, args?: { req?: Record<string, unknown> }) => {
      if (cmd === "settings_get") {
        return { ...mockAppSettings };
      }
      if (cmd === "external_term_list") {
        return [
          {
            id: "ghostty",
            displayName: "Ghostty",
            detected: true,
            priorityHint: 100,
          },
          {
            id: "iterm2",
            displayName: "iTerm2",
            detected: true,
            priorityHint: 90,
          },
          {
            id: "missing",
            displayName: "Nope",
            detected: false,
            priorityHint: 0,
          },
        ];
      }
      if (cmd === "settings_update") {
        const req = args?.req as {
          externalTermPreferred?: string | null;
          externalTermDontAskAgain?: boolean;
        };
        if (req.externalTermPreferred !== undefined) {
          mockAppSettings.externalTermPreferred = req.externalTermPreferred;
        }
        if (req.externalTermDontAskAgain !== undefined) {
          mockAppSettings.externalTermDontAskAgain =
            req.externalTermDontAskAgain;
        }
        return { ...mockAppSettings };
      }
      return null;
    },
  ),
}));

import { invoke } from "@tauri-apps/api/core";
import { reloadSettings } from "../../../src/stores/settings";
import { ExternalTerminalGroup } from "../../../src/panels/Settings/ExternalTerminalGroup";

beforeEach(async () => {
  resetMockSettings();
  vi.mocked(invoke).mockClear();
  await reloadSettings();
});

describe("ExternalTerminalGroup", () => {
  it("renders dropdown with Ask every time and detected terminals", async () => {
    render(() => <ExternalTerminalGroup />);

    expect(await screen.findByRole("combobox")).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "Ask every time" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(
        screen.getByRole("option", { name: "Ghostty" }),
      ).toBeInTheDocument();
    });
    expect(screen.getByRole("option", { name: "iTerm2" })).toBeInTheDocument();
    expect(
      screen.queryByRole("option", { name: "Nope" }),
    ).not.toBeInTheDocument();
  });

  it("selecting a terminal calls settings_update with externalTermPreferred", async () => {
    render(() => <ExternalTerminalGroup />);

    const select = await screen.findByRole("combobox");
    await waitFor(() => {
      expect(
        screen.getByRole("option", { name: "Ghostty" }),
      ).toBeInTheDocument();
    });
    vi.mocked(invoke).mockClear();
    fireEvent.change(select, { target: { value: "ghostty" } });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "settings_update",
      expect.objectContaining({
        req: expect.objectContaining({ externalTermPreferred: "ghostty" }),
      }),
    );
  });

  it("Don't ask again toggle is disabled when externalTermPreferred is null", async () => {
    render(() => <ExternalTerminalGroup />);

    const toggle = await screen.findByRole("switch");
    expect(toggle).toBeDisabled();
  });

  it("clicking Don't ask again calls settings_update with inverted value when preferred is set", async () => {
    mockAppSettings.externalTermPreferred = "ghostty";
    await reloadSettings();

    render(() => <ExternalTerminalGroup />);

    const toggle = await screen.findByRole("switch");
    expect(toggle).not.toBeDisabled();
    fireEvent.click(toggle);

    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "settings_update",
      expect.objectContaining({
        req: expect.objectContaining({ externalTermDontAskAgain: true }),
      }),
    );
  });

  it("renders all env whitelist keys", async () => {
    render(() => <ExternalTerminalGroup />);

    await screen.findByRole("combobox");
    for (const key of ["PATH", "HOME", "LANG", "TERM", "SHELL", "USER"]) {
      expect(screen.getByText(key)).toBeInTheDocument();
    }
  });
});
