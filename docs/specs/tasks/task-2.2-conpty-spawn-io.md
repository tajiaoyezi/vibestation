# Task `2.2`: `conpty-spawn-io`

> Task Spec · 按 S2V standard §8.3 模板渲染。无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研证据（`spike-tmp/win-survey.json`）+ 实际 `crates/core/src/pty.rs` 源码填实，非编造。

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`（standard §10.5.1）。

**Priority**: P0
**Owner**: 主 agent
**Related Phase**: Phase 2 `shell-runtime`（[`../phases/phase-2-shell-runtime.md`](../phases/phase-2-shell-runtime.md)）
**Dependencies**: 依赖 2.1（Windows shell 探测链就位 · `resolve_shell` / `default_shell_path` 能在 Windows 解析出可拉起的 shell）；间接依赖 1.1（ConPTY reader 路径骨架已 `#[cfg]` 隔离）

## 1. Background

Phase 1 在 `pty.rs` 把 Unix-only 的 reader loop（`mio::unix::SourceFd` + `libc::read`）与 ConPTY 阻塞读路径做了编译期 `#[cfg(target_os)]` 分离，task 2.1 让 Windows 能解析出可拉起的 shell。但"ConPTY 真能 spawn 一个 shell、读到它的输出、检测到它退出、能向它发终止信号"这一行为在 Windows 上**尚无任何集成测试验证**，且 signal 路径仍是 Unix-only：`PtySession::signal()` 经 `parse_signal()`（`libc::SIGINT` 等常量）+ `libc::kill()` + `signal_target()`（`libc::tcgetpgrp`）实现，Windows 上无这些 POSIX 概念。

PRD §Technical Risks R1 把 ConPTY reader 退出检测 / 信号语义差异列为「高概率 × 高影响」首要风险——可能 hang、漏 exit、读不到尾部输出。本 task 用"先写 ConPTY spawn/exit/echo 集成测试（TDD RED 先行）再补必要的 Windows reader / signal 收尾"的方式收口这一风险，是 Phase 2 的最后一块。

## 2. Goal

任务完成后应该成立的事实：

- Windows 上 ConPTY 能 spawn `cmd.exe` / `pwsh`，向 stdin 写命令后能从 `PtyEvent::Stdout` 读到 prompt / echo 输出。
- shell 进程退出后，reader 路径经 `child.try_wait()` 在有界时间内检测到退出并 emit `PtyEvent::Exited`（不 hang、不漏 exit）。
- Windows 上 `PtySession::signal()` 把 `SIGINT` / `SIGTERM` / `SIGKILL` 映射为 `child.kill()`（底层 `TerminateProcess`）的 graceful / 强制终止——无对应信号概念时走 graceful kill，且 `terminate()` 仍能可靠收尾。
- `crates/core/tests/` 新增 Windows-gated（`#[cfg(windows)]`）集成测试覆盖 spawn / echo / exit / signal-terminate。
- macOS / Linux 上 reader（mio）/ signal（`libc::kill` + `tcgetpgrp` 前台进程组）/ 退出检测行为**零回归**。

## 3. Scope

### In Scope

- `crates/core/src/pty.rs`：
  - `PtySession::signal()`：加 `#[cfg(windows)]` 分支——映射 `SIGKILL` → `child.kill()`（强制），`SIGINT` / `SIGTERM`（及未知）→ graceful kill（无 ConSole ctrl event 概念时统一走 `child.kill()`）；保留 `#[cfg(unix)]` 的 `parse_signal` + `libc::kill` 路径。
  - `signal_target()`：`#[cfg(windows)]` 直接返回 `Some(SignalTarget::Process(pid))`（ConPTY 无前台进程组 / `tcgetpgrp` 概念），`#[cfg(unix)]` 保留 `tcgetpgrp` + leader 逻辑。
  - `SignalTarget` 枚举：Windows 上 `ProcessGroup` 变体不再被构造（仅 `Process`），按需 `#[cfg]` 或 `#[allow(dead_code)]` 收敛 clippy。
  - 必要的 Windows reader 收尾：确保 ConPTY 阻塞读循环 + `child.try_wait()` 退出检测路径完整（Phase 1 骨架的补全，非重写）。
- `crates/core/tests/`：新增 `pty_windows_conpty_integration.rs`（`#[cfg(windows)]` 全门控）——spawn cmd.exe/pwsh、读 prompt/echo、检测退出、signal-terminate 集成测试。

### Out Of Scope

- Windows 精确 Ctrl-C 投递（`GenerateConsoleCtrlEvent`，MVP 用 graceful `child.kill()` 兜底；精确 SIGINT 语义留后续，PRD §Technical Risks R1 缓解策略已注明）。
- `detect_process_cwd` Windows 精确实现（task 2.1 已用缓存兜底）。
- shell 探测链 / 枚举（task 2.1）。
- Unix epoll/kqueue reader 行为变更（仅保不回归，不优化）。
- 前端 Tab 创建 UX（Phase 4 / runtime smoke）。

## 4. Users / Actors

- **Windows 11 上的 AI-agent 开发者**：新建 Tab 后经 ConPTY 与 pwsh/cmd 交互（输入命令、看回显、退出 / 中断 agent 进程）——本 task 的 spawn/read/exit/signal 直接决定该体验。
- **`crates/app` Tauri 启动层**：经 `PtyManager::spawn` / `terminate` / signal IPC 命令驱动 ConPTY 会话，须在 Windows 上不 hang。
- **windows-latest CI**：跑 `cargo test --workspace`，本 task 的 `#[cfg(windows)]` 集成测试在此执行（无 GUI，纯进程级）。

## 5. Behavior Contract

### 5.1 Required Reading

- 上游 task spec：[`task-2.1-windows-shell-detection.md`](./task-2.1-windows-shell-detection.md)（Windows shell 解析前置 · spawn 需先能 `resolve_shell`）、[`task-1.1-pty-platform-split.md`](./task-1.1-pty-platform-split.md)（reader/signal/fcntl 的 `#[cfg]` 分离骨架）。
- 关联 ADR：[adr-001-pty-windows-cfg-separation.md](../../decisions/adr-001-pty-windows-cfg-separation.md)（PTY cfg 分离 + portable-pty ConPTY + `child.try_wait` 退出检测）、[adr-005-windows-test-gating-strategy.md](../../decisions/adr-005-windows-test-gating-strategy.md)（Windows 专测 + Unix ignore 门控）。
- BDD feature：[`test/features/pty.feature`](../../../test/features/pty.feature)（task 1.1 / 2.1 / 2.2 共用，ConPTY spawn/echo/exit 场景）。

### 5.2 Imports

真实 imports（Windows 分支以 `#[cfg(windows)]` 门控；Unix 分支保留 `libc`）：

```rust
// 现有（保留 · Unix signal 路径）
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};
use portable_pty::{Child, ExitStatus, PtySize};   // Child::kill() / try_wait() 跨平台

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;          // status.signal()
// libc::{SIGINT, SIGTERM, SIGTSTP, SIGKILL, kill, tcgetpgrp} 仅 #[cfg(unix)] 路径引用

// 集成测试 crates/core/tests/pty_windows_conpty_integration.rs（全文件 #[cfg(windows)]）
#![cfg(windows)]
use vibestation_core::pty::{PtyManager, PtySpawnRequest, PtyEvent};
use std::time::Duration;
```

> 不引入新 crate。`child.kill()` / `child.try_wait()` 是 `portable_pty::Child` 跨平台方法（底层 Windows = `TerminateProcess`），已在现有 `terminate()` / `try_wait()` 中使用。

### 5.3 函数签名

据现有 `pty.rs`（`signal` line 298-317 / `signal_target` line 455-468 / `SignalTarget` line 112-114 / `terminate` line 319-330）给出 Windows 适配后的真实签名骨架（公开行为契约不变，内部 `#[cfg]` 分支）：

```rust
enum SignalTarget {
    #[cfg(unix)]
    ProcessGroup(libc::pid_t),   // Windows 无前台进程组概念 → 仅 #[cfg(unix)]
    Process(u32),                // 跨平台 · 持进程 pid（Windows / Unix 通用）
}

impl PtySession {
    // signal：Windows 映射为 child.kill()（graceful/强制统一），Unix 保留 libc::kill
    fn signal(&self, signal: &str) -> Result<(), PtyError> {
        #[cfg(windows)]
        {
            // ConPTY 无 POSIX 信号；SIGKILL/SIGTERM/SIGINT/未知 一律走 child.kill()
            // （底层 TerminateProcess · graceful 概念在 Windows 退化为直接终止）
            let _ = signal;
            let mut child = lock(&self.child);
            child.kill().map_err(PtyError::Io)
        }
        #[cfg(unix)]
        {
            let signal_number = parse_signal(signal)?;          // libc::SIG* · 仅 unix
            let pid = match self.signal_target() {
                Some(SignalTarget::ProcessGroup(group)) => -group,
                Some(SignalTarget::Process(pid)) => pid as libc::pid_t,
                None => return Err(PtyError::OpenFailed(format!(
                    "cannot resolve process target for {signal}"))),
            };
            let result = unsafe { libc::kill(pid, signal_number) };
            if result != 0 { return Err(PtyError::Io(io::Error::last_os_error())); }
            Ok(())
        }
    }

    // signal_target：Windows 直接返回 Process(pid)（无 tcgetpgrp），Unix 保留前台进程组探测
    fn signal_target(&self) -> Option<SignalTarget> {
        #[cfg(windows)]
        {
            self.process_id.map(SignalTarget::Process)
        }
        #[cfg(unix)]
        {
            let foreground = unsafe { libc::tcgetpgrp(self.fd) };
            if foreground > 0 { return Some(SignalTarget::ProcessGroup(foreground)); }
            let leader = lock(&self.master).process_group_leader();
            if let Some(group) = leader.filter(|g| *g > 0) {
                return Some(SignalTarget::ProcessGroup(group));
            }
            self.process_id.map(|pid| SignalTarget::Process(pid as libc::pid_t))
        }
    }

    // try_wait / terminate / wait_for_exit：已跨平台（portable_pty::Child）· 行为不变 · Windows 复用
    fn try_wait(&self) -> Result<Option<ExitStatus>, PtyError>;          // 现状即跨平台
    fn terminate(&self, events: &DropOldestSender<PtyEvent>) -> Result<Option<i32>, PtyError>;
    fn wait_for_exit(&self, timeout: Duration) -> Result<Option<ExitStatus>, PtyError>;

    // parse_signal 仅 #[cfg(unix)]（Windows 不调用）
}

#[cfg(unix)]
fn parse_signal(signal: &str) -> Result<i32, PtyError>;   // 现有 · 加 cfg 门控

// 集成测试骨架（crates/core/tests/pty_windows_conpty_integration.rs）
#![cfg(windows)]
// SCEN-2.2.1 / AC1
#[test] fn test_2_2_1_conpty_spawn_cmd_reads_prompt();
// SCEN-2.2.2 / AC2
#[test] fn test_2_2_2_conpty_echo_roundtrip();
// SCEN-2.2.3 / AC3
#[test] fn test_2_2_3_conpty_detects_process_exit_no_hang();
// SCEN-2.2.4 / AC4
#[test] fn test_2_2_4_signal_terminate_kills_conpty_child();
```

> `SignalTarget::Process` 的载荷类型在 Unix 当前是 `libc::pid_t`（=`i32`），Windows 无 `libc::pid_t`；本 task 统一为 `u32`（与 `PtySession.process_id: Option<u32>` 一致），Unix 分支构造处 `pid as libc::pid_t` 转回。这是本 task 唯一的类型形变，改动局部。

## 6. Acceptance Criteria

<!-- 模式 A：完整给值 + PRD 引用。review 通过无需删本注释。 -->

- [ ] **AC1** (PRD §Core Capabilities 1/2 · §Success Metrics 次要指标「PTY runtime smoke」): Windows 上经 ConPTY spawn `cmd.exe`（或 pwsh）后，能在有界时间内从 `PtyEvent::Stdout` 读到非空 prompt 输出（不 hang、不返回空）。
- [ ] **AC2** (PRD §Core Capabilities 2 · §User Flow 主流程 3): Windows 上向 ConPTY 会话 stdin 写一条 echo 命令（如 `echo vibestation`），能从 `PtyEvent::Stdout` 读到包含该 echo 内容的回显输出。
- [ ] **AC3** (PRD §Technical Risks R1): Windows 上 shell 进程退出（命令 `exit`）后，reader 经 `child.try_wait()` 在 `EXIT_WAIT_TIMEOUT` 内检测到退出并 emit `PtyEvent::Exited`——不 hang、不漏 exit 事件。
- [ ] **AC4** (PRD §Core Capabilities 1 · 本 task 推导): Windows 上 `PtySession::signal("SIGTERM")` / `terminate()` 经 `child.kill()`（底层 `TerminateProcess`）可靠终止 ConPTY 子进程并 emit `Exited`，不依赖任何 `libc::*` 调用。
- [ ] **AC5** (PRD §Anti-metrics 「不能为编译过阉割 Unix PTY」· §Compatibility requirements): macOS / Linux 上 `signal()`（`libc::kill` + `SIGINT`/`SIGTERM`/`SIGKILL`）/ `signal_target()`（`tcgetpgrp` 前台进程组）/ reader（mio）退出检测行为零回归——既有 Unix PTY 信号 / 前台进程组 / 退出测试全绿。
- [ ] **AC6** (PRD §Decisions Log D5 · §Technical Risks R3): 全部 Windows 改动经 `#[cfg(windows)]` / `#[cfg(unix)]` 分支落地，`cargo clippy --workspace --all-targets -- -D warnings` 在两平台均零警告（含 `SignalTarget::ProcessGroup` 的 dead_code 收敛）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 ConPTY spawn 读到 prompt | SCEN-2.2.1 | TEST-2.2.1 `test_2_2_1_conpty_spawn_cmd_reads_prompt` | `crates/core/tests/pty_windows_conpty_integration.rs`（`#[cfg(windows)]`） | cargo test --workspace | Not Started |
| AC2 echo 回显 round-trip | SCEN-2.2.2 | TEST-2.2.2 `test_2_2_2_conpty_echo_roundtrip` | 同上集成测试 | cargo test --workspace | Not Started |
| AC3 退出检测不 hang | SCEN-2.2.3 | TEST-2.2.3 `test_2_2_3_conpty_detects_process_exit_no_hang` | 同上集成测试 | cargo test --workspace | Not Started |
| AC4 signal/terminate 经 child.kill | SCEN-2.2.4 | TEST-2.2.4 `test_2_2_4_signal_terminate_kills_conpty_child` | 同上集成测试 | cargo test --workspace | Not Started |
| AC5 mac/Linux signal/reader 零回归 | SCEN-2.2.5 | TEST-2.2.5 既有 `signal_sigterm_exits_exec_session` 等 Unix signal 测试（`#[cfg(unix)]`，保不回归） | 既有 `crates/core` Unix PTY 集成测试 | cargo test --workspace（macOS/Linux runner） | Not Started |
| AC6 clippy 双平台零警告 | SCEN-2.2.6 | N/A（lint 门，非行为测试 — 见 §9 Lint） | N/A | cargo clippy --workspace --all-targets -- -D warnings | Not Started |

## 8. Risks

- **R-2.2-a**（关联 PRD §Technical Risks R1 ConPTY 行为差异）：ConPTY reader 退出检测与 Unix 不同，集成测试可能在 CI windows-latest 上偶发 timing flaky（类似既有 Linux epoll ignore 的成因）。缓解：测试 timeout 按本地最大运行时长 ×2 设置（dispatch §2.11 跨平台 timeout 约束）；退出检测用 `child.try_wait()` 轮询 + `EXIT_WAIT_TIMEOUT` 上界；若 windows-latest 仍 flaky，按 adr-005 加 `#[cfg_attr(windows, ignore = "...")]` + 技术债记录，不陷入 timeout 扩张循环。
- **R-2.2-b**（关联 PRD §Technical Risks R1）：Windows graceful kill 用 `child.kill()`（`TerminateProcess`）不投递真正的 Ctrl-C，AI agent 进程可能来不及清理。缓解：MVP 接受（PRD §Out of Scope 标精确 Ctrl-C 投递留后续）；§10 剩余风险记录；`terminate()` 仍先尝试再强制。
- **R-2.2-c**（关联 PRD §Technical Risks R3 mac/Linux 回归）：`SignalTarget::Process` 载荷由 `libc::pid_t` 改 `u32` 触及 Unix 构造点。缓解：改动局部、Unix 构造处显式 `as libc::pid_t`、既有 Unix signal 测试全绿锁定。
- **R-2.2-d**（关联 PRD §Technical Risks R5 CI 无 GUI）：windows-latest headless 跑不了完整 GUI smoke。缓解：本 task 集成测试纯进程级（无 GUI 依赖）；GUI critical UX path 靠开发者本机 §2.14 实跑（归 Phase 6 / runtime smoke）。

## 9. Verification Plan

> Rust task，命令对齐 adapter §Commands（Rust 主槽位）。Windows 改动的实施与验证在 Windows 11 本机；mac/Linux 回归由 CI 矩阵 / reviewer 保证。集成测试随 `cargo test --workspace` 一起跑（`crates/core/tests/`）。

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace（含 `crates/core/tests/pty_windows_conpty_integration.rs` 集成测试 · `#[cfg(windows)]` 门控）
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：
  - `crates/core/src/pty.rs`（修改 · Windows signal 映射 + signal_target + SignalTarget cfg 收敛 + reader 退出检测收尾）
  - `crates/core/tests/pty_windows_conpty_integration.rs`（新增 · `#[cfg(windows)]` 集成测试）
  - <TBD-after-impl>
- **commit 列表**：
  - <TBD-after-impl> test: 加 SCEN-2.2.1 ~ 2.2.4 的 RED 集成测试 + §5.3 骨架
  - <TBD-after-impl> feat: 实现 Windows ConPTY signal 映射 + 退出检测收尾通过全部测试
  - <TBD-after-impl> refactor:（如有）
- **§9 Verification 结果**：
  - install: <TBD-after-impl>
  - typecheck: <TBD-after-impl>
  - unit-test: <TBD-after-impl> passed / 0 failed
  - build: <TBD-after-impl>
  - lint: <TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>（预期：精确 Ctrl-C 投递未做 · 用 child.kill 兜底，OQ 留后续）
- **下游 task 影响**：<TBD-after-impl>（Phase 6 端到端 smoke 矩阵依赖本 task 的 ConPTY 真能 spawn/读写/退出）
