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
reviewer: Codex CLI · self-review
---

# MVP-18: AI-Aware Pane 联动

> **状态**：`draft`（v1.0 vision · spec 已详化为 ready-candidate；最终 `ready` 翻转由 Arbiter approve 后主 agent 独立提交）
> **依赖**：[MVP-14](./MVP-14-pane-advanced-layout.md)（Pane 高级布局已就绪）+ [SPIKE-07](./SPIKE-07-cli-protocol-parser.md)（CLI 协议 parser 验证必须 PASS 后才能实施）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) · [`implementation-plan.md §5.3.6`](../implementation-plan.md) · [`implementation-plan.md §1.1`](../implementation-plan.md)
> **详化时间**：2026-05-14 session 31 · Codex CLI self-review（单人项目 v2-D.2 模式）
> **对外叙事边界**：内部 task / ADR / implementation plan 可讨论本能力；公开 README / landing / release / 社交文案在功能真实落地前只允许使用脱敏代号 `v1.0 vision feature X`，不得展开具体能力。

---

## §A. 目标（Goal）

MVP-18 的目标是在现有 Pane 系统上实现 AI Pane 与 Runner / Watch / Log / Build Pane 之间的**订阅 + 失败反哺**机制：用户可以显式把一个 AI Pane 订阅到一个执行型 Pane，当执行型 Pane 发生 build fail、test fail、command error 等失败事件时，系统解析失败输出并把 `parsed_issues` 作为候选上下文送到 AI Pane 顶部，用户手动确认后再追加到当前对话输入。

这个目标保留占位 spec 的核心语义：AI 订阅某个 Pane 的失败事件；该 Pane 失败时把 `parsed_issues` 反哺给 AI；parser 不可靠时降级为原始文本；用户可以 unlink / re-link；永远不自动触发 AI 回复。具体业务场景包括：Rust 项目里 Claude CLI 改代码后 `cargo watch -x test` 失败，AI Pane 收到 rustc diagnostics；前端项目里 Codex CLI 修改组件后 `pnpm test --watch` 失败，AI Pane 收到 Vitest failure + 文件定位；Python 项目里 `pytest -q` 失败，AI Pane 收到 traceback 摘要但用户仍需按 Enter 才会发送。

本 MVP 不是新建 AI runtime，也不是替代 IDE Problems 面板。它只把 Vibestation 已有的 Pane、PTY、TaskRunner、CLI parser 和手动确认 UX 串成一个低风险工作流，让用户少一次复制错误日志、多一次可审计的上下文确认。

## §B. 背景（Context）

`implementation-plan.md §1.1` 明确当前产品对外只承诺“多 Tab 终端 + Git 工作台”，AI session 感知类能力作为 v1.0 vision 内部保留。`implementation-plan.md §5.3.6` 进一步把 AI-Aware Pane 联动推到 v1.0，并要求正式实现前必须通过 parser-oriented spike：真实 Claude CLI transcript、Codex CLI transcript，以及 rustc / tsc / gcc 等编译器输出样本需要证明 `parsed_issues` 字段可行且稳定。MVP-18 详化正是把这个 v1.0 内部设计从占位 spec 扩展为可下发实施的工程 contract。

本 spec 详化本身**不依赖 SPIKE-07 结果**。原因是详化只定义目标、边界、数据模型、IPC、测试矩阵、风险与验收，不做 parser 结论，也不降低 R1。但 MVP-18 的任何实现 PR 都必须在 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) PASS 后启动；若 SPIKE-07 未通过，MVP-18 只能保持 draft / ready-candidate，不得进入 implementation。§H 决策表中保留“留 SPIKE-07 决策”项，避免详化 PR 伪造 parser 准确率或替 Arbiter 接受 R1。

上游 [MVP-14](./MVP-14-pane-advanced-layout.md) 已把高级 Pane 布局、Pane identity、workspace 隔离、Smart Layouts 与最大化等基础能力打稳；MVP-18 复用这些能力，不改 LayoutNode schema，不改 Pane PTY 生命周期。下游 [MVP-19](./MVP-19-session-commit-binding.md) 需要本 MVP 提供稳定的 pane link 和失败反哺事件，才能把 AI session 与 commit 绑定起来；[MVP-20](./MVP-20-ai-one-click-rollback.md) 则依赖 MVP-19 的 session 级数据做安全回滚。

本能力的价值不是“自动让 AI 修复一切”。相反，核心是把失败信号变成**用户可见、可确认、可撤销**的上下文候选。这样可以降低 copy/paste 成本，但仍保留人的最终发送动作，避免 parser 错误、恶意日志或路径注入被直接送入 AI 对话。

## §C. 功能范围（Scope）

**Do**：

- 新增 Pane link 概念：`parent_pane_id` 表示接收上下文的 AI Pane，`child_pane_id` 表示被订阅的 Runner / Watch / Log / Build Pane。
- 新增 `pane:link` IPC 命令，用于建立同 workspace、同 tab 或同 workspace 可见 Pane 之间的订阅关系。
- 新增 `pane:unlink` 或等价参数化命令，用于用户手动解除订阅，防止 AI context 污染。
- 新增 `pane:linked` 事件，用于通知前端 store 更新订阅边、UI badge、mini-toolbar 状态。
- 新增 `pane:trigger` 事件，用于记录被订阅 Pane 的失败触发源，例如 exit code、signal、watch rerun、manual rerun。
- 新增 `pane:build-failed` 事件，payload 必须包含 `parsed_issues`、原始文本摘要、parser 置信度和 fallback 标记。
- AI Pane 接收失败反哺后，只把候选上下文追加到待发送输入区域或顶部 callout，不自动发送。
- 用户点击“插入到输入”后，AI Pane 才把 sanitized prompt fragment 合并到当前 draft。
- 支持 build / test / command error 三类失败源，首批至少覆盖 `cargo`、`pnpm` / `npm`、`pytest`、`go test`。
- 支持 parser 失败 fallback：结构化解析失败时显示原始文本摘要，标记 `fallbackMode = rawText`，不崩溃。
- 支持每条 link 的 enable / disable 状态，用户可以临时暂停订阅而不删除关系。
- 支持同一 AI Pane 订阅多个执行型 Pane，但每次失败只展示一个明确来源，避免混合日志。
- 支持一个执行型 Pane 被多个 AI Pane 订阅，但每个接收 Pane 都有独立 confirmation state。
- 支持 workspace 内持久化 link；重启后恢复 link 列表，但不会恢复未确认的 prompt draft。
- 支持 link 关系审计字段：created_at、updated_at、last_triggered_at、created_by。
- 提供 a11y 完整路径：键盘建立 link、键盘 unlink、screen reader 可读来源和状态。
- 记录 runtime evidence：link 建立、失败反哺、fallback、unlink、跨 workspace 禁止五类证据。

**Don't**：

- 不自动 trigger AI 的回复；发送永远需要用户手动 Enter 或点击发送。
- 不跨 workspace 联动；workspace A 的 Pane 不得订阅 workspace B 的 Pane。
- 不做跨机器、远程 workspace、SSH session 级联动。
- 不做基于行为预测的自动建议，例如“系统认为你应该订阅这个 Pane”。
- 不做 AI-on-AI 推理、自动总结错误原因、自动生成 patch。
- 不在 README、landing、release notes、社交文案里展开本能力；公开材料只允许使用脱敏代号。
- 不修改 [MVP-14](./MVP-14-pane-advanced-layout.md) 的 LayoutNode schema，也不为了 link 新增 Pane 类型。
- 不修改 [MVP-17](./MVP-17-external-terminal-pane-detach.md) 的 detached window lifecycle；detached Pane 场景只做 link 状态降级说明。
- 不把 parser prototype 直接塞进本 MVP；parser source-of-truth 来自 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) 后续产物。
- 不把原始日志无限量写入 AI prompt；必须限长、脱敏、去控制字符。
- 不支持第三方 AI CLI 统一抽象；首批只围绕 SPIKE-07 验证通过的 CLI kinds。
- 不提供“静默学习用户修复方式”或 telemetry 上传。

## §D. UI wireframe（文字描述）

### D.1 Pane header link affordance

AI Pane header 右侧新增 link icon button。点击后打开 lightweight popover，列出当前 workspace 内可订阅的 Runner / Watch / Log / Build Pane。每一行显示 Pane title、当前命令、最近状态、是否已有 link。用户选择目标 Pane 后提交 `pane:link`，成功后 AI Pane header 显示 `Linked: Runner` chip。

### D.2 Runner Pane source badge

被订阅的执行型 Pane header 显示 `Feeds AI` badge。badge hover 展示订阅它的 AI Pane 列表；键盘 focus 时同样可读。若一个 Runner 被多个 AI Pane 订阅，badge 文案显示数量而不是堆叠多个 chip。

### D.3 Failure feedback callout

执行型 Pane 失败后，AI Pane 顶部出现 callout：来源 Pane、失败类型、exit code、parser 置信度、issue count、文件数量、时间。callout 提供三个按钮：`Insert`、`View raw`、`Dismiss`。`Insert` 只把 sanitized fragment 写入 AI Pane draft，不发送。

### D.4 Link management panel

Workspace 级 command palette 或 Pane link popover 内提供 `Manage links` 视图，表格列出 parent、child、kind、enabled、last_triggered_at。用户可 disable、unlink、jump to pane。若 child pane 已关闭，行状态显示 `Missing pane`，提供 remove stale link。

### D.5 Fallback and error state

parser 失败时 callout 不隐藏，而是显示 `Raw text fallback` 状态、原始文本摘要和明确警告。跨 workspace 建立 link、Pane 已关闭、AI Pane 不支持接收、child Pane 无失败输出等错误，均在 popover 内 inline 展示，不用全局 toast 淹没上下文。

## §E. Acceptance

### A. Scope and dependency gates

- [ ] A.1 frontmatter `status` 保持 `draft`；本 PR 不 flip `ready`，ready 翻转由 Arbiter approve 后主 agent 独立提交。
- [ ] A.2 本 spec 详化不依赖 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) 结果；但实施前 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) 必须 PASS，并在 PR body 明确引用其 report。
- [ ] A.3 若 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) 未通过，MVP-18 implementation PR 必须被 blocker gate 拒绝，不允许以 raw-text-only 方式偷跑完整功能。
- [ ] A.4 所有公开文案禁区保持不变；MVP-18 只允许在内部 docs/tasks、ADR、implementation plan 中讨论具体能力。

### B. Link creation and persistence

- [ ] B.1 `pane:link` 只能在同一 `workspace_id` 内建立关系；跨 workspace 请求返回 `PaneLinkError::CrossWorkspaceDenied`。
- [ ] B.2 `parent_pane_id` 必须是支持接收 prompt context 的 AI Pane；非 AI Pane 请求返回 `PaneLinkError::InvalidParentPaneType`。
- [ ] B.3 `child_pane_id` 必须是 Runner / Watch / Log / Build / Shell command pane；纯 UI Pane 请求返回 `PaneLinkError::InvalidChildPaneType`。
- [ ] B.4 重复建立同一 `(workspace_id, parent_pane_id, child_pane_id, link_kind)` 返回已有 link，不产生重复 DB 行。
- [ ] B.5 link 创建成功后 `pane:linked` 事件在 200ms 内到达前端 store，AI Pane header chip 与 child badge 同步更新。
- [ ] B.6 app 重启后恢复 enabled links；missing pane links 标记为 stale，不自动删除。
- [ ] B.7 用户 unlink 后 DB 行软删除或状态变为 disabled，前端 200ms 内移除 header chip。

### C. Failure trigger and parsed issues

- [ ] C.1 被订阅 Pane exit code 非 0 时生成 `pane:trigger`，payload 包含 source pane、exit code、command、cwd、timestamp。
- [ ] C.2 build / test / command error 三类 trigger 都能进入统一 failure pipeline，且 error kind 不依赖 shell 文案猜测。
- [ ] C.3 `pane:build-failed` payload 包含 `parsed_issues: ParsedIssue[]`、`raw_excerpt`、`parser_confidence`、`fallback_mode`。
- [ ] C.4 `parsed_issues` 字段准确率目标沿用 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) 结论；若 SPIKE-07 未给出数字，本 MVP 不得自行发明准确率。
- [ ] C.5 单次 failure 最多注入 20 条 issue；超过时按 severity、文件、行号去重并提示 truncated count。
- [ ] C.6 parser crash、timeout、unsupported format 均降级为 raw text fallback，不崩溃、不丢事件来源。

### D. AI Pane feedback UX

- [ ] D.1 AI Pane 接收 `pane:build-failed` 后只显示候选上下文 callout，不自动发送 prompt。
- [ ] D.2 用户点击 `Insert` 后，sanitized prompt fragment 追加到当前 draft，光标定位到 draft 末尾。
- [ ] D.3 用户点击 `Dismiss` 后只关闭当前 failure callout，不删除 link。
- [ ] D.4 `View raw` 展示限长后的原始文本，保留 ANSI-stripped 内容，不展示隐藏 token 或控制序列。
- [ ] D.5 若 AI Pane 当前已有未发送 draft，插入前显示 merge preview，用户可选择 append 或 cancel。
- [ ] D.6 同一 child Pane 连续失败时按 `(command_run_id, failure_hash)` 去重；重复 failure 不刷屏。

### E. Security and sanitization

- [ ] E.1 prompt fragment 必须经过 `sanitize_ai_prompt` 等价流程：去 ANSI、去 OSC52、拒绝 NUL、限长 8K、路径规范化。
- [ ] E.2 `raw_excerpt` 与 prompt fragment 均不得包含常见 secret key pattern；命中后 redacted 并记录 redaction count。
- [ ] E.3 failure text 中的 shell metachar 不得作为命令执行；本 MVP 只把文本写入 AI draft。
- [ ] E.4 绝对路径指向 `/etc`、`/System`、`/usr`、`/bin`、`/sbin` 时，在 prompt fragment 中转为脱敏路径或 workspace-relative path。
- [ ] E.5 link metadata 不上传、不 telemetry；所有数据只进本地 rusqlite。

### F. Workspace and pane lifecycle boundaries

- [ ] F.1 workspace A 的 links 不影响 workspace B；切换 workspace 后只显示当前 workspace links。
- [ ] F.2 child Pane 被关闭后，link 标记 stale；再次打开同 id 不应发生，除非 Pane id 明确由 backend 恢复。
- [ ] F.3 parent AI Pane 被关闭后，对应 links disabled 或 soft-deleted；child failure 不再产生 callout。
- [ ] F.4 detached Pane 场景若没有可靠 main window target，failure event 进入 backlog，reattach 后再显示或标记 expired。
- [ ] F.5 app quit / crash 后未确认 callout 不恢复，避免把旧错误误注入新 session。

### G. Performance and reliability

- [ ] G.1 从 child Pane failure 到 AI Pane callout visible P99 ≤ 200ms（不含 parser 冷启动，parser cold path 单独记录）。
- [ ] G.2 link / unlink IPC round-trip P99 ≤ 50ms，本地 DB 事务不超过 1 次写放大。
- [ ] G.3 单 workspace 100 条 links 时，store selector 更新不导致所有 Pane body 重渲染。
- [ ] G.4 单次 parser pipeline timeout 默认 2s；timeout 后 fallback raw text，UI 仍在 200ms 内显示 “解析中 / fallback” 状态。
- [ ] G.5 100 次连续 failure 去重后 UI callout 不超过 5 条 backlog，内存增长 < 10MB。

### H. Accessibility and keyboard

- [ ] H.1 link popover 可全键盘操作：打开、选择 child、确认、取消、unlink。
- [ ] H.2 header chip、source badge、failure callout 均有 aria-label，screen reader 能读出 parent / child / enabled / stale 状态。
- [ ] H.3 `Insert`、`View raw`、`Dismiss` 三个按钮 focus order 与视觉顺序一致。
- [ ] H.4 reduced-motion 下 callout 不使用 slide / scale 动画，状态切换仍可见。
- [ ] H.5 错误状态不用颜色单独表达，必须有文字 label。

### I. Testing and evidence

- [ ] I.1 core 单元测试覆盖 link create / duplicate / unlink / stale / cross workspace denied / invalid pane type。
- [ ] I.2 parser pipeline 集成测试使用 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) fixture；无 SPIKE-07 fixture 时测试必须 skipped with reason，而不是伪造样本。
- [ ] I.3 Tauri IPC integration 覆盖 `pane:link` → child failure → `pane:build-failed` → AI callout state。
- [ ] I.4 Playwright E2E 覆盖 link 建立、failure callout、insert、dismiss、unlink、fallback。
- [ ] I.5 Phase D runtime evidence 至少包含 5 张截图或录屏：link 创建、child badge、failure callout、raw fallback、cross workspace denied。

## §F. 测试矩阵

| 层次               | 范围                                                                   | Fixture / 输入                           | 命令                                                             | 覆盖路径              |
| ------------------ | ---------------------------------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------- | --------------------- |
| core unit          | `PaneLink` validator、workspace guard、duplicate handling、soft delete | in-memory rusqlite + synthetic pane rows | `cargo test -p vibestation-core pane_link::`                     | B.1-B.7 / F.1-F.3     |
| parser bridge unit | SPIKE-07 parser output → `ParsedIssue` normalization                   | SPIKE-07 report fixtures                 | `cargo test -p vibestation-core parser_bridge::`                 | C.3-C.6 / E.1-E.4     |
| app integration    | `pane:link` IPC、state emit、DB transaction rollback                   | Tauri test harness + fake pane registry  | `cargo test -p vibestation-app --features integration pane_link` | B.5 / C.1 / C.2       |
| frontend unit      | Solid store selectors、callout reducer、dedupe、stale state            | mocked Tauri events                      | `pnpm -C web exec vitest run tests/panels/Terminal/pane-link`    | D.1-D.6 / G.3         |
| E2E                | user creates link, child fails, AI callout visible, insert, unlink     | Playwright + fixture workspace           | `pnpm -C web exec playwright test pane-linking.spec.ts`          | D.1-D.5 / H.1-H.5     |
| performance bench  | 100 links, 100 failures, parser timeout fallback                       | synthetic events + performance marks     | `pnpm -C web exec vitest bench pane-linking`                     | G.1-G.5               |
| manual QA          | macOS / Linux / Windows keyboard and window lifecycle                  | real app dev mode                        | `pnpm tauri:dev`                                                 | §L / runtime evidence |

### F.1 Core fixture plan

Core tests should build a minimal workspace graph:

```rust
fn fixture_workspace() -> WorkspaceId;
fn fixture_ai_pane(workspace_id: &str, title: &str) -> PaneId;
fn fixture_runner_pane(workspace_id: &str, command: &str) -> PaneId;
fn fixture_shell_pane(workspace_id: &str, command: &str) -> PaneId;
fn fixture_other_workspace_pane() -> (WorkspaceId, PaneId);
fn fixture_failure(exit_code: i32, stderr: &str) -> PaneFailureEvent;
```

Tests must not depend on the user's real workspace, shell, AI CLI, or compiler. If a test needs parser data, it must read versioned fixture copied from [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) deliverables.

### F.2 Regression proof

- Temporarily force `PaneLinkRequest.parent_pane_id` to accept a non-AI pane in test-only code and verify `invalid_parent_pane_type_is_rejected` fails.
- Temporarily bypass `sanitize_ai_prompt` in the prompt fragment builder and verify secret / OSC52 tests fail.
- Temporarily remove `workspace_id` from the DB unique index and verify duplicate / cross workspace tests fail.

These proof steps belong in implementation PR notes or runtime evidence; this spec PR only records the required proof path.

### F.3 Fixture catalog

| Fixture                       | Purpose                    | Required fields                                 |
| ----------------------------- | -------------------------- | ----------------------------------------------- |
| `pane_link_same_workspace()`  | happy path link            | one AI parent, one Runner child, same workspace |
| `pane_link_cross_workspace()` | boundary rejection         | parent and child in different workspaces        |
| `pane_link_duplicate()`       | idempotency                | existing enabled row with same unique tuple     |
| `pane_link_stale_child()`     | missing child lifecycle    | deleted child pane row + active link            |
| `pane_failure_rustc()`        | parsed issue normalization | file, line, column, error code, message         |
| `pane_failure_vitest()`       | JS test output             | test name, assertion summary, file path         |
| `pane_failure_pytest()`       | Python traceback           | file stack, assertion line, short error         |
| `pane_failure_ansi_json()`    | hard parser case           | mixed ANSI and JSON fragments                   |
| `pane_failure_secret()`       | sanitization               | fake token, URL, environment-looking value      |
| `pane_failure_osc52()`        | control sequence stripping | OSC52 payload and normal text                   |

### F.4 Runtime evidence checklist

Runtime evidence must include raw command output and visual proof:

- `cargo test --workspace` tail with exit code.
- `pnpm lint` and `pnpm typecheck` output with exit code.
- `pnpm -C web exec vitest run tests/panels/Terminal/pane-link` output with exit code.
- Playwright trace or screenshot for link create.
- Playwright trace or screenshot for failure callout.
- Manual dev mode screenshot for raw fallback.
- Manual dev mode screenshot for cross workspace denial.

## §G. 数据模型

MVP-18 adds a dedicated `pane_links` table. It does not modify [MVP-14](./MVP-14-pane-advanced-layout.md) LayoutNode, and it does not store prompt drafts.

```sql
CREATE TABLE pane_links (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  parent_pane_id TEXT NOT NULL,
  child_pane_id TEXT NOT NULL,
  link_kind TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  fallback_mode TEXT NOT NULL DEFAULT 'structured',
  created_by TEXT NOT NULL DEFAULT 'user',
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  last_triggered_at INTEGER,
  deleted_at INTEGER,
  UNIQUE(workspace_id, parent_pane_id, child_pane_id, link_kind)
);

CREATE INDEX idx_pane_links_workspace
  ON pane_links(workspace_id, enabled, deleted_at);

CREATE INDEX idx_pane_links_child
  ON pane_links(workspace_id, child_pane_id, enabled, deleted_at);

CREATE INDEX idx_pane_links_parent
  ON pane_links(workspace_id, parent_pane_id, enabled, deleted_at);
```

### G.1 Column semantics

| Column              | Type    | Semantics                                                                |
| ------------------- | ------- | ------------------------------------------------------------------------ |
| `id`                | TEXT    | UUID v4 or repo-local id; never derived from pane ids                    |
| `workspace_id`      | TEXT    | hard boundary for all link operations                                    |
| `parent_pane_id`    | TEXT    | AI Pane that receives failure context                                    |
| `child_pane_id`     | TEXT    | Runner / Watch / Log / Build Pane that emits failure                     |
| `link_kind`         | TEXT    | first version supports `failureFeedback`; future kinds require migration |
| `enabled`           | INTEGER | 1 active, 0 paused                                                       |
| `fallback_mode`     | TEXT    | `structured`, `rawText`, `disabledByParser`                              |
| `created_by`        | TEXT    | `user`, `preset`, `migration`; MVP-18 uses `user`                        |
| `created_at`        | INTEGER | unix millis                                                              |
| `updated_at`        | INTEGER | unix millis                                                              |
| `last_triggered_at` | INTEGER | nullable unix millis for UI sorting                                      |
| `deleted_at`        | INTEGER | soft delete timestamp                                                    |

### G.2 Migration strategy

- Migration uses `PRAGMA user_version` and runs in a single transaction, matching the safety pattern from [SPIKE-04 B.3](./SPIKE-04-storage-benchmark.md).
- New table creation is additive and must be idempotent.
- Existing databases with no `pane_links` table migrate to an empty table; no synthetic links are created.
- If migration fails halfway, startup must leave the old DB usable and surface a user-readable error.
- A follow-up [SPIKE-04.5](./SPIKE-04.5-rusqlite-safety-verification.md) style manifest check is recommended if future implementation adds bulk import/export of links.

### G.3 Retention and privacy

`pane_links` stores metadata only. It must not store raw compiler output, AI prompt text, transcript text, token counts, or secret redaction samples. Failure callouts are transient frontend state; structured issues may be cached only if a later spec adds an explicit diagnostics history table.

## §H. 决策表

| Decision                                                   | Status                 | Owner                       | Rationale                                                                                                                   | Implementation note                                |
| ---------------------------------------------------------- | ---------------------- | --------------------------- | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| H.1 MVP-18 spec can be detailed before SPIKE-07 finishes   | Accepted for spec only | Codex CLI self-review       | Spec defines contract and gates; it does not claim parser feasibility                                                       | This PR keeps `status: draft`                      |
| H.2 MVP-18 implementation requires SPIKE-07 PASS           | Pending SPIKE-07       | Arbiter + main agent        | R1 remains blocking until parser evidence exists                                                                            | Implementation PR must cite SPIKE-07 report        |
| H.3 留 SPIKE-07 决策                                       | Pending                | OpenCode SPIKE-07 + Arbiter | Parser accuracy, supported CLI kinds, and fallback thresholds are not invented here                                         | §E C.4 forbids fabricated numbers                  |
| H.4 User confirmation before AI send                       | Accepted               | MVP-18 implementer          | Prevents parser mistakes and prompt injection from becoming automatic AI actions                                            | `Insert` writes draft only                         |
| H.5 Same-workspace-only links                              | Accepted               | MVP-18 implementer          | Avoids context leaks and mental model confusion                                                                             | DB unique index includes `workspace_id`            |
| H.6 Raw text fallback is allowed but not a SPIKE-07 bypass | Accepted               | Reviewer gate               | Fallback is runtime resilience, not a replacement for parser feasibility                                                    | If SPIKE-07 fails, no implementation               |
| H.7 No LayoutNode schema change                            | Accepted               | MVP-18 implementer          | Pane identity already exists; link graph is separate relationship data                                                      | New `pane_links` table only                        |
| H.8 No public positioning copy                             | Accepted               | All agents                  | [ADR-009](../adr/ADR-009-ai-aware-v1-vision.md) and `implementation-plan.md §1.1` keep this internal until real v1.0 launch | Public copy uses `v1.0 vision feature X` if needed |

## §I. 实施 Phase 拆分

| Phase                            | Estimate | Scope                                                                                                   | Acceptance subset                | Exit gate                                     |
| -------------------------------- | -------: | ------------------------------------------------------------------------------------------------------- | -------------------------------- | --------------------------------------------- |
| A · backend IPC + DB             |       4d | `pane_links` migration, core validator, `pane:link`, unlink/list commands, `pane:linked` events         | §E A/B/F, §G, §K command structs | `cargo test --workspace` + integration tests  |
| B · frontend subscription store  |       3d | Solid store for link graph, header chip, source badge, manage links popover, stale state                | §D D.1/D.2/D.4, §E B/F/H         | `pnpm lint` + `pnpm typecheck` + Vitest       |
| C · failure feedback wire        |       5d | child failure pipeline, parser bridge, `pane:trigger`, `pane:build-failed`, callout, insert/dismiss/raw | §E C/D/E/G                       | parser fixture tests + E2E                    |
| D · runtime evidence + hardening |       3d | performance bench, fallback proof, a11y pass, cross-platform smoke, docs evidence                       | §E G/H/I, §L                     | runtime evidence folder + PR body raw outputs |

### I.1 Phase A file ownership

Expected backend files:

- `crates/core/src/pane_links.rs` for pure data types and validation.
- `crates/core/src/db/migrations/*` or existing migration carrier for `pane_links`.
- `crates/app/src/lib.rs` for Tauri command registration.
- `crates/app/build.rs` for ts-rs export.
- `web/src/bindings/*` generated by `cargo build -p vibestation-app`.

### I.2 Phase B file ownership

Expected frontend files:

- `web/src/stores/paneLinks.ts` for link graph store.
- `web/src/panels/Terminal/PaneHeader.tsx` or equivalent header component for chip/badge.
- `web/src/panels/Terminal/PaneLinkPopover.tsx` for create/manage UI.
- Tests under `web/tests/panels/Terminal/pane-linking/`.

### I.3 Phase C file ownership

Expected parser and feedback files:

- `crates/core/src/parser_bridge.rs` or SPIKE-07 parser crate integration point.
- `crates/app/src/pane_failure.rs` for failure event plumbing.
- `web/src/panels/Terminal/PaneFailureCallout.tsx`.
- E2E under `web/tests/e2e/pane-linking.spec.ts`.

### I.4 Phase D evidence

Runtime evidence should live under:

```text
docs/runtime-evidence/mvp-18/
├── README.md
├── 01-link-create.png
├── 02-child-badge.png
├── 03-failure-callout.png
├── 04-raw-fallback.png
├── 05-cross-workspace-denied.png
└── pane-linking-performance.raw.log
```

### I.5 Phase implementation checklist

Phase A implementer must:

- Re-read [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) PASS report before writing parser bridge code.
- Confirm `pane_links` migration uses current rusqlite migration carrier.
- Add ts-rs exports before touching frontend imports.
- Add backend errors as tagged enums, not free-form strings.
- Prove duplicate link insertion is idempotent.

Phase B implementer must:

- Use generated binding types only.
- Keep store selectors scoped by `workspace_id`.
- Render chip and badge without mounting heavy Pane body dependencies.
- Cover missing child and disabled link states.
- Keep all visible labels keyboard and screen-reader accessible.

Phase C implementer must:

- Treat parser output as untrusted input.
- Run sanitizer before preview and before insert.
- Separate parser timeout from parser failure in telemetry-free local logs.
- Preserve user's existing draft unless they confirm append.
- Add fallback evidence for unsupported compiler output.

Phase D implementer must:

- Run dev mode on macOS and Linux if available.
- Capture runtime evidence before marking checklist done.
- Include raw outputs in PR body.
- Document any skipped Windows verification with reason.
- Verify no public docs accidentally picked up concrete capability wording.

## §J. 风险表

| Risk                            | Severity        | Description                                                                                              | Mitigation                                                                                                              | Gate                                        |
| ------------------------------- | --------------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| R1 parser instability           | High / High     | CLI and compiler output cannot be parsed reliably, causing bad `parsed_issues`                           | [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) must PASS before implementation; raw fallback only handles runtime misses | Block implementation                        |
| R2 prompt injection             | High / High     | Failure logs can contain malicious text, control chars, secret-looking values, or instructions to the AI | sanitize, redaction, no auto-send, raw preview, 8K limit                                                                | Security tests + external audit before v1.0 |
| R3 cross-workspace context leak | Medium / High   | User accidentally links unrelated workspaces and leaks project context into another AI Pane              | hard workspace_id guard in validator and DB queries                                                                     | Unit + integration tests                    |
| R4 UI overload                  | Medium / Medium | Frequent watch failures can spam AI Pane and make the terminal unusable                                  | dedupe by failure hash, backlog cap, dismiss, disable link                                                              | E2E + stress bench                          |
| R5 stale pane lifecycle         | Medium / Medium | Pane close/detach/reopen leaves links pointing to missing panes                                          | stale state, soft delete, cleanup command, no implicit id reuse                                                         | Lifecycle integration tests                 |
| R6 parser timeout               | Medium / Medium | Parser cold start or pathological output delays feedback                                                 | 2s timeout, immediate “parsing” state, raw fallback                                                                     | Performance bench                           |
| R7 data migration               | Low / High      | `pane_links` migration corrupts existing DB or silently overwrites data                                  | additive table, transaction, `PRAGMA user_version`, startup assertion                                                   | Migration tests                             |

## §K. IPC contract

All structs / enums must be Rust source-of-truth with `serde` + `ts_rs::TS`, exported through `crates/app/build.rs`. Frontend must import generated bindings from `web/src/bindings/*`; no parallel handwritten TypeScript interfaces.

### K.1 Commands

| Command                       | Request                     | Response                   | Notes                                             |
| ----------------------------- | --------------------------- | -------------------------- | ------------------------------------------------- |
| `pane:link`                   | `PaneLinkRequest`           | `PaneLinkResult`           | create or return existing link                    |
| `pane:unlink`                 | `PaneUnlinkRequest`         | `PaneUnlinkResult`         | disable or soft delete link                       |
| `pane:links:list`             | `PaneLinksListRequest`      | `PaneLinksListResult`      | list current workspace links                      |
| `pane:links:set_enabled`      | `PaneLinkSetEnabledRequest` | `PaneLinkResult`           | pause / resume link                               |
| `pane:failure:preview_prompt` | `PaneFailurePreviewRequest` | `PaneFailurePreviewResult` | build sanitized prompt fragment without inserting |

### K.2 Events

| Event               | Payload                | Emitted by                  | Semantics                                           |
| ------------------- | ---------------------- | --------------------------- | --------------------------------------------------- |
| `pane:linked`       | `PaneLinkedEvent`      | app backend                 | link created / enabled / disabled / stale / removed |
| `pane:trigger`      | `PaneTriggerEvent`     | app backend                 | child Pane produced a failure trigger               |
| `pane:build-failed` | `PaneBuildFailedEvent` | parser bridge / app backend | parsed or fallback failure ready for AI Pane        |
| `pane:link-error`   | `PaneLinkErrorEvent`   | app backend                 | recoverable create/unlink/trigger error             |

### K.3 Binding list

Expected generated binding files:

1. `PaneLink.ts`
2. `PaneLinkKind.ts`
3. `PaneLinkStatus.ts`
4. `PaneLinkRequest.ts`
5. `PaneLinkResult.ts`
6. `PaneUnlinkRequest.ts`
7. `PaneUnlinkResult.ts`
8. `PaneLinksListRequest.ts`
9. `PaneLinksListResult.ts`
10. `PaneLinkSetEnabledRequest.ts`
11. `PaneLinkedEvent.ts`
12. `PaneTriggerEvent.ts`
13. `PaneBuildFailedEvent.ts`
14. `PaneFailurePreviewRequest.ts`
15. `PaneFailurePreviewResult.ts`
16. `ParsedIssue.ts`
17. `ParsedIssueSeverity.ts`
18. `PaneLinkError.ts`
19. `PaneLinkErrorEvent.ts`

### K.4 Payload sketch

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq, Eq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneLinkRequest {
    pub workspace_id: String,
    pub parent_pane_id: String,
    pub child_pane_id: String,
    pub link_kind: PaneLinkKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS, PartialEq)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaneBuildFailedEvent {
    pub workspace_id: String,
    pub link_id: String,
    pub parent_pane_id: String,
    pub child_pane_id: String,
    pub command_run_id: String,
    pub exit_code: Option<i32>,
    pub raw_excerpt: String,
    pub parsed_issues: Vec<ParsedIssue>,
    pub parser_confidence: f32,
    pub fallback_mode: PaneFailureFallbackMode,
    pub occurred_at: i64,
}
```

### K.5 Error contract

`PaneLinkError` must be a tagged enum with stable variants:

- `CrossWorkspaceDenied`
- `InvalidParentPaneType`
- `InvalidChildPaneType`
- `PaneNotFound`
- `LinkNotFound`
- `ParserUnavailable`
- `ParserTimeout`
- `PromptSanitizationFailed`
- `DbError`
- `UnsupportedCliKind`

Each error variant exposed to UI must include a user-readable `message` and a machine-readable `kind`.

### K.6 Event payload examples

```json
{
  "event": "pane:linked",
  "payload": {
    "workspaceId": "workspace-a",
    "linkId": "link-1",
    "parentPaneId": "pane-claude",
    "childPaneId": "pane-runner",
    "linkKind": "failureFeedback",
    "status": "enabled",
    "updatedAt": 1760000000000
  }
}
```

```json
{
  "event": "pane:trigger",
  "payload": {
    "workspaceId": "workspace-a",
    "childPaneId": "pane-runner",
    "commandRunId": "run-42",
    "reason": "exitCode",
    "exitCode": 101,
    "command": "cargo test",
    "occurredAt": 1760000000100
  }
}
```

```json
{
  "event": "pane:build-failed",
  "payload": {
    "workspaceId": "workspace-a",
    "linkId": "link-1",
    "parentPaneId": "pane-claude",
    "childPaneId": "pane-runner",
    "commandRunId": "run-42",
    "exitCode": 101,
    "rawExcerpt": "error[E0425]: cannot find value",
    "parsedIssues": [
      {
        "severity": "error",
        "file": "src/lib.rs",
        "line": 42,
        "column": 13,
        "message": "cannot find value `foo` in this scope"
      }
    ],
    "parserConfidence": 0.98,
    "fallbackMode": "structured",
    "occurredAt": 1760000000200
  }
}
```

These examples are illustrative payload shapes, not SPIKE-07 parser accuracy evidence.

## §L. 跨平台考量

| Platform | Considerations                                                                                                                                                                  |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS    | Keyboard shortcuts must avoid conflict with existing Pane split / maximize commands; screenshots for runtime evidence should include Retina scaling and reduced-motion check.   |
| macOS    | Shell commands often run under zsh; failure trigger must not depend on bash-only exit status formatting.                                                                        |
| Linux    | Ubuntu 24 Wayland / X11 differences affect focus and window events, but this MVP should operate inside existing main Webview; link state must not depend on native window APIs. |
| Linux    | Common compilers include gcc, rustc, go, pytest; parser fixtures should include ANSI color output from non-interactive terminals.                                               |
| Windows  | v1.0 may include Windows, but implementation must treat ConPTY output and CRLF paths as distinct parser input; no hardcoded `/` path assumptions in `ParsedIssue`.              |
| Windows  | Prompt sanitization must handle drive letters and UNC paths; system path denylist differs from macOS / Linux and should be platform-gated.                                      |

## §M. 自审四问

1. **递归完备性**：本 spec 从 link 创建、持久化、事件、parser bridge、AI callout、sanitize、unlink、stale lifecycle、runtime evidence 到 IPC binding 都有闭环；原占位 spec 中“订阅 + 反哺 + 降级 + unlink”语义被保留并扩展为可实施 contract。
2. **反向场景**：SPIKE-07 未通过、parser crash、parser timeout、跨 workspace、非 AI parent、非执行型 child、重复 link、Pane close、app quit、连续 failure spam、secret 命中、OSC52 / NUL 控制字符、detached Pane target 不可靠都在 acceptance 或风险表中有处理。
3. **边界适用性**：范围限定在同 workspace Pane 之间；不自动发送 AI prompt；不跨 workspace、不跨机器、不引入新 LayoutNode、不偷跑 parser、不对外宣传具体能力，避免 v1.0 vision 泄露到 MVP 或公开文案。
4. **YAGNI**：不做行为预测、不做 AI 自动修复、不做 AI-on-AI、不做 telemetry、不做第三方 CLI 泛化、不做 diagnostics history 表；只实现用户显式 link 与失败上下文候选插入。

## 详化完成度评估

| Required area     | Status | Notes                                                                                               |
| ----------------- | ------ | --------------------------------------------------------------------------------------------------- |
| frontmatter       | Done   | `status: draft` preserved; `reviewer: Codex CLI · self-review` added                                |
| §A Goal           | Done   | >100 words and includes 3 concrete business scenarios                                               |
| §B Context        | Done   | R1 / SPIKE-07 / strategic value clarified                                                           |
| §C Scope          | Done   | Do 17 items; Don't 12 items                                                                         |
| §D UI wireframe   | Done   | 5 core interactions described in text                                                               |
| §E Acceptance     | Done   | 48 checkboxes across dependency, link, parser, UX, security, lifecycle, performance, a11y, evidence |
| §F Test matrix    | Done   | unit / integration / E2E / performance / manual QA covered                                          |
| §G Data model     | Done   | full `pane_links` schema + migration strategy referencing SPIKE-04 B.3                              |
| §H Decision table | Done   | includes explicit “留 SPIKE-07 决策” pending row                                                    |
| §I Phase split    | Done   | A backend IPC / B subscription store / C feedback wire / D runtime evidence                         |
| §J Risk table     | Done   | R1-R7 with mitigation and gates                                                                     |
| §K IPC contract   | Done   | commands, events, binding list, payload sketch, error enum                                          |
| §L Cross-platform | Done   | macOS / Linux / Windows each has two considerations                                                 |
| §M Self-review    | Done   | Four questions answered concretely                                                                  |

**完成度**：13/13 = 100%（建议 Arbiter review 通过后，由 main agent 独立 commit 翻 `status: ready`）。

**遗留问题**：无详化 blocker；实施 blocker 仍是 [SPIKE-07](./SPIKE-07-cli-protocol-parser.md) PASS 与后续 ADR-011 / Arbiter greenlight。
