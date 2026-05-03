import {
  createMemo,
  createSignal,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import type { AuthMethod } from "../../bindings";
import "./authDialog.css";

type AuthMode = "ssh-agent" | "ssh-key" | "https-helper" | "https-manual";

interface AuthDialogProps {
  remoteUrl: string;
  submitting: boolean;
  error?: string | null;
  onSubmit: (method: AuthMethod) => Promise<void>;
  onCancel: () => void;
}

export const AuthDialog: Component<AuthDialogProps> = (props) => {
  const [mode, setMode] = createSignal<AuthMode>(
    props.remoteUrl.startsWith("http") ? "https-manual" : "ssh-agent",
  );
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [keyPath, setKeyPath] = createSignal("~/.ssh/id_ed25519");
  const [passphrase, setPassphrase] = createSignal("");
  const [saveToKeychain, setSaveToKeychain] = createSignal(true);

  const resetForm = () => {
    setUsername("");
    setPassword("");
    setKeyPath("~/.ssh/id_ed25519");
    setPassphrase("");
    setSaveToKeychain(true);
  };

  onCleanup(resetForm);

  const isHttps = createMemo(() => props.remoteUrl.startsWith("http"));
  const canSubmit = () => {
    if (props.submitting) return false;
    switch (mode()) {
      case "ssh-agent":
      case "https-helper":
        return true;
      case "ssh-key":
        return keyPath().trim().length > 0;
      case "https-manual":
        return username().trim().length > 0 && password().length > 0;
    }
  };

  const selectedMethod = (): AuthMethod => {
    switch (mode()) {
      case "ssh-agent":
        return { kind: "sshAgent" };
      case "ssh-key":
        return {
          kind: "sshKeyFile",
          path: keyPath().trim(),
          passphrase: passphrase() || null,
        };
      case "https-helper":
        return { kind: "httpsHelper" };
      case "https-manual":
        return {
          kind: "httpsManual",
          username: username().trim(),
          password: password(),
        };
    }
  };

  const submit = async () => {
    if (!canSubmit()) return;
    try {
      await props.onSubmit(selectedMethod());
    } finally {
      resetForm();
    }
  };

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      resetForm();
      props.onCancel();
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      void submit();
    }
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-auth-dialog-title"
      onKeyDown={handleKeyDown}
    >
      <div class="vs-dialog vs-auth-dialog">
        <h3 id="vs-auth-dialog-title" class="vs-dialog-title">
          需要凭证
        </h3>
        <p class="vs-auth-remote">{props.remoteUrl}</p>

        <div
          class="vs-auth-tabs"
          role="tablist"
          aria-label="Authentication method"
        >
          <button
            type="button"
            classList={{ "is-active": mode() === "https-manual" }}
            onClick={() => setMode("https-manual")}
          >
            HTTPS
          </button>
          <button
            type="button"
            classList={{ "is-active": mode() === "https-helper" }}
            onClick={() => setMode("https-helper")}
          >
            helper
          </button>
          <button
            type="button"
            classList={{ "is-active": mode() === "ssh-agent" }}
            onClick={() => setMode("ssh-agent")}
          >
            ssh-agent
          </button>
          <button
            type="button"
            classList={{ "is-active": mode() === "ssh-key" }}
            onClick={() => setMode("ssh-key")}
          >
            SSH key
          </button>
        </div>

        <div class="vs-dialog-form">
          <Show when={mode() === "https-manual"}>
            <label class="vs-dialog-label">
              Username
              <input
                class="vs-dialog-input"
                value={username()}
                onInput={(event) => setUsername(event.currentTarget.value)}
                autocomplete="username"
                autofocus={isHttps()}
              />
            </label>
            <label class="vs-dialog-label">
              Password / token
              <input
                class="vs-dialog-input"
                type="password"
                value={password()}
                onInput={(event) => setPassword(event.currentTarget.value)}
                autocomplete="current-password"
              />
            </label>
            <label class="vs-dialog-checkbox">
              <input
                type="checkbox"
                checked={saveToKeychain()}
                onChange={(event) =>
                  setSaveToKeychain(event.currentTarget.checked)
                }
              />
              <span>保存到系统 keychain</span>
            </label>
          </Show>

          <Show when={mode() === "https-helper"}>
            <p class="vs-auth-copy">
              使用系统 git credential helper 重新读取凭证。
            </p>
          </Show>

          <Show when={mode() === "ssh-agent"}>
            <p class="vs-auth-copy">
              使用系统 ssh-agent。若 agent 没有加载 key，请先在终端执行
              ssh-add。
            </p>
          </Show>

          <Show when={mode() === "ssh-key"}>
            <label class="vs-dialog-label">
              SSH key path
              <input
                class="vs-dialog-input"
                value={keyPath()}
                onInput={(event) => setKeyPath(event.currentTarget.value)}
              />
            </label>
            <label class="vs-dialog-label">
              Passphrase
              <input
                class="vs-dialog-input"
                type="password"
                value={passphrase()}
                onInput={(event) => setPassphrase(event.currentTarget.value)}
                autocomplete="current-password"
              />
            </label>
          </Show>
        </div>

        <Show when={props.error}>
          <p class="vs-auth-error" role="alert">
            {props.error}
          </p>
        </Show>

        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            disabled={props.submitting}
            onClick={() => {
              resetForm();
              props.onCancel();
            }}
          >
            Cancel
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            disabled={!canSubmit()}
            onClick={() => void submit()}
          >
            {props.submitting ? "验证中…" : "Confirm"}
          </button>
        </div>
      </div>
    </div>
  );
};
