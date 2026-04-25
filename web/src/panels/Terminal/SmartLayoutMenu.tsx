import { createSignal, For, Show, type Component } from "solid-js";
import type { LayoutNode, PaneState } from "../../bindings";

// Smart Layout 预设标识 · 由组件层定义为 camelCase TS 字面量
// caller（Track A · Terminal.tsx 集成）负责把 onApply 的 preset 翻译成
// backend LayoutApplyRequest.preset 的 snake_case（"solo" / "ai_runner"）
export type SmartLayoutPreset = "solo" | "aiAndRunner";

export type SmartLayoutMenuProps = {
  /** 是否打开 menu · false 时不渲染（节省 DOM · Esc / 点击外部 / 应用完成都通过 onClose 关闭） */
  open: boolean;
  /** 当前 tab 的 pane 状态 · 用于 dry-run 预览 will-close 数量 + shell/cwd 摘要 */
  panes: PaneState[];
  /** 当前 layout 树 · 保留为 prop 以便未来扩展（如 2×2 → 单层降级路径展示）· 当前未使用 */
  layout: LayoutNode;
  /** 当前 focused pane id · null = 未聚焦（极端 case）· will-close 计算需要这个锚点 */
  focusedPaneId: string | null;
  /** 用户选 preset 后调（async · caller 在内部 invoke('pane_layout_apply', ...)） */
  onApply: (preset: SmartLayoutPreset) => Promise<void>;
  /** 关闭 menu 的统一回调（Esc / 点击 backdrop / 应用完成 / 取消按钮） */
  onClose: () => void;
};

const truncate = (text: string, max: number): string =>
  text.length > max ? `${text.slice(0, max - 3)}...` : text;

// 中段省略 · 长 cwd 路径保留首尾片段 · 例 /Users/.../vibestation/web
const truncatePath = (path: string, max: number): string => {
  if (path.length <= max) return path;
  const head = Math.floor((max - 3) / 2);
  const tail = max - 3 - head;
  return `${path.slice(0, head)}...${path.slice(path.length - tail)}`;
};

// 计算给定 preset 下 will-close 的 panes 列表（dry-run 预览数据源）
// Solo：保留 focused · 关其他
// AI+Runner：保留 focused + 第一个非 focused（次主 pane）· 关其他；< 2 panes 返回空（按钮已 disabled）
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
  return [];
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

  // AI+Runner 需要至少 2 个 pane · 单 pane 时禁用按钮 + 提示
  const aiRunnerDisabled = (): boolean => props.panes.length <= 1;

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
      // caller 抛 Error 的 message 直接展示 · 非 Error 转字符串
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setApplying(false);
    }
  };

  // Esc 行为分两层：先取消选中（若有）· 再关闭整个 menu
  // 与命令面板交互习惯一致（先 deselect · 再 dismiss）
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
          // 仅当点到 backdrop（非内部卡片）时关闭
          if (event.target === event.currentTarget) props.onClose();
        }}
        onKeyDown={handleKeyDown}
        ref={(el) => {
          // 自动 focus overlay · 保证 Esc keydown 能命中
          // setTimeout 0 让 Show 渲染完成后再 focus
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
            <button
              type="button"
              class={`vs-smart-layout-preset ${selected() === "solo" ? "is-selected" : ""}`}
              onClick={() => setSelected("solo")}
            >
              <div class="vs-smart-layout-preset-icon" aria-hidden="true">
                ▢
              </div>
              <div class="vs-smart-layout-preset-name">Solo</div>
              <div class="vs-smart-layout-preset-desc">
                保留当前 Pane · 关闭其他
              </div>
            </button>
            <button
              type="button"
              class={`vs-smart-layout-preset ${selected() === "aiAndRunner" ? "is-selected" : ""}`}
              onClick={() => setSelected("aiAndRunner")}
              disabled={aiRunnerDisabled()}
              title={aiRunnerDisabled() ? "需要至少 2 个 Pane" : ""}
            >
              <div class="vs-smart-layout-preset-icon" aria-hidden="true">
                ▢│▢
              </div>
              <div class="vs-smart-layout-preset-name">AI + Runner</div>
              <div class="vs-smart-layout-preset-desc">
                左 AI · 右 Runner · 50/50 右分屏
              </div>
            </button>
          </div>

          <Show when={selected()}>
            <div class="vs-smart-layout-preview">
              <div class="vs-smart-layout-preview-header">
                将关闭 {willClose().length} 个 Pane
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
                when={willClose().length === 0 && selected() === "aiAndRunner"}
              >
                <div class="vs-smart-layout-preview-warning">
                  当前只有 1 个 Pane · AI+Runner 需要至少 2 个 Pane（先 ⌘\
                  分屏后再使用）
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
                (selected() === "aiAndRunner" && aiRunnerDisabled())
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
