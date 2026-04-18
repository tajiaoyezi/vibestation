---
id: MVP-11
type: mvp
title: Git Push / Pull / Fetch（远端同步）
status: draft
owner:
phase: v0.2
depends_on: ["MVP-09"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1（MVP 范围 · push/pull/fetch 砍到 v0.2）
risk_ref:
reviewer:
---

# MVP-11: Git Push / Pull / Fetch（远端同步）

> **状态**：`draft`（v0.2 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-09（commit 写路径已通）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.2`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 MVP-09 commit 基础上补远端同步三件套：`git push` / `git pull` / `git fetch`，让用户不必回终端。

## 📖 背景（Context）

- `implementation-plan.md §10.1` 明确把 Push / Pull / Fetch 砍到 v0.2（MVP-09 只做本地 commit）
- 本 PR 是**占位 spec**：字段完整 + 粗粒度 Scope + Acceptance 骨架；v0.2 启动时再详化
- 详化触发条件：MVP-09 done + v0.2 kickoff 确认范围

---

## 🎨 功能范围（Scope）

**Do**（v0.2 启动后详化）：
- `git push` 支持 HTTPS + SSH + Git 原生三种 remote
- `git pull` 支持 rebase / merge 策略二选一（默认 merge）
- `git fetch --all` 刷新所有 remote 的 refs
- 认证：SSH key 走系统 agent；HTTPS 走凭据 helper（`git credential-<os>`）
- 错误处理：rejected / non-fast-forward / merge conflict 有明确 UI 提示

**Don't**（明确不做）：
- Force push（推到 v0.3，默认禁用，UI 需二次确认）
- Remote add / remove（用户用终端 `git remote`）
- Submodule 的 push/pull 级联（保持 v1 不崩即可）

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` Git Log 工具栏右侧预留按钮位
- 详化时补截图到 `docs/tasks/assets/MVP-11/`

## ✅ Acceptance（v0.2 启动后详化）

骨架：
- [ ] Push 成功后 Git Log 视图立即刷新 remote tracking branch
- [ ] Pull conflict 场景给出清晰 UI（哪些文件冲突 + 如何解决提示）
- [ ] Fetch 后 status 栏显示 `behind N / ahead M`
- [ ] 认证失败提示对应 remote + 建议配置 SSH key 或 credential helper
- [ ] 跨平台（macOS + Ubuntu）Push / Pull / Fetch 均可用

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | `core/` git2 push/pull 包装 | `cargo test` |
| 集成 | 本地 bare repo 做 remote 做端到端 push/pull | `cargo test --features integration` |
| E2E | Playwright 模拟用户点 Push 按钮 | Playwright |

## 💾 数据模型变更

- 无新表，复用 MVP-07 的 commit 表（remote refs 缓存在 `refs_cache` 临时表）

---

## 📝 Notes / 讨论

- 占位 spec，v0.2 kickoff 时详化。优先级参考 v0.2 路线图。

## 🔗 相关

- 对应 `CLAUDE.md` 决策表：#13（git2 0.20 写路径）
- `implementation-plan.md` §10.1 · §5.4
- 上游：MVP-09
- 下游：MVP-13（分支操作）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：Scope + Acceptance 骨架齐全 ✅
2. **反向场景**：认证失败 / conflict 明确在 Don't / Acceptance 提及 ✅
3. **边界适用性**：两平台都要验 ✅
4. **YAGNI**：占位阶段不详化，v0.2 kickoff 再补 ✅
