/**
 * PaneSplitter · MVP-05 Phase C Track B
 *
 * 1px 浅色分隔条 · hover 主色 + resize 光标。
 * §D 双击复位 50/50 · 拖拽改 ratio + 60FPS rAF 节流 + Esc 取消 + clamp [0.1, 0.9]。
 *
 * 拖拽中绕过 SolidJS reactive：直接写父 `.vs-pane-split` 的 CSS var
 * `--vs-pane-ratio` 实时改 flex · 0 reconcile。释放鼠标才调 `onDragEnd`
 * 触发 caller 调 `pane_split_ratio_update` IPC + reload PaneListResponse ·
 * SolidJS 重渲染同步 ratio。
 */
import { onCleanup, type Component } from "solid-js";
import type { SplitDir } from "../../bindings";

type PaneSplitterProps = {
  direction: SplitDir;
  ratio: number;
  parentPaneId: string;
  onDragEnd?: (parentPaneId: string, ratio: number) => void;
};

// 拖拽边界 · 防 0 宽 pane（与 backend `validate_mvp_05` 的 ratio > 0 && < 1 一致 ·
// 前端额外加 0.1 / 0.9 边距是 UX clamp · 不和 backend 决策耦合）。
const RATIO_MIN = 0.1;
const RATIO_MAX = 0.9;

const clampRatio = (value: number): number =>
  Math.min(RATIO_MAX, Math.max(RATIO_MIN, value));

export const PaneSplitter: Component<PaneSplitterProps> = (props) => {
  // 拖拽状态 · 用 let 而非 createSignal 是有意的 · 拖拽中频繁更新如果走
  // SolidJS reactive 会触发 reconcile · 60FPS 下成本不可接受。
  let isDragging = false;
  let pointerId = -1;
  let startRatio = 0;
  let pendingRatio = props.ratio;
  let containerEl: HTMLElement | null = null;
  let splitterEl: HTMLDivElement | undefined;
  let rafId: number | null = null;

  const writeRatio = (ratio: number): void => {
    if (!containerEl) return;
    containerEl.style.setProperty("--vs-pane-ratio", ratio.toFixed(3));
    // 拖拽中绕过 SolidJS reactive 直接 setAttribute · 拖拽结束 props.ratio 更新后
    // SolidJS 重渲染会再次写入正确的 aria-valuenow（基于 props.ratio）。
    splitterEl?.setAttribute("aria-valuenow", String(Math.round(ratio * 100)));
  };

  const computeRatio = (clientX: number, clientY: number): number => {
    if (!containerEl) return startRatio;
    const rect = containerEl.getBoundingClientRect();
    const raw =
      props.direction === "horizontal"
        ? (clientX - rect.left) / rect.width
        : (clientY - rect.top) / rect.height;
    return clampRatio(raw);
  };

  const stopRaf = (): void => {
    if (rafId !== null) {
      cancelAnimationFrame(rafId);
      rafId = null;
    }
  };

  const cleanupDrag = (): void => {
    stopRaf();
    if (splitterEl) {
      splitterEl.classList.remove("is-dragging");
      if (pointerId >= 0 && splitterEl.hasPointerCapture(pointerId)) {
        splitterEl.releasePointerCapture(pointerId);
      }
    }
    window.removeEventListener("keydown", handleKeyDown);
    isDragging = false;
    pointerId = -1;
    containerEl = null;
  };

  const finishDrag = (commit: boolean): void => {
    if (!isDragging) return;
    const finalRatio = pendingRatio;
    const initialRatio = startRatio;
    cleanupDrag();
    if (commit && Math.abs(finalRatio - initialRatio) > 0.001) {
      props.onDragEnd?.(props.parentPaneId, finalRatio);
    } else if (!commit) {
      // Esc / cancel · 把 DOM CSS var 还原到起始值 · 否则用户看到的视觉
      // 状态会停在最后一帧 pendingRatio · 与 SolidJS 的 props.ratio 不一致。
      writeRatio(initialRatio);
    }
  };

  const handlePointerDown = (event: PointerEvent): void => {
    // 只接受主键（鼠标左键 / 触摸 / 笔）· 防右键 / 中键误触。
    if (event.button !== 0) return;
    event.preventDefault();
    splitterEl = event.currentTarget as HTMLDivElement;
    // splitter 是 .vs-pane-split 的直接子节点 · parentElement 即为容器。
    // 用 closest 兼容未来嵌套结构变化 · 但当前 PaneSplitView 是直接父子。
    const container = splitterEl.parentElement?.closest(".vs-pane-split");
    if (!(container instanceof HTMLElement)) return;
    containerEl = container;
    isDragging = true;
    pointerId = event.pointerId;
    startRatio = props.ratio;
    pendingRatio = props.ratio;
    splitterEl.setPointerCapture(event.pointerId);
    splitterEl.classList.add("is-dragging");
    window.addEventListener("keydown", handleKeyDown);
  };

  const handlePointerMove = (event: PointerEvent): void => {
    if (!isDragging) return;
    pendingRatio = computeRatio(event.clientX, event.clientY);
    if (rafId === null) {
      // 60FPS 节流 · 多次 PointerMove 合并到一帧 · 避免每个 mousemove 写 DOM。
      rafId = requestAnimationFrame(() => {
        rafId = null;
        writeRatio(pendingRatio);
      });
    }
  };

  const handlePointerUp = (event: PointerEvent): void => {
    if (!isDragging || event.pointerId !== pointerId) return;
    // PointerUp 之前可能还有 pending rAF · 强制刷一次确保 DOM 状态等于 pendingRatio ·
    // 否则 commit 的 ratio 和视觉最后一帧不一致。
    stopRaf();
    writeRatio(pendingRatio);
    finishDrag(true);
  };

  const handlePointerCancel = (event: PointerEvent): void => {
    if (!isDragging || event.pointerId !== pointerId) return;
    finishDrag(false);
  };

  const handleKeyDown = (event: KeyboardEvent): void => {
    if (event.key === "Escape" && isDragging) {
      event.preventDefault();
      finishDrag(false);
    }
  };

  const handleDoubleClick = (): void => {
    // §D 双击复位 50/50 · 拖拽中不响应（pointer 优先）。
    if (isDragging) return;
    props.onDragEnd?.(props.parentPaneId, 0.5);
  };

  /**
   * MVP-14 Phase C §a11y · splitter 键盘 resize ·
   * - 普通 ArrowLeft/Right (horizontal splitter) · ArrowUp/Down (vertical splitter) → ±1% step
   * - Shift+Arrow → ±5% step
   * - Home / End → 跳到最小 / 最大
   * - Enter / Space → 双击复位 50/50
   *
   * 仅 splitter focus 时响应 · 不全局监听 · 防止干扰 PTY / 输入。
   */
  const handleSplitterKeyDown = (event: KeyboardEvent): void => {
    if (isDragging) return;

    const horizontal = props.direction === "horizontal";
    const decreaseKey = horizontal ? "ArrowLeft" : "ArrowUp";
    const increaseKey = horizontal ? "ArrowRight" : "ArrowDown";
    const step = event.shiftKey ? 0.05 : 0.01;

    let target: number | null = null;
    if (event.key === decreaseKey) {
      target = clampRatio(props.ratio - step);
    } else if (event.key === increaseKey) {
      target = clampRatio(props.ratio + step);
    } else if (event.key === "Home") {
      target = RATIO_MIN;
    } else if (event.key === "End") {
      target = RATIO_MAX;
    } else if (event.key === "Enter" || event.key === " ") {
      target = 0.5;
    }

    if (target === null) return;
    event.preventDefault();
    event.stopPropagation();
    if (Math.abs(target - props.ratio) > 0.001) {
      props.onDragEnd?.(props.parentPaneId, target);
    }
  };

  // 组件卸载时清理 · 防 worker tab 切换 / route 切换造成的悬挂 listener。
  onCleanup(() => {
    cleanupDrag();
  });

  return (
    <div
      ref={splitterEl}
      class={`vs-pane-splitter vs-pane-splitter-${props.direction}`}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      onPointerCancel={handlePointerCancel}
      onDblClick={handleDoubleClick}
      onKeyDown={handleSplitterKeyDown}
      role="separator"
      aria-orientation={
        props.direction === "horizontal" ? "vertical" : "horizontal"
      }
      aria-valuenow={Math.round(props.ratio * 100)}
      aria-valuemin={Math.round(RATIO_MIN * 100)}
      aria-valuemax={Math.round(RATIO_MAX * 100)}
      aria-label={
        props.direction === "horizontal"
          ? "Pane 横向分隔条 · 方向键 ←/→ 调整宽度"
          : "Pane 纵向分隔条 · 方向键 ↑/↓ 调整高度"
      }
      tabindex={0}
    />
  );
};
