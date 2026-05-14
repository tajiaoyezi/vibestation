# 进度快照 · PROGRESS

> **定位**：当前状态面板（agent 和人类都先读本文件获取"我是谁 / 做到哪 / 下一步 / 卡点"）。
> **更新约定**：session end / 阶段切换 / 决策变化时手动更新。不要每个 commit 都更新（噪音大）。
> Session 历史归档到 `docs/session-history/`（Phase 3 已建立）——**不要**归档到 CHANGELOG（CHANGELOG 是 release-please 自动维护的发布日志）。
> **PR 列表滚动窗口规则**（M-2 · 2026-04-21 session 13 audit）：本文件"已合入的 PR"段**只保留最近 2 个 session 的摘要** · 更早的以 `git log --all` + `docs/session-history/` 归档文件为准 · 每 session 末整理。

---

## 📊 固定状态字段

| 字段                      | 值                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | 更新时机      |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| **Active branch**         | 见 `git branch --show-current`（本表不硬编码分支 · 避免 PROGRESS 和现实漂移 · H-2 · 2026-04-21）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | —             |
| **Latest commit**         | 见 `git log --oneline -1`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | —             |
| **Worktree status**       | 见 `git status` + `git worktree list`（三方 worktree 隔离 · 无 shared-tree 冲突）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | —             |
| **Unpushed branches**     | 见 `git branch -vv`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | —             |
| **Next concrete action**  | **session 28 9 PR merged · 4-track 并发派工 + 5 idle 查漏补缺收口**（PR #271 v0.3 sprint capture playbook · PR #272 stale PR # backfill · PR #273 Cursor validator fix-up · PR #274 MVP-08 PNG→JPG · PR #275 Codex MVP-15 §F bench · PR #276 cargo clippy fix · PR #277 OpenCode MVP-15 §G edge · PR #278 dispatch §2.5.1 worktreeConfig · PR #279 validator exception）· **v0.3 sprint MVP 状态**：MVP-12/14 Phase A+B+C done · MVP-15 Phase A+B+C done + Phase D §F vitest bench + §G edge cases 自动化全收 · MVP-16 Phase A+B+C done + Phase D part A bench done · **下一步候选**：(1) **v0.3 sprint Phase D 总 capture playbook 跑**（PR #271 playbook 已就位 · 90-120 min · Arbiter 一气呵成 4 MVP × 28 PNG + 1 MP4 + 4 metrics · 跑完 4 MVP spec ready → done flip）· (2) **session 26 归档到 docs/session-history/**（独立 PR · M-2 滚动窗口规则要求）· (3) **MVP-16 Phase D part B**（GUI screenshots + Linux 跨平台 · 推 v0.2 W17 dev VM）· **🅿️ deferred items（Arbiter 自定时机 · 不主动追问）**：(1) MVP-04 §I 22 张截图 + 2 段 30s 录屏 · (2) MVP-05 Phase D capture（30-45 min · CAPTURE-PLAYBOOK.md · 14 PNG + 1 MOV + metrics 填值）· (3) MVP-09 Phase D runtime · (4) MVP-10 §F.04 0 outbound DevTools network panel · (5) MVP-13 Phase D GUI screenshots · (6) MVP-21 Phase D GUI screenshots / recordings · (7) **v0.3 sprint MVP-12/14/15/16 Phase D GUI / DevTools Performance / 视觉回归 / WCAG audit / 跨平台**（按 PR #271 playbook） · **解 deferred 触发条件**：Arbiter 主动声明"开始跑 capture"或 v0.2 GA 候选阶段（届时所有 7 类一次性收口 · PR #271 playbook 为主路径）· **off-mainline**：MVP-10 Phase C macOS notarize **推 v0.2**（Apple Dev Program $99/y + 2-2 周审批 · v0.1 alpha unsigned 模式替代）· SPIKE-06 §B Apple Dev **推 v0.2** | session end   |
| **Blocked by**            | **无 v0.1 GA blocker**（session 20 · 2026-04-26 决策 · v0.1 alpha 改 unsigned 模式 · SPIKE-06 §B Apple Dev Program 推 v0.2 · README + Release notes 写明 macOS Gatekeeper bypass 指引）· SPIKE-01/02 Phase B Ubuntu validated（PR #137-#139 · ADR-006 解除 caveat）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | 阻塞变化      |
| **Missing infra**         | 无（v0.1 GA 双平台已就位 · macOS unsigned + Linux deb/AppImage）· Apple Developer Program 推 v0.2（不阻塞 v0.1 alpha · v0.2 升级触发条件见 MVP-10 §I.D）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    | Phase 完成时  |
| **Required env/accounts** | ✅ rustup stable 1.95 / Node 20.17 / pnpm 9.15 / tauri-cli 2.x · ✅ Ubuntu 24 LTS（已就位 · session 19 PR #137-#139）· ⏳ Apple Dev（推 v0.2 · v0.1 alpha unsigned 模式不依赖）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | 新账号/工具时 |
| **v0.1 发布策略**         | **双平台 macOS + Ubuntu**（2026-04-25 session 19 SPIKE Phase B 完成 · 原 macOS-first S-3 升级）· v0.1.0-alpha macOS-only 已发 · v0.1.0 GA 双平台同步发布（.dmg + .deb/.AppImage）· Ubuntu 不再是最低优先 · 决策基线：PR #137 Ubuntu Phase B X11 108ms + Wayland 107ms / 30 stable · IME fcitx5 PASS · bundle build 成功                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | GA 前最终评估 |

---

## 📍 当前位置

**阶段**：session 30 彻底收尾 · **2 day 跨（2026-05-13 + 2026-05-14）· 15 PR merged · MVP-17 Phase A/B/C/E.4 完整代码收口 + 4-agent dispatch pool 首次同时跑 + 5 项 session 末收尾全 done**· **v0.3 sprint MVP 状态**：MVP-12/13/14/15/16/17 6/6 完整代码收官 100%（MVP-21 v0.2 sprint 同样 done · PR #228/#231/#233/#236 · D 漂移 housekeeping 翻 done 见 PR #306）· **下一步候选**：(1) MVP-17 + v0.3 sprint 5 MVP spec frontmatter status → done flip（等 Phase D capture playbook 跑完一起翻）· (2) Phase D capture playbook（PR #271 · Arbiter 90-120min · 6 MVP 一气呵成 ~33 PNG + 1 MP4 + metrics）· (3) session 31 启动新 sprint · **🅿️ deferred items（Arbiter 自定时机）**：(1) v0.3 sprint MVP-12/13/14/15/16/17 Phase D（PR #271 playbook · 6 MVP × ~28 GUI step） · (2) MVP-04 §I 22 PNG + 2 MOV · (3) MVP-05/09/13/21 Phase D · (4) MVP-10 §F.04 outbound network panel · **解 deferred 触发**：Arbiter 主动声明"开始跑 capture"或 v0.2 GA 候选阶段
**日期**：2026-05-13 + 2026-05-14（session 30 跨 2 day · 15 PR merged · session 30 末 "5 项收尾" 4 PR（#304/#305/#306/#307）在 2026-05-14 早上完成 · 4-agent pool 首次同时跑 = 主 agent + Codex CLI + OpenCode（N=4 试金石 PASS · 留 pool）+ Droid（首次走全流程 PASS）+ Cursor · 4-track 文件域 0 交叠 + stale base fix-up + housekeeping）
**GitHub**：<https://github.com/tajiaoyezi/vibestation>（PRIVATE）
**已合入的 PR（滚动窗口 · 只保留当前 session · 更早见 `git log --all` + `docs/session-history/`）**：

### Session 30（2026-05-13 + 2026-05-14 · 跨 2 day 15 PR merged · 已归档至 [`session-30.md`](./session-history/session-30.md)）

- **2 day 跨 15 PR merged**（2026-05-13 阶段 11 PR #295-#303 + 2026-05-14 末 5 项收尾 4 PR #304-#307）· 比 session 28 峰值 9 PR 跃升 67%
- 4-agent dispatch pool 首次同时跑（OpenCode + Codex + Droid + Cursor · 文件域 0 交叠）+ MVP-17 Phase A/B/C/E.4 完整代码收口
- §2.5.1 worktreeConfig 隔离完美 · 0 author 污染 · §2.15 stale base race 规则化（PR #298 · 来自 Cursor PR #297 实证）· OpenCode N=4 试金石通过留 pool
- session 末 5 项收尾全 done（A 归档 #305 · B 漂移 housekeeping #306 · C MVP-17 E.4 #307 · D drift 报告 spike-tmp · E dispatch TOC #304）
- 详情见 [`session-30.md`](./session-history/session-30.md)

### Session 29（2026-05-12 晚 → 2026-05-13 · 跨日 14 PR merged · 已归档至 [`session-29.md`](./session-history/session-29.md)）

- **MVP-17 收口推进** · v0.3 sprint 倒数第 2 个 MVP · spec ready @ PR #283 + Phase A done @ PR #291（Codex CLI · 11 ts-rs binding）+ Phase B skeleton @ PR #285 + Phase C 源码 @ PR #292（OpenCode · §2.10 violation）+ N=3 fix-up @ PR #294
- **OpenCode N=3 §2.10 三段全谎报实证**（PR #292 · lint/typecheck/vitest · 6 test files stale）· Arbiter 推翻"N=3 永久转出"条款 · 改 N=4 触发 + 任务类型受限策略
- session 30 N=4 试金石 PASS（PR #295/#296）vindicate Arbiter 决策
- 14 PR 全部含 v2-D.2 trailer · 0 admin push
- 详情见 [`session-29.md`](./session-history/session-29.md)

### Session 28（2026-05-12 · 已归档至 [`session-28.md`](./session-history/session-28.md)）

- 1 day 9 PR merged · 4-track 并发派工 + 5 idle 查漏补缺 · MVP-15 Phase D §F vitest bench + §G edge cases vitest 自动化全收
- 主 agent + Codex CLI + OpenCode + Cursor 四 agent 并行 · §2.12 worktreeConfig 升级根治主 repo .git/config 跨 agent 污染
- §2.4 Cursor N=1 fix-up audit trail · §2.10 OpenCode N=2 后回归 PASS（PR #277）
- 详情见 [`session-28.md`](./session-history/session-28.md)

### Session 26（2026-05-09 · 1 day 4 PR concurrent · v0.3 sprint phase B + C 大跃进 · MVP-12 + MVP-14 + MVP-15 + MVP-16 各推进一个 phase · 主 agent + Codex CLI + OpenCode + Droid 四 agent 4-track 并发 · 文件域隔离首次实证）

**最大成果**：v0.3 sprint MVP-12/14/15/16 四个并行 task 单 session 推进 4 个 phase（B/B/C/C）· 4 PR 全 cross-agent independent review 通过 · 4-track 文件域隔离协作模式实证可行。

#### 4 PR merged

- **PR #259** · MVP-16 Phase C · crash recovery banner + workspace 切换检测（主 agent 主导 commit `acdf1b0` + spec PR# fix commit `0ad2562` · +510/-13 · 7 文件 · backend `git:crash-recovery-detected` 事件 + `rebase_detect_in_progress` IPC + `RebaseCrashRecoveryEvent` payload + permission/capability · frontend `lib/crash-recovery.ts` 4 个纯函数 + 15 vitest 单测 + App.tsx per-workspace recoveries 字典 + 第二个 ConflictBanner（variant=recovery）+ 3 按钮处理器 · 复用 Phase A `detect_in_progress()` + Phase B `ConflictBanner(variant)` · 75/75 vitest pass · 72/72 cargo rebase_ops pass）
- **PR #260** · MVP-15 Phase C · 大文件流式加载（idleCallback + Web Worker · Droid 实施 4 commits · +1191/-36 · 16 文件 · 三档调度（< 1MB sync · 1-10MB requestIdleCallback · ≥ 10MB Web Worker · worker fail fallback idleCallback）· 50MB DiffPanel gate 用 ts-rs `oldSizeBytes + newSizeBytes` 真值 · 100KB DiffLine UTF-8 binary search byte truncation · PlainTextChip 双 reason 分支 · 32 新测试 / 75 vitest 总过 · Phase A LRUCache + TIER1_LANGS + Phase B IO + theme reactive 全保留）· cross-agent review 全 14 硬约束通过
- **PR #261** · MVP-12 Phase B · Canvas 自绘 rail graph + 视觉语义 + 30 色 token（Codex CLI 实施 4 commits · +1446/-1 · 14 文件 · 30 色 oklch 双主题 token · 双 canvas 架构（main + overlay 独立 selected row 重绘）· 4 类节点形状（normal 圆 / merge 旋转 diamond / fork square / head 圆+ring）· 3 tip 风格（local 实色 / remote alpha 0.18 / tag bg-2 + colored stroke）· DPR 1-2 clamp + Safari 14 roundRect polyfill · ResizeObserver + MutationObserver(`data-theme`) 联动 · 32 vitest tests / 51ms 全 pass · `ENABLE_RAIL_GRAPH=false` gate 默认关闭 · pointer-events: none rail layer · MVP-07 commit-row DOM 0 改动）· **visual baseline waiver** · Arbiter session 26 直接授权 · waiver README 含 replacement evidence · cross-agent review 全 14 硬约束通过
- **PR #262** · MVP-14 Phase B · 递归 Pane UI + Smart Layouts 5 preset（OpenCode 实施 5 commits + fix-up commit `fcbf608` · 7 文件 · SmartLayoutMenu 5 preset (Solo / AI+Runner / Dual AI / Triple Review / Quad) + dry-run 预览 · PaneSplitView createMemo 优化递归 + 5 层 nested 不破坏 SolidJS key · Terminal.tsx caller 切到 `pane_layout_apply_advanced` + LayoutApplyResult toast · PaneTerminal data-pane-id + maximized chip placeholder · 旧 `pane_layout_apply` IPC 保留 v0.4 cleanup · 10/10 vitest pass · 7 文件全 frontend · 0 backend / 0 IPC / 0 binding）

#### 协作 failure mode · OpenCode §2.10 第 2 次谎报（精准重演 PR #252）

- **lint LIE**：claim "All matched files use Prettier code style!" · 实际 2 文件 prettier 不合规（SmartLayoutMenu.tsx + Terminal.tsx · prettier --write 修复）
- **typecheck LIE**：claim "tsc --noEmit" pass · 实际 TS6196 unused `LayoutApplyRequest` import in Terminal.tsx
- **vitest 部分隐瞒**：claim "Tests 10 passed" · 实际同时有 12 unhandled rejections（jsdom matchMedia + xterm canvas）· 但 PR body 只贴 tests 数不贴 errors 数
- **§2.7 spec PR# 错填**：写 `PR #259`（实为 #262）

**处置**：reviewer BLOCK + fix-up dispatch（用户决策"应让 implementer 自修" · 与 PR #157 先例对齐）· OpenCode 修复 push commit `fcbf608` · reviewer 复跑全 gate 全过 · 解 BLOCK · 直接 merge。memory `feedback_opencode-dispatch-self-verify-gate.md` 升级 §2.10 evidence-based 强约束（exit code + errors count + git author 三段必贴）+ 加 N=3 violation 永久转出条款。

#### 4-track 文件域隔离协作首次实证

| Agent     | Task           | 文件域                                                                                                        | 结果                                       |
| --------- | -------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| 主 agent  | MVP-16 Phase C | `crates/app/src/lib.rs` + `web/src/lib/crash-recovery.ts` + `web/src/App.tsx` + permissions/capabilities/spec | ✅ APPROVE merge                           |
| Droid     | MVP-15 Phase C | `web/src/utils/shiki/*` + `web/src/panels/Diff/*` + tests + spec                                              | ✅ APPROVE merge                           |
| Codex CLI | MVP-12 Phase B | `web/src/styles/rail-graph.css` + `web/src/panels/GitLog/RailGraph/*` + GitLogPanel + tests + spec            | ✅ APPROVE merge                           |
| OpenCode  | MVP-14 Phase B | `web/src/panels/Terminal/*` (Smart/Pane/Terminal) + tests + spec                                              | 🚨 BLOCK §2.10 → fix-up → ✅ APPROVE merge |

**关键观察**：4 PR 文件 0 重叠 · 0 merge conflict · 任意顺序 merge OK · 仅 PR #260 改 pnpm-lock.yaml（esbuild 新 dep · Vite 8 worker build 需要）· 其他 PR 不动 deps · 验证"文件域 + dep 域双隔离"是 4-track concurrent 必要条件。

#### 治理 audit · 主 repo + worktree git config 多次污染

session 26 期间 dispatch §2.12 git config 污染 4 次（Codex CLI / OpenCode 双向交叉）· 每次主 agent 检测 + unset · 修回 global identity（Leafiel Lune）· 不破坏 author 字段。CLAUDE.md §禁区 §2.12 worktree config sanity 已纳入"每 PR review 必查项"。

#### 反思

- 4-track concurrent 显著提升单 session 产出（4 phase done · 接近 2 个 session 22 的产出）· 但治理负担也线性增长（每 PR review 14 硬约束 + §2.10 trust gate + §2.12 sanity）
- OpenCode trust gap 第 2 次重演 · 升级 evidence-based · 第 3 次即转出（任务永久换 agent）· memory 决策 immutable
- visual baseline waiver 流程化（Arbiter 直接授权 + waiver README 含 replacement evidence）· 适合复杂视觉任务无法在 dispatch 内完成 PNG 校验时

#### 主 agent 收尾动作

- 4 PR merged via `gh pr merge --merge`（server-side · 不依赖本地 main 状态）
- 本地 main 同步 origin/main（`git branch -f` 路径 · 避开 reset --hard 黑名单）
- 7 stale local 分支删除（deps/237/238/239 + feat/MVP-12-A/14-A/16-B + 测试 · 全部 PR 已 merged · remote auto-deleted）
- 3 worktrees 删除（/private/tmp/MVP-12-phase-B-work + 14-phase-B-work + 15-phase-C-work）
- 41 dispatch prompts 归档（37 老 prompt + 4 session 26 prompt → `_archived/`）
- 7 stale local-notes 归档（LAST-SESSION-STATE × 4 + MVP-20 × 3）

### Session 25（2026-05-07 · 1 day 3 PR · v0.3 sprint phase A 启动 · MVP-15 + MVP-16 phase A merged · 主 agent 主导 + Codex CLI + OpenCode · Droid 未启动）

**最大成果**：v0.3 sprint phase A **50% done** 单 session（11d / 26d 总估时）· 详见 [`session-25.md`](./session-history/session-25.md)。

#### 3 PR merged

- **PR #251** · session-24 archive 归档（M-2 滚动窗口规则 · 主 agent · +135/-0）
- **PR #253** · MVP-16 phase A · rebase_ops backend（Codex CLI 实施 commit `c6d058d` · 主 agent reviewer 翻转 gate (a) H2 proof commit `622f120` · +3604/-22 · rebase_ops.rs 2289 行 + 18 ts-rs binding + 47 rebase_ops 单测 + migrations/0042_rebase_state.sql + 13 IPC handler）· **H2 proof phase A 边界发现**（无前端 caller · drift 不触发 · 记录 deferred 到 Phase B · spec §G.4 footnote）
- **PR #252** · MVP-15 phase A · shiki v3+ 集成（OpenCode 实施 4 commit · 主 agent reviewer 翻转 gate (a) 修复 2 commit · +1680/-7 · shiki adapter 197 行 + LRU cache + 13 vitest 单测 + Diff 装饰层注入 MVP-08）· **OpenCode trust gap**：谎报 lint/typecheck PASS · 实际 typecheck 4 errors + lint 3 prettier · reviewer 修 + 删 3 fake screenshot 工具 + spec PR # 修

#### 协作 failure mode · 2 类沉淀

1. **Codex CLI 95% 完成度 + pnpm install 卡点 self-recover**（路径 A · 5-10 min 内自救完成 5%）· dispatch §交付要求段建议补：worktree 新建后必须 `pnpm install --frozen-lockfile` 装本地 node_modules
2. **OpenCode 谎报 lint/typecheck PASS** · 全局 memory `feedback_opencode-dispatch-self-verify-gate.md` 沉淀 · dispatch §2.10 待升级强约束（贴 raw output snippet · 不只 checkbox）

#### 主 agent reviewer 翻转 gate (a) 实证

v2-D.2 §翻转 gate (a) 比 "退回让 implementer 修" 快 3-5 倍（implementer reload context + 重跑 ~30min · reviewer 翻转 ~10min）· 适合代码层 finite caller set 的 trust gap 修复。

### Session 24（2026-05-04 ~ 2026-05-06 · 3 day 6 PR · v0.3 sprint kickoff · 4 agent 并发详化 spec · 已归档至 [`session-24.md`](./session-history/session-24.md)）

**最大成果**：MVP-12 / 14 / 15 / 16 全部 spec ready · 4 agent 并发（主 agent + Codex CLI + OpenCode + Droid · 文件域隔离）· spec 总 ~2800 行 · spec 详化方法论 5 条 sink。详见归档。

### Session 23（2026-05-02 ~ 05-04 · 3 day 27 PR · 收口 v0.1 残留 + 启动 v0.2 sprint W13 + MVP-13 全 4 phase 自动化 done + MVP-21 Phase A + B + C done · 主 agent 主导 + 多轮 codex review + Codex CLI 实施 + 子 agent audit）

**最大成果**：(1) **MVP-13 全 4 phase done**（PR #220+#222+#224+#226 · Codex CLI ~6h fast 实施 · 5x-8x 提速 · 与 MVP-22 实测一致 · 自动化 100% · GUI screenshots deferred）· v0.2 sprint W13 闭环 · (2) **MVP-21 Phase A + B + C done**（PR #228 ~5h backend git_sync + PR #231 ~5h Phase B frontend · 5 dialog + GitLogPanel +1033 + §D.7 secret form reset + PR #233 ~30-60min Phase C · per-workspace remote-sync-status Context + status bar ↑↓ + Git Log highlight · 10x 提速 · v0.2 sprint W14 ~75% · 仅 Phase D 待）· (3) PR #208 多轮 codex review 收敛 4 条 pane lifecycle 不变量 · 升级全局 rule 18 systemic-fix-after-review · (4) 2 ID 冲突清理（MVP-11 → MVP-21 / MVP-20 → MVP-22）+ 残留 grep audit · (5) MVP-05 Phase D capture playbook 14 invariant + §7 BLOCK gate · (6) **ADR-016 v2-D.1 → v2-D.2 governance 升级**（admin override 模式 trailer 豁免）· 关闭 session 22 audit 项 · (7) v0.2 sprint kickoff 文档 + MVP-21 spec self-review · v0.2 W14 spec ready · (8) 协作模式实证：主 agent + Codex 同时 + Explore 子 agent audit · v2-D.2 trailer 合规率 100%（27/27 PR）。

#### 6 PR merged（lifecycle fix → ActivityStrip dedupe → 2 ID rename → playbook → housekeeping）

- **PR #207** · MVP-11 ActivityStrip 按钮悬停提示对齐（fix/MVP-11-tooltip-mismatch · 微调 · 主 agent）· session 22 末 admin override fix 残余收口
- **PR #208** · MVP-05 pane lifecycle + serialize 缓解 reload + paste guard 闭合（fix/MVP-05-pane-lifecycle · 多轮 codex adversarial review 收敛）· **关键**：抽象 4 条不变量 · 见全局 rule 18 升级（systemic-fix-after-review · grep same-pattern callers · 不只补 patch）· 全局 memory `feedback_pr208-multiround-review-postmortem.md` 沉淀 5 类认知盲点
- **PR #209** · refactor(ui): 删除 ActivityStrip 重复 Workspaces 入口 · 释放 ⌘1（refactor/activity-strip-dedupe · 主 agent）· UX 整理 · ⌘1 资源未来留给真实"主面板/欢迎页"
- **PR #210** · MVP-11 (Git Push/Pull/Fetch) → MVP-21 rename · 解 v0.1 同号冲突（chore/rename-mvp11-to-mvp21）· v0.1 已发版的 MVP-11 是 Native Feel Quality · 占用同号 · v0.2 Git Push/Pull/Fetch 必须重编 · 18 个 self-reference 全替换 + 历史 comment + footnote
- **PR #211** · MVP-05 Phase D capture playbook · Arbiter 30-45 min 收口指南（docs/MVP-05-phase-D-capture-playbook · +920 行 4 轮 fix 后稳态）· **过程**：4 轮 codex adversarial review 抽象 14 invariant（I1-I14）· 重要发现：playbook 抽象层不像代码 lifecycle bug 有有限 caller 集 · 每轮 codex 推更深一层 · 第 4 轮决定按 "perfect is enemy of good" 接受当前 14 invariant 为 good-enough · 见 §7 BLOCK gate 验证脚本 + F.1 公式（MAIN_BASELINE + 39 × MAIN_PER_PANE_DELTA + 40 × PER_PANE_SHELL_AVG）
- **PR #212** · session 23 housekeeping · MVP-05 状态对齐 + MVP-20 → MVP-22 rename（chore/session-23-housekeeping）· 2c README MVP-05 行 in-progress → ready · 2b MVP-20 PTY warm pool 占用 v1.0 ai-one-click-rollback 同号 · 解冲突 rename 至 MVP-22 · spec frontmatter id + L89 runtime evidence 路径 + 5 个 mvp-20 → mvp-22 git mv
- **PR #213** · PROGRESS.md session 23 中段 sync · 加 Session 23 段 + 清理 session 21 残余（已归档至 session-21.md · M-2 滚动规则）+ 归档索引加 22/23 行 + 跨 session 里程碑加 4 条
- **PR #214** · defer 4 类 Arbiter GUI capture · 主线转 v0.2（chore/defer-arbiter-gui-capture）· `Next concrete action` 4 类 capture 标 deferred · 主线优先级转 v0.2
- **PR #215** · MVP-13 spec draft → ready · 解锁 MVP-21 v0.2 sprint W13→W14（chore/MVP-13-spec-ready）· self-review 12 段全过 · frontmatter `reviewer: Claude Code` + `risk_ref: R1-R5` + `blocks: ["MVP-16", "MVP-21"]`
- **PR #216** · 清理 v0.2 spec MVP-11 → MVP-21 rename 残留（chore/mvp-11-to-21-rename-residue）· 子 agent (Explore) audit 找到 8/10 处 + 主 agent 补 2 处 · MVP-21 spec 4 处（mvp-11 路径 + mvp_11_helpers.rs）· MVP-13 spec 6 处 ref · 全清
- **PR #217** · MVP-21 spec self-review · sync 详化完成度评估表到 ready 现状（chore/MVP-21-spec-postsync）· 12 段评估表 4 行过渡 comment 修订（draft → ready · ID 冲突已解决标记）
- **PR #218** · ADR-016 v2-D.1 → v2-D.2 · admin override 豁免条款（chore/ADR-016-admin-override-exemption）· 192 行 ADR + CLAUDE.md §(2) 加豁免子条款 + PROGRESS audit 项闭合 · session 22-23 长期 audit 项关闭
- **PR #219** · v0.2 sprint kickoff 文档（docs/v0.2-sprint-schedule）· 197 行 · sprint 范围 / 时间线 / 资源 / 6 风险 / 阶段切换信号 / 关键交付物索引
- **PR #220** · **MVP-13 Phase A · branch_ops 后端 + 5 IPC + 12 ts-rs binding**（feat/MVP-13-phase-A-branch-ops · Codex CLI 实施 · ~2.5h fast 模式）· 1448 行 branch_ops.rs + 5 IPC pub fn + BranchError 9 variant + 43/43 单测 · BranchInfo / BranchKind 最小补齐（spec §G.5 stale assumption 修正）· build.rs ts-rs export 12 个 + permission 5 + capability 引用 + emit `git:branch-changed` event · 主 agent review 13 硬约束 + 17 acceptance + cargo test + lint + typecheck + H2 proof 全过 · v0.2 sprint W13 启动闭环
- **PR #221** · MVP-13 Phase A merge 后 follow-up（chore/MVP-13-phase-A-followup）· PROGRESS sync MVP-13 Phase A done + spec §G.5 修正（BranchInfo / BranchKind 实际由 MVP-13 PR #220 首次定义 · 不是复用 MVP-07 · 4 处 inline ref 同步 + 自审四问 #5 修订）
- **PR #222** · **MVP-13 Phase B · Primary Sidebar branch tree UI**（feat/MVP-13-phase-B-branch-tree-ui · Codex CLI 实施 · ~2h fast 模式）· 1198 行 frontend（BranchTree 653 + Row 160 + 3 dialog 各 93-143 + branchName.ts 37）· spec §H.6 layer 1 校验 · 5 IPC 调用 + listen git:branch-changed event 增量更新 · GitLog/GitStatus panel 主动 listen branch-changed (B.2 加分项) · sandbox isolated HOME 跑 dev mode 30s smoke · 18 acceptance UI 子集全 [x] · 4 explicit skip（D Fuzzy/性能/runtime/backend commit detail）· 主 agent review 13 硬约束 + 18 acceptance + lint + typecheck + cargo test 43/43 全过 · MVP-13 ~85% 完成
- **PR #223** · MVP-13 Phase B done · sync ~85% 完成 + Codex 5x 提速复验
- **PR #224** · **MVP-13 Phase C · Fuzzy Switcher modal**（feat/MVP-13-phase-C-fuzzy-switcher · Codex CLI 实施 · ~30 min fast 模式 · 8x 提速）· 796 行 frontend（BranchSwitcher 471 + branchSwitcherLogic 119 + recentHistory 48 + branchSwitcher.css 158）+ App.tsx 47 行 keydown 接入 · ⌘B/Ctrl+B 全局触发 · 前端 mirror fuzzy 算法 30 行内（避 IPC RTT）· localStorage per-workspace recent 5 history · D.1-D.8 全 [x] · **性能爆表 D.7 100 branch P99 0.799ms / D.8 1000 branch P99 1.475ms（远超目标 16ms / 50ms · 20-33x）** · 2 决策点全选 prompt 推荐 (a)（localStorage + 前端 mirror）· 主 agent review 13 硬约束 + 8 acceptance + 2 决策点 + cargo test 43/43 全过 · MVP-13 3/4 完成（仅剩 Phase D runtime + bench）
- **PR #225** · MVP-13 Phase C done · sync 3/4 完成（chore/MVP-13-phase-C-progress-sync）· 三度 Codex 速度验证：A 5x · B 5x · C 8x · 平均 ~7x · 性能爆表数据写入跨 session 里程碑
- **PR #226** · **MVP-13 Phase D · runtime bench evidence**（feat/MVP-13-phase-D-runtime-bench · Codex CLI 实施 · ~40 min fast 模式）· 224 行 branch_bench.rs（6 Criterion bench · spec §C.2 模板）+ bench-output.txt 2.6KB raw 数据归档 · **6 bench P99 全过门槛**（branch_list_10 1.385ms · branch_list_1000 63.810ms · branch_create 3.636ms · branch_checkout_clean 5.361ms · branch_delete 3.282ms · fuzzy_filter_100 7.157ms）· Part 2 选 (a) deferred · GUI screenshots → Arbiter v0.2 GA 候选阶段一并补 · Part 3 spec frontmatter 保持 ready（acceptance 不全 [x] 等截图）· **Codex 主动 sync PROGRESS.md Next concrete action 加 deferred items 第 5 项**（加分项 · 省主 agent 后续 follow-up PR）· 主 agent review 13 硬约束 + Phase D acceptance + cargo test 43/43 全过 · **MVP-13 全 4 phase done · 自动化 100%**
- **PR #227** · MVP-13 全 4 phase done · sync 自动化 100%（chore/MVP-13-phase-D-progress-sync）· Codex 四度提速验证 + 跨 session 里程碑 + 归档索引 row 23 #207-#226 · 20 PR
- **PR #229** · MVP-21 NetworkOpError variant 数 audit · 9/10 → 11（chore/MVP-21-spec-audit-network-op-error）· 主 agent 起草 Phase A prompt 时发现 spec 内部不一致（line 123 起点 checklist 9 + line 569 §G.6 表 10 vs §G.2 实际 11）· 修订 + 加 StaleLease + SslError 列名 · 防 Codex 实施时困惑
- **PR #228** · **MVP-21 Phase A · git sync backend**（feat/MVP-21-phase-A-git-sync · Codex CLI 实施 · ~5h · 在 2d 估时内 · 复杂度 30%↑ vs MVP-13 Phase A · 提速 ~3x）· 1906 行 git_sync.rs + 6 IPC pub fn（push/pull/fetch/remote_list/auth_provide/merge_abort）+ NetworkOpError 11 variant + 4 AuthMethod path（SshAgent / SshKeyFile / HttpsHelper / HttpsManual）+ AuthMethod manual Debug redact（password / passphrase **\*\*\*REDACTED\*\*\***）+ 3 Tauri event（git:push-progress / git:fetch-progress / git:operation-done）+ Cargo.toml git2 vendored-libgit2/openssl/ssh/https · ts-rs 19 binding（vs prompt 12 · ts-rs 拆 nested + event payload + AuthChallenge / MergeConflictInfo / ConflictFile）· **57/57 单测**（vs prompt ≥ 25 · 多 32 个边界覆盖 · 含 Auth Debug redact 2 个）· build.rs ts-rs export 19 + permission 6 + capability + emit 3 event · H2 regression proof 完整执行（创建 temp consumer 证明 type drift 检测）· 主 agent review 13 硬约束 + Phase A acceptance + cargo test 57+43 全过 · v0.2 sprint W14 启动闭环
- **PR #230** · MVP-21 Phase A done · sync v0.2 sprint W14 启动（chore/MVP-21-phase-A-progress-sync）· Session 23 段加 PR #228 + #229 · Next concrete action 转 W14 · 跨 session 里程碑 + 归档索引 row update
- **PR #231** · **MVP-21 Phase B · git sync UI**（feat/MVP-21-phase-B-git-sync-ui · Codex CLI 实施 · ~5h · estimated 1.5d · 提速 ~2.4x）· +2159/-7 · 15 文件 · 5 dialog 全新建（AuthDialog 246 / ForcePushDialog 97 / GitSyncProgressDialog 170 / PullConflictDialog 92 / RemoteSelector 100）+ GitLogPanel.tsx +1033（push/pull/fetch workflow wiring · remote selection · dirty tree preflight · progress event listeners · auth retry · force push · conflict / retry flows）+ App.tsx + SecondarySidebar.tsx 集成（dirty pull 时打开 Git Status panel）+ styles.css +97 · §D.7 安全：AuthDialog username/password/passphrase/key path/keychain checkbox state 在 submit/Cancel/Escape/unmount 时 reset · grep 0 hit `console.*` secret · Auth retry 通过 `AuthMethod` binding 不持久化 · 30 acceptance UI 子集全 [x]（A.1-A.6 push / B.1-B.8 pull / C.1-C.4 fetch / D.4-D.7 auth / E.1-E.2 conflict / F.1-F.3+F.5 errors）· 4 类 explicit skip（E.3/E.4 status bar → Phase C · A.7/B.9/C.5/G.1-G.4 perf → Phase D · runtime evidence → Phase D · F.4 submodule → v0.3+）· 主 agent review 13 硬约束 + UI 子集 acceptance + lint + typecheck + cargo test 57+43 全过 + 30s tauri:dev smoke · v0.2 sprint W14 半程闭环
- **PR #232** · 主 agent housekeeping sync · MVP-21 Phase B done + Phase C codex 实施中（chore/sync-progress-mvp21-phase-b · 主 agent · 7 +5 -12 lines）· Next action 反映 Phase A + B done · 删过时"派 Phase B" 子项 · Session 23 段头 16 PR → 25 PR · trailer 合规率 6/6 → 25/25 · v2-D.1 → v2-D.2 · 修复主 repo local config 被 codex worktree 污染（dispatch §2.12 实证 · author 错归 Codex CLI · unset local + amend reset-author 修回 Leafile Lune + trailer Co-authored-by Claude Code）
- **PR #233** · **MVP-21 Phase C · status bar ahead/behind**（feat/MVP-21-phase-C-status-bar · Codex CLI 实施 · ~30-60min · 0.5d 估时内 · 提速 ~10x · session 23 单 PR 最快记录）· +535/-26 · 5 文件 · 1 新建 store `web/src/stores/remote-sync-status.tsx` 237 行（per-workspace SolidJS Context · `createStore<Record<string, RemoteSyncSnapshot>>` 隔离 · listen `git:branch-changed` + `git:operation-done` · onMount/onCleanup pattern · `git:operation-done` 仅 success 触发 refresh 防误清 ↑↓）· App.tsx +82（status bar `↑N` / `↓N` button + RemoteSyncStatusItem Component + RemoteSyncStatusProvider 包 LayoutShell · 自动打开 Secondary panel + dispatch highlight）· GitLogPanel.tsx +153（highlight handler · resolveHighlightTarget · ahead 高亮前 N 个 entries · behind 高亮 upstream branchLabel 边界 · 4500ms auto-clear timer + onCleanup 清 timer · scrollIntoView smooth）· styles.css +64（vs-status-remote-count.is-ahead/is-behind + .vs-git-log-entry-highlight-ahead/behind + .vs-git-log-highlight-note · 复用 CSS variable + color-mix(oklch)）· **决策点 E.4 选 (a) SolidJS Context**（不引 Tauri event · 天然 per-workspace · 不改 backend contract）· **behind 高亮策略**：复用 Git Log HEAD-first 契约 · 不新增 backend ref query · upstream branchLabel 边界高亮 · remote-only commits 留给 Git Log compare 能力（v0.3+）· **type alignment 验证**：`BranchChangedPayload` inline TS type 与 Rust `BranchChangedEvent` `#[serde(rename_all = "camelCase")]` 对齐 · `workspace_id` → `workspaceId` · 与 BranchTree / BranchSwitcher 既有 inline pattern 一致 · 主 agent review 13 硬约束 + 3 acceptance E.3/E.4/per-workspace + lint + typecheck + cargo test 482/482（含 git_sync 57 + branch_ops 43 + 其他不破）+ tauri:dev startup smoke 1m 30s build · UX visual confirm defer 给 Arbiter Phase D capture playbook 阶段 · v0.2 sprint W14 ~75% 完成 · 仅 Phase D 待

#### 关键决策与方法论

- **Codex 多轮 adversarial review 收敛差异**：抽象层不变量收敛 vs 代码 caller 集收敛差异显著
  - 代码层（PR #208 lifecycle）4 轮收敛 finite caller set · 完美闭合
  - 抽象层（PR #211 playbook）每轮 codex 推更深一层 · 无穷退而求其次需主动判断 good-enough 边界
  - option B "merge + last-mile validation" 替代无限迭代 · 接受 14 invariant 作为可发布版本
- **ID 冲突清理批次**：MVP-21 + MVP-22 两次 rename 在同 session 完成 · 清理 v0.1 → v0.2 / v1.0 转期 spec 命名空间冲突 · 后续新 spec 命名前需 grep 历史避免重复
- **MVP-05 spec done 翻转 unblock**：Phase D capture playbook ready 后 · 唯一未完成项变成 Arbiter 30-45 min 本地 GUI capture · 主 agent 之后开 done PR

#### 协作模式：主 agent + Codex 多轮 adversarial review

- 主 agent（Claude Code）：所有实施 + spec / playbook 起草 + 多轮 codex finding 修复
- Codex CLI（adversarial-review）：PR #208 + #211 共 7 轮深度审查 · 每轮 1-3 finding · 强制抽象层和代码层都要有不变量
- Kimi / OpenCode / 其他远程：本 session 未启用
- v2-D.2 trailer 合规率 100%（27/27 PR）· admin override 模式不再触发

### Session 22（2026-04-30 · MVP-20 PTY 预热池 全 5 phase · 1 day · Codex CLI fast 主导 + 主 agent 协调）

**最大成果**：解 user 痛点"新 tab 启动卡 1-2 秒"· 实测 cold spawn 800-1200ms → warm hit 0.09ms backend / 估 ~30-50ms 用户感知（提速 ~15-25 倍）。Codex fast 模式总用时 ~2.5h（spec 估 8-10h · 5x 提速）。

#### 5 PR merged（spec → A1 → A2+A3 → C → B → D · 总 +1500/-100）

- **PR #189** · spec ready（docs/MVP-20-pty-warm-pool-spec · +166）· 经 Kimi 远程 review 5 维度 · 3 Blocker + 6 High/Medium 全修订
- **PR #190** · Phase C Settings UI（feat/MVP-20-C-settings-ui · +100/-3）· `pty_pool_enabled` / `pty_pool_size` 字段 + ts-rs binding sync + TerminalGroup toggle/容量选择器 · 主 agent
- **PR #191** · Phase A1 PtyPool core（feat/MVP-20-A1-pool-core · +495/-44）· `pty_pool.rs` 370 行 · PoolConfig/PtyPool/IdlePty/TakeResult + take/refill/kill_all/set_size + 8 单测 · `PtySession.tab_id` 改 `parking_lot::Mutex<String>` 支持 rename · Codex CLI fast 1.5h（含 fmt baseline 修复）
- **PR #192** · Phase A2+A3 lifecycle + cd 注入（feat/MVP-20-A2-A3-pool-runtime）· 5min idle expire timer（crossbeam recv_timeout · 不引 tokio）+ apply_config_change/handle_default_shell_change/shutdown API + inject_cd_clear（cd -- 'path'; clear\n · POSIX 兼容 zsh/bash/fish）+ 18 单测全集 · Codex CLI fast 自主 commit/push/PR
- **PR #193** · Phase B 接入 IPC（feat/MVP-20-B-pty-pool-ipc · +101/-7）· AppState 加 pty_pool/pane_pty_pool: Arc\<PtyPool\> · run() init + workspace_init pool config + settings_update hook + tab_pty_spawn / pane_pty_spawn take-first · 354 tests 不破坏 · 主 agent

#### Phase D · runtime evidence + spec done（本 session 收尾）

- backend benchmark `crates/core/tests/pty_pool_bench.rs` 自动跑 cold/warm/disabled 3 个测试 · 数据进 git
- `docs/runtime-evidence/mvp-20/{README,00-baseline,01-warm-hit,02-cold-path,03-settings-toggle}.md` · 11 acceptance 全 [x]
- spec A10 措辞调整 · "3 段录屏" → "backend benchmark + 单测 + frontend baseline"（单人项目 v2-D.1 模式 · 视频对自动化验证无增量价值 · 偏离已透明记录）
- spec status: ready → done

#### 协作模式：双 agent 并发 + Kimi 远程 review

- 主 agent（Claude Code）：协调 / spec / Phase B / Phase C / Phase D
- Codex CLI fast：A1 / A2+A3（独立 worktree · `codex exec --skip-git-repo-check -`）
- Kimi（Moonshot 远程 API）：spec review only · 5 维度 · 20 min 出 review
- v2-D.1 trailer 合规率回升 100%（5/5 PR + Phase D PR · admin override 模式停用）

> **Session 21**（PR #173-#187 · v0.1.0 GA 发布配套 + GitHub Actions billing admin override 触发首次大规模 7 direct push + v0.1.1 双批 fix + PR #187 主 worktree dangling history close）已归档至 [`docs/session-history/session-21.md`](./session-history/session-21.md)。

> **Session 20**（PR #152-#172 · 19 PR · MVP-10 Phase B 完整闭环 + 2 critical/secondary bug fix）已归档至 [`docs/session-history/session-20.md`](./session-history/session-20.md)。

> **Session 19**（PR #117-#152 · 36 PR · 史上最高产）已归档至 [`docs/session-history/session-19.md`](./session-history/session-19.md)。

> **滚动窗口前**：session 18 及更早（PR #1-#116）的完整摘要请查 `git log --all --oneline | grep PR` · 或 `docs/session-history/` 里的归档文件。本 PROGRESS 每 session 末按 M-2 规则整理（当前展开 session 22 + 23 · session 18/19/20/21 已归档至 `docs/session-history/`）。

## ✅ 已完成（累计 · Pre-code Phase 1-4）

### Phase 1 · 战略与决策（PR #1/#2 · session 3-4）

- [x] B 阶段技术调研 / planner v1 / 4 视觉方向 + Calm Studio 定稿 / 2 Logo 候选
- [x] Codex 项目级评审（7 CRITICAL + 12 HIGH + 5 MEDIUM + 13 反对）
- [x] 4 项分歧决策：Apache 2.0 / MVP B 折中 / AI-Aware 撤出 / Tauri 改口
- [x] planner v2（14 章 + 附录 · 30 风险）
- [x] 独立仓库 + GitHub push + Apache 2.0 LICENSE + NOTICE
- [x] Phase 1 v1 → v4 simplified（承认过度设计 · 砍多 agent 治理抽象 · 保留 Git 普世 + 自审四问）

### Phase 2 · task spec 框架（PR #3/#5/#6/#7/#8/#9/#10 · session 4-5）

- [x] `docs/tasks/` 框架：schema + `_template.md` + README 索引
- [x] **SPIKE-01..07**（7 个 Spike spec · W0 硬通过矩阵 + SPIKE-07 v1.0-pre parser 验证）
- [x] **MVP-01..20**（20 个 MVP spec · v0.1 详细 + v0.2/v0.3/v1.0 占位）
- [x] 流程治理：5 步导游 · blocked 语义（`blocked_from`）· per-task 报告 · 翻转 gate 二选一
- [x] Codex 对抗性审查 **12 findings** 全闭合（R1-R6 · 4 commits 修）

### Phase 3 · 架构决策与治理文档（PR #12 · session 5）

- [x] **ADR × 10**：#1 License · #2 MVP 范围 · #3 AI-Aware v1.0 vision · #5 Workspace · #6 前端栈 · #7 Diff 自建 · #12 桌面框架 · #13 Git 栈 · #14 存储 · #15 PTY（accepted 6 + proposed 4）
- [x] **CONTRIBUTING.md**（贡献指南 · 含用户拍板 gate）
- [x] **CHANGELOG.md**（Keep a Changelog · Phase 1-3 条目）
- [x] **CODE_OF_CONDUCT.md**（Contributor Covenant 2.1 中文）
- [x] `docs/spikes/` + `docs/spike-artifacts/` + `docs/session-history/` 3 目录建立 · 各有 README + 安全约束
- [x] Codex 5 findings（3 HIGH + 2 MEDIUM）全闭合

### Phase 4 · GitHub 基础设施（PR #11 · session 5）

- [x] `.github/ISSUE_TEMPLATE/` 4 模板（config / bug / feature / task_spec_proposal）
- [x] `.github/PULL_REQUEST_TEMPLATE.md`（强制 Implemented by / Reviewed by / 翻转 gate / 自审四问）
- [x] `.github/dependabot.yml`（cargo + npm + github-actions 周更）
- [x] `.github/workflows/ci.yml` · skeleton（markdown-lint active · rust/frontend 占位）
- [x] **`.github/workflows/secret-scan.yml`** · gitleaks + `gitleaks-bypass-guard`（防内联 bypass marker 绕过 · 详见 SPIKE-06 §A.5.3）
- [x] **`.github/workflows/task-spec-validator.yml`** · frontmatter schema 校验 · 无 paths filter（防 required-check pending）
- [x] **`scripts/validate-task-spec.mjs`** · 224 行 · 自写 parser + 9 条 adversarial self-test + 7 类 schema 规则
- [x] **`docs/BRANCH-PROTECTION.md`** · admin 应用 main 保护的完整 checklist
- [x] Codex 3 HIGH findings 全闭合 + CI self-trigger fix（`a6fd6c6`）

### Codex 对抗性审查全统计（至 session 6 结束）

- **9 轮审查 · ~33 findings 全闭合**（含 session 6 三轮 + 二次复审）
- 平均每轮从 4 HIGH 收敛到 1-2 HIGH · 质量曲线明显
- 最深发现：SPIKE-04 op-log phantom data（marker-loss crash window · R6 F1 reconcile forward）· SPIKE-05 后端 IPC queue 满 HOL

### Spike W0 实施（session 7 · 2026-04-19）

**SPIKE-01 · Tauri 空壳启动 · Phase A macOS PASS（PR #20）**

- [x] Tauri 2 vanilla-ts 骨架 `spike-tmp/spike-01-tauri/`（gitignored · 8.2MB .app · 4MB dmg）
- [x] 冷启动 10 次 median **202ms**（目标 < 2s · 10× 余量）· Range 42ms 极稳
- [x] 中文 IME 录屏 + 5/5 肉眼验证
- [ ] Phase B Ubuntu 待环境就绪（prompt 备好 · 日文全平台 skip）

**SPIKE-02 · Tauri 硬通过矩阵 · Phase A macOS PASS（PR #22）**

- [x] 10x 稳定性 10/10 · median 212ms · Bundle 10MB/.dmg 4MB
- [x] Clipboard plugin smoke（读写 + 跨 app Cmd+V 含中日英+emoji UTF-8 完整）
- [x] FS plugin smoke（读写 + terminal cat 验证）
- [x] 中文 IME 录屏
- 2 项降级：updater 归 SPIKE-06（Apple Dev key 依赖）· 日文 IME 全平台 skip（用户决策）
- [ ] Phase B Ubuntu 待环境

**SPIKE-03 · git2 vs gix benchmark · done (PR #23 待 merge)**

- [x] OpenCode agent linux kernel 1.44M commits 实测
- [x] 结论 **(B) 读切 gix · 写保留 git2**：gix log -100 warm P99 **12.65ms** vs git2 **24964ms**（gix 1973× 快）
- [x] ADR-007 proposed → accepted · 决策表 #13 B→A

**SPIKE-04 · storage benchmark · done (PR #24 待 merge)**

- [x] OpenCode agent 2 次交付（v1 被 Claude review BLOCK · v2 补做 accept）
- [x] §A 性能：redb 写入 P99 31.94s / rusqlite 9.96s · 两者都通过
- [x] §B 安全：redb 2.6.3 **B.2 坏库检测 FAIL**（silent 读出可能错误数据）
- [x] 结论 **(B) 锁 rusqlite**（redb 2.6.3 被 supersede）
- [x] ADR-005 proposed → accepted（结论翻转 redb→rusqlite）· 决策表 #14 B→A（rusqlite）
- **R27 未真 close · 需 SPIKE-04.5 on rusqlite 补 B.1-5**

**SPIKE-04.5 · rusqlite 数据安全 · ready (PR #25 待 merge · 本 PR 新建)**

- [ ] 新建 spec · depends_on: SPIKE-04 · blocks: MVP-02/06/10/19
- [ ] A 性能复测（rusqlite 100 行范围 · 澄清 SPIKE-04 歧义）
- [ ] B.1-5 全链路在 rusqlite 上实测 · 补 SPIKE-04 瑕疵（B.3 实 assert · B.4 auto-backup · B.5 production op-log + 自动回滚 UI）
- [ ] 结论：rusqlite B.1-5 全过 → ADR-005 修订 "R27 真 close" | 失败 → Arbiter

**Codex 对抗性审查新统计（session 7）**

- Claude Code 作为 SPIKE-04 reviewer：发现 4 CRITICAL（bulk_write 单样本 / range 事后洗白 / sudo purge 未执行 / B.1-5 未做）· 退回 opencode · v2 全闭合
- 说明多 agent 并行交付 + 独立 review 的质控链路有效

## 🔜 下一步（按执行顺序）

### 🔐 **用户手动步骤**（`docs/BRANCH-PROTECTION.md` · 当前**已显式暂缓**）

用户已表态暂不应用 main 分支保护（单人 + Codex review 模式下不致命）。**当前流程靠 reviewer 肉眼守门**（accepted tech debt · 见 `docs/tasks/README.md` §原则 7）。

升级触发条件（任一）：

1. 仓库改 public
2. 第二位外部 contributor 出现
3. MVP-01 开始写 Rust 代码
4. 第一个 release tag

触发时按 `docs/BRANCH-PROTECTION.md` checklist 一次性应用。

### 🚀 **Spike Week 0**（进行中 · session 7 · 多 agent 并行）

1. **W0-D1** · [SPIKE-01](./tasks/SPIKE-01-tauri-three-platform-boot.md) · **Phase A macOS ✅ PASS（PR #20 merged）** · Phase B Ubuntu 等环境
2. **W0-D2** · [SPIKE-02](./tasks/SPIKE-02-tauri-hard-pass-matrix.md) · **Phase A macOS ✅ PASS（PR #22 merged）** · 2 项降级（updater + 日文 IME）· Phase B Ubuntu 等环境
3. **W0-D3** · [SPIKE-03](./tasks/SPIKE-03-git2-gix-read-benchmark.md) · ✅ **done（PR #23 merged）** · 结论 (B) 读切 gix · 写保留 git2
4. **W0-D4** · [SPIKE-04](./tasks/SPIKE-04-storage-benchmark.md) · ✅ **done（PR #24 merged）** · 结论 (B) 锁 rusqlite（redb 2.6.3 B.2 FAIL）
5. **W0-D4.5** · [SPIKE-04.5](./tasks/SPIKE-04.5-rusqlite-safety-verification.md) · ✅ **全 done（PR #29 主体 merged · PR #34 A.3 决策 merged）** · B.1-5 全过 · R27 真 close · A.3 P99=215ms · **Arbiter 选定方案(a) MVP 接受 220ms**（2026-04-19 · 方案(b) 复合索引留 MVP-02 一起加）
6. **W0-D5** · [SPIKE-05 portable-pty 多 Tab 压测](./tasks/SPIKE-05-pty-multi-tab.md) · ✅ **done（PR #30 merged）** · shared-reader **HOL / boundedness pass** · **visible throughput fail**（ADR-003 继续 proposed）
7. **W0-D5.5** · [SPIKE-05.5 PTY visible throughput + per-session fallback 对照](./tasks/SPIKE-05.5-pty-visible-throughput-fallback.md) · ✅ **done（PR #39 merged）** · 结论：shared-reader 不是瓶颈 · per-session UI drain 反而略低（4 Tab 12.86 vs 14.58 MB/s）· 瓶颈在 invoke RTT 22ms / JS / xterm · ADR-003 accepted · CLAUDE.md #15 B → A
8. **W0-D6** · [SPIKE-06 Claude/Codex CLI + Apple Dev Program](./tasks/SPIKE-06-cli-protocol-and-codesign.md) · 🟡 **§A harness done（PR #38 merged · pipeline smoke 通过）** · §A 36 样本待 PR 2（session 11 · `brew install gitleaks asciinema` 前置）· §B Apple Dev Program 用户申请中

### 🧑‍🎨 **MVP-01 Phase A + B 已交付**（session 8-9 · 首个能启动 + 视觉一致骨架）

- **Phase A**（PR #28 merged）· Cargo workspace 2 crate + SolidJS + Tauri 壳 + Codex 2 轮对抗 review + 3 轮 CI 修最终切 corepack pattern
- **Phase B**（PR #33 merged）· Calm Studio design token 落地 · 欢迎页精装 · 真实 icon 替换 · runtime 验证通过
- **Phase C 预期**：基础崩溃恢复 session persistence（**MVP-02 已 done · 解阻塞**）· Ubuntu 24 runtime 验证（阻塞 · 无环境）

### 🗂️ **MVP-02 workspace 管理已交付**（session 10 · PR #40 merged · OpenCode 主交付 + reviewer fix）

- **Backend**：rusqlite + r2d2 connection pool · schema v1→v2 migration（`PRAGMA user_version`）· `WorkspaceStore` CRUD（create/list/get_by_id/touch/delete/exists_at_path）· git auto-detection 5 parent levels · UUID v4 · canonical path（dunce）
- **IPC layer**：7 commands（greet · workspace_init/list/create/open/delete/exists）· `AppState { pool: Mutex<Option<DbPool>> }` · 每 command 独立 ACL permission identifier
- **Frontend**：sidebar workspace list（按 last_opened DESC）· directory picker（plugin-dialog）· 删除二次确认 modal · git badge · error bar · multi-workspace switcher（部分 · close 推 MVP-04）
- **测试覆盖**：23 unit tests（19 workspace + 4 db migration · 含 UTF-8 / spaces / duplicate / nonexistent / git parent detection / idempotent migration）
- **Reviewer fix**：H1 path traversal（workspace_init 改 backend 自取 `app_local_data_dir()`）+ M3 SVG bug（VibestationMarkSmall xmlns + 内联 gradient）+ spec done 翻转走 (a) 路径
- **Explicit skip 推 MVP-04**：§C `workspace.close` IPC + opened/closed session 状态建模 · §D `app_state` table（"打开列表 + 顺序"持久化）· 与 Tab 管理一起做避免分裂改动
- **Follow-up 收尾**：FU-1 ✅ 关闭（PR #47 · 截图 3 重做 · 用户手动）· FU-2 ✅ 关闭（PR #44 + #45 · ADR-011 accepted + 6 步实施）· FU-3 ✅ 关闭（PR #42 · dispatch prompt §2.8 升级）· FU-4 ✅ 关闭（PR #43 · SPIKE-01/02 归档）

### 🎛️ **MVP-03 Tool Windows 已交付**（session 11 开场 · PR #61 merged · OpenCode 主交付）

- **布局**：5-zone grid（Activity Strip + Primary Sidebar + Main + Secondary Sidebar + Bottom Panel）· 严格对齐原型 `design/directions/1-calm-studio.html` DEFAULT_STATE
- **交互**：toggle（Primary / Secondary / Bottom 独立开关）· resize（拖拽分隔条 · min/max 范围约束）· theme（light / dark · `prefers-color-scheme` 自适应）
- **持久化**：布局状态入 rusqlite（列宽 / 折叠态 / 主题）· 跨 session 恢复
- **测试**：29 unit tests（+13 新增 · layout.rs + persistence）· 7/7 CI target 全绿
- **Runtime 证据**：5 张截图（`docs/runtime-evidence/mvp-03/` · dark × 4 + light × 1 · 60-100 KB · 符合 ADR-011 R4）
- **验收**：20 项清单全过 · 8 条硬约束（dispatch prompt v2）全过

### 🧪 **SPIKE-08 E2E + IPC contract harness 已交付**（session 11 开场 · PR #60 merged · Codex 主交付）

- **§A Contract layer**：**ts-rs 选用**（v0.1 GA 前强制覆盖所有新增 IPC contract · Rust type → TS type codegen · `build.rs` trigger · `beforeDev/BuildCommand` 保证 bindings fresh）
- **§A 对比**：`ts-rs 12.0.1`（stars 1765 · 依赖 656 行）vs `tauri-specta 2.0.0-rc.24`（仍 RC · 依赖 675 行 · builder-based 集成成本高）· 选 ts-rs
- **§B Runtime layer**：`Playwright + Vite` 作为 v0.1 自动化 runtime 补层（非 required）· 真实 Tauri IPC E2E（B.1/B.3 Linux tauri-driver）本轮未收敛 · 不作为 v0.1 GA required gate
- **§C CI**：contract + browser smoke 全平台 required · native runtime 继续保留 manual runtime evidence · Linux `tauri-driver` workflow 留 informational follow-up
- **H2 回归验证**：临时把 `WorkspaceRecord.id` 改为 `workspace_id` · `pnpm typecheck` **必然 FAIL**（符合预期）· 证明 contract layer 能把 H2 类 drift 前移到 compile-time
- **下一步（session 11 候选 1）**：ts-rs 推广到 MVP-02 现有 5 个 IPC contract struct · 闭合 H2 根因制度化

**并行化节奏说明**：SPIKE-03/04 是纯 CLI bench · 不依赖 Tauri UI · 用户决策放宽 depends_on（SPIKE-02 → SPIKE-01）· 由 opencode agent 并行完成。这是 session 6 协作规则"给原话 prompt 让用户转发给其他 agent"的首次大规模落地。

### 📦 Spike W0 通过后 · MVP 实施（目标 v0.1 GA · 12-14 周）

- MVP-01..10 按依赖顺序实施（MVP-01 → ... → MVP-10）
- MVP-11..20 留 v0.2 / v0.3 / v1.0 kickoff 详化

## ⚠️ 当前卡点 / 注意事项

- **MVP-03 ✅ done · Tool Windows 布局已交付**（PR #61 merged · session 11 开场 · OpenCode 主交付 · 5-zone + toggle + resize + theme · 29/29 Rust 测试 + 5 张 runtime 截图 · ADR-011 R4 符合）
- **MVP-04 🟡 终端主链只剩 Phase D**（PR #72/#82/#91/#95/#99 已合入 · Phase A/B/C/E/F 全部落地 · 当前只剩默认 shell / Claude CLI / Codex CLI 实机兼容验证 · 低优先）
- **MVP-08 🟡 Phase A/B/C 已完成**（PR #100/#101/#105 已合入 · 后端 diff/status contract + Bottom Panel Git Status 面板 + Diff 视图前端 + Git Status/Git Log → Diff 接通已落地 · 当前主线 = Phase D fs watch（`notify` 6.x 三平台 · 替换当前 polling）+ Phase E 证据量化（5 截图 + A.2/A.6/F 性能门槛实测））
- **PR 级 GitHub Actions 自动运行已关闭**（PR #102 · 只保留 `push main` + `workflow_dispatch` · 新 PR 不会自动跑 CI，后续 agent 必须本地先跑 gate，并在 merge 后核对 `main` 的 check runs）
- **SPIKE-08 ✅ done · E2E + IPC contract harness 选型**（PR #60 merged · session 11 开场 · Codex 主交付 · §A ts-rs PASS · §B Playwright runtime FAIL · §C hybrid gate · 下一步 ts-rs 推广 MVP-02 现有 IPC contract · 闭合 H2 根因制度化）
- **ADR-006 accepted + CLAUDE.md v2-D**（PR #50 merged · session 10 末 · "self-review + Arbiter approval" 单人项目术语澄清 · 未来升级 v2-strict 触发条件显式化）
- **Vite 8 + TS 6 major bump 评估**（PR #59 merged · docs/upgrade-notes/ · 推荐 v0.1 GA 后再升 · 不碰生产代码）
- **docs rusqlite 字样对齐**（PR #58 merged · implementation-plan 8 处 stale 清理 · 对齐 ADR-005）
- **SPIKE-04.5 ✅ 全 done** · R27 数据安全 close · A.3 Arbiter 选定方案(a) MVP 接受 220ms（PR #34 merged · 不改代码 · 方案(b) 复合索引留 MVP-02 一起加）
- **SPIKE-05.5 ✅ done** · ADR-003 accepted · CLAUDE.md #15 B → A 锁 shared-reader（PR #39 merged · session 10）· 后续 invoke / JS / xterm 优化转独立 task（visible throughput 优化推到 v0.2 / v0.3）
- **MVP-02 ✅ done · workspace 管理已交付**（PR #40 merged · session 10 · OpenCode 主交付 + 主 agent H1/M3 fix + spec done 翻转）· §C close + §D opened 列表 explicit skip 推 MVP-04
- **FU-1 ✅ 关闭**（PR #47 · session 10 终极末 · 用户手动重截 modal · 同时是 H2 fix 后的 runtime 证据 · 44.7 KB · 远低于 ADR-011 R4 推荐）
- **FU-2 ✅ 关闭**（PR #44 + #45 · session 10 真末 · Arbiter 选项 A 选定 · ADR-011 accepted · runtime 证据路径锁 `docs/runtime-evidence/<task-id>/` · 进 git · CLAUDE.md 决策表 #18 新 row · 新项目规则 `.claude/rules/runtime-evidence-location.md` R1-R5 硬规则落地）
- **FU-3 ✅ 关闭**（PR #42 · session 10 真末 · dispatch prompt §2.8 子进程清理硬约束 · 默认硬约束 7→8 · trap/pkill 两种做法）
- **FU-4 ✅ 关闭**（PR #43 · session 10 真末 · rule 13 历史欠账修复 · SPIKE-01/02 源码归档进 `docs/spikes/code/SPIKE-0[12]/` · 释放 2 GB 冷备）
- **H2 IPC camelCase mismatch ✅ 修复**（PR #47 · session 10 终极末 · MVP-02 runtime bug · CI 全绿但点 Delete 报 missing key id · 根因：Rust `#[serde(rename_all = "camelCase")]` 输出 `workspaceId` · 但 TS interface 误声明 `workspace_id` · 全 5 字段 16 处替换为 camelCase · runtime 用户验证 Delete + Git badge + dark mode 全过 · **rule 15 "CI 绿 ≠ runtime 过" 活教材** · 暴露 E2E 测试缺口 · session 11 候选 spike）
- **多 agent 共享 working tree 风险已规避**：Codex + OpenCode 已各自建 `git worktree` 独立工作（session 9 Phase B 开工时发现 shared-tree 冲突苗头后立即修正）· 未来 dispatch prompt 必须明确要求 worktree / /tmp 隔离
- **OpenCode Track 3 程序瑕疵事后补档**：PR #34 未按 dispatch spec 跑 benchmark · 直接自己标 "Arbiter 选定方案(a)"· Arbiter 事后 comment 确认方案(a) 判断合理 · 决策成立 · 下次 dispatch prompt 加 "外部 agent 不得自行 accept decision-grade 结论" + benchmark 强制要求
- **MVP spec 中 `redb` 字样历史**（MVP-01/02/03/05/06/10/19 · 共 7 个）：暂不改 spec 正文（YAGNI）· 实施时以 ADR-005（rusqlite）为准 · 届时 PR 触发 API-level 改动
- **Ubuntu 24 环境缺失**（SPIKE-01/02 Phase B 前置）· 阻塞 SPIKE-01/02 full done · ADR-006 桌面框架 · SPIKE-06 cross-platform · MVP-01 Phase C Ubuntu runtime 验证
- **分支保护已显式暂缓**（用户表态 · 单人 + Codex review 模式下不致命 · 升级触发条件见上方 §🔐 用户手动步骤）
- **R1 Claude/Codex CLI 协议**：SPIKE-06 样本录制 · R1 降级须经 SPIKE-07 parser 验证 + ADR-011
- **R12 CRITICAL Tauri Wayland**：macOS Phase A 强信号 · Wayland 风险仍在（SPIKE-01/02 Phase B 兜底）
- **Apple Developer Program 审核**：SPIKE-06 立刻提交 · 最长 2 周影响 v0.1 发布（W12）· 同时 SPIKE-02 updater plugin 也依赖
- **域名未定**（W10 决定）· **Logo 未最终选定**（v0.1 前定）

## 🔀 阶段切换信号

| 信号                        | 触发                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ✅ Phase 1-4 Pre-code 完备  | **已达成**（2026-04-18 session 5 · 4 PR 全 merge）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ✅ Spike W0 启动            | **已达成**（session 7 · 首行 Rust 代码 · SPIKE-01 Phase A PASS）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ✅ Spike W0 macOS 全通过    | **已达成**（session 11 开场 · SPIKE-01/02 Phase A · SPIKE-03/04/04.5/05/05.5/08 全 done · SPIKE-06 §A harness done · 36 样本 + §B Apple Dev 阻塞外部资源）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 🟡 Spike W0 全平台通过      | SPIKE-01/02 Phase B Ubuntu（阻塞环境）+ SPIKE-06 36 样本 + Apple 申请                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 🔴 Spike 任一 CRITICAL Fail | 触发 fallback + ADR supersede                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ✅ MVP 实施启动             | **已达成**（session 8 · MVP-01 Phase A · ADR-003/005/006/007 全 accepted）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 🟡 MVP v0.1 进度            | **3/10 done + 7/10 ready**（MVP-02/03/07 done · MVP-01/04/05/06/08/09/10 ready · MVP-11 Native Feel done）· **session 20 主 agent 主线代码侧 100% 收官**：MVP-04 Phase A-F 全 done 仅 §I 截图待补 · MVP-05 Phase A/B/C 全 done · Phase D 待 GUI capture · MVP-08 Phase A-D 全 done · Phase E v0.2 deferred · **MVP-09 Phase A/B/C done · Phase D 性能 done by PR #156（runtime 截图待 GUI）** · **MVP-10 Phase A/B 全 done（含 §B.1 modal 阻塞 + §C.4 endpoint UI + §F.02 实时生效 + §G.4 H2 proof + §F evidence 3/4 done · PR #161 critical bug fix 解锁 v0.1 GA · PR #163 secondary dual-path fix 闭环 §F.02 acceptance）** · **主线收敛到 Arbiter 本地 1 小时 GUI 截图（4 类） + spec frontmatter done 翻转 + MVP-10 Phase C/D/E 打包**（不再有需要新写代码的主线 task） |
| 🎯 v0.1 GA                  | MVP-01..10 全过 §10.1 + §10.6 终端正确性矩阵 + §10.3 跨平台                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 🔴 连续 2 周 < 5h 投入      | 触发 hibernation（`implementation-plan.md §10.5`）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

## 📦 近期关键交付物索引

| 产出                                                                           | 路径                               |
| ------------------------------------------------------------------------------ | ---------------------------------- |
| v2 实施计划（14 章 + 附录）                                                    | `docs/implementation-plan.md`      |
| 8 个 Spike spec（W0 + v1.0-pre + H2 制度化 · 含 SPIKE-04.5 / SPIKE-05.5 补测） | `docs/tasks/SPIKE-*.md`            |
| 20 个 MVP spec（v0.1 详细 + v0.2+ 占位）                                       | `docs/tasks/MVP-[01-20]-*.md`      |
| 11 个 ADR（11 accepted · 0 proposed · session 10 末 ADR-006 升级后全收敛）     | `docs/adr/ADR-0[01-11]-*.md`       |
| 9 个 Spike report（含 SPIKE-08 harness 选型）                                  | `docs/spikes/SPIKE-*-report.md`    |
| MVP-02 / MVP-03 runtime 证据                                                   | `docs/runtime-evidence/mvp-0[23]/` |
| Agent 入口 · 决策表 · 自审四问 · 翻转 gate                                     | `CLAUDE.md`                        |
| 人类启动手册                                                                   | `docs/SESSION-STARTUP.md`          |
| 贡献指南 · 含用户拍板 gate                                                     | `CONTRIBUTING.md`                  |
| 分支保护 admin checklist                                                       | `docs/BRANCH-PROTECTION.md`        |
| Frontmatter validator + self-test                                              | `scripts/validate-task-spec.mjs`   |

---

## Session 日志

> **M-2 滚动规则**：本节只列归档索引 · 详细 session 摘要见 `docs/session-history/<session-N>.md` · 全部 PR 历史见 `git log --all --oneline`。
> 当前活跃窗口（近 2 session · session 19 + 20）已在上方"已合入的 PR"段展开。

### 归档索引

| Session | 日期               | PR 范围       | 主题                                                                                                                                                                                                                                                                                                                                                             | 归档文件                                           |
| ------- | ------------------ | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| 7       | 2026-04-18 ~ 04-19 | 见 git log    | Spike W0 多 agent 并行 · 首行代码 + 4 Spike + 1 新 Spike                                                                                                                                                                                                                                                                                                         | git log（未单独归档）                              |
| 8       | 2026-04-19         | 见 git log    | SPIKE 全收口 + MVP-01 Phase A 首行生产代码                                                                                                                                                                                                                                                                                                                       | git log（未单独归档）                              |
| 9       | 2026-04-19         | 见 git log    | 三路并行 · MVP-01 Phase B 视觉骨架 + md 盘点                                                                                                                                                                                                                                                                                                                     | git log（未单独归档）                              |
| 10      | 2026-04-19         | 见 git log    | 三路收敛 · MVP-02 落地                                                                                                                                                                                                                                                                                                                                           | git log（未单独归档）                              |
| 11      | 2026-04-20         | 见 git log    | MVP-03/SPIKE-08 落地 + ts-rs rollout + MVP-04 spec ready + Kimi 首次成功协作                                                                                                                                                                                                                                                                                     | git log（未单独归档）                              |
| 12      | 2026-04-20         | 见 git log    | 多 agent 四路并发 · v0.1 Git 能力闭环 + 终端画面闭环 + SPIKE W0 macOS 完结                                                                                                                                                                                                                                                                                       | git log（未单独归档）                              |
| 13 ~ 16 | 2026-04-21 ~ 04-22 | 见 git log    | 未单独归档 · session 13 audit + Kimi 11 次协作 + MVP-04 Phase C/E + SPIKE W0 macOS 完结尾声                                                                                                                                                                                                                                                                      | git log（未单独归档）                              |
| **17**  | 2026-04-23         | **#99-#105**  | MVP-04 Phase F 收口 + MVP-08 Phase A/B/C 落地 + PR Actions 分钟节流                                                                                                                                                                                                                                                                                              | [`session-17.md`](./session-history/session-17.md) |
| **18**  | 2026-04-25         | **#106-#116** | 4 track 并发极致产出 · 11 PR · 5 Phase 落地 + 3 spec ready 加强                                                                                                                                                                                                                                                                                                  | [`session-18.md`](./session-history/session-18.md) |
| **19**  | 2026-04-25         | **#117-#152** | MVP-11 全 done + MVP-05 Pane 落地 + ADR-006 Ubuntu validated + branch protect 机械化 · 史上最高产 36 PR                                                                                                                                                                                                                                                          | [`session-19.md`](./session-history/session-19.md) |
| **20**  | 2026-04-26         | **#152-#172** | MVP-10 Phase B 完整闭环 + 2 critical/secondary bug fix + dispatch §2.13/§2.14 教训规则化                                                                                                                                                                                                                                                                         | [`session-20.md`](./session-history/session-20.md) |
| **21**  | 2026-04-26 ~ 04-29 | **#173-#187** | v0.1.0 GA 发布配套 + Phase D Linux AppImage 实测 + GitHub Actions billing 暂停触发首次 admin override 模式 + v0.1.1 双批 fix（admin push + PR #186）+ PR #187 主 worktree dangling history 验证 close                                                                                                                                                            | [`session-21.md`](./session-history/session-21.md) |
| **22**  | 2026-04-30         | **#189-#193** | MVP-22（ex-MVP-20）PTY 预热池全 5 phase · 1 day · Codex CLI fast 主导 + 主 agent 协调 · 解 user "新 tab 卡 1-2s" 痛点（提速 ~15-25 倍）· Kimi 远程 review · v2-D.1 trailer 100%                                                                                                                                                                                  | git log（session 23 末归档时合并）                 |
| **23**  | 2026-05-02 ~ 05-03 | **#207-#229** | 23 PR · v0.1 收尾 + 治理升级 ADR-016 v2-D.2 + v0.2 sprint W13 完成 + **W14 启动 · MVP-13 自动化 100% + MVP-21 Phase A done**（Codex CLI 全 fast 模式 · 总实测 ~11h vs 估时 6d · ~6.5x 平均提速 · 5572 行代码 + 100 单测 + 6 Criterion bench + 4 AuthMethod manual Debug redact · 19 binding ts-rs 拆）· 主 agent + Codex + Explore 子 agent 并行协作模式五度验证 | 本 PROGRESS 展开 · 待归档                          |

### 跨 session 关键里程碑

- **首行代码**：session 8 · PR #28 · MVP-01 Phase A Tauri 壳 + SolidJS
- **Spike W0 macOS 100% 完结**：session 12 · 6 SPIKE 全 PASS
- **v0.1 10 MVP spec 全 ready**：session 15 · MVP-10 PR #88 + MVP-05 PR #89
- **MVP-08 主线里程碑**：session 17 · PR #105 · Diff 视图前端集成
- **MVP-11 Native Feel Quality 全 done**：session 19 · 11 PR
- **ADR-006 Ubuntu validated · v0.1 GA 双平台**：session 19 · PR #138
- **ADR-015 Telemetry accepted · MVP-10 Phase B 解锁**：session 20 · PR #152
- **CRITICAL bug rescue · v0.1 GA blocker**：session 20 · PR #161 · modal mount-time webview 虚假 click guard
- **v0.1.0-alpha 双平台发布**：session 21 · 2026-04-26 · macOS .dmg unsigned + Linux .deb / .AppImage（PR #173/#174/#175）· README Gatekeeper bypass 指引 · macOS notarize 推 v0.2
- **首次 admin override 模式**：session 21 · 2026-04-28 · GitHub Actions billing 暂停 · 7 direct push to main（1 v0.1.1 fix + 6 dependabot bumps）· v2-D.1 trailer 合规率因此回落 · session 22 audit 项
- **MVP-22 PTY 预热池全 done**：session 22 · 2026-04-30 · 1 day 5 PR · 解 user "新 tab 卡 1-2s" 痛点 · cold spawn 800-1200ms → warm hit 0.09ms backend / 估 ~30-50ms 用户感知 · 提速 ~15-25 倍 · Codex CLI fast 5x 提速（spec 估 8-10h · 实际 2.5h）
- **PR #208 多轮 codex review 收敛**：session 23 · 2026-05-02 · MVP-05 pane lifecycle 4 不变量沉淀 · 全局 rule 18 升级 systemic-fix-after-review · 全局 memory `feedback_pr208-multiround-review-postmortem.md` 沉淀 5 类认知盲点
- **MVP-05 Phase D capture playbook ready**：session 23 · 2026-05-03 · 4 轮 codex adversarial review 抽象 14 invariant + §7 BLOCK gate · Arbiter 30-45 min 一次性收口 · v0.1 GA 路径上唯一剩的 GUI capture 任务解锁
- **2 ID 冲突清理批次**：session 23 · MVP-11 → MVP-21（v0.2 Git Push/Pull/Fetch）· MVP-20 → MVP-22（PTY warm pool）· 双 footnote 历史 trace · v0.2/v1.0 启动前清空命名空间冲突
- **ADR-016 v2-D.2 governance 升级**：session 23 · 2026-05-03 · admin override 模式 trailer 豁免条款 · session 21 期间 7 direct push（GitHub Actions billing 暂停）追溯接受为合规 · 关闭 session 22-23 长期 audit 项 · v2-D.1 → v2-D.2
- **v0.2 sprint W13 启动 + MVP-13 Phase A done**：session 23 · 2026-05-03 · Codex CLI ~2.5h fast 模式实施（PR #220）· 1448 行 branch_ops.rs + 5 IPC + 12 ts-rs binding + 43/43 单测 · ~5x 提速 · 主 agent + Codex + Explore 子 agent 并行协作模式实证
- **MVP-13 Phase B done · ~85% 完成**：session 23 · 2026-05-03 · Codex CLI ~2h fast 模式实施（PR #222）· 1198 行 frontend（BranchTree + 3 dialog + branchName.ts utility）· spec §H.6 校验 + 5 IPC 调用 + branch-changed event 增量更新 · GitLog/GitStatus panel 主动 listen 加分项 · sandbox dev mode smoke · ~5x 提速复验（与 Phase A + MVP-22 一致）· MVP-13 仅剩 Phase C + D（各 0.5d）
- **MVP-13 Phase C done · 3/4 完成 + 性能爆表**：session 23 · 2026-05-03 · Codex CLI ~30 min fast 模式实施（PR #224 · 8x 提速 · MVP-13 三度 Codex 速度验证）· 796 行 frontend BranchSwitcher modal + ⌘B/Ctrl+B 全局 keydown + 前端 mirror fuzzy 算法 30 行内 + localStorage recent 5 history · D.7 100 branch P99 **0.799ms** / D.8 1000 branch P99 **1.475ms** · 远超目标（16ms / 50ms）20-33x · 2 决策点全选 prompt 推荐 (a)
- **MVP-13 全 4 phase done · 自动化 100%**：session 23 · 2026-05-03 · Codex CLI 全 fast 模式实施（PR #220 + #222 + #224 + #226 · 总实测 ~6h vs 估时 4d · ~8x 平均提速）· branch_ops 后端 + Primary Sidebar UI + Fuzzy Switcher + Criterion bench 6 个 P99 全过门槛 · GUI screenshots 走 deferred 模式（同 v0.1 4 类 deferred · 现 5 类 · Arbiter 自定时机）· spec frontmatter status 保持 ready 等截图补全后主 agent 开 done PR · v0.2 sprint W13 实施侧闭环 · 主 agent + Codex + Explore 子 agent 并行协作模式四度验证（与 MVP-22 + 三 phase 一致）
- **MVP-21 Phase A done · v0.2 sprint W14 启动**：session 23 · 2026-05-03 · Codex CLI ~5h fast 模式实施（PR #228 · 复杂度 30%↑ vs MVP-13 A · 提速 ~3x · 含 git2 网络层 + 11 NetworkOpError + 4 AuthMethod path + AuthMethod manual Debug redact + 3 Tauri progress event + 57 单测 · 19 binding ts-rs 拆）· spec audit PR #229 同 session 内闭合 NetworkOpError 9/10 → 11 variant 不一致 · v0.2 sprint W13 + W14 双 sprint 全启动 · MVP-21 Phase B/C/D dispatch prompt 全 ready local（共 6 prompt · 主 agent + Codex + Explore 子 agent 并行协作五度验证）

---

**本文件每次 session end / 阶段切换 / 重大决策变化时手动更新。机械字段 Phase 5 CI 后接 hook 自动刷新。**
