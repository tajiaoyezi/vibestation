/**
 * MVP-19 Phase C · §D.2 · Session badge for Git Log commit rows.
 *
 * Consumes the shared A.0 `SessionsContext` via `useSessions()` — never
 * re-invents navigation or store shape (HC-2 / #353 lesson).
 *
 * States (§D.2):
 *   confirmed — link is confirmedAuto or confirmedManual; full-opacity badge.
 *   pending   — link exists but not yet confirmed; weakened style + ◌ icon.
 *   stale     — link exists but the session record is missing; ⚠ icon.
 *
 * Reduced-motion degradation (HC-4) is handled entirely by CSS
 * via `@media (prefers-reduced-motion: reduce)`.
 */
import { type Component, createMemo, Show } from "solid-js";
import { useSessions } from "../../stores/sessions-context";
import { isLinkConfirmed } from "../../stores/sessions";
import "./sessionBadge.css";

export interface SessionBadgeProps {
  /** Short SHA of the commit whose session links to display. */
  commitSha: string;
  /** Reactive accessor for the current workspace id (§D.2 workspace-scoped). */
  workspaceId: () => string;
}

type BadgeStatus = "confirmed" | "pending" | "stale";

export const SessionBadge: Component<SessionBadgeProps> = (props) => {
  // HC-2: consume A.0 shared context; never duplicate navigation or store shape.
  const { selectorsFor, openDetail } = useSessions();

  // createWorkspaceScopedSelectors creates memos — called once per component
  // instance in the SolidJS component body (runs once, not on each render).
  const selectors = selectorsFor(props.workspaceId);

  // §D.2 (a): at most one primary badge per commit row.
  const primaryLink = createMemo(() =>
    selectors.primaryLinkForCommit(props.commitSha),
  );

  // §D.2 (b): count all links to compute the +N secondary marker.
  const allLinks = createMemo(() => selectors.linksForCommit(props.commitSha));

  // §D.2 (f): session missing → stale style.
  const session = createMemo(() => {
    const link = primaryLink();
    return link ? selectors.sessionById(link.sessionId) : undefined;
  });

  const badgeStatus = createMemo((): BadgeStatus => {
    const link = primaryLink();
    if (!link) return "pending"; // guarded by outer Show; unreachable at render
    if (!session()) return "stale"; // §D.2 (f)
    return isLinkConfirmed(link) ? "confirmed" : "pending"; // §D.2 (c)
  });

  // §D.2 (b): number of non-primary links visible as +N.
  const secondaryCount = createMemo(() =>
    Math.max(0, allLinks().length - (primaryLink() ? 1 : 0)),
  );

  // HC-4: aria-label carries title + status + confidence (colour is not sole indicator).
  const ariaLabel = createMemo(() => {
    const link = primaryLink();
    if (!link) return "";
    const title = session()?.title ?? "unknown session";
    const pct = Math.round(link.confidence * 100);
    return `Session: ${title} · ${badgeStatus()} · confidence ${pct}%`;
  });

  // §D.2 (d): hover tooltip — title + time + confidence + source.
  const tooltipText = createMemo(() => {
    const link = primaryLink();
    if (!link) return "";
    const title = session()?.title ?? "Session not found";
    const pct = Math.round(link.confidence * 100);
    const reason = link.confidenceReason;
    const time = link.linkedAt
      ? new Date(link.linkedAt).toLocaleString()
      : "unknown";
    return `${title}\nLinked: ${time}\nConfidence: ${pct}%\nSource: ${reason}`;
  });

  // §D.2 (e): click → A.0 shared openDetail (HC-2: no self-invented navigation).
  const handleClick = () => {
    const link = primaryLink();
    if (link) openDetail(link.sessionId, props.commitSha);
  };

  // HC-4: keyboard reachability — Enter / Space both trigger openDetail.
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      handleClick();
    }
  };

  // §D.2 (g): commit not judged (no primary link) → render nothing.
  return (
    <Show when={primaryLink()}>
      <span class="vs-session-badges">
        {/*
         * HC-4: <button> for keyboard reachability (Tab focus + Enter/Space);
         * aria-label for screen readers; colour is not the sole state indicator
         * (icon present for pending/stale).
         */}
        <button
          type="button"
          class={`vs-session-badge vs-session-badge--${badgeStatus()}`}
          aria-label={ariaLabel()}
          title={tooltipText()}
          onClick={handleClick}
          onKeyDown={handleKeyDown}
        >
          {/* HC-4: non-colour icon for pending state */}
          <Show when={badgeStatus() === "pending"}>
            <span class="vs-session-badge__icon" aria-hidden="true">
              ◌
            </span>
          </Show>
          {/* HC-4: non-colour icon for stale state */}
          <Show when={badgeStatus() === "stale"}>
            <span class="vs-session-badge__icon" aria-hidden="true">
              ⚠
            </span>
          </Show>
          <span class="vs-session-badge__label">{session()?.title ?? "…"}</span>
        </button>
        {/* §D.2 (b): +N secondary marker for additional link candidates */}
        <Show when={secondaryCount() > 0}>
          <span
            class="vs-session-badge__secondary"
            aria-label={`${secondaryCount()} more session links`}
          >
            +{secondaryCount()}
          </span>
        </Show>
      </span>
    </Show>
  );
};
