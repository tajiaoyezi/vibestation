import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  RollbackPreview,
  RollbackAbortResult,
  RollbackStatus,
  RollbackProgress,
} from "./rollbackContract";
import type { RollbackError } from "../../bindings/RollbackError";

export async function rollbackPreview(
  sessionId: string,
): Promise<RollbackPreview> {
  return invoke<RollbackPreview>("rollback_preview", {
    req: { session_id: sessionId },
  });
}

export async function rollbackExecute(
  sessionId: string,
  includeShas: string[],
): Promise<RollbackProgress> {
  return invoke<RollbackProgress>("rollback_execute", {
    req: { session_id: sessionId, include_shas: includeShas },
  });
}

export async function rollbackAbort(
  sessionId: string,
): Promise<RollbackAbortResult> {
  return invoke<RollbackAbortResult>("rollback_abort", {
    req: { session_id: sessionId },
  });
}

export async function rollbackStatus(
  sessionId: string,
): Promise<RollbackStatus> {
  return invoke<RollbackStatus>("rollback_status", {
    req: { session_id: sessionId },
  });
}

export function onRollbackProgress(
  cb: (p: RollbackProgress) => void,
): Promise<UnlistenFn> {
  return listen<RollbackProgress>("git:rollback-progress", (e) =>
    cb(e.payload),
  );
}

export function onRollbackDone(
  cb: (p: { session_id: string; revert_shas: string[] }) => void,
): Promise<UnlistenFn> {
  return listen<{ session_id: string; revert_shas: string[] }>(
    "git:rollback-done",
    (e) => cb(e.payload),
  );
}

export function onRollbackAborted(
  cb: (p: { session_id: string; head_sha: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ session_id: string; head_sha: string }>(
    "git:rollback-aborted",
    (e) => cb(e.payload),
  );
}

export function onRollbackConflict(
  cb: (p: { path: string; commit_sha: string }) => void,
): Promise<UnlistenFn> {
  return listen<{ path: string; commit_sha: string }>(
    "git:rollback-conflict",
    (e) => cb(e.payload),
  );
}

export type { RollbackError };
