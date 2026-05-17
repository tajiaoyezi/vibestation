import { afterEach, describe, expect, it, vi } from "vitest";
import { render, fireEvent, screen, cleanup } from "@solidjs/testing-library";
import { RollbackConfirmDialog } from "../../../src/panels/SessionDetail/RollbackConfirmDialog";

// 每个测试后卸载已 mount 组件 · 移除其 document keydown listener
// 否则跨测试累积泄漏污染后续测试文件（reviewer-fix · 仿 sessionDetail.test.tsx 既有模式）
afterEach(() => cleanup());

describe("RollbackConfirmDialog", () => {
  it("renders with session ID prompt and commit count", () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={5}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    expect(screen.getByText("⚠ 确认回滚")).toBeInTheDocument();
    expect(screen.getByText(/5 个新 revert commit/)).toBeInTheDocument();
    expect(screen.getByText(/执行回滚（5 个 commit）/)).toBeInTheDocument();
  });

  it("execute button is disabled when input is empty", () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={5}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    const executeBtn = screen.getByText(/执行回滚/);
    expect(executeBtn).toBeDisabled();
  });

  it("execute button is disabled when input does not match session ID", async () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={5}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    const input = screen.getByLabelText("输入 session ID 以确认回滚");
    await fireEvent.input(input, { target: { value: "wrong-id" } });

    const executeBtn = screen.getByText(/执行回滚/);
    expect(executeBtn).toBeDisabled();
  });

  it("execute button is enabled when input matches session ID", async () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={5}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    const input = screen.getByLabelText("输入 session ID 以确认回滚");
    await fireEvent.input(input, { target: { value: "sess-42" } });

    const executeBtn = screen.getByText(/执行回滚/);
    expect(executeBtn).not.toBeDisabled();
  });

  it("calls onExecute when button clicked with correct ID", async () => {
    const onExecute = vi.fn();
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={5}
        onCancel={vi.fn()}
        onExecute={onExecute}
      />
    ));

    const input = screen.getByLabelText("输入 session ID 以确认回滚");
    await fireEvent.input(input, { target: { value: "sess-42" } });

    const executeBtn = screen.getByText(/执行回滚/);
    await fireEvent.click(executeBtn);
    expect(onExecute).toHaveBeenCalledTimes(1);
  });

  it("calls onExecute on Enter when ID matches", async () => {
    const onExecute = vi.fn();
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={3}
        onCancel={vi.fn()}
        onExecute={onExecute}
      />
    ));

    const input = screen.getByLabelText("输入 session ID 以确认回滚");
    await fireEvent.input(input, { target: { value: "sess-42" } });
    await fireEvent.keyDown(document, { key: "Enter" });

    expect(onExecute).toHaveBeenCalledTimes(1);
  });

  it("does not call onExecute on Enter when ID does not match", async () => {
    const onExecute = vi.fn();
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={3}
        onCancel={vi.fn()}
        onExecute={onExecute}
      />
    ));

    const input = screen.getByLabelText("输入 session ID 以确认回滚");
    await fireEvent.input(input, { target: { value: "wrong" } });
    await fireEvent.keyDown(document, { key: "Enter" });

    expect(onExecute).not.toHaveBeenCalled();
  });

  it("calls onCancel when Esc is pressed", async () => {
    const onCancel = vi.fn();
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={3}
        onCancel={onCancel}
        onExecute={vi.fn()}
      />
    ));

    await fireEvent.keyDown(document, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("execute button uses error color styling class", () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={3}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    const executeBtn = screen.getByText(/执行回滚/);
    expect(executeBtn.classList.contains("vs-rollback-execute-btn")).toBe(true);
  });

  it("has proper dialog role and aria-modal", () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={3}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
  });

  it("shows valid/invalid input visual state", async () => {
    render(() => (
      <RollbackConfirmDialog
        sessionId="sess-42"
        commitCount={3}
        onCancel={vi.fn()}
        onExecute={vi.fn()}
      />
    ));

    const input = screen.getByLabelText("输入 session ID 以确认回滚") as HTMLInputElement;

    await fireEvent.input(input, { target: { value: "wrong" } });
    expect(input.dataset.invalid).toBeDefined();

    await fireEvent.input(input, { target: { value: "sess-42" } });
    expect(input.dataset.valid).toBeDefined();
  });
});
