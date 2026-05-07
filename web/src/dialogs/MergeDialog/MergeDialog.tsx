import {
  createEffect,
  createMemo,
  createSignal,
  For,
  on,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  BranchInfo,
  BranchListRequest,
  BranchListResponse,
  GitStatusResponse,
  MergeRequest,
  MergeStatus,
  MergeStrategy,
} from "../../bindings";

type MergeDialogProps = {
  workspaceId: string;
  currentBranch: string;
  initialSource?: string | null;
  onCancel: () => void;
  onMerged?: (status: MergeStatus) => void;
  onOpenGitStatus?: () => void;
  onError?: (message: string) => void;
};

const strategies: Array<{ value: MergeStrategy; label: string }> = [
  { value: "fastForward", label: "Fast-forward" },
  { value: "noFastForward", label: "Merge commit (no-ff)" },
  { value: "squash", label: "Squash" },
];

export const MergeDialog: Component<MergeDialogProps> = (props) => {
  const [branches, setBranches] = createSignal<BranchInfo[]>([]);
  const [query, setQuery] = createSignal("");
  const [sourceBranch, setSourceBranch] = createSignal(
    props.initialSource ?? "",
  );
  const [strategy, setStrategy] = createSignal<MergeStrategy>("fastForward");
  const [message, setMessage] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [dirtyWarning, setDirtyWarning] = createSignal<string | null>(null);

  const candidates = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return branches()
      .filter((branch) => branch.kind !== "tag")
      .filter((branch) => branch.name !== props.currentBranch)
      .filter((branch) =>
        needle.length === 0 ? true : branch.name.toLowerCase().includes(needle),
      )
      .slice(0, 8);
  });

  const selectedSource = () => sourceBranch().trim();
  const requiresMessage = () => strategy() !== "fastForward";
  const canMerge = () =>
    !submitting() &&
    selectedSource().length > 0 &&
    (!requiresMessage() || message().trim().length > 0);

  createEffect(
    on(
      () => props.workspaceId,
      (workspaceId) => {
        if (!workspaceId) return;
        void loadBranches(workspaceId);
      },
      { defer: false },
    ),
  );

  createEffect(
    on(
      () => props.initialSource,
      (initialSource) => {
        if (initialSource) {
          setSourceBranch(initialSource);
          setQuery(initialSource);
        }
      },
    ),
  );

  const loadBranches = async (workspaceId: string) => {
    setLoading(true);
    setError(null);
    const req: BranchListRequest = { workspaceId };
    try {
      const response = await invoke<BranchListResponse>("branch_list", { req });
      setBranches(response.branches);
      const initial =
        props.initialSource ??
        response.branches.find(
          (branch) =>
            branch.kind !== "tag" && branch.name !== props.currentBranch,
        )?.name ??
        "";
      setSourceBranch(initial);
      setQuery(initial);
    } catch (err) {
      const next = err instanceof Error ? err.message : String(err);
      setError(next);
      props.onError?.(next);
    } finally {
      setLoading(false);
    }
  };

  const isDirty = (status: GitStatusResponse): boolean =>
    status.staged.length + status.unstaged.length + status.untracked.length > 0;

  const runMerge = async () => {
    if (!canMerge()) {
      return;
    }
    setSubmitting(true);
    setError(null);
    setDirtyWarning(null);
    try {
      const status = await invoke<GitStatusResponse>("git_status_query", {
        req: { workspaceId: props.workspaceId },
      });
      if (isDirty(status)) {
        setDirtyWarning("工作区有未提交修改 · 请先 commit / stash / discard");
        props.onOpenGitStatus?.();
        return;
      }

      const req: MergeRequest = {
        workspaceId: props.workspaceId,
        sourceBranch: selectedSource(),
        strategy: strategy(),
        commitMessage: requiresMessage() ? message().trim() : null,
      };
      const result = await invoke<MergeStatus>("merge_start", { req });
      props.onMerged?.(result);
    } catch (err) {
      const next = err instanceof Error ? err.message : String(err);
      setError(next);
      props.onError?.(next);
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
      aria-labelledby="vs-merge-dialog-title"
      onKeyDown={handleKeyDown}
    >
      <section class="vs-mvp16-dialog vs-merge-dialog">
        <header class="vs-mvp16-dialog-head">
          <h3 id="vs-merge-dialog-title">
            合并 {selectedSource() || "source"} 到 {props.currentBranch}
          </h3>
          <button
            type="button"
            class="vs-mvp16-icon-btn"
            onClick={props.onCancel}
            aria-label="Close merge dialog"
          >
            x
          </button>
        </header>

        <div class="vs-mvp16-dialog-body">
          <label class="vs-mvp16-field">
            <span>source branch</span>
            <input
              class="vs-mvp16-input"
              value={query()}
              placeholder={loading() ? "Loading branches..." : "Search branch"}
              onInput={(event) => {
                setQuery(event.currentTarget.value);
                setSourceBranch(event.currentTarget.value);
              }}
            />
          </label>

          <div class="vs-merge-branch-list">
            <For each={candidates()}>
              {(branch) => (
                <button
                  type="button"
                  classList={{
                    "is-selected": branch.name === selectedSource(),
                  }}
                  onClick={() => {
                    setSourceBranch(branch.name);
                    setQuery(branch.name);
                  }}
                >
                  <span>{branch.name}</span>
                  <span>{branch.kind}</span>
                </button>
              )}
            </For>
          </div>

          <div class="vs-merge-strategy" role="radiogroup">
            <For each={strategies}>
              {(item) => (
                <label>
                  <input
                    type="radio"
                    name="merge-strategy"
                    checked={strategy() === item.value}
                    onChange={() => setStrategy(item.value)}
                  />
                  <span>{item.label}</span>
                </label>
              )}
            </For>
          </div>

          <Show when={requiresMessage()}>
            <label class="vs-mvp16-field">
              <span>commit message</span>
              <textarea
                class="vs-mvp16-textarea"
                rows={5}
                value={message()}
                onInput={(event) => setMessage(event.currentTarget.value)}
              />
            </label>
          </Show>

          <Show when={dirtyWarning()}>
            {(warning) => (
              <div class="vs-mvp16-inline-warning" role="alert">
                {warning()}
              </div>
            )}
          </Show>
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
            onClick={props.onCancel}
            disabled={submitting()}
          >
            Cancel
          </button>
          <button
            type="button"
            class="vs-mvp16-btn-primary"
            disabled={!canMerge()}
            onClick={() => void runMerge()}
          >
            {submitting() ? "Merging..." : "Merge"}
          </button>
        </footer>
      </section>
    </div>
  );
};
