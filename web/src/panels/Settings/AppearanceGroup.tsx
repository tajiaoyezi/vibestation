import { type Component, For } from "solid-js";
import { t, normalizeLanguage, type Language } from "../../i18n";
import {
  MIN_BG_OPACITY,
  useSettings,
  type ThemeSetting,
} from "../../stores/settings";

export const AppearanceGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const languages = (): { value: Language; label: string }[] => [
    { value: "en", label: label("settings.appearance.english") },
    {
      value: "zh-Hans",
      label: label("settings.appearance.simplifiedChinese"),
    },
  ];

  // updateSettings 走 settings_update IPC · emit settings_changed
  // → ThemeProvider listen 同步 internal signal · settings store applyCssVars
  // 设 data-theme attr · UI 主题 token 实时切换。不再需要 themeCtx.setTheme 双调
  // （previous dual-path bug 根因 · 见 PR #163）。
  const handleThemeChange = (theme: ThemeSetting) => {
    void updateSettings({ theme });
  };

  const themes = (): { value: ThemeSetting; label: string }[] => [
    { value: "auto", label: label("settings.appearance.auto") },
    { value: "light", label: label("settings.appearance.light") },
    { value: "dark", label: label("settings.appearance.dark") },
  ];

  const fonts = [
    "JetBrains Mono",
    "JetBrains Mono Variable",
    "JetBrainsMono NF",
    "Cascadia Code",
    "Fira Code",
    "SF Mono",
    "Consolas",
    "Monaco",
  ];

  const cursorStyles = (): { value: string; label: string }[] => [
    { value: "block", label: label("settings.appearance.cursorBlock") },
    { value: "bar", label: label("settings.appearance.cursorBar") },
    {
      value: "underline",
      label: label("settings.appearance.cursorUnderline"),
    },
  ];

  return (
    <div class="vs-settings-fields">
      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.language")}
        </span>
        <select
          class="vs-settings-select"
          value={language()}
          aria-label={label("settings.appearance.language")}
          onChange={(e) =>
            updateSettings({ language: e.currentTarget.value as Language })
          }
        >
          <For each={languages()}>
            {(item) => <option value={item.value}>{item.label}</option>}
          </For>
        </select>
      </label>

      <fieldset class="vs-settings-fieldset">
        <legend class="vs-settings-label">
          {label("settings.appearance.theme")}
        </legend>
        <div class="vs-settings-radio-row">
          <For each={themes()}>
            {(theme) => (
              <label class="vs-settings-radio-label">
                <input
                  type="radio"
                  name="theme"
                  value={theme.value}
                  checked={settings.theme === theme.value}
                  onChange={() => handleThemeChange(theme.value)}
                  class="vs-settings-radio"
                />
                <span>{theme.label}</span>
              </label>
            )}
          </For>
        </div>
      </fieldset>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.fontFamily")}
        </span>
        <select
          class="vs-settings-select"
          value={settings.fontFamily}
          onChange={(e) =>
            updateSettings({ fontFamily: e.currentTarget.value })
          }
        >
          <For each={fonts}>{(f) => <option value={f}>{f}</option>}</For>
        </select>
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.fontSize")}{" "}
          <span class="vs-settings-value">{settings.fontSize}px</span>
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

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.backgroundOpacity")}{" "}
          <span class="vs-settings-value">{settings.bgOpacity.toFixed(2)}</span>
        </span>
        <input
          type="range"
          min={MIN_BG_OPACITY}
          max={1}
          step={0.05}
          value={settings.bgOpacity}
          onInput={(e) =>
            updateSettings({ bgOpacity: Number(e.currentTarget.value) })
          }
          class="vs-settings-slider"
        />
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.backgroundBlur")}{" "}
          <span class="vs-settings-value">{settings.bgBlur}px</span>
        </span>
        <input
          type="number"
          min={0}
          max={100}
          step={1}
          value={settings.bgBlur}
          onChange={(e) =>
            updateSettings({ bgBlur: Number(e.currentTarget.value) })
          }
          class="vs-settings-input vs-settings-input--narrow"
        />
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.windowPaddingX")}{" "}
          <span class="vs-settings-value">{settings.windowPaddingX}px</span>
        </span>
        <input
          type="number"
          min={0}
          max={20}
          step={1}
          value={settings.windowPaddingX}
          onChange={(e) =>
            updateSettings({ windowPaddingX: Number(e.currentTarget.value) })
          }
          class="vs-settings-input vs-settings-input--narrow"
        />
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.appearance.windowPaddingY")}{" "}
          <span class="vs-settings-value">{settings.windowPaddingY}px</span>
        </span>
        <input
          type="number"
          min={0}
          max={20}
          step={1}
          value={settings.windowPaddingY}
          onChange={(e) =>
            updateSettings({ windowPaddingY: Number(e.currentTarget.value) })
          }
          class="vs-settings-input vs-settings-input--narrow"
        />
      </label>

      <fieldset class="vs-settings-fieldset">
        <legend class="vs-settings-label">
          {label("settings.appearance.cursorStyle")}
        </legend>
        <div class="vs-settings-radio-row">
          <For each={cursorStyles()}>
            {(cs) => (
              <label class="vs-settings-radio-label">
                <input
                  type="radio"
                  name="cursorStyle"
                  value={cs.value}
                  checked={settings.cursorStyle === cs.value}
                  onChange={() => updateSettings({ cursorStyle: cs.value })}
                  class="vs-settings-radio"
                />
                <span>{cs.label}</span>
              </label>
            )}
          </For>
        </div>
      </fieldset>

      <label class="vs-settings-field vs-settings-field--row">
        <span class="vs-settings-label">
          {label("settings.appearance.cursorBlink")}
        </span>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{ active: settings.cursorBlink }}
          onClick={() => updateSettings({ cursorBlink: !settings.cursorBlink })}
          aria-pressed={settings.cursorBlink}
          role="switch"
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </label>
    </div>
  );
};
