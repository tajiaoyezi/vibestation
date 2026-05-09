import {
  createEffect,
  createMemo,
  createSignal,
  onMount,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import {
  shikiAdapter,
  guessLanguageFromPath,
  fallbackToPlainText,
} from "../../utils/shiki";
import { useShikiTheme } from "../../utils/shiki/theme-store";

interface DiffLineProps {
  content: string;
  filePath: string;
  lineType: string;
  fileSize?: number;
  disableHighlight?: boolean;
}

const MAX_LINE_BYTES = 100 * 1024;
const textEncoder = new TextEncoder();

function getByteLength(content: string): number {
  return textEncoder.encode(content).length;
}

function truncateLineContent(content: string): {
  content: string;
  isTruncated: boolean;
  originalBytes: number;
} {
  const originalBytes = getByteLength(content);
  if (originalBytes <= MAX_LINE_BYTES) {
    return { content, isTruncated: false, originalBytes };
  }

  let low = 0;
  let high = content.length;

  while (low < high) {
    const mid = Math.ceil((low + high) / 2);
    const currentBytes = getByteLength(content.slice(0, mid));
    if (currentBytes <= MAX_LINE_BYTES) {
      low = mid;
    } else {
      high = mid - 1;
    }
  }

  return {
    content: content.slice(0, low),
    isTruncated: true,
    originalBytes,
  };
}

export const DiffLineContent: Component<DiffLineProps> = (props) => {
  const [highlighted, setHighlighted] = createSignal<string | null>(null);
  const [visible, setVisible] = createSignal(false);
  const truncatedLine = createMemo(() => truncateLineContent(props.content));
  let highlightRequestId = 0;
  let lineEl: HTMLSpanElement | undefined;

  // IntersectionObserver lazy load (spec §B.1) · 仅 viewport 内行触发 highlight
  // rootMargin 200px · 滚动方向预加载缓冲区
  onMount(() => {
    if (!lineEl) return;
    if (typeof IntersectionObserver === "undefined") {
      // jsdom / 旧环境 · 直接标 visible 不阻塞
      setVisible(true);
      return;
    }
    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setVisible(true);
            io.unobserve(entry.target);
          }
        }
      },
      { rootMargin: "200px" },
    );
    io.observe(lineEl);
    onCleanup(() => io.disconnect());
  });

  createEffect(() => {
    // viewport 外不 highlight · 节省 shiki parse 成本
    if (!visible()) return;

    if (props.disableHighlight) {
      setHighlighted(null);
      return;
    }

    const displayContent = truncatedLine().content;
    const lang = guessLanguageFromPath(props.filePath);
    if (!lang) {
      setHighlighted(null);
      return;
    }

    // theme 是 reactive signal · 切换主题时自动重渲（spec §D.1）
    const theme = useShikiTheme();
    const currentRequestId = ++highlightRequestId;

    shikiAdapter
      .highlight(displayContent, lang, theme, props.fileSize)
      .then((html) => {
        if (currentRequestId === highlightRequestId) {
          setHighlighted(html);
        }
      })
      .catch(() => {
        if (currentRequestId === highlightRequestId) {
          setHighlighted(null);
        }
      });
  });

  // 安全：fallback 时也 escape · 防止 diff 行内容含 HTML / script 注入
  return (
    <>
      <span
        ref={lineEl}
        class="vs-diff-line-content"
        innerHTML={
          highlighted() ?? fallbackToPlainText(truncatedLine().content)
        }
      />
      <Show when={truncatedLine().isTruncated}>
        <span
          class="vs-diff-line-truncated"
          title={`Line too long · truncated at 100KB / total ${truncatedLine().originalBytes}B`}
        >
          Line too long · truncated at 100KB / total{" "}
          {truncatedLine().originalBytes}B
        </span>
      </Show>
    </>
  );
};
