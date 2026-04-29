import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { CanvasAddon } from "@xterm/addon-canvas";
import { FitAddon } from "@xterm/addon-fit";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { WebglAddon } from "@xterm/addon-webgl";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm } from "@xterm/xterm";
import {
  createEffect,
  createSignal,
  onCleanup,
  onMount,
  type Component,
} from "solid-js";
import type { PtyExitedEvent, PtyStdoutEvent, TabState } from "../../bindings";
import { useSettings } from "../../stores/settings";
import {
  DEFAULT_PTY_COLS,
  DEFAULT_PTY_ROWS,
  fetchScrollback,
  getShortcutAction,
  type RendererKind,
  type TabRuntimeState,
  writeScrollbackToTerm,
} from "./hooks";

type XTermCursorStyle = "block" | "underline" | "bar";

const toCursorStyle = (s: string): XTermCursorStyle => {
  if (s === "bar" || s === "underline") return s;
  return "block";
};

type PaneApi = {
  focus: () => void;
  paste: (text: string) => void;
  clear: () => void;
  copy: () => void;
  selectAll: () => void;
};

type TerminalPaneProps = {
  active: boolean;
  isNewlyCreated: boolean;
  pasteGuardDisabled: boolean;
  runtime: TabRuntimeState;
  tab: TabState;
  onExit: (tabId: string, exitCode: number | null) => void;
  onPasteRequest: (tabId: string, text: string) => void;
  onRegisterApi: (tabId: string, api: PaneApi) => void;
  onRendererChange: (tabId: string, renderer: RendererKind) => void;
  onResize: (tabId: string, cols: number, rows: number) => Promise<void>;
  onStart: (tab: TabState, cols: number, rows: number) => Promise<void>;
  onStdinError: (message: string) => void;
  onStdout: (tabId: string) => void;
  onUnregisterApi: (tabId: string) => void;
};

const createTheme = () => {
  const styles = getComputedStyle(document.documentElement);
  const read = (name: string, fallback: string) =>
    styles.getPropertyValue(name).trim() || fallback;

  return {
    background: read("--bg-1", "#11141b"),
    foreground: read("--text-1", "#f5f7ff"),
    cursor: read("--text-1", "#f5f7ff"),
    cursorAccent: read("--bg-1", "#11141b"),
    selectionBackground: read("--accent-soft", "rgba(120, 169, 255, 0.18)"),
    black: read("--bg-0", "#0d1016"),
    brightBlack: read("--text-4", "#6c7485"),
    red: read("--danger", "#ff7575"),
    brightRed: read("--danger", "#ff7575"),
    green: read("--success", "#4cd38f"),
    brightGreen: read("--success", "#4cd38f"),
    yellow: read("--warning", "#f6c24a"),
    brightYellow: read("--warning", "#f6c24a"),
    blue: read("--accent", "#6fa9ff"),
    brightBlue: read("--accent", "#6fa9ff"),
    magenta: read("--accent", "#6fa9ff"),
    brightMagenta: read("--accent", "#6fa9ff"),
    cyan: read("--accent", "#6fa9ff"),
    brightCyan: read("--accent", "#6fa9ff"),
    white: read("--text-2", "#c3cad8"),
    brightWhite: read("--text-1", "#f5f7ff"),
  };
};

const setupRenderer = (
  term: XTerm,
  tabId: string,
  onRendererChange: (renderer: RendererKind) => void,
): { webgl?: WebglAddon; canvas?: CanvasAddon } => {
  try {
    const webgl = new WebglAddon();
    term.loadAddon(webgl);
    onRendererChange("webgl");
    return { webgl };
  } catch (error) {
    console.warn(
      `[mvp-04] ${tabId} webgl renderer unavailable, falling back to canvas`,
      error,
    );
  }

  try {
    const canvas = new CanvasAddon();
    term.loadAddon(canvas);
    onRendererChange("canvas");
    return { canvas };
  } catch (error) {
    console.warn(
      `[mvp-04] ${tabId} canvas renderer unavailable, falling back to DOM`,
      error,
    );
  }

  onRendererChange("dom");
  return {};
};

const hasPrintableOutput = (data: string): boolean => {
  const stripped = data
    .replace(/\u001b\[[0-9;?]*[ -/]*[@-~]/g, "")
    .replace(/\u001b\][^\u0007]*(\u0007|\u001b\\)/g, "")
    .replace(/\u001b[PX^_].*?(?:\u001b\\)/g, "")
    .replace(/[\r\n\t ]+/g, "");

  return stripped.length > 0;
};

export const TerminalPane: Component<TerminalPaneProps> = (props) => {
  const { settings } = useSettings();
  const [hasVisibleOutput, setHasVisibleOutput] = createSignal(false);
  let paneRef: HTMLDivElement | undefined;
  let hostRef: HTMLDivElement | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let fitAddon: FitAddon | undefined;
  let term: XTerm | undefined;
  // WebGL/Canvas addon ref · syncTheme 时 dispose + reload · 强制 atlas 重建用新 theme 色
  let activeWebglAddon: WebglAddon | undefined;
  let activeCanvasAddon: CanvasAddon | undefined;
  let unlistenStdout: UnlistenFn | undefined;
  let unlistenExited: UnlistenFn | undefined;
  let themeObserver: MutationObserver | undefined;
  let handlePasteCapture: ((event: ClipboardEvent) => void) | undefined;

  const currentSize = () => ({
    cols: term?.cols || props.runtime.cols || DEFAULT_PTY_COLS,
    rows: term?.rows || props.runtime.rows || DEFAULT_PTY_ROWS,
  });

  const syncTheme = () => {
    if (!term) {
      return;
    }

    term.options.theme = createTheme();
    // WebglAddon / CanvasAddon 自己 cache 字符 glyph 到 texture atlas · clearTextureAtlas
    // 不彻底（实测 atlas 仍按旧 theme 色重新生成）。dispose + 重新 loadAddon 才能强制
    // 用新 theme 色重建 atlas · 这是 xterm 5.x WebGL theme 切换的唯一可靠方案。
    if (activeWebglAddon) {
      try {
        activeWebglAddon.dispose();
      } catch {
        // ignore · disposed 状态再 dispose 抛错
      }
      activeWebglAddon = undefined;
    }
    if (activeCanvasAddon) {
      try {
        activeCanvasAddon.dispose();
      } catch {
        // ignore
      }
      activeCanvasAddon = undefined;
    }
    const renderers = setupRenderer(term, props.tab.tabId, (renderer) =>
      props.onRendererChange(props.tab.tabId, renderer),
    );
    activeWebglAddon = renderers.webgl;
    activeCanvasAddon = renderers.canvas;
  };

  const queueFit = () => {
    if (!fitAddon) {
      return;
    }

    requestAnimationFrame(() => {
      try {
        fitAddon?.fit();
      } catch {
        // Hidden host can throw during panel transitions; next focus retries.
      }
    });
  };

  const beginStart = () => {
    void props.onStart(props.tab, currentSize().cols, currentSize().rows);
  };

  createEffect(() => {
    if (!props.active) {
      return;
    }

    syncTheme();
    queueFit();
    term?.focus();
  });

  createEffect(() => {
    const blink = settings.cursorBlink;
    const style = toCursorStyle(settings.cursorStyle);
    if (term) {
      term.options.cursorBlink = blink;
      term.options.cursorStyle = style;
    }
  });

  onMount(async () => {
    handlePasteCapture = (event: ClipboardEvent) => {
      const text = event.clipboardData?.getData("text") ?? "";
      if (props.pasteGuardDisabled || !/[\r\n]/.test(text)) {
        return;
      }

      event.preventDefault();
      props.onPasteRequest(props.tab.tabId, text);
    };

    term = new XTerm({
      allowProposedApi: true,
      convertEol: false,
      cursorBlink: settings.cursorBlink,
      cursorStyle: toCursorStyle(settings.cursorStyle),
      fontFamily: [
        settings.fontFamily,
        "DejaVu Sans Mono",
        "Ubuntu Mono",
        "ui-monospace",
        "Liberation Mono",
        "monospace",
      ].join(", "),
      fontSize: 13,
      lineHeight: 1.3,
      scrollback: 10000,
      theme: createTheme(),
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new Unicode11Addon());
    term.unicode.activeVersion = "11";
    term.loadAddon(new WebLinksAddon());
    term.attachCustomKeyEventHandler(
      (event) =>
        getShortcutAction(event, { allowEditableTarget: true }) === null,
    );

    if (!hostRef) {
      return;
    }

    term.open(hostRef);
    {
      const renderers = setupRenderer(term, props.tab.tabId, (renderer) =>
        props.onRendererChange(props.tab.tabId, renderer),
      );
      activeWebglAddon = renderers.webgl;
      activeCanvasAddon = renderers.canvas;
    }
    props.onRegisterApi(props.tab.tabId, {
      focus: () => term?.focus(),
      paste: (text) => term?.paste(text),
      clear: () => term?.reset(),
      copy: () => {
        const selection = term?.getSelection() ?? "";
        if (selection) {
          void navigator.clipboard.writeText(selection);
        }
      },
      selectAll: () => term?.selectAll(),
    });

    term.onData((data) => {
      if (props.runtime.phase === "exited" || props.runtime.phase === "error") {
        if (data === "\r" || data === "\n") {
          setHasVisibleOutput(false);
          term?.reset();
          beginStart();
        }
        return;
      }

      void invoke("tab_pty_stdin", {
        tabId: props.tab.tabId,
        data,
      }).catch((error) => {
        props.onStdinError(
          error instanceof Error ? error.message : String(error),
        );
      });
    });

    term.onResize(({ cols, rows }) => {
      void props.onResize(props.tab.tabId, cols, rows);
    });

    resizeObserver = new ResizeObserver(() => {
      queueFit();
    });
    resizeObserver.observe(hostRef);
    if (handlePasteCapture) {
      paneRef?.addEventListener("paste", handlePasteCapture, true);
    }

    themeObserver = new MutationObserver(() => {
      syncTheme();
    });
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    if (!props.isNewlyCreated) {
      try {
        const lines = await fetchScrollback(props.tab.tabId);
        await writeScrollbackToTerm(term, lines);
        if (lines.length > 0) {
          setHasVisibleOutput(true);
        }
      } catch (error) {
        props.onStdinError(
          `恢复终端 scrollback 失败：${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }

    unlistenStdout = await listen<PtyStdoutEvent>("tab_pty_stdout", (event) => {
      if (event.payload.tabId !== props.tab.tabId) {
        return;
      }

      if (hasPrintableOutput(event.payload.data)) {
        setHasVisibleOutput(true);
      }
      props.onStdout(props.tab.tabId);
      term?.write(event.payload.data);
    });

    unlistenExited = await listen<PtyExitedEvent>("tab_pty_exited", (event) => {
      if (event.payload.tabId !== props.tab.tabId) {
        return;
      }

      props.onExit(props.tab.tabId, event.payload.exitCode);
      term?.write(
        `\r\n\r\n[Process exited (code ${event.payload.exitCode ?? "signal"}). Press Enter to restart]\r\n`,
      );
    });

    try { fitAddon?.fit(); } catch {}
    if (props.runtime.phase === "idle") {
      beginStart();
    }
  });

  onCleanup(() => {
    props.onUnregisterApi(props.tab.tabId);
    resizeObserver?.disconnect();
    themeObserver?.disconnect();
    if (handlePasteCapture) {
      paneRef?.removeEventListener("paste", handlePasteCapture, true);
    }
    unlistenStdout?.();
    unlistenExited?.();
    // WebGL/Canvas addon 必须在 term.dispose() 之前清理 · 否则 addon 内部
    // 访问 this._terminal._core._store._isDisposed 抛 undefined 错误
    try { activeWebglAddon?.dispose(); } catch {}
    try { activeCanvasAddon?.dispose(); } catch {}
    term?.dispose();
  });

  return (
    <div
      ref={paneRef}
      id={`terminal-pane-${props.tab.tabId}`}
      class={`vs-terminal-pane ${props.active ? "is-active" : ""}`}
      aria-hidden={!props.active}
      onContextMenu={(e) => {
        e.preventDefault();
        void invoke("menu_show_terminal", {
          x: e.clientX,
          y: e.clientY,
        });
      }}
    >
      <div
        class={`vs-terminal-loading ${!hasVisibleOutput() && props.runtime.phase !== "error" && props.runtime.phase !== "exited" ? "is-visible" : ""}`}
        aria-hidden={
          hasVisibleOutput() ||
          props.runtime.phase === "error" ||
          props.runtime.phase === "exited"
        }
      >
        <span class="vs-terminal-loading-label">
          Launching {props.tab.shell}…
        </span>
        <span class="vs-terminal-loading-hint">
          Waiting for the first shell output
        </span>
      </div>
      <div ref={hostRef} class="vs-terminal-host" />
    </div>
  );
};
