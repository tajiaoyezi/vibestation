// MVP-12 Phase A · Lane allocator (FIFO placeholder)
//
// SPIKE-09 PENDING: This is a Phase A minimal FIFO implementation.
// After SPIKE-09 benchmarks three candidate algorithms (Coffman-Graham / Sugiyama / custom),
// the winner will replace this placeholder. Phase A acceptance does NOT require an optimal
// algorithm — only determinism (same input → same output) and correct edge cases.
//
// Algorithm: "open lanes frontier" for NEWEST-FIRST input ordering (git log order).
// Uses a Map<oid, laneIndex> for O(1) parent lookups and a free-lane pool for O(1) allocation.
// Input ordering: newest commit = row 0, oldest = last row.

import type { RailGraphInputCommit, RailLaneAssignment } from "./types";
import { branchNameToColorKey } from "./color-mapper";

/**
 * Assign a lane index to each input commit.
 * Input must be in newest-first order (matches MVP-07 git log sort order).
 * Deterministic: same input array (by value) always produces identical output.
 * Does NOT mutate the input array.
 */
export function allocateLanes(
  input: RailGraphInputCommit[],
): RailLaneAssignment[] {
  if (input.length === 0) return [];

  // Map from OID → lane index: "this OID is expected on this lane"
  const oidToLane = new Map<string, number>();
  // Set of free lane indices available for reuse
  const freeLanes: number[] = [];
  // High-water mark for lane count
  let laneCount = 0;

  const result: RailLaneAssignment[] = [];

  function openLane(): number {
    if (freeLanes.length > 0) return freeLanes.pop()!;
    return laneCount++;
  }

  function closeLane(lane: number): void {
    freeLanes.push(lane);
  }

  for (let rowIndex = 0; rowIndex < input.length; rowIndex++) {
    const commit = input[rowIndex];

    // 1. Find this commit's lane (was it expected by a previously processed child?)
    let laneIndex = oidToLane.get(commit.oid) ?? -1;
    if (laneIndex === -1) {
      // Not in frontier: root, orphan, or first commit — open new lane
      laneIndex = openLane();
    }
    oidToLane.delete(commit.oid);

    // 2. Route parents into lanes
    if (commit.parents.length === 0) {
      // Root commit: close lane
      closeLane(laneIndex);
    } else {
      // First parent inherits this lane
      const firstParent = commit.parents[0];
      if (!oidToLane.has(firstParent)) {
        oidToLane.set(firstParent, laneIndex);
      } else {
        // First parent already claimed by another child: close this lane
        closeLane(laneIndex);
      }

      // Additional parents (merge commit): open new lanes
      for (let p = 1; p < commit.parents.length; p++) {
        const extraParent = commit.parents[p];
        if (!oidToLane.has(extraParent)) {
          oidToLane.set(extraParent, openLane());
        }
        // else: already claimed — skip (convergence handled by whichever child arrived first)
      }
    }

    // 3. Determine color key from first local/remote branch ref, fallback "main"
    const primaryBranch =
      commit.refNames.find(
        (_n, i) =>
          commit.refKinds[i] === "local" || commit.refKinds[i] === "remote",
      ) ??
      commit.refNames[0] ??
      "main";

    result.push({
      rowIndex,
      laneIndex,
      colorKey: branchNameToColorKey(primaryBranch),
    });
  }

  return result;
}
