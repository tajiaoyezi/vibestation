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
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal as XTerm } from "@xterm/xterm";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import type {
  PanePtyExitedEvent,
  PanePtySpawnRequest,
  PanePtyStdoutEvent,
  PaneState,
  SpawnResult,
  SplitDir,
} from "../../bindings";
import { useSettings } from "../../stores/settings";
import {
  createPaneContextMenu,
  PaneContextMenuOverlay,
} from "./PaneContextMenu";
import { detachPane } from "../../lib/pane-detach";
import { formatShortcut } from "../../lib/format-shortcut";
import { setPopToExternalRequest } from "../../lib/external-term";
import { usePaneLinks } from "../../stores/paneLinks-context";
import { PaneLinkChip } from "./PaneLinkChip";
import { RunnerSourceBadge } from "./RunnerSourceBadge";
import { LinkManagePopover } from "./LinkManagePopover";
import { PaneLinkErrorState } from "./PaneLinkErrorState";
import {
  PaneLinkCreateMenu,
  type PaneLinkCreateRequest,
} from "./PaneLinkCreateMenu";
import { usePaneDrafts } from "../../stores/paneDrafts-context";
import { FailureCallout } from "./FailureCallout";
import { PaneDraftComposer } from "./PaneDraftComposer";
import { buildPreviewRequest } from "./paneFailurePreview";
import type { PaneFailureCallout } from "../../stores/paneLinks";
import {
  buildTerminalFontStack,
  TERMINAL_FONT_WEIGHT,
  TERMINAL_FONT_WEIGHT_BOLD,
  TERMINAL_LINE_HEIGHT,
} from "./terminalTypography";
import { createTerminalTheme, resolveTerminalThemeMode } from "./terminalTheme";
import {
  MAX_TERMINAL_PANE_COLUMNS,
  MAX_TERMINAL_PANE_ROWS,
  MAX_TERMINAL_PANES,
} from "./paneSplitLimits";

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
  serializeText: () => string;
};

/**
 * 模块级 snapshot store · key=paneId · value=SerializeAddon 输出的 ANSI / text 快照。
 * 在 layout 切换前由 Terminal.tsx 的 handlePaneSplit / handlePaneClose 写入 ·
 * 新 PaneTerminal mount 时在 Terminal.open() 前 write 回 xterm，避免额外 DOM 遮罩污染画面。
 */
export type PaneSnapshot = {
  ansi: string;
  text?: string;
};

export const paneSnapshots = new Map<string, PaneSnapshot | string>();

const snapshotTextAsAnsi = (
  snapshot: PaneSnapshot | string | undefined,
): string => {
  if (!snapshot || typeof snapshot === "string") return "";
  const text = snapshot.text?.trimEnd();
  return text && /\S/.test(text) ? text.replace(/\r?\n/g, "\r\n") : "";
};

const snapshotAnsi = (snapshot: PaneSnapshot | string | undefined): string =>
  typeof snapshot === "string"
    ? snapshot
    : (snapshot?.ansi ?? "") || snapshotTextAsAnsi(snapshot);

const serializeVisibleText = (term?: XTerm): string => {
  const buffer = term?.buffer.active;
  if (!term || !buffer) return "";
  const start = Math.max(0, buffer.viewportY);
  const end = Math.min(buffer.length, start + term.rows);
  const lines: string[] = [];
  for (let row = start; row < end; row += 1) {
    lines.push(buffer.getLine(row)?.translateToString(true) ?? "");
  }
  return lines.join("\n").trimEnd();
};

type PaneTerminalProps = {
  paneId: string;
  shell: string;
  cwd: string;
  /** MVP-18 Wave 2 · scope pane-link selectors / create requests to this workspace. */
  workspaceId: string;
  /** MVP-18 Wave 2 · other panes in the tab · candidate AI parents for link create. */
  siblingPanes: PaneState[];
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
  canSplitRight?: boolean;
  canSplitDown?: boolean;
  onSplit?: (direction: SplitDir, paneId: string) => void;
  onClose?: (paneId: string) => void;
  // cmd+V paste guard 路径 · 与 menu paste / legacy TerminalPane DOM paste 对齐：
  // PaneTerminal 不知道 workspace skip state · 把 paste 委托给上层决策（multiline check
  // + skipPasteConfirmByWorkspace） · 上层根据需要 setPendingPaste 弹 confirm dialog 或
  // 直接 paneApi.paste。未提供 callback 时 fallback 直接 paste（向后兼容）。
  onPasteRequest?: (paneId: string, text: string) => void;
};

type ActiveRenderers = {
  canvas?: CanvasAddon;
};

const setupCanvasRenderer = (term: XTerm, paneId: string): ActiveRenderers => {
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

const setupRenderer = (term: XTerm, paneId: string): ActiveRenderers =>
  setupCanvasRenderer(term, paneId);

export const PaneTerminal: Component<PaneTerminalProps> = (props) => {
  const { settings } = useSettings();
  const [spawnError, setSpawnError] = createSignal<string | null>(null);

  const paneCount = () => props.siblingPanes.length + 1;
  const fallbackCanSplit = () => paneCount() < MAX_TERMINAL_PANES;
  const canSplitRight = () => props.canSplitRight ?? fallbackCanSplit();
  const canSplitDown = () => props.canSplitDown ?? fallbackCanSplit();
  const splitLimitTitle = (direction: SplitDir): string => {
    if (paneCount() >= MAX_TERMINAL_PANES) {
      return `已达总上限（${MAX_TERMINAL_PANES} 个 Pane）`;
    }
    return direction === "horizontal"
      ? `左右分屏已达上限（最多 ${MAX_TERMINAL_PANE_COLUMNS} 列）`
      : `上下分屏已达上限（最多 ${MAX_TERMINAL_PANE_ROWS} 行）`;
  };

  // ── MVP-18 Wave 2 · pane linking integration ──────────────────────────────
  const paneLinks = usePaneLinks();
  const linkSel = paneLinks.selectorsFor(() => props.workspaceId);
  const paneDrafts = usePaneDrafts();
  // Failure callouts surface on the AI *parent* pane that receives the feedback.
  const myFailureCallouts = createMemo(() =>
    linkSel.failureCallouts().filter((c) => c.parentPaneId === props.paneId),
  );
  const handleInsertFailure = (callout: PaneFailureCallout) => {
    void paneLinks
      .previewPrompt(buildPreviewRequest(callout))
      .then((result) => {
        // D.2 / D.5 — insertFragment appends to the parent pane draft, or
        // raises a merge-preview that PaneDraftComposer renders reactively.
        paneDrafts.insertFragment(callout.parentPaneId, result.promptFragment);
      })
      .catch((err: unknown) => {
        console.warn("[mvp-18] previewPrompt failed:", err);
      });
  };
  const handleDismissCallout = (commandRunId: string) => {
    paneLinks.store.dismissCallout(props.workspaceId, commandRunId);
  };
  const handleSendDraft = (_paneId: string, text: string) => {
    if (text === "") return;
    // Flush the composed draft into this (AI) pane's PTY stdin, then clear it.
    term?.paste(text);
    paneDrafts.clearDraft(props.paneId);
  };
  const [linkManageOpen, setLinkManageOpen] = createSignal(false);
  const [linkCreateOpen, setLinkCreateOpen] = createSignal(false);

  // Chip must reflect any lifecycle status (enabled / disabled / stale), so use
  // the raw workspace links, not the active-filtered linkForPane selector.
  const myLink = createMemo(() =>
    linkSel
      .links()
      .find(
        (l) =>
          l.parentPaneId === props.paneId || l.childPaneId === props.paneId,
      ),
  );
  const linkRole = (): "parent" | "child" | null => {
    const link = myLink();
    if (!link) return null;
    return link.parentPaneId === props.paneId ? "parent" : "child";
  };
  const myLinks = createMemo(() =>
    linkSel
      .links()
      .filter(
        (l) =>
          l.parentPaneId === props.paneId || l.childPaneId === props.paneId,
      ),
  );
  const candidatePanes = createMemo(() =>
    props.siblingPanes.map((p) => ({
      paneId: p.paneId,
      label: `${p.shell.split("/").pop() || p.shell} — ${p.cwd}`,
    })),
  );

  const handleCreateLink = (req: PaneLinkCreateRequest) => {
    void paneLinks
      .createLink({ workspaceId: props.workspaceId, ...req })
      .then(() => setLinkCreateOpen(false))
      .catch((err: unknown) => {
        // Recoverable errors are designed to arrive via pane:link-error, which
        // the backend does not emit yet (MVP-18 escalate · see review-prep);
        // surface in console until that lands so the failure is not silent.
        console.warn("[mvp-18] createLink failed:", err);
      });
  };
  const handleToggleEnabled = (linkId: string, enabled: boolean) => {
    void paneLinks
      .setEnabled({ workspaceId: props.workspaceId, linkId, enabled })
      .catch((err: unknown) => {
        console.warn("[mvp-18] setEnabled failed:", err);
      });
  };
  const handleUnlink = (linkId: string) => {
    void paneLinks
      .unlink({ workspaceId: props.workspaceId, linkId })
      .catch((err: unknown) => {
        console.warn("[mvp-18] unlink failed:", err);
      });
  };
  // ──────────────────────────────────────────────────────────────────────────
  // MVP-17 Phase C wiring · 右键菜单 overlay state · 替代 backend native menu_show_terminal
  const paneMenu = createPaneContextMenu();
  let hostRef: HTMLDivElement | undefined;
  let term: XTerm | undefined;
  let fitAddon: FitAddon | undefined;
  let serializeAddon: SerializeAddon | undefined;
  let resizeObserver: ResizeObserver | undefined;
  let unlistenStdout: UnlistenFn | undefined;
  let unlistenExited: UnlistenFn | undefined;
  // theme 切换需 dispose + reload addon · Canvas atlas 缓存 glyph 不会自动更新
  let activeCanvasAddon: CanvasAddon | undefined;
  let themeObserver: MutationObserver | undefined;
  let paneContainerRef: HTMLDivElement | undefined;
  // MVP-20 BUG-001 · 收集所有 setTimeout · onCleanup 时 clear 防 unmount 后 fire
  // 触发 SolidJS <Show> stale accessor 警告（异步 callback 在 component unmount 后
  // 访问 reactive props 的典型 race）。
  const pendingTimers: Array<ReturnType<typeof setTimeout>> = [];
  let mounted = true;

  // 同步 xterm theme · CanvasAddon 缓存 atlas 必须 dispose+reload 才能用新色
  // （xterm 5.x term.options.theme 单独设不够 · clearTextureAtlas 实测也不彻底 ·
  //  TerminalPane 已验证 · 此处对齐）
  const syncTheme = () => {
    if (!term) return;
    term.options.theme = createTerminalTheme(resolveTerminalThemeMode());
    if (activeCanvasAddon) {
      try {
        activeCanvasAddon.dispose();
      } catch {}
      activeCanvasAddon = undefined;
    }
    const renderers = setupRenderer(term, props.paneId);
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

  // 字体栈 · 普通 JetBrains Mono 优先，Nerd Font 仅作图标兜底。
  const fontStack = () => buildTerminalFontStack(settings.fontFamily);

  createEffect(() => {
    if (props.active) {
      queueFit();
      if (props.focused) {
        term?.focus();
      } else {
        // 失焦时 blur xterm · 否则旧 pane 的 xterm 仍持 DOM focus ·
        // 浏览器不会自动把焦点移到新 pane 的 xterm · 导致无法切换
        hostRef
          ?.querySelector<HTMLInputElement>(".xterm-helper-textarea")
          ?.blur();
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
    serializeAddon = new SerializeAddon();
    term.loadAddon(serializeAddon);
    term.loadAddon(new Unicode11Addon());
    term.unicode.activeVersion = "11";
    term.loadAddon(new WebLinksAddon());

    // layout split/close 会 remount affected PaneTerminal。按 SerializeAddon 建议，
    // 在 Terminal.open() 之前重放 ANSI/text snapshot；不再使用 DOM snapshot 遮罩，
    // 避免旧画面在 resize/remount 期间覆盖当前 xterm。
    const snapshot = paneSnapshots.get(props.paneId);
    if (snapshot) {
      paneSnapshots.delete(props.paneId);
      term.write(snapshotAnsi(snapshot));
    }

    if (!hostRef) return;
    term.open(hostRef);
    const renderers = setupRenderer(term, props.paneId);
    activeCanvasAddon = renderers.canvas;
    if (snapshot) {
      try {
        term.refresh(0, term.rows - 1);
      } catch {}
    }

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
      serializeText: () => serializeVisibleText(term),
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

    // 监听 data-theme / style attribute 变化 · 触发 syncTheme · 跟 TerminalPane fix8 一致。
    // style 覆盖 Background opacity/blur 这类 CSS var 更新；data-theme 覆盖主题切换。
    // 用 attribute observer 而非 settings.theme reactive · 避免 setSettings/applyCssVars
    // 顺序 race（applyCssVars 设 CSS vars/data-theme · createTheme 读最新值 · 有保证）
    themeObserver = new MutationObserver(() => syncTheme());
    themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme", "style"],
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

  // mousedown capture: 点击 xterm canvas 时 xterm 可能 stopPropagation 吞 click ·
  // capture 阶段 mousedown 在 xterm 处理之前触发 · 保证 pane 焦点切换不被拦截
  onMount(() => {
    paneContainerRef?.addEventListener(
      "mousedown",
      () => {
        props.onClick?.(props.paneId);
      },
      true,
    ); // true = capture phase
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
      activeCanvasAddon?.dispose();
    } catch {}
    try {
      term?.dispose();
    } catch {}
  });

  return (
    <div
      ref={paneContainerRef}
      class={`vs-pane-terminal ${props.focused ? "is-focused" : ""} ${props.maximized ? "is-maximized-pane" : ""}`}
      data-pane-id={props.paneId}
      data-is-maximized={props.maximized ? "true" : "false"}
      role="region"
      aria-label={`Terminal pane · ${props.shell}`}
      onClick={() => props.onClick?.(props.paneId)}
      onContextMenu={(e) => {
        // MVP-17 Phase C wiring · 右键菜单：Detach / Pop to External / Close
        // 不抑制系统 xterm 选区 · xterm host 内的右键由 xterm 自处理（target 检测）
        const target = e.target as Element | null;
        if (
          target?.closest?.(
            ".xterm-rows, .xterm-screen, .xterm-helper-textarea",
          )
        ) {
          return; // xterm 内容区 · 让 xterm 自处理（如复制选区）
        }
        e.preventDefault();
        e.stopPropagation();
        const paneId = props.paneId;
        paneMenu.show(e.clientX, e.clientY, [
          {
            id: "pop-external",
            label: "Pop to External…",
            shortcut: formatShortcut("⌘⇧O", "Ctrl+Shift+O"),
            onClick: () => setPopToExternalRequest({ paneId }),
          },
          {
            id: "detach",
            label: "Detach Pane",
            shortcut: formatShortcut("⌘⇧D", "Ctrl+Shift+D"),
            onClick: () => {
              void detachPane({ paneId }).catch((err) => {
                console.warn("[mvp-17] detachPane failed:", err);
              });
            },
          },
          {
            id: "sep-1",
            label: "",
            separator: true,
            onClick: () => {},
          },
          {
            id: "close",
            label: "Close",
            shortcut: formatShortcut("⌘⌃W", "Ctrl+Alt+W"),
            onClick: () => props.onClose?.(paneId),
          },
        ]);
      }}
    >
      <div
        class="vs-pane-maximized-chip"
        aria-hidden="true"
        data-visible={props.maximized ? "true" : "false"}
      />
      <div ref={hostRef} class="vs-pane-terminal-host" />
      <div class="vs-pane-actions" data-pane-actions>
        <Show when={myLink()}>
          {(link) => (
            <PaneLinkChip
              link={link()}
              role={linkRole() ?? "child"}
              onClick={() => setLinkManageOpen(true)}
            />
          )}
        </Show>
        <button
          type="button"
          class="vs-pane-action-btn"
          title="链接 Pane 失败反馈到 AI Pane"
          aria-label="Link pane failures to an AI pane"
          onClick={(e) => {
            e.stopPropagation();
            setLinkCreateOpen(true);
          }}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path
              d="M6.5 9.5a2.5 2.5 0 0 1 0-3.5l2-2a2.5 2.5 0 0 1 3.5 3.5l-1 1M9.5 6.5a2.5 2.5 0 0 1 0 3.5l-2 2a2.5 2.5 0 0 1-3.5-3.5l1-1"
              stroke="currentColor"
              stroke-width="1.3"
              stroke-linecap="round"
            />
          </svg>
        </button>
        <button
          type="button"
          class={`vs-pane-action-btn${!canSplitRight() ? " is-disabled" : ""}`}
          title={
            canSplitRight()
              ? `右分屏 (${formatShortcut("⌘\\", "Ctrl+\\")})`
              : splitLimitTitle("horizontal")
          }
          aria-label="右分屏"
          disabled={!canSplitRight()}
          onClick={(e) => {
            e.stopPropagation();
            if (canSplitRight()) {
              props.onSplit?.("horizontal", props.paneId);
            }
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
          class={`vs-pane-action-btn${!canSplitDown() ? " is-disabled" : ""}`}
          title={
            canSplitDown()
              ? `下分屏 (${formatShortcut("⌘⇧\\", "Ctrl+Shift+\\")})`
              : splitLimitTitle("vertical")
          }
          aria-label="下分屏"
          disabled={!canSplitDown()}
          onClick={(e) => {
            e.stopPropagation();
            if (canSplitDown()) {
              props.onSplit?.("vertical", props.paneId);
            }
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
          title={`关闭 Pane (${formatShortcut("⌘⌃W", "Ctrl+Alt+W")})`}
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
      {/* MVP-17 Phase C wiring · 右键菜单 overlay · 在 div 内不影响 stacking */}
      <PaneContextMenuOverlay state={paneMenu.state} onHide={paneMenu.hide} />
      {/* MVP-18 Wave 2 · pane link UX overlays */}
      <Show when={linkRole() === "child" ? myLink() : null}>
        {(link) => <RunnerSourceBadge link={link()} />}
      </Show>
      <LinkManagePopover
        open={linkManageOpen()}
        links={myLinks()}
        currentPaneId={props.paneId}
        onClose={() => setLinkManageOpen(false)}
        onToggleEnabled={handleToggleEnabled}
        onUnlink={handleUnlink}
      />
      <PaneLinkCreateMenu
        open={linkCreateOpen()}
        currentPaneId={props.paneId}
        candidatePanes={candidatePanes()}
        onClose={() => setLinkCreateOpen(false)}
        onCreate={handleCreateLink}
      />
      <PaneLinkErrorState
        error={paneLinks.store.lastError}
        onDismiss={() => paneLinks.store.clearError()}
      />
      {/* MVP-18 Wave 2-3 · failure feedback + draft composer on the AI parent pane */}
      <Show when={linkRole() === "parent"}>
        <For each={myFailureCallouts()}>
          {(callout) => (
            <FailureCallout
              callout={callout}
              onDismiss={handleDismissCallout}
              onInsert={handleInsertFailure}
            />
          )}
        </For>
        <PaneDraftComposer
          paneId={props.paneId}
          drafts={paneDrafts}
          onSend={handleSendDraft}
        />
      </Show>
    </div>
  );
};
