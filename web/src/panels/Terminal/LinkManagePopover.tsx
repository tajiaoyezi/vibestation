/**
 * D.4 · Link management popover.
 *
 * Shows active links for a pane with enable/disable toggle and unlink action.
 * Opens from PaneLinkChip (D.1) click.
 */
import { type Component, For, Show, createSignal, onCleanup } from "solid-js";
import type { PaneLink } from "../../bindings";

export type LinkManagePopoverProps = {
  open: boolean;
  links: PaneLink[];
  currentPaneId: string;
  onClose: () => void;
  onToggleEnabled?: (linkId: string, enabled: boolean) => void;
  onUnlink?: (linkId: string) => void;
};

const roleFor = (link: PaneLink, currentPaneId: string): "parent" | "child" =>
  link.parentPaneId === currentPaneId ? "parent" : "child";

const targetIdFor = (link: PaneLink, currentPaneId: string): string =>
  link.parentPaneId === currentPaneId ? link.childPaneId : link.parentPaneId;

export const LinkManagePopover: Component<LinkManagePopoverProps> = (props) => {
  let popoverRef: HTMLDivElement | undefined;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      props.onClose();
    }
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (popoverRef && !popoverRef.contains(e.target as Node)) {
      props.onClose();
    }
  };

  const setupListeners = () => {
    document.addEventListener("keydown", handleKeyDown);
    document.addEventListener("mousedown", handleClickOutside);
  };

  const teardownListeners = () => {
    document.removeEventListener("keydown", handleKeyDown);
    document.removeEventListener("mousedown", handleClickOutside);
  };

  const [_ready, setReady] = createSignal(false);

  const onRef = (el: HTMLDivElement) => {
    popoverRef = el;
    if (props.open) {
      setupListeners();
      setReady(true);
    }
  };

  onCleanup(teardownListeners);

  return (
    <Show when={props.open}>
      <div
        ref={onRef}
        class="vs-link-manage-popover"
        role="dialog"
        aria-label="Manage pane links"
        aria-modal="true"
      >
        <div class="vs-link-manage-popover-header">
          <span class="vs-link-manage-popover-title">Pane Links</span>
          <button
            type="button"
            class="vs-link-manage-popover-close"
            aria-label="Close link management"
            onClick={props.onClose}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <line
                x1="4"
                y1="4"
                x2="12"
                y2="12"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
              <line
                x1="12"
                y1="4"
                x2="4"
                y2="12"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
              />
            </svg>
          </button>
        </div>
        <Show
          when={props.links.length > 0}
          fallback={
            <div class="vs-link-manage-popover-empty" role="status">
              No active links
            </div>
          }
        >
          <ul class="vs-link-manage-popover-list" role="list">
            <For each={props.links}>
              {(link) => {
                const role = () => roleFor(link, props.currentPaneId);
                const targetId = () => targetIdFor(link, props.currentPaneId);
                return (
                  <li
                    class="vs-link-manage-popover-item"
                    aria-label={`Link ${link.id}: ${role()} → ${targetId()} · ${link.enabled ? "enabled" : "disabled"}`}
                  >
                    <span class="vs-link-manage-popover-item-info">
                      <span class="vs-link-manage-popover-item-role">
                        {role()}
                      </span>
                      <span class="vs-link-manage-popover-item-target">
                        {targetId()}
                      </span>
                    </span>
                    <span class="vs-link-manage-popover-item-actions">
                      <button
                        type="button"
                        class="vs-link-manage-popover-toggle"
                        role="switch"
                        aria-checked={link.enabled}
                        aria-label={`${link.enabled ? "Disable" : "Enable"} link`}
                        onClick={() =>
                          props.onToggleEnabled?.(link.id, !link.enabled)
                        }
                      >
                        <span
                          class={`vs-link-manage-toggle-track ${link.enabled ? "is-on" : ""}`}
                        >
                          <span class="vs-link-manage-toggle-thumb" />
                        </span>
                      </button>
                      <button
                        type="button"
                        class="vs-link-manage-popover-unlink"
                        aria-label={`Unlink ${targetId()}`}
                        onClick={() => props.onUnlink?.(link.id)}
                      >
                        Unlink
                      </button>
                    </span>
                  </li>
                );
              }}
            </For>
          </ul>
        </Show>
      </div>
    </Show>
  );
};
