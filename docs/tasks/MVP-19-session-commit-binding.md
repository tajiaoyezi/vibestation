---
id: MVP-19
type: mvp
title: AI session ↔ commit 自动绑定
status: in-progress
owner: Claude Code
phase: v1.0
depends_on: ["MVP-18"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1 · §5.3.6 · §1.1
risk_ref: R1
reviewer: Cursor · self-review
---

# MVP-19: AI session ↔ commit 自动绑定

> **状态**：`in-progress`（owner: Claude Code · 2026-05-17 认领实施 · **Arbiter waive MVP-18-done gate**：tajiaoyezi 2026-05-17 "waive gate, 全速 MVP-19" —— 显式 waive 本 spec 原 "实施仍 gated on MVP-18 done"。依据：MVP-19 真实上游依赖 = MVP-18 pane 联动**能力**（Wave 1+2 merged + escalate #2 闭合 · 功能性完成）· Phase D/E 是验证产物非功能前置。每 MVP-19 PR body Arbiter approval trailer 引此 waive · 多 phase 任务 status 至 Phase E 收尾 gate 才翻 done · W1-A.0/A.1/B + W2-A.0 merged（#365-#369）· W2-B/C/D 四路并行 in flight（#370 + parallel Codex/Cursor/Droid））· **v1.0 vision** · README / landing 完全不宣传
> **依赖**：MVP-18（pane 联动能力上游）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v1.0`](../implementation-plan.md) · [`§5.3.6`](../implementation-plan.md)
> **详化时间**：2026-05-14 session 31 · Cursor self-review（单人项目 v2-D.2）
> **对外叙事边界**：内部 docs/tasks 可讨论能力细节；公开文案仅允许脱敏代号 `v1.0 vision feature X`。

---

## 占位语义保留（原始骨架）

把一次完整的 AI 对话（Claude / Codex CLI session）识别为一个**逻辑工作单元**，并自动关联到它期间产生的 commits，使得：

- 一键看"这个 commit 是 AI 哪段对话产出的"
- 一键看"这个 session 一共改了哪些文件 / 产生了哪些 commit"

占位背景要点保留如下（后续各节展开）：

- v1.0 vision 把 session 视为 AI 工作单元
- 上游依赖 MVP-18 和 SPIKE-07
- 本能力只在内部文档讨论，不进入公开宣发
- 关联与反查要可撤销、可审计

占位范围要点保留如下（后续各节展开）：

- Session 边界识别（新进程、显式清空、手动标记、空闲阈值）
- Session 元数据存储
- Commit 自动关联与置信度记录
- Git Log 徽章与 Session 详情反查
- 用户手动解绑
- v1.0 不做跨 workspace 聚合和语义自动分类

---

## §A. 目标（Goal）

MVP-19 的目标是在 workspace 维度内建立一条可追溯链路：从 AI CLI 对话 session 到 Git commit，再从 commit 反查 session。用户在真实开发中经常遇到以下问题：今天修了一个 bug，但忘了哪次对话给出的 patch 最终被 commit；或者 AI 在 30 分钟内连续修改 5 个文件并生成多次提交，用户想快速确认每个提交是否属于同一思路；再或者 code review 里看到一个可疑 commit，希望一键回放对应对话摘要评估风险。MVP-19 通过自动绑定 + 手动纠错，让这条链路可用、可解释、可撤销。

该目标并不要求“自动理解对话语义”或“替用户做质量判断”。本 task 只解决 provenance（来源追踪）问题：哪些 commit 来自哪个 session，关联是否可靠，关联出错如何修正。通过把 session 作为第一等对象，后续 MVP-20 的 session 级回滚才有稳定输入，不需要依赖模糊的时间猜测或人工记忆。

## §B. 背景（Context）

`implementation-plan.md §1.1` 与 `§5.3.6` 明确了 v1.0 vision 路径：在保持对外叙事克制的前提下，内部能力逐步补齐“session 感知”。MVP-19 的定位处于中间层：上接 MVP-18 的 pane 与失败上下文联动，下接 MVP-20 的安全回滚。若没有稳定的 session↔commit 绑定，MVP-20 只能退回“按时间窗口猜 commit”的低置信方案，风险高且不可审计。

R1（CLI 协议解析可行性）由 SPIKE-07 管理，本 spec 详化不替代 spike 结论。这里必须强调两点边界：

- 本文可以定义识别策略、fallback、置信度和误差处理，但不能伪造 “SPIKE-07 已证明某阈值”。
- 实施阶段若 SPIKE-07 未给出可执行 parser 合同，MVP-19 只能使用保守模式（有限自动识别 + 强制人工确认），并在风险与决策表保留 pending 行。

与数据持久化相关的实现约束沿用 SPIKE-04 B.3（rusqlite migration safety）模式：新增表必须是 additive migration、事务内完成、失败可回滚、版本推进可重复。因为 session 与 commit 的关联是审计数据，任何 silent corruption 都会直接损害用户对“回滚依据”的信任。

与 MVP-18 的关系：

- MVP-18 给出可订阅的 pane 事件流和基础 session 语义触发点。具体订阅哪些 MVP-18 event/binding（如 `pane:trigger` / `pane:build-failed` / `pane:linked`）映射为 session 边界，在实施期 Phase A 对接时按 MVP-18 §K 实际 contract 固化（**已知接口对齐点 · 非 spec 详化阻塞** · session 32 ready-gate 预审记录）。
- MVP-19 把这些事件抽象成 session 生命周期（start/end/idle/clear/manual split）并持久化。
- 若 MVP-18 后续字段调整，MVP-19 通过 ts-rs binding 同步，不允许前后端平行定义。

与 SPIKE-07 的关系：

- 识别精度门槛的根依据来自 SPIKE-07 的 parser 证据，不在本 spec 伪造。
- §H 决策表保留“留 SPIKE-07 决策”条目，表示该输入仍是外部 gate。
- 若 SPIKE-07 最终结论为单 CLI 可行，MVP-19 范围需要收敛为单 CLI 支持并写入 implementation phase 入口条件。

## §C. 功能范围（Scope）

### C.1 Do（必须做）

1. 识别 session 边界：新进程启动、显式 `/clear`、手动“开始新 session”、空闲阈值触发软结束。
2. 记录 session 元数据：`id`、`workspace_id`、`cli_kind`、`started_at`、`ended_at`、`source`、`title`、`prompt_count`、`token_count?`。
3. 自动尝试 commit 绑定：在 commit 创建后依据时间窗口 + pane/source + 命令上下文计算候选 session。
4. 记录绑定置信度：`confidence`、`confidence_reason`、`strategy_version`。
5. 允许人工确认：低置信候选显示“待确认”，用户可确认或忽略。
6. Git Log 显示 session 徽章：commit 行可见 session 简要标签。
7. 支持从 commit 反查 session：点击徽章进入 Session 详情视图。
8. 支持从 session 看 commits：Session 详情列出 commit 列表、文件统计、时间轴。
9. 支持手动解绑：对错误关联执行 unlink，保留审计痕迹。
10. 支持手动重绑：将某 commit 指派到另一个 session（仅同 workspace）。
11. 支持 stale 处理：session 删除或损坏时 link 行状态可见，不 silent drop。
12. 支持只读审计字段：谁改了绑定、何时改、改动前后值。
13. 支持脱敏摘要：详情页可查看对话摘要，但必须先过脱敏管线。
14. 支持最小化回填：旧 commit 可在用户触发时尝试回填关联，但默认不全仓扫描。
15. 支持导出调试快照：仅本地文件，包含 link 判断依据，不含明文 secrets。

### C.2 Don't（明确不做）

1. 不做跨 workspace 聚合 session 视图。
2. 不做“自动语义分类”如 feature/fix/refactor。
3. 不做“自动改绑”覆盖用户确认结果。
4. 不做第三方未知 CLI 全量支持（按 SPIKE-07 scope 收敛）。
5. 不把原始长日志全文落库（只存摘要与 hash）。
6. 不做云端同步或遥测上传。
7. 不在公开文案披露具体能力细节。
8. 不将解绑操作设计成硬删除不可追踪。

### C.3 关键子能力映射

- Session 边界识别：`session:start` / `session:end` / idle cutoff / manual split。
- Commit 关联：`session:bind-commit` 自动路径 + 人工路径。
- 反查：commit badge -> session timeline。
- 解绑：`session:unbind` + 审计 trail。

---

## §D. UI wireframe（文字描述）

### D.1 Session 详情视图（核心一）

入口：

- 从左侧 Session 列表进入。
- 从 Git Log commit 徽章点击跳转进入（带 anchor 到对应 commit）。

主要区域：

- Header：session 标题、CLI 类型、起止时间、状态（active/ended/idle-cutoff/manual-ended）。
- Summary strip：commit 数量、文件数量、绑定置信均值、最近一次解绑操作。
- Timeline：按时间展示用户输入摘要、assistant 摘要、绑定事件、解绑事件。
- Commit panel：该 session 关联 commit 列表，支持跳转 Git Log 定位。
- Actions：`手动结束`、`批量重算候选`、`导出调试快照`。

状态说明：

- active：仍在写入事件。
- ended：显式结束或新 session 切分结束。
- idle-cutoff：空闲阈值触发结束。
- orphaned-link：部分 commit link 指向损坏 session（只读告警）。

### D.2 Git Log session 徽章（核心二）

视觉行为：

- 每条 commit 右侧最多显示 1 个主徽章（主 session）。
- 如存在多候选未确认，显示 `+N` 次级标记。
- 置信度低于阈值时徽章使用弱化样式并有 tooltip。

交互行为：

- hover 展示 session 标题、时间、置信度、来源策略。
- click 打开 Session 详情并定位到当前 commit。
- 右键上下文菜单：`解绑`、`改绑到...`、`查看判断依据`。

异常行为：

- 若 session 缺失，徽章显示 `stale` 状态，点击进入故障说明。
- 若 commit 尚未判定，显示 `pending` 小点，后台异步完成后更新。

### D.3 解绑确认 modal（核心三）

触发场景：

- 用户在 Git Log 徽章菜单点击“解绑”。
- 用户在 Session 详情 commit 行点击“解绑”。

modal 内容：

- 标题：`确认解绑 commit 与 session`
- 主体：显示 commit short sha、session 标题、当前绑定策略、置信度。
- 风险提示：解绑会影响后续 session 级回滚候选集。
- 审计输入：可选填写 reason（默认“manual correction”）。

按钮：

- `Cancel`：关闭，不改动。
- `Unbind`：执行解绑并记录审计字段。
- `Unbind and recalc`：解绑后立即触发候选重算（可选）。

### D.4 可访问性说明

- 所有徽章和按钮都有 aria-label。
- modal 打开后 focus trap，Esc 关闭，Enter 触发默认主按钮。
- Session 详情时间线支持键盘逐项导航和屏幕阅读器摘要。

## §E. Acceptance（28+ checklist）

### E.1 Frontmatter 与范围守卫

- [ ] E1.1 `status` 保持 `draft`，本 PR 不翻 `ready`。
- [ ] E1.2 `reviewer` 填写 `Cursor · self-review`。
- [ ] E1.3 其他 frontmatter 字段保持原语义不漂移。
- [ ] E1.4 本文件不引入跨 task 的状态修改描述。

### E.2 Session 边界识别

- [ ] E2.1 新 CLI 进程启动触发 `session:start`。
- [ ] E2.2 显式 `/clear` 触发旧 session `session:end` + 新 session `session:start`。
- [ ] E2.3 手动“开始新 session”操作稳定可用。
- [ ] E2.4 空闲阈值触发 soft end，并带 `end_reason=idle_cutoff`。
- [ ] E2.5 边界识别准确率目标以 SPIKE-07 fixture 为准，阈值记录为 `>= 90%`（未得最终证据前标记 provisional）。
- [ ] E2.6 连续 3 次边界触发不出现重叠 active session。

### E.3 Commit 关联正确率

- [ ] E3.1 commit 创建后自动进入候选匹配流程。
- [ ] E3.2 关联算法至少包含时间窗口 + 来源上下文两路信号。
- [ ] E3.3 自动关联正确率目标 `>= 95%`（以实现期基准测试报告为准）。
- [ ] E3.4 低置信关联必须落 `pending`，禁止默默当作 confirmed。
- [ ] E3.5 用户手动确认后 link 状态更新为 `confirmed_manual`。
- [ ] E3.6 同一 commit 同时只能有 1 条 `is_primary=1` link。

### E.4 反查与 UI

- [ ] E4.1 Git Log commit 行显示 session 徽章。
- [ ] E4.2 徽章点击可进入 Session 详情并定位 commit。
- [ ] E4.3 Session 详情可列出关联 commit 列表与基础统计。
- [ ] E4.4 stale / pending / low-confidence 状态均有可见标记。
- [ ] E4.5 详情页对话摘要已过脱敏管线。

### E.5 解绑与改绑

- [ ] E5.1 解绑必须弹确认 modal。
- [ ] E5.2 解绑后保留审计记录（操作者、时间、原因）。
- [ ] E5.3 解绑不会删除 commit 或 session 主体数据。
- [ ] E5.4 支持改绑到同 workspace 其他 session。
- [ ] E5.5 改绑后旧 link 状态改为 superseded。

### E.6 数据与迁移安全

- [ ] E6.1 新表 migration 采用事务 + `PRAGMA user_version` 递增。
- [ ] E6.2 migration 失败时回滚，不破坏已有表。
- [ ] E6.3 `ai_sessions` 与 `session_commit_links` 建立必要索引。
- [ ] E6.4 旧版本数据库升级后能正常读取无 link 场景。
- [ ] E6.5 migration 安全模式参考 SPIKE-04 B.3 一致实践。

### E.7 脱敏与安全

- [ ] E7.1 session 摘要中命中 token/PII 模式时必须 redaction。
- [ ] E7.2 `gitleaks` 规则集可复用于摘要脱敏检测。
- [ ] E7.3 原始日志不全文持久化，仅保存限长摘要和 hash。
- [ ] E7.4 脱敏失败时详情页显示受限提示，不回退到明文。

### E.8 a11y 与可用性

- [ ] E8.1 徽章、按钮、modal 均可键盘操作。
- [ ] E8.2 屏幕阅读器能读出徽章状态与置信信息。
- [ ] E8.3 颜色不是唯一状态表达方式。
- [ ] E8.4 reduced-motion 环境下过渡动画降级。

### E.9 质量门槛

- [ ] E9.1 本 spec 行数保持 550-750。
- [ ] E9.2 无明显 typo 和占位空语句。
- [ ] E9.3 不出现公开宣发语气的能力描述。
- [ ] E9.4 §N 自审四问为明确回答而非套话。

## §F. 测试矩阵

| 层次         | 范围                 | 关键输入                            | 命令/入口                                                               | 覆盖路径 |
| ------------ | -------------------- | ----------------------------------- | ----------------------------------------------------------------------- | -------- |
| 单元（core） | session 边界识别函数 | 合成 CLI 事件流                     | `cargo test -p vibestation-core session_boundary::`                     | E2       |
| 单元（core） | 关联评分算法         | 时间窗口/上下文样本                 | `cargo test -p vibestation-core session_link_scoring::`                 | E3       |
| 单元（core） | 脱敏函数             | token/PII/路径样本                  | `cargo test -p vibestation-core session_redaction::`                    | E7       |
| 集成（app）  | IPC + DB roundtrip   | `session:start` -> `bind` -> `list` | `cargo test -p vibestation-app --features integration session_commit::` | E2-E6    |
| 集成（app）  | migration 升级       | 旧 schema fixture DB                | 同上                                                                    | E6       |
| 前端单测     | Git Log 徽章状态渲染 | confirmed/pending/stale mock        | `pnpm -C web exec vitest run tests/git-log/session-badge`               | E4/E8    |
| 前端单测     | 解绑 modal 交互      | keyboard + confirm/cancel           | `pnpm -C web exec vitest run tests/session/unbind-modal`                | E5/E8    |
| E2E          | 端到端绑定与反查     | session + 3 commits                 | `pnpm -C web exec playwright test session-commit-binding.spec.ts`       | E2-E5    |
| E2E          | 脱敏展示             | 含敏感片段对话摘要                  | 同上                                                                    | E7       |
| 手动 QA      | 多平台 lifecycle     | macOS/Linux CLI 进程差异            | `pnpm tauri:dev`                                                        | §L       |

### F.1 单元测试最小样本集合

- `fixture_session_start_by_process_spawn`
- `fixture_session_split_by_clear`
- `fixture_session_end_by_idle_cutoff`
- `fixture_commit_bind_high_confidence`
- `fixture_commit_bind_low_confidence_pending`
- `fixture_unbind_and_rebind`
- `fixture_redaction_token`
- `fixture_redaction_email_phone`

### F.2 集成测试关键断言

1. `session:start` 后 `ai_sessions` 新增一行 active 记录。
2. commit 事件到来后 `session_commit_links` 新增候选记录。
3. 关联成功时 `is_primary` 唯一约束成立。
4. 解绑后 link 状态不为 hard delete。
5. migration 从旧 schema 升级后可继续新增 link。

### F.3 E2E 用例骨架

1. 启动 CLI session，触发 2 次 prompt。
2. 执行 3 次 commit。
3. 在 Git Log 验证 3 条 commit 有 badge。
4. 点击任一 badge 进入 session 详情，检查 commit 列表。
5. 对其中 1 条执行解绑，验证 badge 状态刷新与审计记录存在。

### F.4 性能验收建议

- 绑定计算单次目标 < 20ms（本地 500 commit 样本）。
- Git Log 列表渲染徽章不引入全表重绘。
- Session 详情首次打开目标 < 200ms（缓存命中 < 80ms）。

## §G. 数据模型（ai_sessions + session_commit_links）

### G.1 目标

新增两个核心实体：

- `ai_sessions`：记录 session 生命周期与元信息。
- `session_commit_links`：记录 commit 与 session 的关联关系及审计字段。

### G.2 `ai_sessions` schema

```sql
CREATE TABLE ai_sessions (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  cli_kind TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT 'auto',
  title TEXT NOT NULL DEFAULT '',
  started_at INTEGER NOT NULL,
  ended_at INTEGER,
  end_reason TEXT,
  prompt_count INTEGER NOT NULL DEFAULT 0,
  token_count INTEGER,
  event_count INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'active',
  parser_version TEXT,
  strategy_version TEXT,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE INDEX idx_ai_sessions_workspace_started
  ON ai_sessions(workspace_id, started_at DESC);

CREATE INDEX idx_ai_sessions_workspace_status
  ON ai_sessions(workspace_id, status, ended_at);
```

字段语义：

- `source`：`auto | manual`，标识 session 创建来源。
- `status`：`active | ended | idle_cutoff | archived`。
- `parser_version`：用于关联 SPIKE-07 结论对应的 parser 版本。
- `metadata_json`：保留扩展字段，严禁写入明文敏感信息。

### G.3 `session_commit_links` schema

```sql
CREATE TABLE session_commit_links (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  commit_sha TEXT NOT NULL,
  is_primary INTEGER NOT NULL DEFAULT 1,
  link_state TEXT NOT NULL DEFAULT 'pending',
  auto_bound INTEGER NOT NULL DEFAULT 1,
  confidence REAL NOT NULL DEFAULT 0.0,
  confidence_reason TEXT NOT NULL DEFAULT '',
  strategy_version TEXT NOT NULL DEFAULT 'v1',
  source_event_id TEXT,
  linked_at INTEGER NOT NULL,
  unlinked_at INTEGER,
  unlinked_reason TEXT,
  superseded_by_link_id TEXT,
  created_by TEXT NOT NULL DEFAULT 'system',
  reviewed_by TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY(session_id) REFERENCES ai_sessions(id)
);

CREATE UNIQUE INDEX ux_session_commit_primary
  ON session_commit_links(workspace_id, commit_sha, is_primary)
  WHERE is_primary = 1 AND unlinked_at IS NULL;

CREATE INDEX idx_session_commit_links_session
  ON session_commit_links(workspace_id, session_id, linked_at DESC);

CREATE INDEX idx_session_commit_links_commit
  ON session_commit_links(workspace_id, commit_sha, linked_at DESC);

CREATE INDEX idx_session_commit_links_state
  ON session_commit_links(workspace_id, link_state, confidence);
```

字段语义：

- `link_state`：`pending | confirmed_auto | confirmed_manual | unlinked | superseded | stale`。
- `auto_bound`：自动推断或人工操作。
- `confidence`：0.0-1.0 置信分。
- `source_event_id`：关联生成绑定的 session 事件。
- `superseded_by_link_id`：改绑后指向新 link。

### G.4 migration 策略

1. 在事务中创建新表和索引。
2. 更新 `PRAGMA user_version`。
3. 若任一步骤失败，回滚并保持旧 schema 可用。
4. 再次执行 migration 不应重复创建失败。
5. migration 脚本需带 smoke 测试（空库、已有数据库、损坏库场景）。

### G.5 数据保留与清理策略

- 默认保留 session 元数据和 link 元数据。
- 摘要文本走独立存储（若后续引入），需 TTL 与大小上限。
- 提供“清理历史 session 索引”工具时，只能 archive，不做 silent hard delete。

### G.6 与 MVP-13 / MVP-09 模式对齐

- 复用 rusqlite migration carrier。
- 错误类型采用 tagged enum。
- 前端消费类型必须来自 ts-rs 生成 binding。

## §H. 决策表

| 决策项                                    | 状态                                            | Owner                 | 理由                                        | 备注                                                                   |
| ----------------------------------------- | ----------------------------------------------- | --------------------- | ------------------------------------------- | ---------------------------------------------------------------------- |
| H1. 本 spec 可先于实现详化                | Accepted（spec 详化范围内 · 不含 runtime 结论） | Cursor self-review    | 详化不等于实现，不需要提前给出 runtime 结论 | session 32 Arbiter approve flip `ready`（实施仍 gated on MVP-18 done） |
| H2. MVP-19 实施需等待 SPIKE-07 可执行输入 | Pending                                         | OpenCode + Arbiter    | R1 仍是上游 gate                            | **留 SPIKE-07 决策**                                                   |
| H3. 绑定算法采用多信号评分而非单时间窗口  | Accepted                                        | MVP-19 implementer    | 纯时间窗口误判率高                          | 评分细则在实现阶段固化                                                 |
| H4. 低置信关联默认 pending                | Accepted                                        | MVP-19 implementer    | 防止 silent wrong binding                   | 用户可手动确认                                                         |
| H5. 解绑不硬删除，必须保留审计            | Accepted                                        | MVP-19 implementer    | 回滚与审计依赖历史                          | link_state 流转                                                        |
| H6. public narrative 仅用脱敏代号         | Accepted                                        | All agents            | 对外边界约束                                | 内部 docs 可详述                                                       |
| H7. idle cutoff 默认值先给保守值          | Pending tune                                    | Arbiter + implementer | 需真实 usage 校准                           | **留 v1.0 启动后 idle 阈值微调**                                       |
| H8. 单 commit 主关联唯一                  | Accepted                                        | MVP-19 implementer    | 避免 UI 混乱与回滚歧义                      | `is_primary` 唯一索引                                                  |
| H9. 脱敏失败时禁止回落明文                | Accepted                                        | Security gate         | 防泄漏优先于可见性                          | 显示 redacted warning                                                  |
| H10. 不做跨 workspace 聚合                | Accepted                                        | Product scope         | 降低复杂度与泄漏面                          | v2+ 再评估                                                             |

## §I. 实施 Phase 拆分（8d）

| Phase                   | 估时 | 范围                                                     | 核心产物                | 退出条件                   |
| ----------------------- | ---: | -------------------------------------------------------- | ----------------------- | -------------------------- |
| A. backend session 识别 |   2d | session lifecycle 引擎、边界识别函数、`ai_sessions` CRUD | core + app backend      | unit/integration 通过      |
| B. IPC + storage        |   2d | `session:*` commands/events、migration、ts-rs binding    | schema + IPC contract   | migration + IPC smoke 通过 |
| C. Git Log 徽章 UI      | 1.5d | badge 渲染、pending/stale 状态、点击跳转                 | frontend Git Log update | vitest + E2E 子集通过      |
| D. 反查视图             | 1.5d | Session 详情页、commit 列表、解绑/改绑交互               | session detail UI       | E2E 路径通过               |
| E. runtime & hardening  |   1d | 脱敏、性能、a11y、跨平台 smoke                           | runtime evidence        | checklists 全绿            |

### I.1 Phase A 细分任务

1. 新增 `SessionBoundaryDetector`。
2. 新增 `SessionLifecycleService`。
3. 新增 idle cutoff 计时器接口（可配置）。
4. 编写边界识别 fixture。
5. 对接 pane/source 事件流。

### I.2 Phase B 细分任务

1. 写 migration `ai_sessions` + `session_commit_links`。
2. 实现 `session:start` / `session:end`。
3. 实现 `session:bind-commit` / `session:unbind`。
4. 实现 `session:list` / `session:get-detail`。
5. 完成 ts-rs export 与前端 binding 更新。

### I.3 Phase C 细分任务

1. Git Log 列表注入徽章组件。
2. 徽章 tooltip 与状态色实现。
3. click -> detail route 跳转。
4. stale/pending 样式与文案。
5. 键盘可访问性与 aria。

### I.4 Phase D 细分任务

1. Session 详情页布局与基础数据。
2. commit 列表与时间线。
3. 解绑 modal 与审计 reason。
4. 改绑动作与 superseded 流转。
5. 错误态（session missing/link stale）处理。

### I.5 Phase E 细分任务

1. 脱敏策略接入与红线测试。
2. 性能 profiling（徽章渲染与详情查询）。
3. a11y 审核（键盘、屏幕阅读器、reduced motion）。
4. macOS/Linux lifecycle smoke。
5. runtime evidence 打包归档。

## §J. 风险表（R1-R5）

| 风险ID | 描述                            | 概率  | 影响  | 缓解措施                                                       | 验证方式              |
| ------ | ------------------------------- | ----- | ----- | -------------------------------------------------------------- | --------------------- |
| R1     | parser/边界识别不稳定导致误绑定 | 中-高 | 高    | SPIKE-07 gate + low-confidence pending + manual confirm        | fixture + E2E         |
| R2     | 解绑误操作导致回滚依据受损      | 中    | 中-高 | 强制 modal + 审计日志 + 可改绑恢复                             | E2E + manual QA       |
| R3     | 数据迁移失败或索引冲突          | 中    | 高    | additive migration + transaction + rollback test               | migration integration |
| R4     | 摘要脱敏遗漏 secrets/PII        | 中    | 高    | 统一 redaction pipeline + gitleaks pattern reuse + fail closed | security tests        |
| R5     | Git Log 渲染性能下降            | 中    | 中    | memoized selector + lazy tooltip + virtual list 兼容           | perf bench            |

### J.1 风险触发信号

- R1 触发：pending 比例长期 > 30%，人工确认负担过高。
- R2 触发：解绑后投诉“找不到原关联”。
- R3 触发：升级后出现 `no such table` 或唯一索引冲突。
- R4 触发：redaction 日志命中但 UI 仍显示可疑明文。
- R5 触发：Git Log FPS 在 1k commits 场景明显下降。

### J.2 风险升级策略

1. 触发 R1/R4 时暂停自动确认，只允许 manual confirm。
2. 触发 R3 时阻止写路径，进入只读保护模式。
3. 触发 R5 时关闭徽章细节渲染，仅保留简化标记。

## §K. IPC contract（含 ts-rs binding）

### K.1 命令列表（必须）

| Command                          | Request                    | Response                  | 说明                        |
| -------------------------------- | -------------------------- | ------------------------- | --------------------------- |
| `session:start`                  | `SessionStartRequest`      | `SessionStartResult`      | 显式或自动开启 session      |
| `session:end`                    | `SessionEndRequest`        | `SessionEndResult`        | 结束 session，写 end_reason |
| `session:bind-commit`            | `SessionBindCommitRequest` | `SessionBindCommitResult` | 自动/手动绑定 commit        |
| `session:unbind`                 | `SessionUnbindRequest`     | `SessionUnbindResult`     | 解绑并审计                  |
| `session:list`                   | `SessionListRequest`       | `SessionListResult`       | 按 workspace 列 session     |
| `session:get-detail`             | `SessionDetailRequest`     | `SessionDetailResult`     | 详情页数据                  |
| `session:rebind`                 | `SessionRebindRequest`     | `SessionRebindResult`     | 改绑 commit                 |
| `session:recalculate-candidates` | `SessionRecalcRequest`     | `SessionRecalcResult`     | 重算候选                    |

### K.2 事件列表（建议）

| Event                    | Payload                     | 说明                          |
| ------------------------ | --------------------------- | ----------------------------- |
| `session:started`        | `SessionStartedEvent`       | session 启动通知              |
| `session:ended`          | `SessionEndedEvent`         | session 结束通知              |
| `session:commit-bound`   | `SessionCommitBoundEvent`   | commit 完成绑定               |
| `session:commit-unbound` | `SessionCommitUnboundEvent` | commit 完成解绑               |
| `session:link-updated`   | `SessionLinkUpdatedEvent`   | pending->confirmed 等状态变更 |
| `session:error`          | `SessionErrorEvent`         | 可恢复错误                    |

### K.3 ts-rs binding 列表（最小）

1. `AiSession.ts`
2. `AiSessionStatus.ts`
3. `AiSessionEndReason.ts`
4. `SessionStartRequest.ts`
5. `SessionStartResult.ts`
6. `SessionEndRequest.ts`
7. `SessionEndResult.ts`
8. `SessionCommitLink.ts`
9. `SessionCommitLinkState.ts`
10. `SessionBindCommitRequest.ts`
11. `SessionBindCommitResult.ts`
12. `SessionUnbindRequest.ts`
13. `SessionUnbindResult.ts`
14. `SessionRebindRequest.ts`
15. `SessionRebindResult.ts`
16. `SessionListRequest.ts`
17. `SessionListResult.ts`
18. `SessionDetailRequest.ts`
19. `SessionDetailResult.ts`
20. `SessionRecalcRequest.ts`
21. `SessionRecalcResult.ts`
22. `SessionError.ts`
23. `SessionStartedEvent.ts`
24. `SessionEndedEvent.ts`
25. `SessionCommitBoundEvent.ts`
26. `SessionCommitUnboundEvent.ts`
27. `SessionLinkUpdatedEvent.ts`
28. `SessionErrorEvent.ts`
29. `SessionConfidenceBreakdown.ts`
30. `SessionAuditEntry.ts`

### K.4 Rust payload 草图

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionBindCommitRequest {
    pub workspace_id: String,
    pub commit_sha: String,
    pub session_id: Option<String>,
    pub mode: SessionBindMode,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SessionCommitLink {
    pub id: String,
    pub workspace_id: String,
    pub session_id: String,
    pub commit_sha: String,
    pub is_primary: bool,
    pub link_state: SessionCommitLinkState,
    #[ts(type = "number")]
    pub confidence: f32,
    pub confidence_reason: String,
    pub auto_bound: bool,
    #[ts(type = "number")]
    pub linked_at: i64,
}
```

### K.5 错误 contract

`SessionError` 变体建议：

- `WorkspaceMismatch`
- `SessionNotFound`
- `CommitNotFound`
- `LinkNotFound`
- `PrimaryLinkConflict`
- `MigrationFailure`
- `RedactionFailed`
- `ParserUnavailable`
- `InvalidStateTransition`
- `DbError`

## §L. 跨平台考量（macOS / Linux）

### L.1 共同原则

- session 识别依赖事件语义，不依赖 shell 文本细节。
- commit 绑定逻辑依赖时间戳与上下文，不依赖 OS path 分隔符。
- 脱敏规则需兼容不同平台路径格式。

### L.2 macOS 重点

1. 常见 shell 为 zsh，prompt 清屏行为与 Linux 略有差异。
2. App nap 可能影响 idle 计时，需要前后台切换补偿。
3. Retina 下徽章渲染需检查像素清晰度。
4. 本地开发时快捷键冲突（Cmd 组合）需回归。

### L.3 Linux 重点

1. Bash/zsh/fish 混用可能改变 prompt pattern。
2. Wayland/X11 的窗口焦点事件差异可能影响“手动分段”快捷键。
3. 路径大小写与权限错误提示更常见。
4. 长时间后台进程更普遍，idle cutoff 要避免误结束。

### L.4 CLI 进程 lifecycle 差异

- 进程重启：macOS 和 Linux 均可通过 pid 变化识别。
- 清屏行为：不能把 terminal clear 误判为 session split。
- 异常退出：崩溃退出应自动 `session:end` 并标记 `end_reason=process_exit`。

## §M. 数据脱敏策略（auth token / PII / gitleaks）

### M.1 目标与原则

- 所有展示到 UI 的 session 摘要必须先脱敏。
- 脱敏失败时 fail closed：宁可隐藏，也不展示可疑明文。
- 脱敏逻辑集中在后端，不让前端各自实现。

### M.2 需要识别的敏感类型

1. API Key（OpenAI、Anthropic、GitHub、自定义 token）。
2. JWT / Bearer token。
3. 邮箱、手机号、身份证样式字符串（按项目策略）。
4. 绝对路径中的用户名目录。
5. 私有仓库 URL 与凭证片段。

### M.3 管线设计

步骤：

1. 输入归一化（去 ANSI、去控制字符）。
2. 基于 regex + gitleaks pattern 扫描命中。
3. 命中后执行 redaction（保留前后少量上下文）。
4. 输出 `sanitized_text` + `redaction_count` + `redaction_kinds`。
5. 若 redaction 引擎异常，返回错误并拒绝展示原文。

### M.4 gitleaks pattern 集成方式

- 复用仓库已有 gitleaks 规则作为 pattern 源。
- 构建 `session_redaction_rules` 时允许附加本地特有 pattern。
- 规则版本记录到 `strategy_version`，便于回溯行为变化。

### M.5 样例（说明性）

输入：
`Authorization: Bearer sk-live-1234567890abcdef`

输出：
`Authorization: Bearer [REDACTED_TOKEN]`

输入：
`/Users/alice/Code/private-repo/.env`

输出：
`/Users/[REDACTED_USER]/Code/private-repo/.env`

### M.6 误报与漏报策略

- 误报：允许用户在本地临时查看原文（仅 debug 模式，默认关闭）。
- 漏报：一旦发现，先升级规则再回归历史摘要重新扫描。

## §N. 自审四问

1. 递归完备性：
   - 本 spec 覆盖了识别、绑定、反查、解绑、改绑、审计、迁移、脱敏、a11y、测试与跨平台，链路闭环完整。
   - 上下游依赖（MVP-18、SPIKE-07、MVP-20）都有清晰接口和 gate。

2. 反向场景：
   - 误绑定、误解绑、migration 失败、脱敏失败、stale link、低置信 pending 都有显式 fallback。
   - 关键风险 R1-R5 都有触发信号与降级动作，不依赖“人工记得处理”。

3. 边界适用性：
   - 仅同 workspace 生效，避免跨项目污染。
   - 支持 macOS/Linux lifecycle 差异，不强依赖单平台行为。
   - 自动与人工路径并存，避免纯自动化误伤。

4. YAGNI：
   - 没有引入跨 workspace 聚合、语义分类、云同步、自动改绑这些超前复杂度。
   - 核心只交付 provenance 链路，为 MVP-20 提供稳定基础。

---

## 附录 O. 术语表

- Session：一次 AI CLI 连续工作单元。
- Link：commit 与 session 的关联记录。
- Primary link：某 commit 的主关联（唯一）。
- Pending：低置信待人工确认关联。
- Superseded：被改绑覆盖的旧关联。
- Idle cutoff：空闲阈值触发的 session 结束。
- Redaction：敏感信息替换。

## 附录 R. 与 MVP-20 的接口约束

MVP-19 实施期必须保证三点：

1. 可按 session 拉取 confirmed commit 列表。
2. 可区分 confirmed 与 pending，回滚默认仅消费 confirmed。
3. 可查询解绑/改绑审计，避免回滚误伤。

完成度结论：spec 已达到 ready-candidate 细化密度；按流程保持 `draft`，等待 Arbiter 与主 agent 后续翻转。
