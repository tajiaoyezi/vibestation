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
  /**
   * 拖拽调换顺序 · drop 完成时调用。`newIndex` 是 0-based 目标位置。
   * 上层应调 `tab_reorder` IPC 持久化 + 更新前端 store。
   */
  onReorder?: (tabId: string, newIndex: number) => void;
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
  // 拖拽中的 tab id · 拖拽时 ghost 半透明
  const [draggedTabId, setDraggedTabId] = createSignal<string | null>(null);
  // 鼠标 hover 的 drop target tab id + 插入位置（before/after）· 显示 vertical insertion line
  const [dragOverInfo, setDragOverInfo] = createSignal<{
    tabId: string;
    position: "before" | "after";
  } | null>(null);
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
    activeResizeObserver?.disconnect();
    removePointerHandlers();
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
  });

  // ===== Pointer-events 拖拽调换顺序 =====
  // 不用 HTML5 drag/drop · 因为 Tauri WebKit 对 drag events 限制（dragstart 后
  // dragover/drop 不 fire · 实测）。改用 pointer events 自定义实现 · VS Code / Notion 用法。
  const DRAG_THRESHOLD_PX = 5; // 鼠标移动超过这个才进入 drag 模式 · 防误触
  let dragStartX = 0;
  let dragStartedTabId: string | null = null; // pointerdown 时记 · move > 阈值才升 draggedTabId
  let pointerHandlersAttached = false;

  const removePointerHandlers = () => {
    if (!pointerHandlersAttached) return;
    pointerHandlersAttached = false;
    window.removeEventListener("pointermove", handlePointerMove);
    window.removeEventListener("pointerup", handlePointerUp);
    window.removeEventListener("pointercancel", handlePointerUp);
  };

  function handlePointerMove(e: PointerEvent) {
    const startedId = dragStartedTabId;
    if (!startedId) return;
    const dx = e.clientX - dragStartX;
    // 还没进入 drag 模式 · 检查阈值
    if (!draggedTabId()) {
      if (Math.abs(dx) < DRAG_THRESHOLD_PX) return;
      setDraggedTabId(startedId);
      // 进 drag 模式 · 锁定 cursor + 禁用 user-select
      document.body.style.cursor = "grabbing";
      document.body.style.userSelect = "none";
    }

    // 找鼠标下面的 tab（用 elementFromPoint · 避开 button/label children）
    const targetEl = document.elementFromPoint(e.clientX, e.clientY);
    const tabEl = targetEl?.closest<HTMLElement>("[data-tab-id]");
    if (!tabEl || !scrollContainer?.contains(tabEl)) {
      setDragOverInfo(null);
      return;
    }
    const overTabId = tabEl.dataset.tabId;
    if (!overTabId || overTabId === startedId) {
      setDragOverInfo(null);
      return;
    }
    // 计算 before/after
    const rect = tabEl.getBoundingClientRect();
    const isAfter = e.clientX > rect.left + rect.width / 2;
    const next = {
      tabId: overTabId,
      position: isAfter ? ("after" as const) : ("before" as const),
    };
    const cur = dragOverInfo();
    if (!cur || cur.tabId !== next.tabId || cur.position !== next.position) {
      setDragOverInfo(next);
    }
  }

  function handlePointerUp(_e: PointerEvent) {
    const sourceId = draggedTabId();
    const info = dragOverInfo();

    // 清状态 · 恢复 body cursor
    document.body.style.cursor = "";
    document.body.style.userSelect = "";
    dragStartedTabId = null;
    setDraggedTabId(null);
    setDragOverInfo(null);
    removePointerHandlers();

    if (!sourceId || !info || sourceId === info.tabId) return;

    const sourceIdx = props.tabs.findIndex((t) => t.tabId === sourceId);
    const targetIdx = props.tabs.findIndex((t) => t.tabId === info.tabId);
    if (sourceIdx === -1 || targetIdx === -1) return;

    let newIdx = info.position === "after" ? targetIdx + 1 : targetIdx;
    if (sourceIdx < newIdx) newIdx -= 1;
    if (newIdx === sourceIdx) return;

    props.onReorder?.(sourceId, newIdx);
  }

  const handlePointerDown = (e: PointerEvent, tabId: string) => {
    // 只响应主键（左键 / 触摸 / 笔）
    if (e.button !== 0) return;
    // editing / leaving 不允许 drag
    if (leavingTabIds().has(tabId) || editingTabId() === tabId) return;
    // 点 close 按钮不触发 drag（close 自己 stopPropagation 不到这里 · 但保险起见）
    const target = e.target as HTMLElement;
    if (target.closest(".vs-terminal-tab-close")) return;

    dragStartX = e.clientX;
    dragStartedTabId = tabId;
    if (!pointerHandlersAttached) {
      pointerHandlersAttached = true;
      window.addEventListener("pointermove", handlePointerMove);
      window.addEventListener("pointerup", handlePointerUp);
      window.addEventListener("pointercancel", handlePointerUp);
    }
  };

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
      // active 暂时找不到 · 保持上次状态 · 不闪一下消失
      return;
    }
    const containerRect = scrollContainer.getBoundingClientRect();
    const tabRect = activeEl.getBoundingClientRect();
    if (tabRect.width <= 0) {
      // tab 还在 enter 动画 width 0→240 中间值 · 跳过本次 update
      return;
    }
    const left = tabRect.left - containerRect.left + scrollContainer.scrollLeft;
    // 所有路径（active 切换 / reorder / enter / resize）都走 CSS 默认 transition
    // （200ms ease）· 让蓝条平滑滑到新位置而不是硬跳。
    // 之前 reorder 路径关 transition 硬跳是因为旧版 enter 用 CSS animation ·
    // SolidJS <For> insertBefore 会让 enter animation 重启 · 让 width 在中间帧
    // 抖动 · 跟着 transition 视觉错乱。改成 CSS transition 实现 enter 后 ·
    // reorder 时 active tab 的 max-width/padding 不再变化 · width 稳定 ·
    // indicator transition 不会跟错值 · 可以放心走 transition。
    indicator.style.left = `${left}px`;
    indicator.style.width = `${tabRect.width}px`;
    indicator.style.opacity = "1";
    if (!indicatorReady()) setIndicatorReady(true);
  };

  // 监听 active 切换 + tabs 列表变化（含 reorder · 长度不变但顺序变） + leaving 变化
  // 用 ResizeObserver 监听 active tab 实际 size 变化（含 enter/reorder 动画过渡的中间值）·
  // 配合 effect 的双 rAF 双管齐下 · 不再用 setTimeout 重算（多 timer 累积可能造成闪烁）
  let activeResizeObserver: ResizeObserver | undefined;
  createEffect(() => {
    // track signals · join tab id 序列让顺序变化也 trigger（length 不够）
    void props.activeTabId;
    void props.tabs.map((t) => t.tabId).join("|");
    void leavingTabIds();
    // 双 rAF 等 SolidJS DOM commit + paint · reorder 后 tab DOM 移到新位置 · 第二帧 rect 准确
    requestAnimationFrame(() => requestAnimationFrame(() => updateIndicator()));
    // 重新挂 ResizeObserver 到当前 active tab · 监听 size 变化（含 enter 动画 width 0→240）
    activeResizeObserver?.disconnect();
    if (typeof ResizeObserver !== "undefined" && scrollContainer) {
      const activeId = props.activeTabId;
      if (activeId) {
        const activeEl = scrollContainer.querySelector<HTMLDivElement>(
          `[data-tab-id="${activeId}"]:not(.is-leaving)`,
        );
        if (activeEl) {
          activeResizeObserver = new ResizeObserver(() => {
            requestAnimationFrame(() => updateIndicator());
          });
          activeResizeObserver.observe(activeEl);
        }
      }
    }
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

            const dragOver = () => dragOverInfo();
            const isDragging = () => draggedTabId() === tab.tabId;
            const isDropBefore = () =>
              dragOver()?.tabId === tab.tabId &&
              dragOver()?.position === "before" &&
              !isDragging();
            const isDropAfter = () =>
              dragOver()?.tabId === tab.tabId &&
              dragOver()?.position === "after" &&
              !isDragging();

            // mount 瞬间用 .is-entering 提供 compressed 起始态 · 双 rAF 后移除
            // 触发 CSS transition 展开 · 替代原 CSS animation 实现。
            // 为什么不用 animation：SolidJS <For> reorder 时 insertBefore 会
            // detach/reattach element · CSS animation 按 spec 会从 0% 重启 ·
            // 让 active tab 的 max-width/padding 中间帧抖动 · indicator 跟着漂移。
            // CSS transition 是 property change-based · reattach 不触发 · 一劳永逸。
            const markEntering = (el: HTMLDivElement) => {
              const motionReduced =
                typeof window !== "undefined" &&
                window.matchMedia("(prefers-reduced-motion: reduce)").matches;
              if (motionReduced) {
                // a11y · reduced motion 用户不要 enter 动画
                return;
              }
              el.classList.add("is-entering");
              requestAnimationFrame(() => {
                requestAnimationFrame(() => {
                  el.classList.remove("is-entering");
                });
              });
            };

            return (
              <div
                ref={markEntering}
                class={`vs-terminal-tab ${active() ? "is-active" : ""} ${leaving() ? "is-leaving" : ""} ${isDragging() ? "is-dragging" : ""} ${isDropBefore() ? "is-drop-before" : ""} ${isDropAfter() ? "is-drop-after" : ""}`}
                role="presentation"
                data-tab-id={tab.tabId}
                onPointerDown={(e) => handlePointerDown(e, tab.tabId)}
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
