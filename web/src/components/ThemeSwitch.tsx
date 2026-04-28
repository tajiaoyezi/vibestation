import { type Component } from "solid-js";
import { useTheme } from "../stores/theme";
import { useSettings, type ThemeSetting } from "../stores/settings";

export const ThemeSwitch: Component = () => {
  const { theme, setTheme } = useTheme();
  const { updateSettings } = useSettings();

  const themes: { id: ThemeSetting; icon: string }[] = [
    { id: "light", icon: "☀" },
    { id: "dark", icon: "◑" },
    { id: "auto", icon: "⊘" },
  ];

  // setTheme 同时走两条 path · 消除 dual-path UI 不刷 bug：
  // - settings store updateSettings · IPC settings_update · 持久化 + emit settings_changed
  //   → applyCssVars 设 data-theme attr · UI 主题 token 实时切换（spec §F.02 实时生效）
  // - useTheme.setTheme · 同步更新 internal signal · ThemeSwitch active radio 即时反映
  //   （不等 IPC roundtrip · 防 click → 视觉延迟感）
  const handleClick = (id: ThemeSetting) => {
    setTheme(id);
    void updateSettings({ theme: id });
  };

  return (
    <div class="vs-theme-switch" role="radiogroup" aria-label="Theme selection">
      {themes.map((t) => (
        <button
          type="button"
          class={`vs-theme-btn${theme() === t.id ? " vs-theme-btn-on" : ""}`}
          data-theme={t.id}
          role="radio"
          aria-checked={theme() === t.id}
          aria-label={`${t.id} theme`}
          onClick={() => handleClick(t.id)}
        >
          {t.icon}
        </button>
      ))}
    </div>
  );
};
