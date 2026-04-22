import { type Component, Show } from "solid-js";
import type { WorkspaceMetadata } from "../bindings";
import { Terminal } from "../panels/Terminal";

interface MainContentProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  onCloseWorkspaceView: (workspaceId: string) => void;
  workspaces: () => WorkspaceMetadata[];
}

export const MainContent: Component<MainContentProps> = (props) => {
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
        <Terminal
          activeWorkspace={props.activeWorkspace}
          onCloseWorkspaceView={props.onCloseWorkspaceView}
          workspaces={props.workspaces}
        />
      </Show>
    </section>
  );
};
