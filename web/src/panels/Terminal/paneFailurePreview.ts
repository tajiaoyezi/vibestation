import type { PaneFailureCallout } from "../../stores/paneLinks";
import type { PaneFailurePreviewRequest } from "../../bindings";

/**
 * MVP-18 Wave 2-3 · map a UI failure-callout summary to a `pane:failure:preview_prompt`
 * request.
 *
 * The stored `PaneFailureCallout` is intentionally minimal (spec §G.3 privacy —
 * it does not retain the full `parsedIssues` array, command, cwd, or cliKind).
 * The backend `previewPrompt` re-derives the sanitized `promptFragment` from
 * `rawOutput`, so leaving `command` / `cwd` / `cliKind` empty and
 * `parsedIssues` `[]` here is a *documented degradation* — the core sanitized
 * fragment still flows. Carrying richer command context is a tracked future
 * backend enrichment (same class as the `pane:link-error` escalate · see
 * `spike-tmp/review-prep/MVP-18-wave1-review-prep.md`), NOT fabricated data.
 */
export function buildPreviewRequest(
  callout: PaneFailureCallout,
): PaneFailurePreviewRequest {
  return {
    workspaceId: callout.workspaceId,
    childPaneId: callout.childPaneId,
    commandRunId: callout.commandRunId,
    exitCode: callout.exitCode,
    command: "",
    cwd: "",
    cliKind: "",
    rawOutput: callout.rawExcerpt,
    parsedIssues: [],
  };
}
