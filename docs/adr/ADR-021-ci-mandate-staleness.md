# ADR-021: CLAUDE.md「合入后 CI 验证」mandate 相对 ci.yml 已 stale

**状态**：accepted
**日期**：2026-05-16（proposed）· 2026-05-17（accepted · Arbiter tajiaoyezi 拍板方案 (b)）
**决策者**：Grok（dispatch 起草 · self-review v2-D.2 单人项目）· Claude Code 主 agent 独立 review · Arbiter tajiaoyezi 拍板
**对应 `CLAUDE.md` 决策表**：—（治理规则 · 本 ADR 记录 CI mandate 漂移 · 对应 CLAUDE.md §"合入后 CI 验证" 条款）
**前置事件**：PR #102（关闭 push/PR 触发）· PR #329（dispatch 压缩）· session 32 续 8 PR merge（#344-349）· ci.yml 最后一次 run 2026-04-25/28（手动 · 全 failure）

---

## 背景与问题（Context）

`CLAUDE.md` 有「合入后 CI 验证」条款（要求 merge 后 5-10min 跑 `gh api .../check-runs` 查合入 commit CI · 任一 failure 立即开 fix PR）· 隐含假设 = CI 在 push/PR 自动跑。**实测现实**：`.github/workflows/ci.yml` 触发 = `on: workflow_dispatch:` **仅手动** · 非 push/PR 触发 · 最后一次 ci.yml run 2026-04-25/28（手动 · 全 failure）· session 32 续 8 PR merge 零 CI auto run。结论：项目 CI 从不自动跑 = by design（**非 billing 暂停**）· 该 mandate 操作上 moot/误导（下个 agent 会按它跑 check-runs 发现空而困惑）。

**实际质量门**：本地 gate（cargo test/clippy/fmt · pnpm lint/typecheck/vitest）+ reviewer §2.14 独立复跑。session 32 续所有 merge 均主 agent 本地全 gate 覆盖。

## 决策（Decision · proposed · Arbiter 拍板后生效）

§2.1 要求：本 ADR 仅记录事实 + 提议选项 · **status 只能 proposed** · 不得自 accept · 不改 CLAUDE.md（accept 后另 PR 执行）· 最终由 Arbiter 裁决采用哪条：

- **(a)** ci.yml 加 `push`/`pull_request` 触发恢复 auto-CI —— 代价：消耗 GitHub Actions 分钟（私有仓非 Pro 预算 · session 21 billing 暂停先例）
- **(b)【推荐】** 改 CLAUDE.md 该条款为「质量门 = 本地 gate + reviewer §2.14 实跑 · CI workflow_dispatch 手动按需」· 承认 no-auto-CI 是既定运营模型 · mandate 与现实对齐
- **(c)** 混合：保留手动 ci.yml + CLAUDE.md 注明"合入后 CI 验证仅在手动 dispatch 后适用 · 默认门 = 本地+reviewer"

无论哪条：**CLAUDE.md 实际改动、ci.yml 改动（若有）—— 均由 Arbiter 拍板明确后在独立 PR 执行 · 本 PR 仅 proposed draft**。

## 约束（Constraints）

- 本 ADR **仅记录+提议** · 不改 CLAUDE.md / ci.yml / 任何决策文件（Arbiter accept 后另 PR 改）
- status **proposed** · 需 Arbiter 拍板 → accepted 后方生效（v2-D.2 单人项目 self-review + Arbiter approval 流程）
- 不得声称 "Arbiter 已同意 X" · 所有选项保持开放供 Arbiter 裁决

## 后果（Consequences）

**正面**：

- 消除 stale mandate 误导 · 未来 agent 不会按已失效的 "合入后 5-10min 查 check-runs" 操作而困惑
- 明确当前运营质量门（本地 gate + §2.14 reviewer 实跑）是项目的实际信任模型
- 为 GitHub Actions 预算恢复后是否重开 auto-CI 提供清晰决策点

**负面 / 风险**：

- 若选择 (a) 恢复 auto-CI：私有仓 Actions 分钟消耗增加 · 需持续监控 billing
- 若选择 (b)：CI workflow 继续手动触发 · 极端情况下可能漏掉某些合成场景的自动验证（但当前本地 gate 已覆盖）
- 任何选项均需后续 PR 实际改 CLAUDE.md（本 PR 不执行）

---

## Arbiter 拍板栏（tajiaoyezi · v2-D.2 单人项目 self-review + Arbiter approval · 2026-05-17 已拍板）

- [x] 事实准确性：ci.yml 仅 `workflow_dispatch` · 最后 run 2026-04-25/28 · session 32 续 8 PR 零 auto CI · 均已 git show / cat 验证（主 agent 2026-05-17 复核：`.github/workflows/ci.yml` `on: workflow_dispatch:` · `gh run list --branch main` 仅 dependabot/renovate · 合入 commit check-runs 空）
- [x] 选项完整：(a)(b)(c) 三条均已列出 · 推荐 (b) 理由已陈述
- [x] 约束遵守：proposed 阶段未碰 CLAUDE.md / 现有 ADR（本 accept PR 才执行 CLAUDE.md 改写 · v2-D.2 流程合规）
- [x] **选定方案：(b)** —— Arbiter tajiaoyezi 2026-05-17 拍板「改 CLAUDE.md 该条款对齐现实 · 承认 no-auto-CI 是既定运营模型」

**accepted 决议**（Arbiter 2026-05-17 flip · 本 PR 同步执行 CLAUDE.md 改写）：

1. 记录事实：CLAUDE.md「合入后 CI 验证」mandate 与 ci.yml 实际触发策略已 drift · 由 design 导致（PR #102 关闭 PR 触发 · session 21 billing 进一步关 push main 触发 · 仅留 `workflow_dispatch`）
2. 质量门现状坐实：**本地 gate（cargo test/clippy/fmt · pnpm lint/typecheck/vitest）+ reviewer §2.14 独立复跑** 是当前唯一有效质量门 · 无自动 CI 是既定运营模型（非临时 billing 故障）
3. 选定 (b)：CLAUDE.md §5「合入后 CI 验证」改写为「合入后质量门验证 = 本地 gate + reviewer §2.14 实跑 · CI = `workflow_dispatch` 手动按需（GitHub Actions 预算恢复 / 仓库公开后可评估重开 auto-CI）」· 本 PR 执行
4. 重开 auto-CI（选项 a）的未来触发条件：仓库变 public 或升级 GitHub Pro（Actions 分钟预算不再是约束）· 届时新开 ADR 评估 push/pull_request 触发恢复

---

**实测坐实**（Grok dispatch · 2026-05-16）：

- ci.yml `on:` 区块：仅 `workflow_dispatch:`（git show 确认）
- CLAUDE.md 相关条款：存在 "合入后 5-10min 跑 gh api check-runs" 要求（主 agent 已知）
- 8 PR merge 记录：#344/#345/#346/#347/#348/#349 + #350 等，零 CI auto 触发（dispatch prompt 事实）
