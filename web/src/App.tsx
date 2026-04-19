import { createSignal, onMount, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

/**
 * MVP-01 Phase A 欢迎页骨架 · 非最终 UI。
 *
 * - D（欢迎页 UI 精装 · Calm Studio token）延到后续 Phase
 * - E（崩溃恢复）等 SPIKE-04.5 · 后续 Phase
 * - 本组件仅验证：Tauri IPC 联通 + SolidJS 渲染 + CI 可过
 */
export const App: Component = () => {
  const [greeting, setGreeting] = createSignal<string>("Bootstrapping…");

  onMount(async () => {
    try {
      const msg = await invoke<string>("greet");
      setGreeting(msg);
    } catch (err) {
      // 防御性处理：IPC 失败时降级显示 · 不吞异常
      setGreeting(
        `IPC error: ${err instanceof Error ? err.message : String(err)}`,
      );
    }
  });

  const handleCreateWorkspace = () => {
    // MVP-02 接管 · 当前仅占位
    // eslint-disable-next-line no-console
    console.log("MVP-02 will implement Create Workspace flow");
  };

  return (
    <main class="vs-root">
      <section class="vs-welcome" aria-labelledby="vs-welcome-title">
        <h1 id="vs-welcome-title" class="vs-title">
          Vibestation
        </h1>
        <p class="vs-subtitle" data-testid="ipc-greeting">
          {greeting()}
        </p>
        <button
          type="button"
          class="vs-cta"
          aria-label="Create first workspace"
          onClick={handleCreateWorkspace}
        >
          Create first workspace
        </button>
        <p class="vs-phase-note">
          MVP-01 Phase A · 骨架验证 · UI 精装延到 Phase B
        </p>
      </section>
    </main>
  );
};
