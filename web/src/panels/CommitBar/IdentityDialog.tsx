import { createSignal, type Component } from "solid-js";

interface IdentityDialogProps {
  onConfirm: (name: string, email: string, saveLocal: boolean) => void;
  onCancel: () => void;
}

export const IdentityDialog: Component<IdentityDialogProps> = (props) => {
  const [name, setName] = createSignal("");
  const [email, setEmail] = createSignal("");
  const [saveLocal, setSaveLocal] = createSignal(true);

  return (
    <div class="vs-dialog-overlay" role="dialog" aria-modal="true">
      <div class="vs-dialog">
        <h3 class="vs-dialog-title">Git Identity 未设置</h3>
        <p class="vs-dialog-body">
          提交需要 user.name 和 user.email。请填写以下信息：
        </p>

        <div class="vs-dialog-form">
          <label class="vs-dialog-label">
            Name
            <input
              type="text"
              class="vs-dialog-input"
              value={name()}
              onInput={(e) => setName(e.currentTarget.value)}
              placeholder="Your Name"
            />
          </label>
          <label class="vs-dialog-label">
            Email
            <input
              type="email"
              class="vs-dialog-input"
              value={email()}
              onInput={(e) => setEmail(e.currentTarget.value)}
              placeholder="you@example.com"
            />
          </label>
          <label class="vs-dialog-checkbox">
            <input
              type="checkbox"
              checked={saveLocal()}
              onChange={(e) => setSaveLocal(e.currentTarget.checked)}
            />
            <span>保存到 local git config</span>
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
            class="vs-dialog-btn-primary"
            disabled={name().trim().length === 0 || email().trim().length === 0}
            onClick={() =>
              props.onConfirm(name().trim(), email().trim(), saveLocal())
            }
          >
            确认
          </button>
        </div>
      </div>
    </div>
  );
};
