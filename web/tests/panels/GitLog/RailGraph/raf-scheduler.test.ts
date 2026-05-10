import { describe, expect, it, vi } from "vitest";
import {
  createRailFrameScheduler,
  createRailPerformanceSampler,
} from "../../../../src/panels/GitLog/RailGraph/raf-scheduler";

function makeRafHarness() {
  let nextHandle = 1;
  const callbacks = new Map<number, FrameRequestCallback>();
  const requestAnimationFrame = vi.fn((callback: FrameRequestCallback) => {
    const handle = nextHandle++;
    callbacks.set(handle, callback);
    return handle;
  });
  const cancelAnimationFrame = vi.fn((handle: number) => {
    callbacks.delete(handle);
  });
  const step = (timestamp = 16) => {
    const pending = Array.from(callbacks.entries());
    callbacks.clear();
    for (const [, callback] of pending) callback(timestamp);
  };

  return { requestAnimationFrame, cancelAnimationFrame, step, callbacks };
}

describe("createRailFrameScheduler", () => {
  it("coalesces many invalidations into one redraw per animation frame", () => {
    const raf = makeRafHarness();
    const scheduler = createRailFrameScheduler(raf);
    const draw = vi.fn();

    for (let i = 0; i < 100; i++) scheduler.invalidate(draw);
    expect(raf.requestAnimationFrame).toHaveBeenCalledTimes(1);

    raf.step();

    expect(draw).toHaveBeenCalledTimes(1);
  });

  it("uses the latest draw callback before the frame runs", () => {
    const raf = makeRafHarness();
    const scheduler = createRailFrameScheduler(raf);
    const stale = vi.fn();
    const latest = vi.fn();

    scheduler.invalidate(stale);
    scheduler.invalidate(latest);
    raf.step();

    expect(stale).not.toHaveBeenCalled();
    expect(latest).toHaveBeenCalledTimes(1);
  });

  it("allows a new redraw on a later animation frame", () => {
    const raf = makeRafHarness();
    const scheduler = createRailFrameScheduler(raf);
    const draw = vi.fn();

    scheduler.invalidate(draw);
    raf.step(16);
    scheduler.invalidate(draw);
    raf.step(32);

    expect(draw).toHaveBeenCalledTimes(2);
  });

  it("cancels a queued redraw when disposed", () => {
    const raf = makeRafHarness();
    const scheduler = createRailFrameScheduler(raf);
    const draw = vi.fn();

    scheduler.invalidate(draw);
    scheduler.dispose();
    raf.step();

    expect(raf.cancelAnimationFrame).toHaveBeenCalledTimes(1);
    expect(draw).not.toHaveBeenCalled();
  });
});

describe("createRailPerformanceSampler", () => {
  it("marks every sampled frame without flooding every draw", () => {
    const mark = vi.fn();
    const sampler = createRailPerformanceSampler({
      sampleEvery: 3,
      performance: { mark },
    });

    for (let i = 0; i < 6; i++) {
      const finish = sampler.startFrame();
      finish();
    }

    expect(mark.mock.calls.map((call) => call[0])).toEqual([
      "mvp-12.draw.start",
      "mvp-12.draw.end",
      "mvp-12.draw.start",
      "mvp-12.draw.end",
    ]);
  });
});
