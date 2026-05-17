import { describe, expect, it } from "vitest";
import { DEFAULT_LAYOUT, layoutReducer } from "../../src/stores/layout";

describe("layoutReducer open-bottom", () => {
  it("opens bottom panel when collapsed", () => {
    const state = { ...DEFAULT_LAYOUT, bottomOpen: false };
    const next = layoutReducer(state, { kind: "open-bottom" });
    expect(next.bottomOpen).toBe(true);
  });

  it("is idempotent when bottom panel already open", () => {
    const state = { ...DEFAULT_LAYOUT, bottomOpen: true };
    const next = layoutReducer(state, { kind: "open-bottom" });
    expect(next).toEqual(state);
  });
});
