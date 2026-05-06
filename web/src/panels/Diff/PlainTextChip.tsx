// MVP-15 Phase B · 不识别语言时的视觉提示 chip
//
// spec §B.6 / §E.5 / UI 引用 line 216-218：
// - 已识别 lang · 返回 null（chip 不渲染）
// - 不识别 lang · 显示 "Plain text" 灰色细字 chip · title hover 解释
// - 不弹 toast（避免烦扰）

import { type Component, Show } from "solid-js";
import { guessLanguageFromPath } from "../../utils/shiki";

interface PlainTextChipProps {
  filePath: string;
}

export const PlainTextChip: Component<PlainTextChipProps> = (props) => {
  const lang = () => guessLanguageFromPath(props.filePath);

  return (
    <Show when={lang() === null}>
      <span
        class="vs-diff-plain-text-chip"
        title="此文件类型暂不支持语法高亮 · 作为纯文本显示"
      >
        Plain text
      </span>
    </Show>
  );
};
