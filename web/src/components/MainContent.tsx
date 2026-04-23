import { type Component, Show } from "solid-js";
import type { WorkspaceMetadata } from "../bindings";
import { DiffPanel } from "../panels/Diff";
import { Terminal } from "../panels/Terminal";

export interface DiffTarget {
  workspaceId: string;
  source: string;
  filePath: string;
}

interface MainContentProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  activeDiff: () => DiffTarget | null;
  onCloseDiff: () => void;
  onCloseWorkspaceView: (workspaceId: string) => void;
  workspaces: () => WorkspaceMetadata[];
}

export const MainContent: Component<MainContentProps> = (props) => {
  const currentDiff = () => {
    const target = props.activeDiff();
    const workspace = props.activeWorkspace();
    if (!target || !workspace || workspace.workspaceId !== target.workspaceId) {
      return null;
    }
    return target;
  };

  return (
    <section
      class="vs-main-content"
      role="main"
      aria-label="Main content area"
      tabindex={0}
    >
      <Show
        when={props.activeWorkspace() !== null}
        fallback={
          <div class="vs-welcome-inner">
            <p class="vs-welcome-hint">
              Select or create a workspace to get started
            </p>
          </div>
        }
      >
        <div class="vs-main-stack">
          <Terminal
            activeWorkspace={props.activeWorkspace}
            onCloseWorkspaceView={props.onCloseWorkspaceView}
            workspaces={props.workspaces}
          />
          <Show when={currentDiff()}>
            {(target) => (
              <div class="vs-main-diff-overlay">
                <div class="vs-main-diff">
                  <div class="vs-panel-head">
                    <span class="vs-panel-title">Diff</span>
                    <div class="vs-panel-actions">
                      <button
                        type="button"
                        class="vs-btn-secondary vs-main-diff-back"
                        onClick={props.onCloseDiff}
                      >
                        Back to Terminal
                      </button>
                    </div>
                  </div>
                  <div class="vs-main-diff-body">
                    <DiffPanel
                      workspaceId={target().workspaceId}
                      source={target().source}
                      filePath={target().filePath}
                    />
                  </div>
                </div>
              </div>
            )}
          </Show>
        </div>
      </Show>
    </section>
  );
};
