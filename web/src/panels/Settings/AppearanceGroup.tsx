import { type Component } from "solid-js";
import { useSettings, type ThemeSetting } from "../../stores/settings";
import { useTheme } from "../../stores/theme";

export const AppearanceGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const themeCtx = useTheme();

  const handleThemeChange = (theme: ThemeSetting) => {
    updateSettings({ theme });
    // 实时生效：同步到全局 ThemeProvider
    themeCtx.setTheme(theme);
  };

  const themes: { value: ThemeSetting; label: string }[] = [
    { value: "auto", label: "Auto" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];

  const fonts = [
    "JetBrains Mono",
    "Fira Code",
    "SF Mono",
    "Consolas",
    "Monaco",
  ];

  return (
    <div class="vs-settings-fields">
      <fieldset class="vs-settings-fieldset">
        <legend class="vs-settings-label">Theme</legend>
        <div class="vs-settings-radio-row">
          {themes.map((t) => (
            <label class="vs-settings-radio-label">
              <input
                type="radio"
                name="theme"
                value={t.value}
                checked={settings.theme === t.value}
                onChange={() => handleThemeChange(t.value)}
                class="vs-settings-radio"
              />
              <span>{t.label}</span>
            </label>
          ))}
        </div>
      </fieldset>

      <label class="vs-settings-field">
        <span class="vs-settings-label">Font family</span>
        <select
          class="vs-settings-select"
          value={settings.fontFamily}
          onChange={(e) =>
            updateSettings({ fontFamily: e.currentTarget.value })
          }
        >
          {fonts.map((f) => (
            <option value={f}>{f}</option>
          ))}
        </select>
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          Font size <span class="vs-settings-value">{settings.fontSize}px</span>
        </span>
        <input
          type="range"
          min={10}
          max={24}
          step={1}
          value={settings.fontSize}
          onInput={(e) =>
            updateSettings({ fontSize: Number(e.currentTarget.value) })
          }
          class="vs-settings-slider"
        />
      </label>
    </div>
  );
};
