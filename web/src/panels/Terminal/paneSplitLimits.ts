import type { LayoutNode, SplitDir } from "../../bindings";

export const MAX_TERMINAL_PANES = 6;
export const MAX_TERMINAL_PANE_COLUMNS = 3;
export const MAX_TERMINAL_PANE_ROWS = 2;

export type PaneLayoutMetrics = {
  paneCount: number;
  columns: number;
  rows: number;
};

export const measurePaneLayout = (layout: LayoutNode): PaneLayoutMetrics => {
  if (layout.kind === "single") {
    return { paneCount: 1, columns: 1, rows: 1 };
  }

  const first = measurePaneLayout(layout.first);
  const second = measurePaneLayout(layout.second);

  if (layout.direction === "horizontal") {
    return {
      paneCount: first.paneCount + second.paneCount,
      columns: first.columns + second.columns,
      rows: Math.max(first.rows, second.rows),
    };
  }

  return {
    paneCount: first.paneCount + second.paneCount,
    columns: Math.max(first.columns, second.columns),
    rows: first.rows + second.rows,
  };
};

const splitPaneInLayout = (
  layout: LayoutNode,
  paneId: string,
  direction: SplitDir,
): LayoutNode | null => {
  if (layout.kind === "single") {
    if (layout.paneId !== paneId) return null;
    return {
      kind: "split",
      direction,
      ratio: 0.5,
      first: layout,
      second: { kind: "single", paneId: "__new_pane__" },
    };
  }

  const first = splitPaneInLayout(layout.first, paneId, direction);
  if (first) {
    return {
      ...layout,
      first,
    };
  }

  const second = splitPaneInLayout(layout.second, paneId, direction);
  if (second) {
    return {
      ...layout,
      second,
    };
  }

  return null;
};

export const canSplitPaneLayout = (
  layout: LayoutNode,
  paneId: string,
  direction: SplitDir,
): boolean => {
  const nextLayout = splitPaneInLayout(layout, paneId, direction);
  if (!nextLayout) return false;

  const next = measurePaneLayout(nextLayout);
  return (
    next.paneCount <= MAX_TERMINAL_PANES &&
    next.columns <= MAX_TERMINAL_PANE_COLUMNS &&
    next.rows <= MAX_TERMINAL_PANE_ROWS
  );
};
