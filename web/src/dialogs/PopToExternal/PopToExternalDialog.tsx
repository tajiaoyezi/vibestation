import {
  createSignal,
  createMemo,
  onMount,
  For,
  Show,
  type Component,
} from "solid-js";
import type { ExternalTerminalInfo } from "../../bindings/_mock/ExternalTerminalInfo";
import type { ExternalTerminalLaunchRequest } from "../../bindings/_mock/ExternalTerminalLaunchRequest";
import type { EnvPreview } from "../../bindings/_mock/EnvPreview";
import {
  listTerminals,
  previewEnv,
  launchTerminal,
} from "../../lib/external-term";

import "./styles.css";

interface PopToExternalDialogProps {
  /** 对话框开关 */
  open: boolean;
  /** 关闭回调 */
  onClose: () => void;
  /** 源 Pane ID */
  paneId: string;
}

/**
 * MVP-17 Phase C · Pop to External 对话框
 *
 * 3 段：
 *   1. 选择终端（单选 · preferred 标记 · Don't ask again）
 *   2. 预览 cwd + shell + env
 *   3. 确认按钮
 */
export const PopToExternalDialog: Component<PopToExternalDialogProps> = (
  props,
) => {
  const [terminals, setTerminals] = createSignal<ExternalTerminalInfo[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedId, setSelectedId] = createSignal<string | null>(null);
  const [envPreview, setEnvPreview] = createSignal<EnvPreview | null>(null);
  const [dontAskAgain, setDontAskAgain] = createSignal(false);
  const [launching, setLaunching] = createSignal(false);

  const selectedTerminal = createMemo(() =>
    terminals().find((t) => t.id === selectedId()),
  );

  onMount(async () => {
    if (!props.open) return;
    await loadData();
  });

  async function loadData(): Promise<void> {
    setLoading(true);
    setError(null);
    try {
      const [terms, env] = await Promise.all([
        listTerminals(),
        previewEnv(props.paneId),
      ]);
      setTerminals(terms);
      setEnvPreview(env);
      // 默认选优先级最高且 detected 的
      const preferred = terms
        .filter((t) => t.detected)
        .sort((a, b) => a.priorityHint - b.priorityHint)[0];
      if (preferred) {
        setSelectedId(preferred.id);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleLaunch(): Promise<void> {
    const termId = selectedId();
    if (!termId) return;

    setLaunching(true);
    try {
      const request: ExternalTerminalLaunchRequest = {
        terminalId: termId,
        paneId: props.paneId,
        overrideEnv: null,
      };
      const result = await launchTerminal(request);
      if (result.success) {
        props.onClose();
      } else {
        setError(result.failedReason ?? "Launch failed");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLaunching(false);
    }
  }

  return (
    <Show when={props.open}>
      <div
        class="vs-pop-to-external-overlay"
        role="dialog"
        aria-modal="true"
        aria-labelledby="vs-pop-to-external-title"
      >
        <div class="vs-pop-to-external-modal">
          <div class="vs-pop-to-external-head">
            <h2 id="vs-pop-to-external-title" class="vs-pop-to-external-title">
              Open in External Terminal
            </h2>
            <button
              class="vs-pop-to-external-close"
              onClick={props.onClose}
              aria-label="Close dialog"
            >
              ×
            </button>
          </div>

          <Show when={error()}>
            <div class="vs-pop-to-external-error">
              <span>{error()}</span>
              <button class="vs-pop-to-external-rescan" onClick={loadData}>
                Retry
              </button>
            </div>
          </Show>

          <div class="vs-pop-to-external-body">
            <Show when={loading()}>
              <p class="vs-pop-to-external-loading">Detecting terminals…</p>
            </Show>

            <Show when={!loading() && terminals().length === 0}>
              <div class="vs-pop-to-external-empty">
                <p>No external terminals detected.</p>
                <p class="vs-pop-to-external-empty-hint">
                  Install Ghostty, iTerm2, or Terminal.app to use this feature.
                </p>
              </div>
            </Show>

            <Show when={!loading() && terminals().length > 0}>
              {/* 段 1 · 选择终端 */}
              <section class="vs-pop-to-external-section">
                <h3 class="vs-pop-to-external-section-title">
                  Select Terminal
                </h3>
                <ul class="vs-pop-to-external-term-list">
                  <For each={terminals()}>
                    {(term) => (
                      <li
                        class={`vs-pop-to-external-term ${
                          term.id === selectedId() ? "is-selected" : ""
                        } ${term.detected ? "is-detected" : "is-missing"}`}
                        onClick={() => term.detected && setSelectedId(term.id)}
                      >
                        <input
                          type="radio"
                          name="terminal"
                          value={term.id}
                          checked={term.id === selectedId()}
                          disabled={!term.detected}
                          onChange={() => setSelectedId(term.id)}
                        />
                        <span class="vs-pop-to-external-term-name">
                          {term.displayName}
                        </span>
                        <Show when={!term.detected}>
                          <span class="vs-pop-to-external-term-missing">
                            Not installed
                          </span>
                        </Show>
                      </li>
                    )}
                  </For>
                </ul>

                <label class="vs-pop-to-external-checkbox">
                  <input
                    type="checkbox"
                    checked={dontAskAgain()}
                    onChange={(e) => setDontAskAgain(e.currentTarget.checked)}
                  />
                  Don't ask again
                </label>
              </section>

              {/* 段 2 · 预览 */}
              <Show when={envPreview()}>
                <section class="vs-pop-to-external-section">
                  <h3 class="vs-pop-to-external-section-title">Preview</h3>

                  <div class="vs-pop-to-external-preview">
                    <Show
                      when={envPreview()!.visibleEntries.length > 0}
                      fallback={
                        <p class="vs-pop-to-external-preview-empty">
                          No environment variables to preview.
                        </p>
                      }
                    >
                      <ul class="vs-pop-to-external-env-list">
                        <For each={envPreview()!.visibleEntries}>
                          {(entry) => (
                            <li class="vs-pop-to-external-env-item">
                              <span class="vs-pop-to-external-env-key">
                                {entry.key}
                              </span>
                              <span class="vs-pop-to-external-env-value">
                                {entry.isSensitiveRedacted
                                  ? "***"
                                  : entry.valueTruncated}
                              </span>
                            </li>
                          )}
                        </For>
                      </ul>
                    </Show>

                    <Show when={envPreview()!.filteredCount > 0}>
                      <p class="vs-pop-to-external-filtered-hint">
                        {envPreview()!.filteredCount} items filtered for
                        security
                      </p>
                    </Show>
                  </div>
                </section>
              </Show>

              {/* 段 3 · 确认 */}
              <div class="vs-pop-to-external-actions">
                <button
                  class="vs-btn-secondary"
                  onClick={props.onClose}
                  disabled={launching()}
                >
                  Cancel
                </button>
                <button
                  class="vs-btn-primary"
                  onClick={handleLaunch}
                  disabled={!selectedId() || launching()}
                >
                  <Show
                    when={launching()}
                    fallback={
                      <span>
                        Open in {selectedTerminal()?.displayName ?? "Terminal"}
                      </span>
                    }
                  >
                    Opening…
                  </Show>
                </button>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};
