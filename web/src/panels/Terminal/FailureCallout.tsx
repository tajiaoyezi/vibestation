/**
 * D.3 · Failure feedback callout.
 *
 * Renders a failure summary with dismiss / view raw / insert-to-agent actions.
 * Data is mock/placeholder until Phase C wires real PaneBuildFailedEvent.
 */
import { type Component, Show, createSignal } from "solid-js";
import type { PaneFailureCallout } from "../../stores/paneLinks";

export type FailureCalloutProps = {
  callout: PaneFailureCallout;
  onDismiss?: (commandRunId: string) => void;
  onViewRaw?: (rawExcerpt: string) => void;
  onInsert?: (callout: PaneFailureCallout) => void;
};

export const FailureCallout: Component<FailureCalloutProps> = (props) => {
  const [expanded, setExpanded] = createSignal(false);

  const exitLabel = () =>
    props.callout.exitCode !== null
      ? `exit ${props.callout.exitCode}`
      : "signal";

  const confidenceLabel = () => {
    const pct = Math.round(props.callout.parserConfidence * 100);
    return `${pct}% confidence`;
  };

  return (
    <div
      class="vs-failure-callout"
      role="alert"
      aria-label={`Build failure: ${exitLabel()} · ${props.callout.parsedIssuesCount} issue${props.callout.parsedIssuesCount !== 1 ? "s" : ""}`}
    >
      <div class="vs-failure-callout-header">
        <span class="vs-failure-callout-icon" aria-hidden="true">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <circle
              cx="8"
              cy="8"
              r="6.5"
              stroke="currentColor"
              stroke-width="1.3"
            />
            <line
              x1="8"
              y1="5"
              x2="8"
              y2="9"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
            <circle cx="8" cy="11.5" r="0.8" fill="currentColor" />
          </svg>
        </span>
        <span class="vs-failure-callout-title">
          Failure · {exitLabel()}
          <Show when={props.callout.parsedIssuesCount > 0}>
            {" "}
            · {props.callout.parsedIssuesCount} issue
            {props.callout.parsedIssuesCount !== 1 ? "s" : ""}
          </Show>
        </span>
        <span class="vs-failure-callout-confidence">{confidenceLabel()}</span>
      </div>

      <Show when={expanded()}>
        <pre class="vs-failure-callout-raw">{props.callout.rawExcerpt}</pre>
      </Show>

      <div class="vs-failure-callout-actions">
        <button
          type="button"
          class="vs-failure-callout-btn"
          aria-label="Dismiss failure notification"
          onClick={() => props.onDismiss?.(props.callout.commandRunId)}
        >
          Dismiss
        </button>
        <button
          type="button"
          class="vs-failure-callout-btn"
          aria-label={expanded() ? "Hide raw output" : "View raw output"}
          onClick={() => setExpanded((prev) => !prev)}
        >
          {expanded() ? "Hide raw" : "View raw"}
        </button>
        <button
          type="button"
          class="vs-failure-callout-btn vs-failure-callout-btn-primary"
          aria-label="Insert failure details to agent pane"
          onClick={() => props.onInsert?.(props.callout)}
        >
          Insert to agent
        </button>
      </div>
    </div>
  );
};
