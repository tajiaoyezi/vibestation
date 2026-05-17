import {
  createSignal,
  createMemo,
  onMount,
  onCleanup,
  type JSX,
} from "solid-js";

interface RollbackConfirmDialogProps {
  sessionId: string;
  commitCount: number;
  onCancel: () => void;
  onExecute: () => void;
}

export function RollbackConfirmDialog(
  props: RollbackConfirmDialogProps,
): JSX.Element {
  const [inputValue, setInputValue] = createSignal("");
  let dialogRef: HTMLDivElement | undefined;
  let inputRef: HTMLInputElement | undefined;

  const idMatches = createMemo(() => inputValue().trim() === props.sessionId);

  onMount(() => {
    inputRef?.focus();
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      props.onCancel();
    }
    if (e.key === "Enter" && idMatches()) {
      e.preventDefault();
      props.onExecute();
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

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-rollback-confirm-title"
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onCancel();
      }}
    >
      <div
        class="vs-dialog vs-rollback-confirm-dialog"
        ref={dialogRef}
        tabIndex={-1}
      >
        <h3
          id="vs-rollback-confirm-title"
          class="vs-dialog-title vs-rollback-title-warning"
        >
          ⚠ 确认回滚
        </h3>
        <div class="vs-dialog-body">
          <p>即将执行 git revert 序列：</p>
          <ul class="vs-rollback-confirm-list">
            <li>{props.commitCount} 个新 revert commit 将写入历史</li>
            <li>原始 commit 不会被删除</li>
            <li>操作可用 git revert &lt;revert-sha&gt; 撤销</li>
          </ul>
          <label class="vs-dialog-label">
            输入 session ID 确认：
            <input
              ref={inputRef}
              class="vs-dialog-input vs-rollback-confirm-input"
              data-valid={idMatches() || undefined}
              data-invalid={
                (inputValue().trim().length > 0 && !idMatches()) || undefined
              }
              type="text"
              value={inputValue()}
              onInput={(e) => setInputValue(e.currentTarget.value)}
              placeholder={props.sessionId}
              aria-label="输入 session ID 以确认回滚"
            />
          </label>
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
            class="vs-rollback-execute-btn"
            onClick={() => props.onExecute()}
            disabled={!idMatches()}
            aria-label={`执行回滚（${props.commitCount} 个 commit）`}
          >
            执行回滚（{props.commitCount} 个 commit）
          </button>
        </div>
      </div>
    </div>
  );
}
