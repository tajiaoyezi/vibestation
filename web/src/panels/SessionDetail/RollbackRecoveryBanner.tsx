import { Show, type JSX } from "solid-js";
import type { RollbackRecoveryUiState } from "../../lib/rollback-recovery";

// MVP-20 Phase D · 全局 rollback crash recovery banner（spec §H.9 · §I）。
//
// app 启动检测到 REVERT_HEAD + DB in_progress/conflict_paused 时渲染。
// 与 MVP-16 ConflictBanner 平行（rollback 是 session 维度 · 自有状态机 ·
// spec §B.4：MVP-20 只复用 MVP-16 冲突 *解决* UI · recovery banner 自建）。
//
// Abort 永远安全（cleanup_state + 反向 revert 已完成项 → 回到 rollback 前）·
// 故为主操作；Resume 仅 conflict_paused 可用（后端 resume 路径 ·
// canResume 守护）；Dismiss 延后由 Session 详情处理。

interface RollbackRecoveryBannerProps {
  state: RollbackRecoveryUiState;
  busy: boolean;
  error: string | null;
  onAbort: () => void;
  onResume: () => void;
  onDismiss: () => void;
}

export function RollbackRecoveryBanner(
  props: RollbackRecoveryBannerProps,
): JSX.Element {
  return (
    <div
      class="vs-rollback-recovery-banner"
      role="alert"
      aria-label={`检测到未完成的回滚 · Session #${props.state.sessionId}`}
    >
      <div class="vs-rollback-recovery-info">
        <span class="vs-rollback-recovery-icon" aria-hidden="true">
          ↩
        </span>
        <span>
          检测到未完成的回滚 · Session #{props.state.sessionId} ·{" "}
          {props.state.currentIdx}/{props.state.total} 已处理
          <Show when={props.state.status === "conflict_paused"}>
            {" "}
            · <strong>存在未解决冲突</strong>
          </Show>
        </span>
      </div>
      <div class="vs-rollback-recovery-actions">
        <button
          type="button"
          class="vs-rollback-abort-btn"
          disabled={props.busy}
          onClick={() => props.onAbort()}
        >
          中止回滚
        </button>
        <Show when={props.state.canResume}>
          <button
            type="button"
            class="vs-rollback-recovery-resume-btn"
            disabled={props.busy}
            onClick={() => props.onResume()}
          >
            继续回滚
          </button>
        </Show>
        <button
          type="button"
          class="vs-rollback-recovery-dismiss-btn"
          disabled={props.busy}
          onClick={() => props.onDismiss()}
        >
          关闭
        </button>
      </div>
      <Show when={props.error}>
        {(message) => (
          <div class="vs-rollback-recovery-error" role="alert">
            {message()}
          </div>
        )}
      </Show>
    </div>
  );
}
