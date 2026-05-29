# Task `5.3`: `prepare-titlebar`

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`。
> 本项目无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研（`spike-tmp/win-survey.json`）+ 实际源码（`package.json` line 12 · `crates/app/src/lib.rs` `configure_title_bar`）填实，非编造，故初始即 Ready。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 5 · build-package-ci
**Dependencies**: 依赖 1.1（pty-platform-split · Windows 编译通过 → `configure_title_bar` Windows 分支能编译验证）

## 1. Background

两处 Windows 缺口在本 task 收口：

1. **`package.json` `prepare` 脚本不跨平台**（survey package.json finding · severity medium）：当前 `"prepare": "git config core.hooksPath .githooks 2>/dev/null || true"` 用 bash 重定向 `2>/dev/null` + `|| true`。Windows PowerShell 把 `2>/dev/null` 重定向到名为 `nul` 的文件、`|| true` 语义不同，导致 Windows 开发者 clone + `pnpm install` 后 git hook 配置静默失败——commit 绕过 pre-push 分支保护（CLAUDE.md §禁区 的机械防护在 Windows 失效）。

2. **`configure_title_bar` 仅 macOS 分支**（survey lib.rs finding · severity medium）：`crates/app/src/lib.rs` 现有 `#[cfg(target_os = "macos")] fn configure_title_bar(...)`（设 `TitleBarStyle::Overlay`）+ `#[cfg(not(target_os = "macos"))] fn configure_title_bar(_app) {}`（空 stub）。需确认 Windows 走空 stub 安全（窗口用默认 Windows 原生 title bar，无 macOS overlay artifact），或显式给 Windows 分支注释/原生实现，使意图清晰且 `cargo build` 在 Windows 通过。

## 2. Goal

任务完成后应成立的事实：

1. `package.json` `prepare` 脚本改为跨平台 node 一行脚本（`node -e "..."` 用 `child_process.execSync` 配置 hooksPath，无 shell 重定向），在 Windows PowerShell/cmd.exe 与 macOS/Linux 上均能正确设 `git config core.hooksPath .githooks`，且失败不阻断 `pnpm install`（对齐原 `|| true` 容错语义）。
2. Windows 上 `pnpm install` 后 `git config --get core.hooksPath` == `.githooks`；macOS/Linux 行为零回归。
3. `configure_title_bar` 在 Windows 走安全分支（默认走现有 `#[cfg(not(target_os="macos"))]` 空 stub，显式注释 Windows 用原生 title bar；或给独立 `#[cfg(target_os="windows")]` 分支），Windows 窗口启动无 macOS 专属 artifact，`cargo build --workspace` 在 Windows 编译通过。
4. 一个轻量单元测试覆盖 prepare 脚本的 hooksPath 配置逻辑（或 node 脚本可被单元断言的纯函数化）+ `configure_title_bar` 的 Windows cfg 分支可编译/可调用。

## 3. Scope

### In Scope

- `package.json`：`scripts.prepare` 由 bash 重定向改为跨平台 node 一行脚本（`node -e "const {execSync}=require('child_process'); try{execSync('git config core.hooksPath .githooks',{stdio:'ignore'})}catch(e){}"` 或抽到 `scripts/configure-hooks.mjs` 由 `prepare` 调用）。
- `crates/app/src/lib.rs`：确认/补强 `configure_title_bar` 的 Windows 分支——保留现有 `#[cfg(not(target_os="macos"))]` 空 stub（Windows 用默认原生 title bar，最小改动），或拆出 `#[cfg(target_os="windows")]` 显式分支 + 注释意图。
- 轻量单元测试：覆盖 `configure_title_bar` Windows cfg 分支可编译/可调用（不 panic）；prepare 脚本若抽成 `.mjs` 则配 vitest 纯函数断言，若保持 node 一行则在 §9 Manual 验证。

### Out Of Scope

- `.githooks/pre-push` hook 脚本本身的 Windows 化（`#!/bin/sh` hook 生产逻辑 · PRD Out of Scope：git hook 脚本除测试层 cfg 处理外的深度 Windows 化不在范围）。
- bundle targets（属 Task 5.1）；CI 矩阵（属 Task 5.2）。
- 单实例强制（PRD §Open Questions OQ2）。
- macOS title bar overlay 逻辑改动（`#[cfg(target_os="macos")]` 分支不动，零回归）。
- `tauri.conf.json` 的 `windowEffects`/`trafficLightPosition` JSON 字段（Task 5.1 §Scope 已覆盖窗口装饰确认）。

## 4. Users / Actors

- **Windows 贡献者**：clone repo → `pnpm install` → git hooks 自动配置 → 本地 commit 受 pre-push 分支保护。
- **macOS/Linux 贡献者**：同上路径，行为零回归。
- **Windows 11 终端用户**：启动 Vibestation → 窗口用 Windows 原生 title bar 正常渲染。

## 5. Behavior Contract

- `prepare` 是 npm/pnpm 生命周期脚本，`pnpm install` 自动触发。新脚本必须跨 shell（不依赖 POSIX 重定向），失败时静默不阻断 install（容错），成功时设 `git config core.hooksPath .githooks`。
- `configure_title_bar` 在 Tauri `.setup()` 回调中被调用（`lib.rs` line 2601）。macOS 设 overlay 标题栏；非 macOS（含 Windows）走空 stub = 使用框架默认 title bar。契约：任何平台调用都不 panic，Windows 不残留 macOS 样式。

### 5.1 Required Reading

- [Phase 5 spec](../phases/phase-5-build-package-ci.md)
- [task-1.1-pty-platform-split.md](./task-1.1-pty-platform-split.md)（上游：Windows 编译通过 → cfg 分支可验证）
- [ADR-005 Windows 测试门控策略（cfg + ignore + 专测）](../../decisions/adr-005-windows-test-gating-strategy.md)（cfg 分支测试门控指导）
- BDD：[test/features/build-and-integration.feature](../../../test/features/build-and-integration.feature)
- 参考现状：`package.json` line 12（`prepare`）· `crates/app/src/lib.rs` line 2391-2404（`configure_title_bar` 两 cfg 分支）+ line 2601（`.setup()` 调用点）
- 关联治理：`CLAUDE.md` §禁区（`.githooks/pre-push` + `package.json prepare` 机械防护，决策表 #20）

## 5.2 Imports

- `crates/app/src/lib.rs`（生产）：现有 `use tauri::Manager;`（`get_webview_window`）+ `tauri::Runtime`；macOS 分支用 `tauri::TitleBarStyle`。Windows 分支若保持空 stub 无新增 import；若给原生实现需 `use tauri::TitleBarStyle;`（Windows 也支持 `set_title_bar_style`，按需）。
- 测试侧（`crates/app/src/lib.rs` `#[cfg(test)] mod tests` 或 `crates/app/tests/`）：`std`（无外部依赖；测 cfg 分支可调用性）。
- prepare 脚本侧：Node 内建 `require('child_process').execSync`（无 npm 依赖新增，符合 PRD §Technical Approach 不引入重依赖）。

## 5.3 函数签名

`configure_title_bar` 当前签名（保留 macOS 分支不动，确认/补强 Windows 分支）：

```rust
// crates/app/src/lib.rs

// macOS：现有实现不动（零回归）
#[cfg(target_os = "macos")]
fn configure_title_bar<R: tauri::Runtime>(app: &tauri::App<R>) {
    let Some(window) = app.get_webview_window("main") else {
        eprintln!("[mvp-11] main window not found for title bar setup");
        return;
    };
    if let Err(error) = window.set_title_bar_style(tauri::TitleBarStyle::Overlay) {
        eprintln!("[mvp-11] title bar overlay setup failed: {error}");
    }
}

// 非 macOS（含 Windows + Linux）：默认原生 title bar · 空 stub 安全
// Windows 适配确认点：Windows 走此分支 = 使用 Windows 原生标题栏，无 macOS overlay artifact。
// （最小改动方案：保留空 stub + 注释意图；如需 Windows 原生样式微调，可拆 #[cfg(target_os="windows")] 独立分支）
#[cfg(not(target_os = "macos"))]
fn configure_title_bar<R: tauri::Runtime>(_app: &tauri::App<R>) {
    // Windows / Linux：沿用框架默认装饰，本 task 不引入 macOS 专属 overlay
}
```

prepare 脚本（`package.json`，声明式，无 Rust 签名；node 一行）：

```jsonc
// package.json scripts（跨平台 · 无 shell 重定向 · 失败容错对齐原 || true）
"prepare": "node -e \"try{require('child_process').execSync('git config core.hooksPath .githooks',{stdio:'ignore'})}catch(e){}\""
```

> 测试骨架（验证 Windows cfg 分支可调用不 panic · 平台无关编译验证）：
>
> ```rust
> // crates/app/src/lib.rs 内 #[cfg(test)] mod tests，或 crates/app/tests/title_bar.rs
> // SCEN-5.3.2 / AC3 · 在非 macOS（含 Windows）build 下 configure_title_bar 走空 stub
> // 不 panic（编译期 cfg 已保证 Windows 走 stub 分支；此测试在 Windows CI leg 编译通过即证 cfg 分支健全）
> #[test]
> fn test_5_3_2_configure_title_bar_non_macos_stub_compiles() {
>     // TEST-5.3.2 · 此测试存在并在 windows-latest 编译通过 = Windows cfg 分支无编译错误。
>     // configure_title_bar 需 tauri::App，单测难构造真 App；
>     // 退而验证 cfg 编译健全 + 标记意图（运行期 GUI 验证走 §9 Runtime smoke 本机 Windows）。
>     assert!(cfg!(any(target_os = "macos", target_os = "windows", target_os = "linux")));
> }
> ```

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities 5 · §Problem Statement): `package.json` `scripts.prepare` 改为跨平台 node 脚本，不含 bash `2>/dev/null` 重定向；在 Windows PowerShell `pnpm install` 后 `git config --get core.hooksPath` 回显 `.githooks`，且无 `nul` 文件被误创建。
- [ ] **AC2** (PRD §Constraints §兼容性 · §Success Metrics 反指标): macOS/Linux 上 `pnpm install` 触发 prepare 后 `core.hooksPath` 仍正确设为 `.githooks`（零回归）；prepare 失败时不阻断 install（对齐原 `|| true` 容错）。
- [ ] **AC3** (PRD §Core Capabilities 5 · survey lib.rs finding): `configure_title_bar` 在 Windows（非 macOS 分支）走安全 stub，`cargo build --workspace` + `cargo test --workspace` 在 windows-latest 编译/跑通无 panic。
- [ ] **AC4** (PRD §User Flow 主流程 4): 本机 Windows 11 启动 Vibestation 后窗口用原生 title bar 正常渲染，无 macOS 专属 overlay / traffic light artifact。
- [ ] **AC5** (PRD §Compatibility · §Success Metrics 反指标): macOS 上 `configure_title_bar` 仍设 `TitleBarStyle::Overlay`（macOS 分支代码与行为零变化）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 · prepare 跨平台设 hooksPath（Windows）| SCEN-5.3.1 | N/A（脚本 · node 一行）| runtime-smoke：Windows `pnpm install` | 本机 Windows `pnpm install` + `git config --get core.hooksPath` | Not Started |
| AC2 · mac/Linux prepare 零回归 + 失败容错 | SCEN-5.3.1 | N/A | runtime-smoke：mac/Linux `pnpm install` | macOS/Linux `pnpm install` + `git config --get core.hooksPath` | Not Started |
| AC3 · configure_title_bar Windows 分支可编译跑通 | SCEN-5.3.2 | TEST-5.3.2 | CI：windows-latest `cargo test --workspace` | `cargo test --workspace`（含 windows-latest leg） | Not Started |
| AC4 · Windows 窗口原生 title bar 无 artifact | SCEN-5.3.2 | N/A（GUI 层）| runtime-smoke：本机 Windows 启动 | 本机 Windows `pnpm tauri:dev` 观察窗口（§2.14） | Not Started |
| AC5 · macOS overlay title bar 零回归 | SCEN-5.3.3 | N/A（GUI 层）| runtime-smoke：macOS 启动 | macOS `pnpm tauri:dev` 观察标题栏 overlay | Not Started |

## 8. Risks

- **R-5.3-a**（关联 PRD R3）：prepare node 脚本在某些 git 客户端 / CI 环境 `execSync` 抛错阻断 install。缓解：`try{...}catch(e){}` 吞错对齐原 `|| true` 语义（install 不阻断）；三平台各跑一次 `pnpm install` 验证（AC1/AC2）。
- **R-5.3-b**（关联 PRD R4）：Windows 空 stub 导致窗口装饰与 macOS 视觉不一致（无 overlay）。缓解：这是预期行为（Windows 用原生 title bar），PRD §Core Capabilities 5 接受平台条件化；如需 Windows 原生样式微调可拆独立 `#[cfg(target_os="windows")]` 分支（本 task 默认最小改动 stub）。
- **R-5.3-c**（关联 PRD §Success Metrics 反指标）：误改 macOS overlay 分支致 mac 回归。缓解：macOS `#[cfg(target_os="macos")]` 分支代码不动；AC5 锁住 macOS 行为。
- **R-5.3-d**（关联 PRD R3）：`configure_title_bar` 单测难构造真 `tauri::App`。缓解：TEST-5.3.2 退化为"Windows cfg 分支编译通过 = 健全"的编译期断言；运行期窗口渲染走 §9 Runtime smoke 本机 Windows（§2.14）。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace（含 TEST-5.3.2 · `configure_title_bar` Windows cfg 分支编译验证 · s2v unit-test 强制）
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Runtime smoke**: 本机 Windows 11 ① `pnpm install` → `git config --get core.hooksPath` 回显 `.githooks`、无 `nul` 文件；② `pnpm tauri:dev` → 窗口原生 title bar 正常无 macOS artifact（§2.14 critical UX path）
- **Manual**: macOS `pnpm install` 确认 `core.hooksPath` 仍为 `.githooks` 且 `pnpm tauri:dev` 标题栏仍是 overlay；故意制造 git config 失败场景确认 install 不被阻断

## 10. Completion Notes

<TBD-after-impl>
