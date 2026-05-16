/**
 * D.1 · Pane header link affordance chip.
 *
 * Shows link status on the pane header · parent side shows "→ child" · child side
 * shows "← parent". Clicking opens the link management popover (D.4).
 */
import { type Component, Show, createMemo } from "solid-js";
import type { PaneLinkView } from "../../stores/paneLinks";

export type PaneLinkChipRole = "parent" | "child";

export type PaneLinkChipProps = {
  link: PaneLinkView;
  role: PaneLinkChipRole;
  targetPaneLabel?: string;
  onClick?: () => void;
};

const statusLabel = (link: PaneLinkView): string => {
  if (link.status !== "enabled") return "paused";
  return "linked";
};

export const PaneLinkChip: Component<PaneLinkChipProps> = (props) => {
  const chipLabel = createMemo(() => {
    const direction = props.role === "parent" ? "→" : "←";
    const target =
      props.targetPaneLabel ?? (props.role === "parent" ? "runner" : "agent");
    return `${direction} ${target}`;
  });

  const ariaLabel = createMemo(() => {
    const role = props.role;
    const status = statusLabel(props.link);
    const target =
      props.targetPaneLabel ?? (role === "parent" ? "runner" : "agent");
    return `Pane link: ${role} ${status} · ${target}`;
  });

  return (
    <button
      type="button"
      class={`vs-pane-link-chip ${props.link.status !== "enabled" ? "is-paused" : ""}`}
      aria-label={ariaLabel()}
      onClick={(e) => {
        e.stopPropagation();
        props.onClick?.();
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          e.stopPropagation();
          props.onClick?.();
        }
      }}
    >
      <svg
        class="vs-pane-link-chip-icon"
        width="12"
        height="12"
        viewBox="0 0 16 16"
        fill="none"
        aria-hidden="true"
      >
        <path
          d="M6.5 3.5L10 8L6.5 12.5"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
      <span class="vs-pane-link-chip-text">{chipLabel()}</span>
      <Show when={props.link.status !== "enabled"}>
        <span class="vs-pane-link-chip-badge" aria-hidden="true">
          paused
        </span>
      </Show>
    </button>
  );
};
