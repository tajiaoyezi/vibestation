import { highlightInWorker } from "./worker-client";

export const STREAMING_THRESHOLDS = {
  SIZE_1MB: 1 * 1024 * 1024,
  SIZE_10MB: 10 * 1024 * 1024,
  CHUNK_SIZE_BYTES: 100 * 1024,
} as const;

const IDLE_TIMEOUT_MS = 1000;

type HighlightTheme = "light" | "dark";

type SyncHighlightFn = (
  code: string,
  lang: string,
  theme: HighlightTheme,
) => Promise<string>;

function runInIdleSlot(task: () => Promise<string>): Promise<string> {
  return new Promise((resolve, reject) => {
    const run = () => {
      void task().then(resolve).catch(reject);
    };

    if (typeof requestIdleCallback === "function") {
      requestIdleCallback(
        () => {
          run();
        },
        { timeout: IDLE_TIMEOUT_MS },
      );
      return;
    }

    setTimeout(run, 0);
  });
}

export async function scheduleHighlight(
  code: string,
  lang: string,
  theme: HighlightTheme,
  fileSize: number,
  syncHighlight: SyncHighlightFn,
): Promise<string> {
  if (fileSize < STREAMING_THRESHOLDS.SIZE_1MB) {
    return syncHighlight(code, lang, theme);
  }

  if (fileSize < STREAMING_THRESHOLDS.SIZE_10MB) {
    return runInIdleSlot(() => syncHighlight(code, lang, theme));
  }

  try {
    return await highlightInWorker(code, lang, theme);
  } catch (err) {
    console.warn(
      "[shiki] worker highlight failed, fallback to requestIdleCallback:",
      err instanceof Error ? err.message : String(err),
    );
    return runInIdleSlot(() => syncHighlight(code, lang, theme));
  }
}
