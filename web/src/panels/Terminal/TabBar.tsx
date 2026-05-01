import { invoke } from "@tauri-apps/api/core";
import {
  createEffect,
  createSignal,
  For,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import type { TabState } from "../../bindings";

type TabBarProps = {
  tabs: readonly TabState[];
  activeTabId: string | null;
  onClose: (tabId: string) => void;
  /**
   * 点 × 触发 leave 动画**开始时**同步调用 · 上层应立即把 active 切到 sibling tab ·
   * 让底部蓝条 indicator 平滑滑过去 · 而不是等 240ms 动画结束才"瞬间跳"。
   * 真正 unmount 仍在 onClose（240ms 后）· 期间 leaving tab DOM 仍存活但不再 active。
   */
  onCloseRequested?: (tabId: string) => void;
  onContextMenuTab?: (tabId: string) => void;
  onCreate: () => void;
  onRename: (tabId: string, name: string) => Promise<void>;
  onSelect: (tabId: string) => void;
  pendingRenameTabId?: string | null;
};

/**
 * Tab leave 动画时长 · 必须和 styles.css `@keyframes vs-tab-leave` 匹配（240ms）。
 * 多维度退场 · fade + 上飞 + 缩小 + 模糊 + 收缩 · Material standard ease。
 */
const TAB_LEAVE_DURATION_MS = 240;

export const TabBar: Component<TabBarProps> = (props) => {
  const [editingTabId, setEditingTabId] = createSignal<string | null>(null);
  const [draftName, setDraftName] = createSignal("");
  // 正在 leave 动画中的 tab id 集合 · CSS 加 `is-leaving` 触发 collapse 动画 ·
  // 动画结束后才调 props.onClose 真正 unmount · 让用户看到关闭过渡
  const [leavingTabIds, setLeavingTabIds] = createSignal<Set<string>>(
    new Set(),
  );
  // pending leave timer · 防 unmount 时遗留 timer 调已 stale 的 onClose
  const leaveTimers = new Map<string, ReturnType<typeof setTimeout>>();
  let renameInput: HTMLInputElement | undefined;

  const startLeave = (tabId: string) => {
    if (leavingTabIds().has(tabId)) return;
    // a11y · 用户开了 prefers-reduced-motion 时 · 不等动画 · 立即关
    const motionReduced =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const duration = motionReduced ? 0 : TAB_LEAVE_DURATION_MS;
    setLeavingTabIds((prev) => {
      const next = new Set(prev);
      next.add(tabId);
      return next;
    });
    // 立即请求上层切 active 到 sibling · 让蓝条 indicator 平滑滑走 ·
    // 而不是等 240ms unmount 后蓝条"瞬间跳"。leaving tab 仍渲染但失去 is-active。
    props.onCloseRequested?.(tabId);
    const timer = setTimeout(() => {
      leaveTimers.delete(tabId);
      props.onClose(tabId);
      setLeavingTabIds((prev) => {
        const next = new Set(prev);
        next.delete(tabId);
        return next;
      });
    }, duration);
    leaveTimers.set(tabId, timer);
  };

  onCleanup(() => {
    for (const t of leaveTimers.values()) {
      clearTimeout(t);
    }
    leaveTimers.clear();
  });

  const startRename = (tab: TabState) => {
    setEditingTabId(tab.tabId);
    setDraftName(tab.name);
  };

  const stopRename = () => {
    setEditingTabId(null);
    setDraftName("");
  };

  const commitRename = async () => {
    const tabId = editingTabId();
    const name = draftName().trim();
    if (!tabId) {
      return;
    }

    if (!name) {
      stopRename();
      return;
    }

    await props.onRename(tabId, name);
    stopRename();
  };

  createEffect(() => {
    if (!editingTabId()) {
      return;
    }

    queueMicrotask(() => {
      renameInput?.focus();
      renameInput?.select();
    });
  });

  createEffect(() => {
    const pendingId = props.pendingRenameTabId;
    if (!pendingId) {
      return;
    }

    const tab = props.tabs.find((t) => t.tabId === pendingId);
    if (tab) {
      // 不切 active · rename 入口（右键菜单 / future shortcut）应该独立于 tab 选中。
      // 双击 label 路径不受影响（第一击已在 trigger button onClick 内 select 过）。
      startRename(tab);
    }
  });

  // 底部独立 indicator · 跟随 active tab 平滑滑动 · 取代每个 tab 的 border-bottom。
  // 关键体验改进：active 切换时蓝条平滑过渡 · 不再"瞬间跳"。
  let scrollContainer: HTMLDivElement | undefined;
  let indicator: HTMLDivElement | undefined;
  const [indicatorReady, setIndicatorReady] = createSignal(false);

  const updateIndicator = () => {
    if (!scrollContainer || !indicator) return;
    const activeId = props.activeTabId;
    if (!activeId) {
      indicator.style.opacity = "0";
      return;
    }
    const activeEl = scrollContainer.querySelector<HTMLDivElement>(
      `[data-tab-id="${activeId}"]:not(.is-leaving)`,
    );
    if (!activeEl) {
      indicator.style.opacity = "0";
      return;
    }
    const containerRect = scrollContainer.getBoundingClientRect();
    const tabRect = activeEl.getBoundingClientRect();
    const left = tabRect.left - containerRect.left + scrollContainer.scrollLeft;
    indicator.style.transform = `translateX(${left}px)`;
    indicator.style.width = `${tabRect.width}px`;
    indicator.style.opacity = "1";
    if (!indicatorReady()) setIndicatorReady(true);
  };

  // 监听 active 切换 + tabs 列表变化 + leaving tabs 变化 · 都触发 indicator 重算
  createEffect(() => {
    // track signals
    void props.activeTabId;
    void props.tabs.length;
    void leavingTabIds();
    // 等下一帧让 DOM commit · 拿正确尺寸
    requestAnimationFrame(() => updateIndicator());
  });

  // ResizeObserver · 容器宽度变化（窗口 resize / 字体变化）时重算 indicator
  let resizeObserver: ResizeObserver | undefined;
  const setScrollRef = (el: HTMLDivElement) => {
    scrollContainer = el;
    if (typeof ResizeObserver !== "undefined") {
      resizeObserver = new ResizeObserver(() => {
        requestAnimationFrame(() => updateIndicator());
      });
      resizeObserver.observe(el);
    }
  };

  onCleanup(() => {
    resizeObserver?.disconnect();
  });

  return (
    <div class="vs-terminal-tabbar" role="tablist" aria-label="Terminal tabs">
      <div class="vs-terminal-tabbar-scroll" ref={setScrollRef}>
        <div
          class={`vs-terminal-tabbar-indicator ${indicatorReady() ? "is-ready" : ""}`}
          ref={indicator}
          aria-hidden="true"
        />
        <For each={props.tabs}>
          {(tab) => {
            const editing = () => editingTabId() === tab.tabId;
            const active = () => props.activeTabId === tab.tabId;
            const leaving = () => leavingTabIds().has(tab.tabId);

            return (
              <div
                class={`vs-terminal-tab ${active() ? "is-active" : ""} ${leaving() ? "is-leaving" : ""}`}
                role="presentation"
                data-tab-id={tab.tabId}
                onContextMenu={(e) => {
                  e.preventDefault();
                  // 把右键命中的 tab id 提前告诉 Terminal · 让 menu:action listener
                  // 用 contextTabId 而非 currentActiveTabId · 修右键非 active tab
                  // 操作（rename / close 等）误作用到 active tab 的 bug。
                  props.onContextMenuTab?.(tab.tabId);
                  void invoke("menu_show_tab", {
                    x: e.clientX,
                    y: e.clientY,
                  });
                }}
              >
                <Show
                  when={editing()}
                  fallback={
                    <button
                      type="button"
                      class="vs-terminal-tab-trigger"
                      role="tab"
                      aria-selected={active()}
                      aria-controls={`terminal-pane-${tab.tabId}`}
                      onClick={() => props.onSelect(tab.tabId)}
                    >
                      <span
                        class="vs-terminal-tab-label"
                        onDblClick={(event) => {
                          event.preventDefault();
                          event.stopPropagation();
                          startRename(tab);
                        }}
                      >
                        {tab.name}
                      </span>
                    </button>
                  }
                >
                  <input
                    ref={renameInput}
                    type="text"
                    class="vs-terminal-tab-input"
                    value={draftName()}
                    onInput={(event) => setDraftName(event.currentTarget.value)}
                    onBlur={() => void commitRename()}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") {
                        event.preventDefault();
                        void commitRename();
                      }

                      if (event.key === "Escape") {
                        event.preventDefault();
                        stopRename();
                      }
                    }}
                  />
                </Show>

                <button
                  type="button"
                  class="vs-terminal-tab-close"
                  aria-label={`关闭 ${tab.name}`}
                  onClick={(event) => {
                    event.preventDefault();
                    event.stopPropagation();
                    // 先触发 leave 动画 · 动画结束后真正调 props.onClose
                    startLeave(tab.tabId);
                  }}
                >
                  ×
                </button>
              </div>
            );
          }}
        </For>
      </div>

      <button
        type="button"
        class="vs-terminal-new-tab"
        onClick={props.onCreate}
        aria-label="新建 Terminal Tab"
      >
        +
      </button>
    </div>
  );
};
