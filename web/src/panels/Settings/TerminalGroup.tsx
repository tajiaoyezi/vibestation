import { type Component } from "solid-js";
import { useSettings } from "../../stores/settings";

export const TerminalGroup: Component = () => {
  const { settings, updateSettings } = useSettings();

  const shells = [
    { value: "/bin/zsh", label: "zsh" },
    { value: "/bin/bash", label: "bash" },
    { value: "/usr/local/bin/fish", label: "fish" },
  ];

  return (
    <div class="vs-settings-fields">
      <label class="vs-settings-field">
        <span class="vs-settings-label">Default shell</span>
        <select
          class="vs-settings-select"
          value={settings.defaultShell}
          onChange={(e) =>
            updateSettings({ defaultShell: e.currentTarget.value })
          }
        >
          {shells.map((s) => (
            <option value={s.value}>{s.label}</option>
          ))}
        </select>
      </label>

      <label class="vs-settings-field vs-settings-field--row">
        <span class="vs-settings-label">Paste protection</span>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{ active: settings.pasteProtection }}
          onClick={() =>
            updateSettings({ pasteProtection: !settings.pasteProtection })
          }
          aria-pressed={settings.pasteProtection}
          role="switch"
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          Unfocused pane opacity{" "}
          <span class="vs-settings-value">
            {settings.unfocusedPaneOpacity.toFixed(2)}
          </span>
        </span>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={settings.unfocusedPaneOpacity}
          onInput={(e) =>
            updateSettings({
              unfocusedPaneOpacity: Number(e.currentTarget.value),
            })
          }
          class="vs-settings-slider"
        />
      </label>
    </div>
  );
};
