import {
  createEffect,
  createSignal,
  For,
  Match,
  Show,
  Switch,
  type Component,
} from "solid-js";
import type {
  FileChange,
  GitStatusCollapseRequest,
  GitStatusGroup,
  GitStatusPanelSettings,
  GitStatusRequest,
  GitStatusResponse,
  WorkspaceMetadata,
} from "../../bindings";
import {
  getPanelSettings,
  queryStatus,
  refreshStatus,
  setGroupCollapsed,
} from "./gitStatusApi";

interface GitStatusPanelProps {
  activeWorkspace: () => WorkspaceMetadata | null;
}

type GroupKey = "staged" | "unstaged" | "untracked";

const EMPTY_STATUS: GitStatusResponse = {
  staged: [],
  unstaged: [],
  untracked: [],
};

const DEFAULT_SETTINGS: GitStatusPanelSettings = {
  stagedCollapsed: false,
  unstagedCollapsed: false,
  untrackedCollapsed: false,
};

const GROUPS: {
  key: GroupKey;
  title: string;
  binding: GitStatusGroup;
}[] = [
  { key: "staged", title: "Staged", binding: "staged" },
  { key: "unstaged", title: "Unstaged", binding: "unstaged" },
  { key: "untracked", title: "Untracked", binding: "untracked" },
];

export const GitStatusPanel: Component<GitStatusPanelProps> = (props) => {
  const [status, setStatus] = createSignal<GitStatusResponse>(EMPTY_STATUS);
  const [settings, setSettings] =
    createSignal<GitStatusPanelSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [lastUpdated, setLastUpdated] = createSignal<Date | null>(null);

  const workspaceId = () => props.activeWorkspace()?.workspaceId ?? "";
  const hasGit = () => props.activeWorkspace()?.hasGit ?? false;

  const request = (): GitStatusRequest => ({
    workspaceId: workspaceId(),
  });

  const groupItems = (group: GroupKey): FileChange[] => status()[group];

  const isCollapsed = (group: GroupKey): boolean => {
    const panelSettings = settings();
    switch (group) {
      case "staged":
        return panelSettings.stagedCollapsed;
      case "unstaged":
        return panelSettings.unstagedCollapsed;
      case "untracked":
        return panelSettings.untrackedCollapsed;
    }
  };

  const setCollapsedLocal = (group: GroupKey, collapsed: boolean) => {
    setSettings((prev) => {
      switch (group) {
        case "staged":
          return { ...prev, stagedCollapsed: collapsed };
        case "unstaged":
          return { ...prev, unstagedCollapsed: collapsed };
        case "untracked":
          return { ...prev, untrackedCollapsed: collapsed };
      }
    });
  };

  const clearPanelState = () => {
    setStatus(EMPTY_STATUS);
    setSettings(DEFAULT_SETTINGS);
    setError(null);
    setLastUpdated(null);
  };

  const loadWorkspace = async (mode: "query" | "refresh" = "query") => {
    if (!workspaceId() || !hasGit()) {
      clearPanelState();
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const fetchStatus = mode === "refresh" ? refreshStatus : queryStatus;
      const [response, panelSettings] = await Promise.all([
        fetchStatus(request()),
        getPanelSettings(workspaceId()),
      ]);
      setStatus(response);
      setSettings(panelSettings);
      setLastUpdated(new Date());
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    void loadWorkspace();
  });

  const handleRefresh = async () => {
    await loadWorkspace("refresh");
  };

  const handleToggleGroup = async (
    group: GroupKey,
    binding: GitStatusGroup,
  ) => {
    const next = !isCollapsed(group);
    const previous = isCollapsed(group);
    setCollapsedLocal(group, next);

    const req: GitStatusCollapseRequest = {
      workspaceId: workspaceId(),
      group: binding,
      collapsed: next,
    };

    try {
      await setGroupCollapsed(req);
    } catch (err) {
      setCollapsedLocal(group, previous);
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  const statusClass = (statusCode: string): string => {
    switch (statusCode) {
      case "A":
        return "vs-git-status-code-A";
      case "M":
        return "vs-git-status-code-M";
      case "D":
        return "vs-git-status-code-D";
      case "R":
        return "vs-git-status-code-R";
      case "?":
        return "vs-git-status-code-unknown";
      default:
        return "vs-git-status-code-neutral";
    }
  };

  const formatStats = (file: FileChange): string | null => {
    if (file.status === "?") return null;

    const parts: string[] = [];
    if (file.additions > 0) parts.push(`+${file.additions}`);
    if (file.deletions > 0) parts.push(`-${file.deletions}`);
    return parts.length > 0 ? parts.join(" ") : "0";
  };

  const lastUpdatedLabel = (): string => {
    const value = lastUpdated();
    if (!value) return "not loaded";
    return value.toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  return (
    <Switch>
      <Match when={!props.activeWorkspace()}>
        <div class="vs-git-status-empty">
          <p class="vs-placeholder-text">
            Select a workspace to inspect status
          </p>
        </div>
      </Match>

      <Match when={!hasGit()}>
        <div class="vs-git-status-empty">
          <p class="vs-placeholder-text">No git repository found</p>
          <p class="vs-placeholder-text">
            Open a workspace containing a <code>.git</code> folder
          </p>
        </div>
      </Match>

      <Match when={true}>
        <div class="vs-git-status">
          <div class="vs-git-status-toolbar">
            <div class="vs-git-status-summary">
              <span class="vs-git-status-summary-item">
                {status().staged.length} staged
              </span>
              <span class="vs-git-status-summary-item">
                {status().unstaged.length} unstaged
              </span>
              <span class="vs-git-status-summary-item">
                {status().untracked.length} untracked
              </span>
            </div>
            <div class="vs-git-status-toolbar-right">
              <span class="vs-git-status-updated">
                updated {lastUpdatedLabel()}
              </span>
              <button
                type="button"
                class="vs-git-status-refresh"
                onClick={() => void handleRefresh()}
                disabled={loading()}
              >
                Refresh
              </button>
            </div>
          </div>

          <Show when={error()}>
            <div class="vs-git-status-error" role="alert">
              <div>{error()}</div>
              <div class="vs-git-status-error-hint">
                If repository health is suspect, try{" "}
                <code>git fsck --full</code>.
              </div>
            </div>
          </Show>

          <div class="vs-git-status-list">
            <Show
              when={
                !loading() ||
                status().staged.length +
                  status().unstaged.length +
                  status().untracked.length >
                  0
              }
              fallback={<div class="vs-git-status-loading">Loading...</div>}
            >
              <For each={GROUPS}>
                {(group) => (
                  <section class="vs-git-status-group">
                    <button
                      type="button"
                      class="vs-git-status-group-head"
                      onClick={() =>
                        void handleToggleGroup(group.key, group.binding)
                      }
                      aria-expanded={!isCollapsed(group.key)}
                    >
                      <div class="vs-git-status-group-titlewrap">
                        <span class="vs-git-status-chevron" aria-hidden="true">
                          {isCollapsed(group.key) ? "▸" : "▾"}
                        </span>
                        <span class="vs-git-status-group-title">
                          {group.title}
                        </span>
                      </div>
                      <span class="vs-git-status-group-count">
                        {groupItems(group.key).length}
                      </span>
                    </button>

                    <Show when={!isCollapsed(group.key)}>
                      <Show
                        when={groupItems(group.key).length > 0}
                        fallback={
                          <div class="vs-git-status-group-empty">
                            Nothing here
                          </div>
                        }
                      >
                        <div class="vs-git-status-items" role="list">
                          <For each={groupItems(group.key)}>
                            {(file) => (
                              <div class="vs-git-status-item" role="listitem">
                                <span
                                  class={`vs-git-status-code ${statusClass(file.status)}`}
                                >
                                  {file.status}
                                </span>
                                <span
                                  class="vs-git-status-path"
                                  title={file.path}
                                >
                                  {file.path}
                                </span>
                                <Show when={formatStats(file)}>
                                  {(stats) => (
                                    <span class="vs-git-status-stats">
                                      {stats()}
                                    </span>
                                  )}
                                </Show>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>
                    </Show>
                  </section>
                )}
              </For>
            </Show>
          </div>
        </div>
      </Match>
    </Switch>
  );
};
