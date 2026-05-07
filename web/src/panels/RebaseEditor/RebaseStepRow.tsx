import { Show, type Component } from "solid-js";
import type { RebaseOp } from "../../bindings";
import { RebaseDragHandle } from "./RebaseDragHandle";
import { RebaseMessageEditor } from "./RebaseMessageEditor";
import { RebaseOpDropdown } from "./RebaseOpDropdown";
import type { EditableRebaseStep } from "./RebaseEditor";

type RebaseStepRowProps = {
  step: EditableRebaseStep;
  index: number;
  onOpChange: (index: number, op: RebaseOp) => void;
  onMessageChange: (index: number, message: string) => void;
  onDragStart: (index: number, event: DragEvent) => void;
  onDragOver: (index: number, event: DragEvent) => void;
  onDrop: (index: number, event: DragEvent) => void;
};

export const RebaseStepRow: Component<RebaseStepRowProps> = (props) => {
  const isDropped = () => props.step.op === "Drop";
  const showsMessageEditor = () =>
    props.step.op === "Reword" || props.step.op === "Edit";

  return (
    <div
      class="vs-rebase-step-row"
      classList={{ "is-drop": isDropped() }}
      onDragOver={(event) => props.onDragOver(props.index, event)}
      onDrop={(event) => props.onDrop(props.index, event)}
    >
      <div class="vs-rebase-step-main">
        <RebaseDragHandle index={props.index} onDragStart={props.onDragStart} />
        <RebaseOpDropdown
          value={props.step.op}
          onChange={(op) => props.onOpChange(props.index, op)}
        />
        <span class="vs-rebase-sha">{props.step.shortSha}</span>
        <span class="vs-rebase-message" title={props.step.message}>
          {props.step.message}
        </span>
        <span class="vs-rebase-author">{props.step.author}</span>
        <span class="vs-rebase-time">{props.step.relativeTime}</span>
      </div>
      <Show when={showsMessageEditor()}>
        <RebaseMessageEditor
          value={props.step.messageOverride ?? props.step.message}
          onChange={(message) => props.onMessageChange(props.index, message)}
        />
      </Show>
    </div>
  );
};
