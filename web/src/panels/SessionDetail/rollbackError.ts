import type { RollbackError } from "../../bindings";

export function parseRollbackError(raw: unknown): RollbackError | null {
  if (typeof raw !== "string") return null;
  try {
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed === "object" && parsed && "kind" in parsed) {
      return parsed as RollbackError;
    }
  } catch {
    // fall through
  }
  return null;
}

export function formatRollbackError(err: RollbackError): string {
  switch (err.kind) {
    case "dirtyWorkingTree":
      return `工作区有未提交改动（modified ${err.modified.length} / staged ${err.staged.length}）`;
    case "conflictDetected":
      return `检测到冲突：${err.commit_sha}`;
    case "sessionNotFound":
      return `Session 不存在：${err.session_id}`;
    case "emptyPlan":
      return `无可回滚 commit：${err.session_id}`;
    case "inProgress":
      return `已有回滚任务进行中：${err.session_id}`;
    case "git2Error":
      return `${err.class}(${err.code}) ${err.message}`;
    case "dbError":
      return err.message;
    default:
      return "未知回滚错误";
  }
}
