import { join } from "node:path";
import { describe, expect, it } from "vitest";

// Vitest 可能改写 import.meta.url（非 file:）· 从 web/ 工作区向上解析仓库根
const REPO_ROOT = join(process.cwd(), "..");
const SCRIPT = join(REPO_ROOT, "scripts", "setup-git-hooks.mjs");

// 动态 import（ESM · .mjs）· 跨平台纯函数 · 不依赖 shell 重定向
async function loadScript() {
  // file:// URL 形式以兼容 Windows 绝对路径（C:\... → file:///C:/...）
  const { pathToFileURL } = await import("node:url");
  return import(pathToFileURL(SCRIPT).href);
}

describe("setup-git-hooks.mjs", () => {
  it("SCEN-5.3.1: buildHooksConfigCommand 构造正确的 git config 命令（无 shell 重定向）", async () => {
    const mod = await loadScript();
    const { command, args } = mod.buildHooksConfigCommand();
    expect(command).toBe("git");
    expect(args).toEqual(["config", "core.hooksPath", ".githooks"]);
    // 不得含任何 shell 重定向 / 操作符（跨平台关键：Windows PowerShell 不解析 2>/dev/null）
    const joined = [command, ...args].join(" ");
    expect(joined).not.toContain("2>/dev/null");
    expect(joined).not.toContain("||");
    expect(joined).not.toContain(">");
  });

  it("SCEN-5.3.1: setupGitHooks 在 runner 抛错时静默吞掉（对齐原 || true 容错 · install 不阻断）", async () => {
    const mod = await loadScript();
    // 注入一个必抛错的 runner（模拟 git 不存在 / config 失败）
    const throwingRunner = () => {
      throw new Error("simulated git failure");
    };
    // 不得向上抛 · 返回 false（失败但不阻断）
    let threw = false;
    let result;
    try {
      result = mod.setupGitHooks(throwingRunner);
    } catch {
      threw = true;
    }
    expect(threw).toBe(false);
    expect(result).toBe(false);
  });

  it("SCEN-5.3.1: setupGitHooks 在 runner 成功时返回 true 且以正确参数调用 git", async () => {
    const mod = await loadScript();
    const calls: Array<{ command: string; args: string[] }> = [];
    const okRunner = (command: string, args: string[]) => {
      calls.push({ command, args });
    };
    const result = mod.setupGitHooks(okRunner);
    expect(result).toBe(true);
    expect(calls).toHaveLength(1);
    expect(calls[0].command).toBe("git");
    expect(calls[0].args).toEqual(["config", "core.hooksPath", ".githooks"]);
  });
});
