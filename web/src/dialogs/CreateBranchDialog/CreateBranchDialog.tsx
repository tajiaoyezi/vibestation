import { createMemo, createSignal, For, Show, type Component } from "solid-js";
import type { BranchCreateRequest, BranchInfo } from "../../bindings";
import { validateBranchName } from "../../utils/branchName";
import "./createBranch.css";

type CreateBranchPayload = Omit<BranchCreateRequest, "workspaceId">;

interface CreateBranchDialogProps {
  branches: BranchInfo[];
  initialFromRef?: string | null;
  onCreate: (payload: CreateBranchPayload) => Promise<boolean>;
  onCancel: () => void;
}

export const CreateBranchDialog: Component<CreateBranchDialogProps> = (
  props,
) => {
  const [name, setName] = createSignal("");
  const [fromRef, setFromRef] = createSignal(props.initialFromRef ?? "");
  const [checkout, setCheckout] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);

  const branchOptions = createMemo(() =>
    props.branches.filter((branch) => branch.kind !== "tag"),
  );
  const validation = createMemo(() => validateBranchName(name()));
  const canSubmit = () => validation().valid && !submitting();

  const handleSubmit = async () => {
    if (!canSubmit()) {
      return;
    }

    setSubmitting(true);
    const shouldClose = await props.onCreate({
      name: name().trim(),
      fromRef: fromRef() || null,
      checkout: checkout(),
    });
    setSubmitting(false);

    if (shouldClose) {
      props.onCancel();
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      props.onCancel();
    }
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      void handleSubmit();
    }
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-create-branch-title"
      onKeyDown={handleKeyDown}
    >
      <div class="vs-dialog vs-create-branch-dialog">
        <h3 id="vs-create-branch-title" class="vs-dialog-title">
          新建分支
        </h3>

        <div class="vs-dialog-form">
          <label class="vs-dialog-label">
            Name
            <input
              type="text"
              classList={{
                "vs-dialog-input": true,
                "vs-branch-input-invalid":
                  name().length > 0 && !validation().valid,
              }}
              value={name()}
              onInput={(event) => setName(event.currentTarget.value)}
              placeholder="feat/my-branch"
              title={!validation().valid ? validation().reason : ""}
              autofocus
            />
            <Show when={name().length > 0 && !validation().valid}>
              <span class="vs-dialog-error-hint">{validation().reason}</span>
            </Show>
          </label>

          <label class="vs-dialog-label">
            From
            <select
              class="vs-dialog-input vs-branch-select"
              value={fromRef()}
              onChange={(event) => setFromRef(event.currentTarget.value)}
            >
              <option value="">HEAD</option>
              <For each={branchOptions()}>
                {(branch) => (
                  <option value={branch.name}>
                    {branch.kind === "remote" ? "remote · " : ""}
                    {branch.name}
                  </option>
                )}
              </For>
            </select>
          </label>

          <label class="vs-dialog-checkbox">
            <input
              type="checkbox"
              checked={checkout()}
              onChange={(event) => setCheckout(event.currentTarget.checked)}
            />
            <span>create and checkout</span>
          </label>
        </div>

        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
            disabled={submitting()}
          >
            取消
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            onClick={() => void handleSubmit()}
            disabled={!canSubmit()}
            title={!validation().valid ? validation().reason : ""}
          >
            {submitting() ? "创建中…" : "确认"}
          </button>
        </div>
      </div>
    </div>
  );
};
