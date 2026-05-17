# Architecture Decision Records (ADR)

> 本目录存放 **Vibestation 的架构决策记录**。每条重大决策（=影响多模块 / 破坏向后兼容 / 跨 phase 影响 / 代价显著）必须对应一个 ADR。
> 格式参考 [MADR 4.0](https://adr.github.io/madr/)，中文化精简。

---

## 📂 ADR 索引

| ID                                                           | 标题                                                                                 | 状态         | 决策表 #                 | Spike / PR                                                                      |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------ | ------------ | ------------------------ | ------------------------------------------------------------------------------- |
| [ADR-001](./ADR-001-license-apache-2.0.md)                   | 许可证 = Apache License 2.0（不签 CLA）                                              | **accepted** | #1                       | 锁定 @ Phase 1                                                                  |
| [ADR-002](./ADR-002-mvp-scope-b-compromise.md)               | MVP 范围 = B 折中方案                                                                | **accepted** | #2                       | 锁定 @ Phase 1                                                                  |
| [ADR-003](./ADR-003-pty-architecture.md)                     | PTY 架构 = portable-pty + 共享读线程 + mpsc                                          | **accepted** | #15                      | [SPIKE-05.5](../spikes/SPIKE-05.5-report.md) accepted @ 2026-04-19              |
| [ADR-004](./ADR-004-frontend-stack.md)                       | 前端栈 = SolidJS + TypeScript + Vite + xterm.js                                      | **accepted** | #6                       | 锁定 @ Phase 1                                                                  |
| [ADR-005](./ADR-005-local-storage.md)                        | 本地存储 = rusqlite（redb 因坏库检测 FAIL superseded）                               | **accepted** | #14                      | [SPIKE-04.5](../spikes/SPIKE-04.5-report.md) accepted @ 2026-04-19              |
| [ADR-006](./ADR-006-desktop-framework.md)                    | 桌面框架 = Tauri 2（fallback: Electron 28+ · Ubuntu Phase B pending caveat）         | **accepted** | #19                      | macOS Phase A 强 PASS · SPIKE-01/02 Ubuntu validated · PR #50 @ 2026-04-19      |
| [ADR-007](./ADR-007-git-stack.md)                            | Git 栈 = git2 0.20（写）+ gix 0.70（读优化）                                         | **accepted** | #13                      | [SPIKE-03](../spikes/SPIKE-03-report.md) accepted @ 2026-04-19                  |
| [ADR-008](./ADR-008-diff-renderer-custom.md)                 | Diff 渲染 = 自建（非 Monaco）                                                        | **accepted** | #7                       | 锁定 @ Phase 1                                                                  |
| [ADR-009](./ADR-009-ai-aware-v1-vision.md)                   | AI-Aware Pane 联动 = v1.0 vision（对外不宣传）                                       | **accepted** | #3                       | 锁定 @ Phase 1                                                                  |
| [ADR-010](./ADR-010-cargo-workspace-2-crate.md)              | Cargo workspace = 2 crate（app + core）                                              | **accepted** | #5                       | 锁定 @ Phase 1                                                                  |
| [ADR-011](./ADR-011-runtime-evidence-location.md)            | Runtime evidence 路径锁 `docs/runtime-evidence/<task-id>/`                           | **accepted** | #18                      | Session 10 FU-2 · PR #44/#45 @ 2026-04-19                                       |
| [ADR-012](./ADR-012-v2d1-arbiter-approval-simplification.md) | v2-D → v2-D.1 · 单人项目 Arbiter approval 规则简化（删 24h 补 comment 硬要求）       | **accepted** | —（治理规则）            | Session 13 开场 · session 12 audit H1 根因 @ 2026-04-21                         |
| [ADR-013](./ADR-013-spike-cold-backup-degradation.md)        | Spike 冷备归档 v1 强制 → v2 推荐（22% 合规率实证 · 3 场景判断清单保留特殊情况）      | **accepted** | —（项目规则）            | Session 13 中 · audit M-1 根因 @ 2026-04-21                                     |
| [ADR-014](./ADR-014-ipc-contract-source-of-truth-ts-rs.md)   | IPC contract source of truth = Rust struct + ts-rs codegen（H2 根因消除）            | **accepted** | —（规范类 · 未在锁定表） | Session 13 中 · SPIKE-08 §A PASS @ 2026-04-20 + PR #63 rollout                  |
| [ADR-015](./ADR-015-telemetry-stack-sentry.md)               | Telemetry crash stack = Sentry SDK + sanitized payload                               | **accepted** | #10 实施子决策           | MVP-10 Phase B pre-spike @ 2026-04-25 · accepted @ 2026-04-26                   |
| [ADR-016](./ADR-016-admin-override-trailer-exemption.md)     | v2-D.1 → v2-D.2 · admin direct push trailer 豁免 + 审计 marker 要求                  | **accepted** | —（治理规则）            | Session 23 · session 21 7 个 admin push audit 项闭合 @ 2026-05-03               |
| [ADR-021](./ADR-021-ci-mandate-staleness.md)                 | CLAUDE.md「合入后 CI 验证」→「合入后质量门验证」· 承认 no-auto-CI 既定（方案 b）     | **accepted** | —（治理规则）            | proposed @ 2026-05-16 · accepted @ 2026-05-17 · session 33 · Arbiter tajiaoyezi |
| [ADR-022](./ADR-022-dispatch-template-ref-path-staleness.md) | dispatch 范本引用断链 · 文档不再承诺 git 路径（方案 d · 原 Context 经主 agent 证伪） | **accepted** | —（治理规则）            | proposed @ 2026-05-16 · accepted @ 2026-05-17 · session 33 · Arbiter tajiaoyezi |

---

## 📝 状态定义

| 状态              | 含义                                                                     |
| ----------------- | ------------------------------------------------------------------------ |
| **proposed**      | 决策草案 · 依赖 Spike 验证或 Arbiter 仲裁 · 当前 `CLAUDE.md` 决策表 B 栏 |
| **accepted**      | 已锁定 · 当前 `CLAUDE.md` 决策表 A 栏 · 除非写新 ADR 推翻否则不得讨论    |
| **rejected**      | 曾考虑但拒绝 · 保留做历史记录                                            |
| **deprecated**    | 曾 accepted · 后续因新决策不再适用 · 通常指向 superseded by              |
| **superseded by** | 被新 ADR 取代 · 见链接                                                   |

**重要**：`proposed → accepted` 的触发是"Spike 通过 + 独立评审 + 用户拍板"三件套。缺一不可。

---

## 📊 ADR Status Timeline（按 phase 分组）

| ADR     | 标题摘要                | 锁定 phase       | 关联 Spike     | accepted 日期 | PR         |
| ------- | ----------------------- | ---------------- | -------------- | ------------- | ---------- |
| ADR-001 | 许可证 Apache 2.0       | Phase 1 设计阶段 | —              | 2026-04-18    | 初始建仓   |
| ADR-002 | MVP 范围 B 折中         | Phase 1 设计阶段 | —              | 2026-04-18    | 初始建仓   |
| ADR-004 | 前端栈 SolidJS          | Phase 1 设计阶段 | —              | 2026-04-18    | 初始建仓   |
| ADR-008 | Diff 自建渲染器         | Phase 1 设计阶段 | —              | 2026-04-18    | 初始建仓   |
| ADR-009 | AI-Aware v1.0 vision    | Phase 1 设计阶段 | —              | 2026-04-18    | 初始建仓   |
| ADR-010 | Cargo workspace 2 crate | Phase 1 设计阶段 | —              | 2026-04-18    | 初始建仓   |
| ADR-003 | PTY portable-pty        | Spike W0 后锁定  | SPIKE-05.5     | 2026-04-19    | PR #50     |
| ADR-005 | 本地存储 rusqlite       | Spike W0 后锁定  | SPIKE-04.5     | 2026-04-19    | PR #50     |
| ADR-007 | Git 栈 git2+gix         | Spike W0 后锁定  | SPIKE-03       | 2026-04-19    | PR #50     |
| ADR-011 | Runtime evidence 路径   | Session 10 FU-2  | —              | 2026-04-19    | PR #44/#45 |
| ADR-006 | 桌面框架 Tauri 2        | Session 10 末    | SPIKE-01/02    | 2026-04-19    | PR #50     |
| ADR-012 | v2-D.1 approval 简化    | 治理升级 S13     | —              | 2026-04-21    | session 13 |
| ADR-013 | Spike 冷备归档降级      | 治理升级 S13     | —              | 2026-04-21    | session 13 |
| ADR-014 | IPC contract ts-rs      | 治理升级 S13     | SPIKE-08       | 2026-04-21    | PR #63     |
| ADR-015 | Sentry telemetry        | MVP 子决策 S20   | MVP-10 Phase B | 2026-04-26    | PR #152    |
| ADR-016 | v2-D.2 admin override   | 治理升级 S23     | —              | 2026-05-03    | PR #218    |
| ADR-021 | CI mandate → 质量门     | 治理对齐 S33     | —              | 2026-05-17    | session 33 |
| ADR-022 | dispatch 范本去断链     | 治理对齐 S33     | —              | 2026-05-17    | session 33 |

**当前统计**：19 accepted · 1 superseded（ADR-017 → ADR-018）· 0 proposed · 0 rejected · 0 deprecated（共 20 · session 33 ADR-021/022 flip 后实测核准 · 原「16 accepted · 0 proposed」双重 stale 同步修正）。

---

## 🔢 决策表 # 反查（CLAUDE.md A/B/C 栏 → ADR）

| 决策表 # | 决策内容摘要                        | 栏位 | ADR 链接                                                 |
| -------- | ----------------------------------- | ---- | -------------------------------------------------------- |
| #1       | 许可证 = Apache 2.0（不签 CLA）     | A    | [ADR-001](./ADR-001-license-apache-2.0.md)               |
| #2       | MVP 范围 = B 折中（砍 push/rebase） | A    | [ADR-002](./ADR-002-mvp-scope-b-compromise.md)           |
| #3       | AI-Aware Pane = v1.0 vision         | A    | [ADR-009](./ADR-009-ai-aware-v1-vision.md)               |
| #4       | 视觉方向 = Calm Studio              | A    | —（锁定 @ `design/directions/1-calm-studio.html`）       |
| #5       | Cargo workspace = 2 crate           | A    | [ADR-010](./ADR-010-cargo-workspace-2-crate.md)          |
| #6       | 前端栈 = SolidJS + TS + xterm.js    | A    | [ADR-004](./ADR-004-frontend-stack.md)                   |
| #7       | Diff 渲染 = 自建（非 Monaco）       | A    | [ADR-008](./ADR-008-diff-renderer-custom.md)             |
| #8       | 平台 MVP = macOS + Ubuntu 24        | A    | —（锁定 @ `implementation-plan.md §3.1`）                |
| #9       | Tool Windows 默认状态               | A    | —（锁定 @ 原型 `1-calm-studio.html` JS DEFAULT_STATE）   |
| #10      | Telemetry = 默认关 + opt-in         | A    | [ADR-015](./ADR-015-telemetry-stack-sentry.md)（子决策） |
| #11      | Landing page = Astro + 自建动效     | A    | —（锁定 @ `implementation-plan.md §12`）                 |
| #13      | Git 栈 = git2（写）+ gix（读）      | A    | [ADR-007](./ADR-007-git-stack.md)                        |
| #14      | 本地存储 = rusqlite + r2d2          | A    | [ADR-005](./ADR-005-local-storage.md)                    |
| #15      | PTY = portable-pty + shared reader  | A    | [ADR-003](./ADR-003-pty-architecture.md)                 |
| #16      | 项目域名 TLD                        | C    | —（时间锁定结果开放 · W10 附近决定 · 不建 ADR）          |
| #17      | Logo 最终定稿                       | C    | —（时间锁定结果开放 · v0.1 发布前决定 · 不建 ADR）       |
| #18      | Runtime evidence 路径               | A    | [ADR-011](./ADR-011-runtime-evidence-location.md)        |
| #19      | 桌面框架 = Tauri 2                  | A    | [ADR-006](./ADR-006-desktop-framework.md)                |
| #20      | Branch protection 机械化            | A    | —（锁定 @ `.githooks/pre-push` · 无独立 ADR）            |

**B 栏当前空**：session 10 末 ADR-006（原 B 栏 #12）升级到 A 栏 #19 · B 栏 header 保留作未来类似决策载体。

---

## 🔗 SPIKE / Session 关联追踪

每个 accepted ADR 的触发溯源（Spike → Session → PR 翻转链路）：

| ADR     | 触发来源                                                                                    | 关键 session            | PR 翻转                        |
| ------- | ------------------------------------------------------------------------------------------- | ----------------------- | ------------------------------ |
| ADR-001 | Phase 1 设计决策                                                                            | Session 3-4             | 初始建仓                       |
| ADR-002 | Phase 1 设计决策                                                                            | Session 3-4             | 初始建仓                       |
| ADR-003 | SPIKE-05 HOL/boundedness PASS · SPIKE-05.5 visible throughput FAIL → shared reader 不是瓶颈 | Session 10              | PR #50                         |
| ADR-004 | Phase 1 设计决策                                                                            | Session 3-4             | 初始建仓                       |
| ADR-005 | SPIKE-04 redb B.2 坏库 FAIL → rusqlite · SPIKE-04.5 B.1-5 全过                              | Session 10              | PR #50                         |
| ADR-006 | SPIKE-01/02 macOS Phase A 强 PASS · SPIKE-01/02 Phase B Ubuntu validated (PR #137-139)      | Session 10 → Session 19 | PR #50 · caveat 解除 PR #138   |
| ADR-007 | SPIKE-03 gix log 100 warm P99 12.65ms vs git2 24964ms（1973× 快）                           | Session 10              | PR #50                         |
| ADR-008 | Phase 1 设计决策                                                                            | Session 3-4             | 初始建仓                       |
| ADR-009 | Phase 1 设计决策（AI-Aware v1.0 vision 对外宣传禁区）                                       | Session 3-4             | 初始建仓                       |
| ADR-010 | Phase 1 设计决策                                                                            | Session 3-4             | 初始建仓                       |
| ADR-011 | Session 10 FU-2 · runtime evidence 路径规范化 · Arbiter 选项 A                              | Session 10              | PR #44/#45                     |
| ADR-012 | Session 12 audit H1 根因 · v2-D "24h 补 comment" 12/12 PR 零执行                            | Session 13              | session 13 开场 PR             |
| ADR-013 | Session 13 audit M-1 根因 · Spike 冷备 22% 合规率实证                                       | Session 13              | session 13 中 PR               |
| ADR-014 | SPIKE-08 §A ts-rs PASS · session 13 audit X-4 · H2 compile-time drift 前移                  | Session 13              | PR #63 rollout                 |
| ADR-015 | MVP-10 Phase B pre-spike · Sentry SDK sanitized payload 4 步验证通过                        | Session 20              | PR #152 · Arbiter @ 2026-04-26 |
| ADR-016 | Session 21 GitHub Actions billing 暂停 · 7 个 admin direct push 触发治理盲区                | Session 23              | PR #218                        |
| ADR-021 | session 32 续 8 PR merge 零 auto CI · ci.yml 仅 workflow_dispatch · mandate 与现实漂移      | Session 32→33           | session 33 · Arbiter 方案 b    |
| ADR-022 | PR #329 压缩遗留 + spike-tmp/ 整个 gitignored · dispatch 范本引用断链（原 Context 经证伪）  | Session 32→33           | session 33 · Arbiter 方案 d    |

---

## 🚀 新增 ADR 的流程

当出现一个**新决策**（同时满足：影响多模块 OR 破坏向后兼容 OR 跨 phase）时：

```bash
# 1. 从模板创建新文件
cp docs/adr/_template.md docs/adr/ADR-017-<slug>.md

# 2. 填写 ADR（见下方 section 结构）
#    - 必须有"考虑的选项"（至少 2 个）
#    - 必须有"后果"（正面 + 负面都列）

# 3. 开 PR（conventional commit + trailer）
git checkout -b docs/adr-017-<slug>
git add docs/adr/ADR-017-<slug>.md docs/adr/README.md
git commit -m "docs(adr): 新增 ADR-017 <中文标题>

Co-authored-by: <Agent Name> <email>"
gh pr create --title "docs(adr): 新增 ADR-017 <标题>"

# 4. 独立评审（≠ 原作者）· 确认 Spike / 讨论依据充分 · 过"翻转 gate"
# 5. merge 后更新 CLAUDE.md 决策表（若对应 B 栏 → A 栏）
```

---

## 📝 ADR Section 结构（MADR 4.0 中文化）

每个 ADR 必含以下章节，顺序固定：

```markdown
# ADR-NNN: <中文标题>

**状态**：proposed | accepted | rejected | deprecated | superseded by [ADR-XXX]
**日期**：YYYY-MM-DD
**决策者**：<作者 agent-id + 独立评审者 + 用户>
**对应 `CLAUDE.md` 决策表**：# N
**对应 Spike**：[SPIKE-NN](../tasks/SPIKE-NN-<slug>.md)（若 Spike 依赖）

## 背景与问题（Context and Problem Statement）

## 决策驱动因素（Decision Drivers）

## 考虑的选项（Considered Options）

## 决策（Decision Outcome）

## 后果（Consequences）

### 正面

### 负面

### 风险

## 与 `implementation-plan.md` 的映射

## 相关（Links）
```

---

## 🔮 未来 ADR 占位（触发条件驱动）

以下方向尚未开 ADR，满足触发条件时新建。

> **编号空间现状**（session 33 · 2026-05-17 · 防 [#351 类 ADR 撞号](#编号撞号教训)复发 · 含 ADR-020 作废）：已建 ADR = **001–018 + 021 + 022**（见上方状态表 · 共 20 文件）· 仍有效预留（号未占用 · 方向未建）= **ADR-019** · 新方向预留从 **ADR-023** 起顺延。
>
> 🪦 **ADR-020 = 作废占位**（session 33 · 2026-05-17 · Arbiter tajiaoyezi 拍板 · 调研见本次 PR）：原占位"v1.0 AI-Aware session 存储方案"经调研确认 = 无技术 scope 的宽口径流程占位 · 其字面想锁的存储方案已被 **ADR-005（存储引擎 rusqlite）+ MVP-18/19 spec §G（pane_links / ai_sessions / session_commit_links 表结构 · migration · 保留 · 脱敏 · 边界算法 · Phase A 已 merged）+ ADR-009/018（AI-Aware vision + R1 greenlight）+ SPIKE-07/07.5（边界识别 parser/IR 前提）** 分布式完全覆盖 · **零未锁定架构决策 gap**（唯一开放项 = MVP-19 H7 idle cutoff 默认值微调 · 参数调优非 ADR 级 · owner 已是 Arbiter+implementer）。**ADR-020 号 tombstone · 永不复用**（保留历史语境 · 防未来无关决策撞用与 AI-Aware 存储历史混淆 · 同 rejected/superseded 号不复用惯例）。
>
> ⚠️ 原占位表曾把"域名 TLD / Logo"预留在 ADR-017 / 018，但 **017 / 018 已被 AI-Aware 决策实际复用**（ADR-017 ai-aware-deferred → superseded by 018 · ADR-018 ai-aware-r1-rejudge accepted @ 2026-05-16）· 该两方向预留号已下移到 023 / 024。
>
> <a id="编号撞号教训"></a>**dispatch 定 ADR 号铁律**：必须取「已建文件 max ∪ 本表所有预留号 ∪ 作废 tombstone 号（020）」之外的下一可用号（#351 事件：Grok dispatch 凭 `ls ADR-*.md` 推断用 019/020 → 撞本表预留 → §2.14 BLOCK · 改用 021/022 才合规）。`ls` 只反映已建文件 max · **不反映本表触发条件驱动的预留号 / tombstone 号** · 都要查。

| 候选 ADR | 方向                                     | 触发条件                                                                                                  |
| -------- | ---------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| ADR-019  | 升级 GitHub Pro + branch protection 启用 | 仓库变 public 或第二位真合作者加入 → v2-strict 治理触发 · 详见 CLAUDE.md §(2)                             |
| ADR-023  | 域名 TLD 选定（`.app` / `.dev` / `.io`） | CLAUDE.md C 栏 #16 到期 · W10 附近 · Arbiter 拍板后新建（原占位号 017 已被 AI-Aware 复用 · 号下移至 023） |
| ADR-024  | Logo 最终定稿方案                        | CLAUDE.md C 栏 #17 到期 · v0.1 发布前 · 设计定稿后新建（原占位号 018 已被 AI-Aware 复用 · 号下移至 024）  |

---

## 🔗 与其他治理文档的关系

- `CLAUDE.md §决策状态表`：**快速索引 + 锁定状态** · 本目录 16 个 ADR 覆盖其中大部分条款（见上方决策表 # 反查）· 其余条款仍由 `implementation-plan.md` / 原型文件锁定
- `implementation-plan.md`：**战略依据** · ADR 的"背景与问题"通常引用此文具体章节
- `docs/tasks/SPIKE-*`：**验证依据** · `proposed` ADR 依赖 Spike 结论 · Spike 通过后更新 ADR 状态
- `docs/tasks/README.md`：**任务流程** · 实施 ADR 决策的具体 task spec

---

## 📊 ADR 覆盖范围（当前精确统计 · 16 accepted · 0 proposed）

**A 档（11 条 accepted · 对应 `CLAUDE.md` 决策表）**：

- Phase 1 锁定 6 条：`#1` License · `#2` MVP 范围 · `#3` AI-Aware v1.0 vision · `#5` Cargo workspace · `#6` 前端栈 · `#7` Diff 自建
- Session 8-10 Spike 通过 3 条：`#13` Git 栈（SPIKE-03）· `#14` 本地存储 rusqlite（SPIKE-04.5）· `#15` PTY 方案（SPIKE-05.5）
- Session 10 FU-2 新增 1 条：`#18` Runtime evidence 路径锁（PR #44/#45）
- Session 10 末 B→A 升级 1 条：`#19` 桌面框架 Tauri 2（macOS Phase A 强 PASS · Ubuntu Phase B validated · PR #50）

**A 档未 ADR 化（5 条 · 已在其他文档锁定）**：

- `#4` Calm Studio 视觉方向 → `design/directions/1-calm-studio.html`
- `#8` 平台 MVP = macOS + Ubuntu → `implementation-plan.md §3.1`
- `#9` Tool Windows 默认状态 → 原型 JS `DEFAULT_STATE`
- `#10` Telemetry 默认关 + opt-in 语义 → `implementation-plan.md §5.1` + R30；实现子决策见 ADR-015
- `#11` Landing page = Astro → `implementation-plan.md §12`
- `#20` Branch protection 机械化 → `.githooks/pre-push` + `package.json prepare`

**治理 / 规范 / 项目规则类（6 条 accepted · 不在决策表）**：ADR-012 · ADR-013 · ADR-014 · ADR-016 · ADR-021（CI mandate → 质量门）· ADR-022（dispatch 范本去断链）

**MVP 实施子决策（1 条 accepted）**：ADR-015（Sentry telemetry stack · session 20 解锁 SDK 编码）

**B 栏当前空**：原 #12 桌面框架 session 10 末升级到 A 栏 #19 · B 栏 header 保留作未来类似决策载体。

**C 档（2 条 · 时间锁定结果开放）**：不建 ADR · 由时间节点触发决策后直接更新 `CLAUDE.md` 决策表。

- `#16` 项目域名 TLD（W10 附近）
- `#17` Logo 最终定稿（v0.1 发布前）

---

_本目录 Phase 3 建立（2026-04-18）· 当前 16 个 ADR（16 accepted · 0 proposed）· 最后更新 2026-05-14 by Droid。_
