import type {
  RailEdgeGeo,
  RailGeometryLayout,
  RailNodeGeo,
  RailPathHighlight,
} from "./types-canvas";

export type RailHitTargetKind = "node" | "edge";

export interface RailHitTarget {
  kind: RailHitTargetKind;
  oid: string;
  rowIndex: number;
  laneIndex: number;
  edgeKey?: string;
  distance: number;
}

export type RailPointerHighlightEvent =
  | {
      type: "hover";
      target: { oid: string } | null;
      layout: RailGeometryLayout;
    }
  | { type: "tap"; target: { oid: string } | null; layout: RailGeometryLayout }
  | { type: "leave" };

export function railEdgeKey(edge: RailEdgeGeo): string {
  return `${edge.fromOid}->${edge.toOid}`;
}

function distanceSquared(
  ax: number,
  ay: number,
  bx: number,
  by: number,
): number {
  const dx = ax - bx;
  const dy = ay - by;
  return dx * dx + dy * dy;
}

function distanceToSegment(
  x: number,
  y: number,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const lengthSquared = dx * dx + dy * dy;
  if (lengthSquared === 0) return Math.sqrt(distanceSquared(x, y, x1, y1));

  const t = Math.max(
    0,
    Math.min(1, ((x - x1) * dx + (y - y1) * dy) / lengthSquared),
  );
  const px = x1 + t * dx;
  const py = y1 + t * dy;
  return Math.sqrt(distanceSquared(x, y, px, py));
}

function bezierPoint(edge: RailEdgeGeo, t: number): { x: number; y: number } {
  const cp1x = edge.fromX;
  const cp1y = edge.fromY + edge.controlOffsetY;
  const cp2x = edge.toX;
  const cp2y = edge.toY - edge.controlOffsetY;
  const mt = 1 - t;

  return {
    x:
      mt * mt * mt * edge.fromX +
      3 * mt * mt * t * cp1x +
      3 * mt * t * t * cp2x +
      t * t * t * edge.toX,
    y:
      mt * mt * mt * edge.fromY +
      3 * mt * mt * t * cp1y +
      3 * mt * t * t * cp2y +
      t * t * t * edge.toY,
  };
}

function distanceToEdge(x: number, y: number, edge: RailEdgeGeo): number {
  if (edge.pathKind === "line") {
    return distanceToSegment(x, y, edge.fromX, edge.fromY, edge.toX, edge.toY);
  }

  let minDistance = Number.POSITIVE_INFINITY;
  let previous = bezierPoint(edge, 0);

  for (let step = 1; step <= 16; step++) {
    const next = bezierPoint(edge, step / 16);
    minDistance = Math.min(
      minDistance,
      distanceToSegment(x, y, previous.x, previous.y, next.x, next.y),
    );
    previous = next;
  }

  return minDistance;
}

export function hitTestRailNode(
  x: number,
  y: number,
  nodes: RailNodeGeo[],
  padding = 6,
): RailHitTarget | null {
  let best: RailHitTarget | null = null;

  for (const node of nodes) {
    const distance = Math.sqrt(distanceSquared(x, y, node.x, node.y));
    if (distance > node.radius + padding) continue;
    if (best && best.distance <= distance) continue;
    best = {
      kind: "node",
      oid: node.oid,
      rowIndex: node.rowIndex,
      laneIndex: node.laneIndex,
      distance,
    };
  }

  return best;
}

export function hitTestRailEdge(
  x: number,
  y: number,
  edges: RailEdgeGeo[],
  tolerance = 5,
): RailHitTarget | null {
  let best: RailHitTarget | null = null;

  for (const edge of edges) {
    const distance = distanceToEdge(x, y, edge);
    if (distance > tolerance) continue;
    if (best && best.distance <= distance) continue;
    best = {
      kind: "edge",
      oid: edge.fromOid,
      rowIndex: edge.fromRowIndex,
      laneIndex: edge.fromLaneIndex,
      edgeKey: railEdgeKey(edge),
      distance,
    };
  }

  return best;
}

export function hitTestRailGeometry(
  x: number,
  y: number,
  layout: RailGeometryLayout,
): RailHitTarget | null {
  return (
    hitTestRailNode(x, y, layout.nodes) ?? hitTestRailEdge(x, y, layout.edges)
  );
}

export function collectRailPathHighlight(
  layout: RailGeometryLayout,
  target: { oid: string } | null,
): RailPathHighlight | null {
  if (!target) return null;
  const knownNodes = new Set(layout.nodes.map((node) => node.oid));
  if (!knownNodes.has(target.oid)) return null;

  const visitedNodes = new Set<string>([target.oid]);
  const visitedEdges = new Set<string>();
  const queue = [target.oid];

  while (queue.length > 0) {
    const oid = queue.shift()!;
    for (const edge of layout.edges) {
      if (edge.fromOid !== oid && edge.toOid !== oid) continue;
      visitedEdges.add(railEdgeKey(edge));
      const nextOid = edge.fromOid === oid ? edge.toOid : edge.fromOid;
      if (!knownNodes.has(nextOid) || visitedNodes.has(nextOid)) continue;
      visitedNodes.add(nextOid);
      queue.push(nextOid);
    }
  }

  const rowByOid = new Map(
    layout.nodes.map((node) => [node.oid, node.rowIndex]),
  );
  const nodeOids = Array.from(visitedNodes).sort(
    (a, b) => (rowByOid.get(a) ?? 0) - (rowByOid.get(b) ?? 0),
  );
  const edgeKeys = layout.edges
    .map((edge) => railEdgeKey(edge))
    .filter((key) => visitedEdges.has(key));

  return {
    targetOid: target.oid,
    nodeOids,
    edgeKeys,
  };
}

export function reduceRailPointerHighlight(
  current: RailPathHighlight | null,
  event: RailPointerHighlightEvent,
): RailPathHighlight | null {
  if (event.type === "leave") return null;
  if (!event.target) return event.type === "hover" ? null : current;
  if (event.type === "tap" && current?.targetOid === event.target.oid) {
    return null;
  }
  return collectRailPathHighlight(event.layout, event.target);
}
