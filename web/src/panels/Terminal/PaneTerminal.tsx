/**
 * PaneTerminal · MVP-05 Phase C
 *
 * 单 Pane xterm.js 容器 · 用 pane_pty_* 5 IPC commands + 2 events 接管 PTY 生命周期。
 * 与 [`TerminalPane.tsx`] 的本质差别：
 * - 用 `pane_pty_*` 而非 `tab_pty_*`（独立命名空间 · §H.6 锁 A）
 * - 监听 `pane_pty_stdout` / `pane_pty_exited`（payload 字段是 `paneId` 而非 `tabId`）
 * - 由父 [`PaneSplitView`] 决定 active / dimensions / focus
 *
 * Phase C scaffolding · 暂未与 [`Terminal.tsx`] 集成 · 集成留 PR #145。
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  readText as readClipboardText,
  writeText as writeClipboardText,
} from "@tauri-apps/plugin-clipboard-manager";

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
import type {
  PanePtyExitedEvent,
  PanePtySpawnRequest,
  PanePtyStdoutEvent,
} from "../../bindings";
import { useSettings } from "../../stores/settings";

type XTermCursorStyle = "block" | "underline" | "bar";

const toCursorStyle = (s: string): XTermCursorStyle => {
  if (s === "bar" || s === "underline") return s;
  return "block";
};

const DEFAULT_COLS = 80;
const DEFAULT_ROWS = 24;

export type PaneTerminalApi = {
  focus: () => void;
  paste: (text: string) => void;
  clear: () => void;
};

type PaneTerminalProps = {
  paneId: string;
  shell: string;
  cwd: string;
  active: boolean;
  focused: boolean;
  onRegisterApi?: (paneId: string, api: PaneTerminalApi) => void;
  onUnregisterApi?: (paneId: string) => void;
  onClick?: (paneId: string) => void;
  onExit?: (paneId: string, exitCode: number | null) => void;
  onError?: (paneId: string, message: string) => void;
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
  };
};

type ActiveRenderers = { webgl?: WebglAddon; canvas?: CanvasAddon };

const setupRenderer = (term: XTerm, paneId: string): ActiveRenderers => {
  try {
    const webgl = new WebglAddon();
    term.loadAddon(webgl);
    return { webgl };
  } catch (error) {
    console.warn(
      `[mvp-05] pane ${paneId} webgl renderer unavailable, falling back to canvas`,
      error,
    );
  }
  try {
    const canvas = new CanvasAddon();
    term.loadAddon(canvas);
    return { canvas };
  } catch (error) {
    console.warn(
      `[mvp-05] pane ${paneId} canvas renderer unavailable, falling back to DOM`,
      error,
    );
  }
  return {};
};

export const PaneTerminal: Component<PaneTerminalProps> = (props) => {
  const { settings } = useSettings();
  const [spawnError, setSpawnError] = createSignal<string | null>(null);
  let hostRef: HTMLDivElement | undefined;
  let term: XTerm | undefined;
  let fitAddon: FitAddon | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let unlistenStdout: UnlistenFn | undefined;
  let unlistenExited: UnlistenFn | undefined;
  // theme 切换需 dispose + reload addon · WebGL texture atlas 缓存 glyph 不会自动更新
  let activeWebglAddon: WebglAddon | undefined;
  let activeCanvasAddon: CanvasAddon | undefined;
  let themeObserver: MutationObserver | undefined;

  // 同步 xterm theme · WebglAddon/CanvasAddon 缓存 atlas 必须 dispose+reload 才能用新色
  // （xterm 5.x term.options.theme 单独设不够 · clearTextureAtlas 实测也不彻底 ·
  //  TerminalPane 已验证 · 此处对齐）
  const syncTheme = () => {
    if (!term) return;
    term.options.theme = createTheme();
    if (activeWebglAddon) {
      try {
        activeWebglAddon.dispose();
      } catch {}
      activeWebglAddon = undefined;
    }
    if (activeCanvasAddon) {
      try {
        activeCanvasAddon.dispose();
      } catch {}
      activeCanvasAddon = undefined;
    }
    const renderers = setupRenderer(term, props.paneId);
    activeWebglAddon = renderers.webgl;
    activeCanvasAddon = renderers.canvas;
  };

  const queueFit = () => {
    if (!props.active || !fitAddon) return;
    requestAnimationFrame(() => {
      try {
        fitAddon?.fit();
      } catch {
        // hidden host can throw during transitions
      }
    });
  };

  createEffect(() => {
    if (props.active) {
      queueFit();
      if (props.focused) {
        term?.focus();
      }
    }
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

    if (!hostRef) return;
    term.open(hostRef);
    const renderers = setupRenderer(term, props.paneId);
    activeWebglAddon = renderers.webgl;
    activeCanvasAddon = renderers.canvas;

    // 拦截 cmd/ctrl+C 复制 · cmd/ctrl+V 粘贴 · cmd/ctrl+A 全选。
    // xterm canvas/webgl 渲染不是原生 selectable 文本 · 系统 cmd+C 路径拿不到字。
    // navigator.clipboard 在 Tauri WKWebView 不稳定 · 用 tauri-plugin-clipboard-manager
    // 走 IPC 调系统 NSPasteboard / GTK clipboard · 必稳。
    // shift 修饰留给 selection 操作（shift+arrows）· 不拦。
    term.attachCustomKeyEventHandler((event) => {
      if (event.type !== "keydown") return true;
      const mod = event.metaKey || event.ctrlKey;
      if (!mod || event.shiftKey || event.altKey) return true;
      const key = event.key.toLowerCase();
      if (key === "c") {
        const sel = term?.getSelection() ?? "";
        if (sel) {
          event.preventDefault();
          void writeClipboardText(sel).catch((err) => {
            console.warn("[clipboard] writeText failed", err);
          });
          term?.clearSelection();
          return false;
        }
        // 没 selection · 让 ^C 发到 pty（SIGINT）
        return true;
      }
      if (key === "v") {
        event.preventDefault();
        void readClipboardText()
          .then((text) => {
            if (text) term?.paste(text);
          })
          .catch((err) => {
            console.warn("[clipboard] readText failed", err);
          });
        return false;
      }
      if (key === "a") {
        event.preventDefault();
        term?.selectAll();
        return false;
      }
      return true;
    });

    props.onRegisterApi?.(props.paneId, {
      focus: () => term?.focus(),
      paste: (text) => term?.paste(text),
      clear: () => term?.reset(),
    });

    term.onData((data) => {
      void invoke("pane_pty_stdin", {
        paneId: props.paneId,
        data,
      }).catch((error: unknown) => {
        const msg = error instanceof Error ? error.message : String(error);
        props.onError?.(props.paneId, msg);
      });
    });

    term.onResize(({ cols, rows }) => {
      void invoke("pane_pty_resize", {
        paneId: props.paneId,
        cols,
        rows,
      });
    });

    resizeObserver = new ResizeObserver(() => queueFit());
    resizeObserver.observe(hostRef);

    // 监听 data-theme attribute 变化 · 触发 syncTheme · 跟 TerminalPane fix8 一致
    // 用 attribute observer 而非 settings.theme reactive · 避免 setSettings/applyCssVars
    // 顺序 race（applyCssVars 设 data-theme · createTheme 读最新 CSS vars · 有保证）
    themeObserver = new MutationObserver(() => syncTheme());
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    // listen 订阅必须在 spawn 之前完成 · Tauri emit 不缓冲 · listener 没 ready
    // 时 backend 已 emit 的 stdout 会丢（如 shell 启动后 prompt 第一行）。
    // 用 Promise.all 并行两个 listen（互不依赖）· 省 1 次 round-trip。
    [unlistenStdout, unlistenExited] = await Promise.all([
      listen<PanePtyStdoutEvent>("pane_pty_stdout", (event) => {
        if (event.payload.paneId !== props.paneId) return;
        term?.write(event.payload.data);
      }),
      listen<PanePtyExitedEvent>("pane_pty_exited", (event) => {
        if (event.payload.paneId !== props.paneId) return;
        props.onExit?.(props.paneId, event.payload.exitCode);
        term?.write(
          `\r\n[Process exited (code ${event.payload.exitCode ?? "signal"})]\r\n`,
        );
      }),
    ]);

    queueFit();
    const cols = term.cols || DEFAULT_COLS;
    const rows = term.rows || DEFAULT_ROWS;
    try {
      await invoke("pane_pty_spawn", {
        req: {
          paneId: props.paneId,
          shell: props.shell,
          cwd: props.cwd,
          cols,
          rows,
        } satisfies PanePtySpawnRequest,
      });
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error);
      setSpawnError(msg);
      props.onError?.(props.paneId, msg);
    }
  });

  onCleanup(() => {
    props.onUnregisterApi?.(props.paneId);
    resizeObserver?.disconnect();
    themeObserver?.disconnect();
    unlistenStdout?.();
    unlistenExited?.();
    void invoke("pane_pty_kill", { paneId: props.paneId }).catch(() => {
      // pane already exited
    });
    try { activeWebglAddon?.dispose(); } catch {}
    try { activeCanvasAddon?.dispose(); } catch {}
    term?.dispose();
  });

  return (
    <div
      class={`vs-pane-terminal ${props.focused ? "is-focused" : ""}`}
      onClick={() => props.onClick?.(props.paneId)}
      role="presentation"
    >
      <div ref={hostRef} class="vs-pane-terminal-host" />
      {spawnError() ? (
        <div class="vs-pane-terminal-error">PTY 启动失败: {spawnError()}</div>
      ) : null}
    </div>
  );
};
