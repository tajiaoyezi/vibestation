import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";

type MockWorkerInstance = {
  onmessage: ((event: MessageEvent<any>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  postMessage: ReturnType<typeof vi.fn>;
  terminate: ReturnType<typeof vi.fn>;
};

const workerHarness = vi.hoisted(() => {
  const instances: MockWorkerInstance[] = [];
  const WorkerConstructor = vi.fn(function MockWorkerConstructor() {
    const instance: MockWorkerInstance = {
      onmessage: null,
      onerror: null,
      postMessage: vi.fn(),
      terminate: vi.fn(),
    };
    instances.push(instance);
    return instance as unknown as Worker;
  });

  return { instances, WorkerConstructor };
});

vi.mock("../../../src/utils/shiki/worker?worker", () => ({
  default: workerHarness.WorkerConstructor,
}));

import {
  getPendingRequestCountForTests,
  highlightInWorker,
  resetWorkerClientForTests,
} from "../../../src/utils/shiki/worker-client";

async function waitForPostedRequest(callIndex: number): Promise<{
  id: string;
  code: string;
  lang: string;
  theme: "light" | "dark";
}> {
  for (let i = 0; i < 50; i += 1) {
    const worker = workerHarness.instances[0];
    if (worker && worker.postMessage.mock.calls.length > callIndex) {
      return worker.postMessage.mock.calls[callIndex][0];
    }
    await new Promise((resolve) => setTimeout(resolve, 0));
  }

  throw new Error("worker request was not posted");
}

function emitWorkerMessage(
  requestId: string,
  payload: { html: string | null; error: string | null },
): void {
  const worker = workerHarness.instances[0];
  if (!worker?.onmessage) {
    throw new Error("worker onmessage is not set");
  }

  worker.onmessage({
    data: { id: requestId, html: payload.html, error: payload.error },
  } as MessageEvent);
}

describe("worker-client", () => {
  beforeEach(async () => {
    workerHarness.instances.length = 0;
    workerHarness.WorkerConstructor.mockClear();
    await resetWorkerClientForTests();
  });

  afterEach(async () => {
    vi.useRealTimers();
    await resetWorkerClientForTests();
  });

  it("多次 highlightInWorker 复用同一个 worker", async () => {
    const firstPromise = highlightInWorker("const a = 1;", "typescript", "light");
    const firstRequest = await waitForPostedRequest(0);
    emitWorkerMessage(firstRequest.id, {
      html: "<span>const a = 1;</span>",
      error: null,
    });
    await expect(firstPromise).resolves.toContain("const a = 1;");

    const secondPromise = highlightInWorker("const b = 2;", "typescript", "dark");
    const secondRequest = await waitForPostedRequest(1);
    emitWorkerMessage(secondRequest.id, {
      html: "<span>const b = 2;</span>",
      error: null,
    });
    await expect(secondPromise).resolves.toContain("const b = 2;");

    expect(workerHarness.WorkerConstructor).toHaveBeenCalledTimes(1);
    expect(workerHarness.instances[0].postMessage).toHaveBeenCalledTimes(2);
  });

  it("worker 请求超时 30s 返回 reject", async () => {
    vi.useFakeTimers();

    const timeoutResult = highlightInWorker(
      "const timeout = true;",
      "typescript",
      "light",
    ).catch((error: unknown) => error);
    await Promise.resolve();
    await vi.advanceTimersByTimeAsync(30_000);

    const error = await timeoutResult;
    expect(error).toBeInstanceOf(Error);
    expect((error as Error).message).toContain("worker timeout");
    expect(getPendingRequestCountForTests()).toBe(0);
  });

  it("worker 成功响应后 pendingRequests 会清理", async () => {
    const promise = highlightInWorker("const x = 1;", "typescript", "light");
    const request = await waitForPostedRequest(0);

    expect(getPendingRequestCountForTests()).toBe(1);

    emitWorkerMessage(request.id, {
      html: "<span>const x = 1;</span>",
      error: null,
    });

    await expect(promise).resolves.toContain("const x = 1;");
    expect(getPendingRequestCountForTests()).toBe(0);
  });
});
