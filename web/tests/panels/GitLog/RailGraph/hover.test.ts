import { describe, expect, it } from "vitest";
import type {
  RailEdgeGeo,
  RailGeometryLayout,
  RailNodeGeo,
} from "../../../../src/panels/GitLog/RailGraph/types-canvas";
import {
  collectRailPathHighlight,
  hitTestRailEdge,
  hitTestRailGeometry,
  hitTestRailNode,
  reduceRailPointerHighlight,
} from "../../../../src/panels/GitLog/RailGraph/interactions";

function node(oid: string, x: number, y: number): RailNodeGeo {
  return {
    oid,
    rowIndex: Number(oid.slice(1)),
    laneIndex: 0,
    colorKey: "color-0",
    x,
    y,
    kind: "normal",
    radius: 6,
    ringWidth: 0,
    parentCount: 1,
    childCount: 1,
  };
}

function edge(from: RailNodeGeo, to: RailNodeGeo): RailEdgeGeo {
  return {
    fromOid: from.oid,
    toOid: to.oid,
    fromRowIndex: from.rowIndex,
    toRowIndex: to.rowIndex,
    fromLaneIndex: from.laneIndex,
    toLaneIndex: to.laneIndex,
    colorKey: from.colorKey,
    fromX: from.x,
    fromY: from.y,
    toX: to.x,
    toY: to.y,
    pathKind: from.x === to.x ? "line" : "bezier",
    controlOffsetY: 16,
  };
}

function layout(): RailGeometryLayout {
  const nodes = [node("c0", 16, 10), node("c1", 16, 38), node("c2", 44, 66)];
  return {
    width: 140,
    height: 96,
    laneCount: 2,
    nodes,
    edges: [edge(nodes[0], nodes[1]), edge(nodes[1], nodes[2])],
    tips: [],
  };
}

describe("rail hover hit testing", () => {
  it("hits the nearest node within radius plus padding", () => {
    expect(hitTestRailNode(18, 12, layout().nodes)).toMatchObject({
      kind: "node",
      oid: "c0",
      rowIndex: 0,
    });
  });

  it("misses nodes outside the padded radius", () => {
    expect(hitTestRailNode(80, 12, layout().nodes)).toBeNull();
  });

  it("hits straight edges within tolerance", () => {
    expect(hitTestRailEdge(18, 25, layout().edges, 4)).toMatchObject({
      kind: "edge",
      oid: "c0",
      edgeKey: "c0->c1",
    });
  });

  it("hits bezier edges by sampling the curve", () => {
    expect(hitTestRailEdge(28, 50, layout().edges, 10)).toMatchObject({
      kind: "edge",
      edgeKey: "c1->c2",
    });
  });

  it("gives nodes precedence when a node overlaps an edge", () => {
    expect(hitTestRailGeometry(16, 38, layout())).toMatchObject({
      kind: "node",
      oid: "c1",
    });
  });

  it("collects the connected rail path for a hover target", () => {
    const highlight = collectRailPathHighlight(layout(), { oid: "c1" });

    expect(highlight?.nodeOids).toEqual(["c0", "c1", "c2"]);
    expect(highlight?.edgeKeys).toEqual(["c0->c1", "c1->c2"]);
  });
});

describe("reduceRailPointerHighlight", () => {
  it("clears mouse hover on pointer leave", () => {
    const current = collectRailPathHighlight(layout(), { oid: "c1" });

    expect(reduceRailPointerHighlight(current, { type: "leave" })).toBeNull();
  });

  it("toggles touch tap highlight on the second tap", () => {
    const current = collectRailPathHighlight(layout(), { oid: "c1" });

    expect(
      reduceRailPointerHighlight(current, {
        type: "tap",
        target: { oid: "c1" },
        layout: layout(),
      }),
    ).toBeNull();
  });

  it("sets touch tap highlight for a new target", () => {
    expect(
      reduceRailPointerHighlight(null, {
        type: "tap",
        target: { oid: "c2" },
        layout: layout(),
      }),
    ).toMatchObject({ targetOid: "c2" });
  });
});
