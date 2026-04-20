import { type Component, For } from "solid-js";
import { useLayout } from "../stores/layout-context";

export const ActivityStrip: Component = () => {
  const { layout, dispatch } = useLayout();

  type PanelId = "primary" | "secondary" | "bottom";

  const items: {
    id: PanelId;
    icon: string;
    label: string;
    shortcut: string;
  }[] = [
    { id: "primary", icon: "⊞", label: "Primary panel", shortcut: "⌘1" },
    { id: "secondary", icon: "⊟", label: "Secondary panel", shortcut: "⌘2" },
    { id: "bottom", icon: "◴", label: "Bottom panel", shortcut: "⌘J" },
  ];

  const isOpen = (id: PanelId): boolean => {
    const l = layout();
    switch (id) {
      case "primary":
        return l.primaryOpen;
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
      aria-label="Tool window toggles"
    >
      <For each={items}>
        {(item) => (
          <button
            type="button"
            class={`vs-as-btn${isOpen(item.id) ? " vs-as-btn-on" : ""}`}
            aria-label={item.label}
            aria-pressed={isOpen(item.id)}
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
