import { createContext, useContext, type ParentComponent } from "solid-js";
import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

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

  onMount(async () => {
    let saved: Theme = "auto";
    try {
      const val = await invoke<string>("theme_get");
      if (val === "light" || val === "dark" || val === "auto") {
        saved = val;
      }
    } catch {
      // fallback to auto
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
  });

  const setTheme = (t: Theme) => {
    setThemeSignal(t);
    setResolved(applyTheme(t));
    invoke("theme_set", { theme: t }).catch(() => {});
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
