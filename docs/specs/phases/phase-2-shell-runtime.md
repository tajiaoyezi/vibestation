# Phase 2: shell-runtime

> Phase Spec · 按 S2V standard §8.2 八项结构渲染。
> Master Spec：[`docs/prds/windows-support.prd.md`](../../prds/windows-support.prd.md) §Implementation Phases 第 2 行。

**Status**: Done

> Allowed values: `Draft / Ready / In Progress / Done / Blocked / Waived`（standard §10.5.1 状态机）。

---

## 1. 阶段目标

在 Phase 1（`pty.rs` 编译期 `#[cfg(target_os)]` 分离 + 跨平台家目录 + shell 默认基元就位）的基础上，让 Windows 上**真正能探测/枚举 shell 并经 ConPTY 拉起 pwsh/cmd 正常读写**：

- `default_shell_path()` 在 Windows 走 `pwsh.exe → powershell.exe → cmd.exe` 探测链（取第一个可用，`cmd.exe` 永远保底）。
- `list_available_shells()` 在 Windows 不读 `/etc/shells`，改走 PATH / `where.exe` 枚举 `pwsh` / `powershell` / `cmd` / git-bash。
- `resolve_shell` / `find_available_shell` 在 Windows 用 `where.exe` 而非 `which`。
- `detect_process_cwd` 在 Windows 用 spawn-time 缓存的 `initial_cwd` 兜底（不做精确 API 查询，OQ3）。
- ConPTY reader 路径能 spawn `cmd.exe` / `pwsh`，读到 prompt / echo 输出，并经 `child.try_wait()` 检测进程退出。
- signal 在 Windows 映射为 `child.kill()` / `TerminateProcess`（无 `SIGINT` 概念时走 graceful kill）。

所有改动严格走 `#[cfg(target_os)]` / `#[cfg(windows)]` / `#[cfg(unix)]` 分支，**mac/Linux 行为零回归**。

## 2. 业务价值

Phase 1 解锁"编译过 + 启动不崩"，但用户在 Windows 上新建 Tab 仍然拉不起可用的交互 shell——这是"多 Tab 终端 + AI agent 会话"产品的命门。Phase 2 交付后：

- Windows 11 上的 AI-agent 开发者新建 Tab 即看到 pwsh / cmd 的真实 prompt，可跑 `git --version` / Claude CLI / Codex CLI 并看到正确回显（PRD §Core Capabilities 2 / §Success Metrics 次要指标「PTY runtime smoke」）。
- "找不到 pwsh 不崩、回落 cmd.exe 保底"消除了 PRD §User Flow 异常流「未装 PowerShell 7」的崩溃风险。
- 为 Phase 5（构建打包 CI）与 Phase 6（端到端 smoke 矩阵）提供"Windows shell 真能跑"的前置事实。

## 3. 涉及模块

| 模块 | 文件 | Windows 适配点 |
|---|---|---|
| pty-shell | `crates/core/src/pty.rs` | `default_shell_path` / `list_available_shells` / `resolve_shell` / `find_available_shell` / `detect_process_cwd` 的 `#[cfg(windows)]` 分支 |
| pty-conpty | `crates/core/src/pty.rs` + `crates/core/tests/` | ConPTY reader 退出检测（`child.try_wait`）+ signal 映射（`child.kill` / `TerminateProcess`）+ Windows-gated 集成测试 |

依赖 Phase 1 已落地：`pty.rs` 的 mio/libc/fcntl/`PermissionsExt` 已 `#[cfg]` 隔离、ConPTY reader 骨架已就位、`home_dir()` 助手 + `dirs` 依赖已可用。

## 4. 任务清单

| Task | 模块 | Spec | Status | 依赖 | 说明 |
|---|---|---|---|---|---|
| 2.1 | pty-shell | [`../tasks/task-2.1-windows-shell-detection.md`](../tasks/task-2.1-windows-shell-detection.md) | Ready | 1.1 | Windows shell 探测链 + 枚举 + `where.exe` 解析 + cwd 缓存兜底 |
| 2.2 | pty-conpty | [`../tasks/task-2.2-conpty-spawn-io.md`](../tasks/task-2.2-conpty-spawn-io.md) | Ready | 2.1 | ConPTY spawn/读写/退出检测集成测试 + Windows signal 映射收尾 |

> Phase 内顺序：2.1 先（提供 Windows shell 解析），2.2 后（依赖 2.1 能解析出可拉起的 shell 才能写 spawn 集成测试）。

## 5. 依赖关系

- **上游**：Phase 1 `foundation-build`（task 1.1 PTY 平台分离 = 本 phase 全部前置；task 1.2 home_dir 助手为 cwd/PATH 解析提供基础）。
- **本 phase 内**：2.2 依赖 2.1。
- **下游**：Phase 5 `build-package-ci`（task 5.2 windows-latest CI 矩阵依赖 2.1 的 shell 解析可用）、Phase 6 `integration-matrix`（端到端 smoke 依赖本 phase 的 ConPTY 真能跑）。
- **可并行**：Phase 2 不与 Phase 3 / 4 并行（PRD 阶段表标「否」——它独占 `pty.rs` 的 shell/reader 文件域，与同改 `pty.rs` 的其他改动需串行避免冲突）。

## 6. 阶段级验收标准

- [ ] **P2-A**：`cargo check --workspace`（Windows）零错误——Phase 2 的 `#[cfg(windows)]` 分支编译通过。
- [ ] **P2-B**：`cargo build --workspace`（Windows）零错误 + `cargo test --workspace`（Windows）全绿——含本 phase 新增的 Windows-gated 集成测试通过、Unix-only 测试正确 `#[cfg(unix)]` / `ignore` 跳过、mac/Linux 测试零回归。
- [ ] **P2-C**：Windows 上 `default_shell_path()` 返回探测链首个可用 shell（装了 pwsh → `pwsh.exe` 全路径；否则 `powershell.exe`；最终保底 `cmd.exe`），`list_available_shells()` 返回非空且不含任何 Unix `/bin/*` 路径。
- [ ] **P2-D**：Windows 上 ConPTY 能 spawn `cmd.exe` / `pwsh` 并读到 prompt + echo 输出，进程退出经 `child.try_wait()` 被检测到（不 hang）。
- [ ] **P2-E（端到端 smoke）**：
  - **Windows**：`cargo build --workspace` 0 错误；`cargo test --workspace` 中 Windows-gated 集成测试（`#[cfg(windows)]` 的 ConPTY spawn/echo/exit + shell 探测测试）全绿；本机 `pnpm tauri:dev` 新建 Tab，pwsh/cmd 显示 prompt，`git --version` / `echo hello` 有正确回显（PRD §Success Metrics 次要指标，§2.14 本机实跑记录证据）。
  - **macOS / Linux**：`cargo test --workspace` 全量仍 100% 绿（Unix mio/libc/signal/cwd 路径零变化），`pnpm tauri:dev` 新建 Tab 仍正常拉起 zsh/bash。

## 7. 阶段级风险

| # | 风险 | 来源 | 缓解 |
|---|---|---|---|
| PR1 | ConPTY reader 退出检测 / 信号语义与 Unix PTY 不同，可能 hang 或漏 exit、读不到尾部输出 | PRD §Technical Risks R1 | 先写 ConPTY spawn/exit/echo 集成测试再实现（TDD RED 先行）；reader 用 `child.try_wait()` + 阻塞读循环；本机 Windows 11 实跑 smoke 验退出与回显；保留 Unix mio 路径不动 |
| PR2 | cfg 分支写错把 Unix 路径也改坏（shell 探测 / signal / cwd） | PRD §Technical Risks R3 | 严格 `#[cfg(windows)]` / `#[cfg(unix)]` 分支；mac/Linux 全量 `cargo test` 本地 + CI 仍绿（反指标硬约束）；TDD 先 RED 锁住 Unix 行为不变 |
| PR3 | `where.exe` 输出多行 / 含 `\r\n` / 含重复路径，解析出错或拉起错误 shell | survey detect.rs `command_exists` Windows 缺口 | `where.exe` 输出按行 trim + 取首个存在的可执行；Windows 路径 round-trip 单元测试；`cmd.exe` 永远保底，绝不拉起不存在的 shell |
| PR4 | `detect_process_cwd` Windows 返回的兜底 cwd 不精确，工作目录追踪降级 | PRD §Open Questions OQ3 | MVP 明确用 spawn-time 缓存 `initial_cwd` 兜底（已存在的 field），不做精确 API 查询；不 panic、安全返回缓存值 |

## 8. 阶段级 Definition of Done

- [ ] Task 2.1 / 2.2 均 Status = Done，各自 §7 追踪表所有行 Status = Done（无残留 Not Started / Verified 中间态）。
- [ ] §6 阶段级验收标准 P2-A ~ P2-E 全部满足（含端到端 smoke 双平台记录）。
- [ ] Windows：`cargo check --workspace` / `cargo build --workspace` / `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` 全绿（raw output 留痕）。
- [ ] macOS / Linux 回归：全量 `cargo test --workspace` 仍 100% 绿（反指标硬约束）。
- [ ] 所有 Windows 改动经 `#[cfg(target_os)]` / `#[cfg(windows)]` / `#[cfg(unix)]` 分支落地，Unix PTY 的信号 / 前台进程组 / cwd 检测 / 非阻塞 fd 行为零回归。
- [ ] 关联 ADR（[adr-001](../../decisions/adr-001-pty-windows-cfg-separation.md) PTY cfg 分离 + ConPTY、[adr-003](../../decisions/adr-003-windows-default-shell-probe-chain.md) shell 探测链、[adr-005](../../decisions/adr-005-windows-test-gating-strategy.md) 测试门控）状态为 Accepted 且实现与决策一致。
- [ ] 两 task §10 Completion Notes 已按 standard §8.3 六项 schema 回填。
