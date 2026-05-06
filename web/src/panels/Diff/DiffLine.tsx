import {
  createEffect,
  createSignal,
  type Component,
} from "solid-js";
import { shikiAdapter, guessLanguageFromPath } from "../../utils/shiki";

interface DiffLineProps {
  content: string;
  filePath: string;
  lineType: string;
}

export const DiffLineContent: Component<DiffLineProps> = (props) => {
  const [highlighted, setHighlighted] = createSignal<string | null>(null);

  createEffect(() => {
    const lang = guessLanguageFromPath(props.filePath);
    if (!lang) {
      setHighlighted(null);
      return;
    }

    // 异步加载 syntax highlight
    shikiAdapter
      .highlight(props.content, lang, getCurrentTheme())
      .then((html) => {
        setHighlighted(html);
      })
      .catch(() => {
        setHighlighted(null);
      });
  });

  return (
    <span
      class="vs-diff-line-content"
      innerHTML={highlighted() ?? props.content}
    />
  );
};

function getCurrentTheme(): "light" | "dark" {
  const theme = document.documentElement.getAttribute("data-shiki-theme");
  return theme === "dark" ? "dark" : "light";
}
