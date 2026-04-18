---
id: MVP-15
type: mvp
title: Diff 语法高亮（shiki lazy load · 对齐 W21）
status: draft
owner:
phase: v0.3
depends_on: ["MVP-08"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §W21（W21 表格 · shiki lazy load + 大文件流式）
risk_ref:
reviewer:
---

# MVP-15: Diff 语法高亮（shiki lazy load）

> **状态**：`draft`（v0.3 · 占位 spec · **对齐 `implementation-plan.md §W21`**）
> **依赖**：MVP-08（Diff 基础视图 · 自绘）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.3`](../implementation-plan.md) · [`§W21`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 MVP-08 基础行对比的 Diff 视图上 · 对齐 `implementation-plan.md §W21` 的具体范围加 **shiki lazy load 语法高亮 + 大文件流式加载**。

## 📖 背景（Context）

- MVP-08 Diff 只做"基础行对比 + 颜色区分增/删"（`implementation-plan.md §10.1`）
- `implementation-plan.md §W21` 明确 v0.3 高级 Diff 范围：`shiki lazy load` + `大文件流式加载` · 目标 `1MB 文件 diff <300ms`
- 技术候选由 v0.3 kickoff 评估 · 本占位 spec **不 pre-decide 具体 parser 架构**（Codex PR #10 review F4 教训：占位 spec 不得引入 scope creep · 只做 implementation-plan 约定范围）

---

## 🎨 功能范围（Scope）

**Do**（v0.3 kickoff 后详化 · 严格对齐 `§W21`）：
- **shiki lazy load 语法高亮**（主线 · `§W21` 指定方案）
  - 主流语言覆盖（具体清单 v0.3 kickoff 基于用户反馈定）
  - Lazy load：只对 viewport 可见行加载 shiki theme + 解析
- **大文件流式加载**（主线 · `§W21` 指定行为）
  - 1MB 文件 diff 首屏 < 300ms（`§W21` 验收）
  - 超 1MB 文件降级为"流式渲染 + 分段加载"

**Don't**（明确排除 · 防 scope creep）：
- tree-sitter 方案（未经 implementation-plan 批准 · 若 v0.3 kickoff 决定换 tree-sitter · **先更新 `§W21` + ADR** 再改本 spec）
- word-level diff（超出 `§W21` 范围 · 若真要做 · v0.3 kickoff 开新 spec 或扩展本 spec · 但先走 implementation-plan 批准）
- LSP 语义高亮（v0.3+ 不做）
- 交互式编辑（Diff 视图只读）
- Monaco editor（`§10.1` 硬禁区）

## 🖼 UI 引用

- 原型：`design/directions/1-calm-studio.html` Diff 视图区
- 详化时补截图到 `docs/tasks/assets/MVP-15/`

## ✅ Acceptance（v0.3 启动后详化 · 对齐 `§W21` 验收）

骨架：
- [ ] **`§W21` 硬指标**：1MB 文件 diff 首屏 < 300ms（shiki lazy load + 流式）
- [ ] shiki 主题兼容 Calm Studio light / dark（WCAG AA 对比度）
- [ ] 不支持的语言降级为纯文本（不崩溃）
- [ ] **10MB+ 文件**：流式加载 · 主线程阻塞 ≤ 16ms
- [ ] 切换主题（light / dark）瞬时生效 · 无重 parse

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | shiki 适配层 + lazy loader | `vitest` |
| 视觉回归 | 若干主流语言 × 2 主题 screenshot | Playwright screenshot diff |
| 性能 | 1MB / 10MB diff 首屏 benchmark | Playwright + perf.now() |

## 💾 数据模型变更

- 无新表 · shiki theme + parse 结果运行时 LRU 缓存

---

## 📝 Notes / 讨论

- **占位 spec 原则**（Codex PR #10 R7 教训）：本 spec 严格对齐 `§W21` · 不 pre-decide tree-sitter / word-level 等更重方案
- v0.3 kickoff 时若决定扩大范围（如换 tree-sitter） → **先更新 `implementation-plan.md §W21` + 开新 ADR** · 再改本 spec
- 候选方案评估留 v0.3 kickoff：shiki 的 wasm vs server-side · 大文件流式的分段大小等

## 🔗 相关

- `implementation-plan.md` §10.1 · §W21
- 上游：MVP-08
- 下游：无（自成一功能）
- 对应 `CLAUDE.md` 决策表：#7（Diff 自建 · [ADR-008](../adr/ADR-008-diff-renderer-custom.md)）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：`§W21` 定义的 shiki + 流式两件事齐全 ✅
2. **反向场景**：不支持语言降级纯文本 · 10MB+ 流式降级 ✅
3. **边界适用性**：`§W21` 1MB < 300ms 硬指标保留 ✅
4. **YAGNI**：不 pre-decide tree-sitter · 不扩 word-level · 严格对齐 implementation-plan 范围 ✅
