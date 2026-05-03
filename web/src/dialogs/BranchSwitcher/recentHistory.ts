export interface BranchRecentEntry {
  name: string;
  checkedOutAt: number;
}

const MAX_RECENT_BRANCHES = 5;

function keyFor(workspaceId: string): string {
  return `branch_recent_${workspaceId}`;
}

export function loadRecentBranches(workspaceId: string): BranchRecentEntry[] {
  try {
    const raw = localStorage.getItem(keyFor(workspaceId));
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter(
        (entry): entry is BranchRecentEntry =>
          typeof entry === "object" &&
          entry !== null &&
          typeof (entry as BranchRecentEntry).name === "string" &&
          typeof (entry as BranchRecentEntry).checkedOutAt === "number",
      )
      .slice(0, MAX_RECENT_BRANCHES);
  } catch {
    return [];
  }
}

export function recordRecentBranch(
  workspaceId: string,
  name: string,
): BranchRecentEntry[] {
  const next = [
    { name, checkedOutAt: Date.now() },
    ...loadRecentBranches(workspaceId).filter((entry) => entry.name !== name),
  ].slice(0, MAX_RECENT_BRANCHES);

  try {
    localStorage.setItem(keyFor(workspaceId), JSON.stringify(next));
  } catch {
    // localStorage may be unavailable in restricted WebView contexts.
  }

  return next;
}
