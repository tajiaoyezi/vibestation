import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const terminalPaneSource = readFileSync(
  resolve(process.cwd(), "src/panels/Terminal/TerminalPane.tsx"),
  "utf8",
);
const paneTerminalSource = readFileSync(
  resolve(process.cwd(), "src/panels/Terminal/PaneTerminal.tsx"),
  "utf8",
);
const typographyPath = resolve(
  process.cwd(),
  "src/panels/Terminal/terminalTypography.ts",
);
const typographySource = existsSync(typographyPath)
  ? readFileSync(typographyPath, "utf8")
  : "";

describe("terminal typography tuning", () => {
  it("keeps xterm options close to Windows Terminal defaults", () => {
    expect(typographySource).toContain("TERMINAL_LINE_HEIGHT = 1.2");
    expect(typographySource).toContain("TERMINAL_FONT_WEIGHT = 450");
    expect(typographySource).toContain("TERMINAL_FONT_WEIGHT_BOLD = 700");
  });

  it("prioritizes plain JetBrains Mono before Nerd Font fallback glyphs", () => {
    const plainIndex = typographySource.indexOf('"JetBrains Mono"');
    const nerdIndex = typographySource.indexOf('"JetBrainsMono NF"');

    expect(plainIndex).toBeGreaterThanOrEqual(0);
    expect(nerdIndex).toBeGreaterThanOrEqual(0);
    expect(plainIndex).toBeLessThan(nerdIndex);
  });

  it("uses the shared typography constants in both xterm entry points", () => {
    for (const source of [terminalPaneSource, paneTerminalSource]) {
      expect(source).toContain("buildTerminalFontStack(settings.fontFamily)");
      expect(source).toContain("lineHeight: TERMINAL_LINE_HEIGHT");
      expect(source).toContain("fontWeight: TERMINAL_FONT_WEIGHT");
      expect(source).toContain("fontWeightBold: TERMINAL_FONT_WEIGHT_BOLD");
      expect(source).not.toContain("lineHeight: 1.3");
    }
  });
});
