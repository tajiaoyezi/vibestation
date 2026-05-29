# Task `2.1`: `windows-shell-detection`

> Task Spec · 按 S2V standard §8.3 模板渲染。无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研证据（`spike-tmp/win-survey.json`）+ 实际 `crates/core/src/pty.rs` 源码填实，非编造。

**Status**: Done

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`（standard §10.5.1）。

**Priority**: P0
**Owner**: 主 agent
**Related Phase**: Phase 2 `shell-runtime`（[`../phases/phase-2-shell-runtime.md`](../phases/phase-2-shell-runtime.md)）
**Dependencies**: 依赖 1.1（`pty.rs` 编译期 `#[cfg(target_os)]` 平台分离已落地，Windows 上 `cargo check` 已能编过 `pty.rs`）

## 1. Background

`crates/core/src/pty.rs` 的 shell 探测全套假设 Unix：`default_shell_path()` 只区分 macOS（`/bin/zsh`）/ else（`/bin/bash`），Windows 上返回不存在的 `/bin/bash`；`list_available_shells()` 读 `/etc/shells`（Windows 没有此文件）；`resolve_shell` / `find_available_shell` 经 PATH 探测时依赖的 `is_executable_file` 用 Unix mode bits（`0o111`）判可执行；`detect_process_cwd` 对 Windows 走 `#[cfg(not(any(linux, macos)))]` 返回 `None`。结果：即便 Phase 1 让 Windows 编译过，新建 Tab 时 shell 解析要么返回空、要么拿到 `/bin/bash` 这类不存在的路径，PTY 拉不起来。

本 task 给这套 shell 探测加 Windows 分支，让 Windows 能解析出真实可拉起的 shell（探测链 `pwsh.exe → powershell.exe → cmd.exe`），为 task 2.2 的 ConPTY spawn 集成测试提供"有 shell 可拉"的前置。

## 2. Goal

任务完成后应该成立的事实：

- Windows 上 `default_shell_path()` 返回探测链首个可用 shell 的全路径（装了 pwsh → `pwsh.exe`，否则 `powershell.exe`，最终保底 `cmd.exe`），绝不返回 Unix `/bin/*`。
- Windows 上 `list_available_shells()` 经 PATH / `where.exe` 枚举出 `pwsh` / `powershell` / `cmd`（及 git-bash 若存在）的非空列表，不读 `/etc/shells`。
- Windows 上 `resolve_shell` / `find_available_shell` 经 `where.exe`（而非 `which`）在 PATH 中定位 shell 可执行；`is_executable_file` 在 Windows 按 `.exe`/`.bat`/`.cmd` 扩展名 + `is_file()` 判可执行（不查 mode bits）。
- Windows 上 `detect_process_cwd` 安全返回 spawn-time 缓存的 `initial_cwd` 兜底（不 panic、不做精确 API 查询，OQ3）。
- macOS / Linux 上以上全部行为**零变化**（Unix 分支不动）。

## 3. Scope

### In Scope

- `crates/core/src/pty.rs`：
  - `default_shell_path()` 加 `#[cfg(windows)]` 探测链分支（返回 `pwsh.exe → powershell.exe → cmd.exe` 首个可用全路径）。
  - `list_available_shells()` 加 `#[cfg(windows)]` 分支（PATH / `where.exe` 枚举，不读 `/etc/shells`）。
  - `resolve_shell` / `resolve_shell_in_path` / `find_available_shell`：Windows 用 `where.exe` + 扩展名匹配。
  - `is_executable_file`：`#[cfg(windows)]` 按 `is_file()` + 可执行扩展名判定；`#[cfg(unix)]` 保留 mode bits。
  - `detect_process_cwd`：把现有 `#[cfg(not(any(linux, macos)))]` 兜底显式化为 Windows 缓存 `initial_cwd` 路径。
- `crates/core/src/pty.rs` 内嵌 `#[cfg(test)] mod tests`：新增 Windows-gated 单元测试（探测链 / 枚举 / 路径 round-trip）；现有引用 `std::os::unix::fs::PermissionsExt` 的测试 import 按平台 cfg-gate。

### Out Of Scope

- ConPTY spawn / 读写 / 退出检测集成测试（task 2.2）。
- `detect_process_cwd` 的 Windows 精确 API 查询（PRD §Out of Scope + OQ3，MVP 用缓存兜底）。
- Windows 注册表扫描 shell（MVP 只走 PATH / `where.exe`，注册表留后续）。
- signal 映射（task 2.2）。
- 外部终端检测的 `command_exists` `where.exe` 改造（task 3.1 `external_term`，文件域不同）。

## 4. Users / Actors

- **Windows 11 上的 AI-agent 开发者**：新建 Tab 时，本 task 的 shell 解析决定 PTY 拉起哪个 shell（看到 pwsh / cmd prompt）。
- **`crates/app` Tauri 启动层**：`default_shell_get` IPC 经 `resolve_default_shell` 间接调用本 task 的 `resolve_shell` / `find_available_shell`，须在 Windows 上不 panic。
- **windows-latest CI**：跑 `cargo test --workspace`，本 task 的 Windows-gated 单元测试在此执行。

## 5. Behavior Contract

### 5.1 Required Reading

- 上游 task spec：[`task-1.1-pty-platform-split.md`](./task-1.1-pty-platform-split.md)（`pty.rs` 平台分离前置 · `#[cfg]` reader/signal/fcntl 隔离）、[`task-1.2-home-dir-helper.md`](./task-1.2-home-dir-helper.md)（跨平台家目录助手 · cwd/PATH 解析基础）。
- 关联 ADR：[adr-003-windows-default-shell-probe-chain.md](../../decisions/adr-003-windows-default-shell-probe-chain.md)（探测链 `pwsh→powershell→cmd`）、[adr-001-pty-windows-cfg-separation.md](../../decisions/adr-001-pty-windows-cfg-separation.md)（cfg 分离原则）、[adr-005-windows-test-gating-strategy.md](../../decisions/adr-005-windows-test-gating-strategy.md)（测试门控）。
- BDD feature：[`test/features/pty.feature`](../../../test/features/pty.feature)（task 1.1 / 2.1 / 2.2 共用，shell 探测场景）。

### 5.2 Imports

真实 imports（Windows 分支新增的以 `#[cfg(windows)]` 门控；Unix 分支保留现状）：

```rust
// 现有（保留 · Unix 分支）
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;     // is_executable_file 的 mode bits（Unix）

// Windows 分支新增
#[cfg(windows)]
use std::process::Command;                  // 调 where.exe 枚举 PATH 中的 shell

// 测试模块（cfg-gate 已有的 Unix import）
#[cfg(all(test, unix))]
use std::os::unix::fs::PermissionsExt;
```

> 不引入新的 crate；`where.exe` 经 `std::process::Command` 调用（系统自带），shell 探测不依赖注册表 crate。

### 5.3 函数签名

据现有 `pty.rs` 源码 + survey `fix_sketch` 给出 Windows 适配后的真实签名骨架（公开签名不变，内部加 `#[cfg]` 分支）：

```rust
// 探测链：Windows 返回首个可用 shell 全路径；Unix 不变
// 注意：现状返回 &'static str，Windows 探测需返回 owned String（探测结果非静态）
//       → 统一改为返回 String（Unix 分支把字面量 .to_string()，调用方已多处 .to_string()）
fn default_shell_path() -> String {
    #[cfg(windows)]
    {
        // 依次探测 pwsh.exe → powershell.exe → cmd.exe，取首个 where.exe 能定位的全路径；
        // 全找不到时保底返回 "cmd.exe"（系统永远存在）
        for candidate in ["pwsh.exe", "powershell.exe", "cmd.exe"] {
            if let Some(path) = resolve_shell(candidate) {
                return path.to_string_lossy().into_owned();
            }
        }
        "cmd.exe".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "/bin/zsh".to_string()
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "/bin/bash".to_string()
    }
}

// 枚举可用 shell：Windows 走 where.exe / PATH，不读 /etc/shells
pub fn list_available_shells() -> Vec<ShellInfo> {
    #[cfg(windows)]
    {
        // 依次对 ["pwsh.exe", "powershell.exe", "cmd.exe", "bash.exe"(git-bash)]
        // 调 resolve_shell；命中者 push ShellInfo{ path: 全路径, label: basename 去 .exe }；
        // dedup；列表为空时保底 push cmd.exe
        windows_list_shells()
    }
    #[cfg(unix)]
    {
        unix_list_shells_from_etc_shells()   // 现有 /etc/shells 逻辑原样迁入
    }
}

// PATH 解析：Windows 经 where.exe，Unix 经 split_paths + is_executable_file
pub(crate) fn resolve_shell(shell: &str) -> Option<PathBuf> {
    let path = Path::new(shell);
    if path.components().count() > 1 {
        return is_executable_file(path).then(|| path.to_path_buf());
    }
    #[cfg(windows)]
    {
        resolve_shell_via_where(shell)       // 调 `where.exe <shell>`，取首行存在的可执行
    }
    #[cfg(unix)]
    {
        resolve_shell_in_path(shell, std::env::var_os("PATH").as_deref())
    }
}

// 可执行判定：Windows 按扩展名 + is_file()，Unix 按 mode bits
fn is_executable_file(path: &Path) -> bool {
    #[cfg(windows)]
    {
        // .exe/.bat/.cmd/.com（不区分大小写）+ 文件存在；Windows 无 mode bits
        std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false)
            && has_windows_executable_ext(path)
    }
    #[cfg(unix)]
    {
        std::fs::metadata(path)
            .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
}

// cwd 检测：Windows 显式兜底缓存 initial_cwd（OQ3 · 不做精确查询）
fn detect_process_cwd(process_id: u32) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    { std::fs::read_link(format!("/proc/{process_id}/cwd")).ok() }
    #[cfg(target_os = "macos")]
    { /* 现有 lsof 逻辑不变 */ unimplemented!() }
    #[cfg(windows)]
    {
        // MVP：不查 Windows API；返回 None，由调用方 working_directory() 回落到
        // 已缓存的 self.initial_cwd（spawn 时 normalize_cwd 写入 · 见 PtySession.initial_cwd）
        let _ = process_id;
        None
    }
}

// Windows 私有 helper（本 task 新增）
#[cfg(windows)]
fn resolve_shell_via_where(shell: &str) -> Option<PathBuf>;     // 调 where.exe · 解析多行输出取首个存在项
#[cfg(windows)]
fn windows_list_shells() -> Vec<ShellInfo>;                     // 枚举 pwsh/powershell/cmd/git-bash
#[cfg(windows)]
fn has_windows_executable_ext(path: &Path) -> bool;             // 判 .exe/.bat/.cmd/.com
```

> `default_shell_path()` 由 `&'static str` 改为 `String` 是本 task 唯一的签名形变（Windows 探测结果非静态）；调用点（`find_available_shell` / `resolve_default_shell` / `effective_shell_for_spawn`）当前已多处 `.to_string()` / 比较，改动为局部，Unix 行为不变。

## 6. Acceptance Criteria

<!-- 模式 A：完整给值 + PRD 引用。review 通过无需删本注释。 -->

- [x] **AC1** (PRD §Core Capabilities 2 · §User Flow 主流程 2): Windows 上 `default_shell_path()` 返回探测链首个可用 shell 全路径——装了 pwsh 时为 `pwsh.exe` 的全路径，否则 `powershell.exe`，二者都缺时保底 `cmd.exe`；返回值绝不为任何 `/bin/*` Unix 路径。✅ TEST-2.1.1 pass（本机装 pwsh 7.6 → 优先返回 pwsh.exe 全路径）。
- [x] **AC2** (PRD §User Flow 异常流「未装 PowerShell 7」): Windows 上即使 `pwsh.exe` 与 `powershell.exe` 都不在 PATH，`default_shell_path()` / `resolve_default_shell(None)` 仍返回 `cmd.exe` 且不 panic（绝不拉起不存在的 `/bin/bash`）。✅ TEST-2.1.2 pass（`windows_cmd_fallback` 验 basename=cmd.exe · resolve_default_shell(None) 不返回 Unix 路径）。
- [x] **AC3** (PRD §Core Capabilities 2 · §Success Metrics 次要指标): Windows 上 `list_available_shells()` 返回非空 `Vec<ShellInfo>`，含已安装的 `pwsh` / `powershell` / `cmd`（及 git-bash 若存在），且不含任何 `/bin/*` / `/etc/shells` 来源项。✅ TEST-2.1.3 pass。
- [x] **AC4** (PRD §Decisions Log D3 · 本 task 推导): Windows 上 `resolve_shell("pwsh.exe")` 经 `where.exe` 在 PATH 命中时返回 `Some(全路径)`；裸名解析不依赖 Unix `which`；`is_executable_file` 对 `.exe`/`.bat`/`.cmd` 返回 true、对无可执行扩展名的普通文件返回 false。✅ TEST-2.1.4 pass（含大小写 + 含空格路径 round-trip）。
- [x] **AC5** (PRD §Open Questions OQ3 · §Out of Scope): Windows 上 `detect_process_cwd(pid)` 安全返回 `None`（不 panic），`working_directory()` 因此回落到 spawn-time 缓存的 `initial_cwd`，返回非空有效路径。✅ TEST-2.1.5 pass（spawn cmd.exe → working_directory 回落有效 cwd）。
- [x] **AC6** (PRD §Anti-metrics · §Compatibility requirements): macOS / Linux 上 `default_shell_path` / `list_available_shells`（`/etc/shells`）/ `resolve_shell`（PATH + mode bits）/ `detect_process_cwd`（/proc、lsof）行为零回归——既有 Unix 单元测试全绿。✅ TEST-2.1.6（`#[cfg(unix)]`）+ 既有 Unix 测试均保留；Unix 分支字面量/逻辑零改动（仅 `default_shell_path` 返回类型 `&'static str → String`，值不变）· 待 CI 矩阵 macOS/Linux runner 实跑确认。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 探测链首个可用 shell | SCEN-2.1.1 | TEST-2.1.1 `test_2_1_1_default_shell_probe_chain_picks_pwsh` (`#[cfg(windows)]`) | N/A（单元覆盖） | cargo test --workspace | Done |
| AC2 全缺保底 cmd.exe 不 panic | SCEN-2.1.2 | TEST-2.1.2 `test_2_1_2_default_shell_falls_back_to_cmd` (`#[cfg(windows)]`) | N/A | cargo test --workspace | Done |
| AC3 list_available_shells 非空且无 Unix 路径 | SCEN-2.1.3 | TEST-2.1.3 `test_2_1_3_list_available_shells_windows_no_unix_paths` (`#[cfg(windows)]`) | N/A | cargo test --workspace | Done |
| AC4 where.exe 解析 + 扩展名可执行判定 | SCEN-2.1.4 | TEST-2.1.4 `test_2_1_4_resolve_shell_via_where_and_exe_ext` (`#[cfg(windows)]`) | N/A | cargo test --workspace | Done |
| AC5 detect_process_cwd 兜底缓存 cwd | SCEN-2.1.5 | TEST-2.1.5 `test_2_1_5_detect_process_cwd_windows_falls_back_to_initial_cwd` (`#[cfg(windows)]`) | N/A | cargo test --workspace | Done |
| AC6 mac/Linux 零回归 | SCEN-2.1.6 | TEST-2.1.6 `test_2_1_6_unix_shell_detection_unchanged` (`#[cfg(unix)]`，复用既有 /etc/shells + mode bits 断言) | N/A | cargo test --workspace（macOS/Linux runner） | Done（Windows 本机不执行 · cfg(unix) gate · 待 CI 矩阵 macOS/Linux runner 跑） |

## 8. Risks

- **R-2.1-a**（关联 PRD §Technical Risks R3 mac/Linux 回归）：`default_shell_path` 返回值由 `&'static str` 改 `String` 触及多个调用点，可能误改 Unix 比较逻辑（如 `effective_shell_for_spawn` 中 `requested == default_shell_path()` 比较语义）。缓解：改动局部、Unix 分支字面量原样、`#[cfg(unix)]` 既有测试全绿锁定。
- **R-2.1-b**（关联 PRD §Technical Risks R4 路径/编码）：`where.exe` 输出多行 / 含 `\r\n` / 含重复 / 含空格路径，解析取错项。缓解：按行 trim、取首个 `is_executable_file` 为 true 的项；补 round-trip 单元测试（含空格路径 fixture）。
- **R-2.1-c**（关联 PRD §Open Questions OQ3）：缓存 `initial_cwd` 兜底使 Windows 工作目录追踪不随 `cd` 更新（功能降级而非崩溃）。缓解：MVP 接受（PRD 明确 Out of Scope），`detect_process_cwd` 返回 `None` 安全回落，文档化于 §10 剩余风险。

## 9. Verification Plan

> Rust task，命令对齐 adapter §Commands（Rust 主槽位）。Windows 改动的实施与验证在 Windows 11 本机；mac/Linux 回归由 CI 矩阵 / reviewer 保证。

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace（或 scoped：`cargo test -p vibestation_core pty::tests`）
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings

## 10. Completion Notes

- **完成日期**：2026-05-29
- **改动文件**：
  - `crates/core/src/pty.rs`（修改 · Windows shell 探测分支 + helper + cfg-gate 测试）
    - `default_shell_path()` 签名 `&'static str → String` + Windows 探测链分支
    - 新增 helper：`windows_cmd_fallback` / `windows_list_shells` / `windows_shell_label` / `resolve_shell_via_where` / `has_windows_executable_ext`（全 `#[cfg(windows)]`）
    - `list_available_shells` 拆 `unix_list_shells_from_etc_shells`（Unix 原逻辑迁入）+ Windows 分支
    - `resolve_shell` / `is_executable_file` 加 `#[cfg(windows)]` 分支
    - `PRIMARY_SHELL_BASENAMES` / `use std::ffi::OsStr` 收敛为 `#[cfg(unix)]`（clippy dead_code/unused-import）
- **commit 列表**：
  - `c60329e` test(pty): 加 SCEN-2.1.1~2.1.6 共 6 个 Windows shell 探测测试 + §5.3 骨架（RED · TEST-2.1.1~5 全 FAIL）
  - `366596b` feat(pty): 实现 Windows shell 探测链 + where.exe 枚举通过全部测试（GREEN）
  - refactor：无（实现已 clean · 无独立 refactor commit）
- **§9 Verification 结果**（Windows 11 本机 · 2026-05-29）：
  - install: 未跑（纯 Rust 改动 · 无 pnpm 依赖变化 · 不触前端）
  - typecheck（`cargo check --workspace`）: 0 error（仅 vibestation-app 2 个 pre-existing warning）
  - unit-test（`cargo test -p vibestation-core --lib pty::tests`）: 38 passed / 0 failed（含 TEST-2.1.1~2.1.5 Windows · TEST-2.1.6 为 `#[cfg(unix)]` 不在本机执行）
  - build（`cargo build --workspace`）: 0 error
  - lint（`cargo clippy -p vibestation-core --lib -- -D warnings`）: pty.rs 0 warning（`fs_watch.rs` / `external_term/detect.rs` 既有 warning 属 task 3.4 / 3.1 · 非本 task）
- **剩余风险 / 未做项**：
  - R-2.1-c（OQ3）：Windows `detect_process_cwd` 仍返回 `None` · 工作目录不随子进程 `cd` 更新（缓存 `initial_cwd` 兜底 · MVP 接受 · 精确 Windows API 查询留后续）。
  - `pty_pool::tests` 13 个 Windows ConPTY 运行期测试 timeout（pre-existing · RED-parent 基线同样失败 · 非本 task 引入）· 归 task-2.2 ConPTY spawn/read/exit 收口。
  - AC6 mac/Linux 零回归靠 `#[cfg(unix)]` 断言 + 改动局部性保证 · Windows 本机无法执行 Unix 分支 · 待 task-5.2 CI 矩阵 macOS/Linux runner 实跑最终确认。
- **下游 task 影响**：task 2.2（conpty-spawn-io）现可在 Windows 经 `resolve_shell("cmd.exe")` / `default_shell_path()` 解析出可拉起的 shell（探测链 + where.exe 已就位）· 解锁 ConPTY spawn 集成测试前置。
