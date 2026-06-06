import { For, Show, type Component } from "solid-js";
import type { BranchInfo } from "../../bindings";
import { t, normalizeLanguage } from "../../i18n";
import { useSettings } from "../../stores/settings";
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
  const { settings } = useSettings();

  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const canDelete = () =>
    props.confirmation.trim() === props.branch.name && !props.deleting;
  const commitRows = () =>
    props.branch.headCommit
      ? [
          {
            sha: props.branch.headCommit.slice(0, 8),
            message: label("dialogs.forceDelete.branchTipMissingMessage"),
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
          {label("dialogs.forceDelete.titlePrefix")} {props.branch.name}
        </h3>
        <div class="vs-dialog-body vs-force-delete-body">
          <p>
            {label("dialogs.forceDelete.warningPrefix")} {props.missingCommits}{" "}
            {label("dialogs.forceDelete.warningSuffix")}
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
            {label("dialogs.forceDelete.confirmLabel")}
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
            {label("dialogs.common.cancel")}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-danger is-armed"
            onClick={() => void props.onConfirm()}
            disabled={!canDelete()}
          >
            {props.deleting
              ? label("dialogs.forceDelete.deleting")
              : label("dialogs.forceDelete.action")}
          </button>
        </div>
      </div>
    </div>
  );
};
