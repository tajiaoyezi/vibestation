---
id: MVP-07
type: mvp
title: Git Log 只读视图 + Commit 详情
status: draft
owner:
phase: W8-W9
depends_on: ["MVP-02", "MVP-03", "SPIKE-03"]
blocks: ["MVP-08"]
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §10.2（10 万 commit < 500ms）
risk_ref: R3
reviewer:
---

# MVP-07: Git Log 只读视图

> **状态**：`draft`
> **依赖**：MVP-02（workspace → repo）· MVP-03（Secondary Sidebar 容器）· SPIKE-03（git 读性能锁定 git2 或 gix）
> **阻塞**：MVP-08（Diff 基于 commit 详情）

---

## 🎯 目标（Goal）

在 Secondary Sidebar 显示 workspace 对应 repo 的 commit log（只读），点击 commit 打开详情视图（元数据 + 变更文件列表）。10 万 commit 首屏 < 500ms。

## 📖 背景（Context）

- `CLAUDE.md` #13（B 栏 → SPIKE-03 锁定）：Git 栈 = git2 0.20 默认，读路径可能引入 gix 0.70 混用
- 禁区：**不做自绘 rail graph**（砍到 v0.2）
- 禁区：**不做 Diff 复杂语法高亮**（MVP-08 只做基础对比）

---

## 🎨 功能范围（Scope）

**Do**：
- Secondary Sidebar 展开 → 显示 Git Log
- 列表每行：commit hash（短 7 位）+ message 第一行 + author name + 相对时间
- 分支 / tag 标签贴（**不画 rail graph**）
- 分页加载：初始 100 条，滚动到底触发加载更多（+ 100）
- 10 万 commit 首屏 < 500ms（`§10.2`）
- 点击 commit → 主区域 Tab 旁打开"Commit 详情"视图
- Commit 详情：hash 全长 / author email / authored date / committer / committed date / parent(s) / 变更文件列表（仅列名 + 加减行数）
- 筛选：按 author / 消息关键词（顶部搜索框）

**Don't**：
- 自绘 rail graph（v0.2+）
- Branch create / checkout / delete（v0.2+）
- Diff 内容（→ MVP-08）
- Cherry-pick / rebase / merge（v0.3+）

## 🖼 UI 引用

- Secondary Sidebar: `design/directions/1-calm-studio.html` 右侧 Git Log 区
- 标签贴：浅色底 + 主色边框 + `branch`/`tag` 文字（不同颜色区分）
- Commit 详情主区：顶部元数据 + 下部文件列表，每文件 `src/foo.rs +10 -5`

## ✅ Acceptance

### A. Log 列表

- [ ] 打开含 git 的 workspace → Secondary Sidebar toggle 显示 Git Log
- [ ] 初始加载最近 100 commit，耗时 < 500ms（linux kernel 仓库测）
- [ ] 每行显示：短 hash / message 首行 / author / 相对时间 (`2 hours ago`)
- [ ] 分支 / tag 标签贴显示在对应 commit 旁
- [ ] 滚动到底触发加载下 100 条，无明显卡顿

### B. Commit 详情

- [ ] 点击 commit 行 → 主区新开 "Commit: {shortsha}" tab（不是终端 Tab，是视图 Tab）
- [ ] 详情页显示：hash / author email / 日期 / parent(s) / 消息全文 / 变更文件列表
- [ ] 文件列表每行：路径 + 加减行数（`+10 -5`）+ 状态（M/A/D/R）
- [ ] 点击文件行 → 打开 Diff 视图（MVP-08 接管）

### C. 筛选 / 搜索

- [ ] 顶部搜索框，输入回车过滤
- [ ] 支持：message 关键词（case-insensitive substring）+ `author:name` + `after:2024-01-01`
- [ ] 过滤结果也应用分页（前 100 + 加载更多）

### D. 性能（`§10.2` + SPIKE-03）

- [ ] 10 万 commit 仓库首屏 < 500ms（多次取 P99）
- [ ] 滚动加载下 100 条 < 200ms
- [ ] Commit 详情打开 < 100ms（元数据 + 文件列表）
- [ ] 筛选/搜索 < 500ms（10 万 commit 基数）

### E. 错误处理

- [ ] 非 git workspace → Secondary Sidebar 显示"This workspace has no git repo"
- [ ] git 仓库损坏 → 明确错误提示（不崩溃）
- [ ] 超大 commit（>10万文件变更）→ 文件列表限显 1000 条 + "Show all" 按钮

### F. 多 workspace 隔离

- [ ] 切换 workspace → Git Log 刷新为对应 repo
- [ ] 多 workspace 并存时，Git Log 状态（滚动位置 / 筛选）per-workspace 独立

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元（core）| git2 / gix 读逻辑 + 元数据解析 |
| 集成 | 小 / 中 / 大 fixture 仓库（1k / 10k / 100k commit）|
| 性能 | linux kernel benchmark（对齐 SPIKE-03）|
| E2E | workspace 打开 → 滚动 → 点 commit → 详情 |

## 💾 数据模型变更

新 table `git_log_cache`（可选，用于加速重复打开）：
- key: `workspace_id + branch_head_sha`
- value: `Vec<CommitSummary>`（最近 1000 条）
- TTL: invalidate when `branch_head_sha` 变化

## ⚠️ 已知风险

- **R3**（git2 大仓库 log 慢，SPIKE-03 消除）：MVP-07 要验证实际 10 万 commit 场景
- **超大 commit**（kernel 全量 merge）文件列表可能卡住 → 限 1000 + lazy load
- **gix 引入成本**：若 SPIKE-03 走 B 路径（git2 写 + gix 读），MVP-07 需两个 crate 并存 bundle +2MB

## 📝 Notes

- MVP-07 **完全不自绘 commit graph**——列表视图即可，图形在 v0.2
- Commit 详情 Tab 和终端 Tab 并列，但类型不同（enum TabKind）

## 🔗 相关

- `CLAUDE.md` #13 Git 栈
- SPIKE-03 git2/gix benchmark
- `implementation-plan.md` §10.1 · §10.2 · §9 R3
- 上游：MVP-02 · MVP-03 · SPIKE-03
- 下游：MVP-08

---

**自审四问**：1. Log + 详情 + 筛选 + 错误 + 多 workspace 覆盖 ✅ · 2. 10 万 commit + 超大变更 边界 ✅ · 3. 三平台性能都测 ✅ · 4. rail graph / branch ops 都推后 ✅
