# Task `6.1`: `windows-test-gating`

> Task Spec（S2V §8.3）· Windows 适配工作流 `feat/windows-support`。
> 本 task 把现存 Unix-only 行为测试按 `#[cfg(target_os)]` / `#[cfg_attr(windows, ignore)]` 门控，并按需补 Windows 专属测试，使 `cargo test --workspace` 在 Windows 显示 `ignored` 而非 `panicked`。

**Status**: Done

> 无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研（`spike-tmp/win-survey.json`）+ 实际测试源码填实，非编造。

**Priority**: P1
**Owner**: 主 agent（solo · 调度 subagent 实施）
**Related Phase**: Phase 6 · integration-matrix
**Dependencies**: Phase 2.x（shell-runtime · ConPTY spawn/IO）+ Phase 3.x（terminal-integration · fs_watch / external_term / config_import）—— 待门控的测试覆盖这些子系统行为，须先有对应生产分支落地

## 1. Background

Vibestation 测试套件大量硬编码 Unix 假设：`crates/app/tests/shell_compat.rs` 用 `which` 命令 + `/bin/zsh` 等绝对路径定位 shell；`crates/core/tests/git_ops_integration.rs` 用 `std::os::unix::fs::PermissionsExt::set_mode(0o755)` + `#!/bin/sh` 脚本造 pre-commit hook；`pty_pool_bench.rs` 硬编码 `/tmp` 作 cwd、回落 `HOME` / `/bin/sh`；`pty_scrollback_integration.rs` 硬编码 shell 参数 `/bin/sh`；`external_term/env_filter.rs` 的 `#[cfg(test)] mod tests` fixture 用 `/usr/bin` / `/bin/zsh`。

这些测试在 Windows 上要么编译失败（`std::os::unix`），要么运行时 `panicked`（找不到 `/bin/sh`、`which` 缺失、`/tmp` 不存在）。PRD 决策 D5（ADR-005）钦定门控策略：Unix-only 行为用 `#[cfg(unix)]` 或 `#[cfg_attr(windows, ignore="...")]`，并补 Windows 专属测试；CI windows-latest 跑 `cargo test --workspace` 自动跳过。本 task 落地这套门控，让 Windows 测试运行不再崩溃，同时 mac/Linux 测试行为零回归。

## 2. Goal

任务完成后：

- Windows 上 `cargo test --workspace` 0 编译错误、0 `panicked`；现存 Unix-only 测试显示为 `ignored`（带明确根因注释）。
- mac/Linux 上同一批测试行为完全不变（既有的 `#[cfg_attr(target_os = "linux", ignore)]` epoll-timing 标记保留，新增门控只对 Windows 生效）。
- 至少补一条 Windows 专属测试（如 Windows shell 候选探测 / `temp_dir()` 可用性），让 Windows 不止是"全 skip"而有真实正向覆盖。

## 3. Scope

### In Scope

- `crates/app/tests/shell_compat.rs`：`locate_shell()`（line 52-77，用 `which` + `/bin/zsh` 等候选路径）→ `#[cfg(unix)]` 分支保留，Windows 走 `where` / PATH 探测或整组用例 `#[cfg_attr(windows, ignore="PTY shell 矩阵在 Windows 走 Task 2.x ConPTY 探测链 · 此处 Unix shell 候选不适用")]`。
- `crates/core/tests/git_ops_integration.rs`：`use std::os::unix::fs::PermissionsExt`（line 19）+ `create_pre_commit_hook()`（line 45-57，`chmod 0o755` + `#!/bin/sh`）→ `#[cfg(unix)]` 包裹 helper 与依赖它的测试；非 hook 类 commit 测试（`test_commit_single_file` 等）保持跨平台。
- `crates/core/tests/pty_pool_bench.rs`：`user_home()`（line 25-29，回落 `PathBuf::from("/")`）+ `user_shell()`（line 31-33，回落 `/bin/sh`）+ `/tmp` 硬编码 cwd（line 106/161/207）→ `std::env::temp_dir()` 跨平台、`USERPROFILE` 兜底；benchmark 本身仍 `#[ignore]`（既有手动触发）。
- `crates/core/tests/pty_scrollback_integration.rs`：`/bin/sh` shell 参数（line 68）→ cfg 分离或调用默认 shell 探测；保留既有 `#[cfg_attr(target_os = "linux", ignore)]`。
- `crates/core/src/external_term/env_filter.rs` 的 `#[cfg(test)] mod tests`（line 105+）：fixture `("PATH", "/usr/bin")` / `("SHELL", "/bin/zsh")`（line 119/123/202/206）→ Windows cfg 分支用 `C:\Windows\System32` / `COMSPEC=cmd.exe`。
- 新增 ≥ 1 条 Windows 专属正向测试（`#[cfg(target_os = "windows")]`）。

### Out Of Scope

- 生产代码 Windows 分支实现（PTY ConPTY / shell 探测链 / fs_watch backend）——由 Phase 1-5 落地，本 task 只门控测试。
- 给 Windows 实装可执行的 git hook 生产逻辑（PRD §Core Capabilities Out of Scope：`#!/bin/sh` hook 生产逻辑深度 Windows 化不在范围；本 task 只处理测试层 cfg）。
- `detect_process_cwd` 的 Windows 精确实现（PRD Out of Scope · 缓存 `initial_cwd` 兜底）。
- 端到端 GUI smoke / 三平台矩阵校验（Task 6.2 负责）。

## 4. Users / Actors

- **windows-latest CI runner**：跑 `cargo test --workspace`，期望 Unix-only 测试 `ignored` 而非 `panicked`，使 CI 不因 Unix 硬假设红。
- **项目 Windows 贡献者 / 主 agent（本机 H:\ Windows 11）**：本机 `cargo test --workspace` 验证改动，看到干净的 pass + ignored 输出。
- **macOS / Ubuntu reviewer / CI 矩阵**：跑同批测试，验证门控未改变 Unix 测试行为（零回归）。

## 5. Behavior Contract

测试门控不改变任何被测生产行为：mac/Linux 上被门控的测试照常执行并断言（Unix 真测保留），Windows 上要么执行 Windows 等价断言（专属测试），要么显式 `ignored` 并带根因。`cargo test --workspace` 在三平台都不出现编译失败 / `panicked`。

### 5.1 Required Reading

- [`docs/specs/phases/phase-6-integration-matrix.md`](../phases/phase-6-integration-matrix.md)（本 Phase §3 涉及模块 + §6 端到端 smoke）
- 上游 task spec：`docs/specs/tasks/task-2.1-windows-shell-detection.md` + `task-2.2-conpty-spawn-io.md`（PTY/shell Windows 探测链 · 被门控测试覆盖其行为）；`task-3.4-fs-watch-windows.md`（fs_watch Windows backend）
- ADR：[`docs/decisions/adr-005-windows-test-gating-strategy.md`](../../decisions/adr-005-windows-test-gating-strategy.md)（cfg + ignore + 专测策略 · 本 task 落地依据）；[`docs/decisions/adr-001-pty-windows-cfg-separation.md`](../../decisions/adr-001-pty-windows-cfg-separation.md)
- BDD：[`test/features/build-and-integration.feature`](../../../test/features/build-and-integration.feature)
- 实际源码：`crates/app/tests/shell_compat.rs` · `crates/core/tests/git_ops_integration.rs` · `crates/core/tests/pty_pool_bench.rs` · `crates/core/tests/pty_scrollback_integration.rs` · `crates/core/src/external_term/env_filter.rs`

### 5.2 Imports

测试文件层（无新增生产依赖）：

```rust
// git_ops_integration.rs — Unix-only import 门控
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// pty_pool_bench.rs — 跨平台临时目录 / 家目录
use std::env;            // env::temp_dir() · env::var("USERPROFILE")/var("HOME")
use std::path::PathBuf;

// 各测试文件已有：tempfile::TempDir / vibestation_core::{PtyManager, ...} / crossbeam_channel
```

不引入新的第三方 crate（门控纯靠 `#[cfg]` / `#[cfg_attr]` + std）。

### 5.3 函数签名

> 据实际测试源码当前签名 + Windows 适配后骨架。所有改动走 cfg 分支，mac/Linux 路径不变。

```rust
// ── crates/app/tests/shell_compat.rs ──
// 当前（line 52）：fn locate_shell(name: &str) -> Option<PathBuf>  // 用 which + /bin/zsh 候选
// 适配后：Unix 路径不变；整组 shell_compat 用例对 Windows ignore
#[cfg_attr(
    target_os = "windows",
    ignore = "PTY shell 矩阵在 Windows 走 Task 2.x ConPTY 探测链 · Unix shell 候选(/bin/zsh + which)不适用"
)]
#[cfg_attr(target_os = "linux", ignore = "Linux PTY epoll close event timing 不稳定 · 沿袭 PR #82")]
fn zsh_01_startup_shows_prompt() { /* Unix 逻辑不变 */ }

// ── crates/core/tests/git_ops_integration.rs ──
// 当前（line 45）：fn create_pre_commit_hook(repo_path: &Path, exit_code: i32, stderr_msg: &str)
//                  // chmod 0o755 + #!/bin/sh
#[cfg(unix)]
fn create_pre_commit_hook(repo_path: &Path, exit_code: i32, stderr_msg: &str) { /* 不变 */ }

#[cfg(unix)]
#[test]
fn test_commit_fails_when_hook_rejects() { /* 依赖 hook · Unix-only */ }
// 非 hook 类 commit 测试（test_commit_single_file 等）保持跨平台 · 不加 cfg

// ── crates/core/tests/pty_pool_bench.rs ──
// 当前（line 25）：fn user_home() -> PathBuf { env::var("HOME")...unwrap_or(PathBuf::from("/")) }
// 当前（line 31）：fn user_shell() -> String { env::var("SHELL")...unwrap_or("/bin/sh") }
fn user_home() -> PathBuf {
    #[cfg(windows)]
    let key = "USERPROFILE";
    #[cfg(unix)]
    let key = "HOME";
    std::env::var(key)
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}
fn bench_cwd() -> PathBuf { std::env::temp_dir() }   // 取代硬编码 "/tmp"

// ── crates/core/src/external_term/env_filter.rs · #[cfg(test)] mod tests ──
// 当前 fixture（line 119/123）：("PATH", "/usr/bin"), ("SHELL", "/bin/zsh")
#[cfg(test)]
mod tests {
    #[cfg(unix)]
    const SAMPLE_PATH: &str = "/usr/bin";
    #[cfg(windows)]
    const SAMPLE_PATH: &str = r"C:\Windows\System32";
    // SHELL(Unix) vs COMSPEC=cmd.exe(Windows) fixture 同理 cfg 分离
}

// ── 新增 Windows 专属正向测试（任一文件，建议 pty_pool_bench / shell_compat）──
#[cfg(target_os = "windows")]
#[test]
fn test_6_1_1_windows_temp_dir_and_userprofile_resolve() {
    // 验证 env::temp_dir() 非空 + user_home() 落在用户目录而非 "/"
}
```

## 6. Acceptance Criteria

- [x] **AC1** (PRD §Decisions Log D5 / §Success Metrics 主要指标): Windows 上 `cargo test --workspace` 0 编译错误、0 `panicked`，现存 Unix-only 测试显示 `ignored`（每条带明确根因注释）。✅ 所有 target `test result: ok` · 40 ignored 带根因。
- [x] **AC2** (PRD §Success Metrics 反指标): mac/Linux 上被门控测试行为零回归——`cargo test --workspace` 仍 100% 绿，原 `#[cfg_attr(target_os = "linux", ignore)]` epoll-timing 标记保留，门控只对 Windows 新增生效。✅ 全部 Windows 门控为新增 `#[cfg_attr(windows, ignore)]` / `#[cfg(unix)]` / `#[cfg(any(macos,linux))]` 叠加 · 未删任何 Unix 断言；mac/Linux 复跑 defer reviewer / CI matrix（本机 Windows 跑不了）。
- [x] **AC3** (PRD §Decisions Log D5): `git_ops_integration.rs` 的 `use std::os::unix::fs::PermissionsExt` + `create_pre_commit_hook`（`chmod 0o755` + `#!/bin/sh`）走 `#[cfg(unix)]`，Windows 上不编译该 Unix-only 路径；非 hook commit 测试保持跨平台。✅ Windows `git_ops_integration` 12 passed（非 hook 测试照跑）。
- [x] **AC4** (PRD §User Flow 异常流 / §Technical Risks R4): `pty_pool_bench.rs` 的 `/tmp` 硬编码改为 `std::env::temp_dir()`、`HOME` 回落改为 `USERPROFILE`（Windows）/`HOME`（Unix）兜底到 `temp_dir()`，Windows 上不再 `panicked` on missing `/tmp`。✅ + `user_shell` COMSPEC 兜底。
- [x] **AC5** (本 task 新增): 至少补一条 `#[cfg(target_os = "windows")]` 正向测试（如 `temp_dir()` / `USERPROFILE` 解析），让 Windows 有真实正向覆盖而非全 skip。✅ `shell_compat.rs::test_6_1_1_windows_temp_dir_and_userprofile_resolve`（Windows 1 passed）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1: Windows test 0 panicked · Unix-only ignored | SCEN-6.1.1 | TEST-6.1.1 `test_6_1_1_windows_temp_dir_and_userprofile_resolve` + Windows `cargo test --workspace` 输出无 panicked | N/A（cargo test 即集成）| `cargo test --workspace`（Windows · 断言 0 panicked + ignored>0）| Done |
| AC2: mac/Linux 零回归 | SCEN-6.1.2 | TEST-6.1.2 既有 Unix 测试集（`zsh_01_*` / `stdout_is_persisted_*` 等）执行通过 | N/A | `cargo test --workspace`（macOS + Ubuntu · 与基线比对）| Done（Windows 侧门控不破 Unix · mac/Linux 复跑 defer reviewer/CI matrix）|
| AC3: hook 测试 `#[cfg(unix)]` 门控 | SCEN-6.1.3 | TEST-6.1.3 `create_pre_commit_hook` + 依赖测试 Unix-only 编译 | N/A | `cargo check --workspace`（Windows · 不编译 unix path）+ `cargo test`（Unix · hook 测试仍跑）| Done |
| AC4: `/tmp`→`temp_dir()` · `HOME`→`USERPROFILE` 兜底 | SCEN-6.1.4 | TEST-6.1.4 `pty_pool_bench` `user_home()` / `bench_cwd()` 跨平台解析 | N/A | `cargo test --test pty_pool_bench -- --ignored`（Windows · 不 panic on `/tmp`）| Done |
| AC5: ≥1 Windows 专属正向测试 | SCEN-6.1.5 | TEST-6.1.5 `test_6_1_1_windows_temp_dir_and_userprofile_resolve` | N/A | `cargo test --workspace`（Windows · 该测试 passed）| Done |

## 8. Risks

- 门控写错把 Unix 路径也 skip → mac/Linux 覆盖静默缩水（关联 PRD §Technical Risks R3：mac/Linux 回归）。缓解：门控只对 Windows 加 cfg/ignore；mac/Linux 测试计数前后比对。
- `/tmp` → `temp_dir()` 改写在 Unix 下行为偏移（`temp_dir()` 在 Unix 通常是 `/tmp` 但不绝对）→ 个别 bench 假设漂移（关联 PRD R4：路径）。缓解：用 `std::env::temp_dir()` 跨平台 API，不假设具体路径字符串。
- `#[cfg_attr(windows, ignore)]` 标过宽，把本可在 Windows 跑的测试也 skip → Windows 覆盖不足。缓解：AC5 强制补 ≥1 Windows 正向测试；ignore 注释须写明"为何 Windows 不适用"。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile  <!-- 与 adapter §Commands Install 一致 -->
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制：含 crates/*/tests/ 集成测试一起跑；Windows 断言 0 panicked + ignored>0 -->
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

> 本 task 是 Rust 测试门控，无 E2E / Coverage（adapter Coverage = N/A）；端到端 GUI smoke 由 Task 6.2 承载，本 task §9 不列 runtime-smoke。三平台零回归校验：reviewer / CI 矩阵在 macOS + Ubuntu 复跑 `cargo test --workspace`。

## 10. Completion Notes

- **完成日期**：2026-05-29
- **改动文件**（全部测试层 + 必要的 fixture 跨平台化 · 无被测生产逻辑变更）：
  - `crates/core/tests/git_ops_integration.rs`：`PermissionsExt` import + `create_pre_commit_hook` + 2 hook 测试 → `#[cfg(unix)]`
  - `crates/app/src/fix_path_env.rs`：`#[cfg(test)] mod tests` 整模块 → `#[cfg(any(macos, linux))]`（修 7× E0425）
  - `crates/app/tests/shell_compat.rs`：7 个 spawn-based shell 用例叠加 `#[cfg_attr(windows, ignore)]` + 新增 `test_6_1_1_windows_temp_dir_and_userprofile_resolve`（AC5）
  - `crates/core/tests/pty_scrollback_integration.rs`：`/bin/sh` 用例叠加 `#[cfg_attr(windows, ignore)]`
  - `crates/core/src/pty_pool.rs`：13 个 spawn `/bin/sh` 预热池 mod 测试叠加 `#[cfg_attr(windows, ignore)]`
  - `crates/core/tests/pty_pool_bench.rs`：`/tmp`→`temp_dir()` · `HOME`→`USERPROFILE`/`HOME` 兜底 `temp_dir()` · `user_shell` COMSPEC 兜底（跨平台化）
  - `crates/core/src/git_sync.rs`：`credential_https_helper_path_is_attempted` 叠加 `#[cfg_attr(windows, ignore)]`（GCM 交互 hang 根因 · 经验式发现 · 不在原始清单）
  - `crates/core/src/rollback_ops.rs`：`GitFixture::new` 固定 `core.autocrlf=false`（修 Windows CRLF checkout 致 LF 断言失败 · 经验式发现 · 不在原始清单）
  - `crates/app/tests/title_bar.rs`：`assert!(cfg!(...))` → `const { assert!(..) }`（修 Windows clippy `assertions_on_constants`）
- **commit 列表**：`75d71fe` test(task-6.1): cfg-gate Unix-only 测试 + 补 Windows 专测
- **§9 Verification 结果**（Windows 11 本机 H:\ · 2026-05-29）：
  - install: N/A（纯 Rust 测试门控 · 无前端改动）
  - typecheck（`cargo check --workspace`）：0 error（含在 build / test 编译内）
  - unit-test（`cargo test --workspace`）：**所有 target `test result: ok` · 0 failed · 0 panicked · 0 hang**。聚合 ~1017 passed / 40 ignored：core lib 901 passed / 14 ignored（13 pty_pool spawn + 1 git_sync GCM）· shell_compat 1 passed / 22 ignored · pty_pool_bench 0 / 3 ignored · pty_scrollback 0 / 1 ignored · pty_windows_conpty 5 passed · git_ops_integration 12 passed
  - build（`cargo build --workspace`）：0 error
  - lint（`cargo clippy --workspace --all-targets -- -D warnings`）：0 error · 0 warning
- **剩余风险 / 未做项**：
  - 经验式发现两处原清单外 Windows 失败点并修复：(1) `git_sync` credential helper 在 Windows 触发 Git Credential Manager 交互提示导致全量 `cargo test` **永久 hang**（非 panic · 单线程才暴露）→ `#[cfg_attr(windows, ignore)]`；(2) `rollback_ops` git checkout CRLF 致 LF 断言 fail → fixture `core.autocrlf=false`。两者均测试/fixture 层 · 不改生产逻辑。
  - mac/Linux 零回归（AC2）本机（Windows）跑不了 → **defer reviewer / CI matrix（task-5.2 windows-latest + mac/Linux leg）复跑**与 Phase 前基线比对。门控全为新增 `#[cfg_attr(windows, ...)]` / `#[cfg(unix)]` / `#[cfg(any(macos,linux))]`，未删/改任何 Unix 跑的断言，零回归风险极低。
  - Windows ignore 技术债解除触发：PTY shell 矩阵随 ConPTY 探测链成熟可逐步从 ignore 解除（GA gate · ADR-005）；`git_sync` GCM ignore 解除需 CI 设 `GIT_CONFIG_NOSYSTEM` / 清 `credential.helper`（CI 默认无交互 helper 时本测可在 Windows CI leg 跑）。
  - `crates/app/tests/bundle_config.rs` 有 pre-existing rustfmt 漂移（非本 task 触碰 · 本 task 改动文件全部 `rustfmt --check` 通过）。
- **下游 task 影响**：task-6.2 在干净的 Windows `cargo test --workspace` 基线上做端到端 smoke + 三平台矩阵校验（依赖本 task Done）。
