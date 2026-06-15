# 进度快照 · PROGRESS

> ⚠️ **文档定位说明（外部读者请先看）**：本文件是**内部协作 ledger** · 主要给主 agent 和项目所有者消费 · 含大量 session 内部术语（PR# / 决策表 / dispatch 事件）。如果你是外部读者：
>
> - 想了解项目当前**功能状态** → 看 [`README.md`](../README.md) 的「特性」表
> - 想了解**路线图细节** → 看 [`docs/implementation-plan.md`](./implementation-plan.md)
> - 想了解**仓库结构** → 看 [`docs/PROJECT-OVERVIEW.md`](./PROJECT-OVERVIEW.md)
>
> 本文件保留在 `docs/` 而非 `docs/internal/` 的原因：被锁定决策表 [CLAUDE.md](../CLAUDE.md) 直接引用，且是 agent 协作的当前态 source of truth，迁移会破坏 24 个 cross-ref。

> **定位**：当前状态面板（agent 和人类都先读本文件获取"我是谁 / 做到哪 / 下一步 / 卡点"）。
> **更新约定**：session end / 阶段切换 / 决策变化时手动更新。不要每个 commit 都更新（噪音大）。
> Session 历史归档到 `docs/internal/session-history/`（Phase 3 已建立）——**不要**归档到 CHANGELOG（CHANGELOG 是 release-please 自动维护的发布日志）。
> **PR 列表滚动窗口规则**（M-2 · 2026-04-21 session 13 audit）：本文件"已合入的 PR"段**只保留最近 2 个 session 的摘要** · 更早的以 `git log --all` + `docs/internal/session-history/` 归档文件为准 · 每 session 末整理。

---

## 📊 固定状态字段

| 字段                      | 值                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | 更新时机      |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| **Active branch**         | 见 `git branch --show-current`（本表不硬编码分支 · 避免 PROGRESS 和现实漂移 · H-2 · 2026-04-21）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | —             |
| **Latest commit**         | 见 `git log --oneline -1`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | —             |
| **Worktree status**       | 见 `git status` + `git worktree list`（三方 worktree 隔离 · 无 shared-tree 冲突）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | —             |
| **Unpushed branches**     | 见 `git branch -vv`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | —             |
| **Next concrete action**  | **session 38 · 2026-06-14 · main HEAD `0960644` · 代码主线 ship 完整收口 · 队列无 ready 工程任务**：v1.0 vision（MVP-18/19/20）+ v0.1/v0.2/v0.3 sprint 全 done · spec frontmatter **22 done / 1 blocked / 1 draft**（blocked = SPIKE-06 §B Apple Dev · draft = _template）。近 session 全 merged：session 35 git2 0.21 migration（#432）· session 36 housekeeping #434-#444 · session 37 Windows titlebar 合入（#452）+ 账本 housekeeping（#445-#451）· **session 38（#459-#465）：FEAT-02 语言设置 spec + i18n/Git 状态·Diff UI/字体拆分设置 + 底部 Output Tab + Windows titlebar 跟进 + dependabot（chrono/shiki 4.2/patch group）**。**下一步候选**（均需 Arbiter 拍板或外部资源 · 详见下方 🏁/「下一步候选」）：① **⚠️ 治理债 P0**：决策表 #8 Windows 治理定向 + 治理文档同步（本 PR docs/sync-governance-status 处理）② 非代码发布物料（域名 TLD 决策表 #16 / Logo #17 / 营销文案）③ Apple Dev Program（推 v0.2 · 解 SPIKE-06 §B blocked）④ version bump + release（当前 1.1.1 · 近期新功能值得发版）⑤ 可选代码改进（SPIKE-07.6 / Windows app-menu 快捷键 gap / deferred capture playbook）。**⚠️ 治理待决**：#452 + #431 已把 Windows 产品代码合入 main · 但锁定决策表 #8 仍写「Windows 推到 v0.4」· 待 Arbiter 定向。 | session end   |
| **Blocked by**            | **无 v0.1 GA blocker**（session 20 · 2026-04-26 决策 · v0.1 alpha 改 unsigned 模式 · SPIKE-06 §B Apple Dev Program 推 v0.2 · README + Release notes 写明 macOS Gatekeeper bypass 指引）· SPIKE-01/02 Phase B Ubuntu validated（PR #137-#139 · ADR-006 解除 caveat）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | 阻塞变化      |
| **Missing infra**         | 无（v0.1 GA 双平台已就位 · macOS unsigned + Linux deb/AppImage）· Apple Developer Program 推 v0.2（不阻塞 v0.1 alpha · v0.2 升级触发条件见 MVP-10 §I.D）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                | Phase 完成时  |
| **Required env/accounts** | ✅ rustup stable 1.95 / Node 20.17 / pnpm 9.15 / tauri-cli 2.x · ✅ Ubuntu 24 LTS（已就位 · session 19 PR #137-#139）· ⏳ Apple Dev（推 v0.2 · v0.1 alpha unsigned 模式不依赖）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | 新账号/工具时 |
| **v0.1 发布策略**         | **双平台 macOS + Ubuntu**（2026-04-25 session 19 SPIKE Phase B 完成 · 原 macOS-first S-3 升级）· v0.1.0-alpha macOS-only 已发 · v0.1.0 GA 双平台同步发布（.dmg + .deb/.AppImage）· Ubuntu 不再是最低优先 · 决策基线：PR #137 Ubuntu Phase B X11 108ms + Wayland 107ms / 30 stable · IME fcitx5 PASS · bundle build 成功                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | GA 前最终评估 |

---

## 📍 当前位置

**阶段**：**session 38 · 2026-06-14 · 治理文档同步 + 近期 feature/i18n/Output Tab 批合入（#459-#465）· main HEAD `0960644`**（见下方 Session 38 条目）。上一 **session 37 · 2026-06-04 · Windows titlebar 合入（PR #452）+ 账本 housekeeping（#445-#451 · 断链 43→17）**。上一 **session 36 · 2026-06-02/03 · 11 PR housekeeping 批（#434-#444 · gix 0.84 + Criterion perf 基线 + markdown-lint + ADR-023 链接/README sync + Windows bench）**。上一 **session 35 · 2026-05-30/31 · dependabot 4 PR + git2 0.21 major migration（PR #432）· 已归档 [`session-35.md`](./internal/session-history/session-35.md)**。上一里程碑 **session 33 · 2026-05-17 · MVP-20 Phase D 全 merged (#394) · v1.0 vision 代码侧完整收口**（Phase A/C/D 全链 · ADR-023 后 spec 全 flip done）
**日期**：2026-06-04（session 37 · housekeeping #445-#451 + Windows titlebar #452 · main `5dc81d0`）· 上一 2026-06-02/03（session 36 · housekeeping 批 #434-#444）· 上一 2026-05-30/31（session 35 · dependabot + git2 0.21 · PR #432）· 上一 2026-05-29（session 34 · Windows 适配 v0.4 · PR #431）· 上一 2026-05-17（session 33 · MVP-20 Phase A+C+D 全链 · merged #385-#394 · 均按 v2-D.2 + §2.14/2.15 流程 · 0 author 污染 · Phase D 主 agent 自实施 TDD）
**GitHub**：<https://github.com/tajiaoyezi/vibestation>（PRIVATE）
**已合入的 PR（滚动窗口 · 只保留当前 session · 更早见 `git log --all` + `docs/internal/session-history/`）**：

### Session 38（2026-06-14 · 治理文档同步 + 近期 feature/i18n/Output Tab 批合入 #459-#465 · 主 agent + Cursor/dependabot）

> session 37 同步点 `5dc81d0` 之后、HEAD `0960644` 之前合入 #459-#465 共 7 PR。

- **#459 FEAT-02 语言设置 spec**：FEAT-02 task spec 登记 · runtime launch smoke 记录
- **#464 i18n + Git 状态/Diff UI + 字体拆分设置**：Git 状态面板与提交栏接入语言字典 + Diff 面板接入语言字典 + 拆分界面字体与终端字体设置（5 个 commit · feat(i18n) Git 状态/提交栏 + feat(i18n) Diff + feat(settings) 字体拆分 + fix(i18n) 审查 blocker + fix(ui) 操作按钮/Diff 切换样式补全 + fix(ui) 全部暂存按钮 + 嵌套 button 压扁）
- **#465 PR #464 审查跟进**：字体栈与 Git 状态交互修复（Git 状态文件行 gutter 区域恢复可点击打开 diff · IPC 失败时回滚乐观更新快照）
- **#463 底部 Output Tab + 修复批**（主 agent + Cursor）：底部面板新增 Output Tab 记录 Git 操作日志（push/pull/fetch 进度 + 结果 + 错误展开）+ 底部 Diff Tab 删除（死按钮）+ diff overlay z-index 修复（2→5 避免悬浮按钮遮挡）+ MVP-18「链接 Pane 失败反馈」入口隐藏（{false &&} · 后端代码保留）+ PaneLinkCreateMenu/LinkManagePopover CSS 补全 · **代码审查（Explore subagent 对抗性）修复 3 Critical**（OutputPanel 响应式 / NetworkOpError 格式化避免 [object Object] / Terminal/styles.css 编码还原）+ 2 Medium（事件监听竞态 mounted 守卫 / tab reset 样式移到全局）
- **#460/#461/#462 dependabot 3 PR**（admin direct merge per ADR-016 v2-D.2 豁免）：#461 chrono patch · #460 npm patch group · #462 shiki 4.1.0→4.2.0 minor
- **当前态**：main `0960644` · open PR 2（#466/#467 dependabot patch · 待处理）· 工作树干净 · 本地 = 远程 main · ⚠️ 治理文档同步中（本 PR docs/sync-governance-status：PROGRESS 补录 + CLAUDE.md「下一步候选」去过时）

### Session 37（2026-06-04 · 账本 housekeeping #445-#451 + Windows titlebar 合入 PR #452 · 主 agent + dispatched 实施）

> session 36 同步点 `aa8c7b2` 之后、HEAD `5dc81d0` 之前合入 #445-#452 共 8 PR。

- **账本 housekeeping 7 PR（#445-#451）**：#445 PROGRESS 追平 session 36（#434-#444）· #446 session 32/33/34 归档至 `session-history/` · #447 14 处 session-history 断链修 · #448 README 双表 dedupe（归并入 Timeline 单一来源）· #449 session-index hygiene（10 处 `./internal/session-history/` 链接路径修）· #450 全仓相对链接审计 + 15 机械断链修（43→28）· #451 11 处判断类断链解决（28→17 · 剩余 17 全为模板占位符/全局规则引用/冻结历史/gitignored · 该留）
- **#452 Windows titlebar 合入**（merge `5dc81d0` · 实现者 Droid 12 commit + 主 agent 合入前修订 `6d08f2b`）：Windows 无边框窗口 + 前端自绘深色标题栏 + WebView2 配色统一 + **字体 latin 子集 bundle**（Inter + JetBrains Mono Variable · dist woff2 88.65KB · 收窄自整包 ~302KB）+ 终端关闭确认自绘模态框 + 字体/字号实时生效 + pane 分屏焦点切换修复（focusin/mousedown capture + optimistic update）+ MAX_PANES 限制
  - **质量门全过**（子 agent 实跑 · raw 进 PR #452）：Rust clippy -D warnings 0 / test core 901·0 / fmt 净 · 前端 typecheck 绿（lint/vitest 失败经核验为 Windows 已知假失败 CRLF + @solid-refresh · 非本分支回归）
  - **4 维对抗评审**（Tauri ACL / SolidJS / Windows-CSS / 安全治理）命中并对抗验证 2 high：① `color-scheme: dark` 错置 light 块 → 移入暗色块；② 字体 bundle 反转 MVP-11 §H.5 锁定 → **Arbiter 2026-06-04 拍板「保留 + 走流程」**（修订 §H.5/§E.3 审计可溯 + NOTICE 补 Inter/JetBrains Mono OFL-1.1 attribution + import 收窄 latin）· 其余 medium/nit 落档非阻塞 · Windows app-menu 快捷键 gap（Ctrl+T/W/,）**Arbiter defer + lib.rs 注释记录**
- **⚠️ 治理待决（决策表 #8 vs 现实漂移）**：#452 已把 **Windows 产品代码合入 main**（GUI 层）· 叠加 session 34 #431 的 Windows 适配 · 但锁定决策表 **#8 仍写「平台 MVP = macOS + Ubuntu · Windows 推到 v0.4」** → 待 Arbiter 定向：(a) Windows 正式提前立项（走 ADR 推翻 #8 + 建 Windows task spec）· 或 (b) 当探索性适配不入决策表（保 #8 文字 + 注记"探索中"）· **主 agent 不替拍**
- **当前态**：main `5dc81d0` · open PR 0 · 全仓断链 17（全部该留）· 本地 `feat/windows-titlebar` 分支保留（remote 未删 · 供 Windows 后续）· `.understand-anything/` 为无关预存 untracked（未提交）· session 35 已归档 [`session-35.md`](./internal/session-history/session-35.md)（M-2 窗口收为 36+37）

### Session 36（2026-06-02/03 · session 35 后 11 PR housekeeping 批 · gix 0.84 + perf 基线 + markdown-lint + 链接/状态 sync + Windows bench 可移植性 · 主 agent + Antigravity/Grok 派工）

> session 35 同步点 `9454b89` 之后、HEAD `aa8c7b2` 之前合入 #434-#444 共 11 PR（均 2026-06-02 merge · Arbiter approval 2026-06-03 standing 授权"review PASS 直接合"）· 本条一次追平。

- **依赖升级 3 PR**（dependabot · admin direct merge per ADR-016 v2-D.2 豁免 trailer · commit 自带 `Bumps X from A to B` audit ref）：**#434** `gitleaks/gitleaks-action` 2→3（`secret-scan.yml`）· **#435** cargo patch-updates group ×2（Cargo.lock）· **#436** `dirs` 5.0.1→6.0.0 major（Cargo.toml + lock · Windows `home_dir()` 跨平台依赖）
- **#438 gix 0.70→0.84 升级**（锁定决策表 #13 Git 读栈 · 走 `build/gix-0.84-migration` 分支 + PR + self-review）：读路径 API 适配（模块 / 签名 / feature 漂移）· 仅 `crates/core/src/git_log.rs` + Cargo.toml + lock · gate 全绿
- **#439 markdown-lint 行尾空格清理**（session 35 留的 next-step candidate ① 收口）：A 类删冗余尾空格 / B 类补空行保 markdown 硬换行语义 · 4 文件（session-history README + session-17/18 + MVP-12）· 甄别故意硬换行 vs 误后再清
- **#440 Criterion 性能基线量化**（v1.0 Phase E perf 部分 · `docs/runtime-evidence/perf-baseline-2026-06.md`）：gix 0.84 + 低风险批后基线快照 · 截图/录屏类已 ADR-023 弃用 · perf 数字量化保留
- **#441 + #442 悬空链接修**（ADR-023 删文件遗留）：**#441** 清 11 处 docs 对已删 capture 文档的活引用（保留历史记录措辞 · MVP-10..17/21 + README + v0.3-sprint）· **#442** 修 CLAUDE.md 决策表 #18 死链 `.claude/rules/runtime-evidence-location` → ADR-023
- **#443 README 状态表 sync**（Antigravity 实施 · 主 agent 独立 review · 含修复轮）：`docs/tasks/README.md` 与各 spec frontmatter 全量 `status` 对齐 · 13 spec README 旧标 `ready` → 真值 `done`（SPIKE-07 · MVP-04/05/06/08/09/10/12/13/14/15/16/17 · 多为 ADR-023 弃用 capture 后实际已 done）· review 修复轮去编造 PR#（`#330/#297` → 可溯源真值 `#113`/ADR-023 `#405`/`#409` 等）· deferred 项措辞改"ADR-023 弃用"· 仅改 README（+28/-28）
- **#444 Windows bench 可移植性修**（Grok 实施 · 主 agent 独立 review PASS · **零产品代码** · 仅 2 bench/test 文件）：① `git_sync_bench.rs` CRLF —— `pull_conflict_abort` 在归一化 `\r\n`/`\r` 后再断言 `local-conflict`（真断言非永真）· ② `pty_pool_bench.rs` warm_hit **bench-env 调查定性**（非产品 bug · 未改 `crates/core/src`）：bench 仅等 `idle_count>=1` 未等生产 `IDLE_MIN_AGE` 1.5s → 补 `wait_idle_ready` sleep；headless bench 未回 ConPTY DSR（`ESC[6n`）致 cmd 永久 stall → 补 `reply_dsr_if_needed`；bench 用 COMSPEC cmd.exe 与生产默认 pwsh 不一致 → `bench_shell()` pwsh→powershell→cmd 探测链 · 修后 `warm_hit_with_pool` ok · P50 523.91ms（pwsh.exe）· `cargo test -p vibestation-core` 901+ ok
- **新 agent 身份入册**：Antigravity（#443 · README 状态 sync · 首次）· Grok（#444 · Windows bench 修 · session 33 Phase D/E playbook 曾用）· 均 v2-D.2 trailer + 主 agent 独立 review gate
- **当前态**：main `aa8c7b2` · open PR 0 · 0 残留 worktree/分支 · session 35 next-step candidate ①（markdown-lint）已由 #439 部分收口 · ②（营销/Apple Dev/域名 TLD）③（deferred capture playbook）仍待 Arbiter 窗口

### Session 35（2026-05-30/31 · dependabot 4 PR + git2 0.21 major migration · PR #432 · 已归档至 [`session-35.md`](./internal/session-history/session-35.md)）

- **dependabot 4 PR 全清**（#427/#428/#430 安全三件套 admin merge）+ **git2 0.20→0.21 major migration（PR #432 · 决策表 #13 · Closes #429）**：19 处 string accessor `Option<&str>→Result` 编译期 breaking 语义保真适配 · 全 gate 绿 · main `9454b89` · CI 矩阵 ubuntu+windows 兜底全 success
- 详情见 [`session-35.md`](./internal/session-history/session-35.md)

### Session 34（2026-05-29 · Windows 适配 v0.4 milestone · S2V 规格驱动 · 无人值守 · PR #431 · 已归档至 [`session-34.md`](./internal/session-history/session-34.md)）

- **为项目适配 Windows 11（x64 MSVC）· 全程 S2V 规格驱动 · 无人值守 `/goal`**：`/s2v-prd` → `/s2v-init`（6 phase + 16 task + 6 ADR + 7 BDD · tier=solo 单分支）→ `/s2v-implement`（16 task 全 Done · 逐 task RED→GREEN→verify）· 推进决策表 #8 / ADR-006 原推 v0.4 的 Windows 路线
- **核心**：`pty.rs` cfg 分离 Unix 内核 + Windows ConPTY reader（修 2 个真实运行期 bug：reader join 死锁 → `Mutex<Option>` + `close_master()` · 自然退出漏检 → `try_wait()`）· shell 探测链 `pwsh→powershell→cmd` · external_term/config_import/keybinding/fs_watch 全平台分支 · 前端 `platform-windows` + `format-shortcut` 11 处 ⌘ 平台感知
- **CI 矩阵实跑闭合 deferred**（run 26638582117）：ubuntu-latest 全绿（闭合 Linux 回归）+ windows-latest 实跑 · 真实产出 `.exe`（NSIS 7.57MB）+ `.msi`（WiX 10.18MB）· ConPTY 真 spawn 实证
- **零回归**：全走 `#[cfg(target_os)]` 分支 · Unix 逻辑零改动 · DB schema 不变 · 5 维对抗式审查 0 confirmed · 单分支 60+ commit · Arbiter 2026-05-29 approve
- ⏳ deferred（环境性）：mac 全量回归（无 mac CI leg）· GUI critical UX path 目视（§2.14 Arbiter 窗口 · 进程级 ConPTY 已自动化兜底）
- 详情见 [`session-34.md`](./internal/session-history/session-34.md)

### Session 33（2026-05-17 · MVP-18/19/20 多 phase + 治理 ADR-021/022 + MVP-20 Phase A/C/D · merged #365-#394 · 已归档至 [`session-33.md`](./internal/session-history/session-33.md)）

- **MVP-18 Phase A/B/C 完整收官**（#344-#364）+ **MVP-19 实施启动 + W1/W2 + Phase C/D/E-impl 全 merged**（#365-#379 · 4-agent 真并行 · 文件域 disjoint · Phase E finalize defer Arbiter playbook #376）
- **MVP-20 Phase A/C/D 全链收口**（v1.0 vision rollback）：Phase A #385-#388（M1 revert-plan + Phase B 前端 + M2 backend + seam→@/bindings reconcile）· Phase C #391/#392/#390（Codex resume + Cursor wire + Grok CAPTURE-PLAYBOOK）· **Phase D #394**（`RollbackStatusKind` typed enum + 全局 `detect_crash_recovery` + `RollbackRecoveryBanner` · 主 agent 自实施 TDD · Arbiter 2026-05-17 22:04 approve）· 🟡 Phase E（runtime 证据 + Criterion 量化）defer Arbiter playbook 窗口
- 治理：ADR-021（CI mandate → 质量门）+ ADR-022（dispatch 范本去断链）proposed→accepted（#381-#384）· 全 v2-D.2 + §2.14/§2.15 · 0 author 污染
- **2026-05-21 audit polish**（#410/#411）：MVP-18 §F.3 fixture 契约 smoke + self-review 5 nit 闭合 · cargo test --workspace 1004+ tests/0 · main HEAD `d18425b`
- 详情见 [`session-33.md`](./internal/session-history/session-33.md)

> **Session 32**（2026-05-15/16 · #328-#364 · v1.0 vision 4 spec ready-gate 通过〔SPIKE-07 + MVP-18/19/20 `draft→ready`〕+ MVP-18 Phase A 实施启动 · 4-agent 并行预审 → 主 agent 核实 → Arbiter 拍板）已归档至 [`session-32.md`](./internal/session-history/session-32.md)。

### Session 30（2026-05-13 + 2026-05-14 · 跨 2 day 15 PR merged · 已归档至 [`session-30.md`](./internal/session-history/session-30.md)）

- **2 day 跨 15 PR merged**（2026-05-13 阶段 11 PR #295-#303 + 2026-05-14 末 5 项收尾 4 PR #304-#307）· 比 session 28 峰值 9 PR 跃升 67%
- 4-agent dispatch pool 首次同时跑（OpenCode + Codex + Droid + Cursor · 文件域 0 交叠）+ MVP-17 Phase A/B/C/E.4 完整代码收口
- §2.5.1 worktreeConfig 隔离完美 · 0 author 污染 · §2.15 stale base race 规则化（PR #298 · 来自 Cursor PR #297 实证）· OpenCode N=4 试金石通过留 pool
- session 末 5 项收尾全 done（A 归档 #305 · B 漂移 housekeeping #306 · C MVP-17 E.4 #307 · D drift 报告 spike-tmp · E dispatch TOC #304）
- 详情见 [`session-30.md`](./internal/session-history/session-30.md)

> **更早 session（22–29 及之前）**：见 `docs/internal/session-history/session-NN.md` 归档（M-2 滚动窗口 · session 33 整理）。

> **Session 21**（PR #173-#187 · v0.1.0 GA 发布配套 + GitHub Actions billing admin override 触发首次大规模 7 direct push + v0.1.1 双批 fix + PR #187 主 worktree dangling history close）已归档至 [`docs/internal/session-history/session-21.md`](./internal/session-history/session-21.md)。

> **Session 20**（PR #152-#172 · 19 PR · MVP-10 Phase B 完整闭环 + 2 critical/secondary bug fix）已归档至 [`docs/internal/session-history/session-20.md`](./internal/session-history/session-20.md)。

> **Session 19**（PR #117-#152 · 36 PR · 史上最高产）已归档至 [`docs/internal/session-history/session-19.md`](./internal/session-history/session-19.md)。

> **滚动窗口前**：session 18 及更早（PR #1-#116）的完整摘要请查 `git log --all --oneline | grep PR` · 或 `docs/internal/session-history/` 里的归档文件。本 PROGRESS 每 session 末按 M-2 规则整理（当前展开 session 22 + 23 · session 18/19/20/21 已归档至 `docs/internal/session-history/`）。

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
- [x] `docs/spikes/` + `docs/spike-artifacts/` + `docs/internal/session-history/` 3 目录建立 · 各有 README + 安全约束
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
- **🆕 决策表 #8（平台 MVP）vs 现实漂移**（session 37 · #452）：Windows 产品代码已两批合入 main（#431 适配 + #452 GUI 标题栏/字体/pane 焦点）· 但锁定 #8 仍写「Windows 推到 v0.4」· **待 Arbiter 定向**：正式提前立项（ADR 推翻 #8 + 建 Windows task spec）或当探索不入表 · 见 Session 37 条目

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
| 人类启动手册                                                                   | `docs/internal/SESSION-STARTUP.md` |
| 贡献指南 · 含用户拍板 gate                                                     | `CONTRIBUTING.md`                  |
| 分支保护 admin checklist                                                       | `docs/BRANCH-PROTECTION.md`        |
| Frontmatter validator + self-test                                              | `scripts/validate-task-spec.mjs`   |

---

## Session 日志

> **M-2 滚动规则**：本节只列归档索引 · 详细 session 摘要见 `docs/internal/session-history/<session-N>.md` · 全部 PR 历史见 `git log --all --oneline`。
> 当前活跃窗口（近 2 session · session 36 + 37）已在上方"已合入的 PR"段展开。

### 归档索引

| Session | 日期               | PR 范围    | 主题                                                                                        | 归档文件              |
| ------- | ------------------ | ---------- | ------------------------------------------------------------------------------------------- | --------------------- |
| 7       | 2026-04-18 ~ 04-19 | 见 git log | Spike W0 多 agent 并行 · 首行代码 + 4 Spike + 1 新 Spike                                    | git log（未单独归档） |
| 8       | 2026-04-19         | 见 git log | SPIKE 全收口 + MVP-01 Phase A 首行生产代码                                                  | git log（未单独归档） |
| 9       | 2026-04-19         | 见 git log | 三路并行 · MVP-01 Phase B 视觉骨架 + md 盘点                                                | git log（未单独归档） |
| 10      | 2026-04-19         | 见 git log | 三路收敛 · MVP-02 落地                                                                      | git log（未单独归档） |
| 11      | 2026-04-20         | 见 git log | MVP-03/SPIKE-08 落地 + ts-rs rollout + MVP-04 spec ready + Kimi 首次成功协作                | git log（未单独归档） |
| 12      | 2026-04-20         | 见 git log | 多 agent 四路并发 · v0.1 Git 能力闭环 + 终端画面闭环 + SPIKE W0 macOS 完结                  | git log（未单独归档） |
| 13 ~ 16 | 2026-04-21 ~ 04-22 | 见 git log | 未单独归档 · session 13 audit + Kimi 11 次协作 + MVP-04 Phase C/E + SPIKE W0 macOS 完结尾声 | git log（未单独归档） |

> **session 17-34 的逐 session 索引见** `docs/internal/session-history/README.md` 的「🧭 Session Archive Timeline」表（单一来源 · 各有 `session-NN.md` 归档文件）· 本表只保留 ≤16 的 git-log-only 早期 session（避免与 Timeline 三表并存再漂移 · 17-23+33 旧行已 dedupe · 2026-06-03）。

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
