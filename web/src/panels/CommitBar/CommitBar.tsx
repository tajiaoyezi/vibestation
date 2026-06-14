import { createSignal, Show, type Component } from "solid-js";
import type {
  CommitRequest,
  CommitResponse,
  CommitError,
  SetGitIdentityRequest,
} from "../../bindings";
import { commit as apiCommit, setGitIdentity } from "../GitStatus/gitStatusApi";
import { formatShortcut } from "../../lib/format-shortcut";
import { t } from "../../i18n";
import { useSettings } from "../../stores/settings";
import { IdentityDialog } from "./IdentityDialog";
import "./styles.css";

interface CommitBarProps {
  workspaceId: () => string;
  stagedCount: () => number;
  onCommitSuccess?: (response: CommitResponse) => void;
  onError?: (message: string) => void;
}

type ToastType = "success" | "error";
interface ToastState {
  visible: boolean;
  message: string;
  type: ToastType;
}

export const CommitBar: Component<CommitBarProps> = (props) => {
  const { settings } = useSettings();
  const language = () => settings.language;
  const label = (key: string) => t(key, language());

  const [message, setMessage] = createSignal("");
  const [amend, setAmend] = createSignal(false);
  const [committing, setCommitting] = createSignal(false);
  const [toast, setToast] = createSignal<ToastState>({
    visible: false,
    message: "",
    type: "success",
  });
  const [identityDialogOpen, setIdentityDialogOpen] = createSignal(false);
  const [detachedHeadDialogOpen, setDetachedHeadDialogOpen] =
    createSignal(false);
  const [hookErrorDialogOpen, setHookErrorDialogOpen] = createSignal(false);
  const [hookErrorContent, setHookErrorContent] = createSignal("");
  const [hookErrorExitCode, setHookErrorExitCode] = createSignal(0);
  const [hookErrorCopied, setHookErrorCopied] = createSignal(false);

  const canCommit = () =>
    props.stagedCount() > 0 && message().trim().length > 0 && !committing();

  const showToast = (msg: string, type: ToastType = "success") => {
    setToast({ visible: true, message: msg, type });
    setTimeout(() => {
      setToast((prev) => ({ ...prev, visible: false }));
    }, 3000);
  };

  const handleCommit = async () => {
    const wsId = props.workspaceId();
    if (!wsId || !canCommit()) return;

    setCommitting(true);

    try {
      const req: CommitRequest = {
        workspaceId: wsId,
        message: message().trim(),
        amend: amend(),
      };
      const response = await apiCommit(req);
      setMessage("");
      setAmend(false);
      showToast(`${label("commitBar.committed")} ${response.shortSha}`);
      props.onCommitSuccess?.(response);
    } catch (err) {
      handleCommitError(err);
    } finally {
      setCommitting(false);
    }
  };

  const handleCommitError = (err: unknown) => {
    const raw = err instanceof Error ? err.message : String(err);

    // 尝试解析 CommitError JSON（Tauri 返回的 tagged union）
    try {
      const parsed: CommitError = JSON.parse(raw);
      switch (parsed.kind) {
        case "identityMissing": {
          setIdentityDialogOpen(true);
          return;
        }
        case "detachedHead": {
          setDetachedHeadDialogOpen(true);
          return;
        }
        case "hookFailed": {
          const lines = parsed.stderr.split("\n");
          const last20 = lines.slice(-20).join("\n");
          setHookErrorContent(last20);
          setHookErrorExitCode(parsed.exit_code);
          setHookErrorCopied(false);
          setHookErrorDialogOpen(true);
          props.onError?.(
            label("commitBar.hookFailedSummary").replace(
              "{code}",
              String(parsed.exit_code),
            ),
          );
          return;
        }
        case "noStagedFiles": {
          showToast(label("commitBar.noStagedChanges"), "error");
          props.onError?.(label("commitBar.noStagedChanges"));
          return;
        }
        case "git2Error": {
          showToast(parsed.message, "error");
          props.onError?.(parsed.message);
          return;
        }
      }
    } catch {
      // 不是 JSON，回退到原始错误
    }

    showToast(raw, "error");
    props.onError?.(raw);
  };

  const handleIdentityConfirm = async (
    name: string,
    email: string,
    saveLocal: boolean,
  ) => {
    const wsId = props.workspaceId();
    if (!wsId) return;

    try {
      const req: SetGitIdentityRequest = {
        workspaceId: wsId,
        name,
        email,
        scope: saveLocal ? "local" : "global",
      };
      await setGitIdentity(req);
      setIdentityDialogOpen(false);
      // 自动重试 commit
      await handleCommit();
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`${label("commitBar.identitySaveFailed")} ${msg}`, "error");
    }
  };

  const handleDetachedHeadConfirm = () => {
    setDetachedHeadDialogOpen(false);
    showToast(label("commitBar.detachedHeadBody"), "error");
  };

  const copyHookStderr = async () => {
    try {
      await navigator.clipboard.writeText(hookErrorContent());
      setHookErrorCopied(true);
      window.setTimeout(() => setHookErrorCopied(false), 1500);
    } catch {
      // 剪贴板拒绝 · 静默失败（用户仍可手动选区复制）
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    // ⌘↵ (macOS) / Ctrl+↵ (Linux)
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      void handleCommit();
    }
  };

  const tooltipText = () => {
    if (props.stagedCount() === 0) return label("commitBar.noStagedChanges");
    if (message().trim().length === 0) return label("commitBar.needsMessage");
    return "";
  };

  // 72 字符提示线计算（基于 JetBrains Mono 平均字符宽度）
  const charWidthPx = 7.2; // JetBrains Mono 14px 近似宽度
  const guideLineOffset = Math.round(72 * charWidthPx);

  return (
    <>
      <div class="vs-commit-bar">
        <div class="vs-commit-input-wrap">
          <div
            class="vs-commit-guide-line"
            style={{ left: `${guideLineOffset}px` }}
          />
          <textarea
            class="vs-commit-message"
            placeholder={`${label("commitBar.messagePlaceholder")} (${formatShortcut("⌘↵", "Ctrl+Enter")} ${label("commitBar.submit")})`}
            value={message()}
            onInput={(e) => setMessage(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            rows={3}
            disabled={committing()}
          />
        </div>

        <div class="vs-commit-actions">
          <label class="vs-commit-amend">
            <input
              type="checkbox"
              checked={amend()}
              onChange={(e) => setAmend(e.currentTarget.checked)}
              disabled={committing()}
            />
            <span>{label("commitBar.amend")}</span>
          </label>

          <div class="vs-commit-btn-wrap" title={tooltipText()}>
            <button
              type="button"
              class="vs-commit-btn"
              onClick={() => void handleCommit()}
              disabled={!canCommit() || committing()}
            >
              {committing()
                ? label("commitBar.committing")
                : label("commitBar.commit")}
            </button>
          </div>
        </div>
      </div>

      <Show when={toast().visible}>
        <div
          class={`vs-toast vs-toast-${toast().type}`}
          onClick={() => setToast((prev) => ({ ...prev, visible: false }))}
        >
          {toast().message}
        </div>
      </Show>

      <Show when={identityDialogOpen()}>
        <IdentityDialog
          onConfirm={handleIdentityConfirm}
          onCancel={() => setIdentityDialogOpen(false)}
        />
      </Show>

      <Show when={detachedHeadDialogOpen()}>
        <div class="vs-dialog-overlay" role="dialog" aria-modal="true">
          <div class="vs-dialog">
            <h3 class="vs-dialog-title">
              {label("commitBar.detachedHeadTitle")}
            </h3>
            <p class="vs-dialog-body">{label("commitBar.detachedHeadBody")}</p>
            <div class="vs-dialog-actions">
              <button
                type="button"
                class="vs-dialog-btn-primary"
                onClick={() => handleDetachedHeadConfirm()}
              >
                {label("commitBar.detachedHeadOk")}
              </button>
            </div>
          </div>
        </div>
      </Show>

      <Show when={hookErrorDialogOpen()}>
        <div class="vs-dialog-overlay" role="dialog" aria-modal="true">
          <div class="vs-dialog">
            <h3 class="vs-dialog-title">
              {label("commitBar.hookFailedTitle")}
            </h3>
            <p class="vs-dialog-hook-meta">
              {label("commitBar.hookFailedMetaPrefix")}{" "}
              <code>{hookErrorExitCode()}</code>{" "}
              {label("commitBar.hookFailedMetaSuffix")}
            </p>
            <pre class="vs-dialog-pre">{hookErrorContent()}</pre>
            <div class="vs-dialog-actions">
              <button
                type="button"
                class="vs-dialog-copy-btn"
                onClick={() => void copyHookStderr()}
                aria-label={label("commitBar.copyStderr")}
              >
                {hookErrorCopied()
                  ? label("commitBar.copied")
                  : label("commitBar.copyStderr")}
              </button>
              <button
                type="button"
                class="vs-dialog-btn-primary"
                onClick={() => setHookErrorDialogOpen(false)}
              >
                {label("commitBar.close")}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </>
  );
};
