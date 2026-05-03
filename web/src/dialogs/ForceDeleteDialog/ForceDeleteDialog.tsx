import { For, Show, type Component } from "solid-js";
import type { BranchInfo } from "../../bindings";
import "./forceDelete.css";

interface ForceDeleteDialogProps {
  branch: BranchInfo;
  missingCommits: number;
  confirmation: string;
  onConfirmationChange: (value: string) => void;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
  deleting: boolean;
}

export const ForceDeleteDialog: Component<ForceDeleteDialogProps> = (props) => {
  const canDelete = () =>
    props.confirmation.trim() === props.branch.name && !props.deleting;
  const commitRows = () =>
    props.branch.headCommit
      ? [
          {
            sha: props.branch.headCommit.slice(0, 8),
            message: "branch tip · Phase A payload 未包含完整 commit message",
          },
        ]
      : [];

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-force-delete-title"
    >
      <div class="vs-dialog vs-force-delete-dialog">
        <h3 id="vs-force-delete-title" class="vs-dialog-title">
          强制删除分支 {props.branch.name}
        </h3>
        <div class="vs-dialog-body vs-force-delete-body">
          <p>
            该分支含 {props.missingCommits} 个未合并 commit · 删除后无法通过 UI
            长时间恢复。
          </p>

          <Show when={commitRows().length > 0}>
            <ul class="vs-force-commit-list">
              <For each={commitRows()}>
                {(commit) => (
                  <li>
                    <code>{commit.sha}</code>
                    <span>{commit.message}</span>
                  </li>
                )}
              </For>
            </ul>
          </Show>

          <label class="vs-dialog-label">
            输入分支名确认
            <input
              type="text"
              class="vs-dialog-input"
              value={props.confirmation}
              onInput={(event) =>
                props.onConfirmationChange(event.currentTarget.value)
              }
              placeholder={props.branch.name}
              autofocus
            />
          </label>
        </div>
        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
            disabled={props.deleting}
          >
            Cancel
          </button>
          <button
            type="button"
            class="vs-dialog-btn-danger is-armed"
            onClick={() => void props.onConfirm()}
            disabled={!canDelete()}
          >
            {props.deleting ? "删除中…" : "Force delete (data loss)"}
          </button>
        </div>
      </div>
    </div>
  );
};
