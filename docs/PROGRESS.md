# 进度快照 · PROGRESS

> **定位**：当前状态面板（agent 和人类都先读本文件获取"我是谁 / 做到哪 / 下一步 / 卡点"）。
> **更新约定**：session end / 阶段切换 / 决策变化时手动更新。不要每个 commit 都更新（噪音大）。
> Session 历史归档到 `docs/session-history/`（Phase 3 已建立）——**不要**归档到 CHANGELOG（CHANGELOG 是 release-please 自动维护的发布日志）。
> **PR 列表滚动窗口规则**（M-2 · 2026-04-21 session 13 audit）：本文件"已合入的 PR"段**只保留最近 2 个 session 的摘要** · 更早的以 `git log --all` + `docs/session-history/` 归档文件为准 · 每 session 末整理。

---

## 📊 固定状态字段

| 字段                      | 值                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | 更新时机      |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| **Active branch**         | 见 `git branch --show-current`（本表不硬编码分支 · 避免 PROGRESS 和现实漂移 · H-2 · 2026-04-21）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | —             |
| **Latest commit**         | 见 `git log --oneline -1`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | —             |
| **Worktree status**       | 见 `git status` + `git worktree list`（三方 worktree 隔离 · 无 shared-tree 冲突）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | —             |
| **Unpushed branches**     | 见 `git branch -vv`（不在此处硬编码）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | —             |
| **Next concrete action**  | **session 20 进行中 · 14 PR merged · 主 agent 主线代码侧 100% 收官 + 2 critical/secondary bug discovered/fixed** · 已合：(a-h) PR #152-#159 入口任务 + 主线收尾 8 PR · (i) PR #160 PROGRESS sync mid · (j) **PR #161 critical bug fix · modal mount-time webview 虚假 click + §F.01/§F.03 截图（v0.1 GA blocker · spec §B.1 隐私关键 path 失效 · 5 轮 dev restart 调试 · 200ms guard）** · (k) PR #162 §F.02 partial + CLI 自动化 GUI 截图能力边界总结 · (l) **PR #163 secondary bug fix · theme dual-path · ThemeProvider listen settings_changed event + §F.02 split before/after 完整闭环（spec §F.02 实时生效 violation · status bar theme_set 不 emit 修复）** · (m) PR #164 dispatch §2.14 reviewer 启 dev mode 跑 critical UX path 教训规则化（PR #159/#161/#163 三连根因）· **剩余 GUI 截图任务**（Arbiter 本地 1 小时一次性）：(1) MVP-05 Phase D runtime capture（`scripts/capture/mvp-05/capture-phase-d.sh` 就位）· (2) MVP-10 §F.04 DevTools 0 outbound（CLI 不能 · 必须 Arbiter）· (3) MVP-04 §I 22 张截图 + 2 录屏（cargo test 7 PASS / 15 ignore-runtime 已就位 · 仅缺截图）· (4) MVP-09 Phase D runtime（stage/commit 流程截图）· **off-mainline**：MVP-10 Phase C/D/E（macOS 公证 / Linux AppImage / GitHub Release · 等 Apple Dev）· MVP-08 Phase E v0.2 deferred · SPIKE-06 §B Apple Dev（用户决策中）· dead code cleanup `theme_set` IPC handler（v0.2 推荐） | session end   |
| **Blocked by**            | SPIKE-06 §B Apple Dev Program 申请（**用户决策中** · $99/y · 审核 2d-2w · 影响 MVP-10 codesign 但不阻塞 MVP-04-09）· SPIKE-01/02 §B + MVP-01 Phase C · **Ubuntu 24 LTS 环境 · 已降为 v0.1 GA 最低优先**（S-3 · 2026-04-21 · 见下方"v0.1 发布策略"）· 不阻塞 macOS 主线                                                                                                                                                                                                                                                                                                                                                                                                                 | 阻塞变化      |
| **Missing infra**         | Ubuntu 24 LTS 环境（最低优先 · macOS 主线不依赖）· Apple Developer Program（SPIKE-06 §B + MVP-10 GA 前置）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Phase 完成时  |
| **Required env/accounts** | ✅ rustup stable 1.95 / Node 20.17 / pnpm 9.15 / tauri-cli 2.x · ⚠️ Ubuntu 24（最低优先 · 非主线）· ⚠️ Apple Dev（GA gate）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            | 新账号/工具时 |
| **v0.1 发布策略**         | **双平台 macOS + Ubuntu**（2026-04-25 session 19 SPIKE Phase B 完成 · 原 macOS-first S-3 升级）· v0.1.0-alpha macOS-only 已发 · v0.1.0 GA 双平台同步发布（.dmg + .deb/.AppImage）· Ubuntu 不再是最低优先 · 决策基线：PR #137 Ubuntu Phase B X11 108ms + Wayland 107ms / 30 stable · IME fcitx5 PASS · bundle build 成功                                                                                                                                                                                                                                                                                                                                                                                                                                                | GA 前最终评估 |

---

## 📍 当前位置

**阶段**：session 20 进行中 · 14 PR merged · **主 agent 主线代码侧 100% 收官 + 2 critical/secondary bug discovered/fixed** · session 19 已 36 PR 收官（PR #117-#152 · 详细见 session-19.md）· `MVP-04` Phase D shell 兼容 done · `MVP-05` Phase A/B/C 全 done · Phase D runtime capture 待 Arbiter 30 min · `MVP-08` Phase A/B/C/D 全 done · Phase E v0.2 deferred · `MVP-09` Phase A done · Phase B Status 面板 + CommitBar done（PR #118）· §D Criterion bench + §E 集成测试 done（PR #156）· **Phase C 错误流 UX 补强 done（PR #159 · 19 dialog/toast CSS class 全填 + Hook stderr Copy 按钮 + exit code · 修补隐藏 critical UX 缺口）** · `MVP-10` Phase A done · Phase B Sentry Spike done（PR #120）· **ADR-015 accepted by Arbiter（PR #152）· Phase B SDK 编码 done（PR #155 · 4 commits · 19 测试全过）· §C.4 endpoint UI + §G.4 H2 proof + §F capture guide + §2.13 dispatch rule done（PR #158）· §B.1 modal mount-time click guard fix（PR #161 · v0.1 GA blocker）· §F.02 theme dual-path fix（PR #163 · spec §F.02 实时生效闭环）· §F evidence 3/4 done（01/02/03 ✓ · 04 DevTools 待 Arbiter）** · `MVP-11` Native Feel Quality 全 done · 5/5 phase（macOS + Linux）· 15 ADR（含 ADR-015 accepted）· v2-D.1 规则稳态运行 · dispatch §2.14 reviewer dev mode 教训制度化（PR #164）
**日期**：2026-04-26（session 20 启动 · session 19 已归档至 session-19.md · 团队 = 主 agent + 本机 Kimi + Ubuntu Claude Code + Ubuntu Kimi · 当日具体派工见 dispatch prompts）
**GitHub**：<https://github.com/tajiaoyezi/vibestation>（PRIVATE）
**已合入的 PR（滚动窗口 · 只保留最近 2 session · 更早见 `git log --all` + `docs/session-history/`）**：

### Session 20（2026-04-26 · 主 agent 主线收尾）

入口任务（#152-#157 · 6 PR）:

- **PR #152**：ADR-015 Telemetry crash stack proposed → accepted by Arbiter · Claude Code 主 agent · 解锁 MVP-10 Phase B Sentry SDK 编码
- **PR #153**：session-history/session-19.md 归档（36 PR · MVP-11 全 done + MVP-05 Phase A/B/C + ADR-006 Ubuntu validated + branch protect 机械化 · 168 行 · 9 主题分组）· Claude Code 主 agent
- **PR #154**：PROGRESS.md M-2 滚动窗口同步 · session 19 移交至 session-19.md 归档 + session 20 入口段建立 · Ubuntu Claude D2
- **PR #155**：MVP-10 Phase B Sentry SDK 主体 · 4 commits · `crates/core/src/telemetry.rs` 268 行（ADR-015 §决策约束 · `default_integrations: false` + `send_default_pii: false` + `before_send` 删 contexts.trace）· `crates/core/tests/telemetry_pii_test.rs` 6 PII 测试 · IPC commands `telemetry_opt_in_set` / `telemetry_status_get` / `app_version_get` + permission · `install_panic_hook` + `try_init_sentry_from_env` · `TelemetryOptInModal` 阻塞 WelcomePage · §B.4 atomic 双门控（19 测试全过 · `should_send_telemetry` runtime opt-in 实时生效）· Claude Code 主 agent · B5
- **PR #156**：MVP-09 §D Criterion bench（git_ops_bench · stage_single_file / commit_typical / stage_all_1000）+ §E 集成测试（`crates/core/tests/git_ops_integration.rs` · Linux 平台基线证据 · stage 0.26ms / commit 0.35ms / stage_1k 31.5ms · 远低于 spec D 性能要求 380×/1400×/63× 余量）· Ubuntu Claude · B4
- **PR #157**：ADR README 索引同步 + 决策表 #10 行（含 round 2 字节级 self-fix · round 1 Kimi 误覆盖 ADR-015 PR #152 措辞 · round 2 用 `git checkout origin/main -- <file>` 字节级恢复）· Ubuntu Kimi · U2

主 agent 主线收尾（#158-#159 · 2 PR）:

- **PR #158**：MVP-10 Phase B follow-up · §C.4 收集端点 host 公开显示 + 复制按钮（`SENTRY_ENDPOINT_HOST` OnceLock + `Copy` 按钮 + "Not configured" fallback）+ §G.4 H2 regression proof（临时改 `font_family` ts(rename) → tsc FAIL 6 处 · annotation rollback）+ §F runtime evidence CAPTURE-GUIDE.md（4 张 GUI 截图采集步骤 · 体积约束 · commit 指令）+ `dispatch-prompt-template.md §2.13`（PR #157 round 1 教训 · 索引同步类禁 inline 已被其他 PR 改过的源文件 · 必须 `git checkout origin/main -- <file>`）+ 顺手修 PR #156 留下的 2 个 clippy errors（`&PathBuf` → `&Path` · `len() > 0` → `!is_empty()`）· Claude Code 主 agent
- **PR #159**：MVP-09 Phase C 错误流 UX 补强 · 发现 19 个 vs-commit-* / vs-toast-* / vs-dialog-* CSS class 在主 styles.css 完全无定义（裸 HTML 显示 · 隐藏 critical UX 缺口）· 新建 `web/src/panels/CommitBar/styles.css` 363 行（Calm Studio token + scale-in/slide-up 动画）+ Hook stderr "Copy" 按钮（spec §C.4 "可复制"）+ exit code 显示（`hook 退出码 N · 显示 stderr 最后 20 行`）· Claude Code 主 agent

GUI capture 探索 + 2 critical/secondary bug fix（#160-#164 · 5 PR）:

- **PR #160**：PROGRESS.md session 20 mid sync · 补 PR #154-#159 6 PR · 主 agent 主线收尾段落新建 + MVP v0.1 进度 phase 级别详化 · Claude Code 主 agent
- **PR #161 🔴 CRITICAL BUG FIX**：Modal mount-time click guard · 实测发现删 DB 重启 dev mode · `telemetry_opt_in` 在 12.5 秒内被自动写入 · modal **用户完全看不见** · 5 轮 dev restart + DB watcher 半秒 poll 调试定位 webview 启动 race · WKWebView 把"启动 ready"事件误派发到 modal 内第一个 focusable button（Decline）· SolidJS event delegation 路由到 onClick · 触发虚假 `decide(false)` · 加 200ms `MOUNT_CLICK_GUARD_MS` · `onMount` 记录 mountedAt · `decide` 入口 `isEarlyClick()` 检查 silently 丢弃 mount-time 虚假 click · spec §B.1 "首次启动弹对话框 · 阻塞欢迎页" v0.1 GA blocker 修复 + 顺手交付 §F.01 settings-panel + §F.03 telemetry-opt-in modal 截图（247 KB / 324 KB · ADR-011 R4 < 500 KB）· Claude Code 主 agent
- **PR #162**：MVP-10 §F.02 partial · dark theme single state（settings-realtime 349 KB）+ CLI 自动化 GUI 截图能力边界总结（macOS `screencapture -l <CGWindowID>` + Swift `CGWindowListCopyWindowInfo` + `cliclick` + `osascript` 能做 vs 不能做完整 inventory · webview 内部 keyboard event 路径分离 · DevTools UI / 复杂 LLM stream 实机交互不能脚本化）· 期间发现 secondary dual-path bug 留 follow-up · Claude Code 主 agent
- **PR #163 🟡 SECONDARY BUG FIX**：theme dual-path · `ThemeSwitch.theme_set` IPC **不 emit `settings_changed`** · status bar click DB 写但 UI 不刷 · violate spec §F.02 "切 theme 后实时生效 · 无重启" · `ThemeProvider` 加 `listen("settings_changed")` 监听 · `ThemeSwitch.handleClick` 双门控调用 setTheme（同步 UI active）+ updateSettings（持久化 + emit）· `AppearanceGroup` 删 redundant `themeCtx.setTheme` · spec §F.02 实时生效闭环 + split before/after evidence（dark + light · 02-settings-realtime-after-light.png 381 KB）· Claude Code 主 agent
- **PR #164**：dispatch §2.14 reviewer 启 dev 模式跑 critical UX path 教训规则化 · session 20 三连 bug（PR #159 / #161 / #163）共同根因 reviewer 把 "cargo test green + pnpm typecheck 0 + ts-rs contract 一致" 等同 "PR 可 merge" · 但 GUI/IPC 类 PR critical path 必须 dev mode 跑过才能 catch webview race / event delegation / dual IPC path / CSS missing 类问题 · 全局 rule 15 在 reviewer 阶段的具体落地 · implementer §2.3 + reviewer §2.14 双管齐下 · Claude Code 主 agent

剩余 GUI 截图任务（CLI 不能跑 · 等 Arbiter 本地 1 小时一次性）:

- MVP-04 §I 22 张截图 + 2 段 30s 录屏（zsh/bash/fish + Claude/Codex CLI 实机 · cargo test 已 7 PASS / 15 ignore-runtime · 仅缺 GUI 录屏）
- MVP-05 Phase D `metrics-mvp-05.md` 实测 + 4-7 张 pane split + memory.sh 量化（capture-phase-d.sh 已就位）
- MVP-09 Phase D runtime（stage/commit 流程截图 · 性能数据已 done by PR #156）
- MVP-10 §F.04 0 outbound DevTools network panel（CLI 完全不能 · 必须 Arbiter）

主 agent 已完成的 §F evidence: §F.01 ✓ §F.02 ✓ §F.03 ✓ · 仅 §F.04 待 Arbiter（3/4 done）。

> **Session 19**（PR #117-#152 · 36 PR · 史上最高产）已归档至 [`docs/session-history/session-19.md`](./session-history/session-19.md) · 详细 PR 摘要 + 9 主题维度 + 13 项特色教训查归档文件。M-2 滚动窗口规则（每 session 末整理）首次完整执行（session 18 → session 19 归档由 PR #134 处理 · session 19 归档由 PR #153 处理）。

> **滚动窗口前**：session 18 及更早（PR #1-#116）的完整摘要请查 `git log --all --oneline | grep PR` · 或 `docs/session-history/` 里的归档文件。本 PROGRESS 每 session 末按 M-2 规则整理（保留最近 2 session · 当前窗口 session 20 · session 18/19 已归档至 `docs/session-history/`）。

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

| 信号                        | 触发                                                                                                                                                                                                                                              |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ✅ Phase 1-4 Pre-code 完备  | **已达成**（2026-04-18 session 5 · 4 PR 全 merge）                                                                                                                                                                                                |
| ✅ Spike W0 启动            | **已达成**（session 7 · 首行 Rust 代码 · SPIKE-01 Phase A PASS）                                                                                                                                                                                  |
| ✅ Spike W0 macOS 全通过    | **已达成**（session 11 开场 · SPIKE-01/02 Phase A · SPIKE-03/04/04.5/05/05.5/08 全 done · SPIKE-06 §A harness done · 36 样本 + §B Apple Dev 阻塞外部资源）                                                                                        |
| 🟡 Spike W0 全平台通过      | SPIKE-01/02 Phase B Ubuntu（阻塞环境）+ SPIKE-06 36 样本 + Apple 申请                                                                                                                                                                             |
| 🔴 Spike 任一 CRITICAL Fail | 触发 fallback + ADR supersede                                                                                                                                                                                                                     |
| ✅ MVP 实施启动             | **已达成**（session 8 · MVP-01 Phase A · ADR-003/005/006/007 全 accepted）                                                                                                                                                                        |
| 🟡 MVP v0.1 进度            | **3/10 done + 7/10 ready**（MVP-02/03/07 done · MVP-01/04/05/06/08/09/10 ready · MVP-11 Native Feel done）· **session 20 主 agent 主线代码侧 100% 收官**：MVP-04 Phase A-F 全 done 仅 §I 截图待补 · MVP-05 Phase A/B/C 全 done · Phase D 待 GUI capture · MVP-08 Phase A-D 全 done · Phase E v0.2 deferred · **MVP-09 Phase A/B/C done · Phase D 性能 done by PR #156（runtime 截图待 GUI）** · **MVP-10 Phase A/B 全 done（含 §B.1 modal 阻塞 + §C.4 endpoint UI + §F.02 实时生效 + §G.4 H2 proof + §F evidence 3/4 done · PR #161 critical bug fix 解锁 v0.1 GA · PR #163 secondary dual-path fix 闭环 §F.02 acceptance）** · **主线收敛到 Arbiter 本地 1 小时 GUI 截图（4 类） + spec frontmatter done 翻转 + MVP-10 Phase C/D/E 打包**（不再有需要新写代码的主线 task） |
| 🎯 v0.1 GA                  | MVP-01..10 全过 §10.1 + §10.6 终端正确性矩阵 + §10.3 跨平台                                                                                                                                                                                       |
| 🔴 连续 2 周 < 5h 投入      | 触发 hibernation（`implementation-plan.md §10.5`）                                                                                                                                                                                                |

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

| Session | 日期 | PR 范围 | 主题 | 归档文件 |
|---|---|---|---|---|
| 7 | 2026-04-18 ~ 04-19 | 见 git log | Spike W0 多 agent 并行 · 首行代码 + 4 Spike + 1 新 Spike | git log（未单独归档） |
| 8 | 2026-04-19 | 见 git log | SPIKE 全收口 + MVP-01 Phase A 首行生产代码 | git log（未单独归档） |
| 9 | 2026-04-19 | 见 git log | 三路并行 · MVP-01 Phase B 视觉骨架 + md 盘点 | git log（未单独归档） |
| 10 | 2026-04-19 | 见 git log | 三路收敛 · MVP-02 落地 | git log（未单独归档） |
| 11 | 2026-04-20 | 见 git log | MVP-03/SPIKE-08 落地 + ts-rs rollout + MVP-04 spec ready + Kimi 首次成功协作 | git log（未单独归档） |
| 12 | 2026-04-20 | 见 git log | 多 agent 四路并发 · v0.1 Git 能力闭环 + 终端画面闭环 + SPIKE W0 macOS 完结 | git log（未单独归档） |
| 13 ~ 16 | 2026-04-21 ~ 04-22 | 见 git log | 未单独归档 · session 13 audit + Kimi 11 次协作 + MVP-04 Phase C/E + SPIKE W0 macOS 完结尾声 | git log（未单独归档） |
| **17** | 2026-04-23 | **#99-#105** | MVP-04 Phase F 收口 + MVP-08 Phase A/B/C 落地 + PR Actions 分钟节流 | [`session-17.md`](./session-history/session-17.md) |
| **18** | 2026-04-25 | **#106-#116** | 4 track 并发极致产出 · 11 PR · 5 Phase 落地 + 3 spec ready 加强 | [`session-18.md`](./session-history/session-18.md) |
| **19** | 2026-04-25 | **#117-#152** | MVP-11 全 done + MVP-05 Pane 落地 + ADR-006 Ubuntu validated + branch protect 机械化 · 史上最高产 36 PR | [`session-19.md`](./session-history/session-19.md) |
| **20** | 2026-04-26 | **#152-#172** | MVP-10 Phase B 完整闭环 + 2 critical/secondary bug fix + dispatch §2.13/§2.14 教训规则化 | [`session-20.md`](./session-history/session-20.md) |

### 跨 session 关键里程碑

- **首行代码**：session 8 · PR #28 · MVP-01 Phase A Tauri 壳 + SolidJS
- **Spike W0 macOS 100% 完结**：session 12 · 6 SPIKE 全 PASS
- **v0.1 10 MVP spec 全 ready**：session 15 · MVP-10 PR #88 + MVP-05 PR #89
- **MVP-08 主线里程碑**：session 17 · PR #105 · Diff 视图前端集成
- **MVP-11 Native Feel Quality 全 done**：session 19 · 11 PR
- **ADR-006 Ubuntu validated · v0.1 GA 双平台**：session 19 · PR #138
- **ADR-015 Telemetry accepted · MVP-10 Phase B 解锁**：session 20 · PR #152
- **CRITICAL bug rescue · v0.1 GA blocker**：session 20 · PR #161 · modal mount-time webview 虚假 click guard

---

**本文件每次 session end / 阶段切换 / 重大决策变化时手动更新。机械字段 Phase 5 CI 后接 hook 自动刷新。**
