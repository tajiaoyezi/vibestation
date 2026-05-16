/**
 * D.4 · Pane link creation affordance.
 *
 * Opened from the `.vs-pane-actions` link button on a pane. Lists the other
 * panes in the tab as candidate AI parents; picking one links *this* pane's
 * failures (child) to that AI pane (parent) with kind `failureFeedback`.
 *
 * LinkManagePopover (#353) only manages existing links — this is the missing
 * create entry point so the feature has an actual starting affordance.
 * Mirrors LinkManagePopover's dialog / a11y / listener idiom.
 */
import { type Component, For, Show, createSignal, onCleanup } from "solid-js";
import type { PaneLinkKind } from "../../bindings";

export type PaneLinkCreateCandidate = {
  paneId: string;
  label: string;
};

export type PaneLinkCreateRequest = {
  parentPaneId: string;
  childPaneId: string;
  linkKind: PaneLinkKind;
};

export type PaneLinkCreateMenuProps = {
  open: boolean;
  currentPaneId: string;
  candidatePanes: PaneLinkCreateCandidate[];
  onClose: () => void;
  onCreate: (req: PaneLinkCreateRequest) => void;
};

export const PaneLinkCreateMenu: Component<PaneLinkCreateMenuProps> = (
  props,
) => {
  let menuRef: HTMLDivElement | undefined;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      props.onClose();
    }
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (menuRef && !menuRef.contains(e.target as Node)) {
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
    menuRef = el;
    if (props.open) {
      setupListeners();
      setReady(true);
    }
  };

  onCleanup(teardownListeners);

  const pick = (parentPaneId: string) => {
    props.onCreate({
      parentPaneId,
      childPaneId: props.currentPaneId,
      linkKind: "failureFeedback",
    });
  };

  return (
    <Show when={props.open}>
      <div
        ref={onRef}
        class="vs-link-create-menu"
        role="dialog"
        aria-label="Link this pane's failures to an AI pane"
        aria-modal="true"
      >
        <div class="vs-link-create-menu-header">
          <span class="vs-link-create-menu-title">Link failures to…</span>
          <button
            type="button"
            class="vs-link-create-menu-close"
            aria-label="Close link creation"
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
          when={props.candidatePanes.length > 0}
          fallback={
            <div class="vs-link-create-menu-empty" role="status">
              No other panes to link
            </div>
          }
        >
          <ul class="vs-link-create-menu-list" role="list">
            <For each={props.candidatePanes}>
              {(candidate) => (
                <li class="vs-link-create-menu-item">
                  <button
                    type="button"
                    class="vs-link-create-menu-option"
                    aria-label={`Link failures to ${candidate.label}`}
                    onClick={() => pick(candidate.paneId)}
                  >
                    <span
                      class="vs-link-create-menu-option-icon"
                      aria-hidden="true"
                    >
                      →
                    </span>
                    <span class="vs-link-create-menu-option-label">
                      {candidate.label}
                    </span>
                  </button>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </div>
    </Show>
  );
};
