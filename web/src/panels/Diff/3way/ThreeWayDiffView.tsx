import {
  createEffect,
  createMemo,
  createSignal,
  For,
  on,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type {
  ConflictedFile,
  ConflictHunk,
  ConflictHunkResolution,
  ConflictResolution,
  ConflictResolveFileRequest,
} from "../../../bindings";
import { DiffLineContent } from "../DiffLine";

interface ThreeWayDiffViewProps {
  workspaceId: string;
  initialFilePath?: string | null;
  onResolvedFile?: (filePath: string) => void;
  onError?: (message: string) => void;
}

type ResolutionMap = Record<string, ConflictResolution>;

export const ThreeWayDiffView: Component<ThreeWayDiffViewProps> = (props) => {
  const [files, setFiles] = createSignal<ConflictedFile[]>([]);
  const [activePath, setActivePath] = createSignal<string | null>(
    props.initialFilePath ?? null,
  );
  const [resolutions, setResolutions] = createSignal<ResolutionMap>({});
  const [loading, setLoading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const activeFile = createMemo(() => {
    const list = files();
    return (
      list.find((file) => file.path === activePath()) ??
      list.find((file) => !file.resolved) ??
      list[0] ??
      null
    );
  });

  const loadConflicts = async (workspaceId: string) => {
    if (!workspaceId) {
      setFiles([]);
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const nextFiles = await invoke<ConflictedFile[]>("conflict_status", {
        workspaceId,
      });
      setFiles(nextFiles);
      const preferred = props.initialFilePath;
      setActivePath(
        preferred && nextFiles.some((file) => file.path === preferred)
          ? preferred
          : (nextFiles.find((file) => !file.resolved)?.path ??
              nextFiles[0]?.path ??
              null),
      );
      setResolutions({});
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      props.onError?.(message);
    } finally {
      setLoading(false);
    }
  };

  createEffect(
    on(
      () => props.workspaceId,
      (workspaceId) => void loadConflicts(workspaceId),
    ),
  );

  createEffect(
    on(
      () => props.initialFilePath,
      (filePath) => {
        if (filePath) {
          setActivePath(filePath);
        }
      },
    ),
  );

  const hunkResolution = (hunk: ConflictHunk): ConflictResolution | null =>
    resolutions()[hunk.id] ?? hunk.resolution;

  const isHunkResolved = (hunk: ConflictHunk): boolean =>
    hunk.resolved || hunkResolution(hunk) !== null;

  const allHunksResolved = () => {
    const file = activeFile();
    return Boolean(
      file && file.hunks.length > 0 && file.hunks.every(isHunkResolved),
    );
  };

  const setHunkResolution = (
    hunk: ConflictHunk,
    resolution: ConflictResolution,
  ) => {
    setResolutions((current) => ({ ...current, [hunk.id]: resolution }));
  };

  const resetHunk = (hunk: ConflictHunk) => {
    setResolutions((current) => {
      const next = { ...current };
      delete next[hunk.id];
      return next;
    });
  };

  const markResolved = async () => {
    const file = activeFile();
    if (!file || !allHunksResolved()) {
      return;
    }
    setSubmitting(true);
    setError(null);
    const hunkResolutions: ConflictHunkResolution[] = file.hunks.map((hunk) => {
      const resolution = hunkResolution(hunk);
      if (!resolution) {
        return { hunkId: hunk.id, resolution: { kind: "acceptOurs" } };
      }
      return { hunkId: hunk.id, resolution };
    });
    const req: ConflictResolveFileRequest = {
      workspaceId: props.workspaceId,
      filePath: file.path,
      resolutions: hunkResolutions,
    };

    try {
      await invoke("conflict_resolve_file", { req });
      setFiles((current) =>
        current.map((item) =>
          item.path === file.path ? { ...item, resolved: true } : item,
        ),
      );
      props.onResolvedFile?.(file.path);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      props.onError?.(message);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section class="vs-three-way-diff" aria-label="3-way conflict diff">
      <aside class="vs-three-way-files" aria-label="Conflict files">
        <div class="vs-three-way-files-title">Conflicts</div>
        <Show
          when={!loading()}
          fallback={<div class="vs-three-way-state">Loading conflicts...</div>}
        >
          <For
            each={files()}
            fallback={<div class="vs-three-way-state">No conflicts</div>}
          >
            {(file) => (
              <button
                type="button"
                class="vs-three-way-file"
                classList={{
                  "is-active": activeFile()?.path === file.path,
                  "is-resolved": file.resolved,
                }}
                onClick={() => setActivePath(file.path)}
              >
                <span>{file.resolved ? "✓" : "!"}</span>
                <span title={file.path}>{file.path}</span>
              </button>
            )}
          </For>
        </Show>
      </aside>

      <div class="vs-three-way-main">
        <Show when={error()}>
          {(message) => (
            <div class="vs-three-way-error" role="alert">
              {message()}
            </div>
          )}
        </Show>

        <Show
          when={activeFile()}
          fallback={
            <div class="vs-three-way-empty">Select a conflict file</div>
          }
        >
          {(file) => (
            <>
              <header class="vs-three-way-head">
                <div>
                  <div class="vs-three-way-kicker">Conflict resolver</div>
                  <h3>{file().path}</h3>
                </div>
                <button
                  type="button"
                  class="vs-mvp16-btn-primary"
                  disabled={!allHunksResolved() || submitting()}
                  onClick={() => void markResolved()}
                >
                  {submitting() ? "Marking..." : "Mark as resolved"}
                </button>
              </header>

              <div class="vs-three-way-column-heads">
                <span>Ours</span>
                <span>Base</span>
                <span>Theirs</span>
              </div>

              <div class="vs-three-way-hunks">
                <For each={file().hunks}>
                  {(hunk) => (
                    <ConflictHunkView
                      hunk={hunk}
                      filePath={file().path}
                      resolution={hunkResolution(hunk)}
                      resolved={isHunkResolved(hunk)}
                      onResolve={(resolution) =>
                        setHunkResolution(hunk, resolution)
                      }
                      onReset={() => resetHunk(hunk)}
                    />
                  )}
                </For>
              </div>
            </>
          )}
        </Show>
      </div>
    </section>
  );
};

const ConflictHunkView: Component<{
  hunk: ConflictHunk;
  filePath: string;
  resolution: ConflictResolution | null;
  resolved: boolean;
  onResolve: (resolution: ConflictResolution) => void;
  onReset: () => void;
}> = (props) => {
  const manualContent = () =>
    props.resolution?.kind === "manual"
      ? props.resolution.content
      : props.hunk.oursContent;

  return (
    <article
      class="vs-three-way-hunk"
      classList={{ "is-resolved": props.resolved }}
    >
      <header class="vs-three-way-hunk-head">
        <span>Hunk {props.hunk.id}</span>
        <Show
          when={props.resolved}
          fallback={
            <div class="vs-three-way-actions">
              <button
                type="button"
                class="is-ours"
                onClick={() => props.onResolve({ kind: "acceptOurs" })}
              >
                Accept Ours
              </button>
              <button
                type="button"
                class="is-theirs"
                onClick={() => props.onResolve({ kind: "acceptTheirs" })}
              >
                Accept Theirs
              </button>
              <button
                type="button"
                class="is-both"
                onClick={() => props.onResolve({ kind: "acceptBoth" })}
              >
                Accept Both
              </button>
              <button
                type="button"
                onClick={() =>
                  props.onResolve({
                    kind: "manual",
                    content: manualContent(),
                  })
                }
              >
                Manual edit
              </button>
            </div>
          }
        >
          <div class="vs-three-way-resolved">
            <span>✓ resolved</span>
            <button type="button" onClick={props.onReset}>
              Reset
            </button>
          </div>
        </Show>
      </header>

      <Show when={props.resolution?.kind === "manual"}>
        <textarea
          class="vs-three-way-manual"
          value={manualContent()}
          onInput={(event) =>
            props.onResolve({
              kind: "manual",
              content: event.currentTarget.value,
            })
          }
        />
      </Show>

      <div class="vs-three-way-grid">
        <DiffColumn
          content={props.hunk.oursContent}
          filePath={props.filePath}
          tone="ours"
        />
        <DiffColumn
          content={props.hunk.baseContent}
          filePath={props.filePath}
          tone="base"
        />
        <DiffColumn
          content={props.hunk.theirsContent}
          filePath={props.filePath}
          tone="theirs"
        />
      </div>
    </article>
  );
};

const DiffColumn: Component<{
  content: string;
  filePath: string;
  tone: "ours" | "base" | "theirs";
}> = (props) => {
  const lines = () => splitLines(props.content);
  return (
    <div class={`vs-three-way-column is-${props.tone}`}>
      <For each={lines()}>
        {(line, index) => (
          <div class="vs-three-way-line">
            <span class="vs-three-way-line-num">{index() + 1}</span>
            <DiffLineContent
              content={line}
              filePath={props.filePath}
              lineType="context"
            />
          </div>
        )}
      </For>
    </div>
  );
};

function splitLines(content: string): string[] {
  const lines = content.split(/\r?\n/);
  if (lines.length > 1 && lines[lines.length - 1] === "") {
    return lines.slice(0, -1);
  }
  return lines;
}
