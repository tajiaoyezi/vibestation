// MVP-12 Phase A · refs normalization
// Classifies BranchInfo refs into local / remote / tag categories

import type { BranchInfo } from "../../../bindings";

export interface NormalizedRefs {
  local: string[];
  remote: string[];
  tag: string[];
}

/**
 * Normalize BranchInfo[] into classified local/remote/tag name lists.
 * Uses fullRef to classify (refs/heads/ → local, refs/remotes/ → remote, refs/tags/ → tag).
 * Deduplicates within each category.
 * Total count = local.length + remote.length + tag.length === input.length (no loss).
 */
export function normalizeRefs(refs: BranchInfo[]): NormalizedRefs {
  const local: string[] = [];
  const remote: string[] = [];
  const tag: string[] = [];

  const seen = new Set<string>();

  for (const ref of refs) {
    const key = ref.fullRef;
    if (seen.has(key)) continue;
    seen.add(key);

    if (ref.fullRef.startsWith("refs/heads/")) {
      local.push(ref.name);
    } else if (ref.fullRef.startsWith("refs/remotes/")) {
      remote.push(ref.name);
    } else if (ref.fullRef.startsWith("refs/tags/")) {
      tag.push(ref.name);
    } else {
      // fallback: classify by BranchKind
      if (ref.kind === "local") local.push(ref.name);
      else if (ref.kind === "remote") remote.push(ref.name);
      else tag.push(ref.name);
    }
  }

  return { local, remote, tag };
}

/**
 * Build a map from commit OID → refs that point to it.
 * Used by buildRailGraphInputFromGitLog to annotate commits.
 */
export function buildOidToRefsMap(
  refs: BranchInfo[],
): Map<
  string,
  { refKinds: ("local" | "remote" | "tag")[]; refNames: string[] }
> {
  const map = new Map<
    string,
    { refKinds: ("local" | "remote" | "tag")[]; refNames: string[] }
  >();

  for (const ref of refs) {
    if (!ref.headCommit) continue;
    const oid = ref.headCommit;

    const existing = map.get(oid) ?? { refKinds: [], refNames: [] };

    let kind: "local" | "remote" | "tag";
    if (ref.fullRef.startsWith("refs/heads/")) kind = "local";
    else if (ref.fullRef.startsWith("refs/remotes/")) kind = "remote";
    else if (ref.fullRef.startsWith("refs/tags/")) kind = "tag";
    else kind = ref.kind;

    if (!existing.refKinds.includes(kind)) {
      existing.refKinds.push(kind);
    }
    existing.refNames.push(ref.name);

    map.set(oid, existing);
  }

  return map;
}
