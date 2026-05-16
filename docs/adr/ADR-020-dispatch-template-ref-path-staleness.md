# ADR-020: dispatch-prompt-template.md §4 参考实现引用路径 stale（指 top-level 实在 \_archived/）

**状态**：proposed
**日期**：2026-05-16（proposed）
**决策者**：Grok（dispatch 起草 · self-review v2-D.2 单人项目）· 主 agent 后续独立 review
**对应 `CLAUDE.md` 决策表**：—（治理规则 · 本 ADR 记录 dispatch 模板路径引用漂移）
**前置事件**：PR #329（dispatch-prompt-template.md 压缩重构 · 883→597 行 · 审计附录拆分到 docs/dispatch-incidents.md）· spike-tmp/dispatch/ 目录 cleanup（top-level 清空 · 示例 prompt 移入 \_archived/ 子目录保留）

---

## 背景与问题（Context）

`.claude/rules/dispatch-prompt-template.md` 第 419 行声明：

> dispatch prompt 文件统一放 `spike-tmp/dispatch/` · 命名格式：`<TASK-ID>[-<phase-or-pr-suffix>]-<agent>-prompt.md`

§4「参考实现 · 选择指南」（当前 origin/main 实际 §4 标题）内表格推荐 4 类范本：

- `MVP-04-storage-prep-opencode-prompt.md`
- `MVP-07-kimi-prompt.md`
- `SPIKE-06-pr2-codex-prompt.md`
- `MVP-02-opencode-prompt.md`

并在 "推荐参考全列表" 链接到 `docs/dispatch-incidents.md §4`。

**实测现实**：`spike-tmp/dispatch/` 顶层目录已被 cleanup 清空 · 这些示例 prompt 实际存放在 `spike-tmp/dispatch/_archived/` 子目录下（top-level 空 · 仅 \_archived/ 保留历史）。§4 及第 419 行的路径声明已**断链** —— 未来 agent 按文档所述路径无法找到范本文件。

注：#329 压缩后 §4 标题已从原 "4 · 参考实现 · 选择指南" 演进为当前形式 · 路径引用问题遗留未修。

## 决策（Decision · proposed · Arbiter 拍板后生效）

§2.1 要求：本 ADR 仅记录事实 + 提议选项 · **status 只能 proposed** · 不得自 accept · 不改 dispatch-prompt-template.md（accept 后另 PR 执行）· 最终由 Arbiter 裁决：

- **(a)【推荐】** 更新 §4（及第 419 行 "统一放" 声明）路径为 `spike-tmp/dispatch/_archived/<name>`（最小改 · 准确反映 cleanup 后实际归档位置 · 推荐模板表中的 basename 引用保持或加 `_archived/` 前缀说明）
- **(b)** 把 8 范本移回 top-level `spike-tmp/dispatch/`（破坏 cleanup 约定 · 增加 repo 根级噪声 · 不推荐）
- **(c)** §4 路径引用改为泛指 "见 `spike-tmp/dispatch/_archived/` 历史归档目录"（不逐一列具体文件名 · 由 incidents.md 维护清单）

无论哪条：**.claude/rules/dispatch-prompt-template.md 实际改动 —— 均由 Arbiter 拍板明确后在独立 PR 执行 · 本 PR 仅 proposed draft**。

## 约束（Constraints）

- 本 ADR **仅记录+提议** · 不改 `.claude/rules/dispatch-prompt-template.md` / `docs/dispatch-incidents.md` / 任何决策文件（Arbiter accept 后另 PR 改）
- status **proposed** · 需 Arbiter 拍板 → accepted 后方生效（v2-D.2 单人项目 self-review + Arbiter approval 流程）
- 路径现状严格以 `git show origin/main:.claude/rules/dispatch-prompt-template.md | grep -n '参考实现\|spike-tmp/dispatch'` 实测为准 · 未臆断 §4 位置
- 不得声称 "Arbiter 已同意 X"

## 后果（Consequences）

**正面**：

- 修复 dispatch 模板引用断链 · 未来 agent（尤其是新 agent 首任务）能按 §4 推荐快速定位历史范本
- 明确 `spike-tmp/dispatch/_archived/` 是 cleanup 后的规范归档位置 · 与 spike-delivery-checklist 等规则保持一致
- 最小改动 (a) 成本最低 · 保持现有 basename 推荐表不变，仅修正目录前缀

**负面 / 风险**：

- 若选择 (b) 回迁：违反 "archive 保留历史 · top-level 保持干净" 的 cleanup 意图 · 增加维护负担
- 任何选项均需后续 PR 实际改 dispatch-prompt-template.md（本 PR 不执行）
- incidents.md §4 历史记录可能需同步更新（非本 PR 范围）

---

## Arbiter 拍板栏（待 tajiaoyezi · v2-D.2 单人项目 self-review + Arbiter approval · 留空待拍板）

- [ ] 事实准确性：origin/main dispatch-prompt-template.md 第 419 行 "统一放 `spike-tmp/dispatch/`" + §4「参考实现 · 选择指南」表格已 git show 确认 · top-level 目录实际为空（\_archived/ 存在）已 ls/find 验证
- [ ] 选项完整：(a)(b)(c) 三条均已列出 · 推荐 (a) 最小改理由已陈述
- [ ] 约束遵守：**status=proposed** · 未碰 dispatch-prompt-template.md · 未改现有 ADR · git diff --stat 仅 3 文件 · §4 现状以 origin/main grep 为准
- [ ] 选定方案：（留空 · 由 Arbiter 在 review comment 或后续 PR 明确 (a)/(b)/(c)）

**proposed 决议**（本 PR 仅 draft · accept 待 Arbiter 单独 flip）：

1. 记录事实：dispatch-prompt-template.md §4 及路径声明引用 top-level `spike-tmp/dispatch/` 已与实际 \_archived/ 归档现实 drift（PR #329 压缩后遗留）
2. 推荐 (a)：更新路径引用为 `_archived/` 前缀 · 最小侵入 · 恢复文档可执行性
3. 后续动作：Arbiter 择一选项后 · 另开 PR 改 `.claude/rules/dispatch-prompt-template.md`（必要时同步 incidents.md §4）· 本 ADR → accepted

---

**实测坐实**（Grok dispatch · 2026-05-16）：

- dispatch-prompt-template.md 路径声明（第 419 行附近）：`spike-tmp/dispatch/`（git show origin/main 确认）
- §4 标题：`## 4 · 参考实现 · 选择指南`（git show origin/main 确认）
- 实际 FS：spike-tmp/dispatch/ 顶层不存在或为空 · \_archived/ 子目录保留历史 prompt（find / ls 验证）
- 推荐模板表：4 个 basename 引用（MVP-04-... 等）存在于 §4
