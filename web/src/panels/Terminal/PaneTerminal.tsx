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
import { SerializeAddon } from "@xterm/addon-serialize";
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
  SpawnResult,
  SplitDir,
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
  // MVP-05 layout 切换 reload 修复 · 让 caller 在 split / close 之前拿到当前 xterm 完整
  // 状态（buffer + alt screen + cursor + 颜色）的 ANSI 字符串 · remount 时 write 回去恢复。
  serialize: () => string;
};

/**
 * 模块级 snapshot store · key=paneId · value=SerializeAddon 输出的 ANSI 字符串。
 * 在 layout 切换前由 Terminal.tsx 的 handlePaneSplit / handlePaneClose 写入 ·
 * 新 PaneTerminal mount 时读取并 write 到新 xterm 后删除 entry。
 */
export const paneSnapshots = new Map<string, string>();

type PaneTerminalProps = {
  paneId: string;
  shell: string;
  cwd: string;
  active: boolean;
  focused: boolean;
  /**
   * MVP-14 Phase C §E · 是否处于临时最大化状态。
   * 仅 visual hint：渲染 `data-is-maximized="true"` + 显示 maximized chip ·
   * CSS 选择器命中后撑满 host。其他 pane 仍 mount · PTY 不被 kill。
   */
  maximized?: boolean;
  onRegisterApi?: (paneId: string, api: PaneTerminalApi) => void;
  onUnregisterApi?: (paneId: string) => void;
  onClick?: (paneId: string) => void;
  onExit?: (paneId: string, exitCode: number | null) => void;
  onError?: (paneId: string, message: string) => void;
  // MVP-05 visible split UI · toolbar 按钮 wire 到这两个 handler
  onSplit?: (direction: SplitDir, paneId: string) => void;
  onClose?: (paneId: string) => void;
  // cmd+V paste guard 路径 · 与 menu paste / legacy TerminalPane DOM paste 对齐：
  // PaneTerminal 不知道 workspace skip state · 把 paste 委托给上层决策（multiline check
  // + skipPasteConfirmByWorkspace） · 上层根据需要 setPendingPaste 弹 confirm dialog 或
  // 直接 paneApi.paste。未提供 callback 时 fallback 直接 paste（向后兼容）。
  onPasteRequest?: (paneId: string, text: string) => void;
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
    // 完整 16 ANSI 调色板 · 否则 xterm 走默认偏暗调色板 · 视觉灰蒙蒙。
    // 与 legacy TerminalPane.tsx 保持一致 · 防 pane vs tab 颜色分裂。
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
  let serializeAddon: SerializeAddon | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let unlistenStdout: UnlistenFn | undefined;
  let unlistenExited: UnlistenFn | undefined;
  // theme 切换需 dispose + reload addon · WebGL texture atlas 缓存 glyph 不会自动更新
  let activeWebglAddon: WebglAddon | undefined;
  let activeCanvasAddon: CanvasAddon | undefined;
  let themeObserver: MutationObserver | undefined;
  // MVP-20 BUG-001 · 收集所有 setTimeout · onCleanup 时 clear 防 unmount 后 fire
  // 触发 SolidJS <Show> stale accessor 警告（异步 callback 在 component unmount 后
  // 访问 reactive props 的典型 race）。
  const pendingTimers: Array<ReturnType<typeof setTimeout>> = [];
  let mounted = true;

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
    if (!fitAddon) return;
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
        // CJK 字符 fallback · 主 mono 字体不含中日韩字符时 · 浏览器走这里 ·
        // 优先选系统已有 / 接近严格 mono ratio 的字体 · 避免光标错位。
        "Sarasa Term SC", // 等距更纱 (用户可选装 · 严格 1:2 mono · 完美对齐)
        "PingFang SC", // macOS 默认中文
        "Hiragino Sans GB", // macOS 备选
        "Microsoft YaHei", // Windows
        "Noto Sans CJK SC", // Linux / 跨平台
        "WenQuanYi Micro Hei", // Linux 备选
        "monospace",
      ].join(", "),
      fontSize: 13,
      lineHeight: 1.3,
      scrollback: 10000,
      theme: createTheme(),
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    serializeAddon = new SerializeAddon();
    term.loadAddon(serializeAddon);
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
            if (!text) return;
            // 走上层 paste guard · multiline + skipPasteConfirm 决策由 Terminal.tsx 做
            if (props.onPasteRequest) {
              props.onPasteRequest(props.paneId, text);
            } else {
              // 向后兼容 · onPasteRequest 未提供时直接 paste（与原行为一致）
              term?.paste(text);
            }
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
      // SerializeAddon 会输出可重放的 ANSI 字符串（含 normal+alt screen + cursor + 颜色）·
      // term 未 ready 时返回空串。限 scrollback 1000 行 · 不传是 full buffer（PaneTerminal
      // 配 10000）· 跑过长输出（cargo build / npm install）后 split/close 同步序列化全 buffer
      // 阻塞 UI 100-300ms · 破 MVP-05 §F P99 latency 目标（Codex review #208 round 4 finding）。
      // 1000 行覆盖正常视觉范围 + 适量向上滚动历史 · viewport + alt screen + cursor 总是含。
      serialize: () => serializeAddon?.serialize({ scrollback: 1000 }) ?? "",
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
    //
    // MVP-20 BUG-001 fix · ANSI clear filter · 隐藏 cd 注入命令的 zsh ZLE echo
    // ----------------------------------------------------------------------
    // pool warm hit 时 · backend 注入 `cd -- '/path'; clear\n` · zsh ZLE 把它当
    // user input 字面 echo 给前端 + zsh-syntax-highlighting redraw 让 cd 命令短暂
    // 可见（截图取证 · 持续 100-300ms）· 然后 clear 命令清屏。
    // 修复：默认开 warmBuffer ON · invoke return 后基于 spawnResult.warm 决定：
    // - warm=true：保持 buffer · 等 ANSI clear sequence 出现 · 只 write clear 之后内容
    // - warm=false（cold）：立即 flush buffer + 关 buffer 模式 · 走正常 write
    let warmBufferActive = true;
    let warmBuffer: string[] = [];
    let warmBufferTimer: number | undefined;
    // ANSI clear sequences · 兼容 clear 命令 / RIS / cursor home + clear 多种序
    const ansiClearRegex = /\x1b\[(?:H\x1b\[2J|2J\x1b\[H|2J|3J|c)/;
    const flushWarmBuffer = (sliceFrom = 0): void => {
      if (warmBuffer.length === 0) {
        warmBufferActive = false;
        return;
      }
      const data = warmBuffer.join("");
      if (sliceFrom > 0 && sliceFrom < data.length) {
        term?.write(data.substring(sliceFrom));
      } else if (sliceFrom === 0) {
        term?.write(data);
      }
      // sliceFrom >= data.length · 整段丢弃
      warmBuffer = [];
      warmBufferActive = false;
      if (warmBufferTimer !== undefined) {
        clearTimeout(warmBufferTimer);
        warmBufferTimer = undefined;
      }
    };

    [unlistenStdout, unlistenExited] = await Promise.all([
      listen<PanePtyStdoutEvent>("pane_pty_stdout", (event) => {
        // mount guard · 防 IPC 事件在 unlisten 与 unmount 之间的窗口 fire ·
        // 此时 access props.paneId 是 stale reactive accessor → SolidJS 警告
        if (!mounted) return;
        if (event.payload.paneId !== props.paneId) return;
        if (warmBufferActive) {
          warmBuffer.push(event.payload.data);
          const data = warmBuffer.join("");
          const match = ansiClearRegex.exec(data);
          if (match) {
            // 找到 ANSI clear · 写 clear 之后的内容（cd echo 部分被丢弃）
            flushWarmBuffer(match.index);
          }
          return;
        }
        term?.write(event.payload.data);
      }),
      listen<PanePtyExitedEvent>("pane_pty_exited", (event) => {
        if (!mounted) return;
        if (event.payload.paneId !== props.paneId) return;
        props.onExit?.(props.paneId, event.payload.exitCode);
        term?.write(
          `\r\n[Process exited (code ${event.payload.exitCode ?? "signal"})]\r\n`,
        );
      }),
    ]);

    // MVP-05 reload 修复 · listener 已注册 · 此刻 backend emit 的 stdout 进 warmBuffer ·
    // 不会丢失 (Codex review #208 round 3 finding 缓解)。snapshot.write 写历史画面 ·
    // 后面 spawn 之后 flushWarmBuffer 接续 live stdout · 顺序：历史 → live · 不重叠。
    // 注：listener 注册前 (旧 PaneTerminal unlisten 与新 PaneTerminal listen 之间) 仍有
    // 几 ms race window · 该 window 内 backend emit 的 stdout 仍丢失 · 完全消除需 backend
    // pane scrollback replay IPC（超出本 PR · 已记录为 limitation）。
    const snapshot = paneSnapshots.get(props.paneId);
    if (snapshot) {
      paneSnapshots.delete(props.paneId);
      term.write(snapshot);
    }

    // 等两次 rAF · 确保 host div CSS layout 完成有实际尺寸后再 fit+spawn
    await new Promise<void>((r) => {
      requestAnimationFrame(() => requestAnimationFrame(() => r()));
    });
    try {
      fitAddon?.fit();
    } catch {}
    const cols = term.cols || DEFAULT_COLS;
    const rows = term.rows || DEFAULT_ROWS;
    try {
      let spawnResult: SpawnResult;
      try {
        spawnResult = await invoke<SpawnResult>("pane_pty_spawn", {
          req: {
            paneId: props.paneId,
            shell: props.shell,
            cwd: props.cwd,
            cols,
            rows,
          } satisfies PanePtySpawnRequest,
        });
      } catch (err: unknown) {
        // Race · 旧 PaneTerminal 的 onCleanup invoke pane_pty_kill 是 async · 新 PaneTerminal
        // 同时 mount 调 pane_pty_spawn · backend 看到 PTY 仍 active → AlreadyRunning。
        // 当作 warm reattach：listener 已订阅 · 后续 stdout 自动写入 · 不弹红色错误条。
        const msg = err instanceof Error ? err.message : String(err);
        if (/already\s*running/i.test(msg)) {
          spawnResult = { warm: true };
        } else {
          throw err;
        }
      }
      // MVP-20 BUG-001 fix · 决定 warmBuffer 命运：
      // warm hit → 设 500ms 兜底 timer（ANSI clear 必然在此前出现 · 否则强制 flush）
      // cold spawn → 立即 flush + 关 buffer · 后续 stdout 直接 write
      if (spawnResult.warm) {
        warmBufferTimer = window.setTimeout(() => {
          flushWarmBuffer();
        }, 500);
        pendingTimers.push(warmBufferTimer);
      } else {
        flushWarmBuffer();
      }
      // xterm 首次加载时 canvas/webgl 渲染管线可能未完成 · write 的数据不显示 ·
      // 给 PTY 50ms 输出 prompt 后强制整屏 refresh。
      const refreshTimer = setTimeout(() => {
        if (!mounted) return;
        try {
          term?.refresh(0, term.rows - 1);
        } catch {}
      }, 50);
      pendingTimers.push(refreshTimer);

      // 兜底 fit · 修 mount 时 host layout 还没稳定 · 第一次 fit 拿到默认 80x24 ·
      // 之后 ResizeObserver 因 host size 没变也不再触发 · PTY 永远停在 80 cols 的边界 case。
      // 实测：createTheme 多读 CSS vars 触发 forced reflow 后 · 偶发 mount race · spawn
      // 用错的 cols 起子进程 · TUI 程序（Claude Code）按 80 cols 渲染 · 屏幕右半空白。
      // 这里 spawn 已完成 · host layout 必稳 · 强制再 fit + 同步 backend resize。
      const fitGuardTimer = setTimeout(() => {
        if (!mounted || !term || !fitAddon) return;
        try {
          fitAddon.fit();
          const cols = term.cols;
          const rows = term.rows;
          if (cols > 0 && rows > 0) {
            void invoke("pane_pty_resize", {
              paneId: props.paneId,
              cols,
              rows,
            });
          }
        } catch {}
      }, 100);
      pendingTimers.push(fitGuardTimer);
      // mount-time auto focus · 修新建 tab 后焦点不在 terminal 的问题：
      // createEffect(props.active+focused) 在 onMount 异步之前已 run · 当时 term
      // 还没创建 · term?.focus() no-op。这里 term 已 ready · 补一次 focus。
      if (props.active && props.focused) {
        term.focus();
      }
    } catch (error: unknown) {
      const msg = error instanceof Error ? error.message : String(error);
      setSpawnError(msg);
      props.onError?.(props.paneId, msg);
    }
  });

  onCleanup(() => {
    // mount guard 立即设 false · 防止 listener 在后续 unlisten 调用之前
    // 还能 fire 一次 access props（stale reactive accessor 的根源）
    mounted = false;
    // clear 所有未 fire 的 setTimeout · 防 unmount 后 fire 引用 stale props/term
    for (const t of pendingTimers) {
      clearTimeout(t);
    }
    pendingTimers.length = 0;
    props.onUnregisterApi?.(props.paneId);
    resizeObserver?.disconnect();
    themeObserver?.disconnect();
    unlistenStdout?.();
    unlistenExited?.();
    // MVP-05 layout 切换 reload 修复 · onCleanup 默认**不 kill PTY** · 让 backend PTY 进程
    // 在 layout 切换（split↔single）时保留 · 新 PaneTerminal mount 时 spawn 命中
    // AlreadyRunning · catch 走 warm reattach + write snapshot 恢复显示。
    // 真"关闭" pane 的 PTY kill 由 caller（handlePaneClose / closeTab）在 invoke backend
    // 之前显式调 pane_pty_kill 完成 · 防 PTY 泄漏。
    try {
      activeWebglAddon?.dispose();
    } catch {}
    try {
      activeCanvasAddon?.dispose();
    } catch {}
    term?.dispose();
  });

  return (
    <div
      class={`vs-pane-terminal ${props.focused ? "is-focused" : ""} ${props.maximized ? "is-maximized-pane" : ""}`}
      data-pane-id={props.paneId}
      data-is-maximized={props.maximized ? "true" : "false"}
      role="region"
      aria-label={`Terminal pane · ${props.shell}`}
      onClick={() => props.onClick?.(props.paneId)}
    >
      <div
        class="vs-pane-maximized-chip"
        aria-hidden="true"
        data-visible={props.maximized ? "true" : "false"}
      />
      <div ref={hostRef} class="vs-pane-terminal-host" />
      <div class="vs-pane-actions" data-pane-actions>
        <button
          type="button"
          class="vs-pane-action-btn"
          title="右分屏 (⌘\\)"
          aria-label="右分屏"
          onClick={(e) => {
            e.stopPropagation();
            props.onSplit?.("horizontal", props.paneId);
          }}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect
              x="1.5"
              y="2.5"
              width="13"
              height="11"
              rx="1.5"
              stroke="currentColor"
              stroke-width="1.3"
            />
            <line
              x1="8"
              y1="2.5"
              x2="8"
              y2="13.5"
              stroke="currentColor"
              stroke-width="1.3"
            />
          </svg>
        </button>
        <button
          type="button"
          class="vs-pane-action-btn"
          title="下分屏 (⌘⇧\\)"
          aria-label="下分屏"
          onClick={(e) => {
            e.stopPropagation();
            props.onSplit?.("vertical", props.paneId);
          }}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <rect
              x="1.5"
              y="2.5"
              width="13"
              height="11"
              rx="1.5"
              stroke="currentColor"
              stroke-width="1.3"
            />
            <line
              x1="1.5"
              y1="8"
              x2="14.5"
              y2="8"
              stroke="currentColor"
              stroke-width="1.3"
            />
          </svg>
        </button>
        <button
          type="button"
          class="vs-pane-action-btn vs-pane-action-btn-danger"
          title="关闭 Pane (⌘⌃W)"
          aria-label="关闭 Pane"
          onClick={(e) => {
            e.stopPropagation();
            props.onClose?.(props.paneId);
          }}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <line
              x1="4"
              y1="4"
              x2="12"
              y2="12"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
            <line
              x1="12"
              y1="4"
              x2="4"
              y2="12"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
      {spawnError() ? (
        <div class="vs-pane-terminal-error">PTY 启动失败: {spawnError()}</div>
      ) : null}
    </div>
  );
};
