import { createSignal, For, Show, type Component } from "solid-js";
import { AppearanceGroup } from "./AppearanceGroup";
import { TerminalGroup } from "./TerminalGroup";
import { GitGroup } from "./GitGroup";
import { PrivacyGroup } from "./PrivacyGroup";
import "./styles.css";

interface SettingsPanelProps {
  visible: boolean;
  onClose: () => void;
}

type GroupDef = {
  id: string;
  title: string;
  component: Component;
};

const GROUPS: GroupDef[] = [
  { id: "appearance", title: "Appearance", component: AppearanceGroup },
  { id: "terminal", title: "Terminal", component: TerminalGroup },
  { id: "git", title: "Git", component: GitGroup },
  { id: "privacy", title: "Privacy", component: PrivacyGroup },
];

export const SettingsPanel: Component<SettingsPanelProps> = (props) => {
  const [expanded, setExpanded] = createSignal<Record<string, boolean>>({
    appearance: true,
    terminal: true,
    git: true,
    privacy: true,
  });

  const toggleGroup = (id: string) => {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  return (
    <Show when={props.visible}>
      <div
        class="vs-settings-overlay"
        role="dialog"
        aria-modal="true"
        aria-label="Preferences"
        onClick={props.onClose}
      >
        <div class="vs-settings-drawer" onClick={(e) => e.stopPropagation()}>
          <div class="vs-settings-header">
            <h2 class="vs-settings-title">Preferences</h2>
            <button
              type="button"
              class="vs-settings-close"
              onClick={props.onClose}
              aria-label="Close settings"
            >
              ✕
            </button>
          </div>

          <div class="vs-settings-body">
            <For each={GROUPS}>
              {(group) => (
                <div class="vs-settings-group">
                  <button
                    type="button"
                    class="vs-settings-group-header"
                    onClick={() => toggleGroup(group.id)}
                    aria-expanded={expanded()[group.id]}
                  >
                    <span
                      class="vs-settings-group-chevron"
                      classList={{ expanded: expanded()[group.id] }}
                    >
                      ▼
                    </span>
                    <span class="vs-settings-group-title">{group.title}</span>
                  </button>
                  <Show when={expanded()[group.id]}>
                    <div class="vs-settings-group-content">
                      <group.component />
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </Show>
  );
};
