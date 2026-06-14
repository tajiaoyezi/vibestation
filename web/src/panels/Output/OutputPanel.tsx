/**
 * Output 面板 · Git 操作日志（push / pull / fetch）。
 *
 * 纯渲染组件 · 事件订阅 + entry 存储在 BottomPanelTabsProvider（常驻挂载），
 * 这样 tab 关闭/切换时不会丢失正在进行的操作记录。
 */
import { type Component, For, Show, createSignal } from "solid-js";
import { t, normalizeLanguage } from "../../i18n";
import { useSettings } from "../../stores/settings";
import { useBottomPanelTabs } from "../../stores/bottom-panel-tabs";
import { formatShortcut } from "../../lib/format-shortcut";
import "./styles.css";

export const OutputPanel: Component = () => {
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const { outputEntries, clearOutputEntries } = useBottomPanelTabs();

  const [expandedId, setExpandedId] = createSignal<string | null>(null);

  const toggleExpand = (id: string) =>
    setExpandedId((prev) => (prev === id ? null : id));

  const formatTime = (ts: number) => {
    const d = new Date(ts);
    return `${String(d.getHours()).padStart(2, "0")}:${String(
      d.getMinutes(),
    ).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}`;
  };

  const kindLabel = (k: "push" | "pull" | "fetch") =>
    k === "push"
      ? label("output.kindPush")
      : k === "pull"
        ? label("output.kindPull")
        : label("output.kindFetch");

  const entries = outputEntries();

  return (
    <div class="vs-output-panel" role="log" aria-label={label("output.title")}>
      <div class="vs-output-head">
        <span class="vs-output-head-title">{label("output.title")}</span>
        <Show when={entries.length > 0}>
          <button
            type="button"
            class="vs-output-clear-btn"
            aria-label={label("output.clear")}
            onClick={clearOutputEntries}
          >
            {label("output.clear")}
          </button>
        </Show>
        <span class="vs-output-head-hint">
          {formatShortcut("⌘J", "Ctrl+J")}
        </span>
      </div>
      <div class="vs-output-body">
        <Show
          when={entries.length > 0}
          fallback={
            <div class="vs-output-empty" role="status">
              {label("output.empty")}
            </div>
          }
        >
          <For each={entries}>
            {(entry) => (
              <div
                class={`vs-output-row vs-output-row-${entry.outcome}`}
                role="row"
              >
                <span class="vs-output-time">
                  {formatTime(entry.startedAt)}
                </span>
                <span class="vs-output-kind">{kindLabel(entry.kind)}</span>
                <span
                  class={`vs-output-outcome vs-output-outcome-${entry.outcome}`}
                  aria-label={entry.outcome}
                >
                  {entry.outcome === "success"
                    ? "✓"
                    : entry.outcome === "error"
                      ? "✗"
                      : entry.outcome === "cancelled"
                        ? "⊘"
                        : "…"}
                </span>
                <span class="vs-output-stage">
                  <Show
                    when={entry.outcome === "running"}
                    fallback={
                      <Show when={entry.error}>
                        <button
                          type="button"
                          class="vs-output-error-toggle"
                          aria-expanded={expandedId() === entry.id}
                          onClick={() => toggleExpand(entry.id)}
                        >
                          {label("output.errorDetail")}
                        </button>
                      </Show>
                    }
                  >
                    {entry.stage ?? "…"}
                    <Show when={entry.objectsTotal > 0}>
                      {" "}
                      · {entry.objectsDone}/{entry.objectsTotal}
                    </Show>
                  </Show>
                </span>
                <Show when={entry.error && expandedId() === entry.id}>
                  <pre class="vs-output-error-detail">{entry.error}</pre>
                </Show>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
};
