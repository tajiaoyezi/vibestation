import { createMemo, createSignal, For, Show, type Component } from "solid-js";
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
  const [confirmDiscard, setConfirmDiscard] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);

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
          切换分支前发现未提交修改
        </h3>
        <div class="vs-dialog-body vs-dirty-tree-body">
          <p>
            切换到 <code>{props.branchName}</code> 前需要处理以下文件：
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
            <p class="vs-branch-more-files">还有 {hiddenCount()} 个文件</p>
          </Show>
          <Show when={confirmDiscard()}>
            <p class="vs-branch-danger-copy">
              将丢弃以上未提交修改 · 该操作不可恢复。
            </p>
          </Show>
        </div>
        <div class="vs-dialog-actions vs-dirty-tree-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            disabled
            title="v0.2 不支持自动 stash · 请在终端执行 git stash 后重试"
          >
            Stash & Switch
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
                ? "切换中…"
                : "确认丢弃并切换"
              : "Discard & Switch"}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
            disabled={submitting()}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};
