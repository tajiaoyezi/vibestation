import { afterEach, describe, expect, it, vi } from "vitest";
import { render, fireEvent, screen, cleanup } from "@solidjs/testing-library";
import { RollbackProgressBanner } from "../../../src/panels/SessionDetail/RollbackProgressBanner";
import type { RollbackProgress } from "../../../src/bindings";

// 每个测试后卸载已 mount 组件 · 移除其 document keydown listener
// 否则跨测试累积泄漏污染后续测试文件（reviewer-fix · 仿 sessionDetail.test.tsx 既有模式）
afterEach(() => cleanup());

describe("RollbackProgressBanner", () => {
  const baseProgress: RollbackProgress = {
    done: 3,
    total: 7,
    currentSha: "bcd2345678901",
    status: 'fix: 修复 stage 路径异常',
  };

  it("renders session ID and progress fraction", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={baseProgress}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText(/Session #sess-42/)).toBeInTheDocument();
    expect(screen.getByText(/3\/\s*7 已完成/)).toBeInTheDocument();
  });

  it("renders percentage", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={baseProgress}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText("43%")).toBeInTheDocument();
  });

  it("shows current commit SHA", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={baseProgress}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText(/bcd2345/)).toBeInTheDocument();
  });

  it("calls onAbort when cancel button clicked", async () => {
    const onAbort = vi.fn();
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={baseProgress}
        onAbort={onAbort}
      />
    ));

    const abortBtn = screen.getByLabelText("取消回滚");
    await fireEvent.click(abortBtn);
    expect(onAbort).toHaveBeenCalledTimes(1);
  });

  it("has role=status and aria-live=polite", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={baseProgress}
        onAbort={vi.fn()}
      />
    ));

    const banner = screen.getByRole("status");
    expect(banner).toHaveAttribute("aria-live", "polite");
  });

  it("renders 0% for zero total", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={{ done: 0, total: 0, currentSha: "", status: "" }}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText("0%")).toBeInTheDocument();
  });

  it("renders 100% when done equals total", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={{ done: 5, total: 5, currentSha: "xyz", status: "done" }}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText("100%")).toBeInTheDocument();
  });
});
