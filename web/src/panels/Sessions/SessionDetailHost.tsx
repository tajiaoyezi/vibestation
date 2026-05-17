import { Show, type JSX } from "solid-js";
import { useSessions } from "../../stores/sessions-context";
import { SessionDetailView } from "./SessionDetailView";

/**
 * MVP-19 §D.1 detail-view host (A.0.1 seam · Phase D body).
 *
 * A.0.1 (main agent) mounts this once inside `SessionsProvider` in `App.tsx`
 * so neither Phase C (Git Log badge) nor Phase D needs to touch `App.tsx`.
 * The C→D navigation seam is `sessions-context` (`activeDetail()` set by
 * Phase C's badge via `openDetail`, consumed here).
 */
export function SessionDetailHost(): JSX.Element {
  const ctx = useSessions();
  return (
    <Show when={ctx.activeDetail()}>
      <SessionDetailView />
    </Show>
  );
}
