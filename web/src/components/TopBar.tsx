import {
  type Component,
  createSignal,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { WorkspaceMetadata } from "../App";
import { formatShortcut } from "../lib/format-shortcut";
import { detectPlatform } from "../lib/platform";
import {
  SidebarLeftIcon,
  WindowCloseIcon,
  WindowMaximizeIcon,
  WindowMinimizeIcon,
  WindowRestoreIcon,
} from "./Icons";

interface TopBarProps {
  activeWorkspace: () => WorkspaceMetadata | null;
  primaryOpen: () => boolean;
  onTogglePrimary: () => void;
  onMouseDown?: (e: MouseEvent) => void;
}

// Windows frameless：前端自绘 min/max/close（原生标题栏已在 Rust 侧关掉，
// 见 crates/app/src/lib.rs configure_title_bar）。macOS / Linux 走系统装饰，不渲染。
const WindowControls: Component = () => {
  const [maximized, setMaximized] = createSignal(false);

  onMount(() => {
    const win = getCurrentWindow();
    void win.isMaximized().then(setMaximized);
    const unlisten = win.onResized(() => {
      void win.isMaximized().then(setMaximized);
    });
    onCleanup(() => {
      void unlisten.then((fn) => fn());
    });
  });

  return (
    <div class="vs-window-controls">
      <button
        type="button"
        class="vs-window-control"
        aria-label="Minimize"
        title="Minimize"
        onClick={() => void getCurrentWindow().minimize()}
      >
        <WindowMinimizeIcon />
      </button>
      <button
        type="button"
        class="vs-window-control"
        aria-label={maximized() ? "Restore" : "Maximize"}
        title={maximized() ? "Restore" : "Maximize"}
        onClick={() => void getCurrentWindow().toggleMaximize()}
      >
        <Show when={maximized()} fallback={<WindowMaximizeIcon />}>
          <WindowRestoreIcon />
        </Show>
      </button>
      <button
        type="button"
        class="vs-window-control vs-window-control-close"
        aria-label="Close"
        title="Close"
        onClick={() => void getCurrentWindow().close()}
      >
        <WindowCloseIcon />
      </button>
    </div>
  );
};

export const TopBar: Component<TopBarProps> = (props) => {
  return (
    <header
      class="vs-top-bar"
      data-tauri-drag-region
      onMouseDown={props.onMouseDown}
    >
      <button
        type="button"
        class={`vs-top-bar-toggle${props.primaryOpen() ? "" : " vs-top-bar-toggle-off"}`}
        aria-label="Toggle primary sidebar"
        aria-pressed={props.primaryOpen()}
        title={`Toggle Primary Sidebar (${formatShortcut("⌘B", "Ctrl+B")})`}
        onClick={props.onTogglePrimary}
      >
        <SidebarLeftIcon />
      </button>
      <Show when={props.activeWorkspace()}>
        {(ws) => (
          <div class="vs-top-bar-meta">
            <span class="vs-top-bar-name">{ws().name}</span>
            <span class="vs-top-bar-path" title={ws().path}>
              {ws().path}
            </span>
          </div>
        )}
      </Show>
      <Show when={detectPlatform() === "windows"}>
        <WindowControls />
      </Show>
    </header>
  );
};
