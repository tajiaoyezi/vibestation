---
id: MVP-18
type: mvp
title: AI-Aware Pane 联动（订阅 + 失败反哺）
status: draft
owner:
phase: v1.0
depends_on: ["MVP-14", "SPIKE-07"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 15d
plan_ref: implementation-plan.md §10.1 · §5.3.6 · §1.1
risk_ref: R1
reviewer:
---

# MVP-18: AI-Aware Pane 联动

> **状态**：`draft`（**v1.0 vision**，README / landing 完全不宣传 · 占位 spec）
> **依赖**：MVP-14（Pane 高级布局已就绪）+ SPIKE-07（CLI 协议 parser 已稳定）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v1.0`](../implementation-plan.md) · [`§5.3.6 AI-Aware Pane 联动`](../implementation-plan.md) · [`§1.1`](../implementation-plan.md)

---

## 🎯 目标（Goal）

实现 AI Pane 与 Runner / Watch / Log / Build Pane 之间的**订阅+反哺**机制：
- AI 订阅某个 Pane 的失败事件
- 该 Pane 失败时（build fail / test fail / command error）自动把 `parsed_issues` 反哺给 AI 作为上下文

## 📖 背景（Context）

- **AI-Aware 是 v1.0 vision**（`CLAUDE.md` 决策表 #3 · `implementation-plan.md §1.1`），**MVP v0.1 / v0.2 / v0.3 均不实现**
- README / landing **不提及**此功能（对外叙事策略 · 禁区）
- 硬前提：SPIKE-07 **CLI 协议 parser** 已稳定（R1 已降级），没有稳定 parser 就没有可靠的 `parsed_issues` 字段
- 占位 spec；v1.0 kickoff 前必经 spike 验证

---

## 🎨 功能范围（Scope）

**Do**（v1.0 启动后详化，且 SPIKE-07 必须已通过）：
- IPC 命令 `pane:link`（parent, child, kind）
- IPC 事件 `pane:linked` / `pane:trigger` / `pane:build-failed`（`parsed_issues` 字段）
- AI Pane 接收 `pane:build-failed` 后自动把 `parsed_issues` 追加到当前对话上下文
- 用户可手动 unlink / re-link Pane
- 降级策略：parser 不可靠时 fallback 为"只贴原始文本，不做结构化"

**Don't**（明确不做）：
- 自动 trigger AI 的回复（永远需要用户手动 Enter 确认）
- 跨 workspace 的 Pane 联动（只在同一 workspace 内）
- 基于行为预测的联动建议（AI-on-AI 推理，留给 v2+）

## 🖼 UI 引用（UI Reference）

- 原型：暂未设计；详化前先出 wireframe
- v1.0 kickoff 时补截图到 `docs/tasks/assets/MVP-18/`

## ✅ Acceptance（v1.0 kickoff + SPIKE-07 pass 后详化）

骨架：
- [ ] `parsed_issues` 字段解析准确率 ≥ 95%（SPIKE-07 samples 作 fixture）
- [ ] 联动延迟 ≤ 200ms（从 build fail 到 AI Pane 收到 context）
- [ ] 用户可一键 unlink（防止 AI context 污染）
- [ ] parser 失败时降级为"原始文本贴入"，不崩溃
- [ ] UI 明确显示"哪个 Pane 正在订阅哪个 Pane"

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | `parsed_issues` 解析（基于 SPIKE-07 fixture）| `cargo test` |
| 集成 | IPC event 流 end-to-end | `cargo test --features integration` |
| E2E | Playwright 模拟 build fail → AI Pane 收 context | Playwright |

## 💾 数据模型变更

- `pane_links` 表：`{workspace_id, parent_pane_id, child_pane_id, link_kind, created_at}`

---

## 📝 Notes / 讨论

- 最大风险：parser 不稳定导致 AI 收到垃圾 context（R1 未降级则硬阻塞）
- 占位 spec，v1.0 前必经 SPIKE-07 独立验证

## 🔗 相关

- **对外宣传**：**严禁**在 README / landing / Twitter / Discord 提及（`CLAUDE.md` 决策表 #3）
- `implementation-plan.md` §10.1 · §5.3.6 · §1.1
- 上游：MVP-14 · **SPIKE-07**（R1 降级前提）
- 下游：MVP-19（session 绑定）· MVP-20（AI 回滚）

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：订阅 + 反哺 + 降级 + unlink 齐全 ✅
2. **反向场景**：parser 失败降级为原始文本 ✅
3. **边界适用性**：同 workspace 内限制明确 ✅
4. **YAGNI**：不做自动 trigger / 行为预测 ✅
