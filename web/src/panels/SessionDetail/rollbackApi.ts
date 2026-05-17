import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  RollbackPreview,
  RollbackAbortResult,
  RollbackStatus,
  RollbackProgress,
  RollbackError,
} from "../../bindings";

export async function rollbackPreview(
  sessionId: string,
): Promise<RollbackPreview> {
  return invoke<RollbackPreview>("rollback_preview", { sessionId });
}

export async function rollbackExecute(
  sessionId: string,
  includeShas: string[],
): Promise<RollbackProgress> {
  return invoke<RollbackProgress>("rollback_execute", {
    sessionId,
    includeShas,
  });
}

export async function rollbackAbort(
  sessionId: string,
): Promise<RollbackAbortResult> {
  return invoke<RollbackAbortResult>("rollback_abort", { sessionId });
}

export async function rollbackStatus(
  sessionId: string,
): Promise<RollbackStatus> {
  return invoke<RollbackStatus>("rollback_status", { sessionId });
}

export function onRollbackProgress(
  cb: (p: RollbackProgress) => void,
): Promise<UnlistenFn> {
  return listen<RollbackProgress>("git:rollback-progress", (e) =>
    cb(e.payload),
  );
}

export function onRollbackDone(
  cb: (p: { sessionId: string; revertShas: string[] }) => void,
): Promise<UnlistenFn> {
  return listen<{ sessionId: string; revertShas: string[] }>(
    "git:rollback-done",
    (e) => cb(e.payload),
  );
}

export function onRollbackAborted(
  cb: (p: { sessionId: string; headSha: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ sessionId: string; headSha: string }>(
    "git:rollback-aborted",
    (e) => cb(e.payload),
  );
}

export function onRollbackConflict(
  cb: (p: { path: string; commitSha: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ path: string; commitSha: string }>(
    "git:rollback-conflict",
    (e) => cb(e.payload),
  );
}

export type { RollbackError };
