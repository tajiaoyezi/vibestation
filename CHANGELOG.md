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

### Added · 代码实施（2026-04-19 ~ 2026-04-21 · session 7-14 · macOS-first）

**Spike W0 · macOS 100% 完结**（session 7）：
- SPIKE-01 Tauri 三平台启动验证 · macOS Phase A PASS · 冷启动 202ms median（PR #20 · [report](docs/spikes/SPIKE-01-report.md)）
- SPIKE-02 Tauri 硬通过矩阵 · macOS Phase A PASS · bundle 10MB / .dmg 4MB（PR #22）
- SPIKE-03 git2 vs gix benchmark · gix log -100 warm P99 12.65ms 比 git2 快 1973×（PR #23 · [ADR-007](docs/adr/ADR-007-git-stack.md) accepted）
- SPIKE-04 + SPIKE-04.5 storage benchmark · rusqlite B.1-5 全过 · redb 2.6.3 B.2 silent corruption FAIL（PR #24/#29/#34/#68 · [ADR-005](docs/adr/ADR-005-local-storage.md) accepted）
- SPIKE-05 + SPIKE-05.5 portable-pty 多 Tab 压测 · shared-reader HOL/boundedness pass · visible throughput 瓶颈在 JS/invoke RTT（PR #30/#39 · [ADR-003](docs/adr/ADR-003-pty-architecture.md) accepted）
- SPIKE-06 §A Claude/Codex CLI 36 脱敏样本 · harness + record.sh + redact.py + gitleaks 0 hit（PR #38/#71 · [SPIKE-06-report](docs/spikes/SPIKE-06-report.md)）
- SPIKE-08 E2E + IPC contract harness · ts-rs 选定 + Playwright 补层（PR #60 · [ADR-014](docs/adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md) accepted）
- ADR accepted 14 个（#001-014）· 14 ADR proposed → accepted 收敛

**MVP 实施**（session 8-14 · macOS-first）：
- **MVP-01 Phase A + B · Tauri 壳 + SolidJS + Calm Studio** · Cargo workspace 2-crate + runtime 验证 + 3 轮 CI 修（PR #28/#33）
- **MVP-02 · workspace 管理 done** · rusqlite + r2d2 pool + WorkspaceStore CRUD + git 自动检测 5 parent + UUID v4 + canonical path · 23 unit tests · H1 path traversal + H2 IPC camelCase 修（PR #40/#44/#45/#47）
- **MVP-03 · Tool Windows 5-zone 布局 done** · Primary/Secondary Sidebar + Activity Strip + Bottom Panel · theme light/dark · 布局持久化到 rusqlite · 29 unit tests · 5 runtime 截图（PR #61）
- **MVP-04 Phase A · tabs 存储层 done** · migration v5 + TabsDao 6 CRUD + 2 scrollback methods + 5 IPC commands + Tauri ACL allow-tab-* + ts-rs 5 bindings + 36 unit tests（PR #72）
- **MVP-04 Phase B · PTY runtime done** · `portable-pty` + `mio` poll + `DropOldestSender` bounded(128) drop-oldest + `crossbeam-channel` · 5 tab_pty_* IPC commands + 5 allow-tab-pty-* permissions + 3 ts-rs bindings（PtyStdoutEvent/PtyExitedEvent/PtySpawnRequest）· `fix_path_env.rs` 53 行本地 shim（crates.io 包不可解析 · 技术债）· `tab_pty_stdout` / `tab_pty_exited` Tauri events · 19 PTY 单元/集成测试 · **Phase C-F xterm 前端 / shell 兼容 / 持久化 / 证据 待**（PR #82）
- **MVP-06 Phase A + A+ · parser 层完整** · `crates/core/src/config_import/` Ghostty TOML + iTerm2 plist（binary/text · Default Bookmark Guid → default profile）+ Alacritty TOML/YAML 双格式 · `ImportedField` 6 variants（FontFamily/FontSize/Theme/Shell/KeyBinding/AnsiColor）· Ghostty `keybind` 重复行逐行扫 filter 降级 · iTerm2 ANSI 0-15 RGB→hex 转换 · Alacritty `[[keyboard.bindings]]` TOML 0.14+ + `key_bindings:` YAML 0.13- · 26 unit tests · **Phase B IPC/UI/apply 待 MVP-04 Phase C-F done 后**（PR #80/#81）
- **MVP-07 · Git Log 只读 done** · `gix 0.70` 分页 revwalk + `GitLogReader::query` + commit detail + branch/tag labels + 筛选（message/author/after） · 3 IPC commands + 3 allow-git-log-* permissions + 7 ts-rs bindings + SolidJS `web/src/panels/GitLog/` 前端 panel（list + detail + filter · Secondary Sidebar 接入）· H2 regression proof 制度化 · 92 workspace tests · **UI 截图 + linux kernel 10 万 commit benchmark GA gate 补**（PR #83）

**Kimi 协作成就**（远程 API agent · 8 次协作 · 100% 成功率）：
- 6 次 spec review：MVP-04/05/06/07/08/09 · 平均 23 min（PR #64/#66/#70/#73/#74/#77）
- 2 次代码实施（首次）：MVP-06 Phase A + A+ parser 模块（PR #80/#81）· 主动优化降级方案（比 dispatch prompt 建议更优）

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
