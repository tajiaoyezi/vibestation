import { createSignal, onMount, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import "./styles.css";

/**
 * MVP-01 Phase B 欢迎页
 * Token 继承自 design/directions/1-calm-studio.html
 * MVP-02 接管 workspace 创建流程 · 当前 CTA 仅占位
 */

type IpcState =
  | { kind: "pending" }
  | { kind: "ok"; message: string }
  | { kind: "error"; message: string };

export const App: Component = () => {
  const [version, setVersion] = createSignal<string>("…");
  const [ipc, setIpc] = createSignal<IpcState>({ kind: "pending" });

  onMount(async () => {
    try {
      const v = await getVersion();
      setVersion(v);
    } catch {
      setVersion("unknown");
    }

    try {
      const msg = await invoke<string>("greet");
      setIpc({ kind: "ok", message: msg });
    } catch (err) {
      setIpc({
        kind: "error",
        message: err instanceof Error ? err.message : String(err),
      });
    }
  });

  const handleCreateWorkspace = () => {
    // MVP-02 接管 · 当前仅占位（eslint-disable 仅用于此单行占位日志）
    // eslint-disable-next-line no-console
    console.log("MVP-02 will implement Create Workspace flow");
  };

  return (
    <main class="vs-root">
      <section class="vs-welcome" aria-labelledby="vs-welcome-title">
        <div class="vs-mark" aria-hidden="true">
          <VibestationMark />
        </div>

        <div class="vs-title-block">
          <h1 id="vs-welcome-title" class="vs-title">
            Vibestation
          </h1>
          <p class="vs-tagline">
            Claude / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台
          </p>
          <span class="vs-version" aria-label={`Version ${version()}`}>
            <span class="vs-version-dot" aria-hidden="true" />v{version()} ·
            alpha
          </span>
        </div>

        <button
          type="button"
          class="vs-cta"
          aria-label="Create first workspace"
          onClick={handleCreateWorkspace}
        >
          Create first workspace
        </button>
      </section>

      <footer class="vs-diag" aria-label="Runtime diagnostics">
        <IpcIndicator state={ipc()} />
      </footer>
    </main>
  );
};

/**
 * 内联 SVG Logo · 源 design/logos/mark.svg
 * 内联的好处：颜色走 currentColor / CSS token · 体积小 · 不走 asset pipeline
 */
const VibestationMark: Component = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 64 64"
    width="64"
    height="64"
    role="img"
    aria-label="Vibestation mark"
  >
    <defs>
      <linearGradient id="m-grad" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0%" stop-color="oklch(0.72 0.16 240)" />
        <stop offset="100%" stop-color="oklch(0.58 0.2 260)" />
      </linearGradient>
    </defs>
    <rect x="4" y="4" width="56" height="56" rx="14" fill="url(#m-grad)" />
    <g
      fill="none"
      stroke="white"
      stroke-width="3.2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M20 24 L28 32 L20 40" />
      <line x1="32" y1="42" x2="46" y2="42" />
    </g>
    <circle cx="46" cy="22" r="2.5" fill="white" opacity="0.9" />
  </svg>
);

interface IpcIndicatorProps {
  state: IpcState;
}

const IpcIndicator: Component<IpcIndicatorProps> = (props) => {
  const label = () => {
    switch (props.state.kind) {
      case "pending":
        return "ipc: connecting…";
      case "ok":
        return `ipc: ${props.state.message}`;
      case "error":
        return `ipc error: ${props.state.message}`;
    }
  };

  const className = () =>
    props.state.kind === "error" ? "vs-diag-error" : "vs-diag-ok";

  return (
    <span class={className()} data-testid="ipc-indicator">
      {label()}
    </span>
  );
};
