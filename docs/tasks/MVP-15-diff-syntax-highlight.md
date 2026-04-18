---
id: MVP-15
type: mvp
title: Diff 复杂语法高亮（tree-sitter / 语言感知）
status: draft
owner:
phase: v0.3
depends_on: ["MVP-08"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 6d
plan_ref: implementation-plan.md §10.1 · §3.1
risk_ref:
reviewer:
---

# MVP-15: Diff 复杂语法高亮

> **状态**：`draft`（v0.3 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-08（Diff 基础视图 · 自绘）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.3`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 MVP-08 基础行对比的 Diff 视图上，加 tree-sitter 驱动的语言感知语法高亮。

## 📖 背景（Context）

- MVP-08 Diff 只做"基础行对比 + 颜色区分增/删"，无语法高亮（`implementation-plan.md §10.1`）
- v0.3 解锁高亮能力，目标对齐 JetBrains 级阅读体验
- 技术候选：tree-sitter（已在 zed/helix 成熟使用）；避免 Monaco（§10.1 硬禁区）
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v0.3 启动后详化）：
- tree-sitter 解析 + 渲染 10+ 主流语言：Rust / TypeScript / JavaScript / Python / Go / Java / C / C++ / HTML / CSS
- 主题兼容 Calm Studio 视觉（`design/directions/1-calm-studio.html` token）
- Diff 增删高亮 × 语法高亮叠加（类 Git diff 的 `word-level diff`）
- Lazy parse：只对 viewport 可见区域 parse

**Don't**（明确不做）：
- LSP 语义高亮（需要 LSP 接入，超出 v0.3）
- 交互式编辑（Diff 视图只读）
- Monaco editor（§10.1 禁区）

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` Diff 视图
- 详化时补截图到 `docs/tasks/assets/MVP-15/`

## ✅ Acceptance（v0.3 启动后详化）

骨架：
- [ ] 10 种语言语法高亮显示正确（对比 Zed / VSCode）
- [ ] 1 万行 Diff 首屏 < 500ms（tree-sitter incremental parse）
- [ ] word-level 高亮不破坏 diff 行级增删视觉
- [ ] 主题切换（Calm Studio light / dark）颜色对比度满足 WCAG AA
- [ ] 不支持的语言降级为纯文本（不崩溃）

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | tree-sitter 适配层 | `cargo test` / `vitest` |
| 视觉回归 | 10 语言 × 2 主题 × 2 diff 模式 screenshot | Playwright screenshot diff |
| 性能 | 1 万行 diff parse benchmark | Playwright + perf.now() |

## 💾 数据模型变更

- 无新表；语法高亮结果运行时计算 + LRU 缓存

---

## 📝 Notes / 讨论

- tree-sitter wasm vs native：两种方案性能 / 包大小差异大，详化时 benchmark
- 占位 spec

## 🔗 相关

- `implementation-plan.md` §10.1 · §3.1
- 上游：MVP-08
- 下游：无（自成一功能）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：10 语言 + 主题 + lazy parse 齐全 ✅
2. **反向场景**：不支持语言降级纯文本 ✅
3. **边界适用性**：1 万行性能门槛 ✅
4. **YAGNI**：不做 LSP，tree-sitter 够 v0.3 ✅
