import { describe, it, expect } from "vitest";
import {
  canResumeRollback,
  payloadToRollbackRecovery,
  reduceRollbackRecoveries,
  type RollbackRecoveryPayload,
  type RollbackRecoveryUiState,
} from "../../src/lib/rollback-recovery";

// MVP-20 Phase D · rollback crash recovery 纯状态机单测
// （镜像 crash-recovery.test.ts · 与 SolidJS / Tauri runtime 解耦）。

describe("canResumeRollback", () => {
  // conflict_paused 可经 rollback_execute resume；raw in_progress 崩溃
  // 后端返回 InProgress 错误（仅可 abort）· 故只有 paused 提供「继续」。
  it("only conflict_paused is resumable", () => {
    expect(canResumeRollback("conflict_paused")).toBe(true);
    expect(canResumeRollback("in_progress")).toBe(false);
    expect(canResumeRollback("completed")).toBe(false);
    expect(canResumeRollback("aborted")).toBe(false);
    expect(canResumeRollback("idle")).toBe(false);
  });
});

describe("payloadToRollbackRecovery", () => {
  const base: RollbackRecoveryPayload = {
    workspaceId: "ws-1",
    sessionId: "sess-1",
    status: "in_progress",
    currentIdx: 1,
    total: 3,
    currentSha: "abc1234",
  };

  it("produces UI state for in_progress payload (abort-only)", () => {
    const state = payloadToRollbackRecovery(base);
    expect(state).toEqual({
      workspaceId: "ws-1",
      sessionId: "sess-1",
      status: "in_progress",
      currentIdx: 1,
      total: 3,
      currentSha: "abc1234",
      canResume: false,
    });
  });

  it("marks conflict_paused as resumable", () => {
    const state = payloadToRollbackRecovery({
      ...base,
      status: "conflict_paused",
    });
    expect(state?.canResume).toBe(true);
  });

  it("returns null for terminal/idle status (defensive · backend should not emit)", () => {
    expect(
      payloadToRollbackRecovery({ ...base, status: "completed" }),
    ).toBeNull();
    expect(payloadToRollbackRecovery({ ...base, status: "aborted" })).toBeNull();
    expect(payloadToRollbackRecovery({ ...base, status: "idle" })).toBeNull();
  });

  it("preserves null currentSha", () => {
    const state = payloadToRollbackRecovery({ ...base, currentSha: null });
    expect(state?.currentSha).toBeNull();
  });
});

describe("reduceRollbackRecoveries", () => {
  const sample: RollbackRecoveryUiState = {
    workspaceId: "ws-A",
    sessionId: "sess-A",
    status: "in_progress",
    currentIdx: 0,
    total: 2,
    currentSha: null,
    canResume: false,
  };

  it("inserts new entry preserving other sessions", () => {
    const prev = { "sess-B": { ...sample, sessionId: "sess-B" } };
    const next = reduceRollbackRecoveries(prev, "sess-A", sample);
    expect(next).toEqual({
      "sess-A": sample,
      "sess-B": { ...sample, sessionId: "sess-B" },
    });
  });

  it("overwrites existing entry by session key", () => {
    const prev = { "sess-A": sample };
    const updated: RollbackRecoveryUiState = { ...sample, currentIdx: 1 };
    const next = reduceRollbackRecoveries(prev, "sess-A", updated);
    expect(next["sess-A"].currentIdx).toBe(1);
  });

  it("removes entry when next is null", () => {
    const prev = {
      "sess-A": sample,
      "sess-B": { ...sample, sessionId: "sess-B" },
    };
    const next = reduceRollbackRecoveries(prev, "sess-A", null);
    expect(next).toEqual({ "sess-B": { ...sample, sessionId: "sess-B" } });
  });

  it("returns same reference when removing absent key (no reactive churn)", () => {
    const prev = { "sess-B": sample };
    const next = reduceRollbackRecoveries(prev, "sess-A", null);
    expect(next).toBe(prev);
  });

  it("does not mutate input dict", () => {
    const prev = { "sess-A": sample };
    const before = JSON.stringify(prev);
    reduceRollbackRecoveries(prev, "sess-A", { ...sample, currentIdx: 9 });
    expect(JSON.stringify(prev)).toBe(before);
  });

  it("removing the only entry returns empty object (not undefined)", () => {
    const prev = { "sess-A": sample };
    const next = reduceRollbackRecoveries(prev, "sess-A", null);
    expect(next).toEqual({});
  });
});
