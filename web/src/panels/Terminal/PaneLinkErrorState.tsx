/**
 * D.5 · Fallback / error state component.
 *
 * Shows recoverable link errors (from PaneLinkErrorEvent) as a dismissable banner.
 * Error state always includes text label (not pure color · H.* a11y).
 */
import { type Component, Show } from "solid-js";
import type { PaneLinkError } from "../../bindings";

export type PaneLinkErrorStateProps = {
  error: PaneLinkError | null;
  onDismiss?: () => void;
};

function errorMessage(error: PaneLinkError): string {
  switch (error.kind) {
    case "crossWorkspaceDenied":
      return "Cannot link panes across workspaces";
    case "invalidParentPaneType":
      return "Parent pane type is not supported for linking";
    case "invalidChildPaneType":
      return "Child pane type is not supported for linking";
    case "paneNotFound":
      return `Pane not found: ${error.detail}`;
    case "linkNotFound":
      return `Link not found: ${error.detail}`;
    case "parserUnavailable":
      return `Parser unavailable: ${error.detail}`;
    case "parserTimeout":
      return `Parser timed out: ${error.detail}`;
    case "promptSanitizationFailed":
      return `Prompt sanitization failed: ${error.detail}`;
    case "dbError":
      return `Database error: ${error.detail}`;
    case "unsupportedCliKind":
      return `Unsupported CLI: ${error.detail}`;
    default:
      return "An unexpected error occurred";
  }
}

function errorSeverity(error: PaneLinkError): "error" | "warning" {
  switch (error.kind) {
    case "parserTimeout":
    case "parserUnavailable":
      return "warning";
    default:
      return "error";
  }
}

export const PaneLinkErrorState: Component<PaneLinkErrorStateProps> = (
  props,
) => {
  return (
    <Show when={props.error}>
      {(error) => {
        const severity = () => errorSeverity(error());
        return (
          <div
            class={`vs-pane-link-error vs-pane-link-error-${severity()}`}
            role="alert"
            aria-label={`Link ${severity()}: ${errorMessage(error())}`}
          >
            <span class="vs-pane-link-error-icon" aria-hidden="true">
              {severity() === "error" ? "✕" : "⚠"}
            </span>
            <span class="vs-pane-link-error-message">
              {errorMessage(error())}
            </span>
            <button
              type="button"
              class="vs-pane-link-error-dismiss"
              aria-label="Dismiss error"
              onClick={() => props.onDismiss?.()}
            >
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <line
                  x1="4"
                  y1="4"
                  x2="12"
                  y2="12"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
                <line
                  x1="12"
                  y1="4"
                  x2="4"
                  y2="12"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            </button>
          </div>
        );
      }}
    </Show>
  );
};
