import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { Terminal } from "@xterm/xterm";

export class XTermWrapper {
  readonly terminal: Terminal;
  readonly fitAddon: FitAddon;
  private opened = false;
  private host?: HTMLElement;

  constructor() {
    this.terminal = new Terminal({
      convertEol: false,
      cursorBlink: false,
      allowProposedApi: false,
      fontFamily: "SFMono-Regular, ui-monospace, Menlo, Monaco, monospace",
      fontSize: 13,
      lineHeight: 1.18,
      scrollback: 5000,
      theme: {
        background: "#0f1115",
        foreground: "#f5f7ff",
        black: "#11131a",
        blue: "#78a9ff",
        green: "#7ad887",
        magenta: "#d49cff",
        red: "#ff8a8a",
        yellow: "#ffd166",
      },
    });

    this.fitAddon = new FitAddon();
    this.terminal.loadAddon(this.fitAddon);
    this.terminal.loadAddon(new WebLinksAddon());
  }

  attach(host: HTMLElement): void {
    this.host = host;
    if (!this.opened) {
      this.terminal.open(host);
      this.opened = true;
      this.fit();
      return;
    }
    this.fit();
  }

  fit(): void {
    if (!this.host) return;
    queueMicrotask(() => {
      try {
        this.fitAddon.fit();
      } catch {
        // hidden host can throw; retry on next activation
      }
    });
  }

  write(data: string, renderDelayMs = 0): void {
    if (!data) return;
    const dispatch = () => this.terminal.write(data);
    if (renderDelayMs > 0) {
      window.setTimeout(dispatch, renderDelayMs);
      return;
    }
    dispatch();
  }

  getPreview(lines = 4): string {
    const buffer = this.terminal.buffer.active;
    const start = Math.max(0, buffer.length - lines);
    const out: string[] = [];
    for (let index = start; index < buffer.length; index += 1) {
      const line = buffer.getLine(index);
      if (!line) continue;
      const translated = line.translateToString(true).trim();
      if (translated) out.push(translated);
    }
    return out.join("\n");
  }

  dispose(): void {
    this.terminal.dispose();
  }
}
