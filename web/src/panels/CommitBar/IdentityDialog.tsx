import { createSignal, Show, type Component } from "solid-js";
import { t } from "../../i18n";
import { useSettings } from "../../stores/settings";

const isValidEmail = (e: string): boolean =>
  /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(e.trim());

interface IdentityDialogProps {
  onConfirm: (name: string, email: string, saveLocal: boolean) => void;
  onCancel: () => void;
}

export const IdentityDialog: Component<IdentityDialogProps> = (props) => {
  const { settings } = useSettings();
  const language = () => settings.language;
  const label = (key: string) => t(key, language());

  const [name, setName] = createSignal("");
  const [email, setEmail] = createSignal("");
  const [saveLocal, setSaveLocal] = createSignal(true);

  const canSave = () =>
    name().trim().length > 0 &&
    email().trim().length > 0 &&
    isValidEmail(email());

  return (
    <div class="vs-dialog-overlay" role="dialog" aria-modal="true">
      <div class="vs-dialog">
        <h3 class="vs-dialog-title">{label("commitBar.identityTitle")}</h3>
        <p class="vs-dialog-body">{label("commitBar.identityBody")}</p>

        <div class="vs-dialog-form">
          <label class="vs-dialog-label">
            {label("commitBar.identityName")}
            <input
              type="text"
              class="vs-dialog-input"
              value={name()}
              onInput={(e) => setName(e.currentTarget.value)}
              placeholder={label("commitBar.identityNamePlaceholder")}
            />
          </label>
          <label class="vs-dialog-label">
            {label("commitBar.identityEmail")}
            <input
              type="email"
              class="vs-dialog-input"
              value={email()}
              onInput={(e) => setEmail(e.currentTarget.value)}
              placeholder="you@example.com"
            />
            <Show when={email().trim().length > 0 && !isValidEmail(email())}>
              <span class="vs-dialog-error-hint">
                {label("commitBar.identityEmailInvalid")}
              </span>
            </Show>
          </label>
          <label class="vs-dialog-checkbox">
            <input
              type="checkbox"
              checked={saveLocal()}
              onChange={(e) => setSaveLocal(e.currentTarget.checked)}
            />
            <span>{label("commitBar.identitySaveLocal")}</span>
          </label>
        </div>

        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => props.onCancel()}
          >
            {label("dialogs.common.cancel")}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            disabled={!canSave()}
            onClick={() =>
              props.onConfirm(name().trim(), email().trim(), saveLocal())
            }
          >
            {label("dialogs.common.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
};