import { createRoot, createSignal } from "solid-js";
import { describe, expect, it } from "vitest";
import type {
  PaneBuildFailedEvent,
  PaneLinkedEvent,
} from "../../../../src/panels/Terminal/paneLinkContract";
import {
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
    expect(store.linksByWorkspace["workspace-a"][0]?.status).toBe("enabled");

    store.applyLinkedEvent(linkedEvent({ status: "disabled", updatedAt: 2 }));
    expect(store.linksByWorkspace["workspace-a"][0]?.status).toBe("disabled");

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
        (link) => link.linkId === "link-a",
      )?.status,
    ).toBe("stale");
    expect(
      store.linksByWorkspace["workspace-a"].find(
        (link) => link.linkId === "link-b",
      )?.status,
    ).toBe("enabled");
    expect(
      store.linksByWorkspace["workspace-b"].find(
        (link) => link.linkId === "link-c",
      )?.status,
    ).toBe("enabled");
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
});
