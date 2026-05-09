// MVP-16 Phase C · crash recovery 状态机纯函数（与 SolidJS / Tauri runtime 解耦）
//
// 抽出 App.tsx 内的两条核心逻辑：
//   - normalize: backend 字符串 operation → frontend ConflictOperation enum
//   - reduceRecoveries: per-workspace recovery 字典的不可变更新（增 / 删 / 替换）
//
// 单测在 web/tests/lib/crash-recovery.test.ts。

import type { ConflictOperation } from "../components/ConflictBanner";

/// backend `git:crash-recovery-detected` event payload · 字段镜像
/// Rust `RebaseCrashRecoveryEvent`。仅 in_progress=true workspace 才 emit。
export type CrashRecoveryPayload = {
  workspaceId: string;
  operation: string | null;
  branch: string | null;
  currentStep: number;
  totalSteps: number;
};

/// 单 workspace 的 crash recovery banner 状态。per-workspace 隔离。
export type RecoveryUiState = {
  workspaceId: string;
  operation: ConflictOperation;
  branch: string | null;
  currentStep: number;
  totalSteps: number;
};

/// backend 返回的 `CrashRecoveryState`（rebase_detect_in_progress IPC 输出）·
/// 与 ts-rs 自动生成的 `CrashRecoveryState` binding 字段同。
export type RecoveryDetectResult = {
  inProgress: boolean;
  operation: string | null;
  branch: string | null;
  currentStep: number;
  totalSteps: number;
};

/// backend 字符串 operation → frontend ConflictOperation。
/// 未知字符串 fallback 到 "rebase"（最常见 · 安全降级）。
export function normalizeOperation(operation: string): ConflictOperation {
  switch (operation) {
    case "merge":
      return "merge";
    case "cherrypick":
      return "cherrypick";
    default:
      return "rebase";
  }
}

/// payload → state · null operation 视为不存在（backend 应该没 emit · defensive）
export function payloadToRecoveryState(
  payload: CrashRecoveryPayload,
): RecoveryUiState | null {
  if (!payload.operation) return null;
  return {
    workspaceId: payload.workspaceId,
    operation: normalizeOperation(payload.operation),
    branch: payload.branch,
    currentStep: payload.currentStep,
    totalSteps: payload.totalSteps,
  };
}

/// IPC 检测结果 → state · 同上 · 多了 inProgress 一层判断
export function detectResultToRecoveryState(
  workspaceId: string,
  result: RecoveryDetectResult,
): RecoveryUiState | null {
  if (!result.inProgress || !result.operation) return null;
  return {
    workspaceId,
    operation: normalizeOperation(result.operation),
    branch: result.branch,
    currentStep: result.currentStep,
    totalSteps: result.totalSteps,
  };
}

/// 不可变 patch · `next === null` 删除 key · 否则覆盖。
/// 切换 workspace 不影响其他 workspace 的 recovery 状态。
/// 幂等：写入相同值返回原对象（避免 SolidJS reactive churn）· 但这里偷懒不做引用比较 · setRecoveries 容许新对象。
export function reduceRecoveries(
  prev: Record<string, RecoveryUiState>,
  workspaceId: string,
  next: RecoveryUiState | null,
): Record<string, RecoveryUiState> {
  if (next === null) {
    if (!(workspaceId in prev)) return prev;
    const { [workspaceId]: _omit, ...rest } = prev;
    return rest;
  }
  return { ...prev, [workspaceId]: next };
}
