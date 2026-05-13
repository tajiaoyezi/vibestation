import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  listTerminals,
  previewEnv,
  launchTerminal,
} from "../../src/lib/external-term";

describe("external-term.ts · IPC wrapper", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("listTerminals 调用正确 IPC command", async () => {
    const invokeSpy = vi.fn().mockResolvedValue([]);
    vi.stubGlobal("__TAURI_INTERNALS__", {
      transformCallback: vi.fn(() => 42),
      invoke: invokeSpy,
    });

    await listTerminals();
    expect(invokeSpy).toHaveBeenCalledWith("external_term_list", {}, undefined);
  });

  it("previewEnv 传 paneId", async () => {
    const invokeSpy = vi.fn().mockResolvedValue({
      visibleEntries: [],
      filteredCount: 0,
    });
    vi.stubGlobal("__TAURI_INTERNALS__", {
      transformCallback: vi.fn(() => 42),
      invoke: invokeSpy,
    });

    await previewEnv("pane-123");
    expect(invokeSpy).toHaveBeenCalledWith(
      "external_term_preview_env",
      { paneId: "pane-123" },
      undefined,
    );
  });

  it("launchTerminal 传 request", async () => {
    const invokeSpy = vi.fn().mockResolvedValue({ success: true });
    vi.stubGlobal("__TAURI_INTERNALS__", {
      transformCallback: vi.fn(() => 42),
      invoke: invokeSpy,
    });

    const request = {
      terminalId: "ghostty",
      paneId: "pane-123",
      overrideEnv: null,
    };
    await launchTerminal(request);
    expect(invokeSpy).toHaveBeenCalledWith(
      "external_term_launch",
      { request },
      undefined,
    );
  });

  it("launchTerminal 失败时 throw", async () => {
    const invokeSpy = vi.fn().mockRejectedValue(new Error("IPC failed"));
    vi.stubGlobal("__TAURI_INTERNALS__", {
      transformCallback: vi.fn(() => 42),
      invoke: invokeSpy,
    });

    await expect(
      launchTerminal({
        terminalId: "ghostty",
        paneId: "pane-123",
        overrideEnv: null,
      }),
    ).rejects.toThrow("IPC failed");
  });

  it("listTerminals 返回空数组时行为正常", async () => {
    const invokeSpy = vi.fn().mockResolvedValue([]);
    vi.stubGlobal("__TAURI_INTERNALS__", {
      transformCallback: vi.fn(() => 42),
      invoke: invokeSpy,
    });

    const result = await listTerminals();
    expect(result).toEqual([]);
  });
});
