import { createSignal } from "solid-js";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppSettings, SettingsUpdateRequest } from "../bindings";

export type ThemeSetting = "light" | "dark" | "auto";

const DEFAULTS: AppSettings = {
  theme: "auto",
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
};

const [settings, setSettings] = createStore<AppSettings>({ ...DEFAULTS });

const [loaded, setLoaded] = createSignal(false);

async function loadSettings(): Promise<void> {
  if (loaded()) return;
  try {
    const s = await invoke<AppSettings>("settings_get");
    setSettings(s);
    applyCssVars(s);
  } catch {}
  setLoaded(true);
}

/**
 * 给 App.tsx onMount 调用 · workspace_init 完成（pool init 完成）后强制重新拉 settings ·
 * 不依赖 settings_changed event · 避免 module-load 期 listen 订阅 race miss event 的问题
 * （session 19 实测：fix7 emit 在 listen 完成订阅前发生 · modal 反复弹）
 */
export async function reloadSettings(): Promise<void> {
  try {
    const s = await invoke<AppSettings>("settings_get");
    setSettings(s);
    applyCssVars(s);
  } catch {}
}

loadSettings();

listen<AppSettings>("settings_changed", (event) => {
  setSettings(event.payload);
  applyCssVars(event.payload);
});

function applyCssVars(s: AppSettings): void {
  const root = document.documentElement.style;
  root.setProperty("--bg-opacity", String(s.bgOpacity));
  root.setProperty("--bg-blur", `${s.bgBlur}px`);
  root.setProperty("--window-padding-x", `${s.windowPaddingX}px`);
  root.setProperty("--window-padding-y", `${s.windowPaddingY}px`);
  root.setProperty("--cursor-style", s.cursorStyle);
  root.setProperty("--unfocused-opacity", String(s.unfocusedPaneOpacity));

  const fallback = 'ui-monospace, "SF Mono", "Menlo", "Consolas", monospace';
  root.setProperty("--font-mono", `"${s.fontFamily}", ${fallback}`);

  // 同步 data-theme attribute · 避免 ThemeProvider race · settings_changed 一并生效
  const resolvedTheme =
    s.theme === "auto"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : s.theme;
  document.documentElement.dataset.theme = resolvedTheme;
}

export function useSettings() {
  return {
    settings,
    async updateSettings(partial: Partial<AppSettings>) {
      setSettings(partial as never);

      const req = {} as SettingsUpdateRequest;
      if (partial.theme !== undefined) req.theme = partial.theme;
      if (partial.fontFamily !== undefined) req.fontFamily = partial.fontFamily;
      if (partial.fontSize !== undefined) req.fontSize = partial.fontSize;
      if (partial.defaultShell !== undefined)
        req.defaultShell = partial.defaultShell;
      if (partial.pasteProtection !== undefined)
        req.pasteProtection = partial.pasteProtection;
      if (partial.telemetryOptIn !== undefined)
        req.telemetryOptIn = partial.telemetryOptIn;
      if (partial.gitUserName !== undefined)
        req.gitUserName = partial.gitUserName;
      if (partial.gitUserEmail !== undefined)
        req.gitUserEmail = partial.gitUserEmail;
      if (partial.bgOpacity !== undefined) req.bgOpacity = partial.bgOpacity;
      if (partial.bgBlur !== undefined) req.bgBlur = partial.bgBlur;
      if (partial.windowPaddingX !== undefined)
        req.windowPaddingX = partial.windowPaddingX;
      if (partial.windowPaddingY !== undefined)
        req.windowPaddingY = partial.windowPaddingY;
      if (partial.cursorStyle !== undefined)
        req.cursorStyle = partial.cursorStyle;
      if (partial.cursorBlink !== undefined)
        req.cursorBlink = partial.cursorBlink;
      if (partial.unfocusedPaneOpacity !== undefined)
        req.unfocusedPaneOpacity = partial.unfocusedPaneOpacity;

      try {
        const updated = await invoke<AppSettings>("settings_update", { req });
        setSettings(updated);
        applyCssVars(updated);
      } catch {
        applyCssVars(settings);
      }
    },
  };
}
