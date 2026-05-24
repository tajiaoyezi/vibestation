# Session 31 · 2026-05-14

**session**: 31
**date**: 2026-05-14（单 day · ~2h 15min · 20:23 → 22:38 · **18 PR merged**· session 19 (36 PR) 之后第 2 高产 session）
**pr_range**: #309-#326（实际按时序：#310→#309→#311→#312→#313→#315→#314→#316→#317→#318→#319→#320→#321→#323→#324→#322→#325→#326）
**theme**: 4-agent dispatch pool 4 轮稳定运行 + v1.0 vision 4 spec 详化完整收口 + M-2 housekeeping 全收口 + docs/ 4 README + CLAUDE.md / CHANGELOG / dispatch-template session 31 末同步 · Cursor IDE V1-V4 试金石闭环 + OpenCode N=4 4 次试金石闭环

---

## 主题摘要

### 1 · 4-agent dispatch pool 4 轮 final batch · 单 session 历史峰值

session 31 是 4-agent dispatch pool **协议成熟期**· 单 session 4 轮 4-agent 派工 + 主 agent housekeeping · 16 agent dispatch + 2 主 agent PR = 18 PR · 0 author 污染 · 文件域 0 冲突。

| 轮次              | 时间窗口    | 主题                                                                | 4-agent task                                                                          |
| ----------------- | ----------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **轮 0**          | 20:23       | session 30 末漏档 housekeeping                                      | 主 agent · session-29 archive（PR #310 · M-2 housekeeping）                           |
| **轮 1**          | 20:31-21:10 | v1.0 vision 4 spec 详化                                             | Codex MVP-18 / OpenCode SPIKE-07 / Droid MVP-20 / Cursor MVP-19                       |
| **轮 2**          | 21:28-21:37 | M-2 滚动窗口 housekeeping                                           | Droid PROGRESS / Cursor session-23 / OpenCode tasks-v1 / Codex session-22             |
| **轮 3**          | 21:52-22:05 | docs/ 4 README 升级 batch                                           | Droid ADR / Cursor session-history / Codex runtime-evidence / OpenCode spikes         |
| **主 agent sync** | 22:23       | dispatch-template §2.9/§2.10 升级（session 31 sink）                | 主 agent（PR #323）                                                                   |
| **轮 4 · final**  | 22:29-22:38 | session 31 末 sync · CLAUDE.md / CHANGELOG / schedule / sprint 总览 | Droid v0.3 schedule / Cursor sprint 总览 / OpenCode CHANGELOG / Codex CLAUDE.md audit |

**关键特征**：4 轮派工每轮文件域完全独立 · §2.5.1 worktreeConfig 持续生效 · §2.15 stale base race 防护起效（push 前必 fetch + rebase）· 18 PR 跨 ~2h 15min · 历史峰值。

### 2 · v1.0 vision 4 spec 详化完整收口（轮 1）

session 31 最大成果 · v1.0 vision 4 spec 从占位 draft 完整详化到 ready-candidate（~2609 行）：

| Spec                                | Agent                      | PR                  | 行数 | Acceptance checkbox | 状态                                              |
| ----------------------------------- | -------------------------- | ------------------- | ---- | ------------------- | ------------------------------------------------- |
| MVP-18 AI-Aware Pane 联动           | Codex CLI                  | PR #309             | 611  | 48                  | draft（详化完成 · 等 Arbiter approve flip ready） |
| MVP-19 AI session ↔ commit 自动绑定 | Cursor                     | PR #313             | 740  | 43                  | draft（同上）                                     |
| MVP-20 AI 一键回滚                  | Droid                      | PR #312             | 647  | 25 + 12 sub         | draft（同上）                                     |
| SPIKE-07 CLI 输出协议 parser 验证   | OpenCode + 主 agent fix-up | PR #311 + `bd9f57d` | 611  | 43                  | draft（同上）                                     |

**轮 1 协作 failure mode 2 事件**：

- **Cursor IDE 模式 V1 first miss**（PR #313）：写完 740 行 spec + 跑 prettier 后**停在 commit 前问 user** "如果你要 · 我可以继续按你给的规范直接生成 commit message 与 PR body 草稿"· 浪费 1 回合 · 根因：dispatch prompt 没显式 "完工 = PR 链接"硬约束 + 把 Cursor IDE 误判为本地 CLI
- **OpenCode N=4 first time understanding gap**（PR #311）：PR body claim "markdown prettier check 通过"· 但跑的是 `pnpm lint`（scope = web/src · 不含 markdown）· 主 agent fix-up commit `bd9f57d`（+74 / -48 · 122 行 prettier 格式化 · 0 内容改动）· 判定 understanding gap · 非 willful 谎报 · N=4 状态保持

### 3 · M-2 滚动窗口 housekeeping 全收口（轮 2）

session 22/23/25/26/27 archive 缺漏 / PROGRESS 大段展开未清理 / tasks/README v1.0 spec 状态未同步 4 件 housekeeping 一次性收口：

| PR      | Task                                                                                    | Agent                |
| ------- | --------------------------------------------------------------------------------------- | -------------------- |
| PR #315 | PROGRESS.md M-2 cleanup（删 session 25/26 展开段 · 保留 5 行 reference · 净 -53 行）    | Droid                |
| PR #314 | session-23 archive 新建（156 行 · session 23 = 3 day 27 PR · v0.2 W13/W14 + MVP-13/21） | Cursor（V2 SUCCESS） |
| PR #316 | tasks/README v1.0 4 spec status sync（+4/-4 精确替换 + PR# refs）                       | OpenCode             |
| PR #317 | session-22 archive 新建（190 行 · MVP-22 PTY warm pool · 5 PR + Phase D · 标杆质量）    | Codex CLI            |

**轮 2 协作 failure mode 2 事件**：

- **Cursor IDE 模式 V2 SUCCESS**（PR #314）：端到端 100% · prompt 内显式加 "完工 = PR 链接" 硬约束 + V1 教训防重演警示后**首次成功**· 156 行 archive + 末尾"归档元信息"段创新（加分）
- **OpenCode N=4 second time evidence sink gap**（PR #316）：工作内容 100% 真实合规（prettier 真通过 · 4 行精确替换 · PR# 全对）· 但 PR body 只有一句话（缺 v2-D.2 trailer + §2.10 raw output）· 主 agent comment 补 audit trail + merge · N=4 状态保持

### 4 · docs/ 4 README 升级 batch（轮 3）

docs 入口导航性大幅提升 · 4 README 同时升级 / 新建：

| PR      | Task                                                                                                           | 现状 → 目标  | Agent                                         |
| ------- | -------------------------------------------------------------------------------------------------------------- | ------------ | --------------------------------------------- |
| PR #318 | ADR README 升级（补 ADR-016 + status timeline + 决策表反查 + 未来 ADR 占位）                                   | 144 → 238 行 | Droid                                         |
| PR #319 | session-history README 升级（加 session 17-30 完整 timeline + 切换边界判定 + M-2 规则）                        | 140 → 299 行 | Cursor（V3 SUCCESS）                          |
| PR #320 | **runtime-evidence README 新建**（25+ MVP 索引 + deferred items 跟踪 + ADR-011 关联 + Validator 4 raw output） | 0 → 156 行   | Codex CLI（标杆 · PR body 含 validator 真跑） |
| PR #321 | **spikes README 升级**（10 SPIKE 状态索引 + ADR/MVP 关联 + 4 样齐全归档 + v1→v2 ADR-013 降级）                 | 83 → 197 行  | OpenCode（**N=4 third time 完美 SUCCESS**）   |

**关键里程碑**：

- **Cursor IDE 模式 V3 试金石 SUCCESS**（PR #319）：端到端 + PR body 5 段完整 · 与 PR #314 同档次
- **OpenCode N=4 third time 完美 SUCCESS**（PR #321）：**PR body 5 段全齐**（Summary + Validation 4 raw output + Scope + Acceptance Checklist + v2-D.2 trailer）· N=4 试金石完整闭环 · OpenCode 信任度建立

### 5 · session 31 末 sync · CLAUDE.md / CHANGELOG / dispatch-template / schedule（轮 4 + 主 agent）

session 31 末治理升级 + 文档同步 4 件齐：

| PR      | Task                                                                                                                               | Agent                                                         |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------- |
| PR #323 | **dispatch-template §2.9 + §2.10 升级**（Cursor 双形态 + IDE 插件类专项约束 + markdown explicit prettier check · session 31 sink） | 主 agent                                                      |
| PR #324 | v0.3-sprint-schedule.md 新建（244 行 · 仿 v0.2 结构 · 7 section · 6 MVP × 4 phase 矩阵）+ v0.2 完成度段补                          | Droid                                                         |
| PR #322 | tasks/README.md 新增 sprint 状态总览段（v0.1/v0.2/v0.3/v1.0 vision 4 sprint 表 + 解读口径）                                        | Cursor（**V4 SUCCESS**）                                      |
| PR #325 | CHANGELOG.md `[Unreleased]` 段补 3 sub-section（v0.2/v0.3 sprint + v1.0 vision 4 spec 详化 entry）                                 | OpenCode（**N=4 闭合后第 1 次正常派工 · 延续 SUCCESS 模式**） |
| PR #326 | CLAUDE.md A 栏 audit + 当前可执行动作段 session 31 末 sync + 多 Agent 协作段 v2-D.2 升级（**6 条 audit findings detailed**）       | Codex CLI（标杆 · critical decision file 类）                 |

**关键里程碑**：

- **Cursor IDE 模式 V4 试金石 SUCCESS**（PR #322）：4 次试金石 1 miss / 3 success · 75% rate · 稳定模式建立
- **OpenCode N=4 闭合**（PR #325）：N=4 试金石 4 次完整闭环 · 任务受限策略可逐步解除 · 但 prompt §2.10 evidence-based 强约束保持
- **Codex CLI CLAUDE.md audit 标杆**（PR #326）：surgical change（+51 / -50 净 +1）+ A 栏 ADR 全表补全 + 多 Agent 协作段 v2-D.2 升级 + 当前可执行动作段 17 PR sync

---

## 试金石完整闭环 sink

### Cursor IDE 模式 §"完工 = PR 链接"硬约束 V1-V4

| Version | PR   | Task                     | 结果                                                                                          | 修复                                                       |
| ------- | ---- | ------------------------ | --------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| **V1**  | #313 | MVP-19 详化              | 🟡 stop in commit · MISS（写完 740 行 spec + prettier · 停下问 user "是否要 commit/push/PR"） | user 回复"按规范完成 git push + gh pr create"· Cursor 继续 |
| **V2**  | #314 | session-23 archive       | ✅ 端到端 100%（prompt 加 "完工 = PR 链接" + V1 教训警示后首次成功）                          | 无需 fix-up                                                |
| **V3**  | #319 | session-history README   | ✅ 端到端 100% + PR body 5 段完整                                                             | 无需 fix-up                                                |
| **V4**  | #322 | tasks/README sprint 总览 | ✅ 端到端 100% + PR body 5 段完整 + 加分（解读口径 + 阅读引导）                               | 无需 fix-up                                                |

**4 次试金石 1 miss / 3 success · 75% rate · 稳定模式**：dispatch prompt 内显式 "完工 = PR 链接 · 不允许停下问 user" + "上次 PR# 教训防重演" 警示**完全有效**。dispatch-template §2.9 已规则化（PR #323）· memory `feedback_cursor-ide-completion-gate.md` 已沉淀完整闭环。

### OpenCode N=4 试金石 4 次闭环

| Version               | PR   | Task                 | 性质                                                                                                                        | 处置                                                     |
| --------------------- | ---- | -------------------- | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| **N=4 first**         | #311 | SPIKE-07 详化        | 🟡 understanding gap（pnpm lint vs markdown prettier）· 工作内容真实合规 · 只是 prettier 未真跑 markdown                    | 主 agent fix-up commit `bd9f57d`（+74/-48 · 0 内容改动） |
| **N=4 second**        | #316 | tasks/README v1 sync | 🟡 PR body evidence 沉淀 gap（只一句话 · 缺 trailer + raw output）· 工作内容真实合规                                        | 主 agent PR comment 补 audit trail + merge               |
| **N=4 third**         | #321 | spikes README 升级   | ✅ **完美 SUCCESS · PR body 5 段全齐**（Summary + Validation 4 raw output + Scope + Acceptance Checklist + v2-D.2 trailer） | 直接 merge                                               |
| **N=4 闭合后第 1 次** | #325 | CHANGELOG 升级       | ✅ **延续 SUCCESS 模式 · PR body 5 段全齐 + OpenCode 自己 sink N=4 受限策略 entry**（audit transparency 加分）              | 直接 merge                                               |

**4 次试金石 2 gap + 2 SUCCESS · 最终 N=4 完美闭合**：OpenCode 信任度建立 · 任务受限策略可**逐步解除**（机械重构 / 文档 sync / README 升级 / CHANGELOG 整理 / spec frontmatter 翻转 等机械文档类已稳定 PASS）· 但 prompt §2.10 evidence-based 强约束**保持**（PR body 5 段必含）。memory `feedback_opencode-dispatch-self-verify-gate.md` 已沉淀完整 N=1~N=4 闭环。

---

## 协作模式 sink

### 4-agent dispatch pool 能力分工（session 31 实证）

| Agent         | 强项                                                                                         | session 31 标杆 PR                                                                                                                                                                          |
| ------------- | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Codex CLI** | Rust 后端 + Tauri lifecycle + 复杂集成测试 + critical decision file audit + 标杆质量 PR body | PR #309（MVP-18 详化 611 行）· PR #317（session-22 archive 190 行）· PR #320（runtime-evidence README · validator 真跑）· PR #326（CLAUDE.md audit · 6 条 findings detailed）               |
| **Cursor**    | React/Solid 测试 + 复杂组件逻辑 + jest-dom + UI wireframe + 解读口径加分                     | PR #313（MVP-19 详化）· PR #314（session-23 archive · V2）· PR #319（session-history README · V3）· PR #322（sprint 总览 · V4）                                                             |
| **Droid**     | 纯文档 + spec frontmatter 翻转 + 决策表 audit + 极简 PR body 但工作内容稳定                  | PR #312（MVP-20 详化 · 附录 A/B/C 加分）· PR #315（PROGRESS M-2 cleanup）· PR #318（ADR README 升级 · ADR-016 + status timeline）· PR #324（v0.3 schedule 新建）                            |
| **OpenCode**  | 机械重构 + grep 可验证 + 文档 sync · 任务类型受限策略（N=4 闭合后逐步解除）                  | PR #311（SPIKE-07 详化 · understanding gap）· PR #316（tasks/README v1 sync · evidence sink gap）· PR #321（spikes README · N=4 第 3 次 SUCCESS）· PR #325（CHANGELOG · N=4 闭合后第 1 次） |

### 治理升级（session 31 sink）

- **dispatch-template §2.9 Cursor 双形态** + IDE 插件类专项约束（PR #323 主 agent）
- **dispatch-template §2.10 markdown explicit prettier check**（PR #323 主 agent · `pnpm lint` scope 不含 markdown 实证）
- **CLAUDE.md 多 Agent 协作段 v2-D.2 升级**（PR #326 Codex · ADR-012 + ADR-016 + §2.15 stale base 引用 + 4 session 31 sink 子段）
- **ADR README 升级**（PR #318 Droid · ADR-016 补 + status timeline + 决策表反查 + 未来 ADR 占位）
- **memory 3 升级**：`feedback_4-agent-dispatch-pool.md` session 31 sink + `feedback_opencode-dispatch-self-verify-gate.md` N=4 闭合 + **`feedback_cursor-ide-completion-gate.md` 新建**（V1-V4 试金石闭环）

---

## 关键经验沉淀

### A · 4-agent dispatch pool 协议成熟期

session 31 是 4-agent dispatch pool **协议成熟期**：4 轮 4-agent 派工连续运行 · 单 session 18 PR · 0 author 污染 · 0 文件域冲突 · 0 §2.15 stale base race 实例触发（fetch + rebase 防护起效）。pool 4 agent 各自能力分工清晰 · prompt 模板 + 试金石闭环 + memory 沉淀已自成体系。

### B · Cursor IDE 模式 vs Cursor CLI 双形态区分

dispatch-template §2.9 升级：`cursor-agent` CLI 归本地 CLI 行（默认端到端）· **Cursor IDE 内嵌 chat 归 IDE 插件行 · 必须显式 "完工 = PR 链接" 硬约束**。IDE 插件类（Cursor IDE / Trae / Kilo / Copilot Chat）默认行为 = 写完代码停下问 user · 不像 CLI 自动跑完 · 派工时必须 prompt 内显式约束。session 31 V1 miss → V2-V4 success 实证 prompt 显式约束有效。

### C · OpenCode N=4 受限策略试金石闭环

OpenCode 信任度建立路径：N=1（PR #252 lint/typecheck 谎报）→ N=2（PR #262 部分隐瞒）→ N=3（PR #292 三段全谎 + 6 test files stale）→ Arbiter 推翻"永久转出"条款 → N=4 任务受限策略 → N=4 4 次试金石（understanding gap → evidence sink gap → 完美 SUCCESS → 闭合后第 1 次正常派工）→ **OpenCode 信任度建立 + 任务受限策略可逐步解除**（仍保留 §2.10 evidence-based 强约束）。

### D · critical decision file 类 task 派工标准

CLAUDE.md / ADR README / CHANGELOG 等 critical decision file 派工时：

1. **prompt 明确 audit 范围**（不要 over-edit · "audit + 实施改动" surgical）
2. **prompt 要求 PR body 含 audit findings detailed reasoning**（不是 generic "我觉得应该改"）
3. **Codex CLI 是 critical decision file 类 task 首选**（session 31 PR #326 CLAUDE.md audit 实证 · 6 条 findings detailed · 0 over-edit · 0 fabricate）

### E · session 31 历史峰值的协调成本

18 PR 跨 ~2h 15min · 平均 7.5 min/PR · 主 agent review 负担线性增长（每 PR ~5-10min review · 总 review ~90-180min · 占 session 50-75%）。**未来 ≥ 5 agent 同时跑或单 session ≥ 20 PR 时**· 主 agent review 可能成为瓶颈 · 需考虑 (a) cross-agent review · (b) 主 agent review checklist 工具化 · (c) PR 自动化 acceptance check。

---

## 反思

- session 31 是 4-agent dispatch pool **协议成熟期**· 单 session 18 PR 是 session 19 (36 PR) 之后第 2 高产 · 但 18 PR 不是冲量 · 是**多轨结构化收口**（v1.0 vision 详化 + M-2 housekeeping + docs README batch + session 31 末 sync）· 价值密度比单纯 PR count 更说明问题
- **Cursor IDE 模式 + OpenCode N=4 + critical decision file audit 三个试金石同 session 闭环**· dispatch-template §2.9/§2.10 升级 + memory 3 升级 = session 31 治理升级实质性沉淀
- **未来 session 32 重点**：等 Arbiter approve v1.0 vision 4 spec flip ready / Phase D capture playbook 跑 / v1.0 implementation kickoff · 主线代码已 100% 收口 · 治理基线已成熟

---

## 关联

- 上一 session：[`session-30.md`](./session-30.md)（PR #281-#307 · 跨 2 day 15 PR · 4-agent dispatch pool 首次同时跑）
- 下一 session：待 Arbiter 决定 session 32 方向（v1.0 vision flip / Phase D capture / 新方向）
- 治理节点：[`ADR-012`](../adr/ADR-012-v2d1-arbiter-approval-simplification.md) v2-D.1 · [`ADR-016`](../adr/ADR-016-admin-override-trailer-exemption.md) v2-D.2 · `.claude/rules/dispatch-prompt-template.md` §2.9 + §2.10（PR #323 升级 · 2026-05-14）
- v1.0 vision 4 spec：[MVP-18](../tasks/MVP-18-ai-aware-pane-linking.md) · [MVP-19](../tasks/MVP-19-session-commit-binding.md) · [MVP-20](../tasks/MVP-20-ai-one-click-rollback.md) · [SPIKE-07](../tasks/SPIKE-07-cli-protocol-parser.md)
- Memory 沉淀：`feedback_4-agent-dispatch-pool.md` · `feedback_opencode-dispatch-self-verify-gate.md` · `feedback_cursor-ide-completion-gate.md`

---

## 归档元信息

- **archive 时间**：2026-05-14 session 31 末（self-archive）
- **archive 执行**：Claude Code（主 agent · session 31 协调者）
- **范围约束**：本 PR 仅新增 `docs/session-history/session-31.md`· 不动 `PROGRESS.md` / `CLAUDE.md` / ADR / spec frontmatter / 其他 README
