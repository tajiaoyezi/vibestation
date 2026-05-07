import { type Component } from "solid-js";

type RebaseDragHandleProps = {
  index: number;
  onDragStart: (index: number, event: DragEvent) => void;
};

export const RebaseDragHandle: Component<RebaseDragHandleProps> = (props) => {
  return (
    <span
      class="vs-rebase-drag-handle"
      draggable={true}
      title="Drag to reorder"
      aria-label={`Reorder commit ${props.index + 1}`}
      onDragStart={(event) => props.onDragStart(props.index, event)}
    >
      ☰
    </span>
  );
};
