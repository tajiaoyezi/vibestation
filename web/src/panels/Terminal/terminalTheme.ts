import type { ITheme } from "@xterm/xterm";

export type TerminalThemeMode = "dark" | "light";

function createExtendedAnsi(overrides: Record<number, string>): string[] {
  const colors: string[] = [];
  for (const [rawIndex, color] of Object.entries(overrides)) {
    const index = Number(rawIndex);
    if (index >= 16 && index <= 255) {
      colors[index - 16] = color;
    }
  }
  return colors;
}

const LIGHT_EXTENDED_ANSI = createExtendedAnsi({
  // PSReadLine ListPredictionSelectedColor defaults to ANSI 48;5;238.
  // On a light terminal surface the default xterm 238 is a black-gray block.
  235: "#f6f8fb",
  236: "#f2f5f9",
  237: "#edf2f7",
  238: "#e8edf4",
  239: "#d9e2ec",
  240: "#cbd5e1",
});

export function resolveTerminalThemeMode(): TerminalThemeMode {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

export function createTerminalTheme(mode: TerminalThemeMode): ITheme {
  if (mode === "light") {
    return {
      background: "rgba(0, 0, 0, 0)",
      foreground: "#1f2937",
      cursor: "#1f2937",
      cursorAccent: "#f8fafc",
      selectionBackground: "rgba(37, 99, 235, 0.18)",
      selectionInactiveBackground: "rgba(37, 99, 235, 0.1)",
      black: "#1f2937",
      brightBlack: "#6b7280",
      red: "#dc2626",
      brightRed: "#ef4444",
      green: "#15803d",
      brightGreen: "#16a34a",
      yellow: "#b45309",
      brightYellow: "#d97706",
      blue: "#2563eb",
      brightBlue: "#3b82f6",
      magenta: "#7c3aed",
      brightMagenta: "#8b5cf6",
      cyan: "#0891b2",
      brightCyan: "#06b6d4",
      white: "#4b5563",
      // PSReadLine InlinePredictionColor defaults to ANSI 97;2;3.
      // Keep bright white dark enough that faint inline predictions remain visible.
      brightWhite: "#374151",
      extendedAnsi: LIGHT_EXTENDED_ANSI,
    };
  }

  return {
    background: "rgba(0, 0, 0, 0)",
    foreground: "#f5f7ff",
    cursor: "#f5f7ff",
    cursorAccent: "#11141b",
    selectionBackground: "rgba(120, 169, 255, 0.18)",
    selectionInactiveBackground: "rgba(120, 169, 255, 0.1)",
    black: "#0d1016",
    brightBlack: "#6c7485",
    red: "#ff7575",
    brightRed: "#ff7575",
    green: "#4cd38f",
    brightGreen: "#4cd38f",
    yellow: "#f6c24a",
    brightYellow: "#f6c24a",
    blue: "#6fa9ff",
    brightBlue: "#6fa9ff",
    magenta: "#b38cff",
    brightMagenta: "#b38cff",
    cyan: "#67d9ff",
    brightCyan: "#67d9ff",
    white: "#c3cad8",
    brightWhite: "#f5f7ff",
  };
}
