import {
  createEffect,
  createSignal,
  For,
  on,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  GitLogEntry,
  RebaseInteractivePlan,
  RebaseInteractiveStep,
  RebaseOp,
  RebaseStatus,
} from "../../bindings";
import { RebaseStepRow } from "./RebaseStepRow";

export type EditableRebaseStep = RebaseInteractiveStep & {
  shortSha: string;
  message: string;
  author: string;
  relativeTime: string;
};

type RebaseEditorProps = {
  workspaceId: string;
  branch: string;
  onto: string;
  plan: RebaseInteractivePlan;
  commits?: GitLogEntry[];
  onCancel: () => void;
  onApplied?: (status: RebaseStatus) => void;
  onError?: (message: string) => void;
};

export function createEditableRebaseSteps(
  plan: RebaseInteractivePlan,
  commits: GitLogEntry[] = [],
): EditableRebaseStep[] {
  return plan.steps.map((step) => {
    const shortSha = step.commitSha.slice(0, 8);
    const meta = commits.find(
      (commit) =>
        step.commitSha.startsWith(commit.shortSha) ||
        commit.shortSha.startsWith(shortSha),
    );
    return {
      ...step,
      shortSha,
      message: meta?.message ?? step.messageOverride ?? "commit message",
      author: meta?.authorName ?? "unknown",
      relativeTime: meta?.relativeTime ?? "",
    };
  });
}

export function validateRebasePlan(
  steps: RebaseInteractiveStep[],
): string | null {
  if (steps.length === 0) {
    return "没有可执行的 commit";
  }
  if (steps.every((step) => step.op === "Drop")) {
    return "不能 drop 全部 commit";
  }
  const first = steps[0];
  if (first?.op === "Squash" || first?.op === "Fixup") {
    return "第一条 commit 不能 squash/fixup";
  }
  const missingMessage = steps.find(
    (step) =>
      step.op === "Reword" &&
      (step.messageOverride === null || step.messageOverride.trim() === ""),
  );
  if (missingMessage) {
    return "reword 需要填写新的 commit message";
  }
  return null;
}

export const RebaseEditor: Component<RebaseEditorProps> = (props) => {
  const [steps, setSteps] = createSignal<EditableRebaseStep[]>(
    createEditableRebaseSteps(props.plan, props.commits),
  );
  const [dragIndex, setDragIndex] = createSignal<number | null>(null);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  createEffect(
    on(
      () => [props.plan, props.commits] as const,
      ([plan, commits]) => setSteps(createEditableRebaseSteps(plan, commits)),
    ),
  );

  const bindingSteps = (): RebaseInteractiveStep[] =>
    steps().map((step) => ({
      stepId: step.stepId,
      op: step.op,
      commitSha: step.commitSha,
      messageOverride:
        step.op === "Reword" || step.op === "Edit"
          ? step.messageOverride
          : null,
    }));

  const validation = () => validateRebasePlan(bindingSteps());
  const canStart = () => !submitting() && validation() === null;

  const handleOpChange = (index: number, op: RebaseOp) => {
    setSteps((current) =>
      current.map((step, itemIndex) =>
        itemIndex === index
          ? {
              ...step,
              op,
              messageOverride:
                op === "Reword" || op === "Edit"
                  ? (step.messageOverride ?? step.message)
                  : null,
            }
          : step,
      ),
    );
    setError(null);
  };

  const handleMessageChange = (index: number, message: string) => {
    setSteps((current) =>
      current.map((step, itemIndex) =>
        itemIndex === index ? { ...step, messageOverride: message } : step,
      ),
    );
    setError(null);
  };

  const handleDragStart = (index: number, event: DragEvent) => {
    setDragIndex(index);
    event.dataTransfer?.setData("text/plain", String(index));
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
    }
  };

  const handleDragOver = (_index: number, event: DragEvent) => {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  };

  const handleDrop = (targetIndex: number, event: DragEvent) => {
    event.preventDefault();
    const from = dragIndex();
    setDragIndex(null);
    if (from === null || from === targetIndex) {
      return;
    }
    setSteps((current) => {
      const next = [...current];
      const [moved] = next.splice(from, 1);
      if (!moved) return current;
      next.splice(targetIndex, 0, moved);
      return next;
    });
    setError(null);
  };

  const startRebase = async () => {
    const reason = validation();
    if (reason) {
      setError(reason);
      props.onError?.(reason);
      return;
    }
    setSubmitting(true);
    setError(null);
    const plan: RebaseInteractivePlan = { steps: bindingSteps() };
    try {
      const status = await invoke<RebaseStatus>("rebase_interactive_apply", {
        workspaceId: props.workspaceId,
        plan,
      });
      props.onApplied?.(status);
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
      aria-labelledby="vs-rebase-editor-title"
      onKeyDown={handleKeyDown}
    >
      <section class="vs-mvp16-dialog vs-rebase-editor">
        <header class="vs-rebase-editor-head">
          <div class="vs-rebase-editor-title-block">
            <h3 id="vs-rebase-editor-title">Interactive rebase</h3>
            <div class="vs-rebase-editor-chips">
              <span class="vs-rebase-chip">branch {props.branch}</span>
              <span class="vs-rebase-chip">onto {props.onto}</span>
            </div>
          </div>
          <button
            type="button"
            class="vs-mvp16-icon-btn"
            aria-label="Close rebase editor"
            onClick={props.onCancel}
            disabled={submitting()}
          >
            x
          </button>
        </header>

        <div class="vs-rebase-editor-body">
          <For each={steps()}>
            {(step, index) => (
              <RebaseStepRow
                step={step}
                index={index()}
                onOpChange={handleOpChange}
                onMessageChange={handleMessageChange}
                onDragStart={handleDragStart}
                onDragOver={handleDragOver}
                onDrop={handleDrop}
              />
            )}
          </For>
        </div>

        <Show when={error() ?? validation()}>
          {(message) => (
            <p class="vs-mvp16-inline-error" role="alert">
              {message()}
            </p>
          )}
        </Show>

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
            disabled={!canStart()}
            onClick={() => void startRebase()}
          >
            {submitting() ? "Starting..." : "Start rebase"}
          </button>
        </footer>
      </section>
    </div>
  );
};
