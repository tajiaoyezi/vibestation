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

interface PendingRequest {
  resolve: (html: string) => void;
  reject: (error: Error) => void;
  timeoutId: ReturnType<typeof setTimeout>;
}

const WORKER_TIMEOUT_MS = 30_000;

let workerPromise: Promise<Worker> | null = null;
let requestCounter = 0;
const pendingRequests = new Map<string, PendingRequest>();

function rejectAllPendingRequests(error: Error): void {
  for (const [id, pending] of pendingRequests) {
    clearTimeout(pending.timeoutId);
    pending.reject(error);
    pendingRequests.delete(id);
  }
}

function handleWorkerMessage(event: MessageEvent<WorkerResponse>): void {
  const { id, html, error } = event.data;
  const pending = pendingRequests.get(id);
  if (!pending) {
    return;
  }

  clearTimeout(pending.timeoutId);
  pendingRequests.delete(id);

  if (error) {
    pending.reject(new Error(error));
    return;
  }

  if (html === null) {
    pending.reject(new Error("worker returned null html"));
    return;
  }

  pending.resolve(html);
}

async function getWorker(): Promise<Worker> {
  if (!workerPromise) {
    workerPromise = (async () => {
      const workerModule = (await import("./worker?worker")) as {
        default: new () => Worker;
      };
      const worker = new workerModule.default();
      worker.onmessage = handleWorkerMessage;
      worker.onerror = (event) => {
        rejectAllPendingRequests(
          new Error(event.message || "worker runtime error"),
        );
      };
      return worker;
    })().catch((error) => {
      workerPromise = null;
      throw error;
    });
  }

  return workerPromise;
}

export async function highlightInWorker(
  code: string,
  lang: string,
  theme: HighlightTheme,
): Promise<string> {
  const worker = await getWorker();
  const id = `req-${++requestCounter}`;

  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      pendingRequests.delete(id);
      reject(new Error("worker timeout"));
    }, WORKER_TIMEOUT_MS);

    pendingRequests.set(id, { resolve, reject, timeoutId });
    worker.postMessage({ id, code, lang, theme } satisfies WorkerRequest);
  });
}

export function getPendingRequestCountForTests(): number {
  return pendingRequests.size;
}

export async function resetWorkerClientForTests(): Promise<void> {
  rejectAllPendingRequests(new Error("worker client reset"));

  if (workerPromise) {
    try {
      const worker = await workerPromise;
      worker.terminate();
    } catch {
      // no-op
    }
  }

  workerPromise = null;
  requestCounter = 0;
}
