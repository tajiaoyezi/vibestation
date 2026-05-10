import { describe, expect, it } from "vitest";
import { computeRailCollapseStrategy } from "../../../../src/panels/GitLog/RailGraph/collapse";
import {
  buildRailRowMetrics,
  computeVisibleRangeFromMetrics,
} from "../../../../src/panels/GitLog/RailGraph/RailGraphVirtualizer";

function percentile(values: number[], p: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((p / 100) * sorted.length) - 1),
  );
  return sorted[index] ?? 0;
}

describe("Phase C virtualizer performance budget", () => {
  it("keeps 100k commit scroll range P99 under 16ms", () => {
    const metrics = buildRailRowMetrics([], 100_000, 32);
    const durations: number[] = [];

    for (let frame = 0; frame < 600; frame++) {
      const scrollTop = (frame / 599) * Math.max(0, metrics.totalHeight - 640);
      const start = performance.now();
      const range = computeVisibleRangeFromMetrics(
        scrollTop,
        640,
        metrics,
        100,
      );
      durations.push(performance.now() - start);

      expect(range.endRow - range.startRow).toBeLessThanOrEqual(221);
    }

    const p99Ms = percentile(durations, 99);
    console.info(`[mvp-12] 100k scroll range P99: ${p99Ms.toFixed(3)}ms`);
    expect(p99Ms).toBeLessThan(16);
  });

  it("estimates 10k commit 10s scroll CPU below a single-core 40% budget", () => {
    const metrics = buildRailRowMetrics([], 10_000, 32);
    const simulatedFrames = 600;
    const start = performance.now();

    for (let frame = 0; frame < simulatedFrames; frame++) {
      const scrollTop = (frame / simulatedFrames) * metrics.totalHeight;
      computeVisibleRangeFromMetrics(scrollTop, 640, metrics, 100);
    }

    const elapsedMs = performance.now() - start;
    const cpuPercentOfSingleCore = (elapsedMs / 10_000) * 100;
    console.info(
      `[mvp-12] 10k/10s virtual scroll CPU estimate: ${cpuPercentOfSingleCore.toFixed(3)}% of one core`,
    );
    expect(cpuPercentOfSingleCore).toBeLessThan(40);
  });

  it("handles 1M commits through metrics + collapse degradation without crashing", () => {
    const metrics = buildRailRowMetrics([], 1_000_000, 32);
    const range = computeVisibleRangeFromMetrics(12_000_000, 720, metrics, 100);
    const strategy = computeRailCollapseStrategy(1_000_000);

    expect(metrics.totalRows).toBe(1_000_000);
    expect(range.endRow - range.startRow).toBeLessThanOrEqual(224);
    console.info(
      `[mvp-12] 1M degradation: rows=${metrics.totalRows}, renderedRange=${range.endRow - range.startRow}, collapse=${strategy.mode}/${strategy.renderedLaneCount}`,
    );
    expect(strategy).toMatchObject({
      mode: "group",
      otherGroupVisible: true,
      renderedLaneCount: 21,
    });
  });
});
