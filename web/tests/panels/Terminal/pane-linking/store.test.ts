import { createRoot, createSignal } from "solid-js";
import { describe, expect, it } from "vitest";
import type { PaneLinkedEvent } from "../../../../src/bindings";
import {
  type PaneBuildFailedEvent,
  FAILURE_BACKLOG_CAP,
  createPaneLinksStore,
} from "../../../../src/stores/paneLinks";

function linkedEvent(
  overrides: Partial<PaneLinkedEvent> = {},
): PaneLinkedEvent {
  return {
    workspaceId: "workspace-a",
    linkId: "link-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    linkKind: "failureFeedback",
    status: "enabled",
    updatedAt: 1760000000000,
    ...overrides,
  };
}

function failedEvent(
  overrides: Partial<PaneBuildFailedEvent> = {},
): PaneBuildFailedEvent {
  return {
    workspaceId: "workspace-a",
    linkId: "link-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    commandRunId: "run-1",
    exitCode: 101,
    rawExcerpt: "error[E0425]: cannot find value `foo` in this scope",
    parsedIssues: [
      {
        severity: "error",
        file: "src/lib.rs",
        line: 42,
        column: 13,
        message: "cannot find value `foo` in this scope",
      },
    ],
    parserConfidence: 0.98,
    fallbackMode: "structured",
    occurredAt: 1760000000200,
    ...overrides,
  };
}

describe("paneLinks store", () => {
  it("applies linked lifecycle updates (B.5/B.7)", () => {
    const store = createPaneLinksStore();
    store.applyLinkedEvent(linkedEvent());
    expect(store.linksByWorkspace["workspace-a"]).toHaveLength(1);
    expect(store.linksByWorkspace["workspace-a"][0]?.enabled).toBe(true);

    store.applyLinkedEvent(linkedEvent({ status: "disabled", updatedAt: 2 }));
    expect(store.linksByWorkspace["workspace-a"][0]?.enabled).toBe(false);

    store.applyLinkedEvent(linkedEvent({ status: "removed", updatedAt: 3 }));
    expect(store.linksByWorkspace["workspace-a"]).toHaveLength(0);
  });

  it("scopes selectors by workspace id and isolates A/B state (F.1/G.3)", () => {
    createRoot((dispose) => {
      const store = createPaneLinksStore();
      const [workspaceId, setWorkspaceId] = createSignal<string>("workspace-a");
      const selectors = store.createWorkspaceScopedSelectors(workspaceId);

      store.applyLinkedEvent(linkedEvent({ workspaceId: "workspace-a" }));
      store.applyLinkedEvent(
        linkedEvent({
          workspaceId: "workspace-b",
          linkId: "link-b",
          childPaneId: "pane-b",
        }),
      );

      expect(selectors.links()).toHaveLength(1);
      expect(selectors.links()[0]?.workspaceId).toBe("workspace-a");

      setWorkspaceId("workspace-b");
      expect(selectors.links()).toHaveLength(1);
      expect(selectors.links()[0]?.workspaceId).toBe("workspace-b");

      dispose();
    });
  });

  it("marks all links stale when child pane closes (F.2)", () => {
    const store = createPaneLinksStore();
    store.applyLinkedEvent(
      linkedEvent({ linkId: "link-a", childPaneId: "pane-x" }),
    );
    store.applyLinkedEvent(
      linkedEvent({ linkId: "link-b", childPaneId: "pane-y" }),
    );
    store.applyLinkedEvent(
      linkedEvent({
        linkId: "link-c",
        workspaceId: "workspace-b",
        childPaneId: "pane-x",
      }),
    );

    store.markChildStale("workspace-a", "pane-x");

    expect(
      store.linksByWorkspace["workspace-a"].find(
        (link) => link.id === "link-a",
      )?.enabled,
    ).toBe(false);
    expect(
      store.linksByWorkspace["workspace-a"].find(
        (link) => link.id === "link-b",
      )?.enabled,
    ).toBe(true);
    expect(
      store.linksByWorkspace["workspace-b"].find(
        (link) => link.id === "link-c",
      )?.enabled,
    ).toBe(true);
  });

  it("deduplicates repeated failures by commandRunId + failureHash (D.6)", () => {
    const store = createPaneLinksStore();

    store.applyBuildFailedEvent(failedEvent());
    store.applyBuildFailedEvent(failedEvent());

    expect(store.failureCalloutsByWorkspace["workspace-a"]).toHaveLength(1);
  });

  it("keeps failure backlog capped at five callouts (G.5)", () => {
    const store = createPaneLinksStore();
    for (let index = 1; index <= FAILURE_BACKLOG_CAP + 2; index += 1) {
      store.applyBuildFailedEvent(
        failedEvent({
          commandRunId: `run-${index}`,
          occurredAt: 1760000000200 + index,
          rawExcerpt: `error-${index}`,
        }),
      );
    }

    const callouts = store.failureCalloutsByWorkspace["workspace-a"];
    expect(callouts).toHaveLength(FAILURE_BACKLOG_CAP);
    expect(callouts[0]?.commandRunId).toBe("run-3");
    expect(callouts[FAILURE_BACKLOG_CAP - 1]?.commandRunId).toBe("run-7");
  });

  it("activeLinks selector filters only enabled links", () => {
    createRoot((dispose) => {
      const store = createPaneLinksStore();
      const [workspaceId] = createSignal<string>("workspace-a");
      const selectors = store.createWorkspaceScopedSelectors(workspaceId);

      store.applyLinkedEvent(linkedEvent({ linkId: "active-1" }));
      store.applyLinkedEvent(
        linkedEvent({ linkId: "disabled-1", status: "disabled" }),
      );

      expect(selectors.activeLinks()).toHaveLength(1);
      expect(selectors.activeLinks()[0]?.id).toBe("active-1");

      dispose();
    });
  });

  it("linkForPane finds link by parent or child pane id", () => {
    createRoot((dispose) => {
      const store = createPaneLinksStore();
      const [workspaceId] = createSignal<string>("workspace-a");
      const selectors = store.createWorkspaceScopedSelectors(workspaceId);

      store.applyLinkedEvent(linkedEvent());

      const byParent = selectors.linkForPane("pane-ai");
      expect(byParent?.id).toBe("link-1");

      const byChild = selectors.linkForPane("pane-runner");
      expect(byChild?.id).toBe("link-1");

      const noMatch = selectors.linkForPane("nonexistent");
      expect(noMatch).toBeUndefined();

      dispose();
    });
  });

  it("dismissCallout removes specific callout by commandRunId", () => {
    const store = createPaneLinksStore();

    store.applyBuildFailedEvent(failedEvent({ commandRunId: "run-a" }));
    store.applyBuildFailedEvent(
      failedEvent({
        commandRunId: "run-b",
        rawExcerpt: "different error",
      }),
    );

    expect(store.failureCalloutsByWorkspace["workspace-a"]).toHaveLength(2);

    store.dismissCallout("workspace-a", "run-a");
    expect(store.failureCalloutsByWorkspace["workspace-a"]).toHaveLength(1);
    expect(
      store.failureCalloutsByWorkspace["workspace-a"][0]?.commandRunId,
    ).toBe("run-b");
  });

  it("applyErrorEvent stores last error and clearError resets it", () => {
    const store = createPaneLinksStore();
    expect(store.lastError).toBeNull();

    store.applyErrorEvent({
      workspaceId: "workspace-a",
      error: { kind: "paneNotFound", detail: "pane-123" },
    });
    expect(store.lastError).toEqual({
      kind: "paneNotFound",
      detail: "pane-123",
    });

    store.clearError();
    expect(store.lastError).toBeNull();
  });
});
