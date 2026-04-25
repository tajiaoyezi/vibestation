import {
  createEffect,
  createSignal,
  For,
  Match,
  onCleanup,
  Show,
  Switch,
  type Component,
} from "solid-js";
import type {
  FileChange,
  FileStatusEvent,
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
  stageFiles,
  subscribeGitStatus,
  unsubscribeGitStatus,
  unstageFiles,
} from "./gitStatusApi";
import { CommitBar } from "../CommitBar";
import type { DiffTarget } from "../../components/MainContent";

interface GitStatusPanelProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpenDiff?: (target: DiffTarget) => void;
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

  const applyStatusEvent = (event: FileStatusEvent) => {
    setStatus({
      staged: event.staged,
      unstaged: event.unstaged,
      untracked: event.untracked,
    });
    setError(null);
    setLoading(false);
    setLastUpdated(new Date());
  };

  const loadWorkspace = async (
    targetWorkspaceId: string,
    mode: "query" | "refresh" = "query",
  ) => {
    if (!targetWorkspaceId) {
      clearPanelState();
      return;
    }

    const req: GitStatusRequest = {
      workspaceId: targetWorkspaceId,
    };

    setLoading(true);
    setError(null);

    try {
      const fetchStatus = mode === "refresh" ? refreshStatus : queryStatus;
      const [response, panelSettings] = await Promise.all([
        fetchStatus(req),
        getPanelSettings(targetWorkspaceId),
      ]);
      if (workspaceId() !== targetWorkspaceId) {
        return;
      }
      setStatus(response);
      setSettings(panelSettings);
      setLastUpdated(new Date());
    } catch (err) {
      if (workspaceId() !== targetWorkspaceId) {
        return;
      }
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    } finally {
      if (workspaceId() === targetWorkspaceId) {
        setLoading(false);
      }
    }
  };

  createEffect(() => {
    const id = workspaceId();
    const gitEnabled = hasGit();

    if (!id) {
      clearPanelState();
      return;
    }

    if (!gitEnabled) {
      clearPanelState();
      return;
    }

    let disposed = false;
    let unlisten: (() => void) | undefined;

    void loadWorkspace(id);

    void (async () => {
      try {
        const stopListening = await subscribeGitStatus(
          id,
          (event: FileStatusEvent) => {
            if (event.workspaceId !== id) {
              return;
            }
            applyStatusEvent(event);
          },
        );

        if (disposed) {
          stopListening();
          return;
        }

        unlisten = stopListening;

        if (disposed) {
          unlisten?.();
          void unsubscribeGitStatus(id);
        }
      } catch (err) {
        if (!disposed) {
          setError(err instanceof Error ? err.message : String(err));
        }
        unlisten?.();
        unlisten = undefined;
      }
    })();

    onCleanup(() => {
      disposed = true;
      unlisten?.();
      void unsubscribeGitStatus(id);
    });
  });

  const handleRefresh = async () => {
    const id = workspaceId();
    if (!id) {
      clearPanelState();
      return;
    }
    await loadWorkspace(id, "refresh");
  };

  // ── Optimistic Stage / Unstage ──

  const moveFileOptimistically = (
    filePath: string,
    from: GroupKey,
    to: GroupKey,
  ) => {
    setStatus((prev) => {
      const fromItems = prev[from].filter((f) => f.path !== filePath);
      const moved = prev[from].find((f) => f.path === filePath);
      if (!moved) return prev;

      const toItems = [...prev[to], moved];
      return { ...prev, [from]: fromItems, [to]: toItems };
    });
  };

  const handleStage = async (filePath: string) => {
    const id = workspaceId();
    if (!id) return;

    // 判断文件当前在哪个组
    const current = status();
    let fromGroup: GroupKey | null = null;
    if (current.unstaged.some((f) => f.path === filePath))
      fromGroup = "unstaged";
    else if (current.untracked.some((f) => f.path === filePath))
      fromGroup = "untracked";
    if (!fromGroup) return;

    const t0 = performance.now();
    moveFileOptimistically(filePath, fromGroup, "staged");
    const optimisticMs = performance.now() - t0;
    console.log(`[mvp-09] optimistic stage: ${optimisticMs.toFixed(2)}ms`);

    try {
      const result = await stageFiles({
        workspaceId: id,
        filePaths: [filePath],
      });
      if (result.failed.length > 0) {
        throw new Error(result.failed[0]?.error ?? "stage failed");
      }
    } catch (err) {
      // revert
      moveFileOptimistically(filePath, "staged", fromGroup);
      const msg = err instanceof Error ? err.message : String(err);
      setError(`无法 stage：${msg}`);
    }
  };

  const handleUnstage = async (filePath: string) => {
    const id = workspaceId();
    if (!id) return;

    const t0 = performance.now();
    moveFileOptimistically(filePath, "staged", "unstaged");
    const optimisticMs = performance.now() - t0;
    console.log(`[mvp-09] optimistic unstage: ${optimisticMs.toFixed(2)}ms`);

    try {
      await unstageFiles({
        workspaceId: id,
        filePaths: [filePath],
      });
    } catch (err) {
      moveFileOptimistically(filePath, "unstaged", "staged");
      const msg = err instanceof Error ? err.message : String(err);
      setError(`无法 unstage：${msg}`);
    }
  };

  const handleStageAll = async (group: GroupKey) => {
    const id = workspaceId();
    if (!id) return;

    const files = status()[group].map((f) => f.path);
    if (files.length === 0) return;

    // optimistic：全部移到 staged
    setStatus((prev) => ({
      ...prev,
      [group]: [],
      staged: [...prev.staged, ...prev[group]],
    }));

    try {
      const result = await stageFiles({
        workspaceId: id,
        filePaths: files,
      });
      if (result.failed.length > 0) {
        setError(`${result.failed.length} 个文件 stage 失败`);
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(`Stage All 失败：${msg}`);
    }
  };

  const handleUnstageAll = async () => {
    const id = workspaceId();
    if (!id) return;

    const files = status().staged.map((f) => f.path);
    if (files.length === 0) return;

    setStatus((prev) => ({
      ...prev,
      staged: [],
      unstaged: [...prev.unstaged, ...prev.staged],
    }));

    try {
      await unstageFiles({
        workspaceId: id,
        filePaths: files,
      });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      setError(`Unstage All 失败：${msg}`);
    }
  };

  const openDiff = (source: string, filePath: string) => {
    if (!props.onOpenDiff || !workspaceId()) return;
    props.onOpenDiff({
      workspaceId: workspaceId(),
      source,
      filePath,
    });
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
                      <Show when={groupItems(group.key).length > 0}>
                        <Show
                          when={group.key !== "staged"}
                          fallback={
                            <button
                              type="button"
                              class="vs-git-status-group-action"
                              onClick={(e) => {
                                e.stopPropagation();
                                void handleUnstageAll();
                              }}
                            >
                              Unstage All
                            </button>
                          }
                        >
                          <button
                            type="button"
                            class="vs-git-status-group-action"
                            onClick={(e) => {
                              e.stopPropagation();
                              void handleStageAll(group.key);
                            }}
                          >
                            Stage All
                          </button>
                        </Show>
                      </Show>
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
                              <button
                                type="button"
                                class="vs-git-status-item"
                                role="listitem"
                                onClick={() =>
                                  openDiff(
                                    group.key === "staged"
                                      ? "staged"
                                      : "unstaged",
                                    file.path,
                                  )
                                }
                              >
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
                                <Show
                                  when={group.key !== "staged"}
                                  fallback={
                                    <button
                                      type="button"
                                      class="vs-git-status-action vs-git-status-action-unstage"
                                      title="Unstage"
                                      onClick={(e) => {
                                        e.stopPropagation();
                                        void handleUnstage(file.path);
                                      }}
                                    >
                                      ✗
                                    </button>
                                  }
                                >
                                  <button
                                    type="button"
                                    class="vs-git-status-action vs-git-status-action-stage"
                                    title="Stage"
                                    onClick={(e) => {
                                      e.stopPropagation();
                                      void handleStage(file.path);
                                    }}
                                  >
                                    ✓
                                  </button>
                                </Show>
                              </button>
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

          <CommitBar
            workspaceId={workspaceId}
            stagedCount={() => status().staged.length}
            onCommitSuccess={() => {
              // commit 成功后触发一次 status 刷新
              const id = workspaceId();
              if (id) void loadWorkspace(id, "refresh");
            }}
            onError={(msg) => setError(msg)}
          />
        </div>
      </Match>
    </Switch>
  );
};
