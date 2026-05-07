import { createSignal, Show, type Component } from "solid-js";

export type ConflictBannerVariant = "active" | "recovery";
export type ConflictOperation = "rebase" | "merge" | "cherrypick";

type ConflictBannerProps = {
  variant: ConflictBannerVariant;
  operation: ConflictOperation;
  source?: string | null;
  target?: string | null;
  currentStep: number;
  totalSteps: number;
  filePath?: string | null;
  allResolved: boolean;
  allowSkip: boolean;
  busy?: boolean;
  onContinue: () => void | Promise<void>;
  onAbort: () => void | Promise<void>;
  onSkip?: () => void | Promise<void>;
  onViewStatus?: () => void;
};

const operationCopy: Record<ConflictOperation, string> = {
  rebase: "Rebasing",
  merge: "Merging",
  cherrypick: "Cherry-picking",
};

export const ConflictBanner: Component<ConflictBannerProps> = (props) => {
  const [confirmingAbort, setConfirmingAbort] = createSignal(false);

  const progress = () =>
    props.totalSteps > 0 ? `${props.currentStep}/${props.totalSteps}` : "0/0";
  const sourceCopy = () => {
    if (props.operation === "merge") {
      return props.source && props.target
        ? `${props.source} into ${props.target}`
        : "selected branch";
    }
    if (props.operation === "cherrypick") {
      return props.source ?? "commit range";
    }
    return props.source && props.target
      ? `${props.source} onto ${props.target}`
      : (props.target ?? "target branch");
  };
  const message = () =>
    props.variant === "recovery"
      ? `上次操作未完成 · ${operationCopy[props.operation]} ${sourceCopy()} · ${progress()}`
      : `${operationCopy[props.operation]} ${sourceCopy()} · ${progress()} conflict on ${props.filePath ?? "selected file"}`;

  const confirmAbort = async () => {
    setConfirmingAbort(false);
    await props.onAbort();
  };

  return (
    <div
      class="vs-conflict-banner"
      classList={{ "is-recovery": props.variant === "recovery" }}
      role="status"
    >
      <div class="vs-conflict-banner-main">
        <span class="vs-conflict-banner-icon" aria-hidden="true">
          !
        </span>
        <span>{message()}</span>
      </div>
      <div class="vs-conflict-banner-actions">
        <Show when={props.variant === "recovery" && props.onViewStatus}>
          <button type="button" onClick={props.onViewStatus}>
            View status
          </button>
        </Show>
        <button
          type="button"
          disabled={!props.allResolved || props.busy}
          onClick={() => void props.onContinue()}
        >
          Continue
        </button>
        <button
          type="button"
          class="is-danger"
          disabled={props.busy}
          onClick={() => setConfirmingAbort(true)}
        >
          Abort
        </button>
        <Show when={props.allowSkip && props.onSkip}>
          <button
            type="button"
            disabled={props.busy}
            onClick={() => void props.onSkip?.()}
          >
            Skip
          </button>
        </Show>
      </div>

      <Show when={confirmingAbort()}>
        <div
          class="vs-conflict-confirm"
          role="dialog"
          aria-modal="true"
          aria-label="Confirm abort"
        >
          <div class="vs-conflict-confirm-box">
            <h3>放弃当前 {props.operation}</h3>
            <p>
              工作区将回滚到操作前的 HEAD。此操作不可从 Vibestation 自动撤销。
            </p>
            <div class="vs-conflict-confirm-actions">
              <button
                type="button"
                class="vs-mvp16-btn-secondary"
                onClick={() => setConfirmingAbort(false)}
              >
                Cancel
              </button>
              <button
                type="button"
                class="vs-mvp16-btn-danger"
                onClick={() => void confirmAbort()}
              >
                Abort
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
