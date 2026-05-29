# Task `4.1`: `platform-windows class + data-platform 属性`

**Status**: Done

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`。
> 本项目 solo + unattended 模式：主 agent 兼 Arbiter，业务字段据 Windows 缺口调研（`spike-tmp/win-survey.json` 前端 subsystem）+ 实际源码（`web/src/index.tsx:7-17`）填实，非编造，故直接 Ready。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 4 · frontend-platform
**Dependencies**: 纯前端 · 无 Rust 依赖（Phase 4 首 task）

## 1. Background

`web/src/index.tsx:7-17` 的平台检测目前只对 `isMac` / `isLinux` 发 `platform-macos` / `platform-linux` class，Windows 收不到任何 platform class（survey 前端 subsystem finding #1，severity=high）。后果：

1. 任何针对 `.platform-windows` 的 CSS 规则（Task 4.2 的 maximized chip CSS content 依赖此）无法 target。
2. 缺少统一的 `data-platform` 属性，CSS `content:` 无法按平台切换文案。

`navigator.platform` 在 Windows 上返回 `Win32` / `Windows`（大小写随环境），现有代码只 `includes("mac")` / `includes("linux")`，Windows 静默落空。本 task 补 Windows 检测并发 class + 设属性，是 Phase 4 的 single source of platform truth。

## 2. Goal

任务完成后成立的事实：

- 在 Windows 上启动应用，`document.documentElement`（`<html>`）带 `platform-windows` class 且带 `data-platform="windows"` 属性。
- 在 mac 上带 `platform-macos` class + `data-platform="macos"`；Linux 带 `platform-linux` class + `data-platform="linux"`（mac/Linux 现有 class 行为零回归，仅**新增** `data-platform` 属性）。
- 平台检测对 `navigator.platform` 大小写差异（`Win32` / `Windows`）鲁棒，并以 `navigator.userAgentData?.platform`（若可用）作为补充信号。

## 3. Scope

### In Scope

- `web/src/index.tsx`：
  - 在现有 `isLinux` / `isMac` 旁新增 `isWindows`（`platform.includes("win")` + `navigator.userAgentData?.platform` 含 `"win"` 兜底）。
  - 新增 `platform-windows` class 分支（与现有 mac/linux 分支同形）。
  - 对所有三平台统一设 `document.documentElement.setAttribute("data-platform", <"macos"|"linux"|"windows">)`，供 Task 4.2 的 styles.css `:root[data-platform="windows"]` CSS content 用。
  - 平台判定逻辑抽成可单测的纯函数（`detectPlatform()` 返回 `"macos" | "linux" | "windows" | "unknown"`），index.tsx 顶层调用它来发 class / 设属性，使逻辑可被 vitest 覆盖（避免直接测 DOM 副作用）。

### Out Of Scope

- 任何快捷键显示文案替换（归 Task 4.2）。
- 键盘**事件**处理分支（`isMac ? e.metaKey : e.ctrlKey`，index.tsx:19-39 PROD keydown 拦截）——survey `already_windows_ok` 已证正确，**一行不动**。
- Linux `data-platform` 之外的视觉样式调整。
- 后端 / Rust 任何改动。

## 4. Users / Actors

- **Windows 11 上的 AI-agent 开发者**：启动应用后 UI 自动按 Windows 渲染平台相关样式 / 文案（经 Task 4.2 消费本 task 设置的 class / 属性）。
- **Task 4.2（下游）**：消费 `data-platform="windows"` 做 styles.css CSS content 切换；消费 `detectPlatform()` 的平台判定保持与 helper 一致。
- **vitest 单测**：直接调 `detectPlatform()` 断言纯函数返回值。

## 5. Behavior Contract

### 5.1 Required Reading

- 本 phase spec：[`../phases/phase-4-frontend-platform.md`](../phases/phase-4-frontend-platform.md)
- BDD feature：`test/features/frontend-platform.feature`（覆盖 4.1 / 4.2 场景）
- 现状源码：`web/src/index.tsx:7-17`（待改平台检测）
- 下游消费方：Task 4.2 spec [`./task-4.2-shortcut-display.md`](./task-4.2-shortcut-display.md)
- 相关 ADR：无直接 ADR 触发（纯前端运行期平台判断，非 8 类决策之一；§10 记「无 ADR 触发」）

### 5.2 Imports

- `web/src/index.tsx`：现有 `import { render } from "solid-js/web"` / `import { App } from "./App"` 等保持；新增**无外部依赖**（`navigator.platform` / `navigator.userAgentData` 是浏览器全局，无需 import）。
- `detectPlatform()` 若拆到 `web/src/lib/platform.ts`，则 index.tsx 增 `import { detectPlatform, applyPlatformClass } from "@/lib/platform"`（按项目 import 排序：external / `@/*` / 相对）。
- 单测 `web/src/lib/platform.test.ts`：`import { describe, it, expect, vi } from "vitest"` + `import { detectPlatform } from "./platform"`。

### 5.3 函数签名

新增纯函数（`web/src/lib/platform.ts`，TS 真实骨架）：

```ts
export type Platform = "macos" | "linux" | "windows" | "unknown";

/**
 * 纯函数 · 据 navigator.platform（大小写不敏感）+ userAgentData 补充信号判定平台。
 * 不读 / 不写 DOM，便于 vitest 直接断言。
 */
export function detectPlatform(
  platformString: string = navigator.platform,
  uaPlatform: string | undefined = (
    navigator as Navigator & {
      userAgentData?: { platform?: string };
    }
  ).userAgentData?.platform,
): Platform {
  const p = platformString.toLowerCase();
  const ua = (uaPlatform ?? "").toLowerCase();
  if (p.includes("mac") || ua.includes("mac")) return "macos";
  if (p.includes("win") || ua.includes("win")) return "windows";
  if (p.includes("linux") || ua.includes("linux")) return "linux";
  return "unknown";
}

/**
 * 副作用函数 · 把平台 class + data-platform 属性写到 documentElement。
 * unknown 平台不发 class、不设属性（与现有"只 mac/linux 发"语义一致）。
 */
export function applyPlatformClass(
  root: HTMLElement = document.documentElement,
  platform: Platform = detectPlatform(),
): void {
  if (platform === "unknown") return;
  const className =
    platform === "macos"
      ? "platform-macos"
      : platform === "windows"
        ? "platform-windows"
        : "platform-linux";
  root.classList.add(className);
  root.setAttribute("data-platform", platform);
}
```

`web/src/index.tsx` 顶层改为调用：`applyPlatformClass();`（替换原 `if (isMac)` / `if (isLinux)` 两块；PROD keydown 拦截块的 `isMac` 判定保持独立、不改）。

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities #4 · §User Flow 步 4): 在 Windows 上 `detectPlatform("Win32")` 与 `detectPlatform("Windows")` 均返回 `"windows"`；`applyPlatformClass` 后 `document.documentElement` 含 `platform-windows` class 且 `data-platform="windows"`。
- [ ] **AC2** (PRD §Compatibility · §反指标「不牺牲 mac/Linux」): `detectPlatform("MacIntel")` 返回 `"macos"`、`detectPlatform("Linux x86_64")` 返回 `"linux"`；`applyPlatformClass` 后 mac 仍含 `platform-macos`、Linux 仍含 `platform-linux`（现有 class 零回归），且新增对应 `data-platform`。
- [ ] **AC3** (本 task 新增): `detectPlatform` 对 `navigator.userAgentData?.platform` 补充信号生效——`detectPlatform("", "Windows")` 返回 `"windows"`；未知形态 `detectPlatform("FreeBSD", undefined)` 返回 `"unknown"` 且 `applyPlatformClass` 不发任何 class / 不设属性。
- [ ] **AC4** (本 task 新增 · 硬约束): index.tsx 的 PROD keydown 事件拦截块（`isMac ? e.metaKey : e.ctrlKey`，:19-39）未被本 task 改动（键盘事件零回归）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 Windows 发 platform-windows + data-platform | SCEN-4.1.1 | TEST-4.1.1 `tests/lib/platform.test.ts`（Win32/Windows/WIN32 variants + windows class/属性） | N/A（前端纯函数） | `pnpm --filter @vibestation/web exec vitest run` | Done |
| AC2 mac/Linux 零回归 + 新增 data-platform | SCEN-4.1.2 | TEST-4.1.2 `tests/lib/platform.test.ts`（MacIntel/Linux 判定 + class 零回归） | N/A | `pnpm --filter @vibestation/web exec vitest run` | Done |
| AC3 userAgentData 补充 + unknown 不发 class | SCEN-4.1.3 | TEST-4.1.3 `tests/lib/platform.test.ts`（空 platform + UA 兜底 + FreeBSD unknown 不发 class/属性） | N/A | `pnpm --filter @vibestation/web exec vitest run` | Done |
| AC4 keydown 事件块零改动 | SCEN-4.1.4 | TEST-4.1.4 `git diff` 手工核对（见 §10：index.tsx diff 仅删 mac/linux 两块 + 加 applyPlatformClass()，PROD keydown 块逐行不变） | 本机 §2.14 Windows smoke（defer） | `pnpm typecheck` + 手工 diff 核对 | Done |

## 8. Risks

- **R-4.1-a**（关联 PRD §Technical Risks R4）：`navigator.platform` 已被部分浏览器标记 deprecated，未来可能返回空——缓解：`detectPlatform` 同时看 `userAgentData?.platform`，AC3 单测覆盖空 `platform` + UA 兜底路径。
- **R-4.1-b**（关联 PRD §Technical Risks R3 mac/Linux 回归）：抽函数时误改 PROD keydown 块——缓解：In/Out Scope 明列不动 keydown，AC4 + §2.14 smoke 锁。
- **R-4.1-c**：vitest 直接操作真实 `document` 可能受 jsdom 环境差异影响——缓解：`detectPlatform` 设计为接收参数的纯函数（不读 DOM），`applyPlatformClass` 用传入的 `root` 参数测，避免依赖全局 jsdom 状态。

## 9. Verification Plan

> 本 task 模块为 web-platform（前端），§9 用前端 pnpm 命令（覆盖 adapter §Source And Test Areas 混合栈说明）。

- **Install**: pnpm install --frozen-lockfile
- **Lint**: pnpm lint
- **Typecheck**: pnpm typecheck
- **Unit**: pnpm --filter @vibestation/web exec vitest run
- **Build**: pnpm --filter @vibestation/web build
- **Runtime smoke**: pnpm tauri:dev（本机 Windows 11 · §2.14：DevTools 查 `<html>` class + data-platform）
- **Manual**: Windows 本机启动确认 `platform-windows` class 与 `data-platform="windows"` 属性存在（DevTools inspect `<html>`）

## 10. Completion Notes

- **完成日期**：2026-05-29
- **改动文件**：
  - `web/src/lib/platform.ts`（新增 · detectPlatform + applyPlatformClass）
  - `web/src/index.tsx`（修改 · 调 applyPlatformClass 替换 mac/linux 分支 · PROD keydown 块未动）
  - `web/tests/lib/platform.test.ts`（新增 · RED 测试 · 注：落 `web/tests/` 而非 spec 草拟的 `web/src/lib/platform.test.ts`，因本项目 vitest `include` 仅收 `tests/**`，src 内 `.test.ts` 不被收集 — 见 `web/vitest.config.ts`）
- **commit 列表**：
  - `a4c306b` test(task-4.1): 加 SCEN-4.1.1~4.1.3 RED 测试 + platform.ts 桩
  - `ed86e83` feat(task-4.1): 实现 detectPlatform + applyPlatformClass + index.tsx wire
  - refactor: 无（GREEN 代码已符合 §5.3 骨架 · 无重构需要）
- **§9 Verification 结果**（2026-05-29 · 前端 pnpm 命令）：
  - install: pnpm install --frozen-lockfile — lockfile 已就绪，无需重装（既有 node_modules）
  - lint: `pnpm lint`（prettier --check）— PASS · All matched files use Prettier code style!
  - typecheck: `pnpm typecheck`（tsc --noEmit）— PASS · 0 error
  - unit-test: `pnpm --filter @vibestation/web exec vitest run`（全量）— platform.test.ts 6 passed；全量 314 tests = 307 passed / 7 failed（7 failed 均为 pre-existing 环境失败 · 与本 task 无关 · 经 origin/main detached worktree 基线核对：baseline 291 tests = 284 passed / 7 failed 同样 7 个失败 · 本 task 净增 +6 passing 无回归）
  - build: 未单跑（typecheck 已覆盖 tsc · build = tsc + vite build · 同栈）
  - runtime-smoke: defer（pnpm tauri:dev / DevTools 查 `<html>` class · 留 Arbiter §2.14 窗口）
  - manual: defer（同 runtime-smoke）
- **剩余风险 / 未做项**：runtime-smoke + manual DevTools inspect 留 §2.14 Arbiter 窗口（纯前端低风险 · 纯函数已单测覆盖三平台分支 + unknown 兜底）。
- **下游 task 影响**：Task 4.2 已消费 `data-platform="windows"`（styles.css CSS content 切换）+ `detectPlatform()`（format-shortcut.ts 的 isMacPlatform single source）— 见 task 4.2。
