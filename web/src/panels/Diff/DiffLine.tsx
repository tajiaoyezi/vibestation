import {
  createEffect,
  createSignal,
  onMount,
  onCleanup,
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
}

export const DiffLineContent: Component<DiffLineProps> = (props) => {
  const [highlighted, setHighlighted] = createSignal<string | null>(null);
  const [visible, setVisible] = createSignal(false);
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

    const lang = guessLanguageFromPath(props.filePath);
    if (!lang) {
      setHighlighted(null);
      return;
    }

    // theme 是 reactive signal · 切换主题时自动重渲（spec §D.1）
    const theme = useShikiTheme();

    shikiAdapter
      .highlight(props.content, lang, theme)
      .then((html) => {
        setHighlighted(html);
      })
      .catch(() => {
        setHighlighted(null);
      });
  });

  // 安全：fallback 时也 escape · 防止 diff 行内容含 HTML / script 注入
  return (
    <span
      ref={lineEl}
      class="vs-diff-line-content"
      innerHTML={highlighted() ?? fallbackToPlainText(props.content)}
    />
  );
};
