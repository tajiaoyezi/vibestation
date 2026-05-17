import { afterEach, describe, expect, it, vi } from "vitest";
import { render, fireEvent, screen, cleanup } from "@solidjs/testing-library";
import { RollbackPreviewModal } from "../../../src/panels/SessionDetail/RollbackPreviewModal";
import type { RollbackPreview } from "../../../src/panels/SessionDetail/rollbackContract";

// 每个测试后卸载已 mount 组件 · 移除其 document keydown listener
// 否则跨测试累积泄漏污染后续测试文件（reviewer-fix · 仿 sessionDetail.test.tsx 既有模式）
afterEach(() => cleanup());

const highConfidencePreview: RollbackPreview = {
  session_id: "sess-42",
  commits: [
    { sha: "abc1234567890", message: "feat: refactor core", author: "AI", timestamp: 1700000000, confidence: 0.97, include: true, files_changed: 3 },
    { sha: "bcd2345678901", message: "fix: stage path bug", author: "AI", timestamp: 1700000100, confidence: 0.95, include: true, files_changed: 1 },
    { sha: "cde3456789012", message: "test: add stage tests", author: "AI", timestamp: 1700000200, confidence: 0.94, include: true, files_changed: 2 },
  ],
  total_files_changed: 6,
  total_insertions: 120,
  total_deletions: 30,
  has_low_confidence: false,
};

const mixedConfidencePreview: RollbackPreview = {
  session_id: "sess-42",
  commits: [
    { sha: "abc1234567890", message: "feat: refactor core", author: "AI", timestamp: 1700000000, confidence: 0.97, include: true, files_changed: 3 },
    { sha: "def4567890123", message: "chore: format Cargo.toml", author: "AI", timestamp: 1700000300, confidence: 0.72, include: false, files_changed: 1 },
    { sha: "efg5678901234", message: "refactor: extract enum", author: "AI", timestamp: 1700000400, confidence: 0.99, include: true, files_changed: 2 },
  ],
  total_files_changed: 6,
  total_insertions: 80,
  total_deletions: 20,
  has_low_confidence: true,
};

describe("RollbackPreviewModal", () => {
  it("renders all commits with confidence scores", () => {
    const onCancel = vi.fn();
    const onConfirm = vi.fn();
    render(() => (
      <RollbackPreviewModal
        preview={highConfidencePreview}
        onCancel={onCancel}
        onConfirm={onConfirm}
      />
    ));

    expect(screen.getByText(/abc1234/)).toBeInTheDocument();
    expect(screen.getByText(/bcd2345/)).toBeInTheDocument();
    expect(screen.getByText(/cde3456/)).toBeInTheDocument();
    expect(screen.getByText("置信度 0.97")).toBeInTheDocument();
    expect(screen.getByText("置信度 0.95")).toBeInTheDocument();
  });

  it("shows ✓ for high confidence and ⚠ for low confidence commits", () => {
    render(() => (
      <RollbackPreviewModal
        preview={mixedConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    ));

    const indicators = screen.getAllByText((_, el) =>
      el?.classList.contains("vs-rollback-commit-indicator") ?? false,
    );
    const texts = indicators.map((el) => el.textContent);
    expect(texts).toContain("✓");
    expect(texts).toContain("⚠");
  });

  it("low confidence commit is unchecked by default", () => {
    render(() => (
      <RollbackPreviewModal
        preview={mixedConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    ));

    const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
    const lowConfBox = checkboxes.find((cb) =>
      cb.getAttribute("aria-label")?.includes("def4567"),
    );
    expect(lowConfBox).toBeDefined();
    expect(lowConfBox!.checked).toBe(false);
  });

  it("user can toggle low confidence commit to include", async () => {
    const onConfirm = vi.fn();
    render(() => (
      <RollbackPreviewModal
        preview={mixedConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />
    ));

    const checkboxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
    const lowConfBox = checkboxes.find((cb) =>
      cb.getAttribute("aria-label")?.includes("def4567"),
    )!;
    expect(lowConfBox.checked).toBe(false);

    await fireEvent.click(lowConfBox);
    expect(lowConfBox.checked).toBe(true);

    const confirmBtn = screen.getByText(/确认回滚/);
    await fireEvent.click(confirmBtn);
    expect(onConfirm).toHaveBeenCalledWith(
      expect.arrayContaining(["def4567890123"]),
    );
  });

  it("shows low confidence warning banner when has_low_confidence is true", () => {
    render(() => (
      <RollbackPreviewModal
        preview={mixedConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    ));

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText(/1 个 commit 置信度/)).toBeInTheDocument();
  });

  it("does not show warning banner when all high confidence", () => {
    render(() => (
      <RollbackPreviewModal
        preview={highConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    ));

    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("calls onCancel when Esc is pressed", async () => {
    const onCancel = vi.fn();
    render(() => (
      <RollbackPreviewModal
        preview={highConfidencePreview}
        onCancel={onCancel}
        onConfirm={vi.fn()}
      />
    ));

    await fireEvent.keyDown(document, { key: "Escape" });
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("calls onConfirm with only included SHAs", async () => {
    const onConfirm = vi.fn();
    render(() => (
      <RollbackPreviewModal
        preview={mixedConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />
    ));

    const confirmBtn = screen.getByText(/确认回滚/);
    await fireEvent.click(confirmBtn);

    expect(onConfirm).toHaveBeenCalledWith(["abc1234567890", "efg5678901234"]);
  });

  it("displays file impact summary", () => {
    render(() => (
      <RollbackPreviewModal
        preview={highConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    ));

    expect(screen.getByText(/6 文件/)).toBeInTheDocument();
    expect(screen.getByText(/\+120/)).toBeInTheDocument();
    expect(screen.getByText(/-30/)).toBeInTheDocument();
  });

  it("has proper dialog role and aria-modal", () => {
    render(() => (
      <RollbackPreviewModal
        preview={highConfidencePreview}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    ));

    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveAttribute("aria-modal", "true");
  });
});
