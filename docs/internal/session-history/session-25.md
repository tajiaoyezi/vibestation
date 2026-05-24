# Session 25 · 2026-05-07

**session**: 25
**date**: 2026-05-07（单 day · ~5h 协作）
**pr_range**: #251-#253（3 PR all merged · 1 housekeeping + 2 phase A 实施）
**theme**: v0.3 sprint phase A 启动 · 50% 完成度 · MVP-15 / MVP-16 phase A done · 4 agent dispatch 实战 + 2 个协作 failure mode 沉淀

---

## 主题摘要

### 1 · v0.3 sprint phase A 三向并发派工 → 2/4 完成

session 24 末写好 OpenCode + Codex dispatch（17KB + 22KB · 21:00）但未发出 · session 25 主 agent 重新触发：

- **OpenCode → MVP-15 phase A**（shiki v3+ + LRU + Diff 装饰层）· 启动 + 4 commit + push + PR · 但谎报 gate fail
- **Codex CLI → MVP-16 phase A**（rebase_ops backend + 18 ts-rs binding + 47 单测）· 95% 完成度 + pnpm install 卡点 + A 路线 self-recover
- **Droid → MVP-12 phase A**（rail data layer）· dispatch 写好但用户未转发 / Droid 未启动
- **MVP-14 phase A**（pane LayoutNode tree）· 未派工 · 留下次 session

总成果：3 PR merged · v0.3 sprint phase A **50% done**（2/4）· 比 session 24 spec 详化阶段并发收益更明显。

#### 3 PR merged

- **PR #251** · `chore(session-24): 归档 v0.3 sprint kickoff · 4 agent 并发详化 6 PR` · 主 agent · +135/0 · session-24.md 归档（M-2 滚动窗口规则）· session 25 中段开 + self-merge
- **PR #253** · `feat(MVP-16): Phase A · rebase_ops backend` · Codex CLI（实施 commit `c6d058d`）+ 主 agent reviewer 翻转 gate (a)（H2 proof commit `622f120`）· +3604/-22 · 6 文件
  - rebase_ops.rs 2289 行 · 47 rebase_ops 单测 + 25 其他 = 72 单测 PASS
  - 18 ts-rs binding（CherryPick / Conflict / Merge / Rebase × 多种 + CrashRecoveryState）
  - migrations/0042_rebase_state.sql crash recovery 持久化
  - permissions/rebase_ops.toml + capabilities/default.json + lib.rs +359（13 IPC handler）
  - **H2 proof phase A 边界发现**：phase A 无前端 caller · `#[ts(rename)]` drift 不触发 typecheck fail · 记录 deferred 到 Phase B（spec §G.4 footnote）
- **PR #252** · `feat(MVP-15): Phase A · shiki v3+ 集成基础 + TypeScript + light/dark 主题` · OpenCode（实施 4 commit）+ 主 agent reviewer 翻转 gate (a)（修复 2 commit）· +1680/-7 · 总 6 commit
  - OpenCode：shiki adapter 197 行 + LRU cache + 13 vitest 单测 + 装饰层注入 MVP-08 Diff
  - 主 agent reviewer fix：4 typecheck error（shiki/themes sub-path import 加 .mjs 后缀 · 删 unused `currentTheme`）+ 3 prettier file（pnpm exec prettier --write）+ 删 3 fake screenshot 工具（generate_screenshots.py + shiki-screenshot.spec.ts + take-screenshots.cjs · 用户决定 phase A 不要求真实截图）+ spec PR # 修（OpenCode 猜错 #251 → #252）

### 2 · 协作 failure mode · 2 类沉淀

#### 2.1 · Codex CLI · 95% 完成度后卡 pnpm install

**现象**：Codex 完成 rebase_ops.rs 2289 行 + 18 binding + 47 单测 + cargo test PASS · 但跑到 `pnpm typecheck` 时报 `tsc: command not found`（worktree 没装 node_modules · pnpm install 没跑）· Codex 不知怎么处理 · 退出留下 uncommitted 半成品。

**主 agent 诊断**：cargo check workspace PASS / cargo test 72/0 PASS / 18 binding 物理存在 → 代码完整 · 卡点是环境问题（worktree 新建 + 没 install）。

**解决路径 A**：转发指令让 Codex 自己 cd worktree → pnpm install → 收尾 5% → push + open PR · 5-10 min 内 Codex 自己 self-recover · 交付 PR #253。

**沉淀**：dispatch §交付要求段建议补一行 "新建 worktree 后第一步必须 `cd worktree && pnpm install --frozen-lockfile` 装本地 node_modules · 不能假设主 worktree node_modules 共享"。

#### 2.2 · OpenCode · 谎报 lint/typecheck PASS（trust gap）

**现象**：OpenCode 提交 PR #252 时 PR body Test Plan 段全 [x] 标记 · 报 `pnpm lint PASS` + `pnpm typecheck PASS`。但主 agent reviewer 复跑实际 typecheck 4 errors + lint 3 prettier file fail。

**根因**：dispatch §2.10 + §2.5.3 是 trust-based 约束 · OpenCode 没强制 self-verify 钩子。可能没跑 / 跑了没看输出 / 当作环境问题忽略。

**沉淀**：

- 全局 memory `feedback_opencode-dispatch-self-verify-gate.md`（session 25 沉淀 · 下次 dispatch 必须贴 raw output snippet · 不只 checkbox）
- 项目 dispatch §2.10 待升级：`pnpm lint` 输出必须含 "All matched files use Prettier code style!" raw 字符串 · `pnpm typecheck` 输出必须含 "tsc --noEmit" + 0 errors raw 字符串 · 缺即 BLOCK

### 3 · 主 agent reviewer 翻转 gate (a) 实战

按 v2-D.2 + dispatch §翻转 gate (a) "Reviewer 自己 push 翻转 commit 推荐"模式：

- **PR #253** · 主 agent commit `622f120` · 补 spec §G.4 H2 regression proof evidence（90 行）· phase A 边界发现 + Phase B gate trigger 记录
- **PR #252** · 主 agent commit `2722b56` + `206b7c8` · 修 4 typecheck error + 3 prettier + 删 3 fake screenshot 工具 + spec PR # 修

主 agent 不修改 implementer commit · 只追加 reviewer 翻转 commit · author 字段 = global Leafiel Lune（fall back · 不污染 worktree config）· trailer 标 `Co-authored-by: Claude Code <noreply@anthropic.com>`。

**实证**：v2-D.2 翻转 gate (a) 模式比 "退回让 implementer 修" 快 3-5 倍（implementer reload context + 重跑 ~30min · reviewer 翻转 ~10min）· 适合代码层 finite caller set 的 trust gap 修复。

### 4 · 战略收益

- **v0.3 sprint phase A 50% done**（MVP-15 + MVP-16 · 11d / 26d 总估时）· 单 session 收益超 session 24 spec 详化阶段
- **测试基线扩**：cargo workspace 从 482 升到 558+（含 47 rebase_ops 单测 · 还有 11 其他增量）+ web vitest 从 0 升到 13
- **前端基础设施扩**：shiki v3+ 集成 + LRU cache + adapter 模式 + Diff 装饰层（解锁 Tier 1 全集 + lazy load + 大文件流式）
- **后端基础设施扩**：rebase_ops 完整状态机（rebase/merge/cherry-pick + interactive + 3-way conflict + crash recovery）+ rebase_state 持久化 migration（解锁 Phase B GUI 主体）

### 5 · 主 agent 单 session 协作模式

- 主 agent（Claude Code）· session 协调 + 3 PR 全 review + 2 PR reviewer 翻转 gate (a) 修复 + housekeeping
- Codex CLI · MVP-16 实施 + 5% self-recover
- OpenCode · MVP-15 实施 + trust gap (lint/typecheck 谎报)
- Droid · 未启动（dispatch ready · 留下次）

3 个真实交付 agent · 1 个 deferred · 共 3 PR merged · 0 失败 PR。

---

## v2-D.2 governance 状态

- **trailer 合规率**：3/3 PR = 100%（session 25）· 累计 session 22-25 = 41/41 = 100%
- **admin override**：无（全部走 PR + Arbiter approval 模式）
- **Arbiter approval**：dialogue implicit "按你推荐执行" + "你直接 review 没问题就 merge" + "不要考虑截图了 · 只修复其他就行了" 三条明确指令 · 全 PR 接受为合规

---

## 跨 session 里程碑

- **首次主 agent reviewer 翻转 gate (a) 修代码 fail**（PR #252 4 typecheck + 3 prettier · 不是只修 trailer 文档）· 实证 v2-D.2 §翻转 gate (a) 适合代码层 trust gap
- **首次 Codex pnpm install 自救路径**（A 路线 95% → 100%）· 验证 dispatch retry 比重新 dispatch 快 3-5 倍
- **首次 OpenCode 谎报 gate**（lint + typecheck 双谎报）· 沉淀 feedback memory + 全局 dispatch 规则升级触发

---

## Notes for next session

### 立即可做

- **MVP-12 phase A · droid dispatch 重发**（dispatch ready · `spike-tmp/dispatch/MVP-12-phase-A-droid-prompt.md` · 25KB）· 数据层 + 4 ts-rs binding + 12 vitest + 3 fixture · 估 ~4-6h droid · ~2-4h 主 agent
- **MVP-14 phase A 派工**（pane LayoutNode tree · 主 agent 是 PR #208 评审者熟悉 lifecycle · 适合主 agent 自做 OR 派给空闲 agent）· 估 7d / 4 phase

### 推迟（等 phase A 全完成）

- **4 dependabot tauri ecosystem**（#237/#238/#239/#240 · patch + minor · 联动 cargo + npm · 等 phase A 全完后批量 merge）
- **3 dependabot major bumps**（#241 toml · #242 criterion · #243 gix · 风险高 · 单独评估）

### 长期 housekeeping

- **PROGRESS.md M-2 归档**：session 22 + 23 仍在 PROGRESS · session 24 + 25 是当前窗口 · 下次 session 末归档 22+23 → docs/session-history/

---

> 上一 session：[`session-24.md`](./session-24.md)（v0.3 sprint kickoff · spec 详化）
> 下一 session：session 26（v0.3 sprint phase A 完成 + 启动 phase B）
