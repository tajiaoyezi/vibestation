---
id: MVP-12
type: mvp
title: 自绘 commit rail graph（Git Log 图形化）
status: draft
owner:
phase: v0.2
depends_on: ["MVP-07"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1 · §5.4
risk_ref:
reviewer:
---

# MVP-12: 自绘 commit rail graph

> **状态**：`draft`（v0.2 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-07（Git Log 只读视图已就绪）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.2`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 MVP-07 Git Log 左侧加自绘的 rail graph（commit DAG 可视化），呈现分支 / 合并 / 分叉关系。

## 📖 背景（Context）

- MVP v0.1 为降风险只做"分支 / tag 标签贴"（`implementation-plan.md §10.1`），rail graph 砍到 v0.2
- rail graph 实现难点：多分支并行 + `merge` commit 的 rail 布局（Shao et al. 算法 / gitgraph.js 算法参考）
- 本 PR 是**占位 spec**，v0.2 启动时详化

---

## 🎨 功能范围（Scope）

**Do**（v0.2 启动后详化）：
- Git Log 最左侧 10% 宽度显示 rail graph（canvas 自绘，非 SVG 慎用）
- 每条 commit 对应一个节点 + 色彩标识其所属分支
- merge commit 显示两条入边 汇合
- 滚动时 rail graph 虚拟化渲染（100 万 commit 不卡）
- 分支数 ≤ 20 时清晰可读（超过 collapse 到 "other"）

**Don't**（明确不做）：
- 交互式 rebase 操作（v0.3 MVP-16 范围）
- 跨 remote 的图（只显示 local refs + origin/*）

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` Git Log 模式（Secondary Sidebar）预留左 rail 区
- 详化时补截图到 `docs/tasks/assets/MVP-12/`

## ✅ Acceptance（v0.2 启动后详化）

骨架：
- [ ] 10 万 commit 的 linux kernel 仓库 rail graph 首屏 < 500ms
- [ ] 滚动 60fps（Canvas 虚拟化命中）
- [ ] 分叉 / 合并视觉清晰（5 分支并行场景有截图）
- [ ] Hover commit 节点高亮该 commit 的完整 rail 路径
- [ ] 颜色方案对色盲友好（WCAG AA）

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | rail 布局算法纯函数 | `cargo test` / `vitest` |
| 视觉回归 | 5 典型 repo 的 rail screenshot | Playwright screenshot diff |
| 性能 | 10 万 commit benchmark | Playwright + perf.now() |

## 💾 数据模型变更

- 无新表，rail 位置实时计算（可缓存到 `commit_rail_cache` 临时表降低重算）

---

## 📝 Notes / 讨论

- 算法候选：gitgraph.js / git-graph-rs 移植；先 spike 一下再详化
- 占位 spec

## 🔗 相关

- `implementation-plan.md` §10.1 · §5.4
- 上游：MVP-07
- 下游：MVP-13（分支操作）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：Scope + Acceptance 骨架齐全 ✅
2. **反向场景**：超 20 分支 collapse 策略已说 ✅
3. **边界适用性**：10 万 commit 有性能门槛 ✅
4. **YAGNI**：占位阶段不实现算法，v0.2 启动后 spike ✅
