import {
  createMemo,
  createResource,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type JSX,
} from "solid-js";
import { useSessions } from "../../stores/sessions-context";
import { useLayout } from "../../stores/layout-context";
import type {
  AiSession,
  SessionCommitLink,
  SessionDetailResult,
} from "../../bindings";
import { isLinkActive, isLinkConfirmed } from "../../stores/sessions";
import { SessionUnbindModal } from "./SessionUnbindModal";
import { SessionRebindModal } from "./SessionRebindModal";
import { RollbackPreviewModal } from "../SessionDetail/RollbackPreviewModal";
import { RollbackConfirmDialog } from "../SessionDetail/RollbackConfirmDialog";
import { RollbackProgressBanner } from "../SessionDetail/RollbackProgressBanner";
import {
  onRollbackAborted,
  onRollbackConflict,
  onRollbackDone,
  rollbackPreview,
  rollbackExecute,
  rollbackAbort,
} from "../SessionDetail/rollbackApi";
import type { RollbackPreview, RollbackProgress } from "../../bindings";
import {
  parseRollbackError,
  formatRollbackError,
} from "../SessionDetail/rollbackError";
import { RollbackConflictView } from "../SessionDetail/RollbackConflictView";
import "./sessionDetail.css";
import "../SessionDetail/rollback.css";

function fmtTime(epoch: number): string {
  return new Date(epoch).toLocaleString();
}

function shortSha(sha: string): string {
  return sha.slice(0, 8);
}

function statusLabel(status: AiSession["status"]): string {
  switch (status) {
    case "active":
      return "Active";
    case "ended":
      return "Ended";
    case "idleCutoff":
      return "Idle cutoff";
    case "archived":
      return "Archived";
    default:
      return status;
  }
}

/**
 * Resolve workspaceId for a sessionId by scanning the store. Returns the first
 * workspace that has a session with the given id. Falls back to the first
 * workspace key if no match (best-effort when store hasn't been hydrated yet).
 */
function resolveWorkspaceId(
  sessionsByWorkspace: Record<string, AiSession[]>,
  sessionId: string,
): string {
  for (const [wsId, sessions] of Object.entries(sessionsByWorkspace)) {
    if (sessions.some((s) => s.id === sessionId)) return wsId;
  }
  const keys = Object.keys(sessionsByWorkspace);
  return keys[0] ?? "";
}

export function SessionDetailView(): JSX.Element {
  const ctx = useSessions();
  const { dispatch } = useLayout();
  const detail = ctx.activeDetail();
  const sessionId = detail?.sessionId ?? "";
  const commitAnchor = detail?.commitAnchor;

  const workspaceId = createMemo(() =>
    resolveWorkspaceId(ctx.store.sessionsByWorkspace, sessionId),
  );

  const [detailData] = createResource(
    () => {
      const wsId = workspaceId();
      if (!wsId) return null;
      return { workspaceId: wsId, sessionId };
    },
    (req) => (req ? ctx.getDetail(req) : Promise.reject("no workspace")),
  );

  const [unbindTarget, setUnbindTarget] =
    createSignal<SessionCommitLink | null>(null);
  const [rebindTarget, setRebindTarget] =
    createSignal<SessionCommitLink | null>(null);

  const [rollbackPreviewData, setRollbackPreviewData] =
    createSignal<RollbackPreview | null>(null);
  const [rollbackConfirmShas, setRollbackConfirmShas] = createSignal<
    string[] | null
  >(null);
  const [rollbackProgress, setRollbackProgress] =
    createSignal<RollbackProgress | null>(null);
  const [lastIncludeShas, setLastIncludeShas] = createSignal<string[]>([]);
  const [rollbackConflict, setRollbackConflict] = createSignal<{
    path: string;
    commitSha: string;
    includeShas: string[];
  } | null>(null);
  const [rollbackUiError, setRollbackUiError] = createSignal<string | null>(
    null,
  );
  const [rolledBack, setRolledBack] = createSignal(false);
  const [rollbackLoading, setRollbackLoading] = createSignal(false);

  const handleUnbind = async (
    link: SessionCommitLink,
    reason: string,
    recalc: boolean,
  ) => {
    const wsId = workspaceId();
    await ctx.unbind({
      workspaceId: wsId,
      linkId: link.id,
      reason: reason || null,
    });
    if (recalc) {
      await ctx.recalc({
        workspaceId: wsId,
        commitSha: link.commitSha,
      });
    }
    setUnbindTarget(null);
  };

  const handleRebind = async (
    link: SessionCommitLink,
    targetSessionId: string,
    reason: string,
  ) => {
    const wsId = workspaceId();
    await ctx.rebind({
      workspaceId: wsId,
      linkId: link.id,
      targetSessionId,
      reason: reason || null,
    });
    setRebindTarget(null);
  };

  const handleEndSession = async (session: AiSession) => {
    await ctx.end({
      workspaceId: workspaceId(),
      sessionId: session.id,
      endReason: "manual_end",
    });
  };

  const handleRecalcAll = async (links: SessionCommitLink[]) => {
    const wsId = workspaceId();
    const shas = [...new Set(links.map((l) => l.commitSha))];
    for (const sha of shas) {
      await ctx.recalc({ workspaceId: wsId, commitSha: sha });
    }
  };

  const handleRollbackError = (err: unknown, contextLabel: string) => {
    const parsed = parseRollbackError(err);
    if (parsed?.kind === "dirtyWorkingTree") {
      dispatch({ kind: "open-bottom" });
      setRollbackUiError(
        `请先提交或丢弃工作区的 ${parsed.modified.length + parsed.staged.length} 处改动 · 已自动打开底部 Git Status 面板`,
      );
      return parsed;
    }
    if (parsed) {
      setRollbackUiError(`${parsed.kind}: ${formatRollbackError(parsed)}`);
      return parsed;
    }
    setRollbackUiError(`${contextLabel} failed: ${String(err)}`);
    return null;
  };

  const handleRollbackClick = async () => {
    setRollbackUiError(null);
    setRollbackLoading(true);
    try {
      const preview = await rollbackPreview(sessionId);
      setRollbackPreviewData(preview);
    } catch (err) {
      handleRollbackError(err, "rollback preview");
    } finally {
      setRollbackLoading(false);
    }
  };

  const handlePreviewConfirm = (shas: string[]) => {
    setRollbackPreviewData(null);
    setRollbackConfirmShas(shas);
  };

  const handleConfirmExecute = async () => {
    const shas = rollbackConfirmShas();
    if (!shas) return;
    setRollbackUiError(null);
    setLastIncludeShas(shas);
    setRollbackConfirmShas(null);
    setRollbackProgress({
      done: 0,
      total: shas.length,
      currentSha: null,
      // 回滚已启动 = in_progress（spec §K union · 替换原越界 magic "starting"）
      status: "in_progress",
    });
    try {
      await rollbackExecute(sessionId, shas);
      setRolledBack(true);
    } catch (err) {
      const parsed = handleRollbackError(err, "rollback execute");
      if (parsed?.kind !== "conflictDetected") {
        setRollbackProgress(null);
      }
    }
  };

  const handleRollbackAbort = async () => {
    setRollbackUiError(null);
    try {
      await rollbackAbort(sessionId);
    } catch (err) {
      handleRollbackError(err, "rollback abort");
    } finally {
      setRollbackProgress(null);
      setRollbackConflict(null);
    }
  };

  onMount(async () => {
    const unlistenConflict = await onRollbackConflict((payload) => {
      setRollbackConflict({
        path: payload.path,
        commitSha: payload.commitSha,
        includeShas: lastIncludeShas(),
      });
      setRollbackProgress(null);
    });
    const unlistenDone = await onRollbackDone(() => {
      setRollbackConflict(null);
      setRollbackProgress(null);
      setRolledBack(true);
    });
    const unlistenAbort = await onRollbackAborted(() => {
      setRollbackConflict(null);
      setRollbackProgress(null);
    });
    onCleanup(() => {
      unlistenConflict();
      unlistenDone();
      unlistenAbort();
    });
  });

  return (
    <div class="vs-session-detail" role="region" aria-label="Session detail">
      <Show when={ctx.store.lastError || rollbackUiError()}>
        <div class="vs-session-error-bar" role="alert" aria-live="polite">
          <span>
            <Show
              when={ctx.store.lastError}
              fallback={`Error: rollback_failed — ${rollbackUiError()}`}
            >
              {(err) => (
                <>
                  Error: {err().kind}
                  {"detail" in err()
                    ? ` — ${(err() as { detail: string }).detail}`
                    : ""}
                </>
              )}
            </Show>
          </span>
          <button
            class="vs-session-error-dismiss"
            onClick={() => {
              ctx.store.clearError();
              setRollbackUiError(null);
            }}
            aria-label="Dismiss error"
          >
            Dismiss
          </button>
        </div>
      </Show>

      <Show when={!rollbackConflict() && rollbackProgress()}>
        {(prog) => (
          <RollbackProgressBanner
            sessionId={sessionId}
            progress={prog()}
            onAbort={() => void handleRollbackAbort()}
          />
        )}
      </Show>

      <Show
        when={!detailData.loading && detailData()}
        fallback={
          <Show
            when={detailData.error}
            fallback={
              <div class="vs-session-detail-loading" aria-live="polite">
                Loading session…
              </div>
            }
          >
            <div
              class="vs-session-detail-error"
              role="alert"
              aria-live="assertive"
            >
              Session not found or failed to load.
            </div>
          </Show>
        }
      >
        {(data: () => SessionDetailResult) => (
          <>
            <SessionHeader
              session={data().session}
              onClose={() => ctx.closeDetail()}
            />
            <SessionSummary data={data()} />
            <SessionTimeRange session={data().session} />
            <SessionCommitPanel
              links={data().links}
              commitAnchor={commitAnchor}
              onUnbind={setUnbindTarget}
              onRebind={setRebindTarget}
            />
            <SessionActions
              session={data().session}
              links={data().links}
              onEnd={handleEndSession}
              onRecalcAll={handleRecalcAll}
              onRollback={() => void handleRollbackClick()}
              rollbackLoading={rollbackLoading()}
              rolledBack={rolledBack()}
            />
          </>
        )}
      </Show>

      <Show when={unbindTarget()}>
        {(link) => (
          <SessionUnbindModal
            link={link()}
            sessionTitle={detailData()?.session.title ?? ""}
            onCancel={() => setUnbindTarget(null)}
            onUnbind={(reason) => handleUnbind(link(), reason, false)}
            onUnbindAndRecalc={(reason) => handleUnbind(link(), reason, true)}
          />
        )}
      </Show>

      <Show when={rebindTarget()}>
        {(link) => (
          <SessionRebindModal
            link={link()}
            workspaceId={workspaceId()}
            currentSessionId={sessionId}
            onCancel={() => setRebindTarget(null)}
            onRebind={(targetId, reason) =>
              handleRebind(link(), targetId, reason)
            }
          />
        )}
      </Show>

      <Show when={rollbackPreviewData()}>
        {(preview) => (
          <RollbackPreviewModal
            preview={preview()}
            onCancel={() => setRollbackPreviewData(null)}
            onConfirm={handlePreviewConfirm}
          />
        )}
      </Show>

      <Show when={rollbackConfirmShas()}>
        {(shas) => (
          <RollbackConfirmDialog
            sessionId={sessionId}
            commitCount={shas().length}
            onCancel={() => setRollbackConfirmShas(null)}
            onExecute={() => void handleConfirmExecute()}
          />
        )}
      </Show>

      <Show when={rollbackConflict()}>
        {(conflict) => (
          <RollbackConflictView
            sessionId={sessionId}
            workspaceId={workspaceId()}
            includeShas={conflict().includeShas}
            progress={
              rollbackProgress() ?? {
                done: 0,
                total: conflict().includeShas.length,
              }
            }
            initialConflictFile={conflict().path}
            onResume={async () => {
              setRollbackUiError(null);
              try {
                await rollbackExecute(sessionId, conflict().includeShas);
              } catch (err) {
                const parsed = handleRollbackError(err, "rollback resume");
                if (parsed?.kind !== "conflictDetected") {
                  setRollbackConflict(null);
                }
                throw err;
              }
            }}
            onAbort={async () => {
              setRollbackUiError(null);
              try {
                await rollbackAbort(sessionId);
                setRollbackConflict(null);
                setRollbackProgress(null);
              } catch (err) {
                handleRollbackError(err, "rollback abort");
                throw err;
              }
            }}
            onCompleted={() => setRollbackConflict(null)}
          />
        )}
      </Show>
    </div>
  );
}

function SessionHeader(props: {
  session: AiSession;
  onClose: () => void;
}): JSX.Element {
  return (
    <div class="vs-session-detail-header">
      <button
        class="vs-session-detail-close"
        onClick={() => props.onClose()}
        aria-label="Close session detail"
      >
        ✕
      </button>
      <h3 class="vs-session-detail-title">{props.session.title}</h3>
      <span
        class="vs-session-detail-cli-tag"
        aria-label={`CLI: ${props.session.cliKind}`}
      >
        {props.session.cliKind}
      </span>
      <span
        class="vs-session-status-badge"
        data-status={props.session.status}
        role="status"
        aria-label={`Status: ${statusLabel(props.session.status)}`}
      >
        {statusLabel(props.session.status)}
      </span>
    </div>
  );
}

function SessionSummary(props: { data: SessionDetailResult }): JSX.Element {
  return (
    <div class="vs-session-summary" aria-label="Session summary">
      <div class="vs-session-summary-item">
        <span>Commits:</span>
        <span class="vs-session-summary-value">{props.data.commitCount}</span>
      </div>
      <div class="vs-session-summary-item">
        <span>Avg confidence:</span>
        <span class="vs-session-summary-value">
          {(props.data.avgConfidence * 100).toFixed(1)}%
        </span>
      </div>
      <div class="vs-session-summary-item">
        <span>Prompts:</span>
        <span class="vs-session-summary-value">
          {props.data.session.promptCount}
        </span>
      </div>
    </div>
  );
}

function SessionTimeRange(props: { session: AiSession }): JSX.Element {
  return (
    <div class="vs-session-time-range" aria-label="Session time range">
      <span>Started: {fmtTime(props.session.startedAt)}</span>
      <Show when={props.session.endedAt}>
        {(end) => <span>Ended: {fmtTime(end())}</span>}
      </Show>
    </div>
  );
}

function SessionCommitPanel(props: {
  links: SessionCommitLink[];
  commitAnchor?: string;
  onUnbind: (link: SessionCommitLink) => void;
  onRebind: (link: SessionCommitLink) => void;
}): JSX.Element {
  return (
    <div class="vs-session-commits">
      <h4 class="vs-session-commits-title">
        Linked commits ({props.links.length})
      </h4>
      <Show
        when={props.links.length > 0}
        fallback={
          <div class="vs-session-detail-empty">No commits linked yet.</div>
        }
      >
        <ul
          class="vs-session-commit-list"
          role="list"
          aria-label="Linked commits"
        >
          <For each={props.links}>
            {(link) => (
              <CommitRow
                link={link}
                isAnchor={link.commitSha === props.commitAnchor}
                onUnbind={() => props.onUnbind(link)}
                onRebind={() => props.onRebind(link)}
              />
            )}
          </For>
        </ul>
      </Show>
    </div>
  );
}

function CommitRow(props: {
  link: SessionCommitLink;
  isAnchor: boolean;
  onUnbind: () => void;
  onRebind: () => void;
}): JSX.Element {
  const active = () => isLinkActive(props.link);
  const confirmed = () => isLinkConfirmed(props.link);

  return (
    <li
      class="vs-session-commit-row"
      data-anchor={props.isAnchor || undefined}
      tabIndex={0}
      role="listitem"
      aria-label={`Commit ${shortSha(props.link.commitSha)}, state: ${props.link.linkState}, confidence: ${(props.link.confidence * 100).toFixed(0)}%`}
    >
      <code class="vs-session-commit-sha">
        {shortSha(props.link.commitSha)}
      </code>
      <span
        class="vs-session-commit-state"
        data-state={props.link.linkState}
        aria-label={`Link state: ${props.link.linkState}`}
      >
        {props.link.linkState}
        {!confirmed() && active() ? " ⚠" : ""}
      </span>
      <span class="vs-session-commit-confidence">
        {(props.link.confidence * 100).toFixed(0)}%
      </span>
      <div class="vs-session-commit-actions">
        <Show when={active()}>
          <button
            class="vs-session-commit-action-btn is-danger"
            onClick={() => props.onUnbind()}
            aria-label={`Unbind commit ${shortSha(props.link.commitSha)}`}
          >
            Unbind
          </button>
          <button
            class="vs-session-commit-action-btn"
            onClick={() => props.onRebind()}
            aria-label={`Rebind commit ${shortSha(props.link.commitSha)}`}
          >
            Rebind
          </button>
        </Show>
      </div>
    </li>
  );
}

function SessionActions(props: {
  session: AiSession;
  links: SessionCommitLink[];
  onEnd: (session: AiSession) => Promise<void>;
  onRecalcAll: (links: SessionCommitLink[]) => Promise<void>;
  onRollback: () => void;
  rollbackLoading: boolean;
  rolledBack: boolean;
}): JSX.Element {
  return (
    <div class="vs-session-actions">
      <Show when={props.session.status === "active"}>
        <button
          class="vs-dialog-btn-secondary"
          onClick={() => void props.onEnd(props.session)}
          aria-label="End this session"
        >
          End session
        </button>
      </Show>
      <Show when={props.links.length > 0}>
        <button
          class="vs-dialog-btn-secondary"
          onClick={() => void props.onRecalcAll(props.links)}
          aria-label="Recalculate all commit bindings"
        >
          Recalc all
        </button>
      </Show>
      <Show
        when={!props.rolledBack}
        fallback={
          <span class="vs-session-rolled-back-badge" title="此 session 已回滚">
            已回滚
          </span>
        }
      >
        <button
          class="vs-session-rollback-btn"
          onClick={() => props.onRollback()}
          disabled={props.rollbackLoading || props.links.length === 0}
          aria-label={`回滚 Session #${props.session.id} 的所有 AI commit`}
          title={props.links.length === 0 ? "无可回滚的 commit" : undefined}
        >
          {props.rollbackLoading ? "加载中…" : "↩ 一键回滚"}
        </button>
      </Show>
    </div>
  );
}
