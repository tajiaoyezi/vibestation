import { type Component } from "solid-js";

interface DetachedPlaceholderProps {
  /** 被 detach 的 Pane ID */
  paneId: string;
  /** detached 窗口 label */
  windowLabel: string;
  /** 点击 placeholder 时回调（focus detached window） */
  onFocusDetached?: () => void;
}

/**
 * MVP-17 Phase C · Pane Detach 占位组件
 *
 * Pane detached 后在原位置显示：
 * - 灰底 + 中央 external-link icon + "Pane detached" 文字
 * - 提示行 "Detached window is open. Close to bring back."
 * - 整块可点 → focus detached window
 */
export const DetachedPlaceholder: Component<DetachedPlaceholderProps> = (
  props,
) => {
  return (
    <div
      class="vs-pane-detached-placeholder"
      role="button"
      tabindex={0}
      aria-label={`Pane ${props.paneId} detached. Click to focus.`}
      onClick={() => props.onFocusDetached?.()}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          props.onFocusDetached?.();
        }
      }}
      data-pane-id={props.paneId}
      data-window-label={props.windowLabel}
    >
      <div class="vs-pane-detached-placeholder__content">
        {/* external-link icon · inline SVG · 不引新 dep */}
        <svg
          class="vs-pane-detached-placeholder__icon"
          width="32"
          height="32"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" />
          <polyline points="15 3 21 3 21 9" />
          <line x1="10" y1="14" x2="21" y2="3" />
        </svg>
        <p class="vs-pane-detached-placeholder__title">Pane detached</p>
        <p class="vs-pane-detached-placeholder__hint">
          Detached window is open. Close to bring back.
        </p>
      </div>
    </div>
  );
};
