import { invoke } from "@tauri-apps/api/core";
import { Terminal as XTerm } from "@xterm/xterm";
import { onCleanup, onMount } from "solid-js";
import type { TabState } from "../../bindings";

export type RendererKind = "webgl" | "canvas" | "dom";
export type TabPhase = "idle" | "starting" | "running" | "exited" | "error";

export type TabRuntimeState = {
  phase: TabPhase;
  exitCode: number | null;
  spawnError: string | null;
  renderer: RendererKind | null;
  cols: number;
  rows: number;
};

export type ShortcutAction =
  | { kind: "new-tab" }
  | { kind: "close-tab" }
  | { kind: "previous-tab" }
  | { kind: "next-tab" }
  | { kind: "jump-tab"; index: number };

export const DEFAULT_PTY_COLS = 120;
export const DEFAULT_PTY_ROWS = 34;
const SCROLLBACK_BATCH_SIZE = 1000;

export const fetchScrollback = async (tabId: string): Promise<string[]> =>
  invoke<string[]>("tab_scrollback_fetch", {
    tabId,
    offset: 0,
    limit: 10_000,
  });

const writeBatch = (term: XTerm, data: string): Promise<void> =>
  new Promise((resolve) => {
    term.write(data, () => resolve());
  });

export const writeScrollbackToTerm = async (
  term: XTerm,
  lines: readonly string[],
): Promise<void> => {
  if (lines.length === 0) {
    return;
  }

  for (let index = 0; index < lines.length; index += SCROLLBACK_BATCH_SIZE) {
    const chunk = lines.slice(index, index + SCROLLBACK_BATCH_SIZE);
    await writeBatch(term, `${chunk.join("\r\n")}\r\n`);
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => resolve());
    });
  }
};

const isMacPlatform = (): boolean =>
  typeof navigator !== "undefined" &&
  navigator.platform.toLowerCase().includes("mac");

export const isEditableTarget = (target: EventTarget | null): boolean => {
  if (!(target instanceof HTMLElement)) return false;
  if (target.closest(".xterm")) return false;
  if (target.isContentEditable) return true;

  const tagName = target.tagName.toLowerCase();
  return tagName === "input" || tagName === "textarea" || tagName === "select";
};

const hasPrimaryModifier = (event: KeyboardEvent): boolean => {
  if (isMacPlatform()) {
    return event.metaKey && !event.ctrlKey;
  }

  return event.ctrlKey && !event.metaKey;
};

export const getShortcutAction = (
  event: KeyboardEvent,
  options?: { allowEditableTarget?: boolean },
): ShortcutAction | null => {
  if (!options?.allowEditableTarget && isEditableTarget(event.target)) {
    return null;
  }

  if (!hasPrimaryModifier(event) || event.altKey) {
    return null;
  }

  if (!event.shiftKey && (event.key === "t" || event.key === "T")) {
    return { kind: "new-tab" };
  }

  if (!event.shiftKey && (event.key === "w" || event.key === "W")) {
    return { kind: "close-tab" };
  }

  if (event.shiftKey && event.code === "BracketLeft") {
    return { kind: "previous-tab" };
  }

  if (event.shiftKey && event.code === "BracketRight") {
    return { kind: "next-tab" };
  }

  if (!event.shiftKey && /^[1-9]$/.test(event.key)) {
    return { kind: "jump-tab", index: Number(event.key) - 1 };
  }

  return null;
};

export const useKeybindings = (
  enabled: () => boolean,
  onAction: (action: ShortcutAction, event: KeyboardEvent) => void,
) => {
  const handleKeyDown = (event: KeyboardEvent) => {
    if (!enabled()) {
      return;
    }

    const action = getShortcutAction(event);
    if (!action) {
      return;
    }

    event.preventDefault();
    onAction(action, event);
  };

  onMount(() => {
    document.addEventListener("keydown", handleKeyDown);
  });

  onCleanup(() => {
    document.removeEventListener("keydown", handleKeyDown);
  });
};

export const pickAdjacentTabId = (
  tabs: readonly TabState[],
  currentTabId: string | null,
  direction: -1 | 1,
): string | null => {
  if (tabs.length === 0) {
    return null;
  }

  const currentIndex = currentTabId
    ? tabs.findIndex((tab) => tab.tabId === currentTabId)
    : -1;

  if (currentIndex === -1) {
    return tabs[0]?.tabId ?? null;
  }

  const nextIndex = (currentIndex + direction + tabs.length) % tabs.length;
  return tabs[nextIndex]?.tabId ?? null;
};

export const pickSiblingTabId = (
  tabs: readonly TabState[],
  closingTabId: string,
): string | null => {
  const index = tabs.findIndex((tab) => tab.tabId === closingTabId);
  if (index === -1) {
    return tabs[0]?.tabId ?? null;
  }

  return tabs[index + 1]?.tabId ?? tabs[index - 1]?.tabId ?? null;
};
