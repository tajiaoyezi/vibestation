<!--
  Pull Request Template · Vibestation
  本模板强制 PR 含必要字段，对齐 CLAUDE.md § 5 步导游 + docs/tasks/README.md 流程
  必须字段不要删除；可选章节按需填写。
-->

## Summary

<!-- 1-3 句话：这个 PR 做了什么，解决哪个 task / issue -->

## Linked Task / Issue

<!-- 必填：
  - Task spec: `docs/tasks/<TYPE-NN-slug>.md`（若本 PR 实施某 task）
  - Issue: closes #NN（若本 PR 解决 issue）
  - 无关联（纯基础设施 / 文档）：说明原因
-->

- Task spec: `docs/tasks/XXX.md`
- Closes: #

## Implemented by

<!-- 作者 agent-id · 对应 commit trailer 的 Co-authored-by -->
- <agent-id>（例：Claude Code / Codex CLI / Human @leaf）

## Reviewed by

<!-- merge 前填写 · 必须 ≠ Implemented by -->
- 待填

## Task Status Transition

<!-- 仅当本 PR 是 task spec 创建 / 实施 PR 时填写；其他 PR（基础设施 / 文档）跳过 -->

- [ ] 本 PR 触发 `draft → ready` 翻转（spec 创建 PR）· 走**翻转 gate**（见 `docs/tasks/README.md` 第 7 步）
- [ ] 本 PR 触发 `in-progress → done` 翻转（实施 PR）· 走**翻转 gate**（见 `CLAUDE.md` 5.4 步）
- [ ] 本 PR **不涉及** task spec status 翻转（基础设施 / 文档 / 其他）

**翻转 gate 二选一（仅当上面任一勾选时必选）**：
- [ ] (a) Reviewer 自己 push 翻转 commit（推荐）
- [ ] (b) Author push 后 Reviewer re-approve 最新 HEAD

## Changes

<!-- 关键变更列表（文件 / 模块 / 行为）· 代码 PR 列 diff 摘要 · 文档 PR 列 section 摘要 -->

-
-

## Test Plan

<!-- 勾选式 · 按 task Acceptance 逐项对照 · 无 task 的基础设施 PR 描述自检项 -->

- [ ]
- [ ]

## Self Review · 自审四问（CLAUDE.md · 所有写规则 / 清单 / 流程 PR 必答）

<!-- 代码实施 PR 可跳过（走 task Acceptance）· 规则 / 文档 / spec / 流程 PR 必答 -->

- [ ] **递归完备性**：规则适用于定义规则的文档自己吗？清单自己在清单里吗？
- [ ] **反向场景**：规则不遵守会怎样？有没有违规激励？
- [ ] **边界适用性**：规则对所有数据形态 / 并发数 / 阶段适用吗？
- [ ] **YAGNI**：当前阶段真需要这条吗？还是 Phase N 真遇到问题再加？

## Breaking Changes

<!-- 有破坏性变更必填 · 无勾选 "N/A" -->

- [ ] N/A
- [ ] 有（描述 · 迁移指南 · migration script 路径）

## Screenshots / Artifacts（可选）

<!-- UI 改动贴截图 · spike 产出贴 benchmark 表 / 录屏链接 · gitleaks 扫描输出截图（SPIKE-06 A.5.3 硬要求） -->

## Related

<!-- 相关 PR / ADR / session-history / 决策表 # -->

- CLAUDE.md 决策表：#
- ADR: `docs/adr/ADR-NNN.md`
- 前置 PR: #
