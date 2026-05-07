// MVP-12 Phase A · Branch name → stable color key
// Uses djb2 hash · 30-color ring · deterministic (same input → same output)

/** Total number of colors in the ring */
const COLOR_RING_SIZE = 30;

/**
 * Hash a branch name to a stable 0-based index in a 30-color ring.
 * Algorithm: djb2 (hash = hash * 33 ^ charCode).
 * Properties: deterministic · uniform distribution · no external deps.
 */
function djb2Hash(s: string): number {
  let hash = 5381;
  for (let i = 0; i < s.length; i++) {
    hash = ((hash << 5) + hash) ^ s.charCodeAt(i);
    hash = hash | 0; // keep as 32-bit int
  }
  return Math.abs(hash);
}

/**
 * Map a branch name to a stable colorKey string (e.g. "color-7").
 * Same branch name always produces the same key across sessions.
 * The key is a string so Phase B can map it to theme tokens without knowing the ring size.
 */
export function branchNameToColorKey(branchName: string): string {
  const index = djb2Hash(branchName) % COLOR_RING_SIZE;
  return `color-${index}`;
}
