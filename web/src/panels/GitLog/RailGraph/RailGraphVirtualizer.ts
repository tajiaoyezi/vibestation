import type { RailGeometryLayout } from "./types-canvas";

export interface RailVisibleRange {
  startRow: number;
  endRow: number;
  firstVisibleRow: number;
  lastVisibleRow: number;
  totalRows: number;
  overscanRows: number;
  startY: number;
  endY: number;
  height: number;
}

export interface RailRowMetrics {
  heights: Float64Array;
  offsets: Float64Array;
  totalRows: number;
  totalHeight: number;
  fallbackHeight: number;
}

function clampInteger(value: number, min: number, max: number): number {
  if (!Number.isFinite(value)) return min;
  return Math.max(min, Math.min(max, Math.floor(value)));
}

function normalizeRowHeight(height: number | undefined, fallback: number) {
  return Number.isFinite(height) && height != null && height >= 0
    ? height
    : fallback;
}

function emptyRange(totalRows = 0): RailVisibleRange {
  return {
    startRow: 0,
    endRow: 0,
    firstVisibleRow: 0,
    lastVisibleRow: 0,
    totalRows,
    overscanRows: 0,
    startY: 0,
    endY: 0,
    height: 0,
  };
}

export function buildRailRowMetrics(
  rowHeights: number[],
  totalRows: number,
  fallbackHeight: number,
): RailRowMetrics {
  const safeTotalRows = Math.max(0, Math.floor(totalRows));
  const safeFallback = Math.max(1, fallbackHeight);
  const heights = new Float64Array(safeTotalRows);
  const offsets = new Float64Array(safeTotalRows + 1);

  for (let rowIndex = 0; rowIndex < safeTotalRows; rowIndex++) {
    const height = normalizeRowHeight(rowHeights[rowIndex], safeFallback);
    heights[rowIndex] = height;
    offsets[rowIndex + 1] = offsets[rowIndex] + height;
  }

  return {
    heights,
    offsets,
    totalRows: safeTotalRows,
    totalHeight: offsets[safeTotalRows] ?? 0,
    fallbackHeight: safeFallback,
  };
}

export function computeVisibleRange(
  scrollTop: number,
  viewportHeight: number,
  rowHeight: number,
  totalRows: number,
  overscanRows = 100,
): RailVisibleRange {
  const safeTotalRows = Math.max(0, Math.floor(totalRows));
  if (safeTotalRows === 0) return emptyRange(0);

  const safeRowHeight = Math.max(1, rowHeight);
  const safeViewportHeight = Math.max(0, viewportHeight);
  const safeScrollTop = Math.max(0, scrollTop);
  const safeOverscan = Math.max(0, Math.floor(overscanRows));
  const firstVisibleRow = clampInteger(
    safeScrollTop / safeRowHeight,
    0,
    safeTotalRows - 1,
  );
  const visibleRows = Math.max(
    1,
    Math.ceil(safeViewportHeight / safeRowHeight),
  );
  const lastVisibleRow = Math.min(
    safeTotalRows - 1,
    firstVisibleRow + visibleRows - 1,
  );
  const startRow = Math.max(0, firstVisibleRow - safeOverscan);
  const endRow = Math.min(safeTotalRows, lastVisibleRow + safeOverscan + 1);
  const startY = startRow * safeRowHeight;
  const endY = endRow * safeRowHeight;

  return {
    startRow,
    endRow,
    firstVisibleRow,
    lastVisibleRow,
    totalRows: safeTotalRows,
    overscanRows: safeOverscan,
    startY,
    endY,
    height: Math.max(0, endY - startY),
  };
}

function firstOffsetGreaterThan(offsets: Float64Array, y: number): number {
  let low = 1;
  let high = offsets.length - 1;
  let result = high;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if ((offsets[mid] ?? 0) > y) {
      result = mid;
      high = mid - 1;
    } else {
      low = mid + 1;
    }
  }

  return result;
}

function firstOffsetAtLeast(offsets: Float64Array, y: number): number {
  let low = 0;
  let high = offsets.length - 1;
  let result = high;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if ((offsets[mid] ?? 0) >= y) {
      result = mid;
      high = mid - 1;
    } else {
      low = mid + 1;
    }
  }

  return result;
}

export function computeVisibleRangeFromMetrics(
  scrollTop: number,
  viewportHeight: number,
  metrics: RailRowMetrics,
  overscanRows = 100,
): RailVisibleRange {
  if (metrics.totalRows === 0) return emptyRange(0);

  const safeScrollTop = Math.max(0, scrollTop);
  const safeViewportHeight = Math.max(0, viewportHeight);
  const safeOverscan = Math.max(0, Math.floor(overscanRows));
  const viewportTop = Math.min(safeScrollTop, metrics.totalHeight);
  const viewportBottom = Math.min(
    metrics.totalHeight,
    viewportTop + safeViewportHeight,
  );
  const firstVisibleRow = Math.min(
    metrics.totalRows - 1,
    Math.max(0, firstOffsetGreaterThan(metrics.offsets, viewportTop) - 1),
  );
  const lastVisibleRow = Math.min(
    metrics.totalRows - 1,
    Math.max(
      firstVisibleRow,
      firstOffsetAtLeast(metrics.offsets, viewportBottom) - 1,
    ),
  );
  const startRow = Math.max(0, firstVisibleRow - safeOverscan);
  const endRow = Math.min(metrics.totalRows, lastVisibleRow + safeOverscan + 1);
  const startY = metrics.offsets[startRow] ?? 0;
  const endY = metrics.offsets[endRow] ?? metrics.totalHeight;

  return {
    startRow,
    endRow,
    firstVisibleRow,
    lastVisibleRow,
    totalRows: metrics.totalRows,
    overscanRows: safeOverscan,
    startY,
    endY,
    height: Math.max(0, endY - startY),
  };
}

export function filterRailGeometryToVisibleRange(
  layout: RailGeometryLayout,
  range: RailVisibleRange,
): RailGeometryLayout {
  if (range.totalRows === 0 || range.endRow <= range.startRow) {
    return { ...layout, nodes: [], edges: [], tips: [] };
  }

  const includesRow = (rowIndex: number) =>
    rowIndex >= range.startRow && rowIndex < range.endRow;
  const intersectsRows = (fromRowIndex: number, toRowIndex: number) =>
    Math.max(fromRowIndex, toRowIndex) >= range.startRow &&
    Math.min(fromRowIndex, toRowIndex) < range.endRow;

  return {
    ...layout,
    height: range.height,
    nodes: layout.nodes
      .filter((node) => includesRow(node.rowIndex))
      .map((node) => ({ ...node, y: node.y - range.startY })),
    edges: layout.edges
      .filter((edge) => intersectsRows(edge.fromRowIndex, edge.toRowIndex))
      .map((edge) => ({
        ...edge,
        fromY: edge.fromY - range.startY,
        toY: edge.toY - range.startY,
      })),
    tips: layout.tips
      .filter((tip) => includesRow(tip.rowIndex))
      .map((tip) => ({ ...tip, y: tip.y - range.startY })),
  };
}
