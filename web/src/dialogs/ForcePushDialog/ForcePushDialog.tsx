import { For, Show, type Component } from "solid-js";
import { t, normalizeLanguage } from "../../i18n";
import { useSettings } from "../../stores/settings";
import "./forcePushDialog.css";

export interface ForcePushCommit {
  sha: string;
  message: string;
}

interface ForcePushDialogProps {
  remote: string;
  branch: string;
  remoteAhead: number;
  expectedRemoteOid: string | null;
  commits: ForcePushCommit[];
  confirmation: string;
  submitting: boolean;
  onConfirmationChange: (value: string) => void;
  onConfirm: () => Promise<void>;
  onCancel: () => void;
}

export const ForcePushDialog: Component<ForcePushDialogProps> = (props) => {
  const { settings } = useSettings();

  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const canConfirm = () =>
    props.confirmation.trim() === props.branch && !props.submitting;
  const visibleCommits = () => props.commits.slice(0, 5);

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-force-push-title"
    >
      <div class="vs-dialog vs-force-push-dialog">
        <h3 id="vs-force-push-title" class="vs-dialog-title">
          {label("dialogs.forcePush.titlePrefix")} {props.branch}{" "}
          {label("dialogs.forcePush.titleTo")} {props.remote}
        </h3>
        <div class="vs-dialog-body vs-force-push-body">
          <p>
            {label("dialogs.forcePush.warningPrefix")} {props.remoteAhead}{" "}
            {label("dialogs.forcePush.warningMiddle")} {props.remote}/
            {props.branch}. {label("dialogs.forcePush.warningSuffix")}
          </p>

          <Show when={props.expectedRemoteOid}>
            <p class="vs-force-push-lease">
              force-with-lease: {props.expectedRemoteOid?.slice(0, 12)}
            </p>
          </Show>

          <Show when={visibleCommits().length > 0}>
            <ul class="vs-force-push-commit-list">
              <For each={visibleCommits()}>
                {(commit) => (
                  <li>
                    <code>{commit.sha.slice(0, 8)}</code>
                    <span>{commit.message}</span>
                  </li>
                )}
              </For>
            </ul>
          </Show>

          <label class="vs-dialog-label">
            {label("dialogs.forcePush.confirmLabel")}
            <input
              class="vs-dialog-input"
              value={props.confirmation}
              placeholder={props.branch}
              onInput={(event) =>
                props.onConfirmationChange(event.currentTarget.value)
              }
              autofocus
            />
          </label>
        </div>
        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={props.onCancel}
            disabled={props.submitting}
          >
            {label("dialogs.common.cancel")}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-danger is-armed"
            onClick={() => void props.onConfirm()}
            disabled={!canConfirm()}
          >
            {props.submitting
              ? label("dialogs.forcePush.submitting")
              : label("dialogs.forcePush.action")}
          </button>
        </div>
      </div>
    </div>
  );
};
