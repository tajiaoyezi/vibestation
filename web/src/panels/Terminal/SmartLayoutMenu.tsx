import { createSignal, For, Show, type Component } from "solid-js";
import type { LayoutNode, LayoutPresetKind, PaneState } from "../../bindings";
import { formatShortcut } from "../../lib/format-shortcut";

export type SmartLayoutPreset = LayoutPresetKind;

export type SmartLayoutMenuProps = {
  open: boolean;
  panes: PaneState[];
  layout: LayoutNode;
  focusedPaneId: string | null;
  onApply: (preset: SmartLayoutPreset) => Promise<void>;
  onClose: () => void;
};

const truncate = (text: string, max: number): string =>
  text.length > max ? `${text.slice(0, max - 3)}...` : text;

const truncatePath = (path: string, max: number): string => {
  if (path.length <= max) return path;
  const head = Math.floor((max - 3) / 2);
  const tail = max - 3 - head;
  return `${path.slice(0, head)}...${path.slice(path.length - tail)}`;
};

function computeWillClose(
  preset: SmartLayoutPreset,
  panes: PaneState[],
  focusedPaneId: string | null,
): PaneState[] {
  if (!focusedPaneId) return [];
  if (preset === "solo") {
    return panes.filter((pane) => pane.paneId !== focusedPaneId);
  }
  if (preset === "aiAndRunner") {
    if (panes.length <= 1) return [];
    const others = panes.filter((pane) => pane.paneId !== focusedPaneId);
    const secondary = others[0];
    return panes.filter(
      (pane) =>
        pane.paneId !== focusedPaneId && pane.paneId !== secondary?.paneId,
    );
  }
  if (preset === "dualAi") {
    if (panes.length <= 2) return [];
    const keep = panes.slice(0, 2);
    return panes.filter((p) => !keep.some((k) => k.paneId === p.paneId));
  }
  if (preset === "tripleReview") {
    if (panes.length <= 3) return [];
    const keep = panes.slice(0, 3);
    return panes.filter((p) => !keep.some((k) => k.paneId === p.paneId));
  }
  if (preset === "quad") {
    if (panes.length <= 4) return [];
    const keep = panes.slice(0, 4);
    return panes.filter((p) => !keep.some((k) => k.paneId === p.paneId));
  }
  return [];
}

function computeWillCreate(
  preset: SmartLayoutPreset,
  panes: PaneState[],
): number {
  if (preset === "solo") return 0;
  if (preset === "aiAndRunner") return Math.max(0, 2 - panes.length);
  if (preset === "dualAi") return Math.max(0, 2 - panes.length);
  if (preset === "tripleReview") return Math.max(0, 3 - panes.length);
  if (preset === "quad") return Math.max(0, 4 - panes.length);
  return 0;
}

const PRESET_CONFIG: Array<{
  preset: SmartLayoutPreset;
  icon: string;
  name: string;
  desc: string;
}> = [
  {
    preset: "solo",
    icon: "▢",
    name: "Solo",
    desc: "保留当前 Pane · 关闭其他",
  },
  {
    preset: "aiAndRunner",
    icon: "▢│▢",
    name: "AI + Runner",
    desc: "左 AI · 右 Runner · 50/50 右分屏",
  },
  {
    preset: "dualAi",
    icon: "▢│▢",
    name: "Dual AI",
    desc: "左 Claude · 右 Codex · 双 AI 并行",
  },
  {
    preset: "tripleReview",
    icon: "▢│▢/▢",
    name: "Triple Review",
    desc: "左 AI · 右上 Runner · 右下 Log",
  },
  {
    preset: "quad",
    icon: "▢│▢/▢│▢",
    name: "Quad",
    desc: "2×2 四格布局",
  },
];

function presetDisabled(
  preset: SmartLayoutPreset,
  panes: PaneState[],
): boolean {
  if (preset === "solo") return false;
  if (preset === "aiAndRunner") return panes.length < 2;
  return false;
}

function presetDisabledReason(
  preset: SmartLayoutPreset,
  panes: PaneState[],
): string {
  if (preset === "aiAndRunner" && panes.length < 2) {
    return "需要至少 2 个 Pane";
  }
  return "";
}

export const SmartLayoutMenu: Component<SmartLayoutMenuProps> = (props) => {
  const [selected, setSelected] = createSignal<SmartLayoutPreset | null>(null);
  const [applying, setApplying] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const willClose = (): PaneState[] => {
    const preset = selected();
    if (!preset) return [];
    return computeWillClose(preset, props.panes, props.focusedPaneId);
  };

  const willCreate = (): number => {
    const preset = selected();
    if (!preset) return 0;
    return computeWillCreate(preset, props.panes);
  };

  const handleApply = async (): Promise<void> => {
    const preset = selected();
    if (!preset || applying()) return;
    setApplying(true);
    setError(null);
    try {
      await props.onApply(preset);
      setSelected(null);
      props.onClose();
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setApplying(false);
    }
  };

  const handleKeyDown = (event: KeyboardEvent): void => {
    if (event.key === "Escape") {
      event.preventDefault();
      if (selected()) {
        setSelected(null);
      } else {
        props.onClose();
      }
    }
  };

  return (
    <Show when={props.open}>
      <div
        class="vs-smart-layout-overlay"
        role="dialog"
        aria-modal="true"
        aria-labelledby="vs-smart-layout-title"
        tabindex={-1}
        onClick={(event) => {
          if (event.target === event.currentTarget) props.onClose();
        }}
        onKeyDown={handleKeyDown}
        ref={(el) => {
          setTimeout(() => el?.focus(), 0);
        }}
      >
        <div class="vs-smart-layout-card">
          <header class="vs-smart-layout-header">
            <h2 id="vs-smart-layout-title">Smart Layouts</h2>
            <button
              type="button"
              class="vs-smart-layout-close"
              onClick={props.onClose}
              aria-label="关闭"
            >
              ×
            </button>
          </header>

          <div class="vs-smart-layout-presets">
            <For each={PRESET_CONFIG}>
              {(config) => {
                const disabled = presetDisabled(config.preset, props.panes);
                return (
                  <button
                    type="button"
                    class={`vs-smart-layout-preset ${selected() === config.preset ? "is-selected" : ""}`}
                    onClick={() => setSelected(config.preset)}
                    disabled={disabled}
                    title={
                      disabled
                        ? presetDisabledReason(config.preset, props.panes)
                        : ""
                    }
                  >
                    <div class="vs-smart-layout-preset-icon" aria-hidden="true">
                      {config.icon}
                    </div>
                    <div class="vs-smart-layout-preset-name">{config.name}</div>
                    <div class="vs-smart-layout-preset-desc">{config.desc}</div>
                  </button>
                );
              }}
            </For>
          </div>

          <Show when={selected()}>
            <div class="vs-smart-layout-preview">
              <div class="vs-smart-layout-preview-header">
                将关闭 {willClose().length} 个 Pane
                <Show when={willCreate() > 0}>
                  {" · 将创建 "}
                  {willCreate()}
                  {" 个新 Pane"}
                </Show>
              </div>
              <Show when={willClose().length > 0}>
                <ul class="vs-smart-layout-preview-list">
                  <For each={willClose()}>
                    {(pane) => (
                      <li>
                        <code>{pane.paneId.slice(0, 8)}</code>
                        <span class="vs-smart-layout-preview-shell">
                          {truncate(pane.shell, 20)}
                        </span>
                        <span class="vs-smart-layout-preview-cwd">
                          {truncatePath(pane.cwd, 36)}
                        </span>
                      </li>
                    )}
                  </For>
                </ul>
              </Show>
              <Show
                when={
                  willClose().length === 0 &&
                  selected() === "aiAndRunner" &&
                  props.panes.length < 2
                }
              >
                <div class="vs-smart-layout-preview-warning">
                  当前只有 {props.panes.length} 个 Pane · AI+Runner 需要至少 2
                  个 Pane（先 {formatShortcut("⌘\\", "Ctrl+\\")} 分屏后再使用）
                </div>
              </Show>
            </div>
          </Show>

          <Show when={error()}>
            <div class="vs-smart-layout-error" role="alert">
              {error()}
            </div>
          </Show>

          <footer class="vs-smart-layout-footer">
            <button
              type="button"
              class="vs-smart-layout-btn vs-smart-layout-btn-cancel"
              onClick={props.onClose}
              disabled={applying()}
            >
              取消
            </button>
            <button
              type="button"
              class="vs-smart-layout-btn vs-smart-layout-btn-apply"
              onClick={handleApply}
              disabled={
                !selected() ||
                applying() ||
                (selected() !== null &&
                  presetDisabled(selected() as SmartLayoutPreset, props.panes))
              }
            >
              {applying() ? "应用中..." : "确认应用"}
            </button>
          </footer>
        </div>
      </div>
    </Show>
  );
};
