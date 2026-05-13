import { Show, For, type Component } from "solid-js";
import { createStore } from "solid-js/store";

interface MenuItem {
  id: string;
  label: string;
  shortcut?: string;
  disabled?: boolean;
  separator?: boolean;
  onClick: () => void;
}

interface PaneContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  items: MenuItem[];
}

/**
 * MVP-17 Phase C · Pane 右键菜单（前端 overlay 版）
 *
 * 替代 backend native menu_show_terminal · 支持动态添加 "Pop to External" / "Detach Pane"
 *
 * 使用：
 *   const menu = createPaneContextMenu();
 *   // 右键时
 *   menu.show(e.clientX, e.clientY, [
 *     { id: "pop-external", label: "Pop to External…", shortcut: "⌘⇧O", onClick: ... },
 *     { id: "detach", label: "Detach Pane", shortcut: "⌘⇧D", onClick: ... },
 *     { id: "sep1", label: "", separator: true, onClick: () => {} },
 *     { id: "close", label: "Close", onClick: ... },
 *   ]);
 */
export function createPaneContextMenu() {
  const [state, setState] = createStore<PaneContextMenuState>({
    visible: false,
    x: 0,
    y: 0,
    items: [],
  });

  const show = (x: number, y: number, items: MenuItem[]): void => {
    setState({ visible: true, x, y, items });
  };

  const hide = (): void => {
    setState("visible", false);
  };

  return { state, show, hide };
}

export const PaneContextMenuOverlay: Component<{
  state: PaneContextMenuState;
  onHide: () => void;
}> = (props) => {
  const handleItemClick = (item: MenuItem): void => {
    if (item.disabled || item.separator) return;
    item.onClick();
    props.onHide();
  };

  return (
    <Show when={props.state.visible}>
      <>
        {/* 点击外部关闭 */}
        <div class="vs-pane-context-menu-backdrop" onClick={props.onHide} />
        <ul
          class="vs-pane-context-menu"
          style={{
            position: "fixed",
            left: `${props.state.x}px`,
            top: `${props.state.y}px`,
            "z-index": "10000",
          }}
          role="menu"
        >
          <For each={props.state.items}>
            {(item) => (
              <Show
                when={!item.separator}
                fallback={
                  <li class="vs-pane-context-menu-separator" role="separator" />
                }
              >
                <li
                  class={`vs-pane-context-menu-item ${
                    item.disabled ? "is-disabled" : ""
                  }`}
                  role="menuitem"
                  onClick={() => handleItemClick(item)}
                >
                  <span class="vs-pane-context-menu-label">{item.label}</span>
                  <Show when={item.shortcut}>
                    <span class="vs-pane-context-menu-shortcut">
                      {item.shortcut}
                    </span>
                  </Show>
                </li>
              </Show>
            )}
          </For>
        </ul>
      </>
    </Show>
  );
};
