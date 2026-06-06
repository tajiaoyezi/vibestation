import { type Component, For } from "solid-js";
import { t, normalizeLanguage } from "../i18n";
import { useLayout } from "../stores/layout-context";
import { useSettings } from "../stores/settings";
import { formatShortcut } from "../lib/format-shortcut";

export const ActivityStrip: Component = () => {
  const { layout, dispatch } = useLayout();
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  // primary sidebar 已由左上角 TopBar 按钮（⌘B）负责 toggle · 此处不再重复入口
  type PanelId = "secondary" | "bottom";

  const items: {
    id: PanelId;
    icon: string;
    labelKey: string;
    shortcut: string;
  }[] = [
    {
      id: "secondary",
      icon: "⊟",
      labelKey: "chrome.activity.gitLog",
      shortcut: formatShortcut("⌘2", "Ctrl+2"),
    },
    {
      id: "bottom",
      icon: "◴",
      labelKey: "chrome.activity.gitStatus",
      shortcut: formatShortcut("⌘J", "Ctrl+J"),
    },
  ];

  const isOpen = (id: PanelId): boolean => {
    const l = layout();
    switch (id) {
      case "secondary":
        return l.secondaryOpen;
      case "bottom":
        return l.bottomOpen;
    }
  };

  return (
    <aside
      class="vs-activity-strip"
      role="toolbar"
      aria-label={label("chrome.activity.panelToggles")}
    >
      <For each={items}>
        {(item) => (
          <button
            type="button"
            class={`vs-as-btn${isOpen(item.id) ? " vs-as-btn-on" : ""}`}
            aria-label={label(item.labelKey)}
            aria-pressed={isOpen(item.id)}
            title={`${label(item.labelKey)} (${item.shortcut})`}
            onClick={() => dispatch({ kind: `toggle-${item.id}` as const })}
          >
            <span aria-hidden="true">{item.icon}</span>
            <span class="vs-kbd-hint">{item.shortcut}</span>
          </button>
        )}
      </For>
      <div class="vs-as-spacer" />
    </aside>
  );
};
