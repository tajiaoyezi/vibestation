import { createSignal, onCleanup, onMount, type Component } from "solid-js";

type PasteConfirmDialogProps = {
  text: string;
  onCancel: () => void;
  onConfirm: (rememberForWorkspace: boolean) => void;
};

const previewText = (text: string): string => {
  const lines = text.split(/\r?\n/);
  const preview = lines.slice(0, 5).map((line) => {
    const chars = Array.from(line);
    return chars.length > 80 ? `${chars.slice(0, 80).join("")}…` : line;
  });

  if (lines.length > 5) {
    preview.push("…");
  }

  return preview.join("\n");
};

export const PasteConfirmDialog: Component<PasteConfirmDialogProps> = (
  props,
) => {
  const [remember, setRemember] = createSignal(false);
  let confirmButton: HTMLButtonElement | undefined;

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      props.onCancel();
    }

    if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      props.onConfirm(remember());
    }
  };

  onMount(() => {
    confirmButton?.focus();
    document.addEventListener("keydown", handleKeyDown);
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyDown);
  });

  return (
    <div class="vs-terminal-modal-backdrop" role="dialog" aria-modal="true">
      <div class="vs-terminal-modal">
        <h3>确认粘贴多行内容？</h3>
        <p>
          检测到剪贴板里包含换行符，先确认一次，避免误把整段命令送进 shell。
        </p>
        <pre class="vs-terminal-paste-preview">{previewText(props.text)}</pre>
        <label class="vs-terminal-checkbox">
          <input
            type="checkbox"
            checked={remember()}
            onChange={(event) => setRemember(event.currentTarget.checked)}
          />
          <span>不再提示本 workspace session</span>
        </label>
        <div class="vs-terminal-modal-actions">
          <button
            type="button"
            class="vs-btn-secondary"
            onClick={props.onCancel}
          >
            取消
          </button>
          <button
            type="button"
            class="vs-btn-primary"
            ref={confirmButton}
            onClick={() => props.onConfirm(remember())}
          >
            继续粘贴
          </button>
        </div>
      </div>
    </div>
  );
};
