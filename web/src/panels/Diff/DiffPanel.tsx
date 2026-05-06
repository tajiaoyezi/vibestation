import {
  createEffect,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import type { DiffHunk, DiffResponse } from "../../bindings";
import { computeDiff, getDiffViewMode, setDiffViewMode } from "./diffApi";
import { DiffLineContent } from "./DiffLine";
import { PlainTextChip } from "./PlainTextChip";

interface DiffPanelProps {
  workspaceId: string;
  source: string;
  filePath: string;
}

type ViewMode = "split" | "unified";

export const DiffPanel: Component<DiffPanelProps> = (props) => {
  const [viewMode, setViewMode] = createSignal<ViewMode>("split");
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [response, setResponse] = createSignal<DiffResponse | null>(null);
  const [allowLargeFile, setAllowLargeFile] = createSignal(false);

  const loadDiff = async (
    workspaceId: string,
    source: string,
    filePath: string,
    allowLarge: boolean,
  ) => {
    if (!workspaceId || !filePath) {
      setResponse(null);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const [diffResponse, mode] = await Promise.all([
        computeDiff({
          workspaceId,
          source,
          filePath,
          allowLargeFile: allowLarge,
        }),
        getDiffViewMode(workspaceId),
      ]);
      setResponse(diffResponse);
      setViewMode(mode);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    const w = props.workspaceId;
    const s = props.source;
    const f = props.filePath;
    setAllowLargeFile(false);
    void loadDiff(w, s, f, false);
  });

  const handleViewModeChange = (mode: ViewMode) => {
    setViewMode(mode);
    void setDiffViewMode(props.workspaceId, mode).catch((err) => {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    });
  };

  const handleShowAnyway = () => {
    setAllowLargeFile(true);
    void loadDiff(props.workspaceId, props.source, props.filePath, true);
  };

  const lineClass = (lineType: string): string => {
    switch (lineType) {
      case "added":
        return "vs-diff-line-added";
      case "removed":
        return "vs-diff-line-removed";
      default:
        return "vs-diff-line-context";
    }
  };

  const formatBytes = (bytes: number | null): string => {
    if (bytes === null) return "—";
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div class="vs-diff-panel">
      <div class="vs-diff-toolbar">
        <span class="vs-diff-filepath" title={props.filePath}>
          {props.filePath}
        </span>
        <div class="vs-diff-toolbar-right">
          <PlainTextChip filePath={props.filePath} />
          <button
            type="button"
            class={`vs-diff-mode-btn ${viewMode() === "split" ? "vs-diff-mode-btn-on" : ""}`}
            onClick={() => handleViewModeChange("split")}
            aria-pressed={viewMode() === "split"}
          >
            Split
          </button>
          <button
            type="button"
            class={`vs-diff-mode-btn ${viewMode() === "unified" ? "vs-diff-mode-btn-on" : ""}`}
            onClick={() => handleViewModeChange("unified")}
            aria-pressed={viewMode() === "unified"}
          >
            Unified
          </button>
        </div>
      </div>

      <Show when={loading() && !response()}>
        <div class="vs-diff-state vs-diff-loading">Loading diff...</div>
      </Show>

      <Show when={error()}>
        <div class="vs-diff-state vs-diff-error" role="alert">
          {error()}
        </div>
      </Show>

      <Show when={!loading() && !error() && response()}>
        {(resp) => (
          <div class="vs-diff-content">
            <Show when={resp().binary}>
              <div class="vs-diff-state vs-diff-binary">
                <div class="vs-diff-binary-title">Binary file</div>
                <Show
                  when={
                    resp().oldSizeBytes !== null || resp().newSizeBytes !== null
                  }
                >
                  <div class="vs-diff-binary-sizes">
                    <span>Old: {formatBytes(resp().oldSizeBytes)}</span>
                    <span>New: {formatBytes(resp().newSizeBytes)}</span>
                  </div>
                </Show>
              </div>
            </Show>

            <Show when={resp().truncated && !allowLargeFile()}>
              <div class="vs-diff-state vs-diff-truncated">
                <div class="vs-diff-truncated-title">Large file</div>
                <Show when={resp().truncatedReason}>
                  <div class="vs-diff-truncated-reason">
                    {resp().truncatedReason}
                  </div>
                </Show>
                <Show when={resp().lineCount !== null}>
                  <div class="vs-diff-truncated-count">
                    {resp().lineCount} lines
                  </div>
                </Show>
                <button
                  type="button"
                  class="vs-diff-show-anyway"
                  onClick={handleShowAnyway}
                >
                  Show anyway
                </button>
              </div>
            </Show>

            <Show
              when={!resp().binary && (!resp().truncated || allowLargeFile())}
            >
              <Show
                when={resp().hunks.length > 0}
                fallback={
                  <div class="vs-diff-state vs-diff-empty">No changes</div>
                }
              >
                <div class="vs-diff-hunks">
                  <For each={resp().hunks}>
                    {(hunk) => (
                      <DiffHunkView
                        hunk={hunk}
                        viewMode={viewMode()}
                        lineClass={lineClass}
                        filePath={props.filePath}
                      />
                    )}
                  </For>
                </div>
              </Show>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
};

interface DiffHunkViewProps {
  hunk: DiffHunk;
  viewMode: ViewMode;
  lineClass: (lineType: string) => string;
  filePath: string;
}

const DiffHunkView: Component<DiffHunkViewProps> = (props) => {
  const oldCount = () =>
    props.hunk.lines.filter((l) => l.lineType !== "added").length;
  const newCount = () =>
    props.hunk.lines.filter((l) => l.lineType !== "removed").length;

  return (
    <div class="vs-diff-hunk">
      <div class="vs-diff-hunk-header">
        @@ -{props.hunk.oldStart},{oldCount()} +{props.hunk.newStart},
        {newCount()} @@
      </div>
      <Show
        when={props.viewMode === "unified"}
        fallback={
          <SplitHunkLines
            hunk={props.hunk}
            lineClass={props.lineClass}
            filePath={props.filePath}
          />
        }
      >
        <UnifiedHunkLines
          hunk={props.hunk}
          lineClass={props.lineClass}
          filePath={props.filePath}
        />
      </Show>
    </div>
  );
};

const UnifiedHunkLines: Component<{
  hunk: DiffHunk;
  lineClass: (lineType: string) => string;
  filePath: string;
}> = (props) => {
  return (
    <div class="vs-diff-unified">
      <For each={props.hunk.lines}>
        {(line) => (
          <div class={`vs-diff-line ${props.lineClass(line.lineType)}`}>
            <span class="vs-diff-line-num vs-diff-line-num-old">
              {line.oldLineNum ?? ""}
            </span>
            <span class="vs-diff-line-num vs-diff-line-num-new">
              {line.newLineNum ?? ""}
            </span>
            <span class="vs-diff-line-marker">
              {line.lineType === "added"
                ? "+"
                : line.lineType === "removed"
                  ? "-"
                  : " "}
            </span>
            <DiffLineContent
              content={line.content}
              filePath={props.filePath}
              lineType={line.lineType}
            />
          </div>
        )}
      </For>
    </div>
  );
};

const SplitHunkLines: Component<{
  hunk: DiffHunk;
  lineClass: (lineType: string) => string;
  filePath: string;
}> = (props) => {
  return (
    <div class="vs-diff-split">
      <For each={props.hunk.lines}>
        {(line) => (
          <div class="vs-diff-split-row">
            <div
              class={`vs-diff-split-cell ${line.lineType === "added" ? "vs-diff-split-empty" : props.lineClass(line.lineType)}`}
            >
              <Show when={line.lineType !== "added"}>
                <span class="vs-diff-line-num">{line.oldLineNum ?? ""}</span>
                <span class="vs-diff-line-marker">
                  {line.lineType === "removed" ? "-" : " "}
                </span>
                <DiffLineContent
                  content={line.content}
                  filePath={props.filePath}
                  lineType={line.lineType}
                />
              </Show>
            </div>
            <div
              class={`vs-diff-split-cell ${line.lineType === "removed" ? "vs-diff-split-empty" : props.lineClass(line.lineType)}`}
            >
              <Show when={line.lineType !== "removed"}>
                <span class="vs-diff-line-num">{line.newLineNum ?? ""}</span>
                <span class="vs-diff-line-marker">
                  {line.lineType === "added" ? "+" : " "}
                </span>
                <DiffLineContent
                  content={line.content}
                  filePath={props.filePath}
                  lineType={line.lineType}
                />
              </Show>
            </div>
          </div>
        )}
      </For>
    </div>
  );
};
