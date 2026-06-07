import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const css = readFileSync(resolve(process.cwd(), "src/styles.css"), "utf8");
const terminalCss = readFileSync(
  resolve(process.cwd(), "src/panels/Terminal/styles.css"),
  "utf8",
);
const terminalPaneSource = readFileSync(
  resolve(process.cwd(), "src/panels/Terminal/TerminalPane.tsx"),
  "utf8",
);
const paneTerminalSource = readFileSync(
  resolve(process.cwd(), "src/panels/Terminal/PaneTerminal.tsx"),
  "utf8",
);
const terminalThemeSource = readFileSync(
  resolve(process.cwd(), "src/panels/Terminal/terminalTheme.ts"),
  "utf8",
);
const tauriConfig = JSON.parse(
  readFileSync(resolve(process.cwd(), "../crates/app/tauri.conf.json"), "utf8"),
);

function backgroundRootRuleBlock(): string {
  const pattern = /\n#root\s*\{/g;
  const matches = [...css.matchAll(pattern)];
  const match = matches.find((candidate) => css[candidate.index! - 1] !== ",");
  expect(match).not.toBeNull();
  const bodyStart = css.indexOf("{", match!.index);
  const bodyEnd = css.indexOf("}", bodyStart);
  return css.slice(bodyStart + 1, bodyEnd);
}

describe("background opacity styling", () => {
  it("does not fade the whole application tree", () => {
    const rootBlock = backgroundRootRuleBlock();

    expect(rootBlock).not.toMatch(/opacity:\s*var\(--bg-opacity\)/);
    expect(rootBlock).toMatch(/background:\s*var\(--surface-0\)/);
  });

  it("routes background opacity into application surface colors", () => {
    expect(css).toMatch(
      /--surface-0:\s*color-mix\(\s*in oklch,\s*var\(--bg-0\)\s*calc\(var\(--bg-opacity\)\s*\*\s*100%\),\s*transparent\s*\)/,
    );
    expect(css).toMatch(
      /--surface-1:\s*color-mix\(\s*in oklch,\s*var\(--bg-1\)\s*calc\(var\(--bg-opacity\)\s*\*\s*100%\),\s*transparent\s*\)/,
    );
    expect(css).toContain("background: var(--surface-0)");
    expect(css).toContain("background: var(--surface-1)");
    expect(css).toContain("background: var(--surface-2)");
  });

  it("allows the native window backdrop to show through transparent surfaces", () => {
    expect(tauriConfig.app.windows[0].transparent).toBe(true);
  });

  it("routes terminal backgrounds through opacity-controlled surfaces", () => {
    expect(terminalCss).not.toMatch(
      /background:\s*[^;{]*\n\s*[^;{]*var\(--bg-1\)\s*;/,
    );
    expect(terminalCss).toContain("var(--surface-1)");

    expect(terminalThemeSource).toContain('background: "rgba(0, 0, 0, 0)"');

    for (const source of [terminalPaneSource, paneTerminalSource]) {
      expect(source).toContain("allowTransparency: true");
      expect(source).toContain(
        "createTerminalTheme(resolveTerminalThemeMode())",
      );
      expect(source).not.toContain('background: read("--bg-1"');
      expect(source).toContain('attributeFilter: ["data-theme", "style"]');
    }
  });

  it("routes tinted chrome backgrounds through opacity-controlled surfaces", () => {
    expect(css).toContain(
      "background: color-mix(in oklch, var(--accent) 14%, var(--surface-1))",
    );
    expect(css).not.toContain(
      "background: color-mix(in oklch, var(--accent) 14%, var(--bg-1))",
    );

    expect(terminalCss).toContain(
      "background: color-mix(in oklch, var(--surface-1) 92%, transparent)",
    );
    expect(terminalCss).toContain(
      "background: color-mix(in oklch, var(--accent) 12%, var(--surface-1))",
    );
    expect(terminalCss).not.toContain(
      "background: color-mix(in oklch, var(--bg-1) 92%, transparent)",
    );
    expect(terminalCss).not.toContain(
      "background: color-mix(in oklch, var(--accent) 12%, var(--bg-1))",
    );
  });
});
