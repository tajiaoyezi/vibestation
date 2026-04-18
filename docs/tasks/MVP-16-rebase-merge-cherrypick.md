---
id: MVP-16
type: mvp
title: Rebase / Merge / Cherry-pick
status: draft
owner:
phase: v0.3
depends_on: ["MVP-13"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 7d
plan_ref: implementation-plan.md §10.1（MVP 范围 · rebase/merge/cherry-pick 砍到 v0.3）
risk_ref:
reviewer:
---

# MVP-16: Rebase / Merge / Cherry-pick

> **状态**：`draft`（v0.3 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-13（分支 CRUD 已就绪）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.3`](../implementation-plan.md)

---

## 🎯 目标（Goal）

补齐三大分支操作：rebase（含交互式）/ merge（含 --squash / --no-ff）/ cherry-pick（单 commit 或 range）。

## 📖 背景（Context）

- 分支操作是 JetBrains 级 Git 工作台的核心差异化（vs GitKraken 的图形化、vs CLI 的效率）
- v0.3 解锁这 3 个操作后，用户不再需要回终端做大部分 Git 工作
- 风险：交互式 rebase 的 UI 设计复杂（步骤 / 冲突解决 / 放弃 / 恢复）
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v0.3 启动后详化）：
- **Rebase**：
  - 常规 rebase onto `<target>`
  - 交互式 rebase（reword / squash / fixup / drop / edit 五种操作）
  - 冲突时进 conflict resolver（调用 MVP-08 Diff 视图）
  - 中断恢复（`--continue` / `--abort` / `--skip`）
- **Merge**：
  - Fast-forward / --no-ff / --squash 三种策略
  - 冲突解决同上
- **Cherry-pick**：
  - 单 commit / range / from another branch
  - `--no-commit` 选项（只放入 working tree）

**Don't**（明确不做）：
- 跨 remote 的 rebase（merge 为主）
- `git reflog` 恢复（v1.0 范围）
- Stash 的交互式管理（v1.0）

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` Git Log 右键菜单 + 独立"Rebase Editor"视图
- 详化时补截图到 `docs/tasks/assets/MVP-16/`

## ✅ Acceptance（v0.3 启动后详化）

骨架：
- [ ] 交互式 rebase editor UI 支持 5 种操作 + 拖拽排序
- [ ] 冲突解决使用与 MVP-08 一致的 Diff 视图
- [ ] Rebase / merge / cherry-pick 中断后恢复路径清晰（UI 顶部醒目 banner）
- [ ] 10 个 commit 的交互式 rebase 流程录屏可作 demo
- [ ] 错误提示具体（不是 "git command failed" 而是 "rebase conflict on file X, line Y"）

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | git2 rebase / merge / cherry-pick 包装 | `cargo test` |
| 集成 | 本地 repo 构造冲突场景端到端 | `cargo test --features integration` |
| E2E | Playwright 模拟完整交互式 rebase | Playwright |

## 💾 数据模型变更

- `rebase_state` 临时表：存储进行中的 rebase 状态（workspace_id, branch, onto, step_index, remaining_steps）

---

## 📝 Notes / 讨论

- 交互式 rebase UI 参考：Fork / Tower / SmartGit；目标是比三者都顺手
- 占位 spec

## 🔗 相关

- `implementation-plan.md` §10.1 · §5.4
- 上游：MVP-13
- 下游：无（自成一功能）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：rebase + merge + cherry-pick 三件套 + 交互式齐全 ✅
2. **反向场景**：中断 / conflict / abort 路径已说 ✅
3. **边界适用性**：冲突 UI 复用 MVP-08 ✅
4. **YAGNI**：reflog / stash 交互式留给 v1.0 ✅
