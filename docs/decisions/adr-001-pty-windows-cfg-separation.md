# ADR `001`: PTY 的 Windows 适配用 `#[cfg(target_os)]` 分离 + portable-pty ConPTY

**Status**: Accepted
**Date**: 2026-05-29
**Category**: 架构
**Related**: PRD §Decisions Log D1

## Context

`crates/core/src/pty.rs` 当前是 Unix-only 内核，Windows 上 `cargo build --workspace` 直接编译失败，是整个 Windows 适配的首要 blocker。survey 确认的 Unix-bound 点集中在一个文件内：

- reader loop 用 `mio::unix::SourceFd`（仅 Unix · `pty.rs` 行 10 + reader_loop）做 epoll/kqueue 轮询；ConPTY 不暴露可 poll 的 fd。
- 信号处理硬编码 `libc::SIGINT/SIGTERM/SIGTSTP/SIGKILL` + `libc::kill`（`parse_signal` / `PtySession::signal`）；Windows 无这些常量与 syscall。
- 前台进程组检测用 `libc::tcgetpgrp`（`signal_target`）；Windows ConPTY 无进程组概念。
- 非阻塞 fd 用 `libc::fcntl(F_SETFL, O_NONBLOCK)`（`set_fd_nonblocking`）；Windows 管道走 overlapped I/O。
- 默认 shell / shell 枚举 / cwd 检测 / 可执行位检测各有 Unix 假设（拆到 ADR-003 与 task-2.x 单独处理）。

关键既有底座：`portable-pty` crate **本就统一抽象了 ConPTY 与 Unix PTY**（tech-research.md:74），`crates/core/Cargo.toml` 的依赖（portable-pty / notify / rusqlite）survey 确认全部支持 Windows。这意味着 Windows 适配不需要换 PTY 库，只需要在 reader / 信号 / fd 这几处把 Unix-only 实现细节做平台分支。本 ADR 决定该分支怎么落地。

## Decision

在 `crates/core/src/pty.rs` 内用 `#[cfg(target_os = "windows")]` / `#[cfg(unix)]`（或 `#[cfg(not(windows))]`）分离两条 reader / 信号 / fd 路径，**复用同一 `PtySession` 抽象 + 局部 helper**，不抽象 `PtyBackend` trait、不复制整模块：

- **Reader**：Windows 走 portable-pty 的 ConPTY 阻塞读路径 —— 阻塞读循环 + `child.try_wait()` 做退出检测；Unix 保留现有 `mio::Poll + SourceFd` 路径不动。
- **信号**：抽出平台感知 helper（语义上 `send_signal_to_process(target, signal_name)`）—— Windows 用 `child.kill()` / `TerminateProcess`（SIGINT 经 ConPTY Ctrl-C 语义），Unix 保留 `libc::kill` 到进程组。
- **前台进程组**：Windows 的 `signal_target` 直接返回单进程目标（无组概念），Unix 保留 `tcgetpgrp`。
- **非阻塞 fd**：Windows 路径不调 `fcntl`（portable-pty 的 ConPTY 内部已处理 overlapped I/O），Unix 保留 `set_fd_nonblocking`。

所有 Windows 改动落在 cfg 分支内，Unix 编译路径字节级不变（Unix 行为零回归是反指标硬约束）。

## Rationale

- **YAGNI**：唯一真正 Unix-bound 的逻辑是 reader loop + 信号 + fd 这几处；`PtyBackend` trait 双实现属过度设计，portable-pty 已经在库层做了统一抽象，再加一层 trait 只增维护面。
- **零漂移**：复用同一 `PtySession` 会话语义，避免两份会话逻辑随时间漂移（候选 (a) 的主要风险）。
- **最小改动 + Unix 零回归**：cfg 分支 + 局部 helper 是改动面最小、对 Unix 路径影响最小的方案，便于 reviewer 三平台审查与 TDD RED 锁住 Unix 行为。

## Alternatives

- **(a) 整模块 Windows 全新实现**：拒绝 —— 会重复维护两份会话逻辑，长期易漂移；多数会话状态机逻辑本就平台无关，重写浪费。
- **(b) 抽象 `PtyBackend` trait 双实现**：拒绝 —— 过度设计（YAGNI）；portable-pty 已统一 ConPTY/Unix PTY 抽象（tech-research:74），唯一 Unix-bound 处是 reader loop，不值得为它引入 trait 层。
- **(c) cfg 分支 + 局部 helper**（**选定**）：最小改动 + 零 Unix 回归 + 复用现有抽象。

## Consequences

**正面**：

- Windows 上 `cargo build -p vibestation_core` / `cargo build --workspace` 可编译通过，解锁 PRD Phase 1 的全部后续 task（决策表 #8 / ADR-006 的 Windows v0.4 适配真正启动）。
- Unix 路径行为零变化（mio 轮询 / 信号语义 / 前台进程组 / 非阻塞 fd 全部保留）。
- portable-pty 统一抽象被复用，未来 ConPTY 行为升级随库升级受益。

**负面 / 风险**：

- ConPTY reader 的退出检测 / 信号语义 / 尾部输出读取与 Unix PTY 存在行为差异，可能 hang 或漏 exit（PRD R1，概率高 / 影响高）。缓解：Windows reader 用 `child.try_wait()` + 阻塞读循环；先写 ConPTY spawn/exit/echo 集成测试再实现；本机 Windows 11 实跑 smoke 验证退出与回显。
- cfg 分支写错可能波及 Unix 路径（PRD R3）。缓解：严格 cfg 隔离 + mac/Linux 全量 `cargo test` 本地+CI 必须仍绿 + TDD 先 RED 锁住 Unix 行为不变。
- `detect_process_cwd` 的 Windows 精确实现不在本 ADR 范围（PRD Out of Scope）—— 用 spawn-time 缓存的 `initial_cwd` 兜底，OQ3 决定是否后续升级到 Windows API 查询。

## Rollback Or Migration Plan

- **回滚**：本决策不引入新外部依赖、不改 DB schema、不改 IPC 契约，回滚成本低 —— 若 ConPTY 路径在本机 smoke 暴露不可接受的 hang/漏 exit，可临时把 Windows 路径降级为编译期 `#[cfg(not(windows))]` 整段排除 PTY（Windows 仅 build 不提供 PTY 运行），保留 Unix 路径不动，再迭代。
- **迁移**：无数据迁移。Windows 用户首次拿到 PTY 能力即新增行为，无既有状态需迁移。

## Follow-ups

- task-1.1（pty-platform-split）落地 reader / 信号 / fd 的 cfg 分离（编译解锁）。
- task-2.1（windows-shell-detection）/ task-2.2（conpty-spawn-io）落地 ConPTY spawn/IO 与退出检测，关联本机 smoke（PRD R1 缓解）。
- OQ3：`detect_process_cwd` Windows 精确实现是否必要 —— MVP 用缓存 cwd 兜底，按反馈决定升级。
- 关联 ADR-003（Windows 默认 shell 探测链）/ ADR-005（Windows 测试门控策略）/ ADR-006（fs_watch Windows backend）。
