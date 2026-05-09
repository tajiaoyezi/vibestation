import { describe, it, expect } from "vitest";
import {
  detectResultToRecoveryState,
  normalizeOperation,
  payloadToRecoveryState,
  reduceRecoveries,
  type CrashRecoveryPayload,
  type RecoveryUiState,
} from "../../src/lib/crash-recovery";

describe("normalizeOperation", () => {
  it("maps known operations to ConflictOperation", () => {
    expect(normalizeOperation("rebase")).toBe("rebase");
    expect(normalizeOperation("merge")).toBe("merge");
    expect(normalizeOperation("cherrypick")).toBe("cherrypick");
  });

  it("falls back to rebase for unknown values (defensive)", () => {
    expect(normalizeOperation("")).toBe("rebase");
    expect(normalizeOperation("revert")).toBe("rebase");
    expect(normalizeOperation("PICK")).toBe("rebase");
  });
});

describe("payloadToRecoveryState", () => {
  const basePayload: CrashRecoveryPayload = {
    workspaceId: "ws-123",
    operation: "rebase",
    branch: "feat/foo",
    currentStep: 2,
    totalSteps: 5,
  };

  it("produces RecoveryUiState for in-progress payload", () => {
    const state = payloadToRecoveryState(basePayload);
    expect(state).toEqual({
      workspaceId: "ws-123",
      operation: "rebase",
      branch: "feat/foo",
      currentStep: 2,
      totalSteps: 5,
    });
  });

  it("returns null when operation is null (defensive · backend should not emit this)", () => {
    expect(
      payloadToRecoveryState({ ...basePayload, operation: null }),
    ).toBeNull();
  });

  it("preserves null branch (detached HEAD)", () => {
    const state = payloadToRecoveryState({ ...basePayload, branch: null });
    expect(state?.branch).toBeNull();
  });

  it("normalizes cherrypick operation", () => {
    const state = payloadToRecoveryState({
      ...basePayload,
      operation: "cherrypick",
    });
    expect(state?.operation).toBe("cherrypick");
  });
});

describe("detectResultToRecoveryState", () => {
  it("returns null when inProgress is false", () => {
    expect(
      detectResultToRecoveryState("ws-1", {
        inProgress: false,
        operation: null,
        branch: "main",
        currentStep: 0,
        totalSteps: 0,
      }),
    ).toBeNull();
  });

  it("returns null when inProgress is true but operation missing", () => {
    expect(
      detectResultToRecoveryState("ws-1", {
        inProgress: true,
        operation: null,
        branch: "main",
        currentStep: 0,
        totalSteps: 0,
      }),
    ).toBeNull();
  });

  it("produces state with provided workspaceId (not from result · result has no ws id)", () => {
    const state = detectResultToRecoveryState("ws-42", {
      inProgress: true,
      operation: "merge",
      branch: "develop",
      currentStep: 1,
      totalSteps: 3,
    });
    expect(state).toEqual({
      workspaceId: "ws-42",
      operation: "merge",
      branch: "develop",
      currentStep: 1,
      totalSteps: 3,
    });
  });
});

describe("reduceRecoveries", () => {
  const sample: RecoveryUiState = {
    workspaceId: "ws-A",
    operation: "rebase",
    branch: "feat/x",
    currentStep: 1,
    totalSteps: 4,
  };

  it("inserts new entry preserving other workspaces", () => {
    const prev = { "ws-B": { ...sample, workspaceId: "ws-B" } };
    const next = reduceRecoveries(prev, "ws-A", sample);
    expect(next).toEqual({
      "ws-A": sample,
      "ws-B": { ...sample, workspaceId: "ws-B" },
    });
  });

  it("overwrites existing entry by key", () => {
    const prev = { "ws-A": sample };
    const updated: RecoveryUiState = { ...sample, currentStep: 3 };
    const next = reduceRecoveries(prev, "ws-A", updated);
    expect(next["ws-A"].currentStep).toBe(3);
  });

  it("removes entry when next is null", () => {
    const prev = {
      "ws-A": sample,
      "ws-B": { ...sample, workspaceId: "ws-B" },
    };
    const next = reduceRecoveries(prev, "ws-A", null);
    expect(next).toEqual({ "ws-B": { ...sample, workspaceId: "ws-B" } });
  });

  it("returns same reference when removing key that doesn't exist (no churn)", () => {
    const prev = { "ws-B": sample };
    const next = reduceRecoveries(prev, "ws-A", null);
    expect(next).toBe(prev);
  });

  it("does not mutate input dict", () => {
    const prev = { "ws-A": sample };
    const before = JSON.stringify(prev);
    reduceRecoveries(prev, "ws-A", { ...sample, currentStep: 99 });
    expect(JSON.stringify(prev)).toBe(before);
  });

  it("removing the only entry returns empty object (not undefined)", () => {
    const prev = { "ws-A": sample };
    const next = reduceRecoveries(prev, "ws-A", null);
    expect(next).toEqual({});
  });
});
