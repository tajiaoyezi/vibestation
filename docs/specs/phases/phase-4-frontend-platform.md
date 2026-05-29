# Phase 4: frontend-platform

**Status**: Ready

> Phase Spec 按 S2V §8.2 八项结构。本 phase 为 Windows 适配纯前端切片，与 Phase 3（纯后端）文件域不交叠，可并行。

---

## 1. 阶段目标

让 Vibestation 前端（SolidJS / TypeScript）成为平台感知的 UI：

1. `web/src/index.tsx` 平台检测补 Windows 分支——发 `platform-windows` CSS class 到 `document.documentElement.classList`，并设 `data-platform` 属性供 CSS `content` 用。
2. 所有快捷键提示按平台显示：Windows 显示 `Ctrl+B`，macOS 显示 `⌘B`——消除"看到 `⌘B` 却要按 `Ctrl+B`"的误导。

**硬约束**：键盘**事件**处理（`isMac ? e.metaKey : e.ctrlKey`）survey 已证正确（见 `already_windows_ok`），本 phase **只改显示文案 / CSS class**，不动任何键盘事件分支逻辑，mac/Linux 显示行为零回归。

---

## 2. 业务价值

- **Windows 用户不再被误导**：快捷键提示（tooltip / placeholder / context menu / maximized chip）显示与实际按键一致的 `Ctrl+...`，降低首次使用摩擦（对应 PRD §Core Capabilities #4「Windows UI 与外设集成」+ §Users & Context 场景 1 多 agent 并行）。
- **解锁 Windows 专属样式能力**：`platform-windows` class + `data-platform` 属性让未来任何 Windows-only CSS 规则可被 target，是 PRD §User Flow happy path 步 4「终态对等 mac/Linux」的 UI 基石。
- **零 Rust 依赖、可独立交付**：纯前端 TS，不阻塞、不被阻塞于后端 cfg 分离，可与 Phase 3 并行推进，缩短 Windows 适配关键路径。

---

## 3. 涉及模块

| 模块 | 文件 | 角色 |
|---|---|---|
| web-platform | `web/src/index.tsx` | 平台检测 + 发 class + 设 `data-platform` |
| web-shortcuts | `web/src/lib/format-shortcut.ts`（新增） | 平台感知快捷键显示助手 |
| web-shortcuts | `web/src/components/TopBar.tsx` | `⌘B` → 平台感知 |
| web-shortcuts | `web/src/components/ActivityStrip.tsx` | `⌘2` / `⌘J` → 平台感知 |
| web-shortcuts | `web/src/panels/Terminal/PaneTerminal.tsx` | `⌘⇧O` / `⌘⇧D` / `⌘⌃W` / `⌘\` / `⌘⇧\` context menu + 按钮 title |
| web-shortcuts | `web/src/panels/CommitBar/CommitBar.tsx` | `⌘↵` placeholder |
| web-shortcuts | `web/src/App.tsx` | `⌘,` settings title |
| web-shortcuts | `web/src/dialogs/ConfigImport/ConfigImportDialog.tsx` | `⌘T / ⌘W / ⌘D / ⌘⇧D / ⌘,` help text |
| web-shortcuts | `web/src/panels/Terminal/styles.css` | `:1195` maximized chip `⌘Enter`（用 `data-platform` CSS content） |

---

## 4. 任务清单

| Task | 模块 | Spec | 依赖 | 说明 |
|---|---|---|---|---|
| 4.1 | web-platform | [`../tasks/task-4.1-platform-windows-class.md`](../tasks/task-4.1-platform-windows-class.md) | 纯前端 · 无 Rust 依赖 | 发 `platform-windows` class + `data-platform` 属性 |
| 4.2 | web-shortcuts | [`../tasks/task-4.2-shortcut-display.md`](../tasks/task-4.2-shortcut-display.md) | 依赖 4.1（`data-platform` 属性 + 平台判定一致性） | 平台感知快捷键显示助手 + 替换所有硬编码 `⌘` 文案 |

---

## 5. 依赖关系

- **Phase 级**：本 phase `depends_on: -`（纯前端，无 Rust 依赖）。PRD §Implementation Phases 标"可与 Phase 3 并行（纯前端 TS，独立于 Rust）"。
- **Task 级**：4.1 先行（建立 `data-platform` 属性 + 平台检测 single source）→ 4.2 依赖 4.1（styles.css 的 `data-platform` CSS content 依赖 4.1 设置属性；format-shortcut 助手与 index.tsx 平台判定保持一致）。
- **下游**：Phase 6 integration-matrix 的 vitest 全绿依赖本 phase 的 helper 单测；本 phase 完成后 Windows UI 文案对等 mac/Linux。

---

## 6. 阶段级验收标准 + 端到端 smoke

**阶段级验收标准**：

- [ ] Task 4.1 + 4.2 全部 `status: Done`，各自 §6 AC 逐条 ✅。
- [ ] `web/src/lib/format-shortcut.ts` helper 存在且被 vitest 覆盖（mac 返回 `⌘`-系符号、非 mac 返回 `Ctrl+`-系文本两路径均测）。
- [ ] 全量 grep `web/src` 无残留硬编码 `⌘` 在 user-facing 文案（title / placeholder / aria-label / context menu shortcut / CSS content）——除 helper 内部 mac 分支与代码注释外。
- [ ] mac/Linux 显示行为零回归：现有 vitest 全绿，`⌘` 在 mac 路径仍正确显示。

**端到端 smoke**（可执行）：

- **前端单测 smoke**：`pnpm lint`（Prettier all matched files pass）+ `pnpm typecheck`（tsc --noEmit 0 errors）+ `pnpm --filter @vibestation/web exec vitest run`（format-shortcut + 受影响组件测试全绿，0 failed）。
- **Windows GUI runtime smoke**（开发者本机 H:\ Windows 11 · 非 CI · §2.14 critical UX path）：`pnpm tauri:dev` 起窗口 → DevTools 确认 `<html>` 含 `platform-windows` class + `data-platform="windows"` 属性 → hover TopBar sidebar toggle 按钮 tooltip 显示 `Ctrl+B`（非 `⌘B`）→ 右键 Pane context menu 显示 `Ctrl+Shift+O` 等 → CommitBar placeholder 显示 `Ctrl+↵` → maximize Pane 浮动 chip 显示 `Ctrl+Enter or Esc`。
- **mac 对照 smoke**（reviewer / CI mac 侧）：同窗口 mac 上 tooltip 仍显示 `⌘B`、chip 仍显示 `⌘Enter`，证明平台分支生效且 mac 零回归。

---

## 7. 阶段级风险

| 风险 | 关联 PRD | 缓解 |
|---|---|---|
| 平台检测在 Windows 上 `navigator.platform` 大小写 / `userAgentData` 差异导致漏判 `platform-windows` | PRD §Technical Risks R4（路径 / 编码）旁系 | 检测同时看 `navigator.platform.toLowerCase().includes("win")` 与 `navigator.userAgentData?.platform`；4.1 单测覆盖 `Win32` / `Windows` / 空 platform 多形态 |
| 改显示时误改键盘**事件**分支，破坏已正确的 `metaKey/ctrlKey` 处理 | PRD §Technical Risks R3（mac/Linux 回归）+ §反指标 | 硬约束：本 phase 只改显示文案 / CSS / class，不动 `usePaneShortcuts.ts` / `mvp17-keyboard.ts` / `pane-keyboard.ts` 等事件 helper；vitest 锁 mac 路径文案不变 |
| 遗漏某处硬编码 `⌘`（survey 列 9 处，可能有未列处） | PRD §Core Capabilities #4 | 4.2 §3 In Scope 列全 9 处 + 完工前全量 grep `⌘` 兜底，发现新点纳入或记 §10 剩余风险 |

---

## 8. 阶段级 Definition of Done

- [ ] Task 4.1 / 4.2 均 `status: Done`，§10 Completion Notes 回填完整。
- [ ] `format-shortcut.ts` helper 落地，mac / 非 mac 两路径 vitest 覆盖且全绿。
- [ ] survey 列出的 9 处硬编码 `⌘` 文案全部平台感知化（或显式记录跳过原因）。
- [ ] `pnpm lint` + `pnpm typecheck` + `pnpm --filter @vibestation/web exec vitest run` 三命令全绿，raw output 留痕。
- [ ] mac/Linux 显示行为零回归（反指标硬约束）；键盘事件 helper 一行未改。
- [ ] Windows 本机 §2.14 GUI smoke 跑过并记录证据（class / tooltip / context menu / chip）。
- [ ] §6 端到端 smoke 全部执行并留结果。
