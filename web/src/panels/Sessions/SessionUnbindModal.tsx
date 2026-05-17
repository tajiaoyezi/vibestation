import { createSignal, onCleanup, onMount, type JSX } from "solid-js";
import type { SessionCommitLink } from "../../bindings";

interface SessionUnbindModalProps {
  link: SessionCommitLink;
  sessionTitle: string;
  onCancel: () => void;
  onUnbind: (reason: string) => Promise<void>;
  onUnbindAndRecalc: (reason: string) => Promise<void>;
}

export function SessionUnbindModal(
  props: SessionUnbindModalProps,
): JSX.Element {
  const [reason, setReason] = createSignal("manual correction");
  const [submitting, setSubmitting] = createSignal(false);
  let dialogRef: HTMLDivElement | undefined;

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
        'button:not(:disabled), input:not(:disabled), [tabindex]:not([tabindex="-1"])',
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

  const submit = async (recalc: boolean) => {
    setSubmitting(true);
    try {
      if (recalc) {
        await props.onUnbindAndRecalc(reason());
      } else {
        await props.onUnbind(reason());
      }
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-unbind-title"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onCancel();
      }}
    >
      <div
        class="vs-dialog vs-session-unbind-dialog"
        ref={dialogRef}
        tabIndex={-1}
      >
        <h3 id="vs-unbind-title" class="vs-dialog-title">
          Unbind commit from session
        </h3>
        <div class="vs-dialog-body">
          <p>
            Commit <code>{props.link.commitSha.slice(0, 8)}</code> will be
            soft-unlinked from session <strong>{props.sessionTitle}</strong>.
          </p>
          <p>
            Strategy: <code>{props.link.strategyVersion}</code> · Confidence:{" "}
            <strong>{(props.link.confidence * 100).toFixed(1)}%</strong>
          </p>
          <p style={{ color: "var(--warning)", "font-size": "0.75rem" }}>
            This action is auditable. The link record will be preserved in
            &quot;unlinked&quot; state for review.
          </p>
          <label class="vs-dialog-label">
            Reason
            <input
              class="vs-dialog-input"
              type="text"
              value={reason()}
              onInput={(e) => setReason(e.currentTarget.value)}
              placeholder="manual correction"
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
            class="vs-dialog-btn-danger"
            onClick={() => void submit(false)}
            disabled={submitting()}
            aria-label="Unbind commit"
          >
            {submitting() ? "Unbinding…" : "Unbind"}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            onClick={() => void submit(true)}
            disabled={submitting()}
            aria-label="Unbind and recalculate"
          >
            {submitting() ? "Processing…" : "Unbind & recalc"}
          </button>
        </div>
      </div>
    </div>
  );
}
