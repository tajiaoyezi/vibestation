import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, cleanup, fireEvent } from "@solidjs/testing-library";
import { PaneLinkChip } from "../../../../src/panels/Terminal/PaneLinkChip";
import { RunnerSourceBadge } from "../../../../src/panels/Terminal/RunnerSourceBadge";
import { FailureCallout } from "../../../../src/panels/Terminal/FailureCallout";
import { LinkManagePopover } from "../../../../src/panels/Terminal/LinkManagePopover";
import { PaneLinkErrorState } from "../../../../src/panels/Terminal/PaneLinkErrorState";
import type { PaneLinkError } from "../../../../src/bindings";
import type {
  PaneFailureCallout,
  PaneLinkView,
} from "../../../../src/stores/paneLinks";

function makeLink(overrides: Partial<PaneLinkView> = {}): PaneLinkView {
  return {
    id: "link-1",
    workspaceId: "ws-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    linkKind: "failureFeedback",
    status: "enabled",
    fallbackMode: "structured",
    createdBy: "user",
    createdAt: 1760000000000,
    updatedAt: 1760000000000,
    lastTriggeredAt: 0,
    ...overrides,
  };
}

function makeCallout(
  overrides: Partial<PaneFailureCallout> = {},
): PaneFailureCallout {
  return {
    workspaceId: "ws-1",
    linkId: "link-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    commandRunId: "run-1",
    failureHash: "hash-1",
    exitCode: 1,
    rawExcerpt: "error[E0425]: cannot find value `foo`",
    parserConfidence: 0.95,
    fallbackMode: "structured",
    parsedIssuesCount: 2,
    occurredAt: 1760000000200,
    ...overrides,
  };
}

describe("PaneLinkChip (D.1)", () => {
  beforeEach(cleanup);

  it("renders parent role chip with arrow direction", () => {
    const { getByRole } = render(() => (
      <PaneLinkChip link={makeLink()} role="parent" />
    ));
    const btn = getByRole("button");
    expect(btn).toBeInTheDocument();
    expect(btn.textContent).toContain("→");
    expect(btn.textContent).toContain("runner");
  });

  it("renders child role chip with reverse arrow", () => {
    const { getByRole } = render(() => (
      <PaneLinkChip link={makeLink()} role="child" />
    ));
    const btn = getByRole("button");
    expect(btn.textContent).toContain("←");
    expect(btn.textContent).toContain("agent");
  });

  it("shows paused badge when link is disabled", () => {
    const { container } = render(() => (
      <PaneLinkChip link={makeLink({ status: "disabled" })} role="parent" />
    ));
    const badge = container.querySelector(".vs-pane-link-chip-badge");
    expect(badge).toBeInTheDocument();
    expect(badge?.textContent).toBe("paused");
  });

  it("flows a stale link through as paused · #353 debt fix (distinct from active)", () => {
    const { container } = render(() => (
      <PaneLinkChip link={makeLink({ status: "stale" })} role="parent" />
    ));
    // stale must NOT render as active/"linked" — the store no longer collapses
    // it into enabled, so the chip correctly shows it paused.
    expect(
      container.querySelector(".vs-pane-link-chip-badge")?.textContent,
    ).toBe("paused");
    expect(
      container.querySelector(".vs-pane-link-chip.is-paused"),
    ).toBeInTheDocument();
  });

  it("fires onClick on click and keyboard Enter", async () => {
    const onClick = vi.fn();
    const { getByRole } = render(() => (
      <PaneLinkChip link={makeLink()} role="parent" onClick={onClick} />
    ));
    const btn = getByRole("button");

    await fireEvent.click(btn);
    expect(onClick).toHaveBeenCalledTimes(1);

    await fireEvent.keyDown(btn, { key: "Enter" });
    expect(onClick).toHaveBeenCalledTimes(2);
  });

  it("has correct aria-label for parent enabled", () => {
    const { getByRole } = render(() => (
      <PaneLinkChip link={makeLink()} role="parent" />
    ));
    expect(getByRole("button").getAttribute("aria-label")).toContain(
      "parent",
    );
    expect(getByRole("button").getAttribute("aria-label")).toContain(
      "linked",
    );
  });

  it("has correct aria-label for child paused", () => {
    const { getByRole } = render(() => (
      <PaneLinkChip
        link={makeLink({ status: "disabled" })}
        role="child"
      />
    ));
    expect(getByRole("button").getAttribute("aria-label")).toContain(
      "child",
    );
    expect(getByRole("button").getAttribute("aria-label")).toContain(
      "paused",
    );
  });
});

describe("RunnerSourceBadge (D.2)", () => {
  beforeEach(cleanup);

  it("renders when link is enabled", () => {
    const { getByRole } = render(() => (
      <RunnerSourceBadge link={makeLink()} />
    ));
    expect(getByRole("status")).toBeInTheDocument();
    expect(getByRole("status").textContent).toContain("Agent");
  });

  it("does not render when link is disabled", () => {
    const { queryByRole } = render(() => (
      <RunnerSourceBadge link={makeLink({ status: "disabled" })} />
    ));
    expect(queryByRole("status")).not.toBeInTheDocument();
  });

  it("shows custom parent label", () => {
    const { getByRole } = render(() => (
      <RunnerSourceBadge link={makeLink()} parentLabel="Claude" />
    ));
    expect(getByRole("status").textContent).toContain("Claude");
  });

  it("has descriptive aria-label", () => {
    const { getByRole } = render(() => (
      <RunnerSourceBadge link={makeLink()} />
    ));
    expect(getByRole("status").getAttribute("aria-label")).toContain(
      "Linked to Agent",
    );
  });
});

describe("FailureCallout (D.3)", () => {
  beforeEach(cleanup);

  it("renders failure summary with exit code and issue count", () => {
    const { getByRole } = render(() => (
      <FailureCallout callout={makeCallout()} />
    ));
    const alert = getByRole("alert");
    expect(alert.textContent).toContain("exit 1");
    expect(alert.textContent).toContain("2 issues");
  });

  it("renders signal label when exitCode is null", () => {
    const { getByRole } = render(() => (
      <FailureCallout callout={makeCallout({ exitCode: null })} />
    ));
    expect(getByRole("alert").textContent).toContain("signal");
  });

  it("fires onDismiss when dismiss button clicked", async () => {
    const onDismiss = vi.fn();
    const { getByLabelText } = render(() => (
      <FailureCallout callout={makeCallout()} onDismiss={onDismiss} />
    ));
    await fireEvent.click(getByLabelText("Dismiss failure notification"));
    expect(onDismiss).toHaveBeenCalledWith("run-1");
  });

  it("toggles raw output on view raw click", async () => {
    const { getByLabelText, container } = render(() => (
      <FailureCallout callout={makeCallout()} />
    ));

    expect(container.querySelector(".vs-failure-callout-raw")).toBeNull();

    await fireEvent.click(getByLabelText("View raw output"));
    const rawEl = container.querySelector(".vs-failure-callout-raw");
    expect(rawEl).toBeInTheDocument();
    expect(rawEl?.textContent).toContain("cannot find value");

    await fireEvent.click(getByLabelText("Hide raw output"));
    expect(container.querySelector(".vs-failure-callout-raw")).toBeNull();
  });

  it("fires onInsert when insert button clicked", async () => {
    const onInsert = vi.fn();
    const callout = makeCallout();
    const { getByLabelText } = render(() => (
      <FailureCallout callout={callout} onInsert={onInsert} />
    ));
    await fireEvent.click(
      getByLabelText("Insert failure details to agent pane"),
    );
    expect(onInsert).toHaveBeenCalledWith(callout);
  });

  it("has accessible aria-label on alert", () => {
    const { getByRole } = render(() => (
      <FailureCallout callout={makeCallout()} />
    ));
    expect(getByRole("alert").getAttribute("aria-label")).toContain(
      "Build failure",
    );
  });
});

describe("LinkManagePopover (D.4)", () => {
  beforeEach(cleanup);

  it("renders link list when open with links", () => {
    const links = [makeLink(), makeLink({ id: "link-2", childPaneId: "pane-b" })];
    const { getByRole, getAllByRole } = render(() => (
      <LinkManagePopover
        open={true}
        links={links}
        currentPaneId="pane-ai"
        onClose={() => {}}
      />
    ));
    expect(getByRole("dialog")).toBeInTheDocument();
    expect(getAllByRole("listitem")).toHaveLength(2);
  });

  it("shows empty state when no links", () => {
    const { getByRole } = render(() => (
      <LinkManagePopover
        open={true}
        links={[]}
        currentPaneId="pane-ai"
        onClose={() => {}}
      />
    ));
    expect(getByRole("status").textContent).toContain("No active links");
  });

  it("does not render when closed", () => {
    const { queryByRole } = render(() => (
      <LinkManagePopover
        open={false}
        links={[makeLink()]}
        currentPaneId="pane-ai"
        onClose={() => {}}
      />
    ));
    expect(queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("toggle switch has correct role and aria-checked", () => {
    const { getAllByRole } = render(() => (
      <LinkManagePopover
        open={true}
        links={[makeLink()]}
        currentPaneId="pane-ai"
        onClose={() => {}}
      />
    ));
    const switches = getAllByRole("switch");
    expect(switches).toHaveLength(1);
    expect(switches[0].getAttribute("aria-checked")).toBe("true");
  });

  it("surfaces lifecycle status in item label · stale ≠ disabled (#353 fix)", () => {
    const { getByRole } = render(() => (
      <LinkManagePopover
        open={true}
        links={[makeLink({ status: "stale" })]}
        currentPaneId="pane-ai"
        onClose={() => {}}
      />
    ));
    // the store no longer collapses stale into disabled — the popover item
    // label reflects the real §K.6 lifecycle status.
    expect(getByRole("listitem").getAttribute("aria-label")).toContain(
      "stale",
    );
    expect(getByRole("switch").getAttribute("aria-checked")).toBe("false");
  });

  it("fires onToggleEnabled on switch click", async () => {
    const onToggle = vi.fn();
    const { getAllByRole } = render(() => (
      <LinkManagePopover
        open={true}
        links={[makeLink()]}
        currentPaneId="pane-ai"
        onClose={() => {}}
        onToggleEnabled={onToggle}
      />
    ));
    await fireEvent.click(getAllByRole("switch")[0]);
    expect(onToggle).toHaveBeenCalledWith("link-1", false);
  });

  it("fires onUnlink on unlink button click", async () => {
    const onUnlink = vi.fn();
    const { getByLabelText } = render(() => (
      <LinkManagePopover
        open={true}
        links={[makeLink()]}
        currentPaneId="pane-ai"
        onClose={() => {}}
        onUnlink={onUnlink}
      />
    ));
    await fireEvent.click(getByLabelText("Unlink pane-runner"));
    expect(onUnlink).toHaveBeenCalledWith("link-1");
  });

  it("closes on Escape key", async () => {
    const onClose = vi.fn();
    render(() => (
      <LinkManagePopover
        open={true}
        links={[makeLink()]}
        currentPaneId="pane-ai"
        onClose={onClose}
      />
    ));
    await fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });

  it("has dialog role with aria-modal and aria-label", () => {
    const { getByRole } = render(() => (
      <LinkManagePopover
        open={true}
        links={[makeLink()]}
        currentPaneId="pane-ai"
        onClose={() => {}}
      />
    ));
    const dialog = getByRole("dialog");
    expect(dialog.getAttribute("aria-modal")).toBe("true");
    expect(dialog.getAttribute("aria-label")).toContain("Manage pane links");
  });
});

describe("PaneLinkErrorState (D.5)", () => {
  beforeEach(cleanup);

  it("renders error message for paneNotFound", () => {
    const error: PaneLinkError = { kind: "paneNotFound", detail: "pane-42" };
    const { getByRole } = render(() => (
      <PaneLinkErrorState error={error} />
    ));
    const alert = getByRole("alert");
    expect(alert.textContent).toContain("Pane not found: pane-42");
  });

  it("renders warning severity for parserTimeout", () => {
    const error: PaneLinkError = {
      kind: "parserTimeout",
      detail: "30s exceeded",
    };
    const { container } = render(() => (
      <PaneLinkErrorState error={error} />
    ));
    const errorEl = container.querySelector(".vs-pane-link-error-warning");
    expect(errorEl).toBeInTheDocument();
    expect(errorEl?.textContent).toContain("Parser timed out");
  });

  it("does not render when error is null", () => {
    const { queryByRole } = render(() => (
      <PaneLinkErrorState error={null} />
    ));
    expect(queryByRole("alert")).not.toBeInTheDocument();
  });

  it("fires onDismiss on dismiss click", async () => {
    const onDismiss = vi.fn();
    const error: PaneLinkError = { kind: "crossWorkspaceDenied" };
    const { getByLabelText } = render(() => (
      <PaneLinkErrorState error={error} onDismiss={onDismiss} />
    ));
    await fireEvent.click(getByLabelText("Dismiss error"));
    expect(onDismiss).toHaveBeenCalled();
  });

  it("has accessible aria-label on alert with severity", () => {
    const error: PaneLinkError = { kind: "dbError", detail: "disk full" };
    const { getByRole } = render(() => (
      <PaneLinkErrorState error={error} />
    ));
    const label = getByRole("alert").getAttribute("aria-label") ?? "";
    expect(label).toContain("Link error");
    expect(label).toContain("Database error");
  });

  it("renders all error kinds with text label (H.* not pure color)", () => {
    const errorKinds: PaneLinkError[] = [
      { kind: "crossWorkspaceDenied" },
      { kind: "invalidParentPaneType" },
      { kind: "invalidChildPaneType" },
      { kind: "paneNotFound", detail: "x" },
      { kind: "linkNotFound", detail: "x" },
      { kind: "parserUnavailable", detail: "x" },
      { kind: "parserTimeout", detail: "x" },
      { kind: "promptSanitizationFailed", detail: "x" },
      { kind: "dbError", detail: "x" },
      { kind: "unsupportedCliKind", detail: "x" },
    ];

    for (const error of errorKinds) {
      cleanup();
      const { getByRole } = render(() => (
        <PaneLinkErrorState error={error} />
      ));
      const message = getByRole("alert").querySelector(
        ".vs-pane-link-error-message",
      );
      expect(message?.textContent?.trim().length).toBeGreaterThan(0);
    }
  });
});
