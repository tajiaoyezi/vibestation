/**
 * D.2 / D.5 · AI draft-buffer composer.
 *
 * Textarea bound to {@link PaneDraftsStore} with merge-preview confirm region.
 * The merge-confirm UI is driven by the store's internal pending state:
 * Wave 2 calls `drafts.insertFragment(paneId, fragment)` → store sets pending →
 * component reactively renders the confirm area. No extra props needed.
 *
 * Wave 2 mounts this inside PaneTerminal; this PR delivers the isolated component.
 */
import { type Component, Show } from "solid-js";
import type {
  PaneDraftsStore,
  PaneDraftsStoreWithMergeState,
} from "../../stores/paneDrafts";

export type PaneDraftComposerProps = {
  paneId: string;
  drafts: PaneDraftsStore;
  onSend: (paneId: string, text: string) => void;
};

function hasMergeState(
  store: PaneDraftsStore,
): store is PaneDraftsStoreWithMergeState {
  return (
    "getPendingMerge" in store &&
    typeof (store as PaneDraftsStoreWithMergeState).getPendingMerge ===
      "function"
  );
}

export const PaneDraftComposer: Component<PaneDraftComposerProps> = (props) => {
  const draft = () => props.drafts.getDraft(props.paneId);
  const isEmpty = () => draft() === "";

  const pendingMerge = () => {
    if (hasMergeState(props.drafts)) {
      return props.drafts.getPendingMerge(props.paneId);
    }
    return null;
  };

  const clearPending = () => {
    if (hasMergeState(props.drafts)) {
      props.drafts.clearPendingMerge(props.paneId);
    }
  };

  const handleInput = (
    e: InputEvent & { currentTarget: HTMLTextAreaElement },
  ) => {
    props.drafts.setDraft(props.paneId, e.currentTarget.value);
  };

  const handleSend = () => {
    const text = draft();
    if (text === "") return;
    props.onSend(props.paneId, text);
  };

  const handleConfirmAppend = () => {
    const merge = pendingMerge();
    if (!merge) return;
    props.drafts.confirmMerge(props.paneId, merge.previewText);
  };

  const handleCancelMerge = () => {
    clearPending();
  };

  const handleConfirmKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      handleCancelMerge();
    }
  };

  return (
    <div class="vs-pane-draft-composer">
      <div class="vs-pane-draft-composer-input">
        <textarea
          class="vs-pane-draft-composer-textarea"
          value={draft()}
          onInput={handleInput}
          placeholder="Type a command to send…"
          aria-label="Draft command input"
          rows={3}
        />
        <button
          type="button"
          class="vs-pane-draft-composer-send"
          disabled={isEmpty()}
          onClick={handleSend}
          aria-label="Send draft command"
        >
          Send
        </button>
      </div>

      <Show when={pendingMerge()}>
        {(merge) => (
          <div
            ref={(el) => el.focus()}
            class="vs-pane-draft-composer-merge-confirm"
            role="region"
            aria-label="Merge preview: new content will be appended to existing draft"
            tabIndex={-1}
            onKeyDown={handleConfirmKeyDown}
          >
            <p class="vs-pane-draft-composer-merge-label">
              Existing draft will be merged with new content:
            </p>
            <pre class="vs-pane-draft-composer-merge-preview">
              {merge().previewText}
            </pre>
            <div class="vs-pane-draft-composer-merge-actions">
              <button
                type="button"
                class="vs-pane-draft-composer-merge-append"
                onClick={handleConfirmAppend}
                aria-label="Append fragment to existing draft"
              >
                Append
              </button>
              <button
                type="button"
                class="vs-pane-draft-composer-merge-cancel"
                onClick={handleCancelMerge}
                aria-label="Cancel merge"
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};
