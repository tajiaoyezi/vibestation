import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SessionBindCommitRequest,
  SessionBindCommitResult,
  SessionCommitBoundEvent,
  SessionCommitUnboundEvent,
  SessionDetailRequest,
  SessionDetailResult,
  SessionEndRequest,
  SessionEndResult,
  SessionEndedEvent,
  SessionErrorEvent,
  SessionLinkUpdatedEvent,
  SessionListRequest,
  SessionListResult,
  SessionRebindRequest,
  SessionRebindResult,
  SessionRecalcRequest,
  SessionRecalcResult,
  SessionStartRequest,
  SessionStartResult,
  SessionStartedEvent,
  SessionUnbindRequest,
  SessionUnbindResult,
} from "../../bindings";

// ── Invoke wrappers (8 commands · HC-3 contract) ──

export async function sessionStart(
  req: SessionStartRequest,
): Promise<SessionStartResult> {
  return invoke<SessionStartResult>("session_start", { req });
}

export async function sessionEnd(
  req: SessionEndRequest,
): Promise<SessionEndResult> {
  return invoke<SessionEndResult>("session_end", { req });
}

export async function sessionBindCommit(
  req: SessionBindCommitRequest,
): Promise<SessionBindCommitResult> {
  return invoke<SessionBindCommitResult>("session_bind_commit", { req });
}

export async function sessionUnbind(
  req: SessionUnbindRequest,
): Promise<SessionUnbindResult> {
  return invoke<SessionUnbindResult>("session_unbind", { req });
}

export async function sessionList(
  req: SessionListRequest,
): Promise<SessionListResult> {
  return invoke<SessionListResult>("session_list", { req });
}

export async function sessionGetDetail(
  req: SessionDetailRequest,
): Promise<SessionDetailResult> {
  return invoke<SessionDetailResult>("session_get_detail", { req });
}

export async function sessionRebind(
  req: SessionRebindRequest,
): Promise<SessionRebindResult> {
  return invoke<SessionRebindResult>("session_rebind", { req });
}

export async function sessionRecalc(
  req: SessionRecalcRequest,
): Promise<SessionRecalcResult> {
  return invoke<SessionRecalcResult>("session_recalc", { req });
}

// ── Event listener helpers (6 events · HC-3 contract) ──

export function onSessionStarted(
  cb: (e: SessionStartedEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionStartedEvent>("session:started", (event) =>
    cb(event.payload),
  );
}

export function onSessionEnded(
  cb: (e: SessionEndedEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionEndedEvent>("session:ended", (event) =>
    cb(event.payload),
  );
}

export function onSessionCommitBound(
  cb: (e: SessionCommitBoundEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionCommitBoundEvent>("session:commit-bound", (event) =>
    cb(event.payload),
  );
}

export function onSessionCommitUnbound(
  cb: (e: SessionCommitUnboundEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionCommitUnboundEvent>("session:commit-unbound", (event) =>
    cb(event.payload),
  );
}

export function onSessionLinkUpdated(
  cb: (e: SessionLinkUpdatedEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionLinkUpdatedEvent>("session:link-updated", (event) =>
    cb(event.payload),
  );
}

export function onSessionError(
  cb: (e: SessionErrorEvent) => void,
): Promise<UnlistenFn> {
  return listen<SessionErrorEvent>("session:error", (event) =>
    cb(event.payload),
  );
}
