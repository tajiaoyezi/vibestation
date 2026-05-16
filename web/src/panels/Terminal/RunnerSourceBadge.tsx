/**
 * D.2 · Runner Pane source badge.
 *
 * Small badge shown on runner (child) pane indicating it's linked to a parent agent pane.
 * Shows the parent pane label and link kind.
 */
import { type Component, Show } from "solid-js";
import type { PaneLink } from "../../bindings";

export type RunnerSourceBadgeProps = {
  link: PaneLink;
  parentLabel?: string;
};

export const RunnerSourceBadge: Component<RunnerSourceBadgeProps> = (props) => {
  const label = () => props.parentLabel ?? "Agent";

  return (
    <Show when={props.link.enabled}>
      <span
        class="vs-runner-source-badge"
        role="status"
        aria-label={`Linked to ${label()} · failure feedback ${props.link.enabled ? "active" : "paused"}`}
      >
        <svg
          class="vs-runner-source-badge-icon"
          width="10"
          height="10"
          viewBox="0 0 16 16"
          fill="none"
          aria-hidden="true"
        >
          <circle cx="8" cy="8" r="3" fill="currentColor" />
        </svg>
        <span class="vs-runner-source-badge-text">← {label()}</span>
      </span>
    </Show>
  );
};
