import { createSignal, For, Show, type Component } from "solid-js";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import type { ConflictFile } from "../../bindings";
import "./pullConflictDialog.css";

interface PullConflictDialogProps {
  workspacePath: string;
  remote: string;
  branch: string;
  files: ConflictFile[];
  onCopied: () => void;
  onClose: () => void;
}

export const PullConflictDialog: Component<PullConflictDialogProps> = (
  props,
) => {
  const [copying, setCopying] = createSignal(false);
  const visibleFiles = () => props.files.slice(0, 8);
  const hiddenCount = () =>
    Math.max(0, props.files.length - visibleFiles().length);
  const command = () =>
    `cd ${shellQuote(props.workspacePath)} && git pull ${props.remote} ${props.branch}`;

  const copyCommand = async () => {
    setCopying(true);
    try {
      await writeText(command());
      props.onCopied();
    } finally {
      setCopying(false);
    }
  };

  return (
    <div
      class="vs-dialog-overlay"
      role="dialog"
      aria-modal="true"
      aria-labelledby="vs-pull-conflict-title"
    >
      <div class="vs-dialog vs-pull-conflict-dialog">
        <h3 id="vs-pull-conflict-title" class="vs-dialog-title">
          合并冲突 · 已自动中止
        </h3>
        <div class="vs-dialog-body vs-pull-conflict-body">
          <p>
            以下 {props.files.length} 个文件含冲突 · 工作区已恢复到 pull
            前状态：
          </p>
          <ul class="vs-pull-conflict-list">
            <For each={visibleFiles()}>
              {(file) => (
                <li>
                  <span>{file.path}</span>
                </li>
              )}
            </For>
          </ul>
          <Show when={hiddenCount() > 0}>
            <p class="vs-pull-conflict-more">还有 {hiddenCount()} 个文件</p>
          </Show>
          <p class="vs-pull-conflict-copy">
            v0.2 不支持 GUI 解决 · 请在终端运行 git pull 后用 git mergetool /
            编辑器手动解决。
          </p>
        </div>
        <div class="vs-dialog-actions">
          <button
            type="button"
            class="vs-dialog-btn-secondary"
            onClick={() => void copyCommand()}
            disabled={copying()}
          >
            {copying() ? "复制中…" : "复制 git pull 命令"}
          </button>
          <button
            type="button"
            class="vs-dialog-btn-primary"
            onClick={props.onClose}
          >
            OK
          </button>
        </div>
      </div>
    </div>
  );
};

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", "'\\''")}'`;
}
