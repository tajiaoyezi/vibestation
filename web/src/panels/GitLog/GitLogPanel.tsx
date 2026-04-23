import {
  type Component,
  createSignal,
  createEffect,
  onMount,
  onCleanup,
  For,
  Show,
} from "solid-js";
import type {
  GitLogEntry,
  GitLogQueryRequest,
  GitLogQueryResponse,
  CommitDetail,
  WorkspaceMetadata,
} from "../../bindings";
import { queryLog, fetchDetail, clearCache } from "./gitLogApi";
import type { DiffTarget } from "../../components/MainContent";

export interface GitLogPanelProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  onOpenDiff?: (target: DiffTarget) => void;
}

export function createGitLogStore() {
  const [entries, setEntries] = createSignal<GitLogEntry[]>([]);
  const [hasMore, setHasMore] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [selectedSha, setSelectedSha] = createSignal<string | null>(null);
  const [filterMessage, setFilterMessage] = createSignal("");
  const [filterAuthor, setFilterAuthor] = createSignal("");
  const [offset, setOffset] = createSignal(0);
  const [detail, setDetail] = createSignal<CommitDetail | null>(null);
  const [detailLoading, setDetailLoading] = createSignal(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  const load = async (workspaceId: string, resetOffset = true) => {
    setLoading(true);
    setError(null);
    const newOffset = resetOffset ? 0 : offset();
    if (resetOffset) {
      setOffset(0);
      setEntries([]);
    }

    const req: GitLogQueryRequest = {
      workspaceId,
      offset: newOffset,
      limit: 100,
      filterMessage: filterMessage() || null,
      filterAuthor: filterAuthor() || null,
      filterAfter: null,
    };

    try {
      const resp: GitLogQueryResponse = await queryLog(req);
      if (resetOffset) {
        setEntries(resp.entries);
      } else {
        setEntries((prev) => [...prev, ...resp.entries]);
      }
      setHasMore(resp.hasMore);
      setOffset(newOffset + resp.entries.length);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const loadMore = (workspaceId: string) => {
    if (hasMore() && !loading()) {
      load(workspaceId, false);
    }
  };

  const loadDetail = async (workspaceId: string, sha: string) => {
    setDetailLoading(true);
    try {
      const result = await fetchDetail(workspaceId, sha);
      setDetail(result);
      setSelectedSha(sha);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setDetailLoading(false);
    }
  };

  const debouncedLoad = (workspaceId: string) => {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => load(workspaceId, true), 300);
  };

  const clearFilter = () => {
    setFilterMessage("");
    setFilterAuthor("");
  };

  return {
    entries,
    hasMore,
    loading,
    error,
    selectedSha,
    detail,
    detailLoading,
    filterMessage,
    filterAuthor,
    setFilterMessage: (v: string) => {
      setFilterMessage(v);
    },
    setFilterAuthor: (v: string) => {
      setFilterAuthor(v);
    },
    load,
    loadMore,
    loadDetail,
    debouncedLoad,
    clearCache,
    clearFilter,
    setError,
  };
}

export type GitLogStore = ReturnType<typeof createGitLogStore>;

export const GitLogPanel: Component<GitLogPanelProps> = (props) => {
  const store = createGitLogStore();
  let scrollContainer: HTMLDivElement | undefined;

  const workspaceId = () => {
    const ws = props.activeWorkspace();
    return ws ? ws.workspaceId : "";
  };

  const hasGit = () => {
    const ws = props.activeWorkspace();
    return ws ? ws.hasGit : false;
  };

  onMount(() => {
    if (hasGit() && workspaceId()) {
      store.load(workspaceId());
    }
  });

  createEffect(() => {
    const wid = workspaceId();
    if (wid && hasGit()) {
      store.load(wid);
    }
  });

  onCleanup(() => {
    store.clearCache();
  });

  const handleScroll = () => {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    if (scrollHeight - scrollTop - clientHeight < 50 && store.hasMore()) {
      store.loadMore(workspaceId());
    }
  };

  const handleFilterKeydown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && hasGit() && workspaceId()) {
      store.load(workspaceId());
    }
  };

  const formatTime = (timestamp: number): string => {
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    if (diff < 60) return "just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  return (
    <Show
      when={hasGit()}
      fallback={
        <div class="vs-git-log-empty">
          <p class="vs-placeholder-text">No git repository found</p>
          <p class="vs-placeholder-text">
            Open a directory containing a .git folder
          </p>
        </div>
      }
    >
      <div class="vs-git-log">
        <div class="vs-git-log-search">
          <input
            type="text"
            class="vs-git-log-search-input"
            placeholder="Search messages..."
            value={store.filterMessage()}
            onInput={(e) => {
              store.setFilterMessage(e.currentTarget.value);
              store.debouncedLoad(workspaceId());
            }}
            onKeyPress={handleFilterKeydown}
          />
          <input
            type="text"
            class="vs-git-log-search-input"
            placeholder="author:name"
            value={store.filterAuthor()}
            onInput={(e) => {
              store.setFilterAuthor(e.currentTarget.value);
              store.debouncedLoad(workspaceId());
            }}
            onKeyPress={handleFilterKeydown}
          />
        </div>

        <Show when={store.error()}>
          <div class="vs-git-log-error">{store.error()}</div>
        </Show>

        <div
          class="vs-git-log-list"
          ref={scrollContainer}
          onScroll={handleScroll}
        >
          <Show
            when={!store.loading() || store.entries().length > 0}
            fallback={<div class="vs-git-log-loading">Loading...</div>}
          >
            <For each={store.entries()}>
              {(entry) => (
                <button
                  class={`vs-git-log-entry ${store.selectedSha() === entry.shortSha ? "vs-git-log-entry-selected" : ""}`}
                  onClick={() =>
                    store.loadDetail(workspaceId(), entry.shortSha)
                  }
                >
                  <div class="vs-git-log-entry-header">
                    <span class="vs-git-log-sha">{entry.shortSha}</span>
                    <span class="vs-git-log-time">
                      {formatTime(entry.authoredDate)}
                    </span>
                  </div>
                  <div class="vs-git-log-message">{entry.message}</div>
                  <div class="vs-git-log-author">{entry.authorName}</div>
                  <div class="vs-git-log-labels">
                    <For each={entry.branchLabels}>
                      {(label) => (
                        <span class="vs-git-log-label vs-git-log-branch">
                          {label}
                        </span>
                      )}
                    </For>
                    <For each={entry.tagLabels}>
                      {(label) => (
                        <span class="vs-git-log-label vs-git-log-tag">
                          {label}
                        </span>
                      )}
                    </For>
                  </div>
                </button>
              )}
            </For>
          </Show>

          <Show when={store.hasMore()}>
            <button
              class="vs-git-log-load-more"
              onClick={() => store.loadMore(workspaceId())}
              disabled={store.loading()}
            >
              {store.loading() ? "Loading..." : "Load more"}
            </button>
          </Show>
        </div>

        <Show when={store.detail()}>
          <div class="vs-git-log-detail">
            <h4 class="vs-git-log-detail-title">
              Commit: {store.detail()?.shortSha}
            </h4>
            <Show when={store.detailLoading()}>
              <div class="vs-git-log-loading">Loading detail...</div>
            </Show>
            <Show when={store.detail() && !store.detailLoading()}>
              <div class="vs-git-log-detail-content">
                <div class="vs-git-log-detail-meta">
                  <div>
                    <strong>Author:</strong> {store.detail()!.author.name} &lt;
                    {store.detail()!.author.email}&gt;
                  </div>
                  <div>
                    <strong>Date:</strong>{" "}
                    {new Date(
                      store.detail()!.author.timestamp * 1000,
                    ).toLocaleString()}
                  </div>
                  <div>
                    <strong>Committer:</strong> {store.detail()!.committer.name}
                  </div>
                  <div>
                    <strong>SHA:</strong> {store.detail()!.fullSha}
                  </div>
                  <Show when={store.detail()!.parents.length > 0}>
                    <div>
                      <strong>Parents:</strong>{" "}
                      <For each={store.detail()!.parents}>
                        {(p) => (
                          <span class="vs-git-log-parent">{p.shortSha}</span>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
                <div class="vs-git-log-detail-message">
                  {store.detail()!.message}
                </div>
                <div class="vs-git-log-detail-files">
                  <Show
                    when={store.detail()!.files.length <= 1000}
                    fallback={
                      <div>
                        <strong>Files ({store.detail()!.files.length}):</strong>{" "}
                        Showing first 1000
                      </div>
                    }
                  >
                    <div>
                      <strong>Files ({store.detail()!.files.length}):</strong>
                    </div>
                  </Show>
                  <For each={store.detail()!.files.slice(0, 200)}>
                    {(file) => (
                      <button
                        type="button"
                        class="vs-git-log-file"
                        onClick={() => {
                          if (!props.onOpenDiff) return;
                          props.onOpenDiff({
                            workspaceId: workspaceId(),
                            source: store.detail()!.fullSha,
                            filePath: file.path,
                          });
                        }}
                      >
                        <span
                          class={`vs-git-log-file-status vs-git-log-status-${file.status}`}
                        >
                          {file.status}
                        </span>
                        <span class="vs-git-log-file-path">{file.path}</span>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </Show>
      </div>
    </Show>
  );
};
