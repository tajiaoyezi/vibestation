import { describe, expect, it } from "vitest";
import {
  buildFontStack,
  buildUiFontStack,
  primaryFontFamily,
  quoteFontFamily,
} from "../../src/utils/font-stack";

describe("font-stack", () => {
  it("quoteFontFamily leaves CSS generic keywords unquoted", () => {
    expect(quoteFontFamily("system-ui")).toBe("system-ui");
    expect(quoteFontFamily("ui-monospace")).toBe("ui-monospace");
    expect(quoteFontFamily("monospace")).toBe("monospace");
    expect(quoteFontFamily("-apple-system")).toBe("-apple-system");
  });

  it("quoteFontFamily quotes named font families", () => {
    expect(quoteFontFamily("Inter")).toBe('"Inter"');
    expect(quoteFontFamily("JetBrains Mono")).toBe('"JetBrains Mono"');
  });

  it("primaryFontFamily takes the first family from legacy stacks", () => {
    expect(
      primaryFontFamily(
        "JetBrains Mono, DejaVu Sans Mono, Ubuntu Mono, monospace",
      ),
    ).toBe("JetBrains Mono");
  });

  it("buildUiFontStack keeps system-ui as a generic keyword", () => {
    expect(buildUiFontStack("system-ui")).toMatch(/^system-ui,/);
    expect(buildUiFontStack("system-ui")).not.toContain('"system-ui"');
  });

  it("buildFontStack deduplicates primary and fallbacks", () => {
    expect(buildFontStack("Inter", ["Inter", "sans-serif"])).toBe(
      '"Inter", sans-serif',
    );
  });
});
