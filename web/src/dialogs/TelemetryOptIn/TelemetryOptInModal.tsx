import { createSignal, onMount, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

import "./styles.css";

interface TelemetryOptInModalProps {
  /**
   * 用户决策后回调（接受 / 拒绝 都触发）。
   * 调用方负责通过 settings store 自然刷新（settings_changed event）·
   * 不在此处直接控制 modal 显示。
   */
  onDecided?: () => void;
}

/**
 * Mount-time click guard 阈值（毫秒）
 *
 * 修复 critical regression：webview 启动 + modal mount 同帧时 ·
 * macOS WKWebView 把"启动 ready"事件误派发到 modal 内第一个 focusable button ·
 * 用户根本没看到 modal · DB 已被写入虚假决策。本字段在 mount 后该窗口内 ·
 * 所有 button click 视为虚假并 silently 丢弃。200ms = webview first paint
 * + reactive settle 的安全余量 · 真用户不可能在窗口可见后 < 200ms 完成有意点击。
 */
const MOUNT_CLICK_GUARD_MS = 200;

/**
 * MVP-10 §B.1 · Telemetry opt-in 对话框
 *
 * 首次启动（settings.telemetryOptIn === null）渲染 · 用户决策后：
 * - 调 telemetry_opt_in_set IPC 持久化
 * - 后端 emit settings_changed · settings store 自动刷新
 * - settings.telemetryOptIn !== null · 调用方 Show 包裹判断不再渲染本组件
 */
export const TelemetryOptInModal: Component<TelemetryOptInModalProps> = (
  props,
) => {
  const [submitting, setSubmitting] = createSignal(false);

  let mountedAt = 0;
  onMount(() => {
    mountedAt = performance.now();
  });

  const isEarlyClick = (): boolean => {
    if (mountedAt === 0) return true;
    return performance.now() - mountedAt < MOUNT_CLICK_GUARD_MS;
  };

  const decide = async (optIn: boolean): Promise<void> => {
    if (submitting()) return;
    if (isEarlyClick()) {
      // 丢弃 mount 同帧 webview 虚假 click（见 MOUNT_CLICK_GUARD_MS doc）
      return;
    }
    setSubmitting(true);
    try {
      await invoke("telemetry_opt_in_set", { req: { optIn } });
      // 后端 emit settings_changed · settings store 自动 set telemetryOptIn 非 null
      props.onDecided?.();
    } catch (err) {
      // Fail-open：IPC 失败也让用户继续（spec §B.4 acceptance · 不能 stuck modal）
      // 默认回退 opt-out（隐私优先 · ADR-015 D1）
      // eslint-disable-next-line no-console
      console.error("telemetry_opt_in_set failed:", err);
      props.onDecided?.();
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      class="vs-telemetry-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-telemetry-title"
      aria-describedby="vs-telemetry-lede"
    >
      <div class="vs-telemetry-modal">
        <h2 id="vs-telemetry-title" class="vs-telemetry-title">
          Help improve Vibestation
        </h2>

        <p id="vs-telemetry-lede" class="vs-telemetry-lede">
          Vibestation is open source. Anonymous crash reports help us ship safer
          releases — but only if you say yes. Your repository, terminal content,
          and personal data never leave this machine.
        </p>

        <div class="vs-telemetry-grid">
          <section class="vs-telemetry-section vs-telemetry-collect">
            <h3>If you opt in, we collect</h3>
            <ul>
              <li>
                <span class="vs-telemetry-marker">✓</span>
                Anonymous crash hash (SHA-256 of stack trace · not the trace)
              </li>
              <li>
                <span class="vs-telemetry-marker">✓</span>
                App version (e.g. 0.1.0)
              </li>
              <li>
                <span class="vs-telemetry-marker">✓</span>
                OS type (macOS / Linux · no kernel version, no distro)
              </li>
            </ul>
          </section>

          <section class="vs-telemetry-section vs-telemetry-never">
            <h3>We never collect</h3>
            <ul>
              <li>
                <span class="vs-telemetry-marker">✕</span>
                Your IP address
              </li>
              <li>
                <span class="vs-telemetry-marker">✕</span>
                File paths or repository names
              </li>
              <li>
                <span class="vs-telemetry-marker">✕</span>
                Commit messages or diffs
              </li>
              <li>
                <span class="vs-telemetry-marker">✕</span>
                Terminal content or commands
              </li>
              <li>
                <span class="vs-telemetry-marker">✕</span>
                Any personally identifiable information
              </li>
            </ul>
          </section>
        </div>

        <p class="vs-telemetry-changeable">
          You can change your mind anytime in{" "}
          <strong>Preferences → Privacy</strong>.
        </p>

        <div class="vs-telemetry-actions">
          <button
            type="button"
            class="vs-btn-secondary vs-telemetry-btn"
            onClick={() => void decide(false)}
            disabled={submitting()}
          >
            Decline
          </button>
          <button
            type="button"
            class="vs-btn-primary vs-telemetry-btn"
            onClick={() => void decide(true)}
            disabled={submitting()}
          >
            Accept
          </button>
        </div>
      </div>
    </div>
  );
};
