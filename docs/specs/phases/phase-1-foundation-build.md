# Phase 1: foundation-build

**Status**: Done
**Phase ID**: 1
**Master Spec**: [`docs/prds/windows-support.prd.md`](../../prds/windows-support.prd.md)
**依赖**: -（首 phase · 解锁全部后续）
**可并行**: 否（基础设施 · 解锁全部后续 phase）

---

## 1. 阶段目标

让 Vibestation 在 Windows 11 (x64 MSVC) 上 `cargo build --workspace` / `cargo check --workspace` 编译通过，并就位两个跨平台基元：统一家目录解析 `home_dir()`、Windows 安全的默认 shell 占位值。当前 Windows 上 `crates/core/src/pty.rs` 因 `mio::unix::SourceFd` + `libc::fcntl/kill/tcgetpgrp` + `std::os::unix::fs::PermissionsExt` 直接编译失败，连跑都跑不起来——本 phase 把这些 Unix-only 内核按 `#[cfg(target_os)]` 分离，Windows 走 `portable-pty` ConPTY 阻塞读路径，从"编译红"翻到"编译绿"，是后续全部 phase（shell-runtime / terminal-integration / build-package-ci / integration-matrix）的硬前置。

本 phase 不追求 PTY 运行期行为正确（那是 Phase 2 shell-runtime 的目标），只追求**编译通过 + Unix 行为零回归 + 基元就位**。

## 2. 业务价值

- **解锁 Windows 贡献者**：Windows 本机 `cargo build/test` 不再 fatal，项目 Windows 贡献者首次能编译并跑测试（PRD §Users & Context 次要用户）。
- **解锁后续全部适配工作**：编译是一切的前提；Phase 2-6 全部依赖本 phase 的编译解锁（PRD §Implementation Phases 依赖列：Phase 2/5 显式 depends_on 1，Phase 3/4 在编译基线上工作）。
- **跨平台路径正确性**：`home_dir()` 助手消除 `std::env::var("HOME").unwrap_or("/")` 在 Windows 回落 `/` 导致 workspace init / config import scan / PTY pool refill 全拿错路径的根因（PRD §Problem Statement / §User Flow 异常流 2）。
- **默认 shell 不再拉起不存在的 `/bin/bash`**：`app_settings.rs` 默认值加 Windows 分支，给 Phase 2 探测链一个安全起点（PRD §Core Capabilities 2）。

## 3. 涉及模块

| 模块 | 文件 | 本 phase 改动性质 |
|---|---|---|
| pty | `crates/core/src/pty.rs` | Unix-only 内核 cfg 分离（mio/libc/signal/fcntl/PermissionsExt 置于 `#[cfg(unix)]`）+ 新增 `#[cfg(windows)]` ConPTY reader 路径骨架 |
| app-home | `crates/app/src/lib.rs` + 根 `Cargo.toml` / `crates/app/Cargo.toml` | 新增 `dirs` workspace 依赖 + 跨平台 `home_dir()` 助手，替换两处 `HOME` 硬编码 |
| app-settings | `crates/core/src/app_settings.rs` | `default_shell` 默认值（`impl Default` + `get_all` fallback）加 `#[cfg(target_os = "windows")]` 分支 |

> 关联 ADR：[adr-001](../../decisions/adr-001-pty-windows-cfg-separation.md)（PTY cfg 分离）· [adr-002](../../decisions/adr-002-cross-platform-home-dir-dirs.md)（dirs 家目录）· [adr-003](../../decisions/adr-003-windows-default-shell-probe-chain.md)（默认 shell）。

## 4. 任务清单

| Task | 模块 | Spec | 依赖 / 顺序 | 说明 |
|---|---|---|---|---|
| 1.1 | pty | [`../tasks/task-1.1-pty-platform-split.md`](../tasks/task-1.1-pty-platform-split.md) | Phase 1 首 · 解锁编译 | pty.rs Unix-only 内核 cfg 分离 + Windows ConPTY reader 路径骨架 |
| 1.2 | app-home | [`../tasks/task-1.2-home-dir-helper.md`](../tasks/task-1.2-home-dir-helper.md) | 可与 1.1 并行（不同文件域）| `dirs` 依赖 + `home_dir()` 助手，替换 lib.rs:336 / :637 HOME 硬编码 |
| 1.3 | app-settings | [`../tasks/task-1.3-shell-default-setting.md`](../tasks/task-1.3-shell-default-setting.md) | 依赖 1.1 编译 | `app_settings.rs` default_shell Windows 分支占位 |

## 5. 依赖关系

```text
Task 1.1 (pty cfg 分离 · 编译解锁) ──┬──> Task 1.3 (app_settings default_shell · 依赖编译能跑 workspace 测试)
                                     │
Task 1.2 (home_dir 助手 · 不同文件域) ┘ 可与 1.1 完全并行（lib.rs/Cargo.toml vs pty.rs 无交叠）
```

- **1.1 是编译解锁前置**：1.3 改 `crates/core` 同 crate，需 1.1 让 `crates/core` 在 Windows 编译通过才能跑 `cargo test --workspace` 验证；1.2 改 `crates/app` + `Cargo.toml`，文件域与 1.1 不交叠，可并行（但其 §9 验证仍受益于 1.1 编译解锁，故标"可并行"而非"零依赖独立交付"）。
- **上游**：无（本 phase 是依赖链根）。
- **下游**：Phase 2（shell-runtime · depends_on 1）· Phase 5（build-package-ci · depends_on 1,2）· Phase 3/4 在本 phase 编译基线上工作。

## 6. 阶段级验收标准

本 phase 完成后必须同时满足（每条可执行验证）：

- **编译绿（核心目标）**：Windows 11 x64 MSVC 上 `cargo check --workspace` + `cargo build --workspace` 0 错误（修复前 `pty.rs` 因 `mio::unix` / `libc::fcntl` 编译 fatal）。
- **Unix 零回归**：macOS + Ubuntu 上 `cargo test --workspace` 仍 100% 绿（所有改动走 `#[cfg(target_os)]` / `#[cfg(unix)]` 分支，Unix reader loop / 信号 / fcntl / PermissionsExt 路径字节级不变）。
- **基元就位**：Windows 上 `AppSettings::default().default_shell` 返回 Windows 占位 shell（非 `/bin/bash`）；`home_dir()` 在 Windows 解析到 `%USERPROFILE%`（非 `/`），Unix 仍解析 `$HOME`。
- **clippy 绿**：`cargo clippy --workspace --all-targets -- -D warnings` 在三平台 0 warning（cfg 分支不得引入 dead_code / unused_imports warning）。

### 端到端 smoke（C1 集成兜底）

phase 最后一个 task 完工前，本机 Windows 11 实跑以下 smoke 并记录结果：

- **windows**：`cargo build --workspace` 退出码 0 + 0 编译错误；`cargo test --workspace` 运行（Windows-gated 测试集自动跳过 Unix-only 项，不 panic、不 hang）；断言 `cargo test --workspace -p vibestation_core app_settings` 中 Windows default_shell 单测通过。
- **macos / ubuntu（回归兜底）**：`cargo test --workspace` 全绿，无新增 ignore 误伤既有 Unix 测试；`cargo clippy --workspace --all-targets -- -D warnings` 0 warning。
- **跨平台基元断言**：新增 `home_dir()` 单测在 Windows 断言返回非 `/` 路径、在 Unix 断言读 `$HOME`；`default_shell` 单测三平台各断言对应默认值。

## 7. 阶段级风险

| # | 风险 | 关联 PRD 风险 | 缓解 |
|---|---|---|---|
| R-P1 | cfg 分离写错把 Unix reader loop / 信号路径也改坏，mac/Linux 回归 | PRD §Technical Risks R3 | 严格 `#[cfg(unix)]` 包裹现有内核，Windows 走独立 `#[cfg(windows)]` 分支；TDD 先 RED 锁 Unix 行为不变；mac/Linux 全量 cargo test 本地+CI 必绿（反指标硬约束） |
| R-P2 | Windows ConPTY reader 骨架在本 phase 仅求编译通过，运行期退出检测/回显可能不正确 | PRD §Technical Risks R1 | 本 phase 范围仅"编译绿 + 骨架就位"，ConPTY 运行期正确性 defer 到 Phase 2 shell-runtime；骨架用 `child.try_wait()` + 阻塞读循环占位，标注 TODO 指向 Phase 2 |
| R-P3 | `dirs` crate 新增依赖引入 supply-chain / 锁文件冲突 | PRD §Technical Risks（依赖面）| `dirs` 纯 Rust 无 C 依赖、事实标准、最小（ADR-002）；`Cargo.lock` 同 PR 提交；不引入与现有锁定栈冲突的重依赖 |
| R-P4 | Windows `default_shell` 占位值与 Phase 2 探测链不一致，导致 Phase 2 返工 | PRD §Technical Risks（兼容性）| 占位用 `cmd.exe`（永远保底）+ 注释明确"Phase 2 resolve_default_shell 探测链细化为 pwsh→powershell→cmd"，与 ADR-003 探测链对齐 |

## 8. 阶段级 Definition of Done

- [ ] Task 1.1 / 1.2 / 1.3 全部 Status = Done，§10 Completion Notes 回填完整。
- [ ] §6 阶段级验收标准全部满足（编译绿 / Unix 零回归 / 基元就位 / clippy 绿）。
- [ ] §6 端到端 smoke 已在本机 Windows 11 实跑并记录（windows build+test + mac/ubuntu 回归兜底）。
- [ ] 所有 Windows 改动走 `#[cfg(target_os)]` / `#[cfg(unix)]` / `#[cfg(windows)]` 分支或运行期平台判断，无裸 Unix 路径泄漏到 Windows 编译单元。
- [ ] ADR-001 / ADR-002 / ADR-003 Status = Accepted 且与实现一致。
- [ ] 追踪表（各 task §7）所有 AC 行 Status = Done（或 Waived 并按 §12.3 登记）。
- [ ] Phase Status 由 Ready → Done（本 phase 收尾时翻转）。
