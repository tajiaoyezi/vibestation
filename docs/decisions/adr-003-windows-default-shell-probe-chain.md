# ADR `003`: Windows 默认 shell 探测链 `pwsh.exe → powershell.exe → cmd.exe`

**Status**: Accepted
**Date**: 2026-05-29
**Category**: 兼容性
**Related**: PRD §Decisions Log D3

## Context

`crates/core/src/pty.rs` 的 shell 解析全部基于 Unix 假设：

- `default_shell_path()`（约行 1019–1025）只区分 macOS（`/bin/zsh`）/ else（`/bin/bash`），无 Windows 分支 —— Windows 上返回 `/bin/bash`（不存在），新建 Tab 必失败。
- `list_available_shells()`（约行 1108–1153）读 `/etc/shells` 并按 `PRIMARY_SHELL_BASENAMES`（zsh/bash/fish）过滤 —— Windows 无 `/etc/shells`，返回空或错误回落。
- `find_available_shell()` / `resolve_shell()` 依赖 `which` Unix 工具做 PATH 探测 —— Windows 上 `which` 通常不存在（应为 `where.exe` 或 PATH 遍历）。
- 可执行位检测 `is_executable_file()` 用 `metadata.permissions().mode() & 0o111`（`std::os::unix::fs::PermissionsExt`）—— Windows 无 mode bits（`.exe`/`.bat` 按扩展名/注册表可执行）。

Windows 必须有自己的"默认 shell 选哪个 + 怎么探测可用 shell"的策略，且**绝不能因为找不到首选 shell 而崩溃或拉起不存在的 `/bin/bash`**（PRD User Flow 异常流硬约束）。本 ADR 定该策略；具体 cfg 实现细节由 task-2.1 落地。

## Decision

Windows 默认 shell 采用**探测链 `pwsh.exe → powershell.exe → cmd.exe`，取第一个可用者**：

- `default_shell_path()` 加 Windows 分支：按 `pwsh.exe`（PowerShell 7+）→ `powershell.exe`（Windows 内置 5.1）→ `cmd.exe`（永远保底）顺序探测，返回第一个在 PATH 中可用的。
- `list_available_shells()` 的 Windows 路径：走 PATH / `where.exe` 探测 `pwsh.exe`/`powershell.exe`/`cmd.exe`（不读 `/etc/shells`）。
- shell 存在性 / 可执行性检测：Windows 用 `metadata.is_file()`（不查 mode bits），命令存在性用 `where`（Unix 保留 `which`）。
- `cmd.exe` 作为最终保底永远成立（系统必装），保证探测链不会落空。

所有 Windows 分支走 `#[cfg(target_os = "windows")]`，Unix 的 `/etc/shells` + `which` + mode-bits 路径不变。拉起的 shell 仍过现有 `sanitize::is_denied_windows_path`（adapter §Security 已存在 `#[cfg(windows)]`）。

## Rationale

- **"有啥用啥" + 永远保底**：探测链让现代环境优先用 `pwsh`，未装时优雅降级到内置 `powershell`，最终 `cmd.exe` 系统必装保底 —— 不会出现"拉起不存在的 shell"的崩溃路径。
- **现代体验优先**：固定 `cmd.exe`（候选 a）太基础；固定 5.1 `powershell.exe`（候选 b）不如 `pwsh` 现代 —— 探测链兼顾两者。
- **对齐 Unix 设计直觉**：Unix 侧本就是"探测可用 shell + 回落默认"的设计，Windows 探测链是同一思路的平台映射。

## Alternatives

- **(a) 固定 `cmd.exe`**：拒绝 —— 体验过于基础，无法满足跑现代 AI agent / PowerShell 脚本的主用户。
- **(b) 固定 `powershell.exe`（5.1）**：拒绝 —— 纯内置 5.1 不如 `pwsh` 现代，且未必所有环境一致。
- **(c) `pwsh → powershell → cmd` 探测链**（**选定**）：有啥用啥，`cmd.exe` 永远保底，绝不拉起不存在的 shell。

## Consequences

**正面**：

- Windows 新建 Tab 能在 `pwsh`（或回落 `cmd.exe`）看到 prompt 并正常读写（PRD Core Capability 2 + Success Metric "PTY runtime smoke"）。
- 未装 PowerShell 7 时优雅降级，不崩溃（PRD 异常流 "未装 PowerShell 7" 缓解）。
- mac/Linux 的 `/etc/shells` + `which` + mode-bits 解析路径零变化（兼容性硬约束）。

**负面 / 风险**：

- 探测链依赖 PATH / `where.exe` 正确解析 —— 与 ADR-002 家目录解析、`fix_path_env` 的 Windows no-op 行为相关（GUI 启动的 PATH 继承在 Windows 与 Unix 不同，survey 标为 medium，本机 smoke 验证）。
- shell 探测属 PTY 子系统，依赖 ADR-001 的 PTY 编译解锁先行（task-2.1 依赖 task-1.1）。
- 注册表 / `App Paths` 探测的复杂度：MVP 优先 PATH + `where.exe` + 已知保底，注册表精确探测按需，避免过度工程。

## Rollback Or Migration Plan

- **回滚**：本决策不改 DB schema / IPC 契约 / 持久化的 shell 设置格式（设置仍存 shell 可执行路径字符串）。若探测链行为有问题，可临时把 Windows `default_shell_path()` 退回固定 `cmd.exe`（最保守、必成立），不影响 Unix。
- **迁移**：无数据迁移。用户若已在设置里手选 shell（ADR-未涉及的 app_settings），其存储值仍是路径字符串，跨平台格式一致；探测链只影响"未显式设置时的默认解析"。

## Follow-ups

- task-2.1（windows-shell-detection）落地 `default_shell_path()` / `list_available_shells()` / `resolve_shell()` 的 Windows 探测链 + `where.exe` + `is_file()` 检测。
- task-1.3（shell-default-setting）落地 `app_settings.rs` 的 Windows 默认 shell 分支（与本探测链一致）。
- 关联 ADR-001（PTY 编译解锁是前置）/ ADR-002（PATH/家目录解析）/ ADR-005（Unix-only shell 行为测试门控）。
