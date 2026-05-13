import { describe, it, expect, vi } from "vitest";
import {
  createPaneContextMenu,
  PaneContextMenuOverlay,
} from "../../../src/panels/Terminal/PaneContextMenu";
import { render, screen, fireEvent } from "@solidjs/testing-library";

describe("PaneContextMenu · createPaneContextMenu", () => {
  it("初始状态不可见", () => {
    const menu = createPaneContextMenu();
    expect(menu.state.visible).toBe(false);
    expect(menu.state.items).toEqual([]);
  });

  it("show 后状态可见", () => {
    const menu = createPaneContextMenu();
    menu.show(100, 200, [
      { id: "a", label: "Item A", onClick: () => {} },
    ]);
    expect(menu.state.visible).toBe(true);
    expect(menu.state.x).toBe(100);
    expect(menu.state.y).toBe(200);
    expect(menu.state.items.length).toBe(1);
  });

  it("hide 后状态不可见", () => {
    const menu = createPaneContextMenu();
    menu.show(0, 0, [{ id: "a", label: "A", onClick: () => {} }]);
    menu.hide();
    expect(menu.state.visible).toBe(false);
  });
});

describe("PaneContextMenuOverlay · 渲染", () => {
  it("不可见时不渲染菜单", () => {
    const menu = createPaneContextMenu();
    render(() => (
      <PaneContextMenuOverlay state={menu.state} onHide={() => {}} />
    ));
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("可见时渲染菜单项", () => {
    const menu = createPaneContextMenu();
    menu.show(0, 0, [
      { id: "pop", label: "Pop to External", shortcut: "⌘⇧O", onClick: () => {} },
      { id: "sep", label: "", separator: true, onClick: () => {} },
      { id: "detach", label: "Detach Pane", shortcut: "⌘⇧D", onClick: () => {} },
    ]);

    render(() => (
      <PaneContextMenuOverlay state={menu.state} onHide={() => {}} />
    ));

    expect(screen.getByText("Pop to External")).toBeInTheDocument();
    expect(screen.getByText("⌘⇧O")).toBeInTheDocument();
    expect(screen.getByText("Detach Pane")).toBeInTheDocument();
    expect(screen.getByText("⌘⇧D")).toBeInTheDocument();
  });

  it("点击菜单项触发 onClick + onHide", () => {
    const menu = createPaneContextMenu();
    const onClick = vi.fn();
    const onHide = vi.fn();

    menu.show(0, 0, [
      { id: "pop", label: "Pop to External", onClick },
    ]);

    render(() => (
      <PaneContextMenuOverlay state={menu.state} onHide={onHide} />
    ));

    const item = screen.getByText("Pop to External");
    fireEvent.click(item);
    expect(onClick).toHaveBeenCalledTimes(1);
    expect(onHide).toHaveBeenCalledTimes(1);
  });

  it("disabled 项不触发 onClick", () => {
    const menu = createPaneContextMenu();
    const onClick = vi.fn();
    const onHide = vi.fn();

    menu.show(0, 0, [
      { id: "pop", label: "Pop to External", disabled: true, onClick },
    ]);

    render(() => (
      <PaneContextMenuOverlay state={menu.state} onHide={onHide} />
    ));

    const item = screen.getByText("Pop to External");
    fireEvent.click(item);
    expect(onClick).not.toHaveBeenCalled();
    expect(onHide).not.toHaveBeenCalled();
  });
});
