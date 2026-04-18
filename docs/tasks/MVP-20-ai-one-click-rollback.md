---
id: MVP-20
type: mvp
title: AI 一键回滚（session 级 revert）
status: draft
owner:
phase: v1.0
depends_on: ["MVP-19"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 6d
plan_ref: implementation-plan.md §10.1 · §5.3.6 · §1.1
risk_ref: R1
reviewer:
---

# MVP-20: AI 一键回滚（session 级 revert）

> **状态**：`draft`（**v1.0 vision**，README / landing 完全不宣传 · 占位 spec）
> **依赖**：MVP-19（session ↔ commit 绑定已就绪）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v1.0`](../implementation-plan.md) · [`§5.3.6`](../implementation-plan.md)

---

## 🎯 目标（Goal）

基于 MVP-19 的 session 绑定，提供"一键回滚整个 AI session 产生的所有 commit"的安全操作：相当于 `git revert` 一批 commit 的原子包装 + UI 化。

## 📖 背景（Context）

- **AI-Aware v1.0 的收尾功能**：session 级 revert 对"AI 试错"场景极有价值（AI 改了但没改好，一键恢复）
- 硬前提：MVP-19 的 session↔commit 关联必须稳定（≥ 95% 准确率）
- 回滚等价于 `git revert <commit1> <commit2> ... <commitN>`（保留历史，不 reset）
- **对外宣传禁区**（`CLAUDE.md` #3）
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v1.0 启动后详化）：
- Session 详情视图顶部加"一键回滚"按钮
- 点击 → 预览 revert diff（用户确认后才执行）
- 执行 = `git revert` 该 session 所有 commit（保留原 commit 在历史中）
- 冲突处理：若任一 revert 冲突 → 停在该 commit + 进 MVP-08 Diff conflict resolver
- 用户可 `--abort` 回到 revert 开始前的状态
- revert 生成的新 commit message 统一加后缀 `[AI session rollback: <session-id>]`

**Don't**（明确不做）：
- `git reset --hard`（危险 · 永远不提供 UI 入口）
- 部分 revert（例如"只回滚 session 里的其中 2 个 commit"）· 留给 v2+
- 跨 session 的 combined revert · 留给 v2+

## 🖼 UI 引用（UI Reference）

- 原型：暂未设计；详化前先出 wireframe
- v1.0 kickoff 时补截图到 `docs/tasks/assets/MVP-20/`

## ✅ Acceptance（v1.0 kickoff 后详化）

骨架：
- [ ] Session 详情视图顶部有明确的"一键回滚"按钮（红色系警告色）
- [ ] 预览 diff 清晰显示"将新增 N 个 revert commit"
- [ ] 冲突解决路径与 MVP-16 rebase / merge 一致
- [ ] 操作中途 `--abort` 能干净回到起点（0 残留 commit）
- [ ] revert 完成后 session 详情视图标记"已回滚"（但不删 session 历史）

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | git2 multi-commit revert 包装 | `cargo test` |
| 集成 | 构造 5 commit session → revert 全部 → 验证 tree | `cargo test --features integration` |
| E2E | Playwright 模拟完整操作（含冲突场景）| Playwright |

## 💾 数据模型变更

- `ai_sessions` 表加 `rolled_back_at: Option<timestamp>` 和 `rollback_commit_shas: Vec<String>` 字段

---

## 📝 Notes / 讨论

- 危险操作 · 必须二次确认（`CLAUDE.md` "🚫 禁区" 中 reset 禁止，revert 因保留历史可接受）
- 占位 spec

## 🔗 相关

- **对外宣传禁区**（`CLAUDE.md` #3）
- `implementation-plan.md` §10.1 · §5.3.6 · §1.1
- 上游：MVP-19
- 下游：无（v1.0 收尾功能）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：预览 + 执行 + 冲突 + abort 路径齐全 ✅
2. **反向场景**：abort 干净回起点明确 ✅
3. **边界适用性**：不做 reset / 部分 revert ✅
4. **YAGNI**：跨 session / 部分 revert 留给 v2+ ✅
