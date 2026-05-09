import type { RailGraphInputCommit, RailLaneAssignment } from "./types";
import type {
  RailEdgeGeo,
  RailGeometryLayout,
  RailNodeGeo,
  RailNodeKind,
  RailTipGeo,
  RailTipKind,
  RailViewOptions,
} from "./types-canvas";
import { DEFAULT_RAIL_VIEW_OPTIONS } from "./types-canvas";

function normalizeRowHeight(
  rowHeights: number[],
  rowIndex: number,
  fallback: number,
): number {
  const measured = rowHeights[rowIndex];
  return Number.isFinite(measured) && measured >= 0 ? measured : fallback;
}

function buildRowCenters(
  inputLength: number,
  rowHeights: number[],
  fallback: number,
): { centers: number[]; heights: number[]; totalHeight: number } {
  const centers: number[] = [];
  const heights: number[] = [];
  let cursorY = 0;

  for (let rowIndex = 0; rowIndex < inputLength; rowIndex++) {
    const height = normalizeRowHeight(rowHeights, rowIndex, fallback);
    heights.push(height);
    centers.push(cursorY + height / 2);
    cursorY += height;
  }

  return { centers, heights, totalHeight: cursorY };
}

function buildChildrenCount(
  input: RailGraphInputCommit[],
): Map<string, number> {
  const knownOids = new Set(input.map((commit) => commit.oid));
  const childrenCount = new Map<string, number>();

  for (const commit of input) {
    for (const parentOid of commit.parents) {
      if (!knownOids.has(parentOid)) continue;
      childrenCount.set(parentOid, (childrenCount.get(parentOid) ?? 0) + 1);
    }
  }

  return childrenCount;
}

function nodeKind(
  commit: RailGraphInputCommit,
  childCount: number,
): RailNodeKind {
  if (commit.isHead) return "head";
  if (commit.parents.length >= 2) return "merge";
  if (childCount >= 2) return "fork";
  return "normal";
}

function nodeRadius(kind: RailNodeKind): number {
  if (kind === "head") return 8;
  if (kind === "merge" || kind === "fork") return 7;
  return 6;
}

function tipKind(kind: RailGraphInputCommit["refKinds"][number]): RailTipKind {
  return kind === "tag" ? "tag" : kind === "remote" ? "remote" : "local";
}

function estimateTipWidth(
  label: string,
  paddingX: number,
  maxWidth: number,
): number {
  return Math.min(maxWidth, Math.max(28, label.length * 6 + paddingX * 2));
}

export function computeRailGeometry(
  input: RailGraphInputCommit[],
  assignments: RailLaneAssignment[],
  rowHeights: number[],
  options: RailViewOptions = {},
): RailGeometryLayout {
  const viewOptions = { ...DEFAULT_RAIL_VIEW_OPTIONS, ...options };
  const { centers, heights, totalHeight } = buildRowCenters(
    input.length,
    rowHeights,
    viewOptions.rowFallbackHeight,
  );
  const assignmentByRow = new Map(
    assignments.map((assignment) => [assignment.rowIndex, assignment]),
  );
  const rowByOid = new Map(
    input.map((commit, rowIndex) => [commit.oid, rowIndex]),
  );
  const childrenCount = buildChildrenCount(input);
  const nodes: RailNodeGeo[] = input.map((commit, rowIndex) => {
    const assignment = assignmentByRow.get(rowIndex);
    const laneIndex = assignment?.laneIndex ?? 0;
    const childCount = childrenCount.get(commit.oid) ?? 0;
    const kind = nodeKind(commit, childCount);

    return {
      oid: commit.oid,
      rowIndex,
      laneIndex,
      colorKey: assignment?.colorKey ?? "color-0",
      x: viewOptions.lanePaddingX + laneIndex * viewOptions.laneGap,
      y: centers[rowIndex] ?? 0,
      kind,
      radius: nodeRadius(kind),
      ringWidth: kind === "head" ? 2 : 0,
      parentCount: commit.parents.length,
      childCount,
    };
  });
  const nodeByRow = new Map(nodes.map((node) => [node.rowIndex, node]));
  const edges: RailEdgeGeo[] = [];

  for (const commit of input) {
    const fromRowIndex = rowByOid.get(commit.oid);
    if (fromRowIndex == null) continue;
    const fromNode = nodeByRow.get(fromRowIndex);
    if (!fromNode) continue;

    for (const parentOid of commit.parents) {
      const toRowIndex = rowByOid.get(parentOid);
      if (toRowIndex == null) continue;
      const toNode = nodeByRow.get(toRowIndex);
      if (!toNode) continue;
      const maxRowHeight = Math.max(
        heights[fromRowIndex] ?? viewOptions.rowFallbackHeight,
        heights[toRowIndex] ?? viewOptions.rowFallbackHeight,
      );

      edges.push({
        fromOid: commit.oid,
        toOid: parentOid,
        fromRowIndex,
        toRowIndex,
        fromLaneIndex: fromNode.laneIndex,
        toLaneIndex: toNode.laneIndex,
        colorKey: fromNode.colorKey,
        fromX: fromNode.x,
        fromY: fromNode.y,
        toX: toNode.x,
        toY: toNode.y,
        pathKind: fromNode.laneIndex === toNode.laneIndex ? "line" : "bezier",
        controlOffsetY: maxRowHeight * 0.5,
      });
    }
  }

  const tips: RailTipGeo[] = [];
  for (const commit of input) {
    const rowIndex = rowByOid.get(commit.oid);
    if (rowIndex == null) continue;
    const node = nodeByRow.get(rowIndex);
    if (!node) continue;
    let nextTipX = viewOptions.tipStartX;

    for (let refIndex = 0; refIndex < commit.refNames.length; refIndex++) {
      const label = commit.refNames[refIndex];
      if (!label) continue;
      const kind = tipKind(commit.refKinds[refIndex] ?? "local");
      const width = estimateTipWidth(
        label,
        viewOptions.tipPaddingX,
        viewOptions.maxTipWidth,
      );

      tips.push({
        oid: commit.oid,
        rowIndex,
        colorKey: node.colorKey,
        kind,
        label,
        x: nextTipX,
        y: node.y,
        width,
        height: viewOptions.tipHeight,
        radius: 4,
      });
      nextTipX += width + viewOptions.tipGap;
    }
  }

  const laneCount =
    assignments.reduce(
      (maxLane, assignment) => Math.max(maxLane, assignment.laneIndex),
      -1,
    ) + 1;
  const computedWidth =
    viewOptions.width ??
    Math.max(
      viewOptions.tipStartX + viewOptions.maxTipWidth,
      viewOptions.lanePaddingX * 2 +
        Math.max(0, laneCount - 1) * viewOptions.laneGap,
    );

  return {
    width: computedWidth,
    height: totalHeight,
    laneCount: Math.max(0, laneCount),
    nodes,
    edges,
    tips,
  };
}
