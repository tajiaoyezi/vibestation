---
id: MVP-20
type: mvp
title: AI 一键回滚（session 级 revert）
status: in-progress
owner: Claude Code
phase: v1.0
depends_on: ["MVP-19", "MVP-16"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 6d
plan_ref: implementation-plan.md §10.1 · §5.3.6 · §1.1
risk_ref: R1
reviewer: Droid · self-review
---

# MVP-20: AI 一键回滚（session 级 revert）

> **状态**：`ready`（**v1.0 vision** · session 32 Arbiter approve flip · README / landing 完全不宣传 · 实施仍 gated on MVP-19 done · 依赖链 MVP-18→19→20 最末端）
> **依赖**：MVP-19（session ↔ commit 绑定 · v1.0 ready）+ MVP-16（rebase / merge / cherry-pick 冲突解决器 · v0.3 ready · 冲突场景直接复用）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) · [`§5.3.6`](../implementation-plan.md) · [`§1.1`](../implementation-plan.md)

---

> ⚠️ **2026-05-20 · capture mandate removed**（ADR-023 supersede ADR-011）：本 spec 中所有 **"Phase E runtime evidence + Criterion 性能量化 / 15 张 GUI 截图 + 30s 录屏 / Linux 跨平台 smoke / §F.1 Criterion bench file 待补" 类 acceptance 项 / Phase 表行** 已 supersede · 不再阻塞 spec done flip。inline 文字保留作 audit 历史 · 但**功能上 deprecated**。代码侧 acceptance（rollback_ops 38 + mvp20_contract 3 + 前端 rollback subset 52 + a11y 5 控件代码就位 + RollbackStatusKind union 保真 + crash recovery 镜像 MVP-16）保留为 done gate。`docs/runtime-evidence/mvp-20/CAPTURE-PLAYBOOK.md` + `PRE-CAPTURE-READINESS.md` + `metrics-mvp-20.md` 由 PR-4 删除。

---

## §A 目标（Goal）

为"AI 辅助编程"工作流提供 **session 级一键安全回滚**：当用户与 AI（Claude CLI / Codex CLI 等）的一轮对话（session）产生了若干 commits，事后发现整体方向不对，可以 **在 UI 中一键触发 `git revert` 序列**，将该 session 的所有 commit 用新的 revert commit 安全撤销，**保留完整历史**（不破坏 `git log` 记录），恢复到 session 开始前的代码状态。

**核心业务场景举例**：

- AI 试错改了 5 个 commit（重构了核心模块），用户看完效果不满意，想一键恢复。不需要手动逐个 `git revert`，不需要 `git reset --hard`（危险）。
- Pair programming session 产生了 8 个 commit，项目方向变了，整个 session 作废，一键 revert 后继续新的方向。
- 快速原型探索：AI 帮忙写了一批探索性代码，探索结束，整个 session 一键清除，主线 branch 保持干净历史。

**设计原则**：**revert = 保留历史的安全撤销**，永远不使用 `git reset --hard` 或任何不可逆操作。

---

## §B 背景（Context）

### B.1 战略地位

`implementation-plan.md §10.1` 将 AI-Aware 相关功能全部砍到 v1.0。本 task 是 v1.0 功能集的**收尾 task**，也是用户最直接感知到"AI 是一等公民"的交互节点——用户不只是看 AI 对话，还能对 AI 产生的 commit 集合做 Git 操作。

`implementation-plan.md §5.3.6`（AI session 操作）+ `§1.1`（AI-Aware Pane v1.0 vision）明确了这一功能作为 session 感知的自然延伸。

**对外宣传禁区**（`CLAUDE.md` #3 · [ADR-009](../adr/ADR-009-ai-aware-v1-vision.md)）：对外 README / landing / CHANGELOG 完全不提及本功能，v1.0 kickoff 内部文档才启用。

### B.2 上游依赖：MVP-19 session 边界识别准确率

MVP-20 的核心前提是 **MVP-19 的 session ↔ commit 关联正确率达标**（MVP-19 `§Acceptance` E3.3 标的 `>= 95%` 为**目标 · 以 MVP-19 实施期 benchmark 报告为准 · 非硬 gate**，见 §H.7）。如果准确率不达标，"一键回滚整个 session"可能误 revert 不该回滚的 commit。

MVP-20 不自行实现 session 识别逻辑——直接消费 MVP-19 的 `ai_sessions` 表和 `session_commit_links` 表，调用 MVP-19 `session:get-detail` IPC（返回 `SessionDetailResult` · 期望含 `commits[].sha` / `confidence` / `link_state` / `auto_bound`；MVP-19 §K 无"按 session 查全部 commit SHA"的专用命令，`session:get-detail` 是接口锚点）查询 session 内所有 commit SHA。

**硬依赖**：MVP-19 `session_commit_links` 表 `auto_bound`（DB `INTEGER` · Rust 侧 `bool` 映射）+ `confidence`（DB `REAL` · Rust 侧 `f32` 映射）+ `link_state` 字段。MVP-20 在执行 revert 前，**仅消费 `link_state ∈ {confirmed_auto, confirmed_manual}` 或（`pending` 且 `confidence ≥ 0.9` 且 `auto_bound = true`）的 commit**，排除 `unlinked` / `superseded` / `stale`；其余低置信度 commit 在预览 diff 中高亮警告（不强制包含）。手动确认绑定的 commit 等价于 `confirmed_manual`。

### B.3 SPIKE-07 的关系

SPIKE-07 正在评估 AI session 边界识别的技术路径（Claude CLI `--session-id` / Codex CLI PID + 时间窗口 / 文件系统 watcher 等方案），其结论直接影响 MVP-19 实现。MVP-20 对 SPIKE-07 的具体技术路径**不预设**——只要 MVP-19 提供了标准 IPC API，MVP-20 就能正常工作。§H 决策表保留"等 SPIKE-07 决策"行。

### B.4 冲突解决器复用：MVP-16

`git revert` 序列中，如果某个 commit 的 revert 与当前 working tree 或后续 revert 产生冲突，**直接复用 MVP-16 的冲突解决 UI**（3-way Diff 视图 + conflict banner + continue / abort / skip 按钮）。

MVP-16 已完整落地（PR #257 / #259 / #266）：

- `crates/core/src/rebase_ops.rs` 含 `Repository::cleanup_state()` 封装
- `web/src/components/ConflictBanner/` 含完整状态机 UI
- `web/src/panels/Diff/3way/` 3-way conflict diff 组件
- IPC `conflict_resolve_file` / `conflict_status` 可复用

MVP-20 **不重新实现冲突解决器**，只实现 `rollback_ops.rs` 后端包装和 session 维度的 UI 状态机，冲突进入后转交 MVP-16 链路。

### B.5 "保留历史"的价值观

本项目坚持 `git revert`（保留历史）而非 `git reset --hard`（抹除历史）的原因：

1. **可审计**：AI 产生了什么、被 revert 了什么，全程在 `git log` 可见。
2. **安全**：revert 即使出错也可以再次 revert 回去；reset 一旦 force push 则协作者历史丢失。
3. **CI/CD 友好**：revert commit 触发 CI，可以验证回滚后状态正确。
4. **CLAUDE.md 禁区**：`git reset --hard` 永远不提供 UI 入口（§禁区）。

---

## §C 功能范围（Scope）

### Do（v1.0 明确实施 · ≥ 10 项）

1. Session 详情视图顶部加"一键回滚"按钮（红色系警告色 · design token `--color-status-error`）
2. 点击"一键回滚"→ 弹预览 modal：显示 revert diff 摘要（N 个 commit 将被 revert · 影响文件列表）
3. 预览 modal 含置信度警告：低置信度（< 0.9）的 commit 高亮 + 提示用户确认是否包含
4. 用户在预览 modal 点"确认回滚"→ 弹**二次确认 dialog**（危险操作防误触）
5. 二次确认通过 → 执行 `git revert` 序列（`rollback:execute` IPC）
6. 执行过程实时反馈：进度 banner `"正在回滚 {N} 个 commit · {done}/{total} 完成"`
7. revert 生成的 commit message 统一加后缀 `[AI session rollback: <session-id>]`
8. 冲突处理：若任一 revert 冲突 → 停在该 commit → **转接 MVP-16 冲突解决 UI**（conflict banner + 3-way diff）
9. 用户可在执行中途 `abort`（`rollback:abort` IPC）→ 干净回到 revert 开始前的 HEAD（使用 `Repository::cleanup_state()` + 反向 revert 已完成的 revert commit）
10. revert 全部完成 → Session 详情视图标记"已回滚"（状态徽章 · 灰色 · 保留历史不删 session）
11. "已回滚"状态下，"一键回滚"按钮变灰 + tooltip `"此 session 已回滚"`
12. 操作日志：每次 rollback 执行（含 abort / conflict）写入 `vibestation.db` `rollback_ops` 表（见 §G + 附录 A）
13. 跨平台支持：macOS + Linux（Ubuntu 24）行为一致

### Don't（明确不做 · ≥ 6 项）

1. **`git reset --hard`**：危险操作，永远不提供 UI 入口（CLAUDE.md 禁区）
2. **部分 revert**（例如"只回滚 session 里的其中 2 个 commit"）→ 留给 v2+（用户可通过 MVP-16 cherry-pick 手动操作）
3. **跨 session 的 combined revert**（把 session A + session B 一起 revert）→ 留给 v2+
4. **自动 stash 工作区改动**（revert 前如有未提交改动，报错提示用户先处理，不自动 stash）
5. **revert --no-commit 模式**（只放入 working tree 不产生 commit）→ v2+ 评估
6. **回滚后自动 push**（revert commit 产生后不自动 push，用户手动 push，MVP-21 范围）
7. **revert 某条 merge commit**（`git revert -m 1/2` 参数 · 复杂度高）→ v1.1 评估

---

## §D UI Wireframe

### D.1 Session 详情视图 · 顶部操作区

```
┌─────────────────────────────────────────────────────────────────┐
│  Session #42 · Claude CLI · 2026-05-10 14:23 – 16:05           │
│  7 commits · 23 files changed · +1,240 / -380 lines            │
│                                                                  │
│  [ℹ Commit 列表]  [↩ 一键回滚]  ← 红色按钮·design token error  │
│                                                                  │
│  ── 已回滚 ─────────────────────── （回滚后状态徽章）           │
└─────────────────────────────────────────────────────────────────┘
```

### D.2 预览 Modal（`rollback:preview` 返回数据渲染）

```
┌────────────────────────────────────────────────────────────────────┐
│  ⚠ 回滚预览：Session #42                                           │
│  ────────────────────────────────────────────────────────────────  │
│  将生成 7 个 revert commit，撤销以下改动：                          │
│                                                                    │
│  ✓  abc1234  feat: 重构 git_ops.rs 核心路径           置信度 0.97  │
│  ✓  bcd2345  fix: 修复 stage 路径异常                  置信度 0.95  │
│  ✓  cde3456  test: 补 stage 单元测试                   置信度 0.94  │
│  ⚠  def4567  chore: 格式化 Cargo.toml                 置信度 0.72  │
│              ↑ 低置信度关联，建议确认是否属于本 session            │
│  ✓  efg5678  refactor: 抽取 GitError enum              置信度 0.99  │
│                                                                    │
│  影响文件：crates/core/src/git_ops.rs · +12 / -8 net             │
│            crates/core/src/lib.rs · +3 / -1 net                  │
│            …（共 23 文件）  [展开查看完整列表]                    │
│                                                                    │
│  ⚠ 警告：1 个 commit 置信度 < 0.9，请确认是否包含在回滚中         │
│  □ 包含低置信度 commit def4567（默认不勾选）                       │
│                                                                    │
│          [取消]          [确认回滚 →]                             │
└────────────────────────────────────────────────────────────────────┘
```

### D.3 二次确认 Dialog（危险操作防误触）

```
┌──────────────────────────────────────────────┐
│  ⚠ 确认回滚                                  │
│  ──────────────────────────────────────────  │
│  即将执行 git revert 序列：                   │
│  · 7 个新 revert commit 将写入历史           │
│  · 原始 commit 不会被删除                    │
│  · 操作可用 git revert <revert-sha> 撤销     │
│                                              │
│  输入 session ID 确认：[____42____]          │
│                                              │
│  [取消]       [执行回滚] ← 红色 · 不可忽略  │
└──────────────────────────────────────────────┘
```

### D.4 执行中进度 Banner

```
┌─────────────────────────────────────────────────────────────┐
│  ↩ 正在回滚 Session #42 · 3/7 已完成 ████████░░░░░░ 43%    │
│  当前：revert bcd2345 "fix: 修复 stage 路径异常"            │
│  [取消回滚]  ← 仅执行中可用                                 │
└─────────────────────────────────────────────────────────────┘
```

---

## §E Acceptance（≥ 25 checkbox）

### E.1 功能正确性

- [ ] E.1.1 Session 详情视图顶部有"一键回滚"按钮（红色系 `--color-status-error`）
- [ ] E.1.2 点击"一键回滚"→ 调 `rollback:preview` IPC → 返回 commit 列表 + 置信度 + 影响文件
- [ ] E.1.3 预览 modal 展示全部 commit + 置信度分级（≥ 0.9 / < 0.9 视觉区分）
- [ ] E.1.4 低置信度 commit（< 0.9）默认不选 · 用户可手动勾选包含
- [ ] E.1.5 点"确认回滚"→ 弹二次确认 dialog · 要求输入 session ID（附录 B 文案模板）
- [ ] E.1.6 二次确认通过 → 调 `rollback:execute` IPC → 后端执行 `git revert` 序列
- [ ] E.1.7 执行中：顶部 progress banner 显示 `{done}/{total}` + 百分比进度条
- [ ] E.1.8 生成的 revert commit message = `"Revert \"{original message}\" [AI session rollback: {session_id}]"`
- [ ] E.1.9 执行完成 → Session 详情标记"已回滚"灰色徽章
- [ ] E.1.10 "已回滚"状态：按钮变灰 + tooltip `"此 session 已回滚"`
- [ ] E.1.11 执行完成 → Git Log 面板刷新 · N 个新 revert commit 可见

### E.2 危险操作安全防护

- [ ] E.2.1 二次确认 dialog 中"执行回滚"按钮为红色（`--color-status-error` 实心）
- [ ] E.2.2 session ID 输入错误时"执行回滚"按钮 disabled
- [ ] E.2.3 **严禁出现** `git reset --hard` 或任何不可逆 reset 操作（代码层面用 `cargo grep` 验证）
- [ ] E.2.4 `rollback:execute` IPC 在后端校验：working tree 有未提交改动 → 返回 `DirtyWorkingTree` 错误 · 前端显示明确提示 + 跳转 Status 面板

### E.3 中断与 Abort

- [ ] E.3.1 执行中点"取消回滚"→ 调 `rollback:abort` IPC
- [ ] E.3.2 `rollback:abort` 成功 → HEAD 回到 revert 序列开始前的 commit SHA（0 残留 revert commit）
- [ ] E.3.3 `rollback:abort` 后 Git Log 刷新 · 原 session commit 仍在历史中（未被影响）
- [ ] E.3.4 abort 后 Session 详情恢复"未回滚"状态（回滚按钮重新可用）

### E.4 冲突处理（复用 MVP-16）

- [ ] E.4.1 某 commit revert 产生冲突 → progress banner 变为 ConflictBanner（MVP-16 组件 · 红色）
- [ ] E.4.2 冲突 UI 显示冲突文件列表 + 3-way Diff（MVP-16 `web/src/panels/Diff/3way/`）
- [ ] E.4.3 用户解决冲突后点 Continue → 继续执行剩余 revert
- [ ] E.4.4 冲突中 Abort → 同 E.3.2 干净回到起点（`Repository::cleanup_state()` + 反向恢复）

### E.5 历史保留验证

- [ ] E.5.1 回滚完成后 `git log` 显示：原 session commit + N 个新 revert commit（总 commit 数 +N）
- [ ] E.5.2 回滚完成后 working tree 状态 = session 开始前的状态（通过 `git diff <pre-session-sha>` 验证为空）
- [ ] E.5.3 回滚产生的 revert commit 包含 `[AI session rollback: {id}]` 后缀

### E.6 可访问性（a11y）

- [ ] E.6.1 "一键回滚"按钮有 `aria-label="回滚 Session #N 的所有 AI commit"`
- [ ] E.6.2 预览 modal 支持 `Esc` 键关闭（等价于点"取消"）
- [ ] E.6.3 二次确认 dialog 支持 `Enter` 提交（session ID 输入框聚焦时）
- [ ] E.6.4 执行中 progress banner 有 `role="status"` + `aria-live="polite"` 供屏幕阅读器读取

---

## §F 测试矩阵

### F.1 单元测试（`cargo test`）

| 测试名                           | 覆盖路径                                                                | 关键断言                                                     |
| -------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------ |
| `test_revert_single_commit`      | `rollback_ops::revert_commit()`                                         | commit 成功 + message 含 `[AI session rollback:]` 后缀       |
| `test_revert_sequence_5_commits` | `rollback_ops::revert_sequence()`                                       | 5 commit 顺序 revert · 无冲突 · HEAD 与 pre-session SHA 一致 |
| `test_revert_conflict_detection` | `rollback_ops::revert_commit()` + `Repository::index().has_conflicts()` | 返回 `Conflict` variant · 不推进 HEAD                        |
| `test_abort_cleanup`             | `rollback_ops::abort_revert()`                                          | `Repository::cleanup_state()` 调用 + HEAD 回到起点           |
| `test_dirty_working_tree_guard`  | `rollback_ops::check_preconditions()`                                   | 有未提交改动时返回 `DirtyWorkingTree` 错误                   |
| `test_low_confidence_filter`     | `rollback_ops::build_revert_plan()`                                     | 置信度 < 0.9 的 commit 默认不包含                            |
| `test_message_suffix_format`     | `rollback_ops::build_revert_message()`                                  | message 格式精确匹配                                         |
| `test_rollback_state_persisted`  | `rollback_ops` + `db::rollback_ops`                                     | 执行中断崩溃后 DB 有 `in_progress` 记录                      |

### F.2 集成测试（`cargo test --features integration`）

| 测试名                                     | 覆盖路径                                     | 场景                                                                               |
| ------------------------------------------ | -------------------------------------------- | ---------------------------------------------------------------------------------- |
| `integration_full_session_revert_5commits` | `rollback_ops` + IPC + SQLite                | 构造 5 commit session fixture → `rollback:execute` → 验证 tree + DB 记录           |
| `integration_abort_mid_revert`             | `rollback_ops::revert_sequence` + `abort`    | 第 3 commit 后 abort → HEAD 精确回到起点 · revert_ops 表记录 `aborted`             |
| `integration_conflict_resume`              | `rollback_ops` + `conflict_resolve_file` IPC | 第 2 commit 冲突 → 手动解决 → continue → 完成 · 验证最终 tree                      |
| `integration_crash_recovery`               | `rollback_ops::detect_in_progress()`         | mock 进程中断场景 · 重启后检测 `REVERT_HEAD` + DB in_progress 记录 · 恢复 UI state |

### F.3 E2E 测试（Playwright）

| 测试名                              | 覆盖路径          | 操作步骤                                                                            |
| ----------------------------------- | ----------------- | ----------------------------------------------------------------------------------- |
| `e2e_one_click_rollback_happy_path` | 完整 UI flow      | 创建 session · 3 commit · 点"一键回滚"→ 预览 modal → 二次确认 → 完成 → 验证 Git Log |
| `e2e_rollback_with_conflict`        | 冲突 UI flow      | 故意制造 revert 冲突 → ConflictBanner 出现 → 3-way diff → 解决 → Continue → 完成    |
| `e2e_abort_rollback`                | abort flow        | 执行中途点"取消回滚"→ 验证 HEAD 未变 · 按钮重新可用                                 |
| `e2e_low_confidence_warning`        | 预览 modal 置信度 | 低置信度 commit 默认未勾选 · 手动勾选后包含在预览中                                 |

---

## §G 数据模型变更

### G.1 `ai_sessions` 表扩展

基于 MVP-19 已定义的 `ai_sessions` 表（`{id, workspace_id, cli_kind, started_at, ended_at, prompt_count, title}`），新增以下字段：

```sql
ALTER TABLE ai_sessions ADD COLUMN rolled_back_at INTEGER;  -- Unix timestamp · NULL = 未回滚
ALTER TABLE ai_sessions ADD COLUMN rollback_commit_shas TEXT;  -- JSON 数组 · revert commit SHA 列表
ALTER TABLE ai_sessions ADD COLUMN rollback_session_id TEXT;   -- 冗余 · 同 id · 方便 grep
```

**迁移策略**：使用 `rusqlite` migration 机制（仿 SPIKE-04.5 B.3 · `CREATE TABLE IF NOT EXISTS` + `ALTER TABLE` 幂等 guard），migration 版本写入 `schema_version` 表。

### G.2 `rollback_ops` 操作日志表（新建）

```sql
CREATE TABLE IF NOT EXISTS rollback_ops (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id  TEXT NOT NULL,
  started_at  INTEGER NOT NULL,  -- Unix timestamp
  finished_at INTEGER,           -- NULL = in-progress / aborted
  status      TEXT NOT NULL,     -- 'in_progress' | 'completed' | 'aborted' | 'conflict_paused'
  commit_plan TEXT NOT NULL,     -- JSON: [{sha, include, confidence, status}]
  current_idx INTEGER NOT NULL DEFAULT 0,
  error_msg   TEXT,              -- 最后一个错误描述（conflict path 等）
  FOREIGN KEY (session_id) REFERENCES ai_sessions(id)
);
```

（字段详见附录 A）

### G.3 IPC Contract

以下为新增 IPC commands（`rollback_ops.rs` 实现 · `crates/app/src/lib.rs` 注册）：

| Command            | Payload                                           | 返回                      | 说明                                      |
| ------------------ | ------------------------------------------------- | ------------------------- | ----------------------------------------- |
| `rollback:preview` | `{session_id: String}`                            | `RollbackPreview`         | 查询 session commits + 置信度 + diff 摘要 |
| `rollback:execute` | `{session_id: String, include_shas: Vec<String>}` | `RollbackProgress` stream | 启动 revert 序列                          |
| `rollback:abort`   | `{session_id: String}`                            | `RollbackAbortResult`     | 中止 + 清理状态                           |
| `rollback:status`  | `{session_id: String}`                            | `RollbackStatus`          | 查询当前执行状态（含 crash recovery）     |

**ts-rs binding 列表**（自动生成到 `web/src/bindings/`）：

```typescript
// 核心类型（8 个新 binding）
RollbackPreview; // commit 列表 + 置信度 + 影响文件
RollbackCommitEntry; // 单个 commit 项 { sha, message, confidence, include }
RollbackProgress; // 执行进度 { done, total, current_sha, status }
RollbackAbortResult; // { success, head_sha, error }
RollbackStatus; // { session_id, status, current_idx, total }
RollbackOpError; // enum: DirtyWorkingTree | ConflictDetected | SessionNotFound | Git2Error | ...
RollbackCommitEntry; // 同上（preview + execute 共用）
DiffSummary; // { files_changed, insertions, deletions }（MVP-08 已有 · 复用）
```

### G.4 Tauri Events（前端订阅）

| Event Name              | Payload                                            | 触发时机                  |
| ----------------------- | -------------------------------------------------- | ------------------------- |
| `git:rollback-progress` | `RollbackProgress`                                 | 每个 revert commit 完成后 |
| `git:rollback-conflict` | `{ path: String, commit_sha: String }`             | revert 产生冲突时         |
| `git:rollback-done`     | `{ session_id: String, revert_shas: Vec<String> }` | 全部完成                  |
| `git:rollback-aborted`  | `{ session_id: String, head_sha: String }`         | abort 完成                |

---

## §H 决策表

| #   | 决策点                                     | 当前状态                                                                                    | 备注                                                                      |
| --- | ------------------------------------------ | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| H.1 | revert vs reset 策略                       | **锁定：revert only**（CLAUDE.md 禁区）                                                     | reset --hard 永远不提供 UI 入口                                           |
| H.2 | 低置信度 commit 处理                       | **锁定：默认不包含 · 用户可勾选**                                                           | confidence < 0.9 高亮警告 · 不强制                                        |
| H.3 | 冲突解决 UI 复用 vs 自建                   | **锁定：复用 MVP-16**（不重新发明轮子）                                                     | `ConflictBanner` + `Diff/3way/` 直接复用                                  |
| H.4 | abort 后的状态恢复策略                     | **锁定：`Repository::cleanup_state()` + 已完成 revert 的反向 revert**                       | 同 MVP-16 §H.2 策略                                                       |
| H.5 | 二次确认方式                               | **锁定：输入 session ID 确认**（仿 GitHub repo delete 确认模式）                            | 防止误触 · 附录 B 文案                                                    |
| H.6 | SPIKE-07 决策                              | **待 SPIKE-07 完成**（MVP-19 session 边界识别方案确认后再 lock）                            | MVP-20 对 SPIKE-07 路径不预设                                             |
| H.7 | MVP-19 session 边界准确率结果              | **待 MVP-19 实施期 benchmark 报告**（MVP-19 E3.3 `>= 95%` 为目标 · 非硬 gate）              | 低于 95% 时**由 Arbiter 决策** block 或降 confidence 阈值（非自动 block） |
| H.8 | 部分 revert（只回滚 N of M commit）        | **推 v2+**（v1.0 只做全部 revert）                                                          | 用户可通过 MVP-16 cherry-pick 手动操作                                    |
| H.9 | crash recovery（app 崩溃中断 revert 序列） | **v1.0 实施 Phase D**：检测 `REVERT_HEAD` + `rollback_ops` DB in_progress 记录 → 重启后提示 | 复用 MVP-16 crash recovery 模式                                           |

---

## §I 实施 Phase 拆分

| Phase                                         | 范围                                                                                                                                                                         | 估时 | 依赖                       |
| --------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- | -------------------------- |
| **Phase A · 后端 rollback 包装**              | 新建 `crates/core/src/rollback_ops.rs`（`revert_sequence` / `abort_revert` / `detect_in_progress`）· 4 IPC + 8 ts-rs binding · 50+ 单元测试 · `rollback_ops` DB 表 migration | 2d   | MVP-19 done                |
| **Phase B · 预览 UI**                         | `web/src/panels/SessionDetail/` 加"一键回滚"按钮 + `RollbackPreviewModal` 组件（commit 列表 · 置信度 · 低置信度警告）+ 二次确认 dialog                                       | 1.5d | Phase A done               |
| **Phase C · 冲突处理 wire MVP-16**            | `RollbackConflictBridge`：接 `git:rollback-conflict` event → 触发 MVP-16 `ConflictBanner` + `Diff/3way/` · 解决后 resume `rollback:execute`                                  | 1d   | Phase B done + MVP-16 done |
| **Phase D · 状态机 + abort + crash recovery** | 执行中 progress banner · `rollback:abort` flow · app 启动检测 `REVERT_HEAD` + DB in_progress → 全局 recovery banner                                                          | 1d   | Phase C done               |
| **Phase E · runtime 证据 + 性能量化**         | 截图：预览 modal / 执行中 / 完成状态 / 冲突场景 · Criterion bench（5 / 20 commit session）· 放 `docs/runtime-evidence/mvp-20/` · macOS + Linux 双平台                        | 0.5d | Phase D done               |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动）：

- [ ] `crates/core/Cargo.toml` 已含 `git2`（继承 MVP-09/16）· 不需要新增依赖
- [ ] 新建 `crates/core/src/rollback_ops.rs`（独立模块 · 不和 `rebase_ops.rs` 混）
- [ ] git2 API 调用链：
  - **Revert 单 commit**：`Repository::revert(commit, None)` → `Repository::index()?.has_conflicts()` → 无冲突时 `Repository::commit()` 写 revert commit
  - **Cleanup**：`Repository::cleanup_state()` 清 `REVERT_HEAD`
  - **Dirty tree guard**：`Repository::statuses(None)?` 检查 `StatusFlags::INDEX_*` / `STATUS_WT_*`
- [ ] IPC commands 注册（`crates/app/src/lib.rs` `invoke_handler!`）：`rollback_preview` / `rollback_execute` / `rollback_abort` / `rollback_status`
- [ ] permission toml：`crates/app/permissions/rollback_ops.toml` 新建 · 含 4 个 `allow-{name}`
- [ ] `rollback_ops` SQLite 表 migration（见 §G.2 SQL 定义）
- [ ] 复用 MVP-09/16 `CommitError` / `RebaseOpError` 模式 → 新 `RollbackError` enum（`DirtyWorkingTree / ConflictDetected / SessionNotFound / InProgress / Git2Error / DbError`）

---

## §J 风险表

| #   | 风险                                                                                | 概率  | 影响 | Mitigation                                                                                                                           |
| --- | ----------------------------------------------------------------------------------- | ----- | ---- | ------------------------------------------------------------------------------------------------------------------------------------ |
| R1  | MVP-19 session 识别准确率 < 95%，导致 revert 误操作                                 | 中    | 高   | Phase A gate：`rollback:execute` 调用前强制查 `confidence ≥ 0.9`；低置信度 commit 默认不包含 + 显著警告                              |
| R2  | revert 序列中途 app 崩溃（或用户强制退出），留下 `REVERT_HEAD` + 部分 revert commit | 中    | 中   | Phase D crash recovery：重启检测 `REVERT_HEAD` + DB in_progress 记录 → 引导用户 continue / abort（复用 MVP-16 §Crash recovery 模式） |
| R3  | revert 产生复杂冲突（原 commit 已被后续 commit 改写），3-way diff 无法自动解决      | 低-中 | 中   | 直接复用 MVP-16 manual edit 模式；用户可在 3-way diff 中手动编辑；提供 abort 逃生路线                                                |
| R4  | `git revert` 顺序问题（commit 越新越要先 revert）导致不必要冲突                     | 低    | 中   | Phase A 单元测试覆盖：`build_revert_plan` 必须按 commit time 降序排列（newest first revert）                                         |
| R5  | 用户误触"一键回滚"造成不可预期 revert                                               | 中    | 高   | 两道防护：预览 modal + session ID 二次确认；`revert` 保留历史可再次 revert 撤销；UI 层面红色警告色 + 明确文案                        |

---

## §K IPC Contract 详细定义

```typescript
// rollback:preview 返回类型
interface RollbackPreview {
  session_id: string;
  commits: RollbackCommitEntry[];
  total_files_changed: number;
  total_insertions: number;
  total_deletions: number;
  has_low_confidence: boolean;
}

interface RollbackCommitEntry {
  sha: string;
  message: string;
  author: string;
  timestamp: number; // Unix timestamp
  confidence: number; // 0.0 - 1.0, from session_commit_links
  include: boolean; // 默认 confidence >= 0.9
  files_changed: number;
}

// rollback:execute payload
interface RollbackExecutePayload {
  session_id: string;
  include_shas: string[]; // 用户最终确认要 revert 的 SHA 列表
}

// rollback:abort payload
interface RollbackAbortPayload {
  session_id: string;
}

// rollback:status 返回类型
interface RollbackStatus {
  session_id: string;
  status: "idle" | "in_progress" | "conflict_paused" | "completed" | "aborted";
  current_idx: number;
  total: number;
  current_sha: string | null;
}
```

**ts-rs binding 映射**（Rust → TypeScript · 自动生成）：

```rust
// crates/core/src/rollback_ops.rs
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RollbackPreview { ... }

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RollbackCommitEntry { ... }

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RollbackError {
    DirtyWorkingTree,
    ConflictDetected { path: String },
    SessionNotFound { session_id: String },
    InProgress { current_sha: String },
    Git2Error(String),
    DbError(String),
}
```

---

## §L 跨平台考量

### L.1 macOS vs Linux 行为差异

| 考量点                          | macOS                                       | Linux（Ubuntu 24）                   | 处理方式                                                   |
| ------------------------------- | ------------------------------------------- | ------------------------------------ | ---------------------------------------------------------- |
| `git revert` 行为               | 标准                                        | 标准                                 | 无差异 · git2 `Repository::revert()` 跨平台一致            |
| 文件 lock（`.git/REVERT_HEAD`） | HFS+ 无强 file lock                         | ext4 无强 file lock · 多进程写需注意 | `rollback_ops.rs` 使用原子文件写操作 · 不依赖 OS file lock |
| `Repository::cleanup_state()`   | 删除 `.git/{REVERT,MERGE,CHERRY_PICK}_HEAD` | 同                                   | git2 跨平台封装 · 行为一致                                 |
| 文件路径大小写                  | HFS+ 默认大小写不敏感                       | ext4 大小写敏感                      | revert commit 路径比较使用 `OsStr` + 规范化                |
| SQLite WAL 模式                 | 兼容                                        | 兼容                                 | 继承 SPIKE-04.5 B.1-5 · 无需额外处理                       |

### L.2 CI 验证

Phase E runtime 证据必须在 macOS + Linux（Ubuntu 24 · GitHub Actions runner）双平台跑 Criterion bench 并归档到 `docs/runtime-evidence/mvp-20/`，格式参考 MVP-09 Phase D 模式。

---

## §M 危险操作 UX 规范

### M.1 设计原则

参考 `CLAUDE.md` "🚫 禁区"：任何可能不可逆的操作（即使 revert 保留历史，对用户来说"回滚"是心理上的高风险操作）必须：

1. **颜色**：按钮使用 `--color-status-error`（红色系 · 与 Calm Studio 设计 token 一致）
2. **文案**：明确说明操作后果（"7 个新 revert commit"而非"回滚"模糊描述）
3. **两道防护**：预览 modal（说明后果）+ session ID 二次确认（防误触）
4. **逃生路线**：任何步骤都有"取消"和明确的 abort 路径

### M.2 "执行回滚"按钮文案规范

```
按钮文案：执行回滚（{N} 个 commit）
按钮颜色：var(--color-status-error) 实心填充
按钮字号：14px · font-weight: 500
disabled 态：opacity: 0.4 · cursor: not-allowed（session ID 未输入或输入错误时）
```

### M.3 禁止出现的 UI 模式

- **禁止**：将"一键回滚"按钮与普通操作按钮放在同一视觉权重层次
- **禁止**：省略预览 modal（即使"只有 1 个 commit"）
- **禁止**：在 tooltip 或帮助文本中使用 `reset`、`删除历史` 等可能误导用户的词汇

---

## §N 自审四问

1. **递归完备性**：spec 是否覆盖了自身定义的所有路径？
   - 预览 → 确认 → 执行 → 完成：§D + §E.1 ✅
   - 预览 → 确认 → 执行 → 冲突 → 解决 → 完成：§C.8 + §E.4 ✅
   - 预览 → 确认 → 执行 → abort：§C.9 + §E.3 ✅
   - 崩溃恢复：§H.9 + §I Phase D + §J R2 ✅
   - 低置信度 commit 处理：§C.3 + §D.2 + §E.1.3-4 ✅

2. **反向场景**：规则不遵守会怎样？
   - 如果跳过二次确认：用户误触导致不必要 revert → 已通过 §E.2.2 disabled guard 防止
   - 如果 abort 不干净：残留 revert commit 导致历史混乱 → §E.3.2 断言 HEAD 精确回到起点
   - 如果 MVP-19 准确率不达标：误 revert 无关 commit → §H.7 明确 block 条件

3. **边界适用性**：
   - 1 个 commit session：支持（执行 1 次 revert）✅
   - 50 个 commit session：支持（序列 revert · §J R4 mitigation 确保顺序）✅
   - 全部低置信度 commit session：支持（用户全部手动勾选后执行）✅
   - 已部分 revert（abort 后重新执行）：支持（DB 状态重置 · `rollback:status` 返回 idle）✅

4. **YAGNI**：
   - 部分 revert：v2+，当前用户 MVP-16 cherry-pick 手动处理 ✅
   - 跨 session revert：v2+，场景罕见 ✅
   - revert --no-commit：v2+，当前预览 modal 已满足"看后再决定"需求 ✅

---

## 附录 A · 操作日志格式（rollback_ops 表字段建议）

`vibestation.db` 中 `rollback_ops` 表完整字段说明：

| 字段          | 类型             | 说明                                                                                            | 示例                                                                   |
| ------------- | ---------------- | ----------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `id`          | INTEGER PK       | 自增主键                                                                                        | `42`                                                                   |
| `session_id`  | TEXT NOT NULL    | 关联 `ai_sessions.id`                                                                           | `"session-abc-123"`                                                    |
| `started_at`  | INTEGER NOT NULL | Unix timestamp（ms）                                                                            | `1715694180000`                                                        |
| `finished_at` | INTEGER          | NULL = in-progress                                                                              | `1715694240000`                                                        |
| `status`      | TEXT NOT NULL    | `in_progress` / `completed` / `aborted` / `conflict_paused`                                     | `"completed"`                                                          |
| `commit_plan` | TEXT NOT NULL    | JSON 数组 `[{sha, include, confidence, status}]` · `status`: `pending` / `reverted` / `skipped` | `[{"sha":"abc","include":true,"confidence":0.97,"status":"reverted"}]` |
| `current_idx` | INTEGER          | 当前执行到第几个 commit（0-based）                                                              | `3`                                                                    |
| `error_msg`   | TEXT             | 最后一个错误（冲突路径等）                                                                      | `"Conflict on src/main.rs"`                                            |

**查询示例**（用于 crash recovery 检测）：

```sql
SELECT * FROM rollback_ops
WHERE status = 'in_progress'
ORDER BY started_at DESC
LIMIT 1;
```

---

## 附录 B · 危险操作 confirm modal 文案模板

**Modal 标题**：`⚠ 确认回滚 Session #{id}`

**正文**（中文 · 明确说明后果）：

```
即将执行 git revert 序列：

  • 将生成 {N} 个新的 revert commit
  • 原始 {N} 个 commit 保留在历史中（不会被删除）
  • 代码状态将回到 {session_start_sha_short}（{session_started_at}）前

此操作可通过再次 revert 相应 commit 撤销。

输入 Session ID 以确认：
```

**按钮区**：

```
[取消]  ·  [执行回滚（{N} 个 commit）]
           ↑ 红色实心按钮 · disabled 直到 session ID 正确输入
```

**字号 / 颜色规范**：

- 标题：16px · `--color-status-error` · `font-weight: 600`
- 正文：14px · `--color-text-primary`
- bullet 列表：`--color-text-secondary`
- 输入框 border（输入正确时）：`--color-status-success`
- 输入框 border（输入错误时）：`--color-status-error`

---

## 附录 C · 测试 fixture 生成器伪代码

构造 5 commit AI session 的 test fixture outline（用于 §F.1 + §F.2 集成测试）：

```rust
/// 在 tempdir 中构造一个包含 5 commit 的 AI session fixture
/// 返回 (repo_path, pre_session_sha, session_id, commit_shas[5])
fn create_5commit_session_fixture() -> (TempDir, String, String, Vec<String>) {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();

    // 配置 identity
    let sig = Signature::now("Test", "test@test.com").unwrap();

    // 初始 commit（pre-session 状态）
    let pre_sha = make_commit(&repo, &sig, "initial: project setup", &[]);

    // 写入 ai_sessions + session_commit_links
    let session_id = "test-session-42";
    // (DB 操作略 · 使用测试 in-memory SQLite)

    // 5 个 session commit（模拟 AI 产生的改动）
    let mut shas = Vec::new();
    for i in 0..5 {
        write_file(dir.path(), &format!("src/file_{i}.rs"), &format!("// AI change {i}"));
        let sha = make_commit(&repo, &sig, &format!("feat: AI change {i}"), &[]);
        link_commit_to_session(session_id, &sha, 0.95 + i as f32 * 0.01);
        shas.push(sha);
    }

    (dir, pre_sha, session_id.to_string(), shas)
}

/// 验证 revert 完成后 working tree = pre-session 状态
fn assert_tree_matches_pre_session(repo: &Repository, pre_sha: &str) {
    let pre_commit = repo.find_commit(Oid::from_str(pre_sha).unwrap()).unwrap();
    let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
    let diff = repo.diff_tree_to_tree(
        Some(&pre_commit.tree().unwrap()),
        Some(&head_commit.tree().unwrap()),
        None,
    ).unwrap();
    assert_eq!(diff.stats().unwrap().files_changed(), 0);
}
```

---

_详化时间：2026-05-14 · Droid (Factory.ai) · self-review · spec draft → 等主 agent 翻 ready_
