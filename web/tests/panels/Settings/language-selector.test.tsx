import { render, screen, fireEvent, waitFor } from "@solidjs/testing-library";
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
    return null;
  }),
}));

import { invoke } from "@tauri-apps/api/core";
import { reloadSettings } from "../../../src/stores/settings";
import { AppearanceGroup } from "../../../src/panels/Settings/AppearanceGroup";

beforeEach(async () => {
  resetMockSettings();
  vi.mocked(invoke).mockClear();
  await reloadSettings();
});

describe("FEAT-02 Language selector", () => {
  it("TEST-FEAT-02.2: renders language selector and updates zh-Hans", async () => {
    render(() => <AppearanceGroup />);

    const select = (await screen.findByLabelText(
      "Language",
    )) as HTMLSelectElement;
    expect(select.value).toBe("en");
    expect(screen.getByRole("group", { name: "Theme" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "English" })).toHaveValue("en");
    expect(
      screen.getByRole("option", { name: "Simplified Chinese" }),
    ).toHaveValue("zh-Hans");

    fireEvent.change(select, { target: { value: "zh-Hans" } });

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith(
        "settings_update",
        expect.objectContaining({
          req: expect.objectContaining({ language: "zh-Hans" }),
        }),
      );
    });
    await waitFor(() => {
      expect(document.documentElement.lang).toBe("zh-Hans");
    });
    const translatedSelect = screen.getByLabelText("语言") as HTMLSelectElement;
    expect(translatedSelect.value).toBe("zh-Hans");
    expect(screen.getByRole("option", { name: "简体中文" })).toHaveValue(
      "zh-Hans",
    );
    expect(screen.getByRole("group", { name: "主题" })).toBeInTheDocument();
    expect(screen.getByLabelText(/字体/)).toBeInTheDocument();
  });

  it("keeps background opacity in a readable glass range", async () => {
    mockAppSettings.bgOpacity = 0.2;
    await reloadSettings();

    render(() => <AppearanceGroup />);

    const slider = (await screen.findByLabelText(
      /Background opacity/,
    )) as HTMLInputElement;
    expect(slider.min).toBe("0.65");
    expect(slider.max).toBe("1");
    expect(slider.step).toBe("0.05");
    expect(slider.value).toBe("0.65");
    expect(document.documentElement.style.getPropertyValue("--bg-opacity")).toBe(
      "0.65",
    );

    fireEvent.input(slider, { target: { value: "0.3" } });

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith(
        "settings_update",
        expect.objectContaining({
          req: expect.objectContaining({ bgOpacity: 0.65 }),
        }),
      );
    });
  });
});
