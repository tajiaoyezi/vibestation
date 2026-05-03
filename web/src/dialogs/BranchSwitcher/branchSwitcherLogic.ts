import type { BranchInfo, SwitcherMatch } from "../../bindings";

export type BranchSwitcherGroup = "current" | "recent" | "local" | "remote";

export interface BranchSwitcherItem extends SwitcherMatch {
  group: BranchSwitcherGroup;
}

const collator = new Intl.Collator(undefined, { sensitivity: "base" });
type FuzzyMatch = NonNullable<ReturnType<typeof fuzzyMatch>>;

export function fuzzyMatch(
  candidate: string,
  query: string,
): { score: number; indices: number[] } | null {
  const chars = Array.from(candidate);
  const lower = chars.map((char) => char.toLowerCase());
  const needle = Array.from(query.toLowerCase());
  const indices: number[] = [];
  let cursor = 0;
  let score = 0;
  for (const char of needle) {
    const next = lower.findIndex(
      (candidateChar, index) => index >= cursor && candidateChar === char,
    );
    if (next < 0) return null;
    score += indices.at(-1) === next - 1 ? 3 : 1;
    if (next === 0 || ["/", "-", "_"].includes(chars[next - 1] ?? ""))
      score += 1;
    indices.push(next);
    cursor = next + 1;
  }
  if (candidate.toLowerCase().includes(query.toLowerCase())) score += 20;
  score -= (indices[0] ?? 0) * 0.01;
  return { score, indices };
}

export function buildSwitcherItems(
  branches: BranchInfo[],
  headName: string | null,
  query: string,
  recentNames: string[],
): BranchSwitcherItem[] {
  const eligible = branches.filter(
    (branch) =>
      (branch.kind === "local" || branch.kind === "remote") &&
      !branch.name.endsWith("/HEAD"),
  );
  const trimmed = query.trim();
  if (trimmed) {
    return eligible
      .map((branch) => ({ branch, match: fuzzyMatch(branch.name, trimmed) }))
      .filter(
        (item): item is { branch: BranchInfo; match: FuzzyMatch } =>
          item.match !== null,
      )
      .map(({ branch, match }) => ({
        branch,
        score: match.score,
        matchIndices: match.indices,
        group:
          branch.kind === "remote" ? ("remote" as const) : ("local" as const),
      }))
      .sort(compareByScore);
  }

  const byName = new Map(eligible.map((branch) => [branch.name, branch]));
  const current = eligible.filter(
    (branch) => branch.kind === "local" && branch.name === headName,
  );
  const recent = recentNames
    .map((name) => byName.get(name))
    .filter(
      (branch): branch is BranchInfo =>
        branch !== undefined &&
        branch.kind === "local" &&
        branch.name !== headName,
    );
  const used = new Set([...current, ...recent].map((branch) => branch.name));
  const locals = eligible
    .filter((branch) => branch.kind === "local" && !used.has(branch.name))
    .sort(compareBranchName);
  const remotes = eligible
    .filter((branch) => branch.kind === "remote" && !used.has(branch.name))
    .sort(compareBranchName);

  return [
    ...current.map((branch) => toItem(branch, "current")),
    ...recent.map((branch) => toItem(branch, "recent")),
    ...locals.map((branch) => toItem(branch, "local")),
    ...remotes.map((branch) => toItem(branch, "remote")),
  ];
}

function toItem(
  branch: BranchInfo,
  group: BranchSwitcherGroup,
): BranchSwitcherItem {
  return { branch, score: 0, matchIndices: [], group };
}

function compareByScore(
  left: BranchSwitcherItem,
  right: BranchSwitcherItem,
): number {
  return (
    right.score - left.score ||
    kindRank(left.branch.kind) - kindRank(right.branch.kind) ||
    collator.compare(left.branch.name, right.branch.name)
  );
}

function compareBranchName(left: BranchInfo, right: BranchInfo): number {
  return collator.compare(left.name, right.name);
}

function kindRank(kind: BranchInfo["kind"]): number {
  return kind === "local" ? 0 : 1;
}
