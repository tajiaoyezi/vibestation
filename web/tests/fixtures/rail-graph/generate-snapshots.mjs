#!/usr/bin/env node
// MVP-12 Phase A · Snapshot generator
// Computes allocateLanes() output for each fixture and saves as JSON baselines.
// Run: node generate-snapshots.mjs

import { readFileSync, writeFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// ── Color mapper (matches color-mapper.ts exactly) ───────────────────────────
const COLOR_RING_SIZE = 30;
function djb2Hash(s) {
  let hash = 5381;
  for (let i = 0; i < s.length; i++) {
    hash = ((hash << 5) + hash) ^ s.charCodeAt(i);
    hash = hash | 0;
  }
  return Math.abs(hash);
}
function branchNameToColorKey(branchName) {
  return `color-${djb2Hash(branchName) % COLOR_RING_SIZE}`;
}

// ── Lane allocator (matches lane-allocator.ts exactly) ───────────────────────
// O(n) via Map<oid,lane> for newest-first input ordering.
function allocateLanes(input) {
  if (input.length === 0) return [];

  const oidToLane = new Map();
  const freeLanes = [];
  let laneCount = 0;
  const result = [];

  function openLane() {
    if (freeLanes.length > 0) return freeLanes.pop();
    return laneCount++;
  }
  function closeLane(lane) {
    freeLanes.push(lane);
  }

  for (let rowIndex = 0; rowIndex < input.length; rowIndex++) {
    const commit = input[rowIndex];

    let laneIndex = oidToLane.get(commit.oid) ?? -1;
    if (laneIndex === -1) {
      laneIndex = openLane();
    }
    oidToLane.delete(commit.oid);

    if (commit.parents.length === 0) {
      closeLane(laneIndex);
    } else {
      const firstParent = commit.parents[0];
      if (!oidToLane.has(firstParent)) {
        oidToLane.set(firstParent, laneIndex);
      } else {
        closeLane(laneIndex);
      }
      for (let p = 1; p < commit.parents.length; p++) {
        const extraParent = commit.parents[p];
        if (!oidToLane.has(extraParent)) {
          oidToLane.set(extraParent, openLane());
        }
      }
    }

    const primaryBranch =
      commit.refNames.find((_n, i) =>
        commit.refKinds[i] === "local" || commit.refKinds[i] === "remote",
      ) ??
      commit.refNames[0] ??
      "main";

    result.push({ rowIndex, laneIndex, colorKey: branchNameToColorKey(primaryBranch) });
  }

  return result;
}

// ── Load fixtures ─────────────────────────────────────────────────────────────
const linear20 = JSON.parse(readFileSync(join(__dirname, "fixture_linear_20.json"), "utf-8"));
const branchy1k = JSON.parse(readFileSync(join(__dirname, "fixture_branchy_1k.json"), "utf-8"));
const kernelLike100k = JSON.parse(readFileSync(join(__dirname, "fixture_kernel_like_100k.json"), "utf-8"));

// ── Compute snapshots ─────────────────────────────────────────────────────────
// For small fixtures: full assignment list.
// For 100k: summary only (first 100 rows + stats) to keep snapshot files < 100KB.
function computeSnapshot(fixture, { summaryOnly = false } = {}) {
  const assignments = allocateLanes(fixture);
  const maxLane = assignments.reduce((m, a) => Math.max(m, a.laneIndex), 0);
  const sample = summaryOnly ? assignments.slice(0, 100) : assignments;
  return {
    assignmentCount: assignments.length,
    maxLaneIndex: maxLane,
    sampleSize: sample.length,
    assignments: sample,
  };
}

const snapshots = {
  "phase-a-linear-20-light": computeSnapshot(linear20),
  "phase-a-linear-20-dark": computeSnapshot(linear20), // same data; theme distinction is Phase B
  "phase-a-branchy-1k-light": computeSnapshot(branchy1k),
  "phase-a-branchy-1k-dark": computeSnapshot(branchy1k),
  "phase-a-kernel-100k-light": computeSnapshot(kernelLike100k, { summaryOnly: true }),
  "phase-a-kernel-100k-dark": computeSnapshot(kernelLike100k, { summaryOnly: true }),
};

for (const [name, snapshot] of Object.entries(snapshots)) {
  const path = join(__dirname, "snapshots", `${name}.json`);
  writeFileSync(path, JSON.stringify(snapshot, null, 2));
  console.log(`Generated ${name}.json (${snapshot.assignmentCount} rows, maxLane=${snapshot.maxLaneIndex})`);
}
