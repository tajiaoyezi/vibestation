import { type Component, Show } from "solid-js";
import type { WorkspaceMetadata } from "../bindings";
import { t, normalizeLanguage } from "../i18n";
import { DiffPanel } from "../panels/Diff";
import { Terminal } from "../panels/Terminal";
import { useSettings } from "../stores/settings";

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
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());

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
      aria-label={label("chrome.main.contentArea")}
      tabindex={0}
    >
      <Show
        when={props.activeWorkspace() !== null}
        fallback={
          <div class="vs-welcome-inner">
            <p class="vs-welcome-hint">{label("chrome.main.emptyWorkspace")}</p>
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
                    <span class="vs-panel-title">
                      {label("chrome.main.diff")}
                    </span>
                    <div class="vs-panel-actions">
                      <button
                        type="button"
                        class="vs-btn-secondary vs-main-diff-back"
                        onClick={props.onCloseDiff}
                      >
                        {label("chrome.main.backToTerminal")}
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
