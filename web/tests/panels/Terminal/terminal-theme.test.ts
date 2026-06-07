import { describe, expect, it } from "vitest";
import { createTerminalTheme } from "../../../src/panels/Terminal/terminalTheme";

describe("terminal theme", () => {
  it("uses a light prediction background for PSReadLine selected suggestions", () => {
    const theme = createTerminalTheme("light");
    const psReadLineSelectedPrediction = theme.extendedAnsi?.[238 - 16];

    expect(psReadLineSelectedPrediction).toBe("#e8edf4");
    expect(psReadLineSelectedPrediction).not.toMatch(/^#0|^#1|^#2|^#3/);
  });

  it("keeps light theme ANSI text readable on the light terminal surface", () => {
    const theme = createTerminalTheme("light");

    expect(theme.background).toBe("rgba(0, 0, 0, 0)");
    expect(theme.foreground).toBe("#1f2937");
    expect(theme.black).toBe("#1f2937");
    expect(theme.brightBlack).toBe("#6b7280");
    expect(theme.brightWhite).toBe("#374151");
  });

  it("keeps PSReadLine inline predictions visible on light theme", () => {
    const theme = createTerminalTheme("light");

    // PowerShell PSReadLine defaults InlinePredictionColor to SGR 97;2;3
    // (bright white + faint + italic). brightWhite must start dark enough that
    // xterm's faint rendering still remains visible on a light surface.
    expect(theme.brightWhite).toBe("#374151");
    expect(theme.brightWhite).not.toBe(theme.brightBlack);
  });
});
