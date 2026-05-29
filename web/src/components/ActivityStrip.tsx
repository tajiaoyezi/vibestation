import { type Component, For } from "solid-js";
import { useLayout } from "../stores/layout-context";
import { formatShortcut } from "../lib/format-shortcut";

export const ActivityStrip: Component = () => {
  const { layout, dispatch } = useLayout();

  // primary sidebar 已由左上角 TopBar 按钮（⌘B）负责 toggle · 此处不再重复入口
  type PanelId = "secondary" | "bottom";

  const items: {
    id: PanelId;
    icon: string;
    label: string;
    shortcut: string;
  }[] = [
    {
      id: "secondary",
      icon: "⊟",
      label: "Git Log",
      shortcut: formatShortcut("⌘2", "Ctrl+2"),
    },
    {
      id: "bottom",
      icon: "◴",
      label: "Git Status",
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
    <aside class="vs-activity-strip" role="toolbar" aria-label="Panel toggles">
      <For each={items}>
        {(item) => (
          <button
            type="button"
            class={`vs-as-btn${isOpen(item.id) ? " vs-as-btn-on" : ""}`}
            aria-label={item.label}
            aria-pressed={isOpen(item.id)}
            title={`${item.label} (${item.shortcut})`}
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
