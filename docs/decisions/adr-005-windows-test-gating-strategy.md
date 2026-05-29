# ADR `005`: Windows 测试门控策略（cfg + ignore + 专测）

**Status**: Accepted
**Date**: 2026-05-29
**Category**: 测试工具链
**Related**: PRD §Decisions Log D5

## Context

Vibestation 的测试套件含大量 Unix 硬假设，Windows 上要么编译失败、要么 panic（survey "PTY/Core Shell Compatibility Tests" 子系统逐条确认）：

- `crates/app/tests/shell_compat.rs`：用 `which` + Unix 绝对路径候选（`/bin/zsh` 等）定位 shell；shell 用例（zsh_01–04 / bash_05–08 / fish_09–12）Windows 上直接 panic（"zsh not found"）或 PTY spawn 失败。当前只有 `#[cfg_attr(target_os = "linux", ignore)]`（epoll timing 原因），无 Windows 标记。
- `crates/core/tests/git_ops_integration.rs`：`create_pre_commit_hook()` 用 `std::os::unix::fs::PermissionsExt::set_mode(0o755)` + `#!/bin/sh` 脚本 —— Windows 无 mode bits，且无法执行 shell 脚本 hook。
- `crates/core/tests/pty_pool_bench.rs`：硬编码 `/tmp`、回落 `HOME→/`、`SHELL→/bin/sh`。
- `crates/core/tests/pty_scrollback_integration.rs`：硬编码 `/bin/sh`，Windows PTY spawn 失败。

部分 Unix-only 行为在 Windows 上**根本无意义**（`/etc/shells`、`chmod 0o755`、`#!/bin/sh` hook），不应强制双平台。同时 PRD 反指标硬约束："不能为 Windows 适配牺牲 mac/Linux 现有行为" —— Unix 真测必须保留，Windows 也需真覆盖。本 ADR 定测试门控策略。

## Decision

采用 **"cfg 分离 + Windows ignore + 补 Windows 专测"** 三件套，CI windows-latest 跑 `cargo test --workspace` 自动跳过 Unix-only 项：

- **Unix-only 行为测试**：用 `#[cfg(unix)]` 条件编译（整个测试/模块），或 `#[cfg_attr(windows, ignore = "<根因>")]` 标注。典型：`/etc/shells` 枚举、`chmod 0o755`、`#!/bin/sh` hook、tcgetpgrp/libc 信号相关测试。
- **timing 敏感的跨平台测试**：保留现有 `#[cfg_attr(target_os = "linux", ignore)]`（epoll 原因），并按需叠加 Windows ignore（PTY 暂未稳定时）；timeout 须 ≥ 本地最大运行时长 ×2 或平台 gate（关联 dispatch §2.11）。
- **Windows 专属测试**：为 Windows 特有行为补真覆盖 —— 默认 shell 探测链（`pwsh→powershell→cmd`）、`home_dir()` 解析 `USERPROFILE`、`where.exe` 命令存在性、Windows 路径 round-trip（dunce）、ConPTY spawn/exit/echo、`%APPDATA%` 配置路径。
- **CI**：windows-latest 跑 `cargo test --workspace`（= adapter Unit Test 命令），`#[cfg]` / `ignore` 标记使 Unix-only 项自动 skip 而非 panic；mac/Linux job 全量 `cargo test` 必须仍 100% 绿（反指标硬约束）。

跨平台路径用 `std::env::temp_dir()` 替代 `/tmp`、`home_dir()`（ADR-002）替代 `HOME→/`。

## Rationale

- **既保留 Unix 真测，又给 Windows 真覆盖**：cfg 分离让 Unix-only 行为继续真测（不被 Windows 拖累降级），Windows 专测覆盖平台特有路径 —— 两端都不是"假绿"。
- **不现实强制双平台**：`/etc/shells`、`chmod 0o755`、`#!/bin/sh` hook 在 Windows 无意义，强制双平台（候选 a）会逼着写无意义的 Windows 桩。
- **不放过 Windows 行为回归**：Windows 只 build 不 test（候选 c）会让 Windows 行为回归无人把关 —— 与 PRD 反指标"不能为编译过而阉割功能"冲突。

## Alternatives

- **(a) 全部测试强制双平台**：拒绝 —— 不现实（`/etc/shells` / `chmod 0o755` / `#!/bin/sh` hook 在 Windows 无意义），会产生大量无意义 Windows 桩。
- **(c) Windows 只 build 不 test**：拒绝 —— 放过 Windows 行为回归，违反反指标硬约束。
- **(b) cfg 分离 + Windows ignore + 补 Windows 专测**（**选定**）：既保留 Unix 真测又给 Windows 真覆盖。

## Consequences

**正面**：

- windows-latest 上 `cargo test --workspace` 显示 Unix-only 项 `ignored` 而非 `panicked`，CI 可绿（PRD Success Metric "三平台 CI 全绿"）。
- Windows 专测锁住平台特有行为（shell 探测链 / home_dir / 路径 round-trip / ConPTY），防回归。
- mac/Linux 全量 `cargo test` 行为零变化（Unix 测试走 `#[cfg(unix)]` 真测，反指标守住）。
- 符合 S2V TDD：Windows 专测可先 RED（survey verify 字段已给每条预期），驱动实现。

**负面 / 风险**：

- ignore 标记需配套技术债登记 + GA gate 解除条件（避免 ignore 永久化）—— 每条 Windows ignore 在对应 task spec §Risks / Master/Phase spec §已知风险 记一条 + 明确解除触发条件。
- CI 无法做 GUI runtime 验证（PRD R5，概率低 / 影响中）：windows-latest headless 跑不了完整 GUI smoke。缓解：CI 限 build + 单元/集成测试 + bundle 产物校验；GUI critical UX path 靠开发者本机（H:\ Windows 11）按 §2.14 实跑并记录证据。
- ConPTY timing 与 Linux epoll / macOS kqueue 语义差异（关联 ADR-001 R1）：纯 timeout 扩张不解决语义问题，必要时切平台 ignore + 技术债（dispatch §2.11）。

## Rollback Or Migration Plan

- **回滚**：测试门控是纯测试层改动，不影响生产代码 —— 移除 Windows ignore / `#[cfg(windows)]` 专测即退回当前状态；若某 Windows 专测不稳定，可临时标 `#[ignore]` + 技术债，不阻塞主线。
- **迁移**：无数据迁移。ignore 的 Unix-only 测试在 Unix 上仍真跑；Windows 专测随 PTY/shell 实现成熟逐步从 ignore 解除（GA gate）。

## Follow-ups

- task-6.1（windows-test-gating）统一落地 cfg/ignore 标记 + Windows 专测集（shell_compat / git_ops_integration / pty_*_integration / env_filter）。
- task-6.2（windows-smoke-matrix）落地三平台测试矩阵 + 本机 GUI smoke 证据（PRD R5 缓解）。
- 各 task spec §Risks 登记其 Windows ignore 的根因 + GA gate 解除触发条件。
- 关联 ADR-001（ConPTY timing → 信号/退出测试门控）/ ADR-003（shell 探测链 → Windows 专测）/ dispatch §2.11（跨平台 timeout 规则）。
