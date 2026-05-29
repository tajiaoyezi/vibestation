# Task `1.3`: `shell-default-setting`

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`

**Priority**: P0
**Owner**: 主 agent
**Related Phase**: 1（foundation-build）
**Dependencies**: 依赖 1.1（pty-platform-split）—— 需 1.1 让 `crates/core` 在 Windows 编译通过，才能 `cargo test --workspace` 验证本 task 的 default_shell 单测

## 1. Background

`crates/core/src/app_settings.rs` 的 `default_shell` 默认值有两处只区分 macOS / else，**无 Windows 分支**，Windows 上回落 `/bin/bash`（不存在）：

- `impl Default for AppSettings`（line 69-72）：
  ```rust
  default_shell: if cfg!(target_os = "macos") {
      "/bin/zsh".to_string()
  } else {
      "/bin/bash".to_string()
  },
  ```
- `AppSettingsStore::get_all`（line 183-191）的 `get_parsed(pool, "default_shell", ...)` fallback：
  ```rust
  if cfg!(target_os = "macos") { "/bin/zsh" } else { "/bin/bash" }
  ```

后果：Windows 上首次启动（DB 无 `default_shell` 记录时）默认 shell = `/bin/bash`，PTY 尝试拉起一个不存在的 Unix 路径。本 task 给这两处加 `#[cfg(target_os = "windows")]` 分支，返回 Windows 安全占位 `cmd.exe`（永远保底，PRD §User Flow 异常流 1）。

ADR-003 选定 Windows 默认 shell 探测链 `pwsh.exe → powershell.exe → cmd.exe`。**本 task 只给安全占位默认值 `cmd.exe`**（"有啥用啥"由 Phase 2 task 2.1 的 `resolve_default_shell` 探测链运行期细化）；本 task 不做探测，只让 `AppSettings::default()` / `get_all()` 在 Windows 返回一个真实存在的 shell 而非 `/bin/bash`。

## 2. Goal

任务完成后应成立的事实：

- `AppSettings::default().default_shell` 与 `AppSettingsStore::get_all(pool).default_shell`（DB 无记录时）在 Windows 返回 `cmd.exe`（占位 · 真实存在 · 永远保底）、macOS 返回 `/bin/zsh`、Linux 返回 `/bin/bash`——三平台各自正确，无平台拿到不存在的 shell 路径。
- 改动严格走 `#[cfg(target_os = "windows")]` 分支；macOS/Linux 默认值字节级零回归。
- 占位值与 ADR-003 探测链对齐（注释明确 Phase 2 `resolve_default_shell` 将细化为 pwsh→powershell→cmd），不与 Phase 2 冲突。

## 3. Scope

### In Scope

- `crates/core/src/app_settings.rs`：
  - `impl Default for AppSettings`（line 69-72）的 `default_shell` 由 macOS/else 二分支改为三分支（macOS → `/bin/zsh`、windows → `cmd.exe`、linux/其他 → `/bin/bash`）。
  - `AppSettingsStore::get_all`（line 183-191）的 `default_shell` `get_parsed` fallback 字面值同步改为三分支。
  - 抽取一个共享的 `fn default_shell_for_platform() -> &'static str` 助手避免两处重复（可选 · 减少漂移）。
- `crates/core/src/app_settings.rs` 内嵌 `#[cfg(test)] mod tests` 新增 default_shell 跨平台默认值单测。

### Out Of Scope

- Windows shell 探测链 `pwsh.exe → powershell.exe → cmd.exe` 运行期实现（Phase 2 task 2.1 windows-shell-detection · `pty.rs::resolve_default_shell` / `list_available_shells`）。
- `pty.rs::default_shell_path()`（line 1019）的 Windows 分支（Phase 2 task 2.1——注意 `app_settings.rs` 与 `pty.rs` 各有独立默认值，本 task 只动 `app_settings.rs`）。
- DB schema / migration 改动（`default_shell` 仍是 `app_settings` 表的字符串 key，不变）。
- 用户在设置 UI 选择 shell 的前端流程（不涉及）。

## 4. Users / Actors

- **Windows 11 AI-agent 开发者**：首次启动（DB 无 default_shell 记录）时默认 shell = `cmd.exe`（真实存在），新建 Tab 不会因 `/bin/bash` 不存在而 spawn 失败（PRD §User Flow happy path 步 2 / 异常流 1）。
- **Phase 2 task 2.1 实施 agent**（下游）：在本 task 的安全占位基础上，把运行期探测细化为 pwsh→powershell→cmd 链。

## 5. Behavior Contract

`AppSettings::default().default_shell` 与 `get_all()`（DB 无记录回退）返回当前平台的合理默认 shell：macOS=`/bin/zsh`、Windows=`cmd.exe`、Linux=`/bin/bash`。Unix 行为与改动前等价。

### 5.1 Required Reading

- 上游 task：[`task-1.1-pty-platform-split.md`](./task-1.1-pty-platform-split.md)（编译解锁前置）。
- [`docs/decisions/adr-003-windows-default-shell-probe-chain.md`](../../decisions/adr-003-windows-default-shell-probe-chain.md)（默认 shell 探测链 · 本 task 给占位起点）。
- BDD: [`test/features/app-foundation.feature`](../../../test/features/app-foundation.feature)。
- 现有源：`crates/core/src/app_settings.rs:69-72`（`impl Default`）、`:183-191`（`get_all` fallback）。

### 5.2 Imports

```rust
// crates/core/src/app_settings.rs · 无新增外部依赖
// 现有：use crate::db::DbPool; use serde::{Deserialize, Serialize}; use ts_rs::TS;
// 仅用 cfg!(target_os = "windows") 编译期判断 · 无新 import
```

### 5.3 函数签名

> Windows 适配后的真实签名骨架。抽取共享助手 + 两处调用点改三分支。

```rust
// crates/core/src/app_settings.rs · 新增共享助手（避免两处字面值漂移）
/// 平台默认 shell。
/// macOS → /bin/zsh · Windows → cmd.exe（占位 · 永远保底 · Phase 2 task-2.1
///   resolve_default_shell 探测链细化为 pwsh→powershell→cmd）· Linux/其他 → /bin/bash。
fn default_shell_for_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    { "/bin/zsh" }
    #[cfg(target_os = "windows")]
    { "cmd.exe" } // 占位 · 与 ADR-003 探测链对齐 · Phase 2 运行期 resolve 细化
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { "/bin/bash" }
}

// impl Default for AppSettings · line 69-72 改为：
//   default_shell: default_shell_for_platform().to_string(),

// AppSettingsStore::get_all · line 183-191 改为：
//   let default_shell = get_parsed(pool, "default_shell", default_shell_for_platform());
```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities 2): `AppSettings::default().default_shell` 在 Windows 返回 `cmd.exe`、macOS 返回 `/bin/zsh`、Linux 返回 `/bin/bash`——Windows 不再回落不存在的 `/bin/bash`。
- [ ] **AC2** (PRD §User Flow 异常流 1): `AppSettingsStore::get_all(pool)`（DB 无 `default_shell` 记录时）的 fallback 与 `impl Default` 一致返回平台默认值；两处由共享助手 `default_shell_for_platform()` 提供，无字面值漂移。
- [ ] **AC3** (PRD §Decisions Log D3): Windows 占位值 `cmd.exe`（永远保底）与 ADR-003 探测链 `pwsh→powershell→cmd` 对齐——代码注释明确 Phase 2 `resolve_default_shell` 运行期细化，本 task 不做探测。
- [ ] **AC4** (PRD §Constraints 兼容性): 改动严格走 `#[cfg(target_os = "windows")]` / `cfg!` 分支；macOS/Linux 默认值字节级零回归（`/bin/zsh` / `/bin/bash` 不变）。
- [ ] **AC5** (PRD §Success Metrics 反指标): macOS + Ubuntu 上 `cargo test --workspace -p vibestation_core app_settings` 仍 100% 绿；新增 default_shell 单测在三平台各断言对应默认值。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 Default default_shell 三平台 | SCEN-1.3.1 | TEST-1.3.1 `test_1_3_1_default_shell_per_platform` | N/A | `cargo test -p vibestation_core app_settings` | Not Started |
| AC2 get_all fallback 一致 | SCEN-1.3.2 | TEST-1.3.2 `test_1_3_2_get_all_fallback_matches_default` | N/A | `cargo test -p vibestation_core app_settings` | Not Started |
| AC3 占位对齐 ADR-003 | SCEN-1.3.3 | TEST-1.3.3 `test_1_3_3_windows_default_is_cmd` | N/A | `cargo test -p vibestation_core app_settings` | Not Started |
| AC4 cfg 分支 · Unix 不变 | SCEN-1.3.4 | TEST-1.3.4 `test_1_3_4_unix_default_unchanged` | N/A | `cargo test --workspace`（mac/ubuntu）| Not Started |
| AC5 Unix 零回归 + 三平台单测 | N/A: 回归由既有套件覆盖 | N/A: 同 TEST-1.3.1/1.3.4 | N/A | `cargo test --workspace` | Not Started |

## 8. Risks

- **R1（关联 PRD §Technical Risks · 兼容性）**：占位 `cmd.exe` 与 Phase 2 探测链结果不一致，导致用户首启用 cmd 但 Phase 2 期望 pwsh。缓解：占位明确是"DB 无记录时的安全保底"，Phase 2 `resolve_default_shell` 在运行期优先探测 pwsh→powershell，仅当都不可用才落 cmd；注释指向 Phase 2，不锁死。
- **R2（关联 PRD §Technical Risks R3 · mac/Linux 回归）**：抽取共享助手时误改 Unix 字面值。缓解：助手三分支用 `#[cfg]` 编译期选择，Unix 字面值 `/bin/zsh` / `/bin/bash` 保持原文；mac/Linux 单测断言不变。
- **R3（与 pty.rs 默认值不同步）**：`app_settings.rs` 与 `pty.rs::default_shell_path()` 是两套独立默认值，本 task 只改前者，Phase 2 task 2.1 改后者，期间二者 Windows 默认值可能短暂不一致。缓解：本 task §3 明确 Out of Scope，Phase 2 task 2.1 §Required Reading 列本 task，确保对齐。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile  <!-- 与 adapter §Commands Install 一致 -->
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制：实施 agent 不允许 N/A -->
- **Build**: cargo build --workspace

> Integration / E2E / Coverage / Runtime smoke 本 task N/A：纯默认值逻辑改动，集成随 `cargo test --workspace` 跑；DB 无记录回退路径由单测覆盖，无需 GUI runtime smoke（Phase 2 PTY 实跑才验证 shell 真拉起）。

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：
  - `crates/core/src/app_settings.rs`（修改 · default_shell 三分支 + 共享助手）
- **commit 列表**：
  - `<TBD-after-impl>` test: 加 SCEN-1.3.1~1.3.4 RED 测试
  - `<TBD-after-impl>` feat: 实现 default_shell_for_platform 三分支通过测试
  - `<TBD-after-impl>` refactor:（如有）
- **§9 Verification 结果**：
  - install: <TBD-after-impl>
  - lint: <TBD-after-impl>
  - typecheck: <TBD-after-impl>
  - unit-test: <TBD-after-impl>
  - build: <TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
