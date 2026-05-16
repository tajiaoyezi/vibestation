import { describe, expect, it } from "vitest";
import { createPaneDraftsStore } from "../../../../src/stores/paneDrafts";

describe("paneDrafts store", () => {
  it("returns empty string for unknown paneId", () => {
    const store = createPaneDraftsStore();
    expect(store.getDraft("nonexistent")).toBe("");
  });

  it("hasDraft returns false for empty/unknown draft", () => {
    const store = createPaneDraftsStore();
    expect(store.hasDraft("pane-1")).toBe(false);
  });

  it("setDraft stores text and getDraft retrieves it", () => {
    const store = createPaneDraftsStore();
    store.setDraft("pane-1", "cargo test");
    expect(store.getDraft("pane-1")).toBe("cargo test");
    expect(store.hasDraft("pane-1")).toBe(true);
  });

  it("clearDraft resets to empty", () => {
    const store = createPaneDraftsStore();
    store.setDraft("pane-1", "some text");
    store.clearDraft("pane-1");
    expect(store.getDraft("pane-1")).toBe("");
    expect(store.hasDraft("pane-1")).toBe(false);
  });

  it("isolates drafts per paneId", () => {
    const store = createPaneDraftsStore();
    store.setDraft("pane-a", "text-a");
    store.setDraft("pane-b", "text-b");
    expect(store.getDraft("pane-a")).toBe("text-a");
    expect(store.getDraft("pane-b")).toBe("text-b");

    store.clearDraft("pane-a");
    expect(store.getDraft("pane-a")).toBe("");
    expect(store.getDraft("pane-b")).toBe("text-b");
  });

  describe("D.2 · insertFragment on empty draft", () => {
    it("directly writes fragment when draft is empty", () => {
      const store = createPaneDraftsStore();
      const result = store.insertFragment("pane-1", "fix the bug");

      expect(result.applied).toBe(true);
      expect(result.needsPreview).toBe(false);
      expect(result.previewText).toBe("fix the bug");
      expect(store.getDraft("pane-1")).toBe("fix the bug");
    });

    it("clears any pending merge on direct apply", () => {
      const store = createPaneDraftsStore();
      store.insertFragment("pane-1", "direct write");
      expect(store.getPendingMerge("pane-1")).toBeNull();
    });
  });

  describe("D.5 · insertFragment on non-empty draft (merge preview)", () => {
    it("returns needsPreview=true and does not write", () => {
      const store = createPaneDraftsStore();
      store.setDraft("pane-1", "existing command");
      const result = store.insertFragment("pane-1", "new fragment");

      expect(result.applied).toBe(false);
      expect(result.needsPreview).toBe(true);
      expect(result.previewText).toBe("existing command\n\nnew fragment");
      expect(store.getDraft("pane-1")).toBe("existing command");
    });

    it("sets reactive pending merge state", () => {
      const store = createPaneDraftsStore();
      store.setDraft("pane-1", "existing");
      store.insertFragment("pane-1", "new");

      const pending = store.getPendingMerge("pane-1");
      expect(pending).not.toBeNull();
      expect(pending!.needsPreview).toBe(true);
      expect(pending!.previewText).toBe("existing\n\nnew");
    });
  });

  describe("confirmMerge", () => {
    it("writes merged text and clears pending state", () => {
      const store = createPaneDraftsStore();
      store.setDraft("pane-1", "original");
      store.insertFragment("pane-1", "appended");

      const pending = store.getPendingMerge("pane-1");
      expect(pending).not.toBeNull();

      store.confirmMerge("pane-1", pending!.previewText);
      expect(store.getDraft("pane-1")).toBe("original\n\nappended");
      expect(store.getPendingMerge("pane-1")).toBeNull();
    });

    it("accepts caller-modified merged text", () => {
      const store = createPaneDraftsStore();
      store.setDraft("pane-1", "a");
      store.insertFragment("pane-1", "b");

      store.confirmMerge("pane-1", "completely custom text");
      expect(store.getDraft("pane-1")).toBe("completely custom text");
    });
  });

  describe("clearPendingMerge", () => {
    it("clears pending without modifying draft", () => {
      const store = createPaneDraftsStore();
      store.setDraft("pane-1", "keep this");
      store.insertFragment("pane-1", "rejected");

      expect(store.getPendingMerge("pane-1")).not.toBeNull();
      store.clearPendingMerge("pane-1");
      expect(store.getPendingMerge("pane-1")).toBeNull();
      expect(store.getDraft("pane-1")).toBe("keep this");
    });
  });

  describe("clearDraft clears pending merge too", () => {
    it("clears both draft and pending on clearDraft", () => {
      const store = createPaneDraftsStore();
      store.setDraft("pane-1", "text");
      store.insertFragment("pane-1", "fragment");

      store.clearDraft("pane-1");
      expect(store.getDraft("pane-1")).toBe("");
      expect(store.getPendingMerge("pane-1")).toBeNull();
    });
  });

  it("getPendingMerge returns null for unknown paneId", () => {
    const store = createPaneDraftsStore();
    expect(store.getPendingMerge("unknown")).toBeNull();
  });
});
