import {
  createSignal,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import type { BranchInfo } from "../../bindings";

interface BranchTreeRowProps {
  branch: BranchInfo;
  guide: string;
  active: boolean;
  deleteDisabledReason?: string;
  checkoutDisabledReason?: string;
  onCheckout: () => void;
  onDelete: () => void;
  onCreateFrom: () => void;
}

// Module-level event name for mutual exclusion across BranchTreeRow instances.
// 触发场景：任一 row 右键 → 所有其他 row 的 menu 关闭（含自己 · 然后自己立即重 set）。
// Bug 修复 (2026-05-04)：旧实现仅 listen "click" · 右键不触发 click · 切 row 右键时旧 menu 残留。
const BRANCH_CONTEXT_MENU_OPEN_EVENT = "vs:branch-context-menu-open";

export const BranchTreeRow: Component<BranchTreeRowProps> = (props) => {
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
  const [acknowledged, setAcknowledged] = createSignal(false);
  let ackTimer: ReturnType<typeof setTimeout> | undefined;

  const closeMenu = () => setMenu(null);

  onMount(() => {
    document.addEventListener("click", closeMenu);
    document.addEventListener("keydown", handleDocumentKeyDown);
    document.addEventListener(BRANCH_CONTEXT_MENU_OPEN_EVENT, closeMenu);
  });

  onCleanup(() => {
    document.removeEventListener("click", closeMenu);
    document.removeEventListener("keydown", handleDocumentKeyDown);
    document.removeEventListener(BRANCH_CONTEXT_MENU_OPEN_EVENT, closeMenu);
    if (ackTimer) {
      clearTimeout(ackTimer);
    }
  });

  function handleDocumentKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closeMenu();
    }
  }

  const flashAcknowledged = () => {
    setAcknowledged(true);
    if (ackTimer) {
      clearTimeout(ackTimer);
    }
    ackTimer = setTimeout(() => setAcknowledged(false), 150);
  };

  const handleClick = () => {
    if (props.active) {
      flashAcknowledged();
      return;
    }
    if (props.checkoutDisabledReason) {
      return;
    }
    if (props.branch.kind === "remote") {
      return;
    }
    props.onCheckout();
  };

  // 阻止右键 mousedown 触发 webview default text selection（user-select: none 不够 ·
  // mousedown 阶段 webview 已 select word · contextmenu 后续 fire 时 selection 已发生）。
  const handleMouseDown = (event: MouseEvent) => {
    if (event.button === 2) {
      event.preventDefault();
    }
  };

  const handleContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    // 双保险：清掉任何残留 selection（防御性 · 若 mouseDown preventDefault 因焦点/平台差异未拦住）。
    window.getSelection()?.removeAllRanges();
    // 通知所有 row（含自己）关闭已有 menu · 然后立即 set 自己新 menu · 实现 mutual exclusion。
    document.dispatchEvent(new CustomEvent(BRANCH_CONTEXT_MENU_OPEN_EVENT));
    setMenu({ x: event.clientX, y: event.clientY });
  };

  const runMenuAction = (action: () => void) => {
    closeMenu();
    action();
  };

  const badge = () => {
    const parts: string[] = [];
    if (props.branch.ahead > 0) {
      parts.push(`↑${props.branch.ahead}`);
    }
    if (props.branch.behind > 0) {
      parts.push(`↓${props.branch.behind}`);
    }
    return parts.join(" ");
  };

  return (
    <div
      classList={{
        "vs-branch-row": true,
        active: props.active,
        acknowledged: acknowledged(),
        "is-tag": props.branch.kind === "tag",
      }}
      title={props.checkoutDisabledReason ?? props.branch.fullRef}
      onClick={handleClick}
      onDblClick={() => {
        if (props.branch.kind === "remote" && !props.active) {
          props.onCheckout();
        }
      }}
      onMouseDown={handleMouseDown}
      onContextMenu={handleContextMenu}
    >
      <span class="vs-branch-guide">{props.guide}</span>
      <span class={`vs-ref-dot ${props.branch.kind}`} />
      <span class="vs-branch-name">{props.branch.name}</span>
      <Show when={props.active}>
        <span class="vs-branch-badge">current</span>
      </Show>
      <Show when={badge()}>
        <span class="vs-branch-badge">{badge()}</span>
      </Show>

      <Show when={menu()}>
        {(position) => (
          <div
            class="vs-branch-context-menu"
            style={{
              left: `${position().x}px`,
              top: `${position().y}px`,
            }}
            onClick={(event) => event.stopPropagation()}
            role="menu"
          >
            <button
              type="button"
              role="menuitem"
              onClick={() => runMenuAction(props.onCreateFrom)}
            >
              New branch from here
            </button>
            <button
              type="button"
              role="menuitem"
              onClick={() => runMenuAction(props.onCheckout)}
              disabled={Boolean(props.checkoutDisabledReason)}
              title={props.checkoutDisabledReason}
            >
              Checkout
            </button>
            <button
              type="button"
              role="menuitem"
              class="danger"
              onClick={() => runMenuAction(props.onDelete)}
              disabled={Boolean(props.deleteDisabledReason)}
              title={props.deleteDisabledReason}
            >
              Delete branch
            </button>
          </div>
        )}
      </Show>
    </div>
  );
};
