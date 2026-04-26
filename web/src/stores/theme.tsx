import { createContext, useContext, type ParentComponent } from "solid-js";
import { createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppSettings } from "../bindings";

type Theme = "light" | "dark" | "auto";

interface ThemeContextValue {
  theme: () => Theme;
  setTheme: (t: Theme) => void;
  resolved: () => "light" | "dark";
}

const ThemeContext = createContext<ThemeContextValue>();

function applyTheme(theme: Theme) {
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const resolved = theme === "auto" ? (prefersDark ? "dark" : "light") : theme;
  // data-theme 由 settings store applyCssVars 唯一控制 · 避免 race condition
  // ThemeProvider 仅维护 resolved signal 给 useTheme() 调用方使用
  return resolved;
}

export const ThemeProvider: ParentComponent = (props) => {
  const [theme, setThemeSignal] = createSignal<Theme>("auto");
  const [resolved, setResolved] = createSignal<"light" | "dark">("dark");

  let unlistenSettings: UnlistenFn | undefined;

  onMount(async () => {
    let saved: Theme = "auto";
    try {
      // 首次启动 · 直接从 settings_get 拿（与 settings store 同源 · 避免 theme_get 双 IPC 漂移）
      const s = await invoke<AppSettings>("settings_get");
      if (s.theme === "light" || s.theme === "dark" || s.theme === "auto") {
        saved = s.theme;
      }
    } catch {
      // fallback to auto · backend pool 未 init / IPC 失败时
    }
    setThemeSignal(saved);
    setResolved(applyTheme(saved));

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => {
      if (theme() === "auto") {
        setResolved(applyTheme("auto"));
      }
    };
    mql.addEventListener("change", handler);

    // 监听 settings_changed event · 同步 theme signal（settings_update IPC 路径）
    // 修 dual-path bug：status bar ThemeSwitch click 走 updateSettings → emit
    // settings_changed → 这里 listen → ThemeProvider theme() signal 实时刷新
    // 同时 settings store applyCssVars 设 data-theme attr · UI realtime 切换
    unlistenSettings = await listen<AppSettings>(
      "settings_changed",
      (event) => {
        const t = event.payload.theme;
        if (t === "light" || t === "dark" || t === "auto") {
          setThemeSignal(t);
          setResolved(applyTheme(t));
        }
      },
    );
  });

  onCleanup(() => {
    unlistenSettings?.();
  });

  // setTheme · 仅更新 internal signal（同步反映 UI active state）
  // 持久化责任交给调用方调 useSettings.updateSettings({ theme }) · 走 settings_update IPC
  // 走 settings_changed event 回环再次同步 signal · UI 一致
  const setTheme = (t: Theme) => {
    setThemeSignal(t);
    setResolved(applyTheme(t));
  };

  return (
    <ThemeContext.Provider value={{ theme, setTheme, resolved }}>
      {props.children}
    </ThemeContext.Provider>
  );
};

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error("useTheme must be used within ThemeProvider");
  return ctx;
}
