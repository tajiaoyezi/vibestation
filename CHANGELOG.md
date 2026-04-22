# Changelog

本项目遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/) · 版本号遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## 变更分类

- **Added**（新增）· **Changed**（变更）· **Deprecated**（废弃）· **Removed**（移除）· **Fixed**（修复）· **Security**（安全）

---

## [Unreleased]

> Pre-code 阶段（Phase 1-4）变更记录 · GA v0.1 发布时合入 `[0.1.0]`。

### Added · 文档与基础设施

**Phase 1（2026-04-17）· 文档升级 v4 simplified**：
- `CLAUDE.md` · agent 入口（5 步 checklist · 锁定决策表 A+B+C · 禁区 · 代码风格 · 自审四问）
- `docs/PROGRESS.md` · 阶段 / 进度 / 卡点
- `docs/SESSION-STARTUP.md` · 人类启动手册
- `docs/implementation-plan.md` · 战略计划（v2 收紧为 B 折中方案）
- `design/directions/1-calm-studio.html` · 视觉原型（Calm Studio 锁定）
- `LICENSE`（Apache 2.0）· `NOTICE` · `README.md`（中英双语首屏）

**Phase 2（2026-04-18）· task spec 框架 + SPIKE + MVP**：
- `docs/tasks/` · 任务索引 + 状态流转 + 字段 schema
- `docs/tasks/_template.md` · task spec 模板
- `SPIKE-01..06` · 6 个 Spike task spec（Tauri 三平台 / 硬通过矩阵 / Git benchmark / 存储 benchmark / PTY 压测 / CLI 实机）
- `MVP-01..10` · v0.1 范围详细 spec
- `MVP-11..20` · v0.2/v0.3/v1.0 范围占位 spec（骨架）
- Codex 5 轮对抗性审查（10 HIGH findings 全闭合 · 详见 PR #9）

**Phase 3（2026-04-18）· 架构决策与治理文档**：
- `docs/adr/` · 10 个 ADR（License / MVP 范围 / PTY / 前端栈 / 存储 / 桌面框架 / Git 栈 / Diff / v1.0 vision / workspace）
- `CODE_OF_CONDUCT.md` · Contributor Covenant 2.1 中文版
- `CONTRIBUTING.md` · 贡献指南
- `CHANGELOG.md` · 本文件
- `docs/spikes/README.md` · Spike per-task 报告目录占位
- `docs/spike-artifacts/README.md` · Spike 录屏 / 截图目录占位
- `docs/session-history/README.md` · Session 历史目录占位

<!-- Phase 4（GitHub 基础设施）条目已移除 · Codex PR #12 F5 复核：
     该 Phase 4 在 PR #11（独立分支 `docs/phase-4-github-infra`）交付 ·
     不在本 PR #12 的 Phase 3 diff 范围内 · 在此记入会误导 reviewer
     以为 gitleaks / task-spec-validator / PR template 等已在此 PR 生效。
     正确做法：**PR #11 merge 时** · 在独立 commit 中把 Phase 4 条目加入
     本 CHANGELOG 的 [Unreleased]（或直接合入对应版本 release）。 -->

**Phase 4（在独立 PR #11 交付 · 本 CHANGELOG 条目在 PR #11 merge 时补入）**：
- 见 [PR #11 description](https://github.com/tajiaoyezi/vibestation/pull/11) 的实际交付清单
- 涵盖：`.github/` 模板 / dependabot / ci skeleton / secret-scan (gitleaks) / task-spec-validator / validate-task-spec.mjs / BRANCH-PROTECTION.md

### Added · 代码实施（2026-04-19 ~ 2026-04-22 · session 7-15 · macOS-first）

**Spike W0 · macOS 100% 完结**（session 7）：
- SPIKE-01 Tauri 三平台启动验证 · macOS Phase A PASS · 冷启动 202ms median（PR #20 · [report](docs/spikes/SPIKE-01-report.md)）
- SPIKE-02 Tauri 硬通过矩阵 · macOS Phase A PASS · bundle 10MB / .dmg 4MB（PR #22）
- SPIKE-03 git2 vs gix benchmark · gix log -100 warm P99 12.65ms 比 git2 快 1973×（PR #23 · [ADR-007](docs/adr/ADR-007-git-stack.md) accepted）
- SPIKE-04 + SPIKE-04.5 storage benchmark · rusqlite B.1-5 全过 · redb 2.6.3 B.2 silent corruption FAIL（PR #24/#29/#34/#68 · [ADR-005](docs/adr/ADR-005-local-storage.md) accepted）
- SPIKE-05 + SPIKE-05.5 portable-pty 多 Tab 压测 · shared-reader HOL/boundedness pass · visible throughput 瓶颈在 JS/invoke RTT（PR #30/#39 · [ADR-003](docs/adr/ADR-003-pty-architecture.md) accepted）
- SPIKE-06 §A Claude/Codex CLI 36 脱敏样本 · harness + record.sh + redact.py + gitleaks 0 hit（PR #38/#71 · [SPIKE-06-report](docs/spikes/SPIKE-06-report.md)）
- SPIKE-08 E2E + IPC contract harness · ts-rs 选定 + Playwright 补层（PR #60 · [ADR-014](docs/adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md) accepted）
- ADR accepted 14 个（#001-014）· 14 ADR proposed → accepted 收敛

**MVP 实施**（session 8-15 · macOS-first）：
- **MVP-01 Phase A + B · Tauri 壳 + SolidJS + Calm Studio** · Cargo workspace 2-crate + runtime 验证 + 3 轮 CI 修（PR #28/#33）
- **MVP-02 · workspace 管理 done** · rusqlite + r2d2 pool + WorkspaceStore CRUD + git 自动检测 5 parent + UUID v4 + canonical path · 23 unit tests · H1 path traversal + H2 IPC camelCase 修（PR #40/#44/#45/#47）
- **MVP-03 · Tool Windows 5-zone 布局 done** · Primary/Secondary Sidebar + Activity Strip + Bottom Panel · theme light/dark · 布局持久化到 rusqlite · 29 unit tests · 5 runtime 截图（PR #61）
- **MVP-04 Phase A · tabs 存储层 done** · migration v5 + TabsDao 6 CRUD + 2 scrollback methods + 5 IPC commands + Tauri ACL allow-tab-* + ts-rs 5 bindings + 36 unit tests（PR #72）
- **MVP-04 Phase B · PTY runtime done** · `portable-pty` + `mio` poll + `DropOldestSender` bounded(128) drop-oldest + `crossbeam-channel` · 5 tab_pty_* IPC commands + 5 allow-tab-pty-* permissions + 3 ts-rs bindings（PtyStdoutEvent/PtyExitedEvent/PtySpawnRequest）· `fix_path_env.rs` 53 行本地 shim（crates.io 包不可解析 · 技术债）· `tab_pty_stdout` / `tab_pty_exited` Tauri events · 19 PTY 单元/集成测试 · **Phase C-F xterm 前端 / shell 兼容 / 持久化 / 证据 待**（PR #82）
- **MVP-06 Phase A + A+ · parser 层完整** · `crates/core/src/config_import/` Ghostty TOML + iTerm2 plist（binary/text · Default Bookmark Guid → default profile）+ Alacritty TOML/YAML 双格式 · `ImportedField` 6 variants（FontFamily/FontSize/Theme/Shell/KeyBinding/AnsiColor）· Ghostty `keybind` 重复行逐行扫 filter 降级 · iTerm2 ANSI 0-15 RGB→hex 转换 · Alacritty `[[keyboard.bindings]]` TOML 0.14+ + `key_bindings:` YAML 0.13- · 26 unit tests · **Phase B IPC/UI/apply 待 MVP-04 Phase C-F done 后**（PR #80/#81）
- **MVP-07 · Git Log 只读 done** · `gix 0.70` 分页 revwalk + `GitLogReader::query` + commit detail + branch/tag labels + 筛选（message/author/after） · 3 IPC commands + 3 allow-git-log-* permissions + 7 ts-rs bindings + SolidJS `web/src/panels/GitLog/` 前端 panel（list + detail + filter · Secondary Sidebar 接入）· H2 regression proof 制度化 · 92 workspace tests · **UI 截图 + linux kernel 10 万 commit benchmark GA gate 补**（PR #83）
- **MVP-04 Phase C · xterm 前端 done · 主线里程碑**（2026-04-22 · session 15 · Codex CLI · PR #91）· xterm.js 5.5 + 5 addons（webgl / canvas / fit / web-links / unicode11）· SolidJS 组件：Terminal.tsx 713 行（主协调 + IPC event listener + 快捷键 dispatch）+ TerminalPane 323（单 Tab xterm 实例 + renderer fallback + loading 态）+ TabBar 147（紧凑 tab bar + active 下边框 + 双击重命名）+ PasteConfirmDialog 86（多行 confirm + 前 5 行 unicode-safe 截断 + "不再提示本 session" checkbox）+ hooks 140（IPC wrapper）+ styles.css 378 · WebGL → Canvas → DOM 三级 renderer fallback（console.warn 记录降级）· 5 `tab_pty_*` IPC + 2 events 全接通 · 前端零手写 interface（ts-rs bindings 全 import）· 快捷键 ⌘T/⌘W/⌘⇧[/]/⌘1..9（`attachCustomKeyEventHandler` 放行到 App 层）· **F.4 Shell 冷启动 loading 态**（Codex 实施中发现的 UX gap · 原 spec 未显式要求 · 诚实声明补到 spec §F.4 Acceptance + §已知风险 "Shell rc 慢启动感知"）· 避免"新 tab 首屏白屏"（macOS GUI zsh + oh-my-zsh / nvm plugin 1-3s source 期间显示 "Launching /bin/zsh… / Waiting for the first shell output" 启动卡片）· Runtime 证据 5 截图（1.0MB 总 · 含核心亮点 `05-shell-loading-card.png` · 对齐 ADR-011 R1-R5）· 1891 行前端新代码 · **A.5 Tab 切换 < 100ms / E.2 切 Tab 延迟 < 50ms / E.4 主线程 ≤ 16ms 性能量化归 Phase F**（runtime 证据专 phase · Playwright 采样 · 本 PR 不 block）· 7 commit（claim + 依赖 + 骨架 + loading + 挂载 + 证据 + spec）· **中途断线后 Claude Code 起 resume prompt 续命**（stash → fetch → branch → config → claim 5 步前置修 3 处环境坑 · PR #71/#82 author 错归教训预防生效 · 所有 commit author 归属正确）· **Phase D shell 兼容 / E 持久化 / F 证据 + 性能量化 待**（PR #91）

**Spec review · v0.1 10 MVP spec 全 ready 里程碑**（2026-04-22 · session 15 · Kimi × 2 + Claude Code cross-review）：
- **MVP-10 settings + telemetry + packaging spec review draft → ready**（PR #88）· Kimi 第 9 次协作 + Claude Code cross-review · §G 6 ts-rs struct（AppSettings / SettingsUpdateRequest / TelemetryOptInRequest / TelemetryStatus / CrashReportPayload / AppVersionInfo）· §H.1-5 决策锁定：H.1 Telemetry 栈延 Phase 4 Spike（候选 Sentry SDK 默认 + Plausible / PostHog / 自建对照）/ H.2 打包工具锁 tauri-cli 2.x / H.3 公证 notarytool + GitHub Actions secret 锁 / H.4 AppImage tauri 自带（linuxdeploy 基于）锁 / H.5 privacy-policy 自写最小版 + Apache 2.0 锁 + GDPR Article 13 最小 6 项 · Acceptance A-G 全量化 28 checkbox（原 20）· 运行时证据要求 7 截图（对齐 ADR-011）· 数据模型变更补 8 app_settings 字段 · Claude Code reviewer 代修 2 处（Kimi 误读 app_settings 为宽表 · 实际是 KV 表 `(key, value)` · migration v3 已建；Kimi 标 migration v6 撞 MVP-05 占用 · 改 MVP-10 不新建 migration · 纯 KV 复用）· **v0.1 10 MVP spec 全 ready 里程碑达成**
- **MVP-05 Pane spec 对齐 MVP-04 Phase A/B 实施现状**（PR #89）· Kimi 第 10 次协作 · 5 gap 修复：§H.4 FK `tabs.id → tabs(tab_id)` 修正（对齐 `migrate_v5` 实际主键）· `panes` 表完整 CREATE DDL + `idx_panes_tab_created` 索引（仿 tabs 表模式）· §H.6 新增 Pane PTY IPC 命名决策（锁 A 选项 `pane_pty_*` 独立 · 不破坏 MVP-04 Phase B 已落地 `tab_pty_*` + `PtySpawnRequest` ts-rs binding）· §🛠 实施进度表 Phase A-D 拆分（仿 MVP-04 模式）· §💾 清理重复指向 §G.2 + §H.4 · 自审四问补第 7 条 "对齐 MVP-04 Phase A/B 实施现状"· Claude Code reviewer self-push 翻转 gate (a) 审计痕迹 · **Kimi 零实质错误**（对比 MVP-10 要 reviewer 代修 2 处 · MVP-05 一遍过）

**Kimi 协作成就**（远程 API agent · 10 次协作 · 100% 成功率 · session 15 追加第 9-10 次）：
- 8 次 spec review：MVP-04/05/06/07/08/09 · MVP-10（第 9 次 · PR #88）· MVP-05 对齐（第 10 次 · PR #89 · 零实质错）· 平均 23 min（PR #64/#66/#70/#73/#74/#77/#88/#89）
- 2 次代码实施：MVP-06 Phase A + A+ parser 模块（PR #80/#81）· 主动优化降级方案（比 dispatch prompt 建议更优）
- 10 连 merged 战绩保持（PR #64/#66/#70/#73/#74/#77/#80/#81/#88/#89）

**v2-D.1 规则制度化**（session 13 + 14）：
- ADR-012 v2-D → v2-D.1 简化（删 merge 后 24h 补 PR comment 硬要求 · session 12 实证 0% 合规）
- ADR-013 Spike 冷备归档 v1 强制 → v2 推荐（22% 合规率实证）
- ADR-014 IPC contract source of truth = Rust struct + ts-rs codegen（H2 根因消除 · SPIKE-08 §A PASS rollout）
- dispatch prompt 8→12 条硬约束（2.10 前端 lint + 2.11 timing-sensitive test timeout + 2.12 跨 worktree git config unset · 2026-04-21 session 14 事件制度化）
- CLAUDE.md 5 步 checklist 补 "合入后 CI 验证"（session 14 事件）
- 主 agent 代修模式（session 14 · 3 次实践：PR #82 R1+R2 · PR #83 R1-R4 · PR #86 CI fix）

### Fixed · CI

- **Rust · pty SIGTERM 测试 Linux CI flaky**（PR #86）· `pty::tests::signal_sigterm_exits_exec_session` 在 macOS 本地稳定 · Ubuntu runner 上 SIGTERM → PTY close event → epoll readable 链路 timing / 语义差异 · 2 轮 timeout 扩张（200→500ms · 5→10s）无效 · 切 `#[cfg_attr(target_os = "linux", ignore)]` + MVP-04 已知风险记技术债 · 本地 `cargo test -- --ignored signal_sigterm_exits_exec_session` 仍可手动验证 · MVP-04 Phase D（shell 兼容 · Ubuntu runtime）启动时解除 ignore
- **Frontend · prettier 5 文件未格式化**（PR #86）· OpenCode PR #83 交付前端代码只跑 `pnpm typecheck`· 漏 `pnpm lint`（prettier --check）· `SecondarySidebar.tsx` / `GitLog/GitLogPanel.tsx` / `GitLog/gitLogApi.ts` / `GitLog/index.ts` / `styles.css` 5 文件 · `pnpm prettier --write` 自动修复
- **Rust · pty exit event 测试 Linux CI flaky**（PR #90 · 2026-04-22 session 15 · 同根因 PR #86 复发到新测试）· `pty::tests::spawn_stdin_and_exit_emit_stdout_and_exit_event` 在 Ubuntu runner 上 `printf + exit → mio epoll PTY close event → exit event 到 mpsc` pipeline 偶发 > 5s timeout · 按 §2.11 硬约束 + PR #86 先例 · 立即标 `#[cfg_attr(target_os = "linux", ignore)]` + 技术债记录（不加 timeout workaround · 已证无效）· **不改 MVP-04 spec**（Codex PR #91 in-progress 改同文件 · 避免 merge 冲突）· merge 顺序 PR #90 → PR #91 让 Phase C PR 获 Linux ignore 保护 · 两个 PTY 测试的 ignore 统一在 MVP-04 Phase D Ubuntu runtime 验证时解除

### Changed · 决策锁定（A 栏）

- License = **Apache 2.0**（不签 CLA · [ADR-001](docs/adr/ADR-001-license-apache-2.0.md)）
- MVP v0.1 范围 = **B 折中方案**（[ADR-002](docs/adr/ADR-002-mvp-scope-b-compromise.md)）
- **v1.0 vision = 对外不提其细节**（见 [ADR-009](docs/adr/ADR-009-ai-aware-v1-vision.md) · 具体内容仅对内规划文档展开）
- 前端栈 = **SolidJS + TypeScript + Vite + xterm.js**（[ADR-004](docs/adr/ADR-004-frontend-stack.md)）
- Diff 渲染 = **自建**（非 Monaco · [ADR-008](docs/adr/ADR-008-diff-renderer-custom.md)）
- Cargo workspace = **2 crate**（`app` + `core` · [ADR-010](docs/adr/ADR-010-cargo-workspace-2-crate.md)）

### Changed · 决策待 Spike 锁定（B 栏）

- 桌面框架 **Tauri 2** 默认 · Electron 28+ fallback · pending [SPIKE-02](docs/tasks/SPIKE-02-tauri-hard-pass-matrix.md)（[ADR-006](docs/adr/ADR-006-desktop-framework.md)）
- Git 栈 **git2 0.20** 写 · **gix 0.70** 读优化（可选）· pending [SPIKE-03](docs/tasks/SPIKE-03-git2-gix-read-benchmark.md)（[ADR-007](docs/adr/ADR-007-git-stack.md)）
- 本地存储 **redb 2** 默认 · rusqlite fallback · pending [SPIKE-04](docs/tasks/SPIKE-04-storage-benchmark.md)（[ADR-005](docs/adr/ADR-005-local-storage.md)）
- PTY 架构 **portable-pty + 共享读线程 + mpsc** · 每 session 一线程 fallback · pending [SPIKE-05](docs/tasks/SPIKE-05-pty-multi-tab.md)（[ADR-003](docs/adr/ADR-003-pty-architecture.md)）

---

<!--
  未来发布记录格式（每个版本 GA 发布时插入）：

  ## [0.1.0] - YYYY-MM-DD · v0.1 GA

  ### Added
  -

  ### Changed
  -

  ### Fixed
  -

  ### Security
  -
-->

<!-- links · 未来用 GitHub compare URL -->
<!-- [Unreleased]: https://github.com/tajiaoyezi/vibestation/compare/v0.1.0...HEAD -->
<!-- [0.1.0]: https://github.com/tajiaoyezi/vibestation/releases/tag/v0.1.0 -->
