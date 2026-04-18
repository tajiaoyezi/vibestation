---
id: MVP-19
type: mvp
title: AI session ↔ commit 自动绑定
status: draft
owner:
phase: v1.0
depends_on: ["MVP-18"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1 · §5.3.6 · §1.1
risk_ref: R1
reviewer:
---

# MVP-19: AI session ↔ commit 自动绑定

> **状态**：`draft`（**v1.0 vision**，README / landing 完全不宣传 · 占位 spec）
> **依赖**：MVP-18（AI-Aware Pane 联动已就绪）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v1.0`](../implementation-plan.md) · [`§5.3.6`](../implementation-plan.md)

---

## 🎯 目标（Goal）

把一次完整的 AI 对话（Claude / Codex CLI session）识别为一个**逻辑工作单元**，并自动关联到它期间产生的 commits，使得：
- 一键看"这个 commit 是 AI 哪段对话产出的"
- 一键看"这个 session 一共改了哪些文件 / 产生了哪些 commit"

## 📖 背景（Context）

- **AI-Aware v1.0 升级故事的核心**（`implementation-plan.md §1.1`）：AI 作为一等公民参与版本控制，session 是"AI 工作单元"
- 硬前提：MVP-18 的 `parsed_issues` 和 session boundary 已可稳定识别（SPIKE-07 R1 已降级）
- **对外宣传禁区**（`CLAUDE.md` #3）
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v1.0 启动后详化）：
- Session 边界识别：
  - Claude CLI：`/clear` / 新进程启动 / 用户手动标记
  - Codex CLI：同上（等 SPIKE-07 确认具体标识）
- Session 元数据：`{id, cli_kind, started_at, ended_at, prompt_count, token_count?, title}`
- Commit 绑定：Git Log 中每个 commit 若产生自某 session，显示小徽章（AI icon）
- 反向查询：点击 session 徽章 → 打开 session 详情视图（commit 列表 + 原始对话摘要 · 脱敏）
- 用户可**手动解绑** session 与 commit（避免误关联）

**Don't**（明确不做）：
- 跨 workspace 的 session 聚合（只在 workspace 内）
- 多 AI 模型的统一抽象（Claude / Codex / Gemini / ... 先只支持前两个）
- Session 的语义分类（"feature" / "bugfix" / "refactor"）留给 v2+

## 🖼 UI 引用（UI Reference）

- 原型：暂未设计；详化前先出 wireframe
- v1.0 kickoff 时补截图到 `docs/tasks/assets/MVP-19/`

## ✅ Acceptance（v1.0 kickoff 后详化）

骨架：
- [ ] Session 边界识别准确率 ≥ 90%（SPIKE-07 samples 作 fixture）
- [ ] Commit ↔ session 关联关系正确率 ≥ 95%（按时间窗口 + diff 签名双重校验）
- [ ] 用户可一键解绑（解除错误关联）
- [ ] Session 详情视图的原始对话**已脱敏**（auth token / PII 必过 gitleaks）
- [ ] 数据持久化到 redb · migration 安全（参考 SPIKE-04 B.3）

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | session 边界识别纯函数 | `cargo test` |
| 集成 | IPC + redb 存储 round-trip | `cargo test --features integration` |
| E2E | Playwright 模拟完整 session + 3 commit 流程 | Playwright |

## 💾 数据模型变更

- `ai_sessions` 表：`{id, workspace_id, cli_kind, started_at, ended_at, prompt_count, title}`
- `session_commit_links` 表：`{session_id, commit_sha, auto_bound: bool, confidence: f32}`

---

## 📝 Notes / 讨论

- Session 边界识别的"时间窗口"阈值（例 5 min idle 算 session 结束）需要用户可调
- 占位 spec

## 🔗 相关

- **对外宣传禁区**（`CLAUDE.md` #3）
- `implementation-plan.md` §10.1 · §5.3.6 · §1.1
- 上游：MVP-18 · SPIKE-07
- 下游：MVP-20（AI 一键回滚）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：识别 + 绑定 + 反查 + 解绑 四件事齐全 ✅
2. **反向场景**：手动解绑路径明确 ✅
3. **边界适用性**：准确率门槛清晰 ✅
4. **YAGNI**：语义分类 / 跨 workspace 聚合留给 v2+ ✅
