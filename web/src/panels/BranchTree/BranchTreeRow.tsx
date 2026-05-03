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

export const BranchTreeRow: Component<BranchTreeRowProps> = (props) => {
  const [menu, setMenu] = createSignal<{ x: number; y: number } | null>(null);
  const [acknowledged, setAcknowledged] = createSignal(false);
  let ackTimer: ReturnType<typeof setTimeout> | undefined;

  const closeMenu = () => setMenu(null);

  onMount(() => {
    document.addEventListener("click", closeMenu);
    document.addEventListener("keydown", handleDocumentKeyDown);
  });

  onCleanup(() => {
    document.removeEventListener("click", closeMenu);
    document.removeEventListener("keydown", handleDocumentKeyDown);
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

  const handleContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
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
