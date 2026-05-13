import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@solidjs/testing-library";
import { DetachedPlaceholder } from "../../../src/panels/Terminal/DetachedPlaceholder";

describe("DetachedPlaceholder", () => {
  it("渲染 paneId + windowLabel", () => {
    render(() => (
      <DetachedPlaceholder
        paneId="pane-abc"
        windowLabel="pane-detach-123"
      />
    ));

    expect(screen.getByText("Pane detached")).toBeInTheDocument();
    expect(
      screen.getByText("Detached window is open. Close to bring back."),
    ).toBeInTheDocument();
  });

  it("点击触发 onFocusDetached", () => {
    const onFocus = vi.fn();
    render(() => (
      <DetachedPlaceholder
        paneId="pane-abc"
        windowLabel="pane-detach-123"
        onFocusDetached={onFocus}
      />
    ));

    const placeholder = screen.getByRole("button");
    fireEvent.click(placeholder);
    expect(onFocus).toHaveBeenCalledTimes(1);
  });

  it("Enter 键触发 onFocusDetached", () => {
    const onFocus = vi.fn();
    render(() => (
      <DetachedPlaceholder
        paneId="pane-abc"
        windowLabel="pane-detach-123"
        onFocusDetached={onFocus}
      />
    ));

    const placeholder = screen.getByRole("button");
    fireEvent.keyDown(placeholder, { key: "Enter" });
    expect(onFocus).toHaveBeenCalledTimes(1);
  });

  it("data 属性正确", () => {
    render(() => (
      <DetachedPlaceholder
        paneId="pane-abc"
        windowLabel="pane-detach-123"
      />
    ));

    const placeholder = screen.getByRole("button");
    expect(placeholder).toHaveAttribute("data-pane-id", "pane-abc");
    expect(placeholder).toHaveAttribute("data-window-label", "pane-detach-123");
  });
});
