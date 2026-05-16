import { beforeEach, describe, expect, it } from "vitest";
import { cleanup, render } from "@solidjs/testing-library";
import {
  PaneDraftsProvider,
  usePaneDrafts,
} from "../../../../src/stores/paneDrafts-context";
import type { PaneDraftsStore } from "../../../../src/stores/paneDrafts";

describe("paneDrafts-context", () => {
  beforeEach(cleanup);

  it("throws when usePaneDrafts is used outside PaneDraftsProvider", () => {
    const Probe = () => {
      usePaneDrafts();
      return null;
    };
    expect(() => render(() => <Probe />)).toThrow(
      /usePaneDrafts must be used within PaneDraftsProvider/,
    );
  });

  it("provides a working PaneDraftsStore inside the provider", () => {
    let captured: PaneDraftsStore | undefined;
    const Probe = () => {
      captured = usePaneDrafts();
      return null;
    };
    render(() => (
      <PaneDraftsProvider>
        <Probe />
      </PaneDraftsProvider>
    ));

    expect(captured).toBeDefined();
    const store = captured as PaneDraftsStore;
    store.setDraft("pane-1", "hello");
    expect(store.getDraft("pane-1")).toBe("hello");
    expect(store.hasDraft("pane-1")).toBe(true);

    // D.5 semantics flow through the provider-held store unchanged.
    const merge = store.insertFragment("pane-1", "world");
    expect(merge.applied).toBe(false);
    expect(merge.needsPreview).toBe(true);
    expect(merge.previewText).toBe("hello\n\nworld");
  });

  it("shares a single store instance across consumers (app-level provider)", () => {
    const seen: PaneDraftsStore[] = [];
    const Probe = () => {
      seen.push(usePaneDrafts());
      return null;
    };
    render(() => (
      <PaneDraftsProvider>
        <Probe />
        <Probe />
      </PaneDraftsProvider>
    ));

    expect(seen).toHaveLength(2);
    // cross-pane Insert flow depends on one shared instance keyed by paneId.
    seen[0].setDraft("shared-pane", "from-consumer-0");
    expect(seen[1].getDraft("shared-pane")).toBe("from-consumer-0");
  });
});
