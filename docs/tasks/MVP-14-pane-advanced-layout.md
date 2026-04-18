---
id: MVP-14
type: mvp
title: Pane 高级布局（任意嵌套 + Dual AI / Triple / Quad + 导航 + 最大化）
status: draft
owner:
phase: v0.2
depends_on: ["MVP-05"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 7d
plan_ref: implementation-plan.md §10.1 · §5.3
risk_ref:
reviewer:
---

# MVP-14: Pane 高级布局

> **状态**：`draft`（v0.2 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-05（单层 Pane 分屏已就绪）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.2`](../implementation-plan.md) · [`§5.3`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 MVP-05 基础上解锁 Pane 任意嵌套 + 3 个新预设（Dual AI / Triple Review / Quad）+ 方向键导航 + ⌘Enter 临时最大化。

## 📖 背景（Context）

- MVP-05 只允许"单层嵌套最多 4 Pane + Solo / AI+Runner 两预设"
- v0.2 解锁全部高级布局能力（`implementation-plan.md §5.3`）
- 任意嵌套的实现风险：SolidJS 递归组件重渲染，超 3 层可能掉帧（见 §5.3.9 性能风险）
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v0.2 启动后详化）：
- 任意嵌套深度（实测上限 5 层，超出弹 toast）
- 3 个新预设：Dual AI（H(0.5, ClaudeCli | CodexCli)）/ Triple Review（H(0.5, AI | V(0.5, Runner | Log))）/ Quad（2×2）
- 方向键导航：`⌘⌥ ←/→/↑/↓` 跳相邻 Pane
- `⌘Enter` 临时最大化当前 Pane，再按恢复

**Don't**（明确不做）：
- Detach 独立窗口（v0.3 MVP-17 范围）
- 自定义预设保存到 TOML（v0.3）
- AI-Aware Pane 联动（v1.0 MVP-18）

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` Pane 区（§5.3.5 图示）
- 详化时补截图到 `docs/tasks/assets/MVP-14/`

## ✅ Acceptance（v0.2 启动后详化）

骨架：
- [ ] 3 预设一键切换（Smart Layouts 菜单）+ 持久化到 workspace
- [ ] 方向键导航在 4 Pane 布局里跳转正确（不跨越分隔条误跳）
- [ ] ⌘Enter 最大化动画 < 200ms，恢复保持原分隔条比例
- [ ] 5 层嵌套 Pane 下主线程阻塞 ≤ 16ms（60fps）
- [ ] 键盘可达性：全部操作可用键盘完成

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | LayoutNode tree 增删改查 | `cargo test` |
| 集成 | 预设切换 + 持久化 round-trip | `cargo test --features integration` |
| E2E | Playwright 模拟方向键 + ⌘Enter | Playwright |
| 性能 | 5 层嵌套 FPS benchmark | Playwright + perf.now() |

## 💾 数据模型变更

- `workspace.layout` 字段从 "preset name" 扩展为"完整 LayoutNode JSON"（向后兼容 MVP-05 的 preset enum）

---

## 📝 Notes / 讨论

- 任意嵌套的递归渲染优化可能需要 `<For>` + `createMemo`，详化时 spike 一下
- 占位 spec

## 🔗 相关

- `implementation-plan.md` §10.1 · §5.3 · §5.3.5 / §5.3.6 / §5.3.9
- 上游：MVP-05
- 下游：MVP-17（Pane Detach）· MVP-18（AI-Aware 联动 · v1.0）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：任意嵌套 + 3 预设 + 导航 + 最大化 四件事齐全 ✅
2. **反向场景**：5 层嵌套上限 + 超限 toast ✅
3. **边界适用性**：60fps 性能门槛 ✅
4. **YAGNI**：Detach / 自定义预设留给后续 ✅
