import { createSignal } from "solid-js";
import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppSettings, SettingsUpdateRequest } from "../bindings";

export type ThemeSetting = "light" | "dark" | "auto";

const DEFAULTS: AppSettings = {
  language: "en",
  theme: "dark",
  fontFamily:
    "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, ui-monospace, Liberation Mono, monospace",
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
  // MVP-20 · PTY 预热池
  ptyPoolEnabled: true,
  ptyPoolSize: 1,
  // 全局 sidebar 尺寸 · 跨 workspace 共享 · 与 DEFAULT_LAYOUT 对齐
  primaryWidth: 236,
  secondaryWidth: 400,
  bottomHeight: 240,
  // MVP-17 · External Terminal preference
  externalTermPreferred: null,
  externalTermDontAskAgain: false,
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

  // 用户选的字体放栈首 · 后接 bundled JetBrains Mono Variable + Nerd Font + 系统等宽兜底
  // （选 "JetBrains Mono" 不匹配 bundled "JetBrains Mono Variable" 时由此兜底渲染）
  const fallback =
    '"JetBrains Mono Variable", "JetBrainsMono NF", "JetBrains Mono", ui-monospace, "Cascadia Code", "SF Mono", "Consolas", monospace';
  root.setProperty("--font-mono", `"${s.fontFamily}", ${fallback}`);

  // 同步 data-theme attribute · 避免 ThemeProvider race · settings_changed 一并生效
  const resolvedTheme =
    s.theme === "auto"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : s.theme;
  document.documentElement.dataset.theme = resolvedTheme;
  syncWindowTheme(resolvedTheme);
}

/// 同步 Tauri 原生窗口主题，让 Windows 原生标题栏跟随 app 暗/亮主题。
///
/// Windows 用原生窗口装饰（标题栏 + 最小化/最大化/关闭），其配色默认不跟随 webview 内
/// 的 `data-theme` → 暗色 app 下标题栏仍是白色那一行。`setTheme` 让 Tauri 应用
/// immersive dark mode 修正之。macOS 走 `titleBarStyle: Overlay` 无原生标题栏，此调用
/// 幂等无害。非 Tauri 上下文（vitest jsdom）静默忽略。
function syncWindowTheme(theme: string): void {
  const resolved = theme === "dark" ? "dark" : "light";
  try {
    void getCurrentWindow()
      .setTheme(resolved)
      .catch(() => {});
  } catch {
    // 非 Tauri 上下文 · 忽略
  }
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
      if (partial.ptyPoolEnabled !== undefined)
        req.ptyPoolEnabled = partial.ptyPoolEnabled;
      if (partial.ptyPoolSize !== undefined)
        req.ptyPoolSize = partial.ptyPoolSize;
      if (partial.externalTermPreferred !== undefined)
        req.externalTermPreferred = partial.externalTermPreferred;
      if (partial.externalTermDontAskAgain !== undefined)
        req.externalTermDontAskAgain = partial.externalTermDontAskAgain;

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
