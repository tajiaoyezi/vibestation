import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import {
  scheduleHighlight,
  STREAMING_THRESHOLDS,
} from "../../../../src/utils/shiki/scheduler";
import { highlightInWorker } from "../../../../src/utils/shiki/worker-client";

vi.mock("../../../../src/utils/shiki/worker-client", () => ({
  highlightInWorker: vi.fn(),
}));

const mockedHighlightInWorker = vi.mocked(highlightInWorker);

describe("G.5 Web Worker 创建失败 fallback", () => {
  beforeEach(() => {
    mockedHighlightInWorker.mockReset();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("Worker 调用失败时应 fallback 到 requestIdleCallback", async () => {
    mockedHighlightInWorker.mockRejectedValue(new Error("Worker disabled"));

    const syncHighlight = vi.fn().mockResolvedValue("idle-fallback-html");
    const idleCallback = vi.fn((cb: IdleRequestCallback) => {
      cb({ didTimeout: false, timeRemaining: () => 50 } as IdleDeadline);
      return 1;
    });
    vi.stubGlobal("requestIdleCallback", idleCallback);

    const result = await scheduleHighlight(
      "const x = 1;",
      "typescript",
      "light",
      20 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("idle-fallback-html");
    expect(idleCallback).toHaveBeenCalledOnce();
    expect(syncHighlight).toHaveBeenCalledOnce();
    expect(mockedHighlightInWorker).toHaveBeenCalledOnce();
  });

  it("Worker timeout 时应 fallback", async () => {
    mockedHighlightInWorker.mockRejectedValue(new Error("worker timeout"));

    const syncHighlight = vi.fn().mockResolvedValue("timeout-fallback-html");
    const idleCallback = vi.fn((cb: IdleRequestCallback) => {
      cb({ didTimeout: false, timeRemaining: () => 50 } as IdleDeadline);
      return 1;
    });
    vi.stubGlobal("requestIdleCallback", idleCallback);

    const result = await scheduleHighlight(
      "const y = 2;",
      "typescript",
      "light",
      20 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("timeout-fallback-html");
    expect(mockedHighlightInWorker).toHaveBeenCalledOnce();
  });

  it("Worker runtime error 时应 fallback", async () => {
    mockedHighlightInWorker.mockRejectedValue(new Error("worker crash"));

    const syncHighlight = vi.fn().mockResolvedValue("error-fallback-html");
    const idleCallback = vi.fn((cb: IdleRequestCallback) => {
      cb({ didTimeout: false, timeRemaining: () => 50 } as IdleDeadline);
      return 1;
    });
    vi.stubGlobal("requestIdleCallback", idleCallback);

    const result = await scheduleHighlight(
      "const z = 3;",
      "typescript",
      "light",
      20 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("error-fallback-html");
  });

  it("requestIdleCallback 不可用时 fallback 到 setTimeout", async () => {
    mockedHighlightInWorker.mockRejectedValue(new Error("Worker not available"));

    const syncHighlight = vi.fn().mockResolvedValue("settimeout-fallback-html");
    vi.stubGlobal("requestIdleCallback", undefined);

    const setTimeoutSpy = vi
      .spyOn(global, "setTimeout")
      .mockImplementation((cb: TimerHandler) => {
        if (typeof cb === "function") cb();
        return 1 as unknown as ReturnType<typeof setTimeout>;
      });

    const result = await scheduleHighlight(
      "const w = 4;",
      "typescript",
      "light",
      20 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("settimeout-fallback-html");
    expect(setTimeoutSpy).toHaveBeenCalled();
  });
});
