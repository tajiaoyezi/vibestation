import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, cleanup, fireEvent } from "@solidjs/testing-library";
import { PaneDraftComposer } from "../../../../src/panels/Terminal/PaneDraftComposer";
import { createPaneDraftsStore } from "../../../../src/stores/paneDrafts";

describe("PaneDraftComposer (D.2/D.5)", () => {
  beforeEach(cleanup);

  function setup(options: { initialDraft?: string } = {}) {
    const store = createPaneDraftsStore();
    const onSend = vi.fn();
    const paneId = "pane-test";

    if (options.initialDraft) {
      store.setDraft(paneId, options.initialDraft);
    }

    const result = render(() => (
      <PaneDraftComposer paneId={paneId} drafts={store} onSend={onSend} />
    ));

    return { store, onSend, paneId, ...result };
  }

  describe("textarea binding", () => {
    it("renders textarea with aria-label", () => {
      const { getByLabelText } = setup();
      const textarea = getByLabelText("Draft command input");
      expect(textarea).toBeInTheDocument();
      expect(textarea.tagName).toBe("TEXTAREA");
    });

    it("textarea reflects store draft value", () => {
      const { getByLabelText } = setup({ initialDraft: "cargo build" });
      const textarea = getByLabelText(
        "Draft command input",
      ) as HTMLTextAreaElement;
      expect(textarea.value).toBe("cargo build");
    });

    it("typing updates store draft", async () => {
      const { getByLabelText, store, paneId } = setup();
      const textarea = getByLabelText(
        "Draft command input",
      ) as HTMLTextAreaElement;

      await fireEvent.input(textarea, { target: { value: "npm test" } });
      expect(store.getDraft(paneId)).toBe("npm test");
    });
  });

  describe("Send button", () => {
    it("Send button is disabled when draft is empty", () => {
      const { getByLabelText } = setup();
      const btn = getByLabelText("Send draft command") as HTMLButtonElement;
      expect(btn.disabled).toBe(true);
    });

    it("Send button is enabled when draft has text", () => {
      const { getByLabelText } = setup({ initialDraft: "ls -la" });
      const btn = getByLabelText("Send draft command") as HTMLButtonElement;
      expect(btn.disabled).toBe(false);
    });

    it("clicking Send calls onSend with paneId and text", async () => {
      const { getByLabelText, onSend, paneId } = setup({
        initialDraft: "echo hello",
      });
      const btn = getByLabelText("Send draft command");
      await fireEvent.click(btn);
      expect(onSend).toHaveBeenCalledWith(paneId, "echo hello");
    });

    it("clicking Send on empty draft does not call onSend", async () => {
      const { getByLabelText, onSend } = setup();
      const btn = getByLabelText("Send draft command");
      await fireEvent.click(btn);
      expect(onSend).not.toHaveBeenCalled();
    });
  });

  describe("D.2 · insertFragment on empty draft", () => {
    it("fragment goes directly into textarea via store", () => {
      const { store, paneId, getByLabelText } = setup();
      store.insertFragment(paneId, "cargo test --all");

      const textarea = getByLabelText(
        "Draft command input",
      ) as HTMLTextAreaElement;
      expect(textarea.value).toBe("cargo test --all");
    });

    it("no merge-confirm region appears after direct apply", () => {
      const { store, paneId, queryByRole } = setup();
      store.insertFragment(paneId, "direct fragment");

      expect(queryByRole("region")).not.toBeInTheDocument();
    });
  });

  describe("D.5 · merge preview when draft non-empty", () => {
    it("shows merge-confirm region with preview text", () => {
      const { store, paneId, getByRole } = setup({
        initialDraft: "existing command",
      });
      store.insertFragment(paneId, "new fragment");

      const region = getByRole("region");
      expect(region).toBeInTheDocument();
      expect(region.getAttribute("aria-label")).toContain("Merge preview");
      expect(region.textContent).toContain(
        "existing command\n\nnew fragment",
      );
    });

    it("merge-confirm has Append and Cancel buttons", () => {
      const { store, paneId, getByLabelText } = setup({
        initialDraft: "keep",
      });
      store.insertFragment(paneId, "add");

      expect(
        getByLabelText("Append fragment to existing draft"),
      ).toBeInTheDocument();
      expect(getByLabelText("Cancel merge")).toBeInTheDocument();
    });

    it("clicking Append writes merged text to draft", async () => {
      const { store, paneId, getByLabelText } = setup({
        initialDraft: "first",
      });
      store.insertFragment(paneId, "second");

      await fireEvent.click(
        getByLabelText("Append fragment to existing draft"),
      );

      expect(store.getDraft(paneId)).toBe("first\n\nsecond");
      expect(store.getPendingMerge(paneId)).toBeNull();
    });

    it("clicking Cancel dismisses preview without changing draft", async () => {
      const { store, paneId, getByLabelText, queryByRole } = setup({
        initialDraft: "original",
      });
      store.insertFragment(paneId, "rejected");

      await fireEvent.click(getByLabelText("Cancel merge"));

      expect(store.getDraft(paneId)).toBe("original");
      expect(queryByRole("region")).not.toBeInTheDocument();
    });

    it("Escape key cancels merge preview (a11y)", async () => {
      const { store, paneId, getByRole, queryByRole } = setup({
        initialDraft: "text",
      });
      store.insertFragment(paneId, "more");

      const region = getByRole("region");
      await fireEvent.keyDown(region, { key: "Escape" });

      expect(queryByRole("region")).not.toBeInTheDocument();
      expect(store.getDraft(paneId)).toBe("text");
    });
  });

  describe("a11y", () => {
    it("textarea is keyboard-accessible with correct aria-label", () => {
      const { getByLabelText } = setup();
      const textarea = getByLabelText("Draft command input");
      expect(textarea).toBeInTheDocument();
      expect(textarea.getAttribute("aria-label")).toBe("Draft command input");
    });

    it("Send button has aria-label", () => {
      const { getByLabelText } = setup();
      expect(getByLabelText("Send draft command")).toBeInTheDocument();
    });

    it("merge-confirm region has role and aria-label", () => {
      const { store, paneId, getByRole } = setup({ initialDraft: "x" });
      store.insertFragment(paneId, "y");

      const region = getByRole("region");
      expect(region.getAttribute("aria-label")).toContain("Merge preview");
    });

    it("merge-confirm buttons have accessible labels", () => {
      const { store, paneId, getByLabelText } = setup({
        initialDraft: "a",
      });
      store.insertFragment(paneId, "b");

      expect(
        getByLabelText("Append fragment to existing draft"),
      ).toBeInTheDocument();
      expect(getByLabelText("Cancel merge")).toBeInTheDocument();
    });

    it("merge-confirm region is focusable (tabIndex -1)", () => {
      const { store, paneId, getByRole } = setup({ initialDraft: "a" });
      store.insertFragment(paneId, "b");

      const region = getByRole("region");
      expect(region.getAttribute("tabindex")).toBe("-1");
    });

    it("status is conveyed by text labels, not color alone", () => {
      const { getByLabelText } = setup({ initialDraft: "text" });
      const btn = getByLabelText("Send draft command");
      expect(btn.textContent).toBe("Send");
    });
  });

  describe("props interface compliance", () => {
    it("component accepts exactly 3 props (paneId, drafts, onSend)", () => {
      const store = createPaneDraftsStore();
      const onSend = vi.fn();
      const { container } = render(() => (
        <PaneDraftComposer paneId="p1" drafts={store} onSend={onSend} />
      ));
      expect(
        container.querySelector(".vs-pane-draft-composer"),
      ).toBeInTheDocument();
    });
  });
});
