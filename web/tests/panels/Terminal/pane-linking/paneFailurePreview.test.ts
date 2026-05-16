import { describe, it, expect } from "vitest";
import { buildPreviewRequest } from "../../../../src/panels/Terminal/paneFailurePreview";
import type { PaneFailureCallout } from "../../../../src/stores/paneLinks";

function callout(
  overrides: Partial<PaneFailureCallout> = {},
): PaneFailureCallout {
  return {
    workspaceId: "ws-1",
    linkId: "link-1",
    parentPaneId: "pane-ai",
    childPaneId: "pane-runner",
    commandRunId: "run-7",
    failureHash: "hash-7",
    exitCode: 101,
    rawExcerpt: "error[E0425]: cannot find value `foo`",
    parserConfidence: 0.9,
    fallbackMode: "structured",
    parsedIssuesCount: 3,
    occurredAt: 1760000000200,
    ...overrides,
  };
}

describe("buildPreviewRequest", () => {
  it("maps callout fields the summary actually carries", () => {
    const req = buildPreviewRequest(callout());
    expect(req.workspaceId).toBe("ws-1");
    expect(req.childPaneId).toBe("pane-runner");
    expect(req.commandRunId).toBe("run-7");
    expect(req.exitCode).toBe(101);
    expect(req.rawOutput).toBe("error[E0425]: cannot find value `foo`");
  });

  it("preserves null exitCode (signal-terminated)", () => {
    expect(
      buildPreviewRequest(callout({ exitCode: null })).exitCode,
    ).toBeNull();
  });

  it("leaves command/cwd/cliKind empty + parsedIssues [] (callout summary is minimal · §G.3 privacy · backend re-derives from rawOutput · documented degradation, not fabricated)", () => {
    const req = buildPreviewRequest(callout());
    expect(req.command).toBe("");
    expect(req.cwd).toBe("");
    expect(req.cliKind).toBe("");
    expect(req.parsedIssues).toEqual([]);
  });
});
