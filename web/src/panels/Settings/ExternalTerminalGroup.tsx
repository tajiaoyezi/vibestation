import {
  createMemo,
  createSignal,
  For,
  onMount,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { ExternalTerminalInfo } from "../../bindings";
import { useSettings } from "../../stores/settings";

/**
 * MVP-17 · §E.4 设置面板 "External Terminal" subsection
 * - Preferred terminal dropdown（动态从后端拿列表）
 * - "Don't ask again" toggle
 * - Env whitelist read-only 列表（v0.3 不允许编辑）
 */

/** 与 `crates/core/src/external_term/env_filter.rs` 中 `WHITELIST` 顺序一致 */
const ENV_WHITELIST: ReadonlyArray<string> = [
  "PATH",
  "HOME",
  "LANG",
  "TERM",
  "SHELL",
  "USER",
];

export const ExternalTerminalGroup: Component = () => {
  const { settings, updateSettings } = useSettings();

  const [terminals, setTerminals] = createSignal<ExternalTerminalInfo[]>([]);

  onMount(() => {
    void invoke<ExternalTerminalInfo[]>("external_term_list")
      .then(setTerminals)
      .catch(() => setTerminals([]));
  });

  const detectedTerminals = createMemo(() =>
    terminals().filter((t) => t.detected),
  );

  return (
    <div class="vs-settings-fields">
      <label class="vs-settings-field">
        <span class="vs-settings-label">
          Preferred terminal
          <span class="vs-settings-help">
            Default for &quot;Pop to External&quot; · null = ask every time
          </span>
        </span>
        <select
          class="vs-settings-select"
          value={settings.externalTermPreferred ?? ""}
          onChange={(e) => {
            const v = e.currentTarget.value;
            void updateSettings({
              externalTermPreferred: v === "" ? null : v,
            });
          }}
        >
          <option value="">Ask every time</option>
          <For each={detectedTerminals()}>
            {(t) => <option value={t.id}>{t.displayName}</option>}
          </For>
        </select>
        <Show when={detectedTerminals().length === 0}>
          <span class="vs-settings-help vs-settings-help--warn">
            No external terminal detected on this machine.
          </span>
        </Show>
      </label>

      <label class="vs-settings-field vs-settings-field--row">
        <span class="vs-settings-label">
          Don&apos;t ask again
          <span class="vs-settings-help">
            Skip the Pop to External dialog and use preferred terminal directly
          </span>
        </span>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{ active: settings.externalTermDontAskAgain }}
          onClick={() =>
            void updateSettings({
              externalTermDontAskAgain: !settings.externalTermDontAskAgain,
            })
          }
          aria-pressed={settings.externalTermDontAskAgain}
          role="switch"
          disabled={settings.externalTermPreferred === null}
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </label>

      <div class="vs-settings-field">
        <span class="vs-settings-label">
          Env whitelist (read-only · v0.4+ will allow custom)
        </span>
        <ul class="vs-settings-env-whitelist">
          <For each={ENV_WHITELIST}>{(name) => <li>{name}</li>}</For>
        </ul>
      </div>
    </div>
  );
};
