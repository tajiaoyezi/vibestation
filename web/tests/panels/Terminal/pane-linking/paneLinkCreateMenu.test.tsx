import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, cleanup, fireEvent } from "@solidjs/testing-library";
import { PaneLinkCreateMenu } from "../../../../src/panels/Terminal/PaneLinkCreateMenu";

const candidates = [
  { paneId: "pane-ai", label: "claude — ~/proj" },
  { paneId: "pane-runner-2", label: "zsh — ~/proj/api" },
];

describe("PaneLinkCreateMenu (D.4 · create affordance)", () => {
  beforeEach(cleanup);

  it("does not render when closed", () => {
    const { queryByRole } = render(() => (
      <PaneLinkCreateMenu
        open={false}
        currentPaneId="pane-runner"
        candidatePanes={candidates}
        onClose={() => {}}
        onCreate={() => {}}
      />
    ));
    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("renders one option per candidate pane when open", () => {
    const { getByRole, getAllByRole } = render(() => (
      <PaneLinkCreateMenu
        open={true}
        currentPaneId="pane-runner"
        candidatePanes={candidates}
        onClose={() => {}}
        onCreate={() => {}}
      />
    ));
    expect(getByRole("dialog")).toBeInTheDocument();
    const opts = getAllByRole("button").filter((b) =>
      b.getAttribute("aria-label")?.startsWith("Link failures to "),
    );
    expect(opts).toHaveLength(2);
  });

  it("shows empty state when no candidate panes", () => {
    const { getByRole } = render(() => (
      <PaneLinkCreateMenu
        open={true}
        currentPaneId="pane-runner"
        candidatePanes={[]}
        onClose={() => {}}
        onCreate={() => {}}
      />
    ));
    expect(getByRole("status").textContent).toContain("No other panes");
  });

  it("fires onCreate with this pane as child + picked pane as parent (failureFeedback)", async () => {
    const onCreate = vi.fn();
    const { getByLabelText } = render(() => (
      <PaneLinkCreateMenu
        open={true}
        currentPaneId="pane-runner"
        candidatePanes={candidates}
        onClose={() => {}}
        onCreate={onCreate}
      />
    ));
    await fireEvent.click(getByLabelText("Link failures to claude — ~/proj"));
    expect(onCreate).toHaveBeenCalledWith({
      parentPaneId: "pane-ai",
      childPaneId: "pane-runner",
      linkKind: "failureFeedback",
    });
  });

  it("closes on Escape and on close button", async () => {
    const onClose = vi.fn();
    const { getByLabelText } = render(() => (
      <PaneLinkCreateMenu
        open={true}
        currentPaneId="pane-runner"
        candidatePanes={candidates}
        onClose={onClose}
        onCreate={() => {}}
      />
    ));
    await fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
    await fireEvent.click(getByLabelText("Close link creation"));
    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it("has dialog role with aria-modal and descriptive aria-label", () => {
    const { getByRole } = render(() => (
      <PaneLinkCreateMenu
        open={true}
        currentPaneId="pane-runner"
        candidatePanes={candidates}
        onClose={() => {}}
        onCreate={() => {}}
      />
    ));
    const dialog = getByRole("dialog");
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(dialog.getAttribute("aria-label")).toContain("Link");
  });
});
