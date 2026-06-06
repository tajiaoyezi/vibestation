import { Show, type Component } from "solid-js";
import type { PullStrategy } from "../../bindings";
import { t, normalizeLanguage } from "../../i18n";
import { useSettings } from "../../stores/settings";
import "./gitSyncProgress.css";

export type GitSyncKind = "push" | "pull" | "fetch";
export type GitSyncStage =
  | "counting"
  | "compressing"
  | "writing"
  | "fetch"
  | "fetching"
  | "indexing"
  | "merge"
  | "rebase"
  | "done";

export interface GitSyncProgressValue {
  current: number;
  total: number;
  bytesDone: number;
  bytesTotal: number;
  bytesPerSec: number;
}

interface GitSyncProgressDialogProps {
  kind: GitSyncKind;
  remote: string;
  branch: string;
  stage: GitSyncStage;
  progress: GitSyncProgressValue;
  abortable: boolean;
  pullStrategy?: PullStrategy;
  prune?: boolean;
  largeTransfer?: boolean;
  onPullStrategyChange?: (strategy: PullStrategy) => void;
  onPruneChange?: (enabled: boolean) => void;
  onCancel: () => void;
}

export const GitSyncProgressDialog: Component<GitSyncProgressDialogProps> = (
  props,
) => {
  const { settings } = useSettings();

  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const title = () => {
    switch (props.kind) {
      case "push":
        return `${label("dialogs.gitSync.pushTitlePrefix")} ${props.remote}`;
      case "pull":
        return `${label("dialogs.gitSync.pullTitlePrefix")} ${props.remote}/${props.branch}`;
      case "fetch":
        return `${label("dialogs.gitSync.fetchTitlePrefix")} ${props.remote}`;
    }
  };
  const ratio = () => {
    if (props.progress.total <= 0) return 0;
    return Math.min(
      100,
      Math.round((props.progress.current / props.progress.total) * 100),
    );
  };
  const objectLabel = () =>
    props.progress.total > 0
      ? `${props.progress.current} / ${props.progress.total} ${label("dialogs.gitSync.objects")}`
      : label("dialogs.gitSync.waitingForRemote");
  const speedLabel = () => {
    if (props.progress.bytesPerSec <= 0) return "0 KB/s";
    return `${Math.max(1, Math.round(props.progress.bytesPerSec / 1024))} KB/s`;
  };
  const sizeLabel = () => {
    const bytes =
      props.progress.bytesTotal > 0
        ? props.progress.bytesTotal
        : props.progress.bytesDone;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-git-sync-progress-title"
    >
      <div class="vs-dialog vs-git-sync-progress-dialog">
        <div class="vs-git-sync-progress-head">
          <div>
            <h3 id="vs-git-sync-progress-title" class="vs-dialog-title">
              {title()}
            </h3>
            <div class="vs-git-sync-progress-sub">
              <span class="vs-git-sync-chip">{props.branch}</span>
              <span>{label(`dialogs.gitSync.stages.${props.stage}`)}</span>
            </div>
          </div>
          <Show when={props.kind === "pull" && props.pullStrategy}>
            <div class="vs-git-sync-segmented" role="radiogroup">
              <button
                type="button"
                classList={{ "is-active": props.pullStrategy === "merge" }}
                onClick={() => props.onPullStrategyChange?.("merge")}
                disabled={!props.abortable}
              >
                merge
              </button>
              <button
                type="button"
                classList={{ "is-active": props.pullStrategy === "rebase" }}
                onClick={() => props.onPullStrategyChange?.("rebase")}
                disabled={!props.abortable}
              >
                rebase
              </button>
            </div>
          </Show>
        </div>

        <Show when={props.kind === "fetch"}>
          <label class="vs-git-sync-prune">
            <input
              type="checkbox"
              checked={Boolean(props.prune)}
              onChange={(event) =>
                props.onPruneChange?.(event.currentTarget.checked)
              }
              disabled={!props.abortable}
            />
            <span>{label("dialogs.gitSync.pruneDeletedRefs")}</span>
          </label>
        </Show>

        <div
          class="vs-git-sync-progress-track"
          aria-label={label("dialogs.gitSync.progress")}
        >
          <div
            class="vs-git-sync-progress-fill"
            style={{ width: `${ratio()}%` }}
          />
        </div>
        <div class="vs-git-sync-progress-meta">
          <span>{objectLabel()}</span>
          <span>{speedLabel()}</span>
        </div>

        <Show when={props.largeTransfer}>
          <p class="vs-git-sync-large">
            {label("dialogs.gitSync.largeTransferPrefix")} ({sizeLabel()})
          </p>
        </Show>

        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={props.onCancel}
          >
            {label("dialogs.common.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
};
