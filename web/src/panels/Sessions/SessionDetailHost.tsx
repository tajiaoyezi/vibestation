import { Show, type JSX } from "solid-js";
import { useSessions } from "../../stores/sessions-context";

/**
 * MVP-19 §D.1 detail-view host (A.0.1 seam stub · Phase D owns the body).
 *
 * A.0.1 (main agent) mounts this once inside `SessionsProvider` in `App.tsx`
 * so neither Phase C (Git Log badge) nor Phase D needs to touch `App.tsx`
 * (§2.16 shared-touch-point owned by the A.0 wave). The C→D navigation seam
 * is the merged `sessions-context` (`activeDetail()` set by Phase C's badge
 * via `openDetail`, consumed here).
 *
 * Phase D replaces the placeholder body with the real Session detail view
 * (§D.1 layout + commit list/timeline + §D.3 unbind/rebind modals + error
 * states). Keep this file in `panels/Sessions/` (Phase D file domain) and do
 * NOT re-touch `App.tsx` — the mount + provider are already wired here.
 */
export function SessionDetailHost(): JSX.Element {
  const { activeDetail } = useSessions();
  return (
    <Show when={activeDetail()}>
      {/* TODO(MVP-19 Phase D · Cursor): render Session 详情视图 here —
          §D.1 header/summary/timeline/commit panel + §D.3 unbind/rebind
          modal + §D 错误态. Uses useSessions() selectors + getDetail/
          unbind/rebind + closeDetail. Placeholder renders nothing. */}
      {null}
    </Show>
  );
}
