#!/usr/bin/env node
// MVP-12 Phase A · Fixture generator
// Generates deterministic RailGraphInputCommit[] fixtures for testing.
// Run: node generate-fixtures.mjs
// Output: fixture_linear_20.json, fixture_branchy_1k.json, fixture_kernel_like_100k.json

import { writeFileSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Simple deterministic pseudo-RNG (xorshift32) for reproducible fixtures
function makeRng(seed) {
  let s = seed >>> 0;
  return () => {
    s ^= s << 13;
    s ^= s >> 17;
    s ^= s << 5;
    s = s >>> 0;
    return s / 0xffffffff;
  };
}

function sha(prefix, index) {
  return `${prefix}${String(index).padStart(7, "0")}`;
}

// ── Linear 20 fixture ────────────────────────────────────────────────────────
// Newest-first: index 0 = HEAD ("a0000000"), index 19 = root ("a0000019")
function generateLinear20() {
  const commits = [];
  for (let i = 0; i <= 19; i++) {
    const oid = sha("a", i);
    const parents = i < 19 ? [sha("a", i + 1)] : [];
    const isHead = i === 0;
    const refKinds = isHead ? ["local"] : [];
    const refNames = isHead ? ["main"] : [];
    commits.push({ oid, parents, refKinds, refNames, isHead });
  }
  return commits;
}

// ── Branchy 1k fixture ───────────────────────────────────────────────────────
// 1000 commits, 20 branches (local 12 / remote 6 / tag 2), 30 merges (1 octopus)
function generateBranchy1k() {
  const rng = makeRng(42);
  const commits = [];
  const total = 1000;

  // Build a simple branching history:
  // Commits are ordered newest-first (index 0 = HEAD).
  // We create 20 branch tips and weave them together with merges.

  const BRANCHES = [
    // local (12)
    "main", "feat/alpha", "feat/beta", "feat/gamma", "feat/delta",
    "fix/issue-1", "fix/issue-2", "fix/issue-3",
    "refactor/core", "refactor/ui", "docs/readme", "chore/deps",
    // remote (6)
    "origin/main", "origin/feat/alpha", "origin/feat/beta",
    "origin/fix/issue-1", "origin/release-1.0", "origin/hotfix",
    // tag (2)
    "v0.1.0", "v0.2.0",
  ];

  const BRANCH_KIND = [
    "local","local","local","local","local","local","local","local","local","local","local","local",
    "remote","remote","remote","remote","remote","remote",
    "tag","tag",
  ];

  // Generate commit graph
  // Strategy: linear backbone of 930 commits, then 30 merge commits,
  // plus a few forks from branch points
  const backbone = [];
  for (let i = 0; i < 940; i++) {
    const oid = sha("b", i);
    backbone.push(oid);
  }

  // Forks: 5 side branches off backbone
  const sideBranches = [[], [], [], [], []];
  const forkPoints = [200, 350, 500, 650, 800];
  for (let b = 0; b < 5; b++) {
    const forkFrom = backbone[forkPoints[b]];
    for (let j = 0; j < 10; j++) {
      sideBranches[b].push(sha(`s${b}`, j));
    }
  }

  // Merge commit OIDs (one per side branch = 5 merges from side branches,
  // plus 24 more merges from backbone segments, plus 1 octopus)
  const mergeOids = [];
  for (let m = 0; m < 30; m++) {
    mergeOids.push(sha("m", m));
  }

  // Build commit list newest-first
  const allCommits = new Map();

  // Backbone (linear, oldest = last)
  for (let i = 0; i < backbone.length - 1; i++) {
    allCommits.set(backbone[i], {
      oid: backbone[i],
      parents: [backbone[i + 1]],
      refKinds: [],
      refNames: [],
      isHead: false,
    });
  }
  // Oldest backbone commit = root
  allCommits.set(backbone[backbone.length - 1], {
    oid: backbone[backbone.length - 1],
    parents: [],
    refKinds: [],
    refNames: [],
    isHead: false,
  });

  // Side branch commits
  for (let b = 0; b < 5; b++) {
    const side = sideBranches[b];
    for (let j = 0; j < side.length; j++) {
      const parent = j === side.length - 1 ? backbone[forkPoints[b]] : side[j + 1];
      allCommits.set(side[j], {
        oid: side[j],
        parents: [parent],
        refKinds: [],
        refNames: [],
        isHead: false,
      });
    }
  }

  // Merge commits: each merges side branch into backbone
  for (let m = 0; m < 5; m++) {
    const mergeOid = mergeOids[m];
    const backboneTarget = backbone[forkPoints[m] - 50];
    const sideTip = sideBranches[m][0];
    allCommits.set(mergeOid, {
      oid: mergeOid,
      parents: [backboneTarget, sideTip],
      refKinds: [],
      refNames: [],
      isHead: false,
    });
  }

  // Regular merge commits from backbone segments
  for (let m = 5; m < 29; m++) {
    const mergeOid = mergeOids[m];
    const p1idx = Math.floor(rng() * 900);
    const p2idx = Math.floor(rng() * 900);
    allCommits.set(mergeOid, {
      oid: mergeOid,
      parents: [backbone[p1idx], backbone[p2idx]],
      refKinds: [],
      refNames: [],
      isHead: false,
    });
  }

  // Octopus merge (parents.length = 4)
  allCommits.set(mergeOids[29], {
    oid: mergeOids[29],
    parents: [backbone[10], backbone[20], backbone[30], backbone[40]],
    refKinds: [],
    refNames: [],
    isHead: false,
  });

  // Annotate branch tips with refs
  // HEAD = backbone[0] with main
  const headOid = backbone[0];
  allCommits.get(headOid).isHead = true;
  allCommits.get(headOid).refKinds = ["local"];
  allCommits.get(headOid).refNames = ["main"];

  // Annotate remaining branches on various commits
  for (let b = 1; b < BRANCHES.length; b++) {
    const targetIdx = Math.floor(rng() * (allCommits.size - 1));
    const keys = Array.from(allCommits.keys());
    const targetOid = keys[targetIdx % keys.length];
    const c = allCommits.get(targetOid);
    const kind = BRANCH_KIND[b];
    if (!c.refKinds.includes(kind)) c.refKinds.push(kind);
    c.refNames.push(BRANCHES[b]);
  }

  // Sort newest-first and take first 1000
  const sorted = Array.from(allCommits.values()).slice(0, total);

  return sorted;
}

// ── Kernel-like 100k fixture ─────────────────────────────────────────────────
// 100k commits, 80 branches, ~12% merge density (12000 merge commits)
function generateKernelLike100k() {
  const rng = makeRng(1337);
  const total = 100000;
  const mergeCount = Math.floor(total * 0.12); // 12000 merges
  const normalCount = total - mergeCount;

  const commits = [];

  // Build a simple linear backbone for performance baseline (newest-first order)
  // Index 0 = HEAD (newest), index normalCount-1 = root (oldest)
  for (let i = 0; i < normalCount; i++) {
    const oid = sha("k", i);
    const parents = i < normalCount - 1 ? [sha("k", i + 1)] : [];
    commits.push({ oid, parents, refKinds: [], refNames: [], isHead: false });
  }

  // Merge commits (2 parents, both from backbone; avoid index 0 = HEAD)
  for (let m = 0; m < mergeCount; m++) {
    const oid = sha("km", m);
    const p1idx = (Math.floor(rng() * (normalCount - 1)) + 1);
    const p2idx = (Math.floor(rng() * (normalCount - 1)) + 1);
    commits.push({
      oid,
      parents: [sha("k", p1idx), sha("k", p2idx)],
      refKinds: [],
      refNames: [],
      isHead: false,
    });
  }

  // Annotate 80 branches
  const BRANCH_PREFIXES = [
    "main","stable","lts","dev","staging","release","hotfix","rc",
    "feat/net","feat/mm","feat/fs","feat/drv","feat/ipc","feat/sched",
    "feat/sec","feat/virt","feat/perf","feat/compat",
    "fix/null","fix/race","fix/oob","fix/uaf","fix/leak","fix/deadlock",
    "fix/timeout","fix/panic",
    "origin/main","origin/stable","origin/lts",
    "origin/feat/net","origin/feat/mm","origin/feat/fs",
    "origin/fix/null","origin/fix/race",
    "v5.0","v5.1","v5.2","v5.3","v5.4","v5.5","v5.6","v5.7","v5.8","v5.9",
    "v6.0","v6.1","v6.2","v6.3","v6.4","v6.5","v6.6","v6.7","v6.8","v6.9",
    "v7.0","v7.1","v7.2","v7.3","v7.4","v7.5",
    "local/ath0","local/ath1","local/ath2","local/ath3","local/ath4",
    "local/ath5","local/ath6","local/ath7","local/ath8","local/ath9",
    "wip/mm-rework","wip/net-refactor","wip/sched-eevdf",
    "ci/test-suite","ci/coverage",
  ];

  for (let b = 0; b < 80 && b < BRANCH_PREFIXES.length; b++) {
    const targetIdx = Math.floor(rng() * total);
    const target = commits[targetIdx % commits.length];
    const kind = b < 30 ? "local" : b < 50 ? "remote" : "tag";
    if (!target.refKinds.includes(kind)) target.refKinds.push(kind);
    target.refNames.push(BRANCH_PREFIXES[b]);
  }

  // HEAD = first commit (newest, index 0)
  commits[0].isHead = true;
  if (!commits[0].refKinds.includes("local")) commits[0].refKinds.push("local");
  if (!commits[0].refNames.includes("main")) commits[0].refNames.push("main");

  return commits;
}

// ── Generate + write ─────────────────────────────────────────────────────────
const linear20 = generateLinear20();
const branchy1k = generateBranchy1k();
const kernelLike100k = generateKernelLike100k();

writeFileSync(
  join(__dirname, "fixture_linear_20.json"),
  JSON.stringify(linear20, null, 2),
);
writeFileSync(
  join(__dirname, "fixture_branchy_1k.json"),
  JSON.stringify(branchy1k, null, 2),
);
writeFileSync(
  join(__dirname, "fixture_kernel_like_100k.json"),
  JSON.stringify(kernelLike100k, null, 2),
);

console.log(`Generated fixture_linear_20.json (${linear20.length} commits)`);
console.log(`Generated fixture_branchy_1k.json (${branchy1k.length} commits)`);
console.log(`Generated fixture_kernel_like_100k.json (${kernelLike100k.length} commits)`);
