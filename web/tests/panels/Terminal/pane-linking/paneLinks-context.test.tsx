import { beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@solidjs/testing-library";
import { createSignal, type Accessor } from "solid-js";
import type {
  PaneBuildFailedEvent,
  PaneFailurePreviewRequest,
  PaneFailurePreviewResult,
  PaneLinkErrorEvent,
  PaneLinkedEvent,
  PaneLinkRequest,
  PaneLinkResult,
  PaneLinkSetEnabledRequest,
  PaneLinksListRequest,
  PaneLinksListResult,
  PaneTriggerEvent,
  PaneUnlinkRequest,
  PaneUnlinkResult,
} from "../../../../src/bindings";
import type { PaneLinksContextValue } from "../../../../src/stores/paneLinks-context";

const { listenMock, unlistenFns, eventHandlers } = vi.hoisted(() => {
  type EventHandler = (event: { payload: unknown }) => void;
  const unlistenFns: Array<ReturnType<typeof vi.fn>> = [];
  const eventHandlers = new Map<string, EventHandler>();
  const listenMock = vi.fn(async (eventName: string, handler: EventHandler) => {
    const unlisten = vi.fn();
    unlistenFns.push(unlisten);
    eventHandlers.set(eventName, handler);
    return unlisten;
  });
  return { listenMock, unlistenFns, eventHandlers };
});

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

const apiMocks = vi.hoisted(() => ({
  linkPanes: vi.fn(),
  unlinkPane: vi.fn(),
  setPaneLinkEnabled: vi.fn(),
  listPaneLinks: vi.fn(),
  previewFailurePrompt: vi.fn(),
}));

vi.mock("../../../../src/panels/Terminal/paneLinkApi", () => apiMocks);

import {
  PaneLinksProvider,
  usePaneLinks,
} from "../../../../src/stores/paneLinks-context";

function linkedEvent(
  overrides: Partial<PaneLinkedEvent> = {},
): PaneLinkedEvent {
  return {
    workspaceId: "ws-1",
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
    workspaceId: "ws-1",
    linkId: "link-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    commandRunId: "run-1",
    exitCode: 1,
    rawExcerpt: "error",
    parsedIssues: [],
    parserConfidence: 0.8,
    fallbackMode: "rawText",
    occurredAt: 1760000000100,
    truncatedCount: 0,
    redactionCount: 0,
    ...overrides,
  };
}

async function flushMicrotasks() {
  await Promise.resolve();
  await Promise.resolve();
}

function mountProvider() {
  let ctx: PaneLinksContextValue | undefined;
  const Probe = () => {
    ctx = usePaneLinks();
    return <div data-testid="probe" />;
  };
  const rendered = render(() => (
    <PaneLinksProvider>
      <Probe />
    </PaneLinksProvider>
  ));
  return { ...rendered, ctx: () => ctx };
}

describe("PaneLinksProvider", () => {
  beforeEach(() => {
    cleanup();
    listenMock.mockClear();
    unlistenFns.length = 0;
    eventHandlers.clear();
    Object.values(apiMocks).forEach((mock) => mock.mockReset());
  });

  it("throws when usePaneLinks is called outside PaneLinksProvider", () => {
    const MissingProviderProbe = () => {
      usePaneLinks();
      return <div />;
    };

    expect(() => render(() => <MissingProviderProbe />)).toThrow(
      "usePaneLinks must be used within PaneLinksProvider",
    );
  });

  it("subscribes to pane link events and routes payloads into the store", async () => {
    const { ctx } = mountProvider();

    await flushMicrotasks();

    expect(listenMock.mock.calls.map(([eventName]) => eventName)).toEqual([
      "pane:linked",
      "pane:build-failed",
      "pane:link-error",
      "pane:trigger",
    ]);

    eventHandlers.get("pane:linked")?.({ payload: linkedEvent() });
    expect(ctx()?.store.linksByWorkspace["ws-1"]).toHaveLength(1);

    eventHandlers.get("pane:build-failed")?.({ payload: failedEvent() });
    expect(ctx()?.store.failureCalloutsByWorkspace["ws-1"]).toHaveLength(1);

    const errorEvent: PaneLinkErrorEvent = {
      workspaceId: "ws-1",
      error: { kind: "parserTimeout", detail: "timed out" },
    };
    eventHandlers.get("pane:link-error")?.({ payload: errorEvent });
    expect(ctx()?.store.lastError).toEqual(errorEvent.error);

    const triggerEvent: PaneTriggerEvent = {
      workspaceId: "ws-1",
      childPaneId: "pane-runner",
      commandRunId: "run-1",
      reason: "exitCode",
      exitCode: 1,
      command: "cargo test",
      occurredAt: 1760000000100,
    };
    expect(() =>
      eventHandlers.get("pane:trigger")?.({ payload: triggerEvent }),
    ).not.toThrow();
  });

  it("calls every event unlisten function during cleanup", async () => {
    mountProvider();
    await flushMicrotasks();

    cleanup();

    expect(unlistenFns).toHaveLength(4);
    for (const unlisten of unlistenFns) {
      expect(unlisten).toHaveBeenCalledTimes(1);
    }
  });

  it("exposes selectorsFor and delegates commands to paneLinkApi", async () => {
    const { ctx } = mountProvider();
    await flushMicrotasks();

    const [workspaceId]: [Accessor<string>] = createSignal("ws-1");
    ctx()?.store.applyLinkedEvent(linkedEvent());
    const selectors = ctx()?.selectorsFor(workspaceId);
    expect(selectors?.links()).toHaveLength(1);

    const linkReq: PaneLinkRequest = {
      workspaceId: "ws-1",
      parentPaneId: "pane-ai",
      childPaneId: "pane-runner",
      linkKind: "failureFeedback",
    };
    const linkResult: PaneLinkResult = {
      link: {
        id: "link-1",
        workspaceId: "ws-1",
        parentPaneId: "pane-ai",
        childPaneId: "pane-runner",
        linkKind: "failureFeedback",
        enabled: true,
        fallbackMode: "structured",
        createdBy: "user",
        createdAt: 1,
        updatedAt: 1,
        lastTriggeredAt: 0,
      },
      alreadyExisted: false,
    };
    apiMocks.linkPanes.mockResolvedValueOnce(linkResult);
    await expect(ctx()?.createLink(linkReq)).resolves.toBe(linkResult);
    expect(apiMocks.linkPanes).toHaveBeenCalledWith(linkReq);

    const unlinkReq: PaneUnlinkRequest = { workspaceId: "ws-1", linkId: "l" };
    const unlinkResult: PaneUnlinkResult = { linkId: "l", removed: true };
    apiMocks.unlinkPane.mockResolvedValueOnce(unlinkResult);
    await expect(ctx()?.unlink(unlinkReq)).resolves.toBe(unlinkResult);
    expect(apiMocks.unlinkPane).toHaveBeenCalledWith(unlinkReq);

    const setEnabledReq: PaneLinkSetEnabledRequest = {
      workspaceId: "ws-1",
      linkId: "l",
      enabled: true,
    };
    apiMocks.setPaneLinkEnabled.mockResolvedValueOnce(linkResult);
    await expect(ctx()?.setEnabled(setEnabledReq)).resolves.toBe(linkResult);
    expect(apiMocks.setPaneLinkEnabled).toHaveBeenCalledWith(setEnabledReq);

    const listReq: PaneLinksListRequest = { workspaceId: "ws-1" };
    const listResult: PaneLinksListResult = { links: [linkResult.link] };
    apiMocks.listPaneLinks.mockResolvedValueOnce(listResult);
    await expect(ctx()?.listLinks(listReq)).resolves.toBe(listResult);
    expect(apiMocks.listPaneLinks).toHaveBeenCalledWith(listReq);

    const previewReq: PaneFailurePreviewRequest = {
      workspaceId: "ws-1",
      childPaneId: "pane-runner",
      commandRunId: "run-1",
      exitCode: 1,
      command: "cargo test",
      cwd: "/repo",
      cliKind: "cargo",
      rawOutput: "error",
      parsedIssues: [],
    };
    const previewResult: PaneFailurePreviewResult = {
      promptFragment: "Fix this",
      rawExcerpt: "error",
      parsedIssues: [],
      parserConfidence: 0,
      fallbackMode: "rawText",
      truncatedCount: 0,
      redactionCount: 0,
    };
    apiMocks.previewFailurePrompt.mockResolvedValueOnce(previewResult);
    await expect(ctx()?.previewPrompt(previewReq)).resolves.toBe(previewResult);
    expect(apiMocks.previewFailurePrompt).toHaveBeenCalledWith(previewReq);
  });
});
