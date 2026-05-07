import { describe, it, expect } from "vitest";
import type { BranchInfo } from "../../../../src/bindings";
import {
  normalizeRefs,
  buildOidToRefsMap,
} from "../../../../src/panels/GitLog/RailGraph/refs-normalize";

function makeBranchInfo(
  name: string,
  fullRef: string,
  kind: "local" | "remote" | "tag",
  headCommit: string | null = null,
): BranchInfo {
  return { name, fullRef, kind, upstream: null, ahead: 0, behind: 0, headCommit };
}

describe("normalizeRefs", () => {
  it("classifies local/remote/tag refs with zero loss (A.7)", () => {
    const refs: BranchInfo[] = [
      makeBranchInfo("main", "refs/heads/main", "local"),
      makeBranchInfo("feat/x", "refs/heads/feat/x", "local"),
      makeBranchInfo("origin/main", "refs/remotes/origin/main", "remote"),
      makeBranchInfo("v1.0", "refs/tags/v1.0", "tag"),
    ];
    const result = normalizeRefs(refs);
    expect(result.local).toHaveLength(2);
    expect(result.remote).toHaveLength(1);
    expect(result.tag).toHaveLength(1);
    expect(result.local.length + result.remote.length + result.tag.length).toBe(
      refs.length,
    );
  });

  it("deduplicates refs with same fullRef (no double-counting)", () => {
    const refs: BranchInfo[] = [
      makeBranchInfo("main", "refs/heads/main", "local"),
      makeBranchInfo("main", "refs/heads/main", "local"), // duplicate fullRef
    ];
    const result = normalizeRefs(refs);
    expect(result.local).toHaveLength(1);
    expect(result.remote).toHaveLength(0);
    expect(result.tag).toHaveLength(0);
  });
});

describe("buildOidToRefsMap", () => {
  it("maps commit OID to its associated refs", () => {
    const refs: BranchInfo[] = [
      makeBranchInfo("main", "refs/heads/main", "local", "abc123"),
      makeBranchInfo("feat", "refs/heads/feat", "local", "def456"),
    ];
    const map = buildOidToRefsMap(refs);
    expect(map.get("abc123")?.refNames).toContain("main");
    expect(map.get("def456")?.refNames).toContain("feat");
    expect(map.has("unknown")).toBe(false);
  });

  it("ignores refs without headCommit", () => {
    const refs: BranchInfo[] = [
      makeBranchInfo("detached", "refs/heads/detached", "local", null),
    ];
    const map = buildOidToRefsMap(refs);
    expect(map.size).toBe(0);
  });
});
