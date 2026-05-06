// MVP-12 Phase A · Adapter: GitLogEntry[] + BranchInfo[] → RailGraphInputCommit[]
// Consumes MVP-07 commit list data. Does NOT change the original list API.

import type { BranchInfo, GitLogEntry } from "../../../bindings";
import type { RailGraphInputCommit } from "./types";
import { buildOidToRefsMap } from "./refs-normalize";

/**
 * Build the rail graph input from the MVP-07 commit list + branch refs.
 *
 * Phase A limitations (will be lifted in Phase D when GitLogEntry gains oid + parents):
 * - Uses shortSha as the commit identifier (oid proxy)
 * - parents is always [] (GitLogEntry v0.1 doesn't expose parent SHAs)
 *   → lane allocator falls back to linear layout
 *
 * @param entries  GitLogEntry[] from MVP-07 data pipeline (time-descending order preserved)
 * @param refs     BranchInfo[] from MVP-13 branch list (used to annotate refs on commits)
 * @param headOid  Current HEAD OID (full SHA or shortSha) or null for detached/unknown
 */
export function buildRailGraphInputFromGitLog(
  entries: GitLogEntry[],
  refs: BranchInfo[],
  headOid: string | null,
): RailGraphInputCommit[] {
  const oidToRefs = buildOidToRefsMap(refs);

  // Deduplicate entries by oid (shortSha) – same commit appearing twice is dropped
  const seen = new Set<string>();
  const result: RailGraphInputCommit[] = [];

  for (const entry of entries) {
    const oid = entry.shortSha;

    if (seen.has(oid)) continue;
    seen.add(oid);

    const refsForCommit = oidToRefs.get(oid) ?? { refKinds: [], refNames: [] };

    // Also check branchLabels / tagLabels from GitLogEntry itself
    const refKinds = new Set(refsForCommit.refKinds);
    const refNames = new Set(refsForCommit.refNames);

    for (const label of entry.branchLabels) {
      refNames.add(label);
      // Infer kind from label format
      if (label.includes("/")) refKinds.add("remote");
      else refKinds.add("local");
    }
    for (const label of entry.tagLabels) {
      refNames.add(label);
      refKinds.add("tag");
    }

    // isHead: shortSha prefix match against headOid
    const isHead =
      headOid != null &&
      (headOid === oid || headOid.startsWith(oid) || oid.startsWith(headOid));

    result.push({
      oid,
      parents: [], // Phase A: no parent info in GitLogEntry · Phase D will add
      refKinds: Array.from(refKinds),
      refNames: Array.from(refNames),
      isHead,
    });
  }

  return result;
}
