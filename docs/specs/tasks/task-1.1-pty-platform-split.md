# Task `1.1`: `pty-platform-split`

**Status**: In Progress

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`

**Priority**: P0
**Owner**: 主 agent
**Related Phase**: 1（foundation-build）
**Dependencies**: -（Phase 1 首 task · 解锁 Windows 编译 · 无上游）

## 1. Background

`crates/core/src/pty.rs` 是 PTY 运行期核心（portable-pty + 单 shared-reader + bounded queue + drop-oldest，ADR-003）。当前其内核**直接耦合 Unix-only API**，导致 Windows 上 `cargo build --workspace` 直接编译 fatal、连跑都跑不起来：

- 文件顶部 `use mio::unix::SourceFd`（line 10）+ `use std::os::fd::RawFd` / `use std::os::unix::fs::PermissionsExt`（line 17-18）。
- `reader_loop`（line 792-873）用 `mio::Poll` + `SourceFd` 注册 fd 做 epoll/kqueue 轮询。
- `read_session_fd`（line 875）裸 libc 风格读。
- 信号：`PtySession::signal` 调 `libc::kill`（line 311）、`signal_target`（line 455）调 `libc::tcgetpgrp`、`parse_signal`（line ~1003）用 `libc::SIG*` 常量。
- `set_fd_nonblocking`（line 1208）调 `libc::fcntl(F_GETFL/F_SETFL, O_NONBLOCK)`。
- `is_executable_file`（line 1172）用 `metadata.permissions().mode() & 0o111`。
- `detect_process_cwd`（line 1178）`#[cfg(target_os)]` 仅覆盖 linux(/proc) / macos(lsof)，Windows 落 `None`。

这是 Windows 适配的**首个 task**——Windows 当前编译失败即本 task 要解决的红。`portable-pty` 本已统一 ConPTY/Unix PTY 抽象（tech-research.md:74），唯一 Unix-bound 处是 reader loop 的 fd 轮询与信号系统调用。ADR-001 选定：cfg 分支 + 局部 helper，复用同一 `PtySession` 抽象，不抽 trait（YAGNI）、不整模块重写（防漂移）。

## 2. Goal

任务完成后应成立的事实：

- Windows 11 x64 MSVC 上 `cargo check --workspace` + `cargo build --workspace` 0 编译错误（`pty.rs` 不再因 `mio::unix` / `libc::fcntl` / `PermissionsExt` fatal）。
- Unix（macOS/Linux）reader loop / 信号 / fcntl 非阻塞 / 前台进程组 / cwd 检测行为**字节级零回归**（现有 Unix 单测/集成测试仍绿）。
- Windows 走 `#[cfg(windows)]` ConPTY reader 路径骨架：`portable-pty` 阻塞读循环 + `child.try_wait()` 退出检测（不用 mio），编译通过且能 spawn 后被关闭/退出而不 hang（运行期回显正确性 defer Phase 2）。

## 3. Scope

### In Scope

- `crates/core/src/pty.rs`：
  - 顶部 imports cfg 分离：`mio::unix::SourceFd` / `mio::{Events,Interest,Poll,Token}` / `std::os::fd::RawFd` / `std::os::unix::fs::PermissionsExt` / `libc` 全部置于 `#[cfg(unix)]`。
  - `reader_loop`：`#[cfg(unix)]` 保留现有 mio Poll + SourceFd 实现；`#[cfg(windows)]` 新增 ConPTY 阻塞读循环 + `child.try_wait()` 退出检测路径（同签名、同 `DropOldestSender<PtyEvent>` 出口契约）。
  - `set_fd_nonblocking`：`#[cfg(unix)]` 保留 `libc::fcntl`；`#[cfg(windows)]` 改为 no-op（portable-pty ConPTY 内部已处理 overlapped I/O）。
  - 信号链 `PtySession::signal` / `signal_target` / `parse_signal`：`#[cfg(unix)]` 保留 `libc::kill` / `tcgetpgrp` / `SIG*`；`#[cfg(windows)]` 用 `Child::kill()` / 直接返回 `SignalTarget::Process(pid)`（ConPTY 无进程组概念）。
  - `is_executable_file`：`#[cfg(unix)]` 保留 mode 位检查；`#[cfg(windows)]` 改为 `path.is_file()`（.exe/.bat 按扩展名+注册表可执行）。
  - `detect_process_cwd`：保持现有 `#[cfg(target_os)]`；Windows 分支显式返回 `None`（用 spawn-time 缓存 `initial_cwd` 兜底，PRD §Out of Scope 明确不做精确实现）。
- `crates/core/src/pty.rs` 内嵌 `#[cfg(test)] mod tests`：现有 `use std::os::unix::fs::PermissionsExt` 等 Unix-only 测试 import 加 cfg-gate；新增 Windows 编译 smoke 测试。

### Out Of Scope

- ConPTY 运行期回显 / 退出语义的完整正确性验证（Phase 2 task 2.2 conpty-spawn-io）。
- Windows shell 探测链 `default_shell_path` / `list_available_shells` / `resolve_shell`（Phase 2 task 2.1）。
- `detect_process_cwd` 的 Windows 精确实现（PRD §Out of Scope · 缓存 cwd 兜底）。
- 集成测试 `crates/core/tests/*` 的 Windows ignore 标记（Phase 6 task 6.1 windows-test-gating）。
- git hook `#!/bin/sh` 生产逻辑 Windows 化（PRD §Out of Scope）。

## 4. Users / Actors

- **项目 Windows 贡献者**：在 Windows 本机 `cargo build/test`，本 task 让其首次能编译 `crates/core`。
- **windows-latest CI**（Phase 5 引入后）：消费本 task 的编译绿作为矩阵 build 前提。
- **下游 task 实施 agent**（2.1 / 2.2）：在本 task 的 cfg 骨架上填 Windows shell 探测 + ConPTY 运行期逻辑。

## 5. Behavior Contract

PTY 公共契约（`PtySession::spawn` / `signal` / reader 经 `PtyEvent` 出口）在 Unix 上**完全不变**；Windows 上同签名编译通过，运行期最低限度可 spawn + 退出检测（不 hang）。

### 5.1 Required Reading

- 本 task 无上游 task（Phase 1 首）。
- [`docs/decisions/adr-001-pty-windows-cfg-separation.md`](../../decisions/adr-001-pty-windows-cfg-separation.md)（PTY cfg 分离 + portable-pty ConPTY 决策）。
- [`docs/decisions/adr-005-windows-test-gating-strategy.md`](../../decisions/adr-005-windows-test-gating-strategy.md)（Unix-only 测试 cfg-gate 策略）。
- BDD: [`test/features/pty.feature`](../../../test/features/pty.feature)。
- 现有源：`crates/core/src/pty.rs`（ADR-003 单 shared-reader + mio poll 架构）；`docs/spikes/SPIKE-05-report.md` / `SPIKE-05.5-report.md`（PTY 架构依据）。

### 5.2 Imports

```rust
// 跨平台（不变）
use portable_pty::{native_pty_system, Child, CommandBuilder, ExitStatus, MasterPty, PtySize};
use crossbeam_channel::{self, Receiver, Sender, TryRecvError, TrySendError};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

// Unix-only（cfg-gate）
#[cfg(unix)]
use mio::unix::SourceFd;
#[cfg(unix)]
use mio::{Events, Interest, Poll, Token};
#[cfg(unix)]
use std::os::fd::RawFd;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
// libc 已是 crates/core 依赖 · 仅在 #[cfg(unix)] 函数体内引用，无需顶层 use 调整

// Windows-only（新增 · cfg-gate）
// portable_pty::Child::kill / try_wait 用于 ConPTY 退出检测，无需额外 winapi 依赖
```

### 5.3 函数签名

> Windows 适配后的真实签名骨架。Unix 路径保留现有实现；Windows 路径为本 task 新增 `#[cfg(windows)]` 分支（编译通过 + 最低运行期，回显正确性 defer Phase 2）。

```rust
// reader loop —— 同一签名，两条 cfg 实现
#[cfg(unix)]
fn reader_loop(
    control_rx: Receiver<ReaderCommand>,
    sessions_by_id: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    events: DropOldestSender<PtyEvent>,
    reader_alive: Arc<AtomicBool>,
) { /* 现有 mio Poll + SourceFd 实现 · 不变 */ }

#[cfg(windows)]
fn reader_loop(
    control_rx: Receiver<ReaderCommand>,
    sessions_by_id: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    events: DropOldestSender<PtyEvent>,
    reader_alive: Arc<AtomicBool>,
) {
    // ConPTY 路径：对每个注册 session 阻塞读 reader（portable-pty master.try_clone_reader）
    // + child.try_wait() 检测退出 → emit_exit_once；不使用 mio Poll/SourceFd。
    // 本 task 仅求编译通过 + spawn 后能退出不 hang；调度/吞吐细化 defer Phase 2 task 2.2。
    unimplemented!("Phase 2 task-2.2 conpty-spawn-io 填运行期细节")
}

// fd 非阻塞 —— Unix fcntl / Windows no-op
#[cfg(unix)]
fn set_fd_nonblocking(fd: RawFd, enabled: bool) -> Result<(), PtyError> { /* 现有 libc::fcntl · 不变 */ }
#[cfg(windows)]
fn set_fd_nonblocking_noop() { /* ConPTY overlapped I/O 由 portable-pty 内部处理 · 无需 fcntl */ }

// 信号目标 —— Unix tcgetpgrp / Windows 单进程
impl PtySession {
    #[cfg(unix)]
    fn signal_target(&self) -> Option<SignalTarget> { /* 现有 libc::tcgetpgrp · 不变 */ }
    #[cfg(windows)]
    fn signal_target(&self) -> Option<SignalTarget> {
        // ConPTY 无前台进程组概念 · 直接路由到单进程
        Some(SignalTarget::Process(self.process_id()))
    }
}

// 可执行文件判定 —— Unix mode 位 / Windows is_file
#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool { /* 现有 (mode & 0o111) != 0 · 不变 */ }
#[cfg(windows)]
fn is_executable_file(path: &Path) -> bool {
    std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
}

// 进程 cwd 检测 —— Windows 显式 None（缓存 initial_cwd 兜底）
fn detect_process_cwd(process_id: u32) -> Option<PathBuf> {
    #[cfg(target_os = "linux")] { /* 现有 /proc/{pid}/cwd · 不变 */ }
    #[cfg(target_os = "macos")] { /* 现有 lsof · 不变 */ }
    #[cfg(target_os = "windows")] { let _ = process_id; None } // PRD §Out of Scope · 缓存兜底
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))] { let _ = process_id; None }
}

// 信号名解析 —— Unix libc::SIG* / Windows 映射 kill 语义
#[cfg(unix)]
fn parse_signal(name: &str) -> Option<i32> { /* 现有 libc::SIGINT/SIGTERM/... · 不变 */ }
#[cfg(windows)]
fn parse_signal_windows(name: &str) -> WindowsSignal { /* SIGINT/SIGTERM → Child::kill 语义 */ }
```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities 1): Windows 11 x64 MSVC 上 `cargo check --workspace` + `cargo build --workspace` 退出码 0、0 编译错误（修复前 `pty.rs` 因 `mio::unix::SourceFd` / `libc::fcntl` 编译 fatal）。
- [ ] **AC2** (PRD §Success Metrics 反指标): macOS + Ubuntu 上 `cargo test --workspace` 仍 100% 绿——Unix reader loop / 信号 / fcntl 非阻塞 / 前台进程组 / `is_executable_file` mode 位行为字节级零回归（所有 Unix 内核置于 `#[cfg(unix)]`，未改逻辑）。
- [ ] **AC3** (PRD §Decisions Log D1): Windows reader 路径走 `#[cfg(windows)]` ConPTY 阻塞读 + `child.try_wait()` 退出检测骨架，**不引用 mio/SourceFd/libc**；`set_fd_nonblocking` 在 Windows 为 no-op（portable-pty 内部处理 overlapped I/O）。
- [ ] **AC4** (PRD §Out of Scope): `detect_process_cwd` 在 Windows 安全返回 `None`（不 panic、不调用 `/proc`/lsof），由 spawn-time 缓存 `initial_cwd` 兜底。
- [ ] **AC5** (本 task 新增): `crates/core/src/pty.rs` 内 `#[cfg(test)] mod tests` 的 Unix-only import（`PermissionsExt` 等）已 cfg-gate；新增一个三平台均可编译的 reader/signal 路径存在性单测，在 Windows 不因缺失符号编译 fail。
- [ ] **AC6** (PRD §Constraints 兼容性): `cargo clippy --workspace --all-targets -- -D warnings` 在三平台 0 warning——cfg 分支不引入 `dead_code` / `unused_imports` / `unused_variables` warning。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 Windows 编译绿 | SCEN-1.1.1 | TEST-1.1.1 `test_1_1_1_windows_pty_compiles` | N/A: 集成随 build 验证 | `cargo build --workspace`（Windows） | Not Started |
| AC2 Unix 零回归 | SCEN-1.1.2 | TEST-1.1.2 `test_1_1_2_unix_reader_signal_unchanged` | crates/core/tests/pty_*（既有 Unix）| `cargo test --workspace`（mac/ubuntu） | Not Started |
| AC3 Windows ConPTY reader 骨架 | SCEN-1.1.3 | TEST-1.1.3 `test_1_1_3_windows_reader_no_mio` | N/A | `cargo build --workspace`（Windows）+ grep 无 mio 于 cfg(windows) | Not Started |
| AC4 detect_process_cwd Windows None | SCEN-1.1.4 | TEST-1.1.4 `test_1_1_4_detect_cwd_windows_none` | N/A | `cargo test -p vibestation_core pty`（Windows） | Not Started |
| AC5 测试 import cfg-gate | SCEN-1.1.5 | TEST-1.1.5 `test_1_1_5_tests_compile_all_platforms` | N/A | `cargo test --workspace --no-run`（三平台） | Not Started |
| AC6 clippy 三平台 0 warning | N/A: 静态检查无业务场景 | N/A: 由 lint 命令覆盖 | N/A | `cargo clippy --workspace --all-targets -- -D warnings` | Not Started |

## 8. Risks

- **R1（关联 PRD §Technical Risks R1 · ConPTY 行为差异）**：Windows reader 骨架的退出检测 / 尾部输出读取与 Unix PTY 语义不同，可能 hang 或漏 exit。缓解：本 task 范围限"编译绿 + spawn 后能退出不 hang"，运行期回显正确性 defer Phase 2；骨架显式 `unimplemented!()` 标注归属，避免"返回零值"型模糊红。
- **R2（关联 PRD §Technical Risks R3 · mac/Linux 回归）**：cfg 分离误把 Unix 路径改坏。缓解：现有 Unix 内核整体包 `#[cfg(unix)]` 不动逻辑；TDD 先 RED 锁 Unix 行为；mac/Linux 全量 cargo test 本地+CI 必绿（反指标）。
- **R3（编译型语言 RED 桥接 · standard §2.5.1）**：Windows 测试引用尚不存在的 `#[cfg(windows)]` 符号 = 编译失败（非合法 RED）。缓解：RED commit 随失败测试一并提交可编译空骨架（`unimplemented!()`），使测试因刻意标记而红、非编译 fail。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile  <!-- 与 adapter §Commands Install 一致 -->
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制：实施 agent 不允许 N/A -->
- **Build**: cargo build --workspace
- **Runtime smoke**: pnpm tauri:dev  <!-- Windows 本机 · Phase 2 才验证 PTY 运行期回显；本 task 仅确认应用能启动不崩 -->

> Integration / E2E / Coverage 本 task N/A：集成测试随 `cargo test --workspace` 一起跑（crates/core/tests/），无独立 e2e 框架，MVP 不强制覆盖率。

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：
  - `crates/core/src/pty.rs`（修改 · cfg 分离）
- **commit 列表**：
  - `<TBD-after-impl>` test: 加 SCEN-1.1.1~1.1.5 RED 测试 + cfg 骨架
  - `<TBD-after-impl>` feat: 实现 pty.rs Windows cfg 分支通过测试
  - `<TBD-after-impl>` refactor:（如有）
- **§9 Verification 结果**：
  - install: <TBD-after-impl>
  - lint: <TBD-after-impl>
  - typecheck: <TBD-after-impl>
  - unit-test: <TBD-after-impl>
  - build: <TBD-after-impl>
  - runtime-smoke: <TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
