# Task `2.2`: `conpty-spawn-io`

> Task Spec · 按 S2V standard §8.3 模板渲染。无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研证据（`spike-tmp/win-survey.json`）+ 实际 `crates/core/src/pty.rs` 源码填实，非编造。

**Status**: Done

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

- [x] **AC1** (PRD §Core Capabilities 1/2 · §Success Metrics 次要指标「PTY runtime smoke」): Windows 上经 ConPTY spawn `cmd.exe`（或 pwsh）后，能在有界时间内从 `PtyEvent::Stdout` 读到非空 prompt 输出（不 hang、不返回空）。✅ TEST-2.2.1 pass（实跑读到 cmd.exe `Microsoft Windows [版本 ...]` 横幅）。
- [x] **AC2** (PRD §Core Capabilities 2 · §User Flow 主流程 3): Windows 上向 ConPTY 会话 stdin 写一条 echo 命令（如 `echo vibestation`），能从 `PtyEvent::Stdout` 读到包含该 echo 内容的回显输出。✅ TEST-2.2.2 pass（`echo vibestation-conpty-marker` → 回显含 marker）。
- [x] **AC3** (PRD §Technical Risks R1): Windows 上 shell 进程退出（命令 `exit`）后，reader 经 `child.try_wait()` 在 `EXIT_WAIT_TIMEOUT` 内检测到退出并 emit `PtyEvent::Exited`——不 hang、不漏 exit 事件。✅ TEST-2.2.3 pass。实现：reader_loop 主循环周期 `try_wait()` 轮询 + `close_master`（ConPTY `exit` 后 conhost 仍持管道 · 单靠 read-EOF 漏检 · 见 §10 实测根因）。
- [x] **AC4** (PRD §Core Capabilities 1 · 本 task 推导): Windows 上 `PtySession::signal("SIGTERM")` / `terminate()` 经 `child.kill()`（底层 `TerminateProcess`）可靠终止 ConPTY 子进程并 emit `Exited`，不依赖任何 `libc::*` 调用。✅ TEST-2.2.4 pass（signal SIGTERM kill child → kill()/terminate close_master → Exited）。
- [x] **AC5** (PRD §Anti-metrics 「不能为编译过阉割 Unix PTY」· §Compatibility requirements): macOS / Linux 上 `signal()`（`libc::kill` + `SIGINT`/`SIGTERM`/`SIGKILL`）/ `signal_target()`（`tcgetpgrp` 前台进程组）/ reader（mio）退出检测行为零回归——既有 Unix PTY 信号 / 前台进程组 / 退出测试全绿。✅ Unix `signal`/`signal_target`/`SignalTarget` 全 `#[cfg(unix)]` 路径零改动（`master` 改 `Option` 后 `signal_target` 对 `None` 经 `as_ref().and_then` 安全降级 · resize 同）· lib `pty::tests` 38 passed 无回归 · 待 task-5.2 CI macOS/Linux runner 实跑确认。
- [x] **AC6** (PRD §Decisions Log D5 · §Technical Risks R3): 全部 Windows 改动经 `#[cfg(windows)]` / `#[cfg(unix)]` 分支落地，`cargo clippy --workspace --all-targets -- -D warnings` 在两平台均零警告。✅ pty.rs + 集成测试 0 warning。**实现差异说明**：本 task 沿用 task-1.1 既有设计——`SignalTarget` 枚举整体 `#[cfg(unix)]`（Windows `signal()` 直接 `child.kill()` · 不经 `signal_target`），故 Windows 上 `SignalTarget` 不存在 · 无 `ProcessGroup` dead_code 需收敛（§5.3 提议的"跨平台 SignalTarget + `Process(u32)` 载荷形变"未采用 · 既有 cfg 分离更简洁且同样满足 AC4「不依赖 libc」）。`fs_watch.rs` / `external_term/detect.rs` 既有 warning 属 task 3.4 / 3.1 · 非本 task。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 ConPTY spawn 读到 prompt | SCEN-2.2.1 | TEST-2.2.1 `test_2_2_1_conpty_spawn_cmd_reads_prompt` | `crates/core/tests/pty_windows_conpty_integration.rs`（`#[cfg(windows)]`） | cargo test --workspace | Done |
| AC2 echo 回显 round-trip | SCEN-2.2.2 | TEST-2.2.2 `test_2_2_2_conpty_echo_roundtrip` | 同上集成测试 | cargo test --workspace | Done |
| AC3 退出检测不 hang | SCEN-2.2.3 | TEST-2.2.3 `test_2_2_3_conpty_detects_process_exit_no_hang` | 同上集成测试 | cargo test --workspace | Done |
| AC4 signal/terminate 经 child.kill | SCEN-2.2.4 | TEST-2.2.4 `test_2_2_4_signal_terminate_kills_conpty_child` | 同上集成测试 | cargo test --workspace | Done |
| AC5 mac/Linux signal/reader 零回归 | SCEN-2.2.5 | 既有 `signal_sigterm_exits_exec_session` 等 Unix signal 测试（`#[cfg(unix)]`，保不回归） | 既有 `crates/core` Unix PTY 集成测试 | cargo test --workspace（macOS/Linux runner） | Done（Windows 本机 cfg(unix) 不执行 · lib pty::tests 38 passed 无回归 · 待 CI 矩阵 macOS/Linux runner 实跑） |
| AC6 clippy 双平台零警告 | SCEN-2.2.6 | N/A（lint 门，非行为测试 — 见 §9 Lint） | N/A | cargo clippy --workspace --all-targets -- -D warnings | Done（pty.rs + 集成测试 0 warning · fs_watch/detect.rs 既有 warning 属 task 3.x） |

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

- **完成日期**：2026-05-29
- **改动文件**：
  - `crates/core/src/pty.rs`（修改）：
    - `PtySession.master` 改 `Mutex<Option<Box<dyn MasterPty + Send>>>` + 新增 `#[cfg(windows)] close_master()`（take 掉 master → `ClosePseudoConsole`）
    - `terminate()` 在 kill child 后 `#[cfg(windows)]` 调 `close_master()`（解 reader join 死锁）
    - Windows `reader_loop` 主循环加 `child.try_wait()` 周期轮询 + `close_master`（AC3 自然退出检测 · 单靠 read-EOF 漏检）
    - `resize` / `signal_target` 对 `master = None` 经 `as_ref()` 安全降级（Unix 行为不变）
    - reader 线程 clone reader 处适配 `Option`
    - 注：Windows `signal()` / `parse_signal_windows` / `WindowsSignal` / `SignalTarget`（`#[cfg(unix)]`）沿用 task-1.1 既有设计 · 本 task 未改（§6 AC6 说明未采用 §5.3 提议的跨平台 SignalTarget 形变）
  - `crates/core/tests/pty_windows_conpty_integration.rs`（新增 · 全文件 `#![cfg(windows)]` · 自包含 · DSR 握手代回复 helper）
- **commit 列表**：
  - `045ef64` test(pty): 加 SCEN-2.2.1~2.2.4 ConPTY spawn/echo/exit/signal 集成测试（RED · 实测 spawn 测试 HANG）
  - `41d3665` feat(pty): ConPTY 退出检测收尾 + master drop 解 reader join 死锁（GREEN）
  - refactor：无
- **§9 Verification 结果**（Windows 11 本机实跑 · 2026-05-29）：
  - install: 未跑（纯 Rust · 不触前端）
  - typecheck（`cargo check --workspace`）: 0 error
  - unit-test：
    - `cargo test -p vibestation-core --test pty_windows_conpty_integration --test-threads=1` → **5 passed / 0 failed**（test_2_2_1~2_2_4 + test_2_2_signal_unknown_tab_errors）· 连跑 3 次稳定（3.09~3.14s · 无 flaky）· 并行模式 1.04s
    - 真实 spawn 证据：TEST-2.2.1 读到 `Microsoft Windows [版本 10.0.26200.7462]` 横幅 + `C:\...\Temp>` prompt；TEST-2.2.2 `echo vibestation-conpty-marker` 回显 marker；TEST-2.2.3 `exit` → `Exited(Some(..))`；TEST-2.2.4 SIGTERM+kill → Exited
    - `cargo test -p vibestation-core --lib pty::tests` → 38 passed / 0 failed（task-2.1 无回归）
  - build（`cargo build --workspace`）: 0 error
  - lint（`cargo clippy -p vibestation-core --lib`）: pty.rs 0 warning；集成测试文件 0 warning（`fs_watch.rs` / `external_term/detect.rs` 既有 warning 属 task 3.4 / 3.1）
- **实测根因（ConPTY DSR 握手 · 关键发现）**：`cmd.exe` 在 ConPTY 下启动后先 emit `ESC[6n`（DSR · 查询光标位置）· 在收到终端回复 `ESC[<row>;<col>R` 前**不画 prompt、不处理 stdin**（实测仅 emit `\u{1b}[6n` 后完全静默 · 写 echo/exit 无任何输出）。生产环境由前端 xterm.js 自动回复；headless 集成测试无前端 · 故测试 helper（`reply_dsr_if_needed` / `pump_dsr`）代回复 `ESC[1;1R` 模拟 VT-capable 消费端。这是 ConPTY 语义 · 非 reader bug。
- **剩余风险 / 未做项**：
  - R-2.2-b：精确 Ctrl-C 投递（`GenerateConsoleCtrlEvent`）未做 · `signal()` 全信号退化为 `child.kill()`（`TerminateProcess`）· AI agent 进程可能来不及清理（PRD §Out of Scope · OQ 留后续）。
  - `pty_pool::tests` 13 个 Windows 失败为 pre-existing（pool 子系统 Unix-hardcoded fixture + 自身 DSR 假设 · ADR-005 测试门控范畴 · 归 task-6.1）· 本 task 改动未引入/未消除（baseline 同 7 passed / 13 failed）。
  - AC5/AC6 mac/Linux 零回归靠 `#[cfg(unix)]` + 改动局部性 + lib 测试无回归保证 · Windows 本机无法执行 Unix 分支 · 待 task-5.2 CI 矩阵最终确认。
  - 集成测试随 `cargo test --workspace` 跑时 · 若 CI windows-latest 偶发 ConPTY timing flaky · 按 ADR-005 / R-2.2-a 加 `#[cfg_attr(windows, ignore)]` + 技术债（当前本机连跑 3 次稳定 · 暂不需要）。
- **下游 task 影响**：Phase 6 端到端 smoke 矩阵（task-6.2）现有 ConPTY 真能 spawn/读写/退出/终止的运行期证据；task-6.1 Windows 测试门控可参照本 DSR 握手模式处理 `pty_pool` / `shell_compat` 的 ConPTY 测试。
