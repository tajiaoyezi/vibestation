import { For, Show, type Component } from "solid-js";
import type { GitLogEntry } from "../../bindings";

export interface GitLogContextMenuState {
  x: number;
  y: number;
  entry: GitLogEntry;
  selectedEntries: GitLogEntry[];
  branchRef: string | null;
}

interface GitLogContextMenuProps {
  state: GitLogContextMenuState;
  currentBranch: string | null;
  onClose: () => void;
  onCherryPickCommit: (entry: GitLogEntry) => void;
  onCherryPickRange: (entries: GitLogEntry[]) => void;
  onRebaseOnto: (branchRef: string) => void;
  onInteractiveRebase: (branchRef: string) => void;
  onMerge: (branchRef: string) => void;
}

export const GitLogContextMenu: Component<GitLogContextMenuProps> = (props) => {
  const selected = () => props.state.selectedEntries;
  const isRange = () => selected().length > 1;
  const current = () => props.currentBranch ?? "current";

  return (
    <div
      class="vs-git-log-context-layer"
      onClick={props.onClose}
      onContextMenu={(event) => {
        event.preventDefault();
        props.onClose();
      }}
    >
      <div
        class="vs-git-log-context-menu"
        style={{ left: `${props.state.x}px`, top: `${props.state.y}px` }}
        role="menu"
        onClick={(event) => event.stopPropagation()}
      >
        <Show
          when={isRange()}
          fallback={
            <ContextMenuButton
              label="Cherry-pick onto current branch"
              onClick={() => props.onCherryPickCommit(props.state.entry)}
            />
          }
        >
          <ContextMenuButton
            label={`Cherry-pick ${selected().length} commits onto current branch`}
            onClick={() => props.onCherryPickRange(selected())}
          />
          <ContextMenuButton
            label="Squash these commits"
            disabled
            title="v0.4+ 范围 · 暂不可用"
          />
        </Show>

        <ContextMenuButton
          label="Reset to here"
          disabled
          title="v1.0 范围 · 暂不可用"
        />
        <ContextMenuButton
          label="Revert this commit"
          disabled
          title="v0.4+ 范围 · 暂不可用"
        />

        <Show when={props.state.branchRef}>
          {(branchRef) => (
            <>
              <div class="vs-git-log-context-separator" />
              <ContextMenuButton
                label={`Rebase ${current()} onto ${branchRef()}`}
                onClick={() => props.onRebaseOnto(branchRef())}
              />
              <ContextMenuButton
                label="Interactive rebase from here"
                onClick={() => props.onInteractiveRebase(branchRef())}
              />
              <ContextMenuButton
                label={`Merge this into ${current()}`}
                onClick={() => props.onMerge(branchRef())}
              />
            </>
          )}
        </Show>

        <Show when={isRange()}>
          <div class="vs-git-log-context-selection">
            <For each={selected().slice(0, 4)}>
              {(entry) => <span>{entry.shortSha}</span>}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

const ContextMenuButton: Component<{
  label: string;
  disabled?: boolean;
  title?: string;
  onClick?: () => void;
}> = (props) => (
  <button
    type="button"
    role="menuitem"
    disabled={props.disabled}
    title={props.title}
    onClick={() => props.onClick?.()}
  >
    {props.label}
  </button>
);
