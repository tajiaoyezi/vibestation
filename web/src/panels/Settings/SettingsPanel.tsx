import { createSignal, For, Show, type Component } from "solid-js";
import { AppearanceGroup } from "./AppearanceGroup";
import { ExternalTerminalGroup } from "./ExternalTerminalGroup";
import { TerminalGroup } from "./TerminalGroup";
import { GitGroup } from "./GitGroup";
import { PrivacyGroup } from "./PrivacyGroup";
import { t, normalizeLanguage } from "../../i18n";
import { useSettings } from "../../stores/settings";
import "./styles.css";

interface SettingsPanelProps {
  visible: boolean;
  onClose: () => void;
  /** MVP-06 · 触发配置导入对话框（Settings 头部 "Import" 按钮） */
  onOpenImport?: () => void;
}

type GroupDef = {
  id: string;
  titleKey: string;
  component: Component;
};

const GROUPS: GroupDef[] = [
  {
    id: "appearance",
    titleKey: "settings.groups.appearance",
    component: AppearanceGroup,
  },
  {
    id: "terminal",
    titleKey: "settings.groups.terminal",
    component: TerminalGroup,
  },
  {
    id: "external-terminal",
    titleKey: "settings.groups.externalTerminal",
    component: ExternalTerminalGroup,
  },
  { id: "git", titleKey: "settings.groups.git", component: GitGroup },
  {
    id: "privacy",
    titleKey: "settings.groups.privacy",
    component: PrivacyGroup,
  },
];

export const SettingsPanel: Component<SettingsPanelProps> = (props) => {
  const { settings } = useSettings();
  const [expanded, setExpanded] = createSignal<Record<string, boolean>>({
    appearance: true,
    terminal: true,
    "external-terminal": false,
    git: true,
    privacy: true,
  });

  const toggleGroup = (id: string) => {
    setExpanded((prev) => ({ ...prev, [id]: !prev[id] }));
  };
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  return (
    <Show when={props.visible}>
      <div
        class="vs-settings-overlay"
        role="dialog"
        aria-modal="true"
        aria-label={label("settings.title")}
        onClick={props.onClose}
      >
        <div class="vs-settings-drawer" onClick={(e) => e.stopPropagation()}>
          <div class="vs-settings-header">
            <h2 class="vs-settings-title">{label("settings.title")}</h2>
            <div class="vs-settings-header-actions">
              <Show when={props.onOpenImport}>
                <button
                  type="button"
                  class="vs-settings-import-btn"
                  onClick={() => {
                    props.onOpenImport?.();
                  }}
                >
                  {label("settings.import")}
                </button>
              </Show>
              <button
                type="button"
                class="vs-settings-close"
                onClick={props.onClose}
                aria-label={label("settings.close")}
              >
                ✕
              </button>
            </div>
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
                    aria-label={label(group.titleKey)}
                  >
                    <span
                      class="vs-settings-group-chevron"
                      classList={{ expanded: expanded()[group.id] }}
                    >
                      ▼
                    </span>
                    <span class="vs-settings-group-title">
                      {label(group.titleKey)}
                    </span>
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
