# Session History · 开发 session 历史 / PR 复盘 / 关键讨论归档

> 本目录存放**关键开发 session 的反思记录 · PR close / revert 的复盘 · 重大讨论的沉淀**。
> Phase 3 建立（2026-04-18）· 随项目演进填充。

---

## 📂 命名约定

```
docs/session-history/
├── README.md                                (本文件)
├── YYYY-MM-DD-<topic-slug>.md               复盘 / 讨论记录
└── pr-NN-<result>.md                        PR close / revert / 复杂讨论
```

**示例**：

- `2026-04-18-phase-2-codex-five-rounds-retrospective.md`（Phase 2 Codex 5 轮对抗性审查复盘）
- `pr-4-close-why-per-task-files.md`（PR #4 close · 为什么改 per-task 报告而不是共享文件）
- `2026-05-15-spike-02-tauri-wayland-issue.md`（若 SPIKE-02 遇 Wayland bug 的解决记录）

---

## 📝 每份文档的推荐结构

```markdown
# <标题>

**日期**：YYYY-MM-DD
**参与者**：<作者 agent-id> · <evaluator> · <用户>
**类型**：PR close / PR revert / Spike 复盘 / 架构决策讨论 / Codex 审查复盘 / 其他

## 背景

<!-- 为什么写这份记录 · 对应哪个事件 -->

## 事实还原

<!-- 按时间线写发生了什么 · 客观事实 -->

## 根因分析

<!-- 为什么会发生 · 深层原因 -->

## 教训 / 改进措施

<!-- 具体可执行的改进 -->

### 通用层（写全局 rule）

<!-- 通用行为规则 · 写到 ~/.claude/rules/ · 避免再遇同类问题 -->

### 项目特有（写本项目 memory）

<!-- 项目特有案例 / 反面样本 · 写到 project memory -->

## 后续动作

- [ ] 更新 `CLAUDE.md` 决策表 / `implementation-plan.md` / ADR 等（如需要）
- [ ] 开对应 PR · 引用本记录

## 相关

- PR：#N
- ADR：ADR-NNN
- 前置讨论：...
```

---

## 🔄 什么时候写 session-history

**必须写**：

- **PR close / revert**（不是 merge · 需要解释为什么不走了）
- **关键架构讨论**（若 ADR 不适合承载完整过程 · 比如多轮评审迭代）
- **Spike 失败**（触发 fallback 时需要留详细根因）
- **Codex / 其他 AI 对抗性审查的整轮复盘**（每 5-10 轮写一次总结）

**建议写**：

- 某个 PR 讨论超过 10 条评论 · 记录关键决策点
- 多 agent 并发出现冲突 · 记录仲裁过程

**不需要写**：

- 常规 PR（按 Conventional Commits + task spec 已够）
- 日常 bug 修复（走 Issues / PR · 无需单独记录）

---

## 📂 已归档 Session

| 文件                           | 日期       | Session | PR 范围   | 主题                                                                                                                           |
| ------------------------------ | ---------- | ------- | --------- | ------------------------------------------------------------------------------------------------------------------------------ |
| [session-27.md](session-27.md) | 2026-05-10 | 27      | #264-#266 | v0.3 sprint phase A+B+C 全收 + Phase D 启动 · 3-track 模式实证 · bench-only PR 模式开启                                        |
| [session-26.md](session-26.md) | 2026-05-09 | 26      | #259-#263 | 4-track 文件域隔离首次实证 · 单 day 4 PR concurrent · v0.3 sprint phase B+C 大跃进 · OpenCode §2.10 第 2 次 → N=3 永久转出条款 |
| [session-25.md](session-25.md) | 2026-05-07 | 25      | #251-#253 | v0.3 sprint phase A 50% 启动 · MVP-15/16 Phase A · 主 agent reviewer 翻转 gate (a) 实战 · OpenCode 谎报 lint/typecheck 首次    |
| [session-24.md](session-24.md) | 2026-05-04 | 24      | spec only | v0.3 sprint kickoff · 4 agent 并发 spec 详化 6 PR                                                                              |
| [session-21.md](session-21.md) | 2026-04-29 | 21      | #173-#175 | v0.1.0-alpha 双平台 GA · macOS unsigned .dmg + Linux .deb / .AppImage                                                          |
| [session-20.md](session-20.md) | 2026-04-26 | 20      | #152-#168 | ADR-015 accepted + PR #157 round 1/2 inline 反模式 → §2.13 规则化                                                              |
| [session-19.md](session-19.md) | 2026-04-25 | 19      | #117-#152 | 史上最高产 36 PR · MVP-11 5/5 ✅ + MVP-05 Phase A/B/C + ADR-006 Ubuntu validated + branch protect 机械化 + ADR-015 accepted    |
| [session-18.md](session-18.md) | 2026-04-25 | 18      | #106-#116 | 4 track 并发极致产出 · 11 PR merge · 5 Phase 落地 + 3 spec ready 加强                                                          |
| [session-17.md](session-17.md) | 2026-04-23 | 17      | #99-#105  | MVP-04 Phase F 收口 + MVP-08 Phase A/B/C 落地 + PR Actions 分钟节流                                                            |

## 🗂️ 待归档（M-2 滚动窗口外 · 按需补）

以下 session 仍在 git 历史中可追溯 · 但未单独归档（PROGRESS.md M-2 滚动窗口仅保留最近 2 session 摘要 · 更早信息需 `git log` 检索 · 不强制单独成文）：

- **session 22 / 23**（2026-04-30 ~ 2026-05-03 · Apple Dev / dependabot 批处理阶段 · 详见 `git log --grep="session 2[23]"`）
- **session 28**（2026-05-12 · 单 day 9 PR merged · 4-track + 5 idle 查漏补缺 · MVP-15 Phase D §F+§G · 当前 PROGRESS.md 头条 · 下次 session 末归档）
- **session 29**（2026-05-12 ~ active · MVP-17 spec ready + Phase B skeleton + 3-track dispatch 启动 · 主 agent 已 7 PR merged · Codex Phase A / OpenCode Phase C 待 push · in-progress 状态 · session 结束后归档）

## 🗂️ 预期归档（Phase 2 历史复盘 · 按需补）

以下为 Phase 2 积累的可归档话题（在事件复盘需要时按需补入）：

- **2026-04-17 ~ 2026-04-18 Phase 2 Codex 5 轮审查**（10 HIGH findings · PR #9 3 commits · Codex companion 对抗性审查方法的效果评估）
- **PR #4 close · 为什么不用共享 `SPIKE-REPORT.md`**（物理隔离 vs 声明式并发治理 · Codex PR #5 R2 F1 教训）
- **Phase 4 CI / gitleaks / task-spec-validator 设计**（从 advisory 到 enforced 的升级路径）

---

## 🔗 相关

- `CLAUDE.md §12. 规则即行动触发器`（元任务的专属检查清单）
- `CONTRIBUTING.md` · 贡献流程（包括什么时候开复盘）
- `docs/adr/` · 正式架构决策的承载（≠ session-history 侧重"过程"）

---

## ⚠️ 安全约束

本目录文件**不得**含：

- auth token / API key / PII
- 真实用户数据 / 生产日志
- 未脱敏的 CLI 输出

gitleaks CI（Phase 4）会扫此目录。

---

**本目录 Phase 3 建立（2026-04-18）· 具体记录在真实事件发生时补入。**

---

## 🧭 Session Archive Timeline（session-17 ~ session-30）

> 数据源：各 `session-NN.md` 顶部元信息（`session/date/pr_range/theme`）实读整理。
> 目标：把 `docs/session-history/` 从“命名约定文档”升级为“可导航索引入口”。

| Session                     | Date                     | PR Range  | Theme（一句话）                           | 标志性事件                           |
| --------------------------- | ------------------------ | --------- | ----------------------------------------- | ------------------------------------ |
| [session-30](session-30.md) | 2026-05-13 + 2026-05-14  | #281-#307 | 4-agent pool 首次同时跑 + MVP-17 完整收口 | OpenCode N=4 试金石通过              |
| [session-29](session-29.md) | 2026-05-12 晚→2026-05-13 | #281-#294 | MVP-17 收口推进 + 协作 failure mode 暴露  | OpenCode N=3 §2.10 violation         |
| [session-28](session-28.md) | 2026-05-12               | #271-#279 | 4-track 并发峰值 + validator 工具化       | `validate-runtime-evidence.mjs` 落地 |
| [session-27](session-27.md) | 2026-05-10               | #264-#266 | v0.3 sprint A/B/C 全收 + D 启动           | MVP-16 Phase D bench 启动            |
| [session-26](session-26.md) | 2026-05-09               | #259-#263 | 四并行 phase 推进 + 文件域隔离实证        | OpenCode §2.10 第 2 次重演           |
| [session-25](session-25.md) | 2026-05-07               | #251-#253 | v0.3 phase A 启动（2/4 完成）             | 主 agent reviewer gate 实战          |
| [session-24](session-24.md) | 2026-05-04 ~ 2026-05-06  | #245-#250 | v0.3 sprint kickoff spec-only 批处理      | 4 agent 并发详化 6 PR                |
| [session-23](session-23.md) | 2026-05-02 ~ 2026-05-04  | #207-#233 | v0.1 收尾 + v0.2 W13/W14 双推进           | 3 day 27 PR（当前最大）              |
| [session-22](session-22.md) | 2026-04-30               | #189-#194 | MVP-20（后重命名为 MVP-22）全 5 phase     | PTY warm pool 一日闭环               |
| [session-21](session-21.md) | 2026-04-26 ~ 2026-04-29  | #173-#187 | v0.1.0 GA + admin override 特殊窗口       | 7 direct pushes 治理触发             |
| [session-20](session-20.md) | 2026-04-26               | #152-#169 | MVP-10 Phase B 完整闭环 + 规则化教训      | dispatch §2.13/§2.14 固化            |
| [session-19](session-19.md) | 2026-04-25               | #117-#152 | 史上最高产窗口（36 PR）                   | ADR-006 Ubuntu validated             |
| [session-18](session-18.md) | 2026-04-25               | #106-#116 | 4-track 并发 11 PR                        | MVP-09 Phase A 落地                  |
| [session-17](session-17.md) | 2026-04-23               | #99-#105  | MVP-04 收口 + MVP-08 A/B/C                | PR Actions 分钟节流                  |

### Timeline 扩展注记（用于快速查读）

#### session-30

- 关键词：`4-agent dispatch`、`MVP-17 收口`、`stale base race`.
- 适用场景：查“并发派工在高压窗口如何收口”。
- 关联风险：§2.15 stale-base 防护规则落地前后差异。

#### session-29

- 关键词：`MVP-17`、`OpenCode N=3`、`协作治理`.
- 适用场景：查“review 不变量收敛”与“agent trust gate”。
- 关联规则：session 切换边界判定（见下节）。

#### session-28

- 关键词：`validator`、`4-track`、`clippy -D warnings`.
- 适用场景：查“runtime evidence 自动校验工具链起点”。
- 关联规则：§2.12 worktreeConfig 根治配置污染。

#### session-27

- 关键词：`phase A/B/C 全收`、`phase D bench`.
- 适用场景：查“从功能开发切到证据阶段”的拐点。
- 关联规则：bench-only PR 模式。

#### session-26

- 关键词：`文件域隔离`、`并发 4 PR`.
- 适用场景：查“多 agent 并发在文件域层面如何防冲突”。
- 关联风险：OpenCode §2.10 第 2 次事件触发 memory 升级。

#### session-25

- 关键词：`phase A 启动`、`reviewer 翻转 gate`.
- 适用场景：查“v0.3 sprint 第一脚如何起步”。
- 关联风险：OpenCode 首次谎报事件。

#### session-24

- 关键词：`spec only`、`kickoff`.
- 适用场景：查“无代码日如何做批量规格收口”。
- 关联输出：MVP-12/14/15/16 spec ready。

#### session-23

- 关键词：`3 day 27 PR`、`W13/W14`、`ADR-016`.
- 适用场景：查“高吞吐 + 治理升级并行”的完整样板。
- 关联决策：v2-D.1 -> v2-D.2。

#### session-22

- 关键词：`PTY warm pool`、`一日 5 phase`.
- 适用场景：查“性能痛点的快速闭环”。
- 关联后续：session-23 中发生 MVP-20 -> MVP-22 rename。

---

## 🧷 Session 切换边界判定原则（来自 session-29 归档沉淀）

参考 `session-29.md` 的边界段（“session 切换边界判定”），当前 README 统一采用以下 3 条原则：

1. **user 起新对话 = 硬信号**
   - 只要用户明确开启新对话，会话边界立即切换。
   - 该规则优先级最高，覆盖 PR merge 时间连续性。

2. **24h 间隔 = 软信号**
   - 超过 24h 可作为“可能切换”的辅助判断。
   - 但不强制，若上下文连续且用户未开新对话，可保持同一 session 叙事。

3. **PR # 连续性不强制**
   - PR 编号连续不等于同一 session。
   - merge 顺序反转（例如 #292/#293 交错）不自动触发重切会话。

实践建议：

- 先看用户对话边界，再看时间，再看 PR 序列。
- archive 文档里若遇边界争议，显式写“决策理由”，不要静默改口径。
- 若历史记录已发布且追溯成本高，可在新 session 文档补“trace 注记”，不做强制回填。

---

## 🔄 M-2 滚动窗口归档规则（PROGRESS 对齐）

`docs/PROGRESS.md` 已固定规则：PR 列表段只保留最近 2 个 session 摘要，更早信息以 `docs/session-history/` 为准。

执行口径：

1. **PROGRESS 负责当前窗口**
   - 最近两个 session：保留详细摘要（便于执行态读取）。
   - 更早 session：转为 reference 链接，不在 PROGRESS 堆积长段。

2. **session-history 负责长期记忆**
   - 每次滚动时新增 `session-NN.md`。
   - README 维护全量 timeline 索引，作为第一入口。

3. **归档时机**
   - 常规：session 末做一次归档。
   - 补档：跨 session 的 housekeeping PR 统一补齐。

4. **最近实例（可追溯）**
   - PR #310：session-29 archive。
   - PR #314：session-23 archive。
   - PR #317：session-22 archive。

5. **PROGRESS 写法建议**
   - 保留 3-5 行 reference 摘要。
   - 用 `session-history/session-NN.md` 链接承接细节。
   - 避免把旧 session 的长表格重新贴回 PROGRESS。

---

## 📚 复盘类档案索引（非 session-NN）

本目录除了 `session-NN.md`，还允许沉淀跨 session 复盘文档，命名为 `YYYY-MM-DD-<topic-slug>.md`。

当前已归档文件：

| 文件                                                                                       | 类型       | 用途                                          |
| ------------------------------------------------------------------------------------------ | ---------- | --------------------------------------------- |
| [2026-04-19-spike-code-loss-retrospective.md](2026-04-19-spike-code-loss-retrospective.md) | Spike 复盘 | 记录跨 session 的代码丢失事件、根因与修复动作 |

使用场景：

- 某次事件跨越多个 session，放在单一 `session-NN.md` 会割裂上下文。
- 需要沉淀“方法论级教训”，但不适合放到 ADR（ADR 偏决策，不偏过程细节）。
- PR close / revert / 大型事故，需要单独复盘并给后续 dispatch 复用。

建议复盘文档至少覆盖：背景时间线、根因、修复证据、对规则/流程的回写项。
