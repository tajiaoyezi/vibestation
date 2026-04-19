---
id: MVP-05
type: mvp
title: Pane 分屏（单层 · 最多 4 Pane · Smart Layouts）
status: draft
owner:
phase: W6-W7
depends_on: ["MVP-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5.3（Pane 系统）
risk_ref:
reviewer:
---

# MVP-05: Pane 分屏（单层）

> **状态**：`draft`
> **依赖**：MVP-04（Tab 终端存在才能分屏）
> **战略依据**：`§10.1` MVP B 折中方案 · `§5.3` Pane 系统

---

## 🎯 目标（Goal）

每个 Tab 内支持**单层**嵌套分屏（最多 4 Pane：1 次右分 + 1 次下分），提供 Smart Layouts 预设（Solo / AI+Runner），拖拽分隔条调整比例。

## 📖 背景（Context）

- `implementation-plan.md §10.1` MVP B 折中方案明确"Pane 最多 1 层嵌套（4 Pane）"
- 任意嵌套 / Dual AI / Triple Review / Quad 预设都砍到 v0.2+
- `CLAUDE.md` 锁定 #2（A 栏 MVP 范围）

---

## 🎨 功能范围（Scope）

**Do**：
- Pane 分屏快捷键：
  - `⌘\` 右分屏
  - `⌘⇧\` 下分屏
  - `⌘⌃W` 关当前 Pane
- 单层嵌套上限：右分一次 + 下分一次 = 最多 4 Pane（矩形 2×2 或 1×2 或 2×1）
- Smart Layouts 预设（一键切换）：
  - **Solo**：单 Pane 全占
  - **AI + Runner**：右分屏 50/50（左 AI CLI · 右 runner/shell）
- 分隔条拖拽调整比例，双击复位到默认比例（50/50）
- 比例持久化到 rusqlite per-Tab
- 每个 Pane 独立 PTY（复用 MVP-04 PTY 架构）
- 当前聚焦 Pane 高亮边框

**Don't**：
- 任意嵌套（v0.2+）
- 方向键跳邻居 Pane / ⌘Enter 最大化（v0.2+）
- Dual AI / Triple Review / Quad 预设（v0.2+）
- Pane Detach（v0.3+）

## 🖼 UI 引用

- `design/directions/1-calm-studio.html` 主区 Pane 分屏 demo（右分屏 + 下分屏 示例）
- 分隔条：1px 浅色，hover 时变主色 + 光标变 resize
- 当前 Pane 边框：主色 1px

## ✅ Acceptance

### A. 分屏操作

- [ ] `⌘\` 从当前 Pane 右分出一个新 Pane，新 Pane 默认运行同 shell
- [ ] `⌘⇧\` 从当前 Pane 下分出一个新 Pane
- [ ] `⌘⌃W` 关当前 Pane（若仅剩 1 个 Pane → 关整个 Tab）
- [ ] 已在右分状态再点 `⌘\` → 被拒绝（超单层上限），显示"Pane 已达单层上限"toast

### B. 单层嵌套规则

- [ ] 合法布局（✅）：Solo / 水平 2 Pane / 垂直 2 Pane / 2×2 (右分再下分)
- [ ] 非法布局（❌）：右分的右 Pane 再右分（= 3 横）、下分的下 Pane 再下分（= 3 竖）
- [ ] 布局上限逻辑用单元测试覆盖所有组合

### C. Smart Layouts

- [ ] 命令面板（临时用菜单）提供 "Apply Layout → Solo" 和 "Apply Layout → AI + Runner"
- [ ] Solo：关所有非当前 Pane
- [ ] AI + Runner：强制右分屏 50/50，左 Pane 保持现有 shell，右 Pane 起 shell（用户可手动启动 Claude CLI）

### D. 分隔条

- [ ] 拖拽水平分隔条调整左右比例
- [ ] 拖拽垂直分隔条调整上下比例
- [ ] 双击任一分隔条 → 复位到 50/50
- [ ] 比例持久化：同 Tab 再打开保持上次比例

### E. Focus

- [ ] 点击 Pane → focus 切换，边框高亮
- [ ] 输入直接到 focus 的 Pane
- [ ] Focus 切换不打断其他 Pane 的 PTY 运行

### F. 性能

- [ ] 4 Pane 并存不额外显著增加内存（每 Pane ≈ MVP-04 单 Tab 开销）
- [ ] 拖拽分隔条 60FPS（不卡顿）
- [ ] 分屏 / 关 Pane 动画 < 150ms

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | Pane 布局树模型（LayoutNode 递归，MVP 限深度 1）+ 非法操作拒绝 |
| 视觉回归 | 4 种合法布局截图对比原型 |
| E2E | 完整流程：分屏 → 拖拽 → 关 → 应用 Layout |
| 手动 QA | 多屏不同 DPI 下分隔条精度 |

## 💾 数据模型变更

扩展 `tabs` table，加入 `layout`：
```rust
enum LayoutNode {
    Single(PaneId),
    Split { direction: SplitDir, ratio: f32, first: Box<LayoutNode>, second: Box<LayoutNode> },
}
// MVP-05 限制：LayoutNode 树深度 ≤ 2（根 + 一层 Split）
```

新 table `panes`（每个 Pane 独立 PTY）：
```rust
struct PaneState {
    pane_id: String,
    tab_id: String,               // FK
    shell: String,
    cwd: String,
    scroll_back: Vec<String>,
}
```

## ⚠️ 已知风险

- **深度限制规则易踩坑**：用户期望"最多 4 Pane 随便分" vs 现实"单层"——UI 错误提示要明确
- **Smart Layouts 覆盖用户布局**：应用预设前需二次确认（"这会关闭现有 X 个 Pane"）

## 📝 Notes

- LayoutNode 用递归 enum 实现，v0.2 扩展到多层时只需放开深度限制
- 当前 Pane focus 不跟随鼠标 hover（避免误触）

## 🔗 相关

- `implementation-plan.md` §5.3 Pane 系统
- 上游：MVP-04
- 下游：无直接（v0.2 多 Pane 预设 / 任意嵌套）

---

**自审四问**：1. 合法/非法 + 视觉覆盖 ✅ · 2. 非法操作 graceful 拒绝 ✅ · 3. 多 DPI 显式测 ✅ · 4. 任意嵌套 / 方向键 都在 v0.2 ✅
