import { describe, expect, it } from "vitest";
import type {
  RailGeometryLayout,
  RailNodeGeo,
} from "../../../../src/panels/GitLog/RailGraph/types-canvas";
import {
  buildRailRowMetrics,
  computeVisibleRange,
  computeVisibleRangeFromMetrics,
  filterRailGeometryToVisibleRange,
} from "../../../../src/panels/GitLog/RailGraph/RailGraphVirtualizer";

function makeNode(rowIndex: number): RailNodeGeo {
  return {
    oid: `c${rowIndex}`,
    rowIndex,
    laneIndex: 0,
    colorKey: "color-0",
    x: 16,
    y: rowIndex * 20 + 10,
    kind: "normal",
    radius: 6,
    ringWidth: 0,
    parentCount: 1,
    childCount: 1,
  };
}

function makeLayout(rows: number): RailGeometryLayout {
  return {
    width: 140,
    height: rows * 20,
    laneCount: 1,
    nodes: Array.from({ length: rows }, (_, row) => makeNode(row)),
    edges: Array.from({ length: Math.max(0, rows - 1) }, (_, row) => ({
      fromOid: `c${row}`,
      toOid: `c${row + 1}`,
      fromRowIndex: row,
      toRowIndex: row + 1,
      fromLaneIndex: 0,
      toLaneIndex: 0,
      colorKey: "color-0",
      fromX: 16,
      fromY: row * 20 + 10,
      toX: 16,
      toY: (row + 1) * 20 + 10,
      pathKind: "line",
      controlOffsetY: 10,
    })),
    tips: [],
  };
}

describe("computeVisibleRange", () => {
  it("returns an empty range for empty input", () => {
    expect(computeVisibleRange(0, 400, 20, 0)).toMatchObject({
      startRow: 0,
      endRow: 0,
      totalRows: 0,
    });
  });

  it("clamps negative scrollTop to the first row with overscan", () => {
    expect(computeVisibleRange(-200, 100, 20, 500, 2)).toMatchObject({
      firstVisibleRow: 0,
      lastVisibleRow: 4,
      startRow: 0,
      endRow: 7,
    });
  });

  it("keeps the end range exclusive and capped at total rows", () => {
    expect(computeVisibleRange(9_820, 200, 20, 500, 100)).toMatchObject({
      firstVisibleRow: 491,
      lastVisibleRow: 499,
      startRow: 391,
      endRow: 500,
    });
  });

  it("handles total rows smaller than overscan", () => {
    expect(computeVisibleRange(40, 120, 20, 12, 100)).toMatchObject({
      startRow: 0,
      endRow: 12,
    });
  });

  it("handles fractional row heights without dropping visible rows", () => {
    expect(computeVisibleRange(45, 40, 17.5, 20, 1)).toMatchObject({
      firstVisibleRow: 2,
      lastVisibleRow: 4,
      startRow: 1,
      endRow: 6,
    });
  });
});

describe("computeVisibleRangeFromMetrics", () => {
  it("uses measured row offsets for variable-height commit rows", () => {
    const metrics = buildRailRowMetrics([10, 30, 50, 20], 4, 44);

    expect(metrics.totalHeight).toBe(110);
    expect(computeVisibleRangeFromMetrics(35, 20, metrics, 1)).toMatchObject({
      firstVisibleRow: 1,
      lastVisibleRow: 2,
      startRow: 0,
      endRow: 4,
      startY: 0,
      endY: 110,
    });
  });

  it("falls back to a stable row height for missing measurements", () => {
    const metrics = buildRailRowMetrics([12], 3, 40);

    expect(Array.from(metrics.heights)).toEqual([12, 40, 40]);
    expect(metrics.totalHeight).toBe(92);
  });
});

describe("filterRailGeometryToVisibleRange", () => {
  it("removes nodes outside viewport overscan before paint", () => {
    const filtered = filterRailGeometryToVisibleRange(makeLayout(20), {
      startRow: 5,
      endRow: 9,
      firstVisibleRow: 5,
      lastVisibleRow: 8,
      totalRows: 20,
      overscanRows: 0,
      startY: 100,
      endY: 180,
      height: 80,
    });

    expect(filtered.nodes.map((node) => node.rowIndex)).toEqual([5, 6, 7, 8]);
    expect(filtered.edges.every((edge) => edge.fromRowIndex < 9)).toBe(true);
    expect(filtered.edges.every((edge) => edge.toRowIndex >= 5)).toBe(true);
  });
});
