import { createMemo, createResource, Show, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "../../stores/settings";
import type { ShellInfo } from "../../bindings";
import { t, normalizeLanguage } from "../../i18n";

export const TerminalGroup: Component = () => {
  const { settings, updateSettings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

  // 从后端拿系统真有的 shell 列表（读 /etc/shells + 过滤可执行）·
  // 不再 hardcoded 写死 zsh/bash/fish · 防止用户选了机器上没装的 shell 导致 PTY spawn 失败。
  const [shells] = createResource<ShellInfo[]>(
    () => invoke<ShellInfo[]>("available_shells_list"),
    { initialValue: [] },
  );

  // 当前 settings.defaultShell 不在 list 里 · 标 "(unavailable)" 提示用户该选项已失效。
  // 不自动 fallback · 让用户主动选 · 避免静默改变数据。
  const isCurrentMissing = createMemo(() => {
    const list = shells();
    if (list.length === 0) return false;
    return !list.some((s) => s.path === settings.defaultShell);
  });

  return (
    <div class="vs-settings-fields">
      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.terminal.defaultShell")}
        </span>
        <select
          class="vs-settings-select"
          value={settings.defaultShell}
          onChange={(e) =>
            updateSettings({ defaultShell: e.currentTarget.value })
          }
        >
          <Show when={isCurrentMissing()}>
            <option value={settings.defaultShell}>
              {settings.defaultShell} ({label("settings.terminal.unavailable")})
            </option>
          </Show>
          {shells().map((s) => (
            <option value={s.path}>
              {s.label} — {s.path}
            </option>
          ))}
        </select>
        <Show when={isCurrentMissing()}>
          <span class="vs-settings-help vs-settings-help--warn">
            {label("settings.terminal.currentShellMissing")}
          </span>
        </Show>
      </label>

      <label class="vs-settings-field vs-settings-field--row">
        <span class="vs-settings-label">
          {label("settings.terminal.pasteProtection")}
        </span>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{ active: settings.pasteProtection }}
          onClick={() =>
            updateSettings({ pasteProtection: !settings.pasteProtection })
          }
          aria-pressed={settings.pasteProtection}
          role="switch"
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </label>

      <label class="vs-settings-field">
        <span class="vs-settings-label">
          {label("settings.terminal.unfocusedPaneOpacity")}{" "}
          <span class="vs-settings-value">
            {settings.unfocusedPaneOpacity.toFixed(2)}
          </span>
        </span>
        <input
          type="range"
          min={0}
          max={1}
          step={0.05}
          value={settings.unfocusedPaneOpacity}
          onInput={(e) =>
            updateSettings({
              unfocusedPaneOpacity: Number(e.currentTarget.value),
            })
          }
          class="vs-settings-slider"
        />
      </label>

      {/* MVP-20 · PTY 预热池 · 新 tab 启动加速 */}
      <label class="vs-settings-field vs-settings-field--row">
        <span class="vs-settings-label">
          {label("settings.terminal.ptyWarmPool")}
          <span class="vs-settings-help">
            {label("settings.terminal.ptyWarmPoolHelp")}
          </span>
        </span>
        <button
          type="button"
          class="vs-settings-toggle"
          classList={{ active: settings.ptyPoolEnabled }}
          onClick={() =>
            updateSettings({ ptyPoolEnabled: !settings.ptyPoolEnabled })
          }
          aria-pressed={settings.ptyPoolEnabled}
          role="switch"
        >
          <span class="vs-settings-toggle-knob" />
        </button>
      </label>

      <Show when={settings.ptyPoolEnabled}>
        <label class="vs-settings-field">
          <span class="vs-settings-label">
            {label("settings.terminal.poolSize")}{" "}
            <span class="vs-settings-value">{settings.ptyPoolSize}</span>
          </span>
          <select
            class="vs-settings-select"
            value={settings.ptyPoolSize}
            onChange={(e) =>
              updateSettings({ ptyPoolSize: Number(e.currentTarget.value) })
            }
          >
            <option value={1}>
              {label("settings.terminal.poolSizeMinimal")}
            </option>
            <option value={2}>
              {label("settings.terminal.poolSizeRapid")}
            </option>
            <option value={3}>{label("settings.terminal.poolSizeMax")}</option>
          </select>
        </label>
      </Show>
    </div>
  );
};
