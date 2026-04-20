---
id: MVP-03
type: mvp
title: Tool Windows 布局（Primary/Secondary/Bottom + Activity Strip）
status: ready
owner:
phase: W3-W4
depends_on: ["MVP-01", "MVP-02"]
blocks: ["MVP-04", "MVP-05", "MVP-07"]
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5（UI 架构）
risk_ref:
reviewer: Claude Code (self-review · advisory gate · 单人项目 v2-D · tasks/README.md §原则 7)
---

# MVP-03: Tool Windows 布局

> **状态**：`ready`（2026-04-20 · MVP-02 done 解阻塞 · self-review 通过 · 可被认领）
> **依赖**：MVP-01 · MVP-02 / **阻塞**：MVP-04（Tab 显示区域）· MVP-05（Pane 嵌入主区）· MVP-07（Git Log → Secondary Sidebar）

---

## 🎯 目标（Goal）

实现 Vibestation 的 Tool Windows 布局（Primary Sidebar + 主区 + Secondary Sidebar + Bottom Panel + Activity Strip），与 `design/directions/1-calm-studio.html` 原型一致。

## 📖 背景（Context）

- `CLAUDE.md` 决策表 **#9**（A 栏锁定）：Tool Windows 默认状态 = Primary Sidebar 展开 · Secondary + Bottom 收起
- 原型 JS `DEFAULT_STATE = { primary: true, secondary: false, bottom: false }`
- 禁区：不得修改 `design/directions/1-calm-studio.html` 的布局结构 / 色彩 token 语义 / 字体（仅允许 token 数值微调、bug 修复、a11y）

---

## 🎨 功能范围（Scope）

**Do**：
- 5 区块布局：Primary Sidebar（左）· 主内容区（中）· Secondary Sidebar（右）· Bottom Panel（下）· Activity Strip（最右细条）
- 默认状态：Primary 展开 · Right Activity Strip 细条可见 · Secondary + Bottom 收起
- Primary Sidebar 包含：workspace switcher（顶部）+ 分支树（中部占位，MVP-07 填充）
- Secondary Sidebar 占位（Git Log 由 MVP-07 接管）
- Bottom Panel 占位（Git Status 由 MVP-08 接管）
- 每个 Tool Window 独立 toggle（快捷键 + 图标）
- 尺寸调整：拖拽边界 resize，比例持久化到 rusqlite
- 主题切换：跟随 OS / 手动 light / dark / auto（继承原型 CSS 变量）

**Don't**：
- 主区内容（由 MVP-04 Tab + MVP-05 Pane 填充）
- Git Log / Status 内容（由 MVP-07 / MVP-08）
- 命令面板（→ 不在 MVP 范围，v0.2+）

## 🖼 UI 引用

- **原型权威**：`design/directions/1-calm-studio.html`（整页布局、DEFAULT_STATE、toggle 逻辑）
- 色彩 token：从原型 CSS 变量直接继承（`--surface-*` / `--text-*` / `--accent-*`）
- 字体：原型定义的 `Inter` + `JetBrains Mono`（Inter 正文 / JetBrains Mono 终端）

## ✅ Acceptance

### A. 布局结构

- [ ] 5 区块 DOM 结构与原型一致（不同实现可用 SolidJS 组件，但语义等价）
- [ ] CSS Grid 布局，主区 flexible，sidebars 固定宽度（默认 240px/320px，可 resize）
- [ ] Primary Sidebar 最小 200px，最大 400px；Secondary 最小 240px，最大 500px；Bottom 最小 150px，最大 500px
- [ ] Activity Strip 细条固定 40px 宽

### B. 默认状态（锁定 #9）

- [ ] 首次打开 workspace：Primary 展开 · Right Activity Strip 细条可见 · Secondary 收起 · Bottom 收起
- [ ] 状态持久化到 rusqlite per-workspace（同 workspace 下次打开保持上次状态）

### C. Toggle 控制

- [ ] Primary toggle：`⌘1`（mac）/ `Ctrl+1`（linux）
- [ ] Secondary toggle：`⌘2` / `Ctrl+2`
- [ ] Bottom toggle：`⌘J` / `Ctrl+J`
- [ ] Activity Strip 图标点击也能 toggle（与快捷键等价）

### D. Resize

- [ ] 拖拽 sidebar 和 bottom panel 的边界 resize
- [ ] 拖拽时实时更新 + debounce 持久化（250ms）
- [ ] 双击边界 → 复位到默认宽度

### E. 主题

- [ ] 设置项：light / dark / auto（跟随 OS）
- [ ] 切换主题 CSS 变量立即生效，无闪烁
- [ ] 主题选择持久化到 rusqlite（应用级别，不是 per-workspace）

### F. a11y

- [ ] 所有 Tool Window 有 `role` 和 `aria-label`
- [ ] 键盘 Tab 遍历顺序合理：Primary → 主区 → Secondary → Bottom
- [ ] Focus ring 在深色 / 浅色主题下都可见

## 🧪 测试策略

| 层次 | 覆盖 |
|------|------|
| 单元 | Layout 状态机（toggle / resize 的 reducer）|
| 视觉回归 | Playwright screenshot diff，对比原型 |
| a11y | axe-core 扫描零 critical |
| E2E | toggle + resize + 主题切换 |
| 手动 QA | 三平台视觉一致 |

## 💾 数据模型变更

扩展 `workspaces` table 添加 `layout_state`：
```rust
struct LayoutState {
    primary_open: bool,
    secondary_open: bool,
    bottom_open: bool,
    primary_width: u32,
    secondary_width: u32,
    bottom_height: u32,
}
```

新增 application-level table `app_settings`：
- key: `"theme"` → value: `"light" | "dark" | "auto"`

## ⚠️ 已知风险

- **原型和实现偏差**：禁止改原型布局，但 MVP-03 是"实装"——若发现原型不合理需通过 ADR 流程改
- **Wayland resize 渲染**：某些 Linux 合成器下 resize 事件频率低 → 需 debounce

## 📝 Notes

- Primary Sidebar 顶部 workspace switcher 复用 MVP-02 的数据源
- MVP-03 不做"save layout as preset"（Smart Layouts 在 MVP-05 Pane 范围内）

## 🔗 相关

- `CLAUDE.md` 决策表 **#9**（A 栏）
- 原型：`design/directions/1-calm-studio.html`
- 上游：MVP-01 · MVP-02
- 下游：MVP-04 · MVP-05 · MVP-07 · MVP-08

---

**自审四问**：1. Acceptance 覆盖结构/状态/交互/a11y ✅ · 2. 原型偏差走 ADR ✅ · 3. 三平台显式测试 ✅ · 4. 不做命令面板等 v0.2+ 功能 ✅
