import { describe, expect, it } from "vitest";
import type {
  RailGraphInputCommit,
  RailLaneAssignment,
} from "../../../../src/panels/GitLog/RailGraph/types";
import {
  collapseRailAssignments,
  collectCollapsedBranchLabels,
  computeRailCollapseStrategy,
  reduceOtherBranchesExpanded,
} from "../../../../src/panels/GitLog/RailGraph/collapse";

function assignments(count: number): RailLaneAssignment[] {
  return Array.from({ length: count }, (_, rowIndex) => ({
    rowIndex,
    laneIndex: rowIndex,
    colorKey: `color-${rowIndex % 30}`,
  }));
}

function commit(
  oid: string,
  refNames: string[] = [],
): RailGraphInputCommit {
  return {
    oid,
    parents: [],
    refKinds: refNames.map(() => "local"),
    refNames,
    isHead: false,
  };
}

describe("computeRailCollapseStrategy", () => {
  it("keeps <=20 lanes fully visible", () => {
    expect(computeRailCollapseStrategy(20)).toMatchObject({
      mode: "full",
      railGap: 16,
      visibleLaneLimit: 20,
      renderedLaneCount: 20,
      otherGroupVisible: false,
    });
  });

  it("compresses 21 lanes to an 8px rail gap with tooltip copy", () => {
    expect(computeRailCollapseStrategy(21)).toMatchObject({
      mode: "compress",
      railGap: 8,
      renderedLaneCount: 21,
      tooltip: "压缩模式 · hover 查看分支名",
    });
  });

  it("keeps 50 compressed lanes before switching to group mode", () => {
    expect(computeRailCollapseStrategy(50)).toMatchObject({
      mode: "compress",
      renderedLaneCount: 50,
      otherGroupVisible: false,
    });
  });

  it("groups lanes above 50 into Other branches", () => {
    expect(computeRailCollapseStrategy(51)).toMatchObject({
      mode: "group",
      visibleLaneLimit: 20,
      otherLaneIndex: 20,
      renderedLaneCount: 21,
      collapsedLaneCount: 31,
      otherGroupVisible: true,
    });
  });
});

describe("collapseRailAssignments", () => {
  it("does not remap full-mode lane assignments", () => {
    const source = assignments(3);

    expect(collapseRailAssignments(source, computeRailCollapseStrategy(3))).toBe(
      source,
    );
  });

  it("remaps lanes over the visible limit into the other group lane", () => {
    const collapsed = collapseRailAssignments(
      assignments(53),
      computeRailCollapseStrategy(53),
    );

    expect(collapsed[19].laneIndex).toBe(19);
    expect(collapsed[20].laneIndex).toBe(20);
    expect(collapsed[52].laneIndex).toBe(20);
  });
});

describe("collectCollapsedBranchLabels", () => {
  it("collects unique branch names from collapsed lanes", () => {
    const input = [
      commit("c0", ["main"]),
      commit("c1", ["feat/a"]),
      commit("c2", ["feat/b"]),
      commit("c3", ["feat/b"]),
    ];
    const lanes = [
      { rowIndex: 0, laneIndex: 0, colorKey: "color-0" },
      { rowIndex: 1, laneIndex: 19, colorKey: "color-1" },
      { rowIndex: 2, laneIndex: 20, colorKey: "color-2" },
      { rowIndex: 3, laneIndex: 21, colorKey: "color-3" },
    ];

    expect(
      collectCollapsedBranchLabels(input, lanes, computeRailCollapseStrategy(60)),
    ).toEqual(["feat/b"]);
  });
});

describe("reduceOtherBranchesExpanded", () => {
  it("toggles group dropdown only while group mode is active", () => {
    expect(
      reduceOtherBranchesExpanded(false, "toggle", computeRailCollapseStrategy(60)),
    ).toBe(true);
    expect(
      reduceOtherBranchesExpanded(true, "toggle", computeRailCollapseStrategy(60)),
    ).toBe(false);
    expect(
      reduceOtherBranchesExpanded(true, "toggle", computeRailCollapseStrategy(30)),
    ).toBe(false);
  });
});
