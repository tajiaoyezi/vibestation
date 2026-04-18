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
- `docs/adr/` · 10 个 ADR（License / MVP 范围 / PTY / 前端栈 / 存储 / 桌面框架 / Git 栈 / Diff / AI-Aware / workspace）
- `CODE_OF_CONDUCT.md` · Contributor Covenant 2.1 中文版
- `CONTRIBUTING.md` · 贡献指南
- `CHANGELOG.md` · 本文件
- `docs/spikes/README.md` · Spike per-task 报告目录占位
- `docs/spike-artifacts/README.md` · Spike 录屏 / 截图目录占位
- `docs/session-history/README.md` · Session 历史目录占位

**Phase 4（2026-04-18）· GitHub 基础设施**：
- `.github/ISSUE_TEMPLATE/` · config / bug_report / feature_request / task_spec_proposal 四模板
- `.github/PULL_REQUEST_TEMPLATE.md` · PR schema（Implemented by / Reviewed by / 翻转 gate / 自审四问）
- `.github/dependabot.yml` · cargo + npm + github-actions 周更
- `.github/workflows/ci.yml` · CI skeleton（markdown-lint active · rust/frontend 占位）
- `.github/workflows/secret-scan.yml` · gitleaks 硬阻塞（落地 SPIKE-06 §A.5.3）
- `.github/workflows/task-spec-validator.yml` · frontmatter schema 校验（落地 README §原则 7）
- `scripts/validate-task-spec.mjs` · 224 行 · 无依赖 · Node 20+
- `docs/BRANCH-PROTECTION.md` · GitHub 分支保护 checklist（reviewer 应用指南）

### Changed · 决策锁定（A 栏）

- License = **Apache 2.0**（不签 CLA · [ADR-001](docs/adr/ADR-001-license-apache-2.0.md)）
- MVP v0.1 范围 = **B 折中方案**（[ADR-002](docs/adr/ADR-002-mvp-scope-b-compromise.md)）
- AI-Aware = **v1.0 vision · 对外不提**（[ADR-009](docs/adr/ADR-009-ai-aware-v1-vision.md)）
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
