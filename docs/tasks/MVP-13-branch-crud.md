---
id: MVP-13
type: mvp
title: 分支 create / checkout / delete
status: draft
owner:
phase: v0.2
depends_on: ["MVP-07", "MVP-09"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5.4
risk_ref:
reviewer:
---

# MVP-13: 分支 create / checkout / delete

> **状态**：`draft`（v0.2 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-07（Git Log 只读）+ MVP-09（git2 写路径）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.2`](../implementation-plan.md)

---

## 🎯 目标（Goal）

Git Log 视图支持新建分支 / 切换分支 / 删除分支三个基础分支操作。

## 📖 背景（Context）

- MVP v0.1 分支只能"看"（标签贴），不能"改"
- v0.2 补齐基本分支 CRUD，让用户不再依赖终端
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v0.2 启动后详化）：
- `git branch <name>`：从当前 HEAD 新建分支（不切换）
- `git checkout -b <name>`：新建并切换
- `git checkout <existing>`：切换到已有分支（本地 dirty 检查）
- `git branch -d <name>`：安全删除（未合并拒绝）
- `git branch -D <name>`：强制删除（UI 二次确认）
- Remote 分支：checkout 远端分支自动 track

**Don't**（明确不做）：
- Rebase / merge（v0.3 MVP-16 范围）
- Branch rename（v0.3）
- 跨 submodule 分支操作

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` 分支树（Primary Sidebar 下方）
- 详化时补截图到 `docs/tasks/assets/MVP-13/`

## ✅ Acceptance（v0.2 启动后详化）

骨架：
- [ ] Primary Sidebar 分支树右键菜单：New Branch / Checkout / Delete
- [ ] Checkout 时 dirty working tree 提示 "stash / discard / cancel"
- [ ] Delete 未合并分支时 `-d` 拒绝 + UI 提示可用 `-D`
- [ ] 新建分支后分支树即时刷新，无需手动 refresh
- [ ] 快捷键 `⌘⇧B` 打开 Branch Switcher（fuzzy search）

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | git2 branch CRUD 包装 | `cargo test` |
| 集成 | 本地 repo 做 CRUD 端到端 | `cargo test --features integration` |
| E2E | Playwright 模拟右键 / fuzzy search | Playwright |

## 💾 数据模型变更

- 无新表，分支列表实时查 git2

---

## 📝 Notes / 讨论

- stash 流程如果走 git2 需要检查 stash API 稳定性，可能要单独 spike
- 占位 spec

## 🔗 相关

- `implementation-plan.md` §10.1 · §5.4
- 上游：MVP-07 / MVP-09
- 下游：MVP-16（rebase / merge / cherry-pick）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：create + checkout + delete 三件套齐全 ✅
2. **反向场景**：dirty tree / 未合并分支保护已说 ✅
3. **边界适用性**：remote 分支 track 明确 ✅
4. **YAGNI**：占位阶段不写 stash 流程 ✅
