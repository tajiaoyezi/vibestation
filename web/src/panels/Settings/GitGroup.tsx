import { type Component } from "solid-js";
import { useSettings } from "../../stores/settings";
import { t, normalizeLanguage } from "../../i18n";

export const GitGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  return (
    <div class="vs-settings-fields">
      <label class="vs-settings-field">
        <span class="vs-settings-label">{label("settings.git.userName")}</span>
        <input
          type="text"
          class="vs-settings-input"
          placeholder={label("settings.git.fromGitConfig")}
          value={settings.gitUserName ?? ""}
          onInput={(e) =>
            updateSettings({
              gitUserName: e.currentTarget.value || null,
            })
          }
        />
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">{label("settings.git.userEmail")}</span>
        <input
          type="text"
          class="vs-settings-input"
          placeholder={label("settings.git.fromGitConfig")}
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
