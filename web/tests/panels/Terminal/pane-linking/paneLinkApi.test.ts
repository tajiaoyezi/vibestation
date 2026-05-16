import { beforeEach, describe, expect, it, vi } from "vitest";
import type {
  PaneFailurePreviewRequest,
  PaneFailurePreviewResult,
  PaneLinkRequest,
  PaneLinkResult,
  PaneLinkSetEnabledRequest,
  PaneLinksListRequest,
  PaneLinksListResult,
  PaneUnlinkRequest,
  PaneUnlinkResult,
} from "../../../../src/bindings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import {
  linkPanes,
  listPaneLinks,
  previewFailurePrompt,
  setPaneLinkEnabled,
  unlinkPane,
} from "../../../../src/panels/Terminal/paneLinkApi";

const linkedResult: PaneLinkResult = {
  link: {
    id: "link-1",
    workspaceId: "ws-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    linkKind: "failureFeedback",
    enabled: true,
    fallbackMode: "structured",
    createdBy: "user",
    createdAt: 1760000000000,
    updatedAt: 1760000000000,
    lastTriggeredAt: 0,
  },
  alreadyExisted: false,
};

const previewResult: PaneFailurePreviewResult = {
  promptFragment: "Fix this failure",
  rawExcerpt: "error",
  parsedIssues: [],
  parserConfidence: 0,
  fallbackMode: "rawText",
  truncatedCount: 0,
  redactionCount: 0,
};

describe("paneLinkApi", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("wraps pane_link with the generated request payload", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(linkedResult);
    const req: PaneLinkRequest = {
      workspaceId: "ws-1",
      parentPaneId: "pane-ai",
      childPaneId: "pane-runner",
      linkKind: "failureFeedback",
    };

    await expect(linkPanes(req)).resolves.toBe(linkedResult);

    expect(invoke).toHaveBeenCalledWith("pane_link", { req });
  });

  it("wraps pane_unlink with the generated request payload", async () => {
    const result: PaneUnlinkResult = { linkId: "link-1", removed: true };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: PaneUnlinkRequest = { workspaceId: "ws-1", linkId: "link-1" };

    await expect(unlinkPane(req)).resolves.toBe(result);

    expect(invoke).toHaveBeenCalledWith("pane_unlink", { req });
  });

  it("wraps pane_links_list with the generated request payload", async () => {
    const result: PaneLinksListResult = { links: [linkedResult.link] };
    vi.mocked(invoke).mockResolvedValueOnce(result);
    const req: PaneLinksListRequest = { workspaceId: "ws-1" };

    await expect(listPaneLinks(req)).resolves.toBe(result);

    expect(invoke).toHaveBeenCalledWith("pane_links_list", { req });
  });

  it("wraps pane_links_set_enabled with the generated request payload", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(linkedResult);
    const req: PaneLinkSetEnabledRequest = {
      workspaceId: "ws-1",
      linkId: "link-1",
      enabled: false,
    };

    await expect(setPaneLinkEnabled(req)).resolves.toBe(linkedResult);

    expect(invoke).toHaveBeenCalledWith("pane_links_set_enabled", { req });
  });

  it("wraps pane_failure_preview_prompt with the generated request payload", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(previewResult);
    const req: PaneFailurePreviewRequest = {
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

    await expect(previewFailurePrompt(req)).resolves.toBe(previewResult);

    expect(invoke).toHaveBeenCalledWith("pane_failure_preview_prompt", {
      req,
    });
  });
});
