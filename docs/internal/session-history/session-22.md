# Session 22 · 2026-04-30

**session**: 22
**date**: 2026-04-30（单 day · MVP-20 PTY 预热池全 5 phase · Codex CLI fast 主导 A1/A2+A3 · 主 agent 协调）
**pr_range**: #189-#194（5 个 spec / implementation PR + Phase D runtime evidence PR · session 23 rename trace：MVP-20 → MVP-22）
**theme**: PTY 预热池从 spec ready 到 Phase D done 的一日闭环 · 用户痛点 cold spawn 800-1200ms → warm hit 0.09ms backend / 估 30-50ms 用户感知 · Codex CLI fast 用约 2.5h 完成 spec 估 8-10h 的后端核心段

---

## 主题摘要

### 1 · PTY 预热池全 5 phase

session 22 主线 = 把“新 tab 启动卡 1-2 秒”的高频痛点拆成可验证的 PTY warm pool 方案，并在同一天跑完 spec → backend core → lifecycle → settings UI → IPC integration → Phase D evidence。

当时任务 ID 为 **MVP-20**。session 23 因 v1.0 占位冲突做 housekeeping rename，历史 trace 改为 **MVP-22**；本 archive 保留实施期 PR 标题里的 MVP-20，不回写历史。

实测核心结果来自 `docs/PROGRESS.md` session 22 段：

- cold spawn：800-1200ms
- warm hit backend benchmark：0.09ms
- 用户感知估算：约 30-50ms
- Codex CLI fast 总用时：约 2.5h
- 原 spec 估时：8-10h（session 记录口径为约 5x 提速）

### 2 · 5 个 spec / implementation PR merged

| PR   | merge commit | merged at                 | merge author | 实施 / 协作归属 | 摘要                                                       |
| ---- | ------------ | ------------------------- | ------------ | --------------- | ---------------------------------------------------------- |
| #189 | `427436e`    | 2026-04-30T20:41:28+08:00 | Leafile Lune | 主 agent + Kimi | spec ready · Kimi 远程 review 5 维度 · 3 Blocker + 6 修订  |
| #190 | `4f5969e`    | 2026-04-30T21:02:28+08:00 | Leafile Lune | 主 agent        | Phase C Settings UI · `pty_pool_enabled` / `pty_pool_size` |
| #191 | `b2b7eb2`    | 2026-04-30T21:03:00+08:00 | Leafile Lune | Codex CLI fast  | Phase A1 PtyPool core · 8 单测 · `tab_id` 支持 rename      |
| #192 | `099095f`    | 2026-04-30T21:15:46+08:00 | Leafile Lune | Codex CLI fast  | Phase A2+A3 lifecycle + `cd` 注入 · 18 单测全集            |
| #193 | `c614352`    | 2026-04-30T21:39:36+08:00 | Leafile Lune | 主 agent        | Phase B IPC 接入 · tab/pane spawn take-first               |

#### PR #189 · spec ready

- branch / scope：`docs/MVP-20-pty-warm-pool-spec`
- 内容：新增 PTY 预热池 spec，约 +166 行
- review：Kimi（Moonshot 远程 API）只做 spec review
- 结果：5 维度 20 分钟输出 review，3 个 Blocker + 6 个 High/Medium 全修订

#### PR #190 · Phase C Settings UI

- branch / scope：`feat/MVP-20-C-settings-ui`
- 内容：设置项字段 `pty_pool_enabled` / `pty_pool_size`
- UI：TerminalGroup toggle + 容量选择器
- 配套：ts-rs binding sync
- 实施：主 agent

#### PR #191 · Phase A1 PtyPool core

- branch / scope：`feat/MVP-20-A1-pool-core`
- 内容：`pty_pool.rs` 约 370 行
- 核心类型：`PoolConfig` / `PtyPool` / `IdlePty` / `TakeResult`
- 核心 API：`take` / `refill` / `kill_all` / `set_size`
- 测试：8 个单测
- 关键设计：`PtySession.tab_id` 改为 `parking_lot::Mutex<String>`，支持 warm hit 后 rename
- 实施：Codex CLI fast，约 1.5h，含 fmt baseline 修复

#### PR #192 · Phase A2+A3 lifecycle + cd 注入

- branch / scope：`feat/MVP-20-A2-A3-pool-runtime`
- lifecycle：5min idle expire timer
- runtime 选择：`crossbeam recv_timeout`，不引入 tokio
- 配置 API：`apply_config_change` / `handle_default_shell_change` / `shutdown`
- cwd 注入：`inject_cd_clear`
- shell 兼容：`cd -- 'path'; clear\n`，覆盖 zsh / bash / fish 的 POSIX 路径
- 测试：18 个单测全集
- 实施：Codex CLI fast 自主 commit / push / PR

#### PR #193 · Phase B IPC 接入

- branch / scope：`feat/MVP-20-B-pty-pool-ipc`
- backend wiring：`AppState` 加 `pty_pool` / `pane_pty_pool: Arc<PtyPool>`
- init：`run()` 初始化 + `workspace_init` 读取 pool config
- settings：`settings_update` hook
- spawn path：`tab_pty_spawn` / `pane_pty_spawn` take-first
- regression：354 tests 不破坏
- 实施：主 agent

### 3 · Phase D · runtime evidence + spec done

Phase D 独立收尾 PR：

| PR   | merge commit | merged at                 | merge author | 摘要                                 |
| ---- | ------------ | ------------------------- | ------------ | ------------------------------------ |
| #194 | `2f0185f`    | 2026-04-30T22:11:29+08:00 | Leafile Lune | Phase D runtime evidence + spec done |

runtime evidence 内容：

- backend benchmark：`crates/core/tests/pty_pool_bench.rs`
- benchmark 覆盖：cold / warm / disabled 3 个路径
- evidence 目录：`docs/runtime-evidence/mvp-20/`
- evidence 文件：`README` / `00-baseline` / `01-warm-hit` / `02-cold-path` / `03-settings-toggle`
- acceptance：11 项全部 `[x]`
- spec 状态：`ready` → `done`

Phase D 还调整了 spec A10 的证据口径：从“3 段录屏”改为“backend benchmark + 单测 + frontend baseline”。原因是单人项目 v2-D.1 模式下，视频对自动化验证无增量价值；偏离已在 spec / evidence 中透明记录。

### 4 · 协作模式 · 双 agent 并发 + Kimi 远程 review

| 角色           | 承担范围                                  | 证据 / 特征                                           | session 22 结论                               |
| -------------- | ----------------------------------------- | ----------------------------------------------------- | --------------------------------------------- |
| 主 agent       | 协调 / spec / Phase B / Phase C / Phase D | 串联全流程、settings UI、IPC wiring、runtime evidence | 负责跨 phase 收口和证据口径判断               |
| Codex CLI fast | Phase A1 / Phase A2+A3                    | 独立 worktree · `codex exec --skip-git-repo-check -`  | 后端核心实现从 8-10h 估时压缩到约 2.5h        |
| Kimi           | spec review only                          | 远程 API · 5 维度 · 20 min review                     | 适合无 worktree access 的 spec-only 独立审查  |
| Arbiter / 用户 | 验收与治理拍板                            | v2-D.1 trailer / 单人项目模式                         | 5/5 PR + Phase D PR trailer 合规率回升到 100% |

关键点不是简单“更多 agent”，而是把任务拆成三类：

- spec review：可远程、只读、上下文可随 prompt 提供
- Rust core：可独立 worktree、接口清晰、适合 Codex CLI fast
- wiring / evidence：依赖全局上下文和用户验收口径，由主 agent 收口

---

## 关键经验沉淀

### A · Codex CLI fast 的有效边界

Codex CLI fast 在 Phase A1 / A2+A3 的表现是 session 22 最大方法论收益。它适合输入明确、验收可测、模块边界稳定的 Rust 后端核心任务。

本次实证：

- spec 估时：8-10h
- 实际 fast 后端核心段：约 2.5h
- Phase A1：1.5h 完成 core + 单测 + fmt baseline
- Phase A2+A3：继续自主完成 lifecycle / cd 注入 / commit / push / PR

沉淀：fast mode 不等于降低验证要求；它需要更硬的 spec、独立 worktree、可复跑的 tests 和明确的 commit / PR trailer。

### B · Kimi 远程 API spec review 可用

Kimi 没有本地 worktree access，因此不适合实现或需要 repo grep 的任务。但 session 22 证明：当 prompt 附足 spec 原文和 review 维度，它可以做有效的 spec-only 审查。

本次价值：

- 20 分钟返回 5 维度 review
- 找出 3 Blocker + 6 High/Medium
- 修订后 spec 才进入 implementation phase

沉淀：Kimi 适合“独立评审者”，不适合“实现者”；适合“读给它的材料”，不适合“让它自己找上下文”。

### C · v2-D.1 trailer 合规率回升

session 21 因 GitHub Actions billing 暂停触发 admin override，大量 direct push 让 trailer 和 PR 流程处于历史低点。session 22 重新回到 PR-based flow。

本 session 结果：

- 5/5 spec / implementation PR 合规
- Phase D PR 合规
- admin override 模式停用
- trailer 合规率回升到 100%

沉淀：只要 GitHub PR 流程可用，就不应让 direct push 成为习惯；即使是单人项目，也要保留 Implemented / Reviewed / Arbiter approval 的可追溯记录。

### D · backend benchmark 比纯录屏更适合 PTY warm pool

PTY warm pool 的核心收益发生在 spawn path 和 pool hit path。对这个场景，backend benchmark + 单测 + frontend baseline 比“3 段录屏”更能证明核心行为。

本次 Phase D 的证据结构：

- cold path 有 benchmark
- warm hit 有 benchmark
- disabled path 有 benchmark
- settings toggle 有 baseline
- acceptance 11 项全部勾选

沉淀：证据形式应服务于风险本身，而不是机械执行早期 spec 文案。

---

## 反思

- **MVP-20 / MVP-22 rename trace 必须保留**：session 22 实施时所有 PR 都是 MVP-20；session 23 因 v1.0 占位冲突 rename 为 MVP-22。archive 应保留这段历史，而不是把旧 PR 标题改写成新 ID。
- **双 agent 并发不是无脑并发**：Codex CLI fast 只接 Phase A1/A2+A3，主 agent 保留 spec / UI / IPC / evidence 收口权，这才避免跨文件域冲突。
- **fast mode 的收益来自约束密度**：能按 session 记录口径 5x 缩短实现时间，是因为 spec、边界、单测和 worktree 全部清晰；不是因为跳过验证。
- **远程 review 的输入质量决定输出质量**：Kimi 的 5 维度 review 有价值，是因为 prompt 给了 spec 原文和明确审查维度；缺少上下文时不能期待它补全 repo reality。
- **v2-D.1 合规恢复是治理节点**：session 21 的 admin override 是环境异常，不应沉淀为常态；session 22 重新证明 PR + trailer 路径可执行。
- **Phase D 证据口径可以调整，但必须透明**：A10 从录屏改成 benchmark + 单测 + baseline 是合理偏离，关键是记录偏离原因和验收覆盖。

---

## 关联

- 上一 session：[`session-21.md`](./session-21.md)（PR #173-#187 · v0.1.0 GA 发布 + v0.1.1 双批 fix + admin override 事件）
- 下一 session：[`session-23.md`](./session-23.md)（待 Cursor 同期归档 · MVP-20 → MVP-22 rename trace 会在该 session 继续出现）
- rename 证据：`64a94e5` · 2026-05-03T10:30:41+08:00 · `chore(tasks): session 23 housekeeping · MVP-05 状态对齐 + MVP-20 → MVP-22 rename (#212)`
- 当前事实源：`docs/PROGRESS.md` session 22 段 L179-L210
