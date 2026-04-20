import { type Component, Show } from "solid-js";

interface MainContentProps {
  activeWorkspace: () => {
    name: string;
    path: string;
    hasGit: boolean;
    repoRoot: string | null;
  } | null;
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
        <div class="vs-workspace-placeholder">
          <p class="vs-ws-heading">{props.activeWorkspace()?.name}</p>
          <p class="vs-ws-path">{props.activeWorkspace()?.path}</p>
          <Show when={props.activeWorkspace()?.hasGit}>
            <p class="vs-ws-git-info">
              <span class="vs-git-badge">Git</span>
              <span class="vs-ws-repo-root">
                {props.activeWorkspace()?.repoRoot}
              </span>
            </p>
          </Show>
          <p class="vs-ws-placeholder">
            Tool Windows + Tab 管理由 MVP-03/04 接管
          </p>
        </div>
      </Show>
    </section>
  );
};
