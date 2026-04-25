import { type Component, For } from "solid-js";
import { useSettings, type ThemeSetting } from "../../stores/settings";
import { useTheme } from "../../stores/theme";

export const AppearanceGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const themeCtx = useTheme();

  const handleThemeChange = (theme: ThemeSetting) => {
    updateSettings({ theme });
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

  const cursorStyles: { value: string; label: string }[] = [
    { value: "block", label: "Block" },
    { value: "bar", label: "Bar" },
    { value: "underline", label: "Underline" },
  ];

  return (
    <div class="vs-settings-fields">
      <fieldset class="vs-settings-fieldset">
        <legend class="vs-settings-label">Theme</legend>
        <div class="vs-settings-radio-row">
          <For each={themes}>
            {(t) => (
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
            )}
          </For>
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
          <For each={fonts}>{(f) => <option value={f}>{f}</option>}</For>
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

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          Background opacity{" "}
          <span class="vs-settings-value">{settings.bgOpacity.toFixed(2)}</span>
        </span>
        <input
          type="range"
          min={0}
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
          Background blur{" "}
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
          Window padding X{" "}
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
          Window padding Y{" "}
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
        <legend class="vs-settings-label">Cursor style</legend>
        <div class="vs-settings-radio-row">
          <For each={cursorStyles}>
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
        <span class="vs-settings-label">Cursor blink</span>
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
