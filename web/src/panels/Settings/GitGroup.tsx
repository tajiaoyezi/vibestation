import { type Component } from "solid-js";
import { useSettings } from "../../stores/settings";

export const GitGroup: Component = () => {
  const { settings, updateSettings } = useSettings();

  return (
    <div class="vs-settings-fields">
      <label class="vs-settings-field">
        <span class="vs-settings-label">User name</span>
        <input
          type="text"
          class="vs-settings-input"
          placeholder="From git config"
          value={settings.gitUserName ?? ""}
          onInput={(e) =>
            updateSettings({
              gitUserName: e.currentTarget.value || null,
            })
          }
        />
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">User email</span>
        <input
          type="text"
          class="vs-settings-input"
          placeholder="From git config"
          value={settings.gitUserEmail ?? ""}
          onInput={(e) =>
            updateSettings({
              gitUserEmail: e.currentTarget.value || null,
            })
          }
        />
      </label>
    </div>
  );
};
