import { describe, it, expect } from "vitest";
import { branchNameToColorKey } from "../../../../src/panels/GitLog/RailGraph/color-mapper";

describe("branchNameToColorKey", () => {
  it("returns same key on 10 consecutive calls for same input (stability)", () => {
    const branch = "feat/my-feature";
    const key = branchNameToColorKey(branch);
    for (let i = 0; i < 9; i++) {
      expect(branchNameToColorKey(branch)).toBe(key);
    }
  });

  it("output is within 30-color ring (color-0 to color-29)", () => {
    const branches = [
      "main",
      "feat/alpha",
      "fix/issue-1",
      "origin/main",
      "v1.0.0",
      "",
      "a".repeat(100),
    ];
    for (const branch of branches) {
      const key = branchNameToColorKey(branch);
      expect(key).toMatch(/^color-\d+$/);
      const idx = parseInt(key.replace("color-", ""), 10);
      expect(idx).toBeGreaterThanOrEqual(0);
      expect(idx).toBeLessThan(30);
    }
  });
});
