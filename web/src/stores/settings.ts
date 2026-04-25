// MVP-10 Phase A · AppSettings mock store
// Phase B 替换为 ts-rs binding + IPC invoke（settings_get / settings_update）

import { createStore } from "solid-js/store";

export type ThemeSetting = "light" | "dark" | "auto";

export interface AppSettings {
  theme: ThemeSetting;
  fontFamily: string;
  fontSize: number;
  defaultShell: string;
  pasteProtection: boolean;
  telemetryOptIn: boolean | null;
  gitUserName: string | null;
  gitUserEmail: string | null;
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme: "auto",
  fontFamily: "JetBrains Mono",
  fontSize: 14,
  defaultShell: "/bin/zsh",
  pasteProtection: true,
  telemetryOptIn: null,
  gitUserName: null,
  gitUserEmail: null,
};

const [settings, setSettings] = createStore<AppSettings>({
  ...DEFAULT_SETTINGS,
});

export function useSettings() {
  return {
    settings,
    updateSettings(partial: Partial<AppSettings>) {
      setSettings(partial);
      // Phase B: invoke('settings_update', partial) → Rust KV write → emit 'settings_changed'

      // E.5 · MVP-10 Font Family 设置实时覆盖 typography.css 默认
      if (partial.fontFamily !== undefined) {
        const fallback =
          'ui-monospace, "SF Mono", "Menlo", "Consolas", monospace';
        document.documentElement.style.setProperty(
          "--font-mono",
          `"${partial.fontFamily}", ${fallback}`,
        );
      }
    },
  };
}
