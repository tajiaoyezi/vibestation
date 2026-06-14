// MVP-15 Phase B · 不识别语言时的视觉提示 chip
//
// spec §B.6 / §E.5 / UI 引用 line 216-218：
// - 已识别 lang · 返回 null（chip 不渲染）
// - 不识别 lang · 显示 "Plain text" 灰色细字 chip · title hover 解释
// - 不弹 toast（避免烦扰）

import { type Component, Show } from "solid-js";
import { t } from "../../i18n";
import { useSettings } from "../../stores/settings";
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
  const { settings: appSettings } = useSettings();
  const language = () => appSettings.language;
  const label = (key: string) => t(key, language());

  const lang = () => guessLanguageFromPath(props.filePath);
  const reason = () => props.reason ?? "unsupported-language";
  const showLargeFileChip = () => reason() === "large-file";
  const showUnsupportedChip = () =>
    reason() === "unsupported-language" && lang() === null;
  const chipText = () => {
    if (!showLargeFileChip()) {
      return label("diff.plainText");
    }

    const disabled = label("diff.syntaxHighlightDisabled");
    if ((props.fileSize ?? 0) <= 0) {
      return `${label("diff.largeFile")} · ${disabled}`;
    }

    return `${label("diff.largeFile")} (${formatBytes(props.fileSize ?? 0)}) · ${disabled}`;
  };
  const chipTitle = () =>
    showLargeFileChip()
      ? label("diff.largeFileTitle")
      : label("diff.plainTextTitle");

  return (
    <Show when={showLargeFileChip() || showUnsupportedChip()}>
      <span class="vs-diff-plain-text-chip" title={chipTitle()}>
        {chipText()}
      </span>
    </Show>
  );
};