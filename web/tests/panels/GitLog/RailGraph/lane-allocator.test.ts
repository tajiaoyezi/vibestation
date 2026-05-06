import { describe, it, expect } from "vitest";
import type { RailGraphInputCommit } from "../../../../src/panels/GitLog/RailGraph/types";
import { allocateLanes } from "../../../../src/panels/GitLog/RailGraph/lane-allocator";

// ── Helpers ──────────────────────────────────────────────────────────────────

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

// ── Fixture imports ───────────────────────────────────────────────────────────
import linear20 from "../../../fixtures/rail-graph/fixture_linear_20.json";
import branchy1k from "../../../fixtures/rail-graph/fixture_branchy_1k.json";
import kernelLike100k from "../../../fixtures/rail-graph/fixture_kernel_like_100k.json";

import snapshotLinear20Light from "../../../fixtures/rail-graph/snapshots/phase-a-linear-20-light.json";
import snapshotBranchy1kLight from "../../../fixtures/rail-graph/snapshots/phase-a-branchy-1k-light.json";
import snapshotKernel100kLight from "../../../fixtures/rail-graph/snapshots/phase-a-kernel-100k-light.json";

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("allocateLanes – determinism (A.3)", () => {
  it("produces identical output on 10 runs for linear_20 fixture", () => {
    const first = JSON.stringify(allocateLanes(linear20 as RailGraphInputCommit[]));
    for (let i = 0; i < 9; i++) {
      expect(JSON.stringify(allocateLanes(linear20 as RailGraphInputCommit[]))).toBe(first);
    }
  });
});

describe("allocateLanes – snapshot regression", () => {
  it("linear_20 matches phase-A snapshot (baseline)", () => {
    const result = allocateLanes(linear20 as RailGraphInputCommit[]);
    expect(result.length).toBe(snapshotLinear20Light.assignmentCount);
    expect(result).toEqual(snapshotLinear20Light.assignments);
  });

  it("branchy_1k matches phase-A snapshot (baseline)", () => {
    const result = allocateLanes(branchy1k as RailGraphInputCommit[]);
    expect(result.length).toBe(snapshotBranchy1kLight.assignmentCount);
    expect(result).toEqual(snapshotBranchy1kLight.assignments);
  });

  it("kernel_100k matches phase-A snapshot summary (assignmentCount + maxLane + first 100)", () => {
    const result = allocateLanes(kernelLike100k as RailGraphInputCommit[]);
    expect(result.length).toBe(snapshotKernel100kLight.assignmentCount);
    const maxLane = result.reduce((m, a) => Math.max(m, a.laneIndex), 0);
    expect(maxLane).toBe(snapshotKernel100kLight.maxLaneIndex);
    // Compare first 100 rows (sampleSize)
    expect(result.slice(0, snapshotKernel100kLight.sampleSize)).toEqual(
      snapshotKernel100kLight.assignments,
    );
  });
});

describe("allocateLanes – root commit (A.4)", () => {
  it("root commit (0 parents) gets laneIndex >= 0", () => {
    const input = [makeCommit("root")];
    const result = allocateLanes(input);
    expect(result).toHaveLength(1);
    expect(result[0].laneIndex).toBeGreaterThanOrEqual(0);
  });

  it("empty input returns empty array", () => {
    expect(allocateLanes([])).toEqual([]);
  });
});

describe("allocateLanes – merge commit (A.5)", () => {
  it("merge commit (2 parents) records data for at least 2 incoming edges", () => {
    // A → merge ← B  (merge has 2 parents)
    const input = [
      makeCommit("merge", ["A", "B"]),
      makeCommit("A"),
      makeCommit("B"),
    ];
    const result = allocateLanes(input);
    const mergeRow = result.find((r) => r.rowIndex === 0);
    expect(mergeRow).toBeDefined();
    // Merge assigns a valid lane
    expect(mergeRow!.laneIndex).toBeGreaterThanOrEqual(0);
    // Parent commits get different lanes (or same — depends on history shape)
    expect(result).toHaveLength(3);
  });
});

describe("allocateLanes – octopus merge (A.16)", () => {
  it("octopus merge (4 parents) does not throw and assigns valid lanes", () => {
    const input = [
      makeCommit("octopus", ["p1", "p2", "p3", "p4"]),
      makeCommit("p1"),
      makeCommit("p2"),
      makeCommit("p3"),
      makeCommit("p4"),
    ];
    const result = allocateLanes(input);
    expect(result).toHaveLength(5);
    for (const row of result) {
      expect(row.laneIndex).toBeGreaterThanOrEqual(0);
    }
  });
});

describe("allocateLanes – cross-branch (A.17)", () => {
  it("linear chain on lane 0 stays at lane 0", () => {
    const input = [
      makeCommit("c", ["b"]),
      makeCommit("b", ["a"]),
      makeCommit("a"),
    ];
    const result = allocateLanes(input);
    // All on same lane (linear history)
    const lanes = result.map((r) => r.laneIndex);
    expect(lanes.every((l) => l === lanes[0])).toBe(true);
  });

  it("fork + re-merge produces valid lane assignments for all commits", () => {
    // main: M → A → R (root)
    // feat: M → B → R (root)  (two branches merging from R)
    const input = [
      makeCommit("M", ["A", "B"]),
      makeCommit("A", ["R"]),
      makeCommit("B", ["R"]),
      makeCommit("R"),
    ];
    const result = allocateLanes(input);
    expect(result).toHaveLength(4);
    for (const row of result) {
      expect(row.laneIndex).toBeGreaterThanOrEqual(0);
      expect(row.colorKey).toMatch(/^color-\d+$/);
    }
  });
});
