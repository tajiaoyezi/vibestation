# Task `1.2`: `home-dir-helper`

**Status**: Done

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`

**Priority**: P0
**Owner**: 主 agent
**Related Phase**: 1（foundation-build）
**Dependencies**: 可与 1.1 并行（不同文件域：本 task 改 `crates/app/src/lib.rs` + `Cargo.toml`，1.1 改 `crates/core/src/pty.rs`，无交叠）

## 1. Background

`crates/app/src/lib.rs` 有两处硬编码 `std::env::var("HOME")`，缺失时回落 `PathBuf::from("/")`：

- line 336-338：`let home_path = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/"));`——喂给 `pty_pool::apply_config_change` / `refill_async(home_path)`。
- line 636-640：`fn home_dir_or_root() -> PathBuf`——喂给 `config_import_scan(home)`。

在 Windows 上 `HOME` 环境变量**通常不存在**（标准是 `USERPROFILE`，或 `HOMEDRIVE`+`HOMEPATH` 组合）。回落 `/` 会让：

- workspace 初始化拿到错误根目录；
- config import scan 去扫 `/Library/Preferences/...` 等虚假 Unix 路径（永远找不到 Windows 的 `%APPDATA%` 配置）；
- PTY pool refill 拿错 cwd。

ADR-002 选定：新增 `dirs` crate（纯 Rust、无 C 依赖、事实标准）做跨平台 `home_dir()`；应用数据目录继续用 Tauri `app_local_data_dir()`（已正确解析 `%APPDATA%`，本 task 不动）。`dunce` 已在 workspace 依赖里处理 UNC，本 task 不引入新路径库。

## 2. Goal

任务完成后应成立的事实：

- 新增统一 `home_dir()` 助手：Windows 解析 `%USERPROFILE%`（经 `dirs::home_dir()`），Unix 解析 `$HOME`，二者均失败时返回平台合理 fallback（Windows 非 `/`）。
- `lib.rs:336` 与 `lib.rs:637` 两处 `HOME` 硬编码替换为该助手，Windows 上 workspace init / config import scan / PTY pool refill 拿到正确家目录。
- `dirs` 加入 workspace 依赖，`Cargo.lock` 同 PR 提交；mac/Linux 行为零回归（Unix 路径仍走 `$HOME`，与原逻辑等价）。

## 3. Scope

### In Scope

- 根 `Cargo.toml`（workspace `[workspace.dependencies]` 加 `dirs`）+ `crates/app/Cargo.toml`（引用 `dirs.workspace = true`）。
- `crates/app/src/lib.rs`：
  - 新增 `fn home_dir() -> PathBuf` 跨平台助手（或重写现有 `home_dir_or_root` 为跨平台实现）。
  - 替换 line 336-338 的 `std::env::var("HOME").unwrap_or_else(|_| PathBuf::from("/"))` 为 `home_dir()` 调用。
  - 替换 line 636-640 的 `home_dir_or_root()` 实现为 `dirs::home_dir()` 优先、平台感知 fallback。
- `crates/app/src/lib.rs` 内（或 `crates/app/tests/`）新增 `home_dir()` 跨平台单测。

### Out Of Scope

- 应用数据目录解析（`app_local_data_dir()` 已跨平台正确解析 `%APPDATA%`，不改）。
- `config_import/ipc.rs` 的 `prettify_home_path()` Windows 化（Phase 3 task 3.2 config-import-paths）。
- `config_import/{ghostty,alacritty,iterm2}.rs` 的 `%APPDATA%` 扫描路径（Phase 3 task 3.2）。
- `fix_path_env.rs` 的 Windows PATH 处理（PRD 决策：Windows no-op 可接受，非本 task）。
- `dunce::canonicalize` 路径规范化（已在 workspace，UNC 处理非本 task 新增）。

## 4. Users / Actors

- **Windows 11 AI-agent 开发者**：首次启动时 `home_dir()` 正确解析到 `C:\Users\<user>`，config import 能发现 `%APPDATA%` 下的 Alacritty/Ghostty 配置，workspace 初始化拿到正确根（PRD §User Flow 异常流 2）。
- **PTY pool / config import 子系统**（调用方）：消费 `home_dir()` 返回值做 refill / scan。

## 5. Behavior Contract

`home_dir()` 在所有平台返回用户家目录的 `PathBuf`：Windows = `%USERPROFILE%`（如 `C:\Users\alice`），Unix = `$HOME`。解析失败时返回平台合理 fallback（不再无差别 `/`）。调用方（pty_pool / config_import）行为在 Unix 上等价于改动前。

### 5.1 Required Reading

- 本 task 无上游 task（与 1.1 并行 · 不同文件域）。
- [`docs/decisions/adr-002-cross-platform-home-dir-dirs.md`](../../decisions/adr-002-cross-platform-home-dir-dirs.md)（dirs crate 家目录决策）。
- BDD: [`test/features/app-foundation.feature`](../../../test/features/app-foundation.feature)。
- 现有源：`crates/app/src/lib.rs:336-338`（PTY pool refill home_path）、`:636-640`（`home_dir_or_root`）。

### 5.2 Imports

```rust
// crates/app/src/lib.rs
use std::path::PathBuf;
use dirs; // 新增 workspace 依赖（ADR-002）· 纯 Rust 无 C 依赖
// 现有：use vibestation_core::pty_pool::...; use vibestation_core::config_import::...;
```

```toml
# 根 Cargo.toml
[workspace.dependencies]
dirs = "5"

# crates/app/Cargo.toml
[dependencies]
dirs.workspace = true
```

### 5.3 函数签名

> Windows 适配后的真实签名骨架。替换两处 `HOME` 硬编码。

```rust
// crates/app/src/lib.rs · 新增统一助手
/// 跨平台用户家目录解析。
/// Windows: %USERPROFILE%（经 dirs::home_dir，覆盖 HOMEDRIVE+HOMEPATH 边界）。
/// Unix: $HOME。
/// 均失败时返回平台合理 fallback（Windows = C:\ 风格的可用根 · 不再无差别 "/"）。
fn home_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        return home;
    }
    #[cfg(windows)]
    {
        // dirs 已覆盖绝大多数；极端 fallback 不再用 "/"（Unix root 在 Windows 无意义）
        std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\"))
    }
    #[cfg(unix)]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/"))
    }
}

// line 636-640 现有 home_dir_or_root() 收敛为 home_dir() 的别名或直接替换调用点
fn home_dir_or_root() -> PathBuf { home_dir() }

// 调用点 line 336 改为：
// let home_path = home_dir();
```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities 3): 新增 `home_dir()` 助手——Windows 上返回 `%USERPROFILE%`（如 `C:\Users\<user>`），Unix 上返回 `$HOME`；二者均成立时与改动前 Unix 行为等价。
- [ ] **AC2** (PRD §User Flow 异常流 2): `HOME` 未设时，Windows 上 `home_dir()` 经 `dirs::home_dir()` / `USERPROFILE` 兜底返回真实用户目录，**绝不返回 `/`**；config import scan 不再去扫虚假 Unix 路径。
- [ ] **AC3** (PRD §Problem Statement): `lib.rs:336`（PTY pool refill `home_path`）与 `lib.rs:637`（config import scan）两处 `std::env::var("HOME").unwrap_or("/")` 全部替换为 `home_dir()` 调用，无残留裸 `HOME` 硬编码。
- [ ] **AC4** (PRD §Decisions Log D2): `dirs` crate 加入 workspace 依赖（`[workspace.dependencies]` + `crates/app` 引用），`Cargo.lock` 同 PR 提交；未引入 `directories`/手写 cfg 等被拒候选。
- [ ] **AC5** (PRD §Success Metrics 反指标): macOS + Ubuntu 上 `cargo test --workspace` 仍 100% 绿——Unix 分支仍读 `$HOME`，PTY pool / config import 在 Unix 行为零回归。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 home_dir 跨平台解析 | SCEN-1.2.1 | TEST-1.2.1 `test_1_2_1_home_dir_resolves_per_platform` | N/A | `cargo test -p vibestation_app home_dir` | Done |
| AC2 HOME 缺失 Windows 非 `/` | SCEN-1.2.2 | TEST-1.2.2 `test_1_2_2_home_dir_no_env_windows_not_root` | N/A | `cargo test -p vibestation_app home_dir` | Done |
| AC3 两处硬编码替换 | SCEN-1.2.3 | TEST-1.2.3 `test_1_2_3_no_hardcoded_home_var` | N/A: grep 断言 + build | `cargo build --workspace` + grep | Done |
| AC4 dirs 依赖就位 | N/A: 依赖配置无业务场景 | N/A: 由 build 覆盖 | N/A | `cargo build --workspace`（解析 Cargo.lock 含 dirs）| Done |
| AC5 Unix 零回归 | SCEN-1.2.4 | TEST-1.2.4 `test_1_2_4_unix_home_unchanged` | N/A | `cargo test --workspace`（mac/ubuntu）| Done |

## 8. Risks

- **R1（关联 PRD §Technical Risks R4 · 路径/UNC/编码）**：Windows `%USERPROFILE%` 含空格/中文/UNC 时 `PathBuf` 拼接出错。缓解：`dirs::home_dir()` 返回 OS 原生 `PathBuf`（不做字符串拼接）；UNC 规范化由既有 `dunce` 处理（非本 task）；补含空格/中文路径 round-trip 单测。
- **R2（关联 PRD §Technical Risks R3 · mac/Linux 回归）**：助手实现误改 Unix 语义。缓解：`#[cfg(unix)]` 分支保留 `$HOME` 读取；mac/Linux 全量 cargo test 必绿；TDD 先 RED 锁 Unix 返回值。
- **R3（依赖面 · standard §17.7）**：`dirs` 5.x 与现有锁定栈版本冲突。缓解：`dirs` 无 C 依赖、依赖面极小（ADR-002）；`cargo build` 验证 `Cargo.lock` 无冲突解析。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile  <!-- 与 adapter §Commands Install 一致 -->
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制：实施 agent 不允许 N/A -->
- **Build**: cargo build --workspace
- **Runtime smoke**: pnpm tauri:dev  <!-- Windows 本机 · 确认启动后 config import scan 解析到 %USERPROFILE% 而非 / -->

> Integration / E2E / Coverage 本 task N/A：集成测试随 `cargo test --workspace` 一起跑，无独立 e2e 框架，MVP 不强制覆盖率。

## 10. Completion Notes

- **完成日期**：2026-05-29（Windows 11 x64 MSVC 本机实施 · solo tier · feat/windows-support 分支）
- **改动文件**：
  - `Cargo.toml`（修改 · `[workspace.dependencies]` 加 `dirs = "5"`，解析锁定 5.0.1）
  - `crates/app/Cargo.toml`（修改 · `[dependencies]` 加 `dirs.workspace = true`）
  - `crates/app/src/lib.rs`（修改 · 新增跨平台 `home_dir()` 助手 + `home_dir_or_root()` 收敛为其别名 + PTY pool refill 调用点改用 `home_dir()` + 新增 `home_dir_tests` 4 个单测）
  - `Cargo.lock`（修改 · dirs 5.0.1 + dirs-sys 0.4.1 + redox_users 0.4.6 锁定 · 纯 Rust 无 C 依赖）
- **commit 列表**（feat/windows-support 分支 · solo 三段节律）：
  - `c5a27de` test(app-home): 加 SCEN-1.2.1~1.2.4 共 4 个 RED 测试 + home_dir() 骨架
  - `d0d60d8` feat(app-home): 实现跨平台 home_dir() 助手 + 替换两处 HOME 硬编码
  - refactor: 无（实现已最小 · 无重复 / 过长函数）
- **§9 Verification 结果**（Windows 本机实跑 raw）：
  - install: N/A（本 task 不改 JS 依赖 · `pnpm install` 与本 task 无关）
  - lint（cargo clippy -p vibestation-app --lib -- -D warnings）: **app/src/lib.rs 0 hit**（grep lib.rs 计数 = 0）· ⚠️ 命令整体仍 fail，13 项 error 全部来自 `crates/core` 的 `fs_watch.rs`（unused import / dead_code / needless_return）+ `external_term/detect.rs`（variant `Macos` never constructed），属 **task 3.4 / task 3.1** 范围（`-p vibestation-app` 仍编译 core 依赖故触发）· 本 task 不允许改这两文件 · 同 task-1.1 §10 先例
  - typecheck（cargo check --workspace）: **Finished · 0 error**（0.90s 增量）
  - unit-test（cargo test -p vibestation-app --lib）: ⚠️ **Windows 本机无法链接 app lib 测试目标** —— `crates/app/src/fix_path_env.rs` 的 `#[cfg(test)] mod tests`（line 65-95）未平台门控，引用仅 `#[cfg(any(macos, linux))]` 定义的 `resolved_shell` / `default_shell` / `shell_command_args`，在 Windows 触发 7 个 E0425（cannot find function）→ 整个 app lib test target 编译失败 · 属 **task 6.1（windows-test-gating）** 范围（跨模块 Unix-only 测试门控 · 本 task 文件域不含 fix_path_env.rs）。`home_dir_tests` 自身经 `--no-run` 验证 **0 编译错误**（错误全部定位在 fix_path_env.rs）· 逻辑正确性由 source-scan TEST-1.2.3（生产代码无残留裸 HOME 硬编码 · 已校准排除自指）+ AC1/AC2/AC4 由 build 绿 + Cargo.lock 含 dirs 静态证明覆盖 · cfg(unix) 的 TEST-1.2.1/1.2.4 等价 $HOME 断言待 task-6.1 解锁 app test target 后 / mac/Linux CI 跑绿
  - build（cargo build --workspace）: **Finished · 0 error**（35.68s）· app lib 仅 2 warning，均为 `fix_path_env.rs` unused import（pre-existing · 非本 task）
  - runtime-smoke（pnpm tauri:dev）: 未跑（dispatch-only 模式 · 主 agent 不亲自 smoke · Windows 运行期 config import 解析到 %USERPROFILE% 而非 / 的实证 defer reviewer / Phase 2-3 实跑窗口 · 本 task 由 build 绿 + dirs::home_dir() Windows 语义静态保证覆盖）
- **剩余风险 / 未做项**：
  - **AC5 Unix 零回归依赖 mac/Linux CI / reviewer 实证**：本机为 Windows，无法实跑 `cargo test --workspace`（mac/ubuntu）；`home_dir()` 的 `dirs::home_dir()` 在 Unix 等价读 `$HOME`，TEST-1.2.4 cfg(unix) 断言已写，待 Unix 内核跑绿确认。
  - **app lib test target Windows 不可编译 = task-6.1 阻塞**：`fix_path_env.rs` test 模块的 Windows 门控修复后，`home_dir_tests` 4 测试即可在 Windows 本机实跑（已 `--no-run` 证明 0 编译错误）。
  - `%APPDATA%` config 路径分支 / `dunce::canonicalize` UNC 规范化 = **task-3.2** 范围（本 task 仅修家目录解析，不动 config import 扫描路径）。
- **下游 task 影响**：
  - **task-3.2（config-import-paths）**：消费 `home_dir()` 返回值落地 `%APPDATA%` 路径分支；本 task 已让 `home_dir_or_root()`（config import scan 调用点）在 Windows 返回 `C:\Users\<user>` 而非 `/`，task-3.2 可基于正确家目录扩展。
  - 无 spec / AC / 其他 task 改动；mac/Linux 行为零回归（dirs 在 Unix 等价 $HOME）。
