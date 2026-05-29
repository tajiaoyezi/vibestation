# Task `3.1`: `external-term-windows`

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`（详见 `docs/s2v/standard.md` §10.5.1 状态机）。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 3 `terminal-integration`
**Dependencies**: 依赖 1.1（`crates/core/src/pty.rs` cfg 分离解锁 Windows 编译 + 提供 `#[cfg(target_os)]` 分支范式）

## 1. Background

`crates/core/src/external_term/` 三个文件全是 Unix 中心：

- `detect.rs`：`DetectionPlatform` 枚举只有 `Macos`/`Linux`，`current_detection_platform()` 在 Windows 返回 `None` → `detect_terminals_with_context` 直接返回空 `Vec` → 用户在 Windows 看不到任何外部终端；`TERMINALS` const 无 Windows 条目；`command_exists()` 写死 `which`（Windows 应用 `where.exe`）。
- `launch.rs`：`Platform` 枚举只有 `Macos`/`Linux`，`current_platform()` 把所有非 macOS 回落成 `Platform::Linux`，`build_launch_command` 各 terminal_id 臂都用 `open`（macOS）或 Linux 直跑二进制 → 在 Windows 拉起命令全错。
- `env_filter.rs`：`WHITELIST` 含 `SHELL`（Unix），无 `COMSPEC`/`PATHEXT`/`USERPROFILE` → Windows env preview 看不到关键变量。

这让 PRD §Core Capabilities #4「Windows UI 与外设集成」中的外部终端能力在 Windows 完全缺失。

## 2. Goal

Windows 上 `external_term` 子系统可用：`detect_terminals()` 返回至少 Windows Terminal / conhost / pwsh（已装时），`build_launch_command(..., Platform::Windows)` 为这些 terminal 返回正确的 `wt.exe` / `start cmd.exe` / `pwsh.exe` 启动命令，`filter_env` 在 Windows 显示 `COMSPEC`/`PATHEXT` 等。mac/Linux 行为零回归。

## 3. Scope

### In Scope

- `crates/core/src/external_term/detect.rs`：
  - `DetectionPlatform` 加 `Windows` 变体。
  - `current_detection_platform()` 加 `#[cfg(target_os = "windows")]` 分支返回 `Some(DetectionPlatform::Windows)`。
  - `TerminalDefinition` 加 `windows_priority: Option<u8>` + `windows_bins: &'static [&'static str]` 字段；`TERMINALS` const 加 Windows 条目（`windows-terminal` → `wt`；`pwsh` → `pwsh`；`conhost`/`cmd` 内置保底）。
  - `detect_terminals_with_context` 的 `match platform` 加 `DetectionPlatform::Windows` 臂（按 `windows_priority` + `windows_bins` 命中 `path_bins`）。
  - `detect_path_bins()` 扩展收集 `windows_bins`。
  - `command_exists()` 用 `#[cfg(windows)]` 走 `where`，`#[cfg(unix)]` 保持 `which`。
- `crates/core/src/external_term/launch.rs`：
  - `Platform` 加 `Windows` 变体。
  - `current_platform()` 加 `#[cfg(target_os = "windows")]` 分支返回 `Platform::Windows`。
  - `build_launch_command` 各 terminal_id 臂加 `Platform::Windows` 配方（`windows-terminal` → `wt.exe -d {cwd} ...`；`conhost`/`cmd` → `start` + `cmd.exe /D {cwd}`；`pwsh` → `pwsh.exe -NoExit -Command "Set-Location {cwd}"`）；macOS-only terminal（iterm2/terminal-app/ghostty mac 配方）在 Windows 返回 `UnsupportedCombination`。
- `crates/core/src/external_term/env_filter.rs`：`WHITELIST` 用 `#[cfg(windows)]` 追加 `COMSPEC`/`PATHEXT`/`USERPROFILE`/`HOMEDRIVE`/`HOMEPATH`（数据扩展，逻辑不变）。
- Windows 专属单元测试（detect context 注入 + launch 配方断言 + env_filter Windows whitelist）。

### Out Of Scope

- Windows 注册表查询检测 Windows Terminal（MVP 用 PATH `where` 探测即可；注册表探测推后，PRD §Out of Scope 精神）。
- 实际拉起外部终端进程的端到端自动化（`build_launch_command` 是纯函数只构造命令；真实 launch 在 §2.14 本机手动验）。
- ConEmu / Git Bash 等更多 Windows 终端（MVP 限 windows-terminal/conhost/pwsh）。
- `detect.rs` 的 `detect_xdg_default_terminal()`（已正确 Linux-gated，survey 标 already-ok，不动）。

## 4. Users / Actors

- **Windows 11 AI-agent 开发者**：右键 Pane → "在外部终端打开"，期望看到已装的 Windows Terminal / pwsh 并能拉起到当前 cwd。
- **`crates/app` Tauri command 层**：调用 `detect_terminals()` / `build_launch_command()`，期望 Windows 上返回非空列表与有效命令而非空 vec / Linux 回落。

## 5. Behavior Contract

### 5.1 Required Reading

- 上游 task spec：`docs/specs/tasks/task-1.1-pty-platform-split.md`（`#[cfg(target_os)]` 分支范式 + Windows 编译基线）。
- 同 phase 参考：`docs/specs/phases/phase-3-terminal-integration.md` §3 涉及模块。
- BDD：`test/features/external-term.feature`（Task 3.1 场景）。
- 相关 ADR：`docs/decisions/adr-001-pty-windows-cfg-separation.md`（cfg 分离范式，本 task 复用同模式，无新增 ADR 触发）。
- 现状源码：`crates/core/src/external_term/detect.rs`（`DetectionPlatform` / `TERMINALS` / `current_detection_platform` / `command_exists`）· `launch.rs`（`Platform` / `current_platform` / `build_launch_command`）· `env_filter.rs`（`WHITELIST`）。

### 5.2 Imports

- `std::process::Command`（已有，`command_exists` 用；Windows 走 `where`）。
- `std::env`（已有，detect context）。
- `std::path::Path`（已有，`build_launch_command` 入参）。
- `serde` / `ts_rs::TS`（已有，`ExternalTerminalInfo` / `LaunchCommand` 导出）。
- 无新增第三方依赖（纯 cfg 分支 + 数据扩展）。

### 5.3 函数签名

Windows 适配后的真实签名骨架（走 cfg 分支，公开签名不变 — 仅枚举增变体 + 内部 match 增臂）：

```rust
// detect.rs — 枚举增 Windows 变体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DetectionPlatform {
    Macos,
    Linux,
    Windows, // 新增
}

// TerminalDefinition 增 Windows 字段
struct TerminalDefinition {
    id: &'static str,
    display_name: &'static str,
    mac_priority: Option<u8>,
    linux_priority: Option<u8>,
    windows_priority: Option<u8>,        // 新增
    mac_app_paths: &'static [&'static str],
    mac_bins: &'static [&'static str],
    linux_bins: &'static [&'static str],
    windows_bins: &'static [&'static str], // 新增（如 ["wt"], ["pwsh"]）
    linux_desktop_markers: &'static [&'static str],
    term_program_markers: &'static [&'static str],
}

// 平台探测增 Windows 分支
fn current_detection_platform() -> Option<DetectionPlatform> {
    #[cfg(target_os = "macos")]   { Some(DetectionPlatform::Macos) }
    #[cfg(target_os = "linux")]   { Some(DetectionPlatform::Linux) }
    #[cfg(target_os = "windows")] { Some(DetectionPlatform::Windows) }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    { None }
}

// command_exists 平台分支
fn command_exists(bin: &str) -> bool {
    #[cfg(windows)]
    let probe = "where";
    #[cfg(not(windows))]
    let probe = "which";
    std::process::Command::new(probe)
        .arg(bin)
        .status()
        .is_ok_and(|status| status.success())
}

// launch.rs — Platform 增 Windows 变体
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Macos,
    Linux,
    Windows, // 新增
}

pub fn current_platform() -> Platform {
    #[cfg(target_os = "macos")]   { Platform::Macos }
    #[cfg(target_os = "windows")] { Platform::Windows }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))] { Platform::Linux }
}

// 签名不变 · match 内增 Platform::Windows 臂
pub fn build_launch_command(
    terminal_id: &str,
    cwd: &Path,
    shell: &str,
    platform: Platform,
) -> Result<LaunchCommand, LaunchError>;
// Windows 配方示例（windows-terminal）：
//   LaunchCommand { program: "wt.exe",
//     args: vec!["-d".into(), cwd, "new-tab".into(), shell.into()] }
// conhost/cmd: program "cmd.exe", args ["/D".into(), format!("/K"), ...] 经 start 包裹
// pwsh: program "pwsh.exe", args ["-NoExit","-Command", format!("Set-Location '{cwd}'")]

// env_filter.rs — WHITELIST cfg 扩展（数据，不改 filter_env 签名）
#[cfg(not(windows))]
pub const WHITELIST: &[&str] = &["PATH", "HOME", "LANG", "TERM", "SHELL", "USER"];
#[cfg(windows)]
pub const WHITELIST: &[&str] =
    &["PATH", "LANG", "TERM", "USER", "COMSPEC", "PATHEXT", "USERPROFILE", "HOMEDRIVE", "HOMEPATH"];
```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities #4 · §Success Metrics「外部终端列表非空」): Windows 上 `detect_terminals_with_context` 给定含 `wt`/`pwsh` 的注入 context 时返回非空列表，且条目 `id` 含 `windows-terminal` / `pwsh`，`detected=true`；未装时该条目不出现（不再因平台 `None` 直接返回空 vec）。
- [ ] **AC2** (PRD §Core Capabilities #4): `build_launch_command("windows-terminal", cwd, "pwsh.exe", Platform::Windows)` 返回 `Ok(LaunchCommand)` 且 `program` 含 `wt.exe`、`args` 含 cwd；`build_launch_command("pwsh", cwd, "pwsh.exe", Platform::Windows)` 返回 `program == "pwsh.exe"`；macOS-only terminal（如 `iterm2`）在 `Platform::Windows` 返回 `Err(UnsupportedCombination)`。
- [ ] **AC3** (PRD §Constraints 兼容性 · §Success Metrics 反指标): mac/Linux 上 `detect_terminals` / `build_launch_command` / `filter_env` 行为零回归 — 现有 macOS/Linux 单元测试全绿（cfg 分支不影响 Unix 路径）。
- [ ] **AC4** (本 task 新增): `filter_env` 在 Windows 上对含 `COMSPEC`/`PATHEXT` 的 env map 把这两个键纳入 `visible_entries`（`#[cfg(windows)]` WHITELIST 生效）；Unix 上 `SHELL` 仍在 whitelist。
- [ ] **AC5** (本 task 新增 · 来自 survey `command_exists` 缺口): `command_exists` 在 Windows 走 `where`、在 Unix 走 `which`（编译期 cfg 选择，Windows 不再调用不存在的 `which`）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 外部终端列表非空 | SCEN-3.1.1 | TEST-3.1.1 `test_3_1_1_windows_detect_returns_wt_pwsh` | N/A（注入 context 单测覆盖） | cargo test -p vibestation_core external_term::detect | Not Started |
| AC2 Windows 启动配方 | SCEN-3.1.2 | TEST-3.1.2 `test_3_1_2_windows_launch_recipes` | N/A | cargo test -p vibestation_core external_term::launch | Not Started |
| AC3 mac/Linux 零回归 | SCEN-3.1.3 | TEST-3.1.3 `test_3_1_3_unix_detect_launch_unchanged`（现有用例集） | N/A | cargo test --workspace（mac/Linux CI） | Not Started |
| AC4 Windows env whitelist | SCEN-3.1.4 | TEST-3.1.4 `test_3_1_4_windows_env_whitelist_comspec` | N/A | cargo test -p vibestation_core external_term::env_filter | Not Started |
| AC5 command_exists where/which | SCEN-3.1.5 | TEST-3.1.5 `test_3_1_5_command_exists_platform_probe` | N/A | cargo test -p vibestation_core external_term::detect | Not Started |

## 8. Risks

- **R1（PRD §Technical Risks R3 · mac/Linux 回归）**：给 `TerminalDefinition` 加字段后 `TERMINALS` const 现有条目需逐个补 `windows_priority: None` / `windows_bins: &[]`，漏改会编译失败（结构体字段非可选）—— 编译期即可捕获，TDD GREEN 前 `cargo build` 必过。
- **R2（PRD §Technical Risks R5 · CI headless）**：`command_exists` 的 `where`/`which` 子进程在 CI 不可控；单元测试用注入 `TerminalDetectionContext`（`path_bins` 预填）绕开真实探测，AC5 仅断言 probe 名选择正确不真跑。
- **R3**：`build_launch_command` Windows 配方的 cwd 含空格 / 反斜杠 / 中文（PRD §Technical Risks R4）需正确转义；Windows 用例 fixture 含特殊路径走文件而非 inline（adapter §Fixture 约定）。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile  <!-- 与 adapter §Commands 一致 -->
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制；scoped 可跑 cargo test -p vibestation_core external_term:: -->
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

> Integration / E2E / Coverage / Runtime-smoke：本 task 为纯逻辑后端，集成随 `cargo test --workspace` 一起跑（crates/core 内嵌 mod tests），无独立 e2e；MVP 不强制覆盖率；外部终端真实拉起在 Phase 3 §6 端到端 smoke / §2.14 本机手动验，不列入本 task §9。

## 10. Completion Notes

<TBD-after-impl>
