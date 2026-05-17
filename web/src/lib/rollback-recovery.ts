// MVP-20 Phase D · rollback crash recovery 状态机纯函数（与 SolidJS / Tauri
// runtime 解耦 · 单测 web/tests/lib/rollback-recovery.test.ts）。
//
// 镜像 lib/crash-recovery.ts（MVP-16 rebase recovery）的模式 · 但 rollback 是
// session 维度（per-session 字典）而非 workspace 维度 · 故平行独立模块。
//
// payload 类型直接消费 ts-rs 生成的 `RollbackCrashRecovery` binding（spec §K
// single source of truth · 不重复声明 · 避免 crash-recovery.ts 手抄 drift 风险）。

import type { RollbackCrashRecovery, RollbackStatusKind } from "../bindings";

/// backend `git:rollback-crash-recovery-detected` event payload。
export type RollbackRecoveryPayload = RollbackCrashRecovery;

/// 单 session 的 rollback recovery banner 状态。per-session 隔离。
/// `canResume` 决定 banner 是否出「继续回滚」按钮（见 canResumeRollback）。
export type RollbackRecoveryUiState = {
  workspaceId: string;
  sessionId: string;
  status: RollbackStatusKind;
  currentIdx: number;
  total: number;
  currentSha: string | null;
  canResume: boolean;
};

/// 仅 `conflict_paused` 可经 `rollback_execute` resume（后端
/// resume_rollback_execute 路径）；raw `in_progress` 崩溃后端返回
/// `InProgress` 错误（防重入 · 仅可 abort）· 其余为终态/空闲不可恢复。
export function canResumeRollback(status: RollbackStatusKind): boolean {
  return status === "conflict_paused";
}

/// payload → UI state。仅 `in_progress` / `conflict_paused` 产生 banner
/// 状态；终态（completed/aborted）/ idle 视为无需恢复（backend 不应 emit ·
/// defensive 返回 null · 同 crash-recovery payloadToRecoveryState 语义）。
export function payloadToRollbackRecovery(
  payload: RollbackRecoveryPayload,
): RollbackRecoveryUiState | null {
  if (
    payload.status !== "in_progress" &&
    payload.status !== "conflict_paused"
  ) {
    return null;
  }
  return {
    workspaceId: payload.workspaceId,
    sessionId: payload.sessionId,
    status: payload.status,
    currentIdx: payload.currentIdx,
    total: payload.total,
    currentSha: payload.currentSha,
    canResume: canResumeRollback(payload.status),
  };
}

/// 不可变 per-session patch · `next === null` 删除 key · 否则覆盖。
/// 切换 session 不影响其他 session 的 recovery 状态。
/// 删除不存在的 key 返回原引用（避免 SolidJS reactive churn）。
export function reduceRollbackRecoveries(
  prev: Record<string, RollbackRecoveryUiState>,
  sessionId: string,
  next: RollbackRecoveryUiState | null,
): Record<string, RollbackRecoveryUiState> {
  if (next === null) {
    if (!(sessionId in prev)) return prev;
    const { [sessionId]: _omit, ...rest } = prev;
    return rest;
  }
  return { ...prev, [sessionId]: next };
}
