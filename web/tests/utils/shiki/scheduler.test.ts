import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";
import {
  scheduleHighlight,
  STREAMING_THRESHOLDS,
} from "../../../src/utils/shiki/scheduler";
import { highlightInWorker } from "../../../src/utils/shiki/worker-client";

vi.mock("../../../src/utils/shiki/worker-client", () => ({
  highlightInWorker: vi.fn(),
}));

const mockedHighlightInWorker = vi.mocked(highlightInWorker);

describe("scheduleHighlight", () => {
  beforeEach(() => {
    mockedHighlightInWorker.mockReset();
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("< 1MB 走同步路径", async () => {
    const syncHighlight = vi.fn().mockResolvedValue("sync-html");
    const idleCallback = vi.fn();
    vi.stubGlobal("requestIdleCallback", idleCallback);

    const result = await scheduleHighlight(
      "const x = 1;",
      "typescript",
      "light",
      STREAMING_THRESHOLDS.SIZE_1MB - 1,
      syncHighlight,
    );

    expect(result).toBe("sync-html");
    expect(syncHighlight).toHaveBeenCalledOnce();
    expect(idleCallback).not.toHaveBeenCalled();
    expect(mockedHighlightInWorker).not.toHaveBeenCalled();
  });

  it("1MB-10MB 走 requestIdleCallback 路径", async () => {
    const syncHighlight = vi.fn().mockResolvedValue("idle-html");
    const idleCallback = vi.fn((cb: IdleRequestCallback) => {
      cb({ didTimeout: false, timeRemaining: () => 50 } as IdleDeadline);
      return 1;
    });
    vi.stubGlobal("requestIdleCallback", idleCallback);

    const result = await scheduleHighlight(
      "const y = 2;",
      "typescript",
      "dark",
      5 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("idle-html");
    expect(idleCallback).toHaveBeenCalledOnce();
    expect(syncHighlight).toHaveBeenCalledOnce();
    expect(mockedHighlightInWorker).not.toHaveBeenCalled();
  });

  it(">= 10MB 走 worker 路径", async () => {
    const syncHighlight = vi.fn().mockResolvedValue("sync-html");
    mockedHighlightInWorker.mockResolvedValue("worker-html");

    const result = await scheduleHighlight(
      "const z = 3;",
      "typescript",
      "light",
      12 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("worker-html");
    expect(mockedHighlightInWorker).toHaveBeenCalledOnce();
    expect(syncHighlight).not.toHaveBeenCalled();
  });

  it("worker 失败后回退到 requestIdleCallback", async () => {
    const syncHighlight = vi.fn().mockResolvedValue("fallback-html");
    mockedHighlightInWorker.mockRejectedValue(new Error("worker failed"));
    const idleCallback = vi.fn((cb: IdleRequestCallback) => {
      cb({ didTimeout: false, timeRemaining: () => 50 } as IdleDeadline);
      return 1;
    });
    vi.stubGlobal("requestIdleCallback", idleCallback);

    const result = await scheduleHighlight(
      "const w = 4;",
      "typescript",
      "dark",
      20 * 1024 * 1024,
      syncHighlight,
    );

    expect(result).toBe("fallback-html");
    expect(mockedHighlightInWorker).toHaveBeenCalledOnce();
    expect(idleCallback).toHaveBeenCalledOnce();
    expect(syncHighlight).toHaveBeenCalledOnce();
  });

  it("导出阈值常量应匹配 1MB / 10MB / 100KB", () => {
    expect(STREAMING_THRESHOLDS.SIZE_1MB).toBe(1 * 1024 * 1024);
    expect(STREAMING_THRESHOLDS.SIZE_10MB).toBe(10 * 1024 * 1024);
    expect(STREAMING_THRESHOLDS.CHUNK_SIZE_BYTES).toBe(100 * 1024);
  });
});
