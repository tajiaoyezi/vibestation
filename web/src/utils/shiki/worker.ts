import { createHighlighterCore, type HighlighterCore } from "shiki/core";
import { createOnigurumaEngine } from "shiki/engine/oniguruma";
import githubLight from "shiki/themes/github-light.mjs";
import githubDark from "shiki/themes/github-dark.mjs";
import javascript from "shiki/langs/javascript.mjs";
import typescript from "shiki/langs/typescript.mjs";
import rust from "shiki/langs/rust.mjs";
import python from "shiki/langs/python.mjs";
import go from "shiki/langs/go.mjs";
import java from "shiki/langs/java.mjs";
import markdown from "shiki/langs/markdown.mjs";
import json from "shiki/langs/json.mjs";
import yaml from "shiki/langs/yaml.mjs";
import shell from "shiki/langs/shell.mjs";

type HighlightTheme = "light" | "dark";

interface WorkerRequest {
  id: string;
  code: string;
  lang: string;
  theme: HighlightTheme;
}

interface WorkerResponse {
  id: string;
  html: string | null;
  error: string | null;
}

interface WorkerScope {
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
  postMessage: (message: WorkerResponse) => void;
}

let highlighterPromise: Promise<HighlighterCore> | null = null;

function getHighlighter(): Promise<HighlighterCore> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighterCore({
      themes: [githubLight, githubDark],
      langs: [
        javascript,
        typescript,
        rust,
        python,
        go,
        java,
        markdown,
        json,
        yaml,
        shell,
      ],
      engine: createOnigurumaEngine(import("shiki/wasm")),
    });
  }

  return highlighterPromise;
}

const workerScope = self as unknown as WorkerScope;

workerScope.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const { id, code, lang, theme } = event.data;

  try {
    const highlighter = await getHighlighter();
    const html = highlighter.codeToHtml(code, {
      lang,
      theme: theme === "light" ? "github-light" : "github-dark",
    });
    workerScope.postMessage({ id, html, error: null } satisfies WorkerResponse);
  } catch (err) {
    workerScope.postMessage({
      id,
      html: null,
      error: err instanceof Error ? err.message : String(err),
    } satisfies WorkerResponse);
  }
};
