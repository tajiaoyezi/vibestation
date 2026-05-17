import {
  createSignal,
  For,
  Show,
  onMount,
  onCleanup,
  type JSX,
} from "solid-js";
import type { RollbackPreview, RollbackCommitEntry } from "./rollbackContract";

interface RollbackPreviewModalProps {
  preview: RollbackPreview;
  onCancel: () => void;
  onConfirm: (includeShas: string[]) => void;
}

function shortSha(sha: string): string {
  return sha.slice(0, 7);
}

export function RollbackPreviewModal(
  props: RollbackPreviewModalProps,
): JSX.Element {
  const [overrides, setOverrides] = createSignal<Record<string, boolean>>({});
  const [filesExpanded, setFilesExpanded] = createSignal(false);
  let dialogRef: HTMLDivElement | undefined;

  onMount(() => dialogRef?.focus());

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

  const isIncluded = (entry: RollbackCommitEntry): boolean => {
    const ov = overrides()[entry.sha];
    if (ov !== undefined) return ov;
    return entry.include;
  };

  const toggleCommit = (sha: string) => {
    setOverrides((prev) => ({ ...prev, [sha]: !isIncludedBySha(sha) }));
  };

  const isIncludedBySha = (sha: string): boolean => {
    const ov = overrides()[sha];
    if (ov !== undefined) return ov;
    const entry = props.preview.commits.find((c) => c.sha === sha);
    return entry?.include ?? false;
  };

  const includedShas = (): string[] =>
    props.preview.commits.filter((c) => isIncluded(c)).map((c) => c.sha);

  const lowConfidenceCount = (): number =>
    props.preview.commits.filter((c) => c.confidence < 0.9).length;

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-rollback-preview-title"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onCancel();
      }}
    >
      <div
        class="vs-dialog vs-rollback-preview-dialog"
        ref={dialogRef}
        tabIndex={-1}
      >
        <h3
          id="vs-rollback-preview-title"
          class="vs-dialog-title vs-rollback-title-warning"
        >
          ⚠ 回滚预览：Session #{props.preview.session_id}
        </h3>
        <div class="vs-dialog-body vs-rollback-preview-body">
          <p class="vs-rollback-preview-summary">
            将生成 {includedShas().length} 个 revert commit，撤销以下改动：
          </p>

          <ul
            class="vs-rollback-commit-list"
            role="list"
            aria-label="Commits to revert"
          >
            <For each={props.preview.commits}>
              {(entry) => (
                <li
                  class="vs-rollback-commit-item"
                  data-low-confidence={entry.confidence < 0.9 || undefined}
                >
                  <label class="vs-rollback-commit-label">
                    <input
                      type="checkbox"
                      checked={isIncluded(entry)}
                      onChange={() => toggleCommit(entry.sha)}
                      aria-label={`Include commit ${shortSha(entry.sha)} in rollback`}
                    />
                    <span
                      class="vs-rollback-commit-indicator"
                      data-included={isIncluded(entry) || undefined}
                      data-low={entry.confidence < 0.9 || undefined}
                    >
                      {entry.confidence >= 0.9 ? "✓" : "⚠"}
                    </span>
                    <code class="vs-rollback-commit-sha">
                      {shortSha(entry.sha)}
                    </code>
                    <span class="vs-rollback-commit-msg">{entry.message}</span>
                    <span class="vs-rollback-commit-confidence">
                      置信度 {entry.confidence.toFixed(2)}
                    </span>
                  </label>
                  <Show when={entry.confidence < 0.9}>
                    <div class="vs-rollback-low-confidence-hint">
                      ↑ 低置信度关联，建议确认是否属于本 session
                    </div>
                  </Show>
                </li>
              )}
            </For>
          </ul>

          <div class="vs-rollback-files-summary">
            影响文件：{props.preview.total_files_changed} 文件 · +
            {props.preview.total_insertions} / -{props.preview.total_deletions}
            <button
              class="vs-rollback-files-toggle"
              onClick={() => setFilesExpanded((prev) => !prev)}
              aria-expanded={filesExpanded()}
            >
              {filesExpanded() ? "收起" : "展开查看完整列表"}
            </button>
          </div>

          <Show when={props.preview.has_low_confidence}>
            <div class="vs-rollback-low-confidence-banner" role="alert">
              ⚠ 警告：{lowConfidenceCount()} 个 commit 置信度 &lt;
              0.9，请确认是否包含在回滚中
            </div>
          </Show>
        </div>

        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
          >
            取消
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            onClick={() => props.onConfirm(includedShas())}
            disabled={includedShas().length === 0}
            aria-label={`确认回滚 ${includedShas().length} 个 commit`}
          >
            确认回滚 →
          </button>
        </div>
      </div>
    </div>
  );
}
