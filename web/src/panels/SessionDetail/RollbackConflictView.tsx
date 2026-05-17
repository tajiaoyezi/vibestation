import { createMemo, createSignal, onMount, Show, type JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { ConflictedFile } from "../../bindings";
import { ConflictBanner } from "../../components/ConflictBanner";
import { ThreeWayDiffView } from "../Diff/3way";

interface RollbackConflictViewProps {
  sessionId: string;
  workspaceId: string;
  includeShas: string[];
  progress: { done: number; total: number };
  initialConflictFile: string;
  onResume: () => Promise<void>;
  onAbort: () => Promise<void>;
  onCompleted: () => void;
}

export function RollbackConflictView(
  props: RollbackConflictViewProps,
): JSX.Element {
  const [resolvedFiles, setResolvedFiles] = createSignal<Set<string>>(
    new Set(),
  );
  const [conflictFiles, setConflictFiles] = createSignal<ConflictedFile[]>([]);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  onMount(async () => {
    try {
      const files = await invoke<ConflictedFile[]>("conflict_status", {
        workspaceId: props.workspaceId,
      });
      setConflictFiles(files);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  });

  const unresolved = createMemo(() =>
    conflictFiles().filter((f) => !f.resolved && !resolvedFiles().has(f.path)),
  );
  const allResolved = createMemo(() => unresolved().length === 0);
  const firstUnresolvedFile = createMemo(
    () => unresolved()[0]?.path ?? props.initialConflictFile,
  );

  const handleContinue = async () => {
    if (!allResolved()) return;
    setBusy(true);
    setError(null);
    try {
      await props.onResume();
      props.onCompleted();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const handleAbort = async () => {
    setBusy(true);
    setError(null);
    try {
      await props.onAbort();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  };

  const markResolved = (filePath: string) => {
    setResolvedFiles((prev) => {
      const next = new Set(prev);
      next.add(filePath);
      return next;
    });
  };

  return (
    <div class="vs-mvp20-rollback-conflict-overlay">
      <ConflictBanner
        variant="active"
        operation="rollback"
        source={props.sessionId}
        target={null}
        currentStep={props.progress.done}
        totalSteps={props.progress.total}
        filePath={firstUnresolvedFile()}
        allResolved={allResolved()}
        allowSkip={false}
        busy={busy()}
        onContinue={handleContinue}
        onAbort={handleAbort}
      />
      <Show when={error()}>
        {(message) => (
          <div class="vs-conflict-error" role="alert">
            {message()}
          </div>
        )}
      </Show>
      <ThreeWayDiffView
        workspaceId={props.workspaceId}
        initialFilePath={props.initialConflictFile}
        onResolvedFile={markResolved}
        onError={setError}
      />
    </div>
  );
}
