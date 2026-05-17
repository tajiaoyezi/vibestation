import { beforeEach, describe, expect, it, vi } from "vitest";
import type { RollbackPreview, RollbackAbortResult, RollbackStatus, RollbackProgress } from "../../../src/panels/SessionDetail/rollbackContract";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  rollbackPreview,
  rollbackExecute,
  rollbackAbort,
  rollbackStatus,
  onRollbackProgress,
  onRollbackDone,
  onRollbackAborted,
  onRollbackConflict,
} from "../../../src/panels/SessionDetail/rollbackApi";

describe("rollbackApi · invoke wrappers", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("wraps rollback_preview", async () => {
    const result: RollbackPreview = {
      session_id: "sess-1",
      commits: [],
      total_files_changed: 0,
      total_insertions: 0,
      total_deletions: 0,
      has_low_confidence: false,
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);

    await expect(rollbackPreview("sess-1")).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("rollback_preview", {
      req: { session_id: "sess-1" },
    });
  });

  it("wraps rollback_execute", async () => {
    const result: RollbackProgress = { done: 0, total: 2, current_sha: "a", status: "starting" };
    vi.mocked(invoke).mockResolvedValueOnce(result);

    await expect(rollbackExecute("sess-1", ["sha1", "sha2"])).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("rollback_execute", {
      req: { session_id: "sess-1", include_shas: ["sha1", "sha2"] },
    });
  });

  it("wraps rollback_abort", async () => {
    const result: RollbackAbortResult = { success: true, head_sha: "abc", error: null };
    vi.mocked(invoke).mockResolvedValueOnce(result);

    await expect(rollbackAbort("sess-1")).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("rollback_abort", {
      req: { session_id: "sess-1" },
    });
  });

  it("wraps rollback_status", async () => {
    const result: RollbackStatus = {
      session_id: "sess-1",
      status: "idle",
      current_idx: 0,
      total: 0,
      current_sha: null,
    };
    vi.mocked(invoke).mockResolvedValueOnce(result);

    await expect(rollbackStatus("sess-1")).resolves.toBe(result);
    expect(invoke).toHaveBeenCalledWith("rollback_status", {
      req: { session_id: "sess-1" },
    });
  });
});

describe("rollbackApi · event listeners", () => {
  beforeEach(() => {
    vi.mocked(listen).mockReset();
  });

  it("onRollbackProgress listens to git:rollback-progress", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    const unlisten = await onRollbackProgress(cb);
    expect(listen).toHaveBeenCalledWith("git:rollback-progress", expect.any(Function));
    expect(unlisten).toBe(mockUnlisten);

    const wrapper = vi.mocked(listen).mock.calls[0][1];
    const payload: RollbackProgress = { done: 1, total: 3, current_sha: "abc", status: "ok" };
    (wrapper as (e: { payload: RollbackProgress }) => void)({ payload });
    expect(cb).toHaveBeenCalledWith(payload);
  });

  it("onRollbackDone listens to git:rollback-done", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    await onRollbackDone(cb);
    expect(listen).toHaveBeenCalledWith("git:rollback-done", expect.any(Function));
  });

  it("onRollbackAborted listens to git:rollback-aborted", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    await onRollbackAborted(cb);
    expect(listen).toHaveBeenCalledWith("git:rollback-aborted", expect.any(Function));
  });

  it("onRollbackConflict listens to git:rollback-conflict", async () => {
    const mockUnlisten = vi.fn();
    vi.mocked(listen).mockResolvedValueOnce(mockUnlisten);
    const cb = vi.fn();

    await onRollbackConflict(cb);
    expect(listen).toHaveBeenCalledWith("git:rollback-conflict", expect.any(Function));
  });
});
