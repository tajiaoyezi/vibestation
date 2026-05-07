// MVP-12 Phase A · Rail graph data layer types
// Phase A: data contract only · no Canvas · no GUI

/** Commit node as consumed by the rail graph algorithm */
export interface RailGraphInputCommit {
  /** Commit identifier (shortSha in Phase A · full OID in Phase D after GitLogEntry extension) */
  oid: string;
  /** Parent OIDs. length === 0 = root · length >= 2 = merge */
  parents: string[];
  /** Ref kinds present on this commit */
  refKinds: ("local" | "remote" | "tag")[];
  /** Ref display names (branch names, tag names) */
  refNames: string[];
  /** Whether this commit is the current HEAD */
  isHead: boolean;
}

/** Lane assignment output for one commit row */
export interface RailLaneAssignment {
  /** Row index (0-based, same order as input commits) */
  rowIndex: number;
  /** Lane column index (0-based) */
  laneIndex: number;
  /** Stable color key derived from branch name hash (30-color ring) */
  colorKey: string;
}

/** Feature flag: phase A declares only · default false · enabled in Phase D */
export const ENABLE_RAIL_GRAPH = false;
