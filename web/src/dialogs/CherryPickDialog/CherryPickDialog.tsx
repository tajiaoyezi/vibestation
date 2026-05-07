import { createSignal, For, Show, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  CherryPickRequest,
  CherryPickStatus,
  GitLogEntry,
} from "../../bindings";

type CherryPickDialogProps = {
  workspaceId: string;
  commits: GitLogEntry[];
  onCancel: () => void;
  onPicked?: (status: CherryPickStatus) => void;
  onError?: (message: string) => void;
};

export const CherryPickDialog: Component<CherryPickDialogProps> = (props) => {
  const [commits, setCommits] = createSignal<GitLogEntry[]>(props.commits);
  const [autoCommit, setAutoCommit] = createSignal(true);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const canSubmit = () => commits().length > 0 && !submitting();

  const removeCommit = (sha: string) => {
    setCommits((current) => {
      if (current.length <= 1) {
        return current;
      }
      return current.filter((commit) => commit.shortSha !== sha);
    });
  };

  const submit = async () => {
    if (!canSubmit()) {
      return;
    }
    setSubmitting(true);
    setError(null);
    const req: CherryPickRequest = {
      workspaceId: props.workspaceId,
      commitShas: commits().map((commit) => commit.shortSha),
      autoCommit: autoCommit(),
    };
    try {
      const result = await invoke<CherryPickStatus>("cherrypick_start", {
        req,
      });
      props.onPicked?.(result);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      props.onError?.(message);
    } finally {
      setSubmitting(false);
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape" && !submitting()) {
      event.preventDefault();
      props.onCancel();
    }
  };

  return (
    <div
      class="vs-mvp16-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-cherry-pick-dialog-title"
      onKeyDown={handleKeyDown}
    >
      <section class="vs-mvp16-dialog vs-cherry-pick-dialog">
        <header class="vs-mvp16-dialog-head">
          <h3 id="vs-cherry-pick-dialog-title">
            Cherry-pick {commits().length} commits
          </h3>
          <button
            type="button"
            class="vs-mvp16-icon-btn"
            onClick={props.onCancel}
            aria-label="Close cherry-pick dialog"
          >
            x
          </button>
        </header>

        <div class="vs-mvp16-dialog-body">
          <div class="vs-cherry-pick-list">
            <For each={commits()}>
              {(commit) => (
                <div class="vs-cherry-pick-row">
                  <span class="vs-cherry-pick-sha">{commit.shortSha}</span>
                  <span class="vs-cherry-pick-message">{commit.message}</span>
                  <button
                    type="button"
                    aria-label={`Remove ${commit.shortSha}`}
                    disabled={commits().length <= 1}
                    onClick={() => removeCommit(commit.shortSha)}
                  >
                    x
                  </button>
                </div>
              )}
            </For>
          </div>

          <label class="vs-mvp16-checkbox">
            <input
              type="checkbox"
              checked={autoCommit()}
              onChange={(event) => setAutoCommit(event.currentTarget.checked)}
            />
            <span>Auto-commit each</span>
          </label>

          <Show when={error()}>
            {(message) => (
              <div class="vs-mvp16-inline-error" role="alert">
                {message()}
              </div>
            )}
          </Show>
        </div>

        <footer class="vs-mvp16-dialog-actions">
          <button
            type="button"
            class="vs-mvp16-btn-secondary"
            disabled={submitting()}
            onClick={props.onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            class="vs-mvp16-btn-primary"
            disabled={!canSubmit()}
            onClick={() => void submit()}
          >
            {submitting() ? "Cherry-picking..." : "Cherry-pick"}
          </button>
        </footer>
      </section>
    </div>
  );
};
