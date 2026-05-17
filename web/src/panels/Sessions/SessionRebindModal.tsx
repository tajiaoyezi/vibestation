import {
  createResource,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from "solid-js";
import type { SessionCommitLink } from "../../bindings";
import { useSessions } from "../../stores/sessions-context";

interface SessionRebindModalProps {
  link: SessionCommitLink;
  workspaceId: string;
  currentSessionId: string;
  onCancel: () => void;
  onRebind: (targetSessionId: string, reason: string) => Promise<void>;
}

export function SessionRebindModal(
  props: SessionRebindModalProps,
): JSX.Element {
  const ctx = useSessions();
  const [targetId, setTargetId] = createSignal("");
  const [reason, setReason] = createSignal("");
  const [submitting, setSubmitting] = createSignal(false);
  let dialogRef: HTMLDivElement | undefined;

  const [sessions] = createResource(
    () => ({ workspaceId: props.workspaceId }),
    (req) => ctx.list(req),
  );

  const availableSessions = () =>
    (sessions()?.sessions ?? []).filter(
      (s) => s.id !== props.currentSessionId && s.status !== "archived",
    );

  onMount(() => {
    dialogRef?.focus();
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      props.onCancel();
    }
    if (e.key === "Tab" && dialogRef) {
      const focusables = dialogRef.querySelectorAll<HTMLElement>(
        'button:not(:disabled), select:not(:disabled), input:not(:disabled), [tabindex]:not([tabindex="-1"])',
      );
      if (focusables.length === 0) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  };

  onMount(() => document.addEventListener("keydown", handleKeyDown));
  onCleanup(() => document.removeEventListener("keydown", handleKeyDown));

  const submit = async () => {
    const id = targetId();
    if (!id) return;
    setSubmitting(true);
    try {
      await props.onRebind(id, reason());
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-rebind-title"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onCancel();
      }}
    >
      <div
        class="vs-dialog vs-session-rebind-dialog"
        ref={dialogRef}
        tabIndex={-1}
      >
        <h3 id="vs-rebind-title" class="vs-dialog-title">
          Rebind commit to another session
        </h3>
        <div class="vs-dialog-body">
          <p>
            Commit <code>{props.link.commitSha.slice(0, 8)}</code> will be
            reassigned. The current link becomes <strong>superseded</strong>.
          </p>
          <label class="vs-dialog-label">
            Target session
            <Show
              when={!sessions.loading}
              fallback={<span>Loading sessions…</span>}
            >
              <select
                class="vs-dialog-input"
                value={targetId()}
                onChange={(e) => setTargetId(e.currentTarget.value)}
                aria-label="Select target session"
              >
                <option value="">— Select —</option>
                <For each={availableSessions()}>
                  {(s) => (
                    <option value={s.id}>
                      {s.title} ({s.cliKind} · {s.status})
                    </option>
                  )}
                </For>
              </select>
            </Show>
          </label>
          <label class="vs-dialog-label">
            Reason (optional)
            <input
              class="vs-dialog-input"
              type="text"
              value={reason()}
              onInput={(e) => setReason(e.currentTarget.value)}
              placeholder="reassignment"
            />
          </label>
        </div>
        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
            disabled={submitting()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            onClick={() => void submit()}
            disabled={submitting() || !targetId()}
            aria-label="Rebind commit"
          >
            {submitting() ? "Rebinding…" : "Rebind"}
          </button>
        </div>
      </div>
    </div>
  );
}
