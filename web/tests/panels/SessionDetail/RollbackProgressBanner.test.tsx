import { describe, expect, it, vi } from "vitest";
import { render, fireEvent, screen } from "@solidjs/testing-library";
import { RollbackProgressBanner } from "../../../src/panels/SessionDetail/RollbackProgressBanner";
import type { RollbackProgress } from "../../../src/panels/SessionDetail/rollbackContract";

describe("RollbackProgressBanner", () => {
  const baseProgress: RollbackProgress = {
    done: 3,
    total: 7,
    current_sha: "bcd2345678901",
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
        progress={{ done: 0, total: 0, current_sha: "", status: "" }}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText("0%")).toBeInTheDocument();
  });

  it("renders 100% when done equals total", () => {
    render(() => (
      <RollbackProgressBanner
        sessionId="sess-42"
        progress={{ done: 5, total: 5, current_sha: "xyz", status: "done" }}
        onAbort={vi.fn()}
      />
    ));

    expect(screen.getByText("100%")).toBeInTheDocument();
  });
});
