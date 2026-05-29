#!/usr/bin/env node
// Task 5.3 · prepare-titlebar
// 跨平台 git hooks 配置脚本（替代原 package.json prepare 的 bash 重定向
// "git config core.hooksPath .githooks 2>/dev/null || true"）。
//
// 原写法依赖 POSIX `2>/dev/null` 重定向 + `|| true`，在 Windows PowerShell/cmd.exe 下：
//   - `2>/dev/null` 会把 stderr 重定向到名为 `nul` 的文件（误创建文件 · 语义错）
//   - `|| true` 语义与 POSIX 不同
// 导致 Windows 开发者 clone + `pnpm install` 后 git hook 配置静默失败，
// commit 绕过 pre-push 分支保护（CLAUDE.md §禁区 机械防护在 Windows 失效）。
//
// 本脚本用 node child_process 直接调 git（无 shell 重定向），try/catch 吞错
// （对齐原 `|| true` 容错语义：失败不阻断 `pnpm install`），跨 Windows/macOS/Linux 一致。

import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

/**
 * 构造配置 git hooksPath 的命令（纯函数 · 可单测）。
 * 不含任何 shell 重定向 / 操作符 —— 跨平台关键。
 * @returns {{ command: string, args: string[] }}
 */
export function buildHooksConfigCommand() {
  return {
    command: "git",
    args: ["config", "core.hooksPath", ".githooks"],
  };
}

/**
 * 设置 git core.hooksPath = .githooks。失败静默吞掉（不阻断 install · 对齐原 || true）。
 * @param {(command: string, args: string[]) => void} [runner] 可注入的执行器（默认 execFileSync · stdio 静默）。测试时注入 mock。
 * @returns {boolean} 成功设置返回 true · 失败（被吞）返回 false
 */
export function setupGitHooks(runner) {
  const exec =
    runner ??
    ((command, args) => {
      // stdio: "ignore" 静默 git 的 stdout/stderr（无需 shell 重定向）
      execFileSync(command, args, { stdio: "ignore" });
    });
  const { command, args } = buildHooksConfigCommand();
  try {
    exec(command, args);
    return true;
  } catch {
    // 容错：git 缺失 / 非 git 仓库 / config 失败 → 不阻断 pnpm install
    return false;
  }
}

// 作为脚本直接运行时（prepare 调用）执行配置；被 import（测试）时不自动跑。
const isMain =
  import.meta.url === pathToFileURL(process.argv[1] ?? "").href;
if (isMain) {
  setupGitHooks();
}
