import { Show, type JSX } from "solid-js";
import type { RollbackProgress } from "./rollbackContract";

interface RollbackProgressBannerProps {
  sessionId: string;
  progress: RollbackProgress;
  onAbort: () => void;
}

function shortSha(sha: string): string {
  return sha.slice(0, 7);
}

export function RollbackProgressBanner(
  props: RollbackProgressBannerProps,
): JSX.Element {
  const pct = (): number =>
    props.progress.total > 0
      ? Math.round((props.progress.done / props.progress.total) * 100)
      : 0;

  return (
    <div
      class="vs-rollback-progress-banner"
      role="status"
      aria-live="polite"
      aria-label={`回滚进度：${props.progress.done}/${props.progress.total} 完成`}
    >
      <div class="vs-rollback-progress-info">
        <span class="vs-rollback-progress-icon">↩</span>
        <span>
          正在回滚 Session #{props.sessionId} · {props.progress.done}/
          {props.progress.total} 已完成
        </span>
        <span class="vs-rollback-progress-pct">{pct()}%</span>
      </div>
      <div class="vs-rollback-progress-bar-track">
        <div
          class="vs-rollback-progress-bar-fill"
          style={{ width: `${pct()}%` }}
        />
      </div>
      <Show when={props.progress.current_sha}>
        <div class="vs-rollback-progress-current">
          当前：revert {shortSha(props.progress.current_sha)} "
          {props.progress.status}"
        </div>
      </Show>
      <button
        type="button"
        class="vs-rollback-abort-btn"
        onClick={() => props.onAbort()}
        aria-label="取消回滚"
      >
        取消回滚
      </button>
    </div>
  );
}
