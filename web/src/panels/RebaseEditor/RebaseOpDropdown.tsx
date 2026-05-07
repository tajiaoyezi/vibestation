import { createSignal, For, Show, type Component } from "solid-js";
import type { RebaseOp } from "../../bindings";

const rebaseOps: Array<{ value: RebaseOp; label: string; className: string }> =
  [
    { value: "Pick", label: "pick", className: "is-pick" },
    { value: "Reword", label: "reword", className: "is-reword" },
    { value: "Squash", label: "squash", className: "is-squash" },
    { value: "Fixup", label: "fixup", className: "is-fixup" },
    { value: "Drop", label: "drop", className: "is-drop" },
    { value: "Edit", label: "edit", className: "is-edit" },
  ];

type RebaseOpDropdownProps = {
  value: RebaseOp;
  onChange: (op: RebaseOp) => void;
};

export const RebaseOpDropdown: Component<RebaseOpDropdownProps> = (props) => {
  const [open, setOpen] = createSignal(false);
  const currentClass = () =>
    rebaseOps.find((item) => item.value === props.value)?.className ??
    "is-pick";
  const currentLabel = () =>
    rebaseOps.find((item) => item.value === props.value)?.label ?? "pick";

  return (
    <div class="vs-rebase-op-dropdown">
      <button
        type="button"
        class={`vs-rebase-op-trigger ${currentClass()}`}
        aria-haspopup="listbox"
        aria-expanded={open()}
        onClick={() => setOpen((value) => !value)}
      >
        <span>{currentLabel()}</span>
        <span aria-hidden="true">⌄</span>
      </button>
      <Show when={open()}>
        <div class="vs-rebase-op-menu" role="listbox">
          <For each={rebaseOps}>
            {(item) => (
              <button
                type="button"
                class={item.className}
                role="option"
                aria-selected={props.value === item.value}
                onClick={() => {
                  props.onChange(item.value);
                  setOpen(false);
                }}
              >
                {item.label}
              </button>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
