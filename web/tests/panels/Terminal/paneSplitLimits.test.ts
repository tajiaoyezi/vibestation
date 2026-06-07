import { describe, expect, it } from "vitest";
import type { LayoutNode, SplitDir } from "../../../src/bindings";
import {
  canSplitPaneLayout,
  MAX_TERMINAL_PANE_COLUMNS,
  MAX_TERMINAL_PANE_ROWS,
  MAX_TERMINAL_PANES,
  measurePaneLayout,
} from "../../../src/panels/Terminal/paneSplitLimits";

const single = (paneId: string): LayoutNode => ({ kind: "single", paneId });

const split = (
  direction: SplitDir,
  first: LayoutNode,
  second: LayoutNode,
): LayoutNode => ({
  kind: "split",
  direction,
  ratio: 0.5,
  first,
  second,
});

describe("pane split limits", () => {
  it("measures pane count and grid dimensions from the layout tree", () => {
    const layout = split(
      "horizontal",
      split("vertical", single("pane-a"), single("pane-d")),
      split(
        "horizontal",
        split("vertical", single("pane-b"), single("pane-e")),
        split("vertical", single("pane-c"), single("pane-f")),
      ),
    );

    expect(measurePaneLayout(layout)).toEqual({
      paneCount: MAX_TERMINAL_PANES,
      columns: MAX_TERMINAL_PANE_COLUMNS,
      rows: 2,
    });
  });

  it("blocks any split that would exceed the six pane total", () => {
    const layout = split(
      "horizontal",
      split("vertical", single("pane-a"), single("pane-d")),
      split(
        "horizontal",
        split("vertical", single("pane-b"), single("pane-e")),
        split("vertical", single("pane-c"), single("pane-f")),
      ),
    );

    expect(canSplitPaneLayout(layout, "pane-a", "horizontal")).toBe(false);
    expect(canSplitPaneLayout(layout, "pane-a", "vertical")).toBe(false);
  });

  it("allows a down split at three columns while total pane count stays below six", () => {
    const layout = split(
      "horizontal",
      single("pane-a"),
      split("horizontal", single("pane-b"), single("pane-c")),
    );

    expect(measurePaneLayout(layout)).toEqual({
      paneCount: 3,
      columns: MAX_TERMINAL_PANE_COLUMNS,
      rows: 1,
    });
    expect(canSplitPaneLayout(layout, "pane-a", "vertical")).toBe(true);
    expect(canSplitPaneLayout(layout, "pane-a", "horizontal")).toBe(false);
  });

  it("blocks a second down split once the layout already has two rows", () => {
    const layout = split("vertical", single("pane-a"), single("pane-b"));

    expect(measurePaneLayout(layout)).toEqual({
      paneCount: 2,
      columns: 1,
      rows: MAX_TERMINAL_PANE_ROWS,
    });
    expect(canSplitPaneLayout(layout, "pane-a", "horizontal")).toBe(true);
    expect(canSplitPaneLayout(layout, "pane-a", "vertical")).toBe(false);
  });
});
