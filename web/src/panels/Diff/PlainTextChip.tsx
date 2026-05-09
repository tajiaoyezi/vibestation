// MVP-15 Phase B · 不识别语言时的视觉提示 chip
//
// spec §B.6 / §E.5 / UI 引用 line 216-218：
// - 已识别 lang · 返回 null（chip 不渲染）
// - 不识别 lang · 显示 "Plain text" 灰色细字 chip · title hover 解释
// - 不弹 toast（避免烦扰）

import { type Component, Show } from "solid-js";
import { guessLanguageFromPath } from "../../utils/shiki";

type PlainTextReason = "unsupported-language" | "large-file";

interface PlainTextChipProps {
  filePath: string;
  reason?: PlainTextReason;
  fileSize?: number;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export const PlainTextChip: Component<PlainTextChipProps> = (props) => {
  const lang = () => guessLanguageFromPath(props.filePath);
  const reason = () => props.reason ?? "unsupported-language";
  const showLargeFileChip = () => reason() === "large-file";
  const showUnsupportedChip = () =>
    reason() === "unsupported-language" && lang() === null;
  const chipText = () => {
    if (!showLargeFileChip()) {
      return "Plain text";
    }

    if ((props.fileSize ?? 0) <= 0) {
      return "Large file · 语法高亮已禁用";
    }

    return `Large file (${formatBytes(props.fileSize ?? 0)}) · 语法高亮已禁用`;
  };
  const chipTitle = () =>
    showLargeFileChip()
      ? "文件过大，语法高亮已禁用"
      : "此文件类型暂不支持语法高亮 · 作为纯文本显示";

  return (
    <Show when={showLargeFileChip() || showUnsupportedChip()}>
      <span class="vs-diff-plain-text-chip" title={chipTitle()}>
        {chipText()}
      </span>
    </Show>
  );
};
