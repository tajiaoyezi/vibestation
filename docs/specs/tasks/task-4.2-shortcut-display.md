# Task `4.2`: `平台感知快捷键显示助手 + 替换硬编码 ⌘ 文案`

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`。
> 本项目 solo + unattended 模式：主 agent 兼 Arbiter，业务字段据 Windows 缺口调研（`spike-tmp/win-survey.json` 前端 subsystem · 9 处硬编码 ⌘ 精确行号）+ 实际源码填实，非编造，故直接 Ready。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 4 · frontend-platform
**Dependencies**: 依赖 Task 4.1（`data-platform` 属性 + 平台判定 single source）

## 1. Background

survey 前端 subsystem 列出 9 处 user-facing 快捷键提示**硬编码 macOS 符号**（`⌘` / `⇧` / `⌥` / `⌃`），Windows 用户看到 `⌘B` 却要按 `Ctrl+B`，造成认知摩擦：

| 文件 | 行 | 硬编码 | severity |
|---|---|---|---|
| `web/src/components/TopBar.tsx` | :24 | `Toggle Primary Sidebar (⌘B)` | high |
| `web/src/components/ActivityStrip.tsx` | :16-17 | `⌘2` / `⌘J` | high |
| `web/src/panels/Terminal/PaneTerminal.tsx` | :695/701/717 | `⌘⇧O` / `⌘⇧D` / `⌘⌃W`（context menu） | high |
| `web/src/panels/Terminal/PaneTerminal.tsx` | :761/791/821 | `⌘\` / `⌘⇧\` / `⌘⌃W`（按钮 title） | high |
| `web/src/panels/CommitBar/CommitBar.tsx` | :192 | `(⌘↵ 提交)` placeholder | medium |
| `web/src/App.tsx` | :996 | `Settings (⌘,)` | medium |
| `web/src/dialogs/ConfigImport/ConfigImportDialog.tsx` | :369-370 | `⌘T / ⌘W / ⌘D / ⌘⇧D / ⌘,` | low |
| `web/src/panels/Terminal/styles.css` | :1195 | `content: "maximized · ⌘Enter or Esc"` | medium |

survey `already_windows_ok` 已确认所有键盘**事件** helper（`usePaneShortcuts.ts` / `mvp17-keyboard.ts` / `pane-keyboard.ts` / `hooks.ts` 等）正确用 `isMac ? metaKey : ctrlKey`——**问题纯在显示文案**，不在事件处理。本 task 新增统一的平台感知显示助手并替换全部 9 处。

## 2. Goal

任务完成后成立的事实：

- 新增 `web/src/lib/format-shortcut.ts`，导出平台感知显示助手：Windows 返回 `Ctrl+B` 系文本、macOS 返回 `⌘B` 系符号。
- survey 列出的 8 个 TS/TSX 处文案全部改为调 helper（按平台动态显示）；`styles.css:1195` 改用 `:root[data-platform="windows"]` CSS content 切换（消费 Task 4.1 设置的 `data-platform` 属性）。
- mac 显示行为零回归（mac 仍显示 `⌘B` / `⌘Enter`）；键盘事件 helper 一行未改。

## 3. Scope

### In Scope

- **新增** `web/src/lib/format-shortcut.ts`：平台感知显示助手（见 §5.3）。
- **替换** 以下 8 个 TS/TSX 文件的硬编码 `⌘` 文案为 helper 调用：
  - `web/src/components/TopBar.tsx`（:24 sidebar toggle title）
  - `web/src/components/ActivityStrip.tsx`（:16-17 `⌘2` / `⌘J`）
  - `web/src/panels/Terminal/PaneTerminal.tsx`（:695/701/717 context menu shortcut + :761/791/821 按钮 title）
  - `web/src/panels/CommitBar/CommitBar.tsx`（:192 placeholder）
  - `web/src/App.tsx`（:996 settings title）
  - `web/src/dialogs/ConfigImport/ConfigImportDialog.tsx`（:369-370 help text）
- **改** `web/src/panels/Terminal/styles.css`（:1195）：用 `:root[data-platform="windows"] .<chip-selector>::before { content: "maximized · Ctrl+Enter or Esc"; }` 平台条件化，保留 mac/默认 `⌘Enter`。
- **新增** `web/src/lib/format-shortcut.test.ts`：helper 单测（mac / 非 mac 两路径）。

### Out Of Scope

- 平台检测 / 发 class / 设 `data-platform` 属性（归 Task 4.1，本 task 消费其产物）。
- 任何键盘**事件**处理 helper（`usePaneShortcuts.ts` / `mvp17-keyboard.ts` / `pane-keyboard.ts` / `usePaneNavigation.ts` / `hooks.ts`）——`already_windows_ok` 已证正确，**一行不动**。
- `pane-keyboard.ts:141-143` 代码注释措辞优化（survey 标 low · 非 user-facing · 可选，默认不在本 task 范围，记 §10 剩余项）。
- 后端 / Rust 任何改动。

## 4. Users / Actors

- **Windows 11 上的 AI-agent 开发者**：hover 按钮 / 看 placeholder / 右键 context menu 时，快捷键提示显示 `Ctrl+...` 与实际按键一致。
- **macOS 用户**：所有提示仍显示 `⌘` 系符号（零回归）。
- **vitest 单测**：调 `formatShortcut(...)` 断言 mac / 非 mac 两路径输出。

## 5. Behavior Contract

### 5.1 Required Reading

- 本 phase spec：[`../phases/phase-4-frontend-platform.md`](../phases/phase-4-frontend-platform.md)
- 上游 task：[`./task-4.1-platform-windows-class.md`](./task-4.1-platform-windows-class.md)（提供 `data-platform` 属性 + `detectPlatform()`）
- BDD feature：`test/features/frontend-platform.feature`
- 现状源码：survey §1 表列的 9 处文件 + 行号
- 参考（不改 · 仅对照事件 helper 正确性）：`web/src/panels/Terminal/usePaneShortcuts.ts`、`web/src/lib/mvp17-keyboard.ts`、`web/src/lib/pane-keyboard.ts`
- 相关 ADR：无直接 ADR 触发（纯显示层 · 非 8 类决策；§10 记「无 ADR 触发」）

### 5.2 Imports

- `web/src/lib/format-shortcut.ts`：复用 Task 4.1 的平台判定——`import { detectPlatform } from "@/lib/platform"`（避免重复实现平台检测，保持 single source）。无其他外部依赖。
- 各被改组件：新增 `import { formatShortcut } from "@/lib/format-shortcut"`（按 import 排序 external / `@/*` / 相对）。
- 单测 `web/src/lib/format-shortcut.test.ts`：`import { describe, it, expect, vi } from "vitest"` + `import { formatShortcut, isMacPlatform } from "./format-shortcut"`。

### 5.3 函数签名

新增（`web/src/lib/format-shortcut.ts`，TS 真实骨架）：

```ts
import { detectPlatform } from "@/lib/platform";

/** 当前是否 mac（消费 Task 4.1 的 detectPlatform，保持 single source）。 */
export function isMacPlatform(): boolean {
  return detectPlatform() === "macos";
}

/**
 * 平台感知快捷键显示。
 * @param mac   macOS 符号文案，如 "⌘B" / "⌘⇧O" / "⌘↵"
 * @param other 非 mac（Windows/Linux）文案，如 "Ctrl+B" / "Ctrl+Shift+O" / "Ctrl+Enter"
 * @returns isMacPlatform() ? mac : other
 *
 * 例：formatShortcut("⌘B", "Ctrl+B")
 *     formatShortcut("⌘⇧O", "Ctrl+Shift+O")
 *     formatShortcut("⌘↵", "Ctrl+Enter")
 */
export function formatShortcut(mac: string, other: string): string {
  return isMacPlatform() ? mac : other;
}
```

各处用法（示例 · 实施按真实上下文嵌入）：

```tsx
// TopBar.tsx :24
title={`Toggle Primary Sidebar (${formatShortcut("⌘B", "Ctrl+B")})`}

// ActivityStrip.tsx :16-17
{ id: "secondary", icon: "⊟", label: "Git Log",    shortcut: formatShortcut("⌘2", "Ctrl+2") },
{ id: "bottom",    icon: "◴", label: "Git Status", shortcut: formatShortcut("⌘J", "Ctrl+J") },

// CommitBar.tsx :192
placeholder={`Commit message… (${formatShortcut("⌘↵", "Ctrl+↵")} 提交)`}
```

`styles.css:1195`（CSS content 平台条件化，消费 Task 4.1 的 `data-platform`）：

```css
/* 默认 / mac：保留 ⌘Enter */
.vs-pane-maximized-chip::before { content: "maximized · ⌘Enter or Esc"; }
/* Windows：用 data-platform 切换（Task 4.1 在 <html> 设 data-platform="windows"） */
:root[data-platform="windows"] .vs-pane-maximized-chip::before {
  content: "maximized · Ctrl+Enter or Esc";
}
```

> 注：实际 chip selector 名以 `styles.css:1195` 现有规则为准，实施时不改 selector，仅加 `:root[data-platform="windows"]` 覆盖规则。

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities #4 「显示 Ctrl+B 而非 ⌘B」): `formatShortcut("⌘B", "Ctrl+B")` 在非 mac（Windows/Linux）平台返回 `"Ctrl+B"`、在 mac 返回 `"⌘B"`；`isMacPlatform()` 在 Windows 返回 `false`、mac 返回 `true`。
- [ ] **AC2** (PRD §Core Capabilities #4 · §Users 场景 1 右键菜单): survey 列出的 8 个 TS/TSX 处硬编码 `⌘` 文案全部改为 `formatShortcut(...)` 调用——全量 grep `web/src/**/*.{ts,tsx}` 在 user-facing 文案（title / placeholder / aria-label / context menu shortcut）无残留裸 `⌘`（helper 内部 mac 参数除外）。
- [ ] **AC3** (PRD §Core Capabilities #4 · §User Flow maximized chip): `styles.css:1195` maximized chip 在 `data-platform="windows"` 下经 CSS content 显示 `"maximized · Ctrl+Enter or Esc"`、默认/mac 显示 `"maximized · ⌘Enter or Esc"`。
- [ ] **AC4** (PRD §Compatibility · §反指标「不牺牲 mac/Linux」): mac 路径所有提示文案与改动前逐字一致（`⌘B` / `⌘2` / `⌘J` / `⌘⇧O` / `⌘↵` / `⌘,` 等不变）；现有 vitest 全绿。
- [ ] **AC5** (本 task 新增 · 硬约束): 键盘事件 helper（`usePaneShortcuts.ts` / `mvp17-keyboard.ts` / `pane-keyboard.ts` / `usePaneNavigation.ts` / `hooks.ts`）一行未改——`git diff` 这些文件为空。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 formatShortcut mac/非 mac 双路径 | SCEN-4.2.1 | TEST-4.2.1 `test_format_shortcut_per_platform` | N/A | `pnpm --filter @vibestation/web exec vitest run` | Not Started |
| AC2 8 处文案替换无残留裸 ⌘ | SCEN-4.2.2 | TEST-4.2.2 `test_no_hardcoded_cmd_symbol`（grep 断言测试） | 本机 §2.14 hover/右键 smoke | `pnpm --filter @vibestation/web exec vitest run` + grep | Not Started |
| AC3 maximized chip CSS content 平台切换 | SCEN-4.2.3 | TEST-4.2.3 `test_maximized_chip_css_content`（解析 styles.css 断言含 windows 规则） | 本机 §2.14 maximize Pane smoke | `pnpm --filter @vibestation/web exec vitest run` | Not Started |
| AC4 mac 文案逐字零回归 | SCEN-4.2.4 | TEST-4.2.4 `test_mac_labels_unchanged` | mac 对照 smoke | `pnpm --filter @vibestation/web exec vitest run` | Not Started |
| AC5 事件 helper 零改动 | SCEN-4.2.5 | TEST-4.2.5 `test_event_helpers_untouched`（git diff 断言 / 快照） | 本机 §2.14 实按键验证 | `pnpm typecheck` + 手工 diff 核对 | Not Started |

## 8. Risks

- **R-4.2-a**（关联 PRD §Technical Risks R3 mac/Linux 回归）：替换文案时误改 mac 符号或漏改某处——缓解：AC4 逐字对照 + AC2 全量 grep 兜底；diff 限定在 9 处文件。
- **R-4.2-b**（关联 PRD §Technical Risks R3 · §反指标）：误把事件 helper 当显示文案改坏键盘处理——缓解：AC5 硬约束 + Out Of Scope 明列 5 个事件 helper 不动；§2.14 本机实按键验证。
- **R-4.2-c**：`styles.css` CSS content 单测难直接渲染——缓解：TEST-4.2.3 用读文件解析断言含 `:root[data-platform="windows"]` 规则 + Windows 本机 §2.14 maximize Pane 目视确认 chip 文案。
- **R-4.2-d**：survey 列 9 处可能未穷尽——缓解：AC2 全量 grep `⌘` 作为兜底，发现新点纳入或记 §10 剩余风险。

## 9. Verification Plan

> 本 task 模块为 web-shortcuts（前端），§9 用前端 pnpm 命令。

- **Install**: pnpm install --frozen-lockfile
- **Lint**: pnpm lint
- **Typecheck**: pnpm typecheck
- **Unit**: pnpm --filter @vibestation/web exec vitest run
- **Build**: pnpm --filter @vibestation/web build
- **Runtime smoke**: pnpm tauri:dev（本机 Windows 11 · §2.14：hover TopBar tooltip=Ctrl+B / 右键 context menu=Ctrl+Shift+O / CommitBar placeholder=Ctrl+↵ / maximize Pane chip=Ctrl+Enter）
- **Manual**: Windows 本机目视确认 8 处文案 + chip 显示 `Ctrl+...`；mac 对照确认仍显示 `⌘`

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：
  - `web/src/lib/format-shortcut.ts`（新增 · formatShortcut + isMacPlatform）
  - `web/src/components/TopBar.tsx`（修改）
  - `web/src/components/ActivityStrip.tsx`（修改）
  - `web/src/panels/Terminal/PaneTerminal.tsx`（修改）
  - `web/src/panels/CommitBar/CommitBar.tsx`（修改）
  - `web/src/App.tsx`（修改）
  - `web/src/dialogs/ConfigImport/ConfigImportDialog.tsx`（修改）
  - `web/src/panels/Terminal/styles.css`（修改 · data-platform CSS content）
  - `web/src/lib/format-shortcut.test.ts`（新增 · RED 测试）
- **commit 列表**：
  - `<TBD-after-impl>` test: 加 SCEN-4.2.1~4.2.5 RED 测试
  - `<TBD-after-impl>` feat: 实现 formatShortcut + 替换 8 处文案 + styles.css 平台条件化
  - `<TBD-after-impl>` refactor:（如有）
- **§9 Verification 结果**：
  - install: <TBD-after-impl>
  - lint: <TBD-after-impl>
  - typecheck: <TBD-after-impl>
  - unit-test: <TBD-after-impl>
  - build: <TBD-after-impl>
  - runtime-smoke: <TBD-after-impl>
  - manual: <TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>（含 `pane-keyboard.ts:141-143` 注释措辞是否一并清晰化）
- **下游 task 影响**：<TBD-after-impl>（Phase 6 vitest 矩阵覆盖本 helper）
