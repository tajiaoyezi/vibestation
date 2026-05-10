import type { RailGraphInputCommit, RailLaneAssignment } from "./types";

export type RailCollapseMode = "full" | "compress" | "group";

export interface RailCollapseStrategy {
  mode: RailCollapseMode;
  railGap: number;
  visibleLaneLimit: number;
  renderedLaneCount: number;
  collapsedLaneCount: number;
  otherLaneIndex: number | null;
  otherGroupVisible: boolean;
  tooltip: string | null;
}

const FULL_LANE_LIMIT = 20;
const COMPRESS_LANE_LIMIT = 50;
const FULL_RAIL_GAP = 16;
const COMPRESSED_RAIL_GAP = 8;
const COMPRESS_TOOLTIP = "压缩模式 · hover 查看分支名";

export function computeRailCollapseStrategy(
  laneCount: number,
): RailCollapseStrategy {
  const safeLaneCount = Math.max(0, Math.floor(laneCount));

  if (safeLaneCount <= FULL_LANE_LIMIT) {
    return {
      mode: "full",
      railGap: FULL_RAIL_GAP,
      visibleLaneLimit: FULL_LANE_LIMIT,
      renderedLaneCount: safeLaneCount,
      collapsedLaneCount: 0,
      otherLaneIndex: null,
      otherGroupVisible: false,
      tooltip: null,
    };
  }

  if (safeLaneCount <= COMPRESS_LANE_LIMIT) {
    return {
      mode: "compress",
      railGap: COMPRESSED_RAIL_GAP,
      visibleLaneLimit: safeLaneCount,
      renderedLaneCount: safeLaneCount,
      collapsedLaneCount: 0,
      otherLaneIndex: null,
      otherGroupVisible: false,
      tooltip: COMPRESS_TOOLTIP,
    };
  }

  return {
    mode: "group",
    railGap: COMPRESSED_RAIL_GAP,
    visibleLaneLimit: FULL_LANE_LIMIT,
    renderedLaneCount: FULL_LANE_LIMIT + 1,
    collapsedLaneCount: safeLaneCount - FULL_LANE_LIMIT,
    otherLaneIndex: FULL_LANE_LIMIT,
    otherGroupVisible: true,
    tooltip: COMPRESS_TOOLTIP,
  };
}

export function collapseRailAssignments(
  assignments: RailLaneAssignment[],
  strategy: RailCollapseStrategy,
): RailLaneAssignment[] {
  if (strategy.mode !== "group" || strategy.otherLaneIndex == null) {
    return assignments;
  }

  const otherLaneIndex = strategy.otherLaneIndex;
  return assignments.map((assignment) =>
    assignment.laneIndex < strategy.visibleLaneLimit
      ? assignment
      : { ...assignment, laneIndex: otherLaneIndex },
  );
}

export function collectCollapsedBranchLabels(
  input: RailGraphInputCommit[],
  assignments: RailLaneAssignment[],
  strategy: RailCollapseStrategy,
): string[] {
  if (strategy.mode !== "group") return [];

  const labels = new Set<string>();
  for (const assignment of assignments) {
    if (assignment.laneIndex < strategy.visibleLaneLimit) continue;
    const commit = input[assignment.rowIndex];
    if (!commit) continue;

    for (const label of commit.refNames) {
      if (label) labels.add(label);
    }
  }

  return Array.from(labels);
}

export function reduceOtherBranchesExpanded(
  current: boolean,
  action: "toggle" | "close",
  strategy: RailCollapseStrategy,
): boolean {
  if (strategy.mode !== "group") return false;
  if (action === "close") return false;
  return !current;
}
