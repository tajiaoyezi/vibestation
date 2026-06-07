import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { CanvasAddon } from "@xterm/addon-canvas";
import { FitAddon } from "@xterm/addon-fit";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { WebglAddon } from "@xterm/addon-webgl";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm, type IDisposable } from "@xterm/xterm";
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
import {
  buildTerminalFontStack,
  TERMINAL_FONT_WEIGHT,
  TERMINAL_FONT_WEIGHT_BOLD,
  TERMINAL_LINE_HEIGHT,
} from "./terminalTypography";
import { createTerminalTheme, resolveTerminalThemeMode } from "./terminalTheme";

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

type ActiveRenderers = {
  webgl?: WebglAddon;
  canvas?: CanvasAddon;
  contextLoss?: IDisposable;
};

const setupCanvasRenderer = (
  term: XTerm,
  tabId: string,
  onRendererChange: (renderer: RendererKind) => void,
): ActiveRenderers => {
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

const setupRenderer = (
  term: XTerm,
  tabId: string,
  onRendererChange: (renderer: RendererKind) => void,
  onReplace?: (renderers: ActiveRenderers) => void,
): ActiveRenderers => {
  try {
    const webgl = new WebglAddon();
    term.loadAddon(webgl);
    onRendererChange("webgl");
    const contextLoss = webgl.onContextLoss(() => {
      console.warn(
        `[mvp-04] ${tabId} webgl context lost, falling back to canvas`,
      );
      try {
        webgl.dispose();
      } catch {}
      const fallback = setupCanvasRenderer(term, tabId, onRendererChange);
      onReplace?.(fallback);
      try {
        term.refresh(0, term.rows - 1);
      } catch {}
    });
    return { webgl, contextLoss };
  } catch (error) {
    console.warn(
      `[mvp-04] ${tabId} webgl renderer unavailable, falling back to canvas`,
      error,
    );
  }

  return setupCanvasRenderer(term, tabId, onRendererChange);
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
  let activeWebglContextLoss: IDisposable | undefined;
  let unlistenStdout: UnlistenFn | undefined;
  let unlistenExited: UnlistenFn | undefined;
  let themeObserver: MutationObserver | undefined;
  let handlePasteCapture: ((event: ClipboardEvent) => void) | undefined;
  // MVP-20 BUG-001 round 2 · setTimeout cleanup + mount guard 防止 listener
  // 在 unmount 后 fire 导致 SolidJS <Show> stale accessor 警告（idle pty 输出
  // pane_pty_stdout 时所有 listener fire · 包括正在 unmount 的 component）
  const pendingTimers: Array<ReturnType<typeof setTimeout>> = [];
  let mounted = true;

  const currentSize = () => ({
    cols: term?.cols || props.runtime.cols || DEFAULT_PTY_COLS,
    rows: term?.rows || props.runtime.rows || DEFAULT_PTY_ROWS,
  });

  // 字体栈 · 普通 JetBrains Mono 优先，Nerd Font 仅作图标兜底。
  const fontStack = () => buildTerminalFontStack(settings.fontFamily);

  const replaceRenderer = (renderers: ActiveRenderers) => {
    try {
      activeWebglContextLoss?.dispose();
    } catch {}
    activeWebglContextLoss = renderers.contextLoss;
    activeWebglAddon = renderers.webgl;
    activeCanvasAddon = renderers.canvas;
  };

  const syncTheme = () => {
    if (!term) {
      return;
    }

    term.options.theme = createTerminalTheme(resolveTerminalThemeMode());
    try {
      activeWebglContextLoss?.dispose();
    } catch {}
    activeWebglContextLoss = undefined;
    // WebglAddon / CanvasAddon 自己 cache 字符 glyph 到 texture atlas · clearTextureAtlas
    // 不彻底（实测 atlas 仍按旧 theme 色重新生成）。dispose + 重新 loadAddon 才能强制
    // 用新 theme 色重建 atlas · 这是 xterm 5.x theme 切换的可靠方案。
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
    const renderers = setupRenderer(
      term,
      props.tab.tabId,
      (renderer) => props.onRendererChange(props.tab.tabId, renderer),
      replaceRenderer,
    );
    activeWebglAddon = renderers.webgl;
    activeCanvasAddon = renderers.canvas;
    activeWebglContextLoss = renderers.contextLoss;
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

  // 字体 / 字号实时生效 · 改 settings 后推到已开终端 + 重新 fit（字号变 → cell 尺寸变 → cols/rows 重算）
  createEffect(() => {
    const family = fontStack();
    const size = settings.fontSize;
    if (term) {
      term.options.fontFamily = family;
      term.options.fontSize = size;
      queueFit();
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
      allowTransparency: true,
      convertEol: false,
      cursorBlink: settings.cursorBlink,
      cursorStyle: toCursorStyle(settings.cursorStyle),
      fontFamily: fontStack(),
      fontSize: settings.fontSize,
      fontWeight: TERMINAL_FONT_WEIGHT,
      fontWeightBold: TERMINAL_FONT_WEIGHT_BOLD,
      lineHeight: TERMINAL_LINE_HEIGHT,
      scrollback: 10000,
      theme: createTerminalTheme(resolveTerminalThemeMode()),
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
      const renderers = setupRenderer(
        term,
        props.tab.tabId,
        (renderer) => props.onRendererChange(props.tab.tabId, renderer),
        replaceRenderer,
      );
      activeWebglAddon = renderers.webgl;
      activeCanvasAddon = renderers.canvas;
      activeWebglContextLoss = renderers.contextLoss;
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
      attributeFilter: ["data-theme", "style"],
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
      // mount guard · idle pty 在 pool 内的 stdout 也 emit 此事件 · 所有 listener fire
      // unmounted listener access props.tab.tabId 是 stale reactive 触发 SolidJS <Show> 警告
      if (!mounted) return;
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
      if (!mounted) return;
      if (event.payload.tabId !== props.tab.tabId) {
        return;
      }

      props.onExit(props.tab.tabId, event.payload.exitCode);
      term?.write(
        `\r\n\r\n[Process exited (code ${event.payload.exitCode ?? "signal"}). Press Enter to restart]\r\n`,
      );
    });

    // 等两次 rAF · 确保 host div CSS layout 完成有实际尺寸后再 fit+spawn
    await new Promise<void>((r) => {
      requestAnimationFrame(() => requestAnimationFrame(() => r()));
    });
    try {
      fitAddon?.fit();
    } catch {}
    if (props.runtime.phase === "idle") {
      beginStart();
      // xterm 首次加载 canvas/webgl 渲染管线可能未完成 · 等 PTY 输出 prompt 后强制重绘
      const refreshTimer = setTimeout(() => {
        if (!mounted) return;
        try {
          term?.refresh(0, term.rows - 1);
        } catch {}
      }, 80);
      pendingTimers.push(refreshTimer);
    }
  });

  onCleanup(() => {
    // mount guard 立即设 false · 防 listener 在 unlisten 调用之前还能 fire 一次
    mounted = false;
    // clear 所有未 fire 的 setTimeout · 防 unmount 后 fire 引用 stale props/term
    for (const t of pendingTimers) {
      clearTimeout(t);
    }
    pendingTimers.length = 0;
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
    try {
      activeWebglContextLoss?.dispose();
    } catch {}
    try {
      activeWebglAddon?.dispose();
    } catch {}
    try {
      activeCanvasAddon?.dispose();
    } catch {}
    try {
      term?.dispose();
    } catch {}
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
