import {
  type Component,
  createSignal,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { WorkspaceMetadata } from "../App";
import { t, normalizeLanguage } from "../i18n";
import { formatShortcut } from "../lib/format-shortcut";
import { detectPlatform } from "../lib/platform";
import { useSettings } from "../stores/settings";
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
  const { settings } = useSettings();
  const [maximized, setMaximized] = createSignal(false);
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const maximizeLabel = () =>
    maximized()
      ? label("chrome.window.restore")
      : label("chrome.window.maximize");

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
        aria-label={label("chrome.window.minimize")}
        title={label("chrome.window.minimize")}
        onClick={() => void getCurrentWindow().minimize()}
      >
        <WindowMinimizeIcon />
      </button>
      <button
        type="button"
        class="vs-window-control"
        aria-label={maximizeLabel()}
        title={maximizeLabel()}
        onClick={() => void getCurrentWindow().toggleMaximize()}
      >
        <Show when={maximized()} fallback={<WindowMaximizeIcon />}>
          <WindowRestoreIcon />
        </Show>
      </button>
      <button
        type="button"
        class="vs-window-control vs-window-control-close"
        aria-label={label("chrome.window.close")}
        title={label("chrome.window.close")}
        onClick={() => void getCurrentWindow().close()}
      >
        <WindowCloseIcon />
      </button>
    </div>
  );
};

export const TopBar: Component<TopBarProps> = (props) => {
  const { settings } = useSettings();
  const language = () => normalizeLanguage(settings.language);
  const label = (key: string) => t(key, language());
  const primarySidebarLabel = () => label("chrome.topbar.togglePrimarySidebar");

  return (
    <header
      class="vs-top-bar"
      data-tauri-drag-region
      onMouseDown={props.onMouseDown}
    >
      <button
        type="button"
        class={`vs-top-bar-toggle${props.primaryOpen() ? "" : " vs-top-bar-toggle-off"}`}
        aria-label={primarySidebarLabel()}
        aria-pressed={props.primaryOpen()}
        title={`${primarySidebarLabel()} (${formatShortcut("⌘B", "Ctrl+B")})`}
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
