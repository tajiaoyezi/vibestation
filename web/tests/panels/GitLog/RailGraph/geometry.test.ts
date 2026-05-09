import { describe, expect, it } from "vitest";
import type { RailGraphInputCommit } from "../../../../src/panels/GitLog/RailGraph/types";
import { allocateLanes } from "../../../../src/panels/GitLog/RailGraph/lane-allocator";
import { computeRailGeometry } from "../../../../src/panels/GitLog/RailGraph/geometry";

function makeCommit(
  oid: string,
  parents: string[] = [],
  opts: Partial<RailGraphInputCommit> = {},
): RailGraphInputCommit {
  return {
    oid,
    parents,
    refKinds: [],
    refNames: [],
    isHead: false,
    ...opts,
  };
}

describe("computeRailGeometry", () => {
  it("aligns node y coordinates to measured row centers", () => {
    const input = [
      makeCommit("c", ["b"]),
      makeCommit("b", ["a"]),
      makeCommit("a"),
    ];
    const layout = computeRailGeometry(
      input,
      allocateLanes(input),
      [20, 40, 20],
    );

    expect(layout.nodes.map((node) => node.y)).toEqual([10, 40, 70]);
    expect(new Set(layout.nodes.map((node) => node.x)).size).toBe(1);
  });

  it("places a single root row at y=0 when the measured row height is zero", () => {
    const input = [makeCommit("root")];
    const layout = computeRailGeometry(input, allocateLanes(input), [0]);

    expect(layout.nodes[0]).toMatchObject({
      oid: "root",
      rowIndex: 0,
      y: 0,
      kind: "normal",
    });
  });

  it("creates one edge per merge parent with endpoints on the parent nodes", () => {
    const input = [
      makeCommit("merge", ["main-parent", "side-parent"]),
      makeCommit("main-parent"),
      makeCommit("side-parent"),
    ];
    const layout = computeRailGeometry(
      input,
      allocateLanes(input),
      [24, 24, 24],
    );

    const merge = layout.nodes.find((node) => node.oid === "merge");
    const mainParent = layout.nodes.find((node) => node.oid === "main-parent");
    const sideParent = layout.nodes.find((node) => node.oid === "side-parent");

    expect(merge?.kind).toBe("merge");
    expect(layout.edges).toHaveLength(2);
    expect(layout.edges).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          fromOid: "merge",
          toOid: "main-parent",
          fromX: merge?.x,
          fromY: merge?.y,
          toX: mainParent?.x,
          toY: mainParent?.y,
        }),
        expect.objectContaining({
          fromOid: "merge",
          toOid: "side-parent",
          fromX: merge?.x,
          fromY: merge?.y,
          toX: sideParent?.x,
          toY: sideParent?.y,
          pathKind: "bezier",
        }),
      ]),
    );
  });

  it("marks commits with multiple children as fork nodes", () => {
    const input = [
      makeCommit("child-a", ["root"]),
      makeCommit("child-b", ["root"]),
      makeCommit("root"),
    ];
    const layout = computeRailGeometry(
      input,
      allocateLanes(input),
      [20, 20, 20],
    );

    expect(layout.nodes.find((node) => node.oid === "root")).toMatchObject({
      kind: "fork",
      childCount: 2,
    });
  });

  it("lets current HEAD styling take precedence over normal node styling", () => {
    const input = [
      makeCommit("head", ["parent"], {
        isHead: true,
      }),
      makeCommit("parent"),
    ];
    const layout = computeRailGeometry(input, allocateLanes(input), [22, 22]);

    expect(layout.nodes[0]).toMatchObject({
      kind: "head",
      radius: 8,
      ringWidth: 2,
    });
  });

  it("builds local, remote, and tag tip geometry from ref metadata", () => {
    const input = [
      makeCommit("tip", [], {
        refKinds: ["local", "remote", "tag"],
        refNames: ["main", "origin/main", "v0.1.0"],
      }),
    ];
    const layout = computeRailGeometry(input, allocateLanes(input), [24], {
      tipStartX: 72,
    });

    expect(layout.tips.map((tip) => tip.kind)).toEqual([
      "local",
      "remote",
      "tag",
    ]);
    expect(layout.tips[0]).toMatchObject({ label: "main", x: 72 });
    expect(layout.tips[1].x).toBeGreaterThan(layout.tips[0].x);
    expect(layout.tips[2].radius).toBe(4);
  });
});
