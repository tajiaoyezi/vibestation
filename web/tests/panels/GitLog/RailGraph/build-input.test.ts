import { describe, it, expect } from "vitest";
import type { GitLogEntry, BranchInfo } from "../../../../src/bindings";
import { buildRailGraphInputFromGitLog } from "../../../../src/panels/GitLog/RailGraph/build-input";

function makeEntry(shortSha: string, branchLabels: string[] = [], tagLabels: string[] = []): GitLogEntry {
  return {
    shortSha,
    message: `commit ${shortSha}`,
    authorName: "Test",
    authoredDate: 0,
    relativeTime: "just now",
    branchLabels,
    tagLabels,
  };
}

function makeBranch(name: string, fullRef: string, kind: "local" | "remote" | "tag", headCommit: string | null = null): BranchInfo {
  return { name, fullRef, kind, upstream: null, ahead: 0, behind: 0, headCommit };
}

describe("buildRailGraphInputFromGitLog", () => {
  it("deduplicates entries with the same shortSha (A-Task 11)", () => {
    const entries = [makeEntry("abc"), makeEntry("abc"), makeEntry("def")];
    const result = buildRailGraphInputFromGitLog(entries, [], null);
    expect(result).toHaveLength(2);
    expect(result.map((c) => c.oid)).toEqual(["abc", "def"]);
  });

  it("marks orphan parent (parent missing from input) as empty parents in Phase A", () => {
    // Phase A: parents always [] from GitLogEntry — shallow clone fallback
    const entries = [makeEntry("abc")];
    const result = buildRailGraphInputFromGitLog(entries, [], null);
    expect(result[0].parents).toEqual([]);
  });

  it("detached HEAD: isHead based on headOid prefix match, refNames may be empty (A.6)", () => {
    const entries = [makeEntry("deadbeef")];
    // headOid is a full SHA that starts with shortSha
    const result = buildRailGraphInputFromGitLog(entries, [], "deadbeefcafe1234");
    expect(result[0].isHead).toBe(true);
    // No branch ref → refNames may or may not have labels (from branchLabels)
    expect(result[0].refNames.includes("deadbeef") || result[0].refNames.length === 0).toBe(true);
  });

  it("annotates refs from BranchInfo.headCommit when it matches shortSha", () => {
    const entries = [makeEntry("abc")];
    const refs = [makeBranch("main", "refs/heads/main", "local", "abc")];
    const result = buildRailGraphInputFromGitLog(entries, refs, null);
    expect(result[0].refNames).toContain("main");
    expect(result[0].refKinds).toContain("local");
  });

  it("annotates refs from GitLogEntry branchLabels and tagLabels", () => {
    const entries = [makeEntry("abc", ["main", "origin/main"], ["v1.0"])];
    const result = buildRailGraphInputFromGitLog(entries, [], null);
    expect(result[0].refNames).toContain("main");
    expect(result[0].refNames).toContain("origin/main");
    expect(result[0].refNames).toContain("v1.0");
    expect(result[0].refKinds).toContain("tag");
  });
});
