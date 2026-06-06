import { createMemo, createSignal, For, Show, type Component } from "solid-js";
import { t, normalizeLanguage } from "../../i18n";
import { useSettings } from "../../stores/settings";
import "./dirtyTree.css";

export interface DirtyFiles {
  modified: string[];
  staged: string[];
  untracked: string[];
}

interface DirtyTreeDialogProps {
  branchName: string;
  dirty: DirtyFiles;
  onDiscard: () => Promise<void>;
  onCancel: () => void;
}

export const DirtyTreeDialog: Component<DirtyTreeDialogProps> = (props) => {
  const { settings } = useSettings();
  const [confirmDiscard, setConfirmDiscard] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);

  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  const files = createMemo(() => [
    ...props.dirty.staged.map((path) => ({ group: "staged", path })),
    ...props.dirty.modified.map((path) => ({ group: "modified", path })),
    ...props.dirty.untracked.map((path) => ({ group: "untracked", path })),
  ]);
  const visibleFiles = createMemo(() => files().slice(0, 5));
  const hiddenCount = () => Math.max(0, files().length - visibleFiles().length);

  const handleDiscard = async () => {
    if (!confirmDiscard()) {
      setConfirmDiscard(true);
      return;
    }

    setSubmitting(true);
    await props.onDiscard();
    setSubmitting(false);
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-dirty-tree-title"
    >
      <div class="vs-dialog vs-dirty-tree-dialog">
        <h3 id="vs-dirty-tree-title" class="vs-dialog-title">
          {label("dialogs.dirtyTree.title")}
        </h3>
        <div class="vs-dialog-body vs-dirty-tree-body">
          <p>
            {label("dialogs.dirtyTree.introPrefix")}{" "}
            <code>{props.branchName}</code>{" "}
            {label("dialogs.dirtyTree.introSuffix")}
          </p>
          <ul class="vs-branch-file-list">
            <For each={visibleFiles()}>
              {(file) => (
                <li>
                  <span class={`vs-branch-file-tag is-${file.group}`}>
                    {file.group}
                  </span>
                  <span class="vs-branch-file-path">{file.path}</span>
                </li>
              )}
            </For>
          </ul>
          <Show when={hiddenCount() > 0}>
            <p class="vs-branch-more-files">
              {label("dialogs.dirtyTree.moreFilesPrefix")} {hiddenCount()}{" "}
              {label("dialogs.dirtyTree.moreFilesSuffix")}
            </p>
          </Show>
          <Show when={confirmDiscard()}>
            <p class="vs-branch-danger-copy">
              {label("dialogs.dirtyTree.dangerCopy")}
            </p>
          </Show>
        </div>
        <div class="vs-dialog-actions vs-dirty-tree-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            disabled
            title={label("dialogs.dirtyTree.stashUnsupportedTitle")}
          >
            {label("dialogs.dirtyTree.stashAndSwitch")}
          </button>
          <button
            type="button"
            classList={{
              "vs-dialog-btn-danger": true,
              "is-armed": confirmDiscard(),
            }}
            onClick={() => void handleDiscard()}
            disabled={submitting()}
          >
            {confirmDiscard()
              ? submitting()
                ? label("dialogs.dirtyTree.switching")
                : label("dialogs.dirtyTree.confirmDiscardAndSwitch")
              : label("dialogs.dirtyTree.discardAndSwitch")}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
            disabled={submitting()}
          >
            {label("dialogs.common.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
};
