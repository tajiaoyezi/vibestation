export interface RailFrameSchedulerHost {
  requestAnimationFrame: (callback: FrameRequestCallback) => number;
  cancelAnimationFrame: (handle: number) => void;
}

export interface RailFrameScheduler {
  invalidate: (draw: () => void) => void;
  dispose: () => void;
}

export interface RailPerformanceLike {
  mark: (name: string) => void;
}

export interface RailPerformanceSampler {
  startFrame: () => () => void;
}

function defaultSchedulerHost(): RailFrameSchedulerHost {
  if (typeof window !== "undefined") {
    return {
      requestAnimationFrame: window.requestAnimationFrame.bind(window),
      cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
    };
  }

  return {
    requestAnimationFrame: (callback) =>
      setTimeout(() => callback(Date.now()), 16) as unknown as number,
    cancelAnimationFrame: (handle) =>
      clearTimeout(handle as unknown as ReturnType<typeof setTimeout>),
  };
}

export function createRailFrameScheduler(
  host: RailFrameSchedulerHost = defaultSchedulerHost(),
): RailFrameScheduler {
  let frameHandle: number | null = null;
  let pendingDraw: (() => void) | null = null;
  let disposed = false;

  const runFrame: FrameRequestCallback = () => {
    frameHandle = null;
    if (disposed) return;

    const draw = pendingDraw;
    pendingDraw = null;
    draw?.();
  };

  return {
    invalidate(draw) {
      if (disposed) return;
      pendingDraw = draw;
      if (frameHandle == null) {
        frameHandle = host.requestAnimationFrame(runFrame);
      }
    },
    dispose() {
      disposed = true;
      pendingDraw = null;
      if (frameHandle != null) {
        host.cancelAnimationFrame(frameHandle);
        frameHandle = null;
      }
    },
  };
}

export function createRailPerformanceSampler(options: {
  sampleEvery?: number;
  performance?: RailPerformanceLike | null;
}): RailPerformanceSampler {
  const sampleEvery = Math.max(1, options.sampleEvery ?? 100);
  const perf =
    options.performance ??
    (typeof performance !== "undefined" ? performance : null);
  let frameCount = 0;

  return {
    startFrame() {
      frameCount += 1;
      const shouldSample = frameCount % sampleEvery === 0;
      if (shouldSample) perf?.mark("mvp-12.draw.start");
      return () => {
        if (shouldSample) perf?.mark("mvp-12.draw.end");
      };
    },
  };
}
