import { type Component } from "solid-js";

type RebaseMessageEditorProps = {
  value: string;
  onChange: (message: string) => void;
};

export const RebaseMessageEditor: Component<RebaseMessageEditorProps> = (
  props,
) => {
  return (
    <label class="vs-rebase-message-editor">
      <span>Commit message</span>
      <textarea
        value={props.value}
        rows={3}
        onInput={(event) => props.onChange(event.currentTarget.value)}
      />
    </label>
  );
};
