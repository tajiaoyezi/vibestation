import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const readSource = (path: string) =>
  readFileSync(resolve(process.cwd(), path), "utf8");

describe("terminal lifecycle", () => {
  it("guards xterm dispose because WebGL teardown can throw during renderer cleanup", () => {
    const guardedDispose =
      /try\s*{\s*term\?\.dispose\(\);\s*}\s*catch\s*{\s*}/s;

    expect(readSource("src/panels/Terminal/TerminalPane.tsx")).toMatch(
      guardedDispose,
    );
    expect(readSource("src/panels/Terminal/PaneTerminal.tsx")).toMatch(
      guardedDispose,
    );
  });

  it("falls back from WebGL when xterm loses the renderer context", () => {
    const contextLossFallback =
      /webgl\.onContextLoss\(\(\)\s*=>[\s\S]*setupCanvasRenderer[\s\S]*term\.refresh/s;

    expect(readSource("src/panels/Terminal/TerminalPane.tsx")).toMatch(
      contextLossFallback,
    );
  });

  it("restores pane snapshots before opening xterm during layout remount", () => {
    const source = readSource("src/panels/Terminal/PaneTerminal.tsx");
    const snapshotRestoreIndex = source.indexOf(
      "paneSnapshots.get(props.paneId)",
    );
    const openIndex = source.indexOf("term.open(hostRef)");

    expect(snapshotRestoreIndex).toBeGreaterThan(-1);
    expect(openIndex).toBeGreaterThan(-1);
    expect(snapshotRestoreIndex).toBeLessThan(openIndex);
  });

  it("does not paint stale DOM snapshots over panes during layout remounts", () => {
    const paneSource = readSource("src/panels/Terminal/PaneTerminal.tsx");
    const terminalSource = readSource("src/panels/Terminal/Terminal.tsx");
    const cssSource = readSource("src/panels/Terminal/styles.css");

    expect(paneSource).not.toContain("serializeHtml");
    expect(paneSource).not.toContain("vs-pane-terminal-snapshot");
    expect(paneSource).not.toContain("snapshotHtml");
    expect(paneSource).not.toContain("setSnapshotHtml");
    expect(paneSource).not.toContain("innerHTML={html()}");
    expect(paneSource).not.toContain("includeGlobalBackground");
    expect(terminalSource).not.toContain("serializeHtml?.()");
    expect(cssSource).not.toContain(".vs-pane-terminal-snapshot");
  });

  it("keeps text snapshot fallback as replay data instead of a visible overlay", () => {
    const paneSource = readSource("src/panels/Terminal/PaneTerminal.tsx");
    const terminalSource = readSource("src/panels/Terminal/Terminal.tsx");

    expect(paneSource).toContain("serializeText");
    expect(paneSource).toContain("snapshotTextAsAnsi");
    expect(paneSource).not.toContain("snapshotText()");
    expect(paneSource).not.toContain("setSnapshotText");
    expect(paneSource).not.toContain("MIN_SNAPSHOT_OVERLAY_MS");
    expect(terminalSource).toContain("serializeText?.()");
  });

  it("stores and replays fallback text snapshots when ANSI serialization is empty", () => {
    const paneSource = readSource("src/panels/Terminal/PaneTerminal.tsx");
    const terminalSource = readSource("src/panels/Terminal/Terminal.tsx");

    expect(terminalSource).toContain("const text = api?.serializeText?.()");
    expect(terminalSource).toContain("if (snapshot || text)");
    expect(paneSource).toContain("snapshotTextAsAnsi");
    expect(paneSource).toContain("snapshotTextAsAnsi(snapshot)");
  });

  it("uses the stable canvas renderer for remount-heavy pane mode", () => {
    const paneSource = readSource("src/panels/Terminal/PaneTerminal.tsx");

    expect(paneSource).not.toContain('from "@xterm/addon-webgl"');
    expect(paneSource).not.toContain("new WebglAddon()");
    expect(paneSource).not.toContain("webgl.onContextLoss");
    expect(paneSource).toContain("setupCanvasRenderer(term, paneId)");
  });
});
