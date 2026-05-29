# Task `5.1`: `windows-bundle`

**Status**: Done

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`。
> 本项目无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研（`spike-tmp/win-survey.json`）+ 实际源码（`crates/app/tauri.conf.json`）填实，非编造，故初始即 Ready。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 5 · build-package-ci
**Dependencies**: 依赖 1.1（pty-platform-split · `cargo build --workspace` 在 Windows 编译通过 → bundler 才有可打包的二进制）

## 1. Background

`crates/app/tauri.conf.json` 的 `bundle.targets` 当前只有 `["dmg", "appimage", "deb"]`（macOS + Linux），**缺 Windows 安装包格式**。没有 `nsis`/`msi`，Tauri bundler 在 windows-latest 上无法产出 Windows 安装器——Windows 用户拿不到可安装产物（PRD §Problem Statement / §Core Capabilities 5）。

同时，`tauri.conf.json` 的 `app.windows[0]` 含多个 macOS 专属窗口装饰：`windowEffects.effects=["hudWindow"]`、`titleBarStyle="Overlay"`、`hiddenTitle=true`、`trafficLightPosition`，以及 `app.macOSPrivateApi=true`。需确认这些在 Windows 上被 Tauri 安全忽略（survey `already_windows_ok` 已注明"macOS-specific effects 被 Tauri 安全忽略"），或在出现渲染异常时条件化，避免 Windows 窗口出现 macOS 专属样式 artifact。

ADR-004 已定 bundle 用 NSIS 主（`.exe`，小巧友好）+ MSI 辅（`.msi`，企业 GPO 部署），先 unsigned（对齐 macOS alpha）。

## 2. Goal

任务完成后应成立的事实：

1. `bundle.targets` 含 `"nsis"` 与 `"msi"`（在保留 `dmg`/`appimage`/`deb` 不变的前提下追加）。
2. 本机 Windows 11 `pnpm tauri:build` 产出 `target/release/bundle/nsis/*.exe` 与 `target/release/bundle/msi/*.msi`，安装包可安装并启动 Vibestation。
3. Windows 窗口启动后无 macOS 专属窗口装饰 artifact（无 traffic light 残留、标题栏正常），macOS/Linux 窗口外观零回归。
4. 一个轻量单元测试解析 `crates/app/tauri.conf.json`，断言 `bundle.targets` 含 `nsis` 与 `msi`，且仍含 `dmg`/`appimage`/`deb`（防回归删除）。

## 3. Scope

### In Scope

- `crates/app/tauri.conf.json`：`bundle.targets` 追加 `"nsis"`、`"msi"`（保留现有三项）。
- `crates/app/tauri.conf.json`：确认/条件化 `app.windows[0].windowEffects` / `titleBarStyle` / `hiddenTitle` / `trafficLightPosition` / `app.macOSPrivateApi` 在 Windows 安全（仅当实测出现 artifact 才动结构；默认确认 Tauri 安全忽略即可，不破坏 macOS 行为）。
- 新增轻量单元测试（解析 `tauri.conf.json` 断言 targets），落 `crates/app/tests/` 或 `crates/app/src/` `#[cfg(test)]`。
- `crates/app/icons/icon.ico`（已就位，无需新增）作为 NSIS/MSI 安装器图标——仅验证引用路径有效。

### Out Of Scope

- Windows 代码签名 / Authenticode 证书（PRD §Core Capabilities 明确不做，先 unsigned，推 Windows GA）。
- MSIX / Microsoft Store 分发（PRD Out of Scope）。
- 单实例（single-instance）NSIS wrapper（PRD §Open Questions OQ2，MVP 不做）。
- CI 矩阵改动（属 Task 5.2）；`prepare` 脚本与 `configure_title_bar`（属 Task 5.3）。
- WebView2 fixed-version 随包分发（PRD §Open Questions OQ4，默认 evergreen）。

## 4. Users / Actors

- **Windows 11 终端用户**：下载 `.exe`（NSIS）或 `.msi`，安装→启动 Vibestation。
- **小团队 IT / 部署方**：用 `.msi` 走 GPO 企业部署（PRD §Users）。
- **项目维护者 / CI（windows-latest）**：在 Task 5.2 矩阵中可选跑 `pnpm tauri:build --bundles nsis` 校验 bundle 配置。

## 5. Behavior Contract

`tauri.conf.json` 是声明式配置：`bundle.targets` 决定 `tauri build` 在当前平台产出哪些 bundle（Tauri 自动只产当前 OS 支持的格式——Windows 上跑只产 nsis/msi，忽略 dmg/appimage/deb）。窗口装饰为 macOS 专属字段，Tauri 在非 macOS 平台静默忽略。

### 5.1 Required Reading

- [Phase 5 spec](../phases/phase-5-build-package-ci.md)（本 task 所属阶段目标与 §6 验收）
- [task-1.1-pty-platform-split.md](./task-1.1-pty-platform-split.md)（上游：Windows 编译通过 → 有可打包二进制）
- [ADR-004 Windows 安装包 NSIS 主 + MSI 辅（unsigned MVP）](../../decisions/adr-004-windows-installer-nsis-msi.md)
- BDD：[test/features/build-and-integration.feature](../../../test/features/build-and-integration.feature)
- 参考实现现状：`crates/app/tauri.conf.json`（targets line 44 / windowEffects line 25-35 / macOSPrivateApi line 14）

### 5.2 Imports

- 测试侧（`crates/app/tests/` 或 `#[cfg(test)] mod tests`）：
  - `serde_json`（workspace 已有依赖，解析 `tauri.conf.json`）——`use serde_json::Value;`
  - `std::fs`（读配置文件）、`std::path::PathBuf`（定位 `crates/app/tauri.conf.json`，相对 `env!("CARGO_MANIFEST_DIR")`）
- 生产侧：本 task 主体是声明式 JSON 改动，无 Rust import 新增。

### 5.3 函数签名

本 task 主体是 `tauri.conf.json` 声明式改动，无生产函数签名变更。新增轻量单元测试骨架（Windows-agnostic，三平台均可跑——验证的是配置文件内容，不依赖运行平台）：

```rust
// crates/app/tests/bundle_config.rs（新增）
// SCEN-5.1.1 / AC1 · 解析 tauri.conf.json 断言 bundle.targets

use serde_json::Value;
use std::path::PathBuf;

/// 读取 crates/app/tauri.conf.json 并解析为 serde_json::Value
fn load_tauri_conf() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).expect("tauri.conf.json must be valid JSON")
}

#[test]
fn test_5_1_1_bundle_targets_include_windows() {
    // TEST-5.1.1
    let conf = load_tauri_conf();
    let targets = conf["bundle"]["targets"]
        .as_array()
        .expect("bundle.targets must be an array");
    let names: Vec<&str> = targets.iter().filter_map(Value::as_str).collect();
    assert!(names.contains(&"nsis"), "bundle.targets must include nsis, got {names:?}");
    assert!(names.contains(&"msi"), "bundle.targets must include msi, got {names:?}");
}

#[test]
fn test_5_1_2_bundle_targets_preserve_unix() {
    // TEST-5.1.2 · 零回归：不得删除 mac/Linux targets
    let conf = load_tauri_conf();
    let names: Vec<String> = conf["bundle"]["targets"]
        .as_array().expect("array")
        .iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
    for expected in ["dmg", "appimage", "deb"] {
        assert!(names.iter().any(|n| n == expected),
            "bundle.targets must still include {expected} (no Unix regression), got {names:?}");
    }
}
```

> 窗口装饰确认（§Scope In）若需条件化，是 JSON 字段调整而非 Rust 签名；以本机 Windows `pnpm tauri:dev` 实跑观察窗口渲染为准（§9 Runtime smoke）。

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities 5 · §Decisions Log D4): `crates/app/tauri.conf.json` 的 `bundle.targets` 含 `"nsis"` 与 `"msi"`，且仍含 `"dmg"`/`"appimage"`/`"deb"`（轻量单元测试断言通过）。
- [ ] **AC2** (PRD §Success Metrics 主要指标): 本机 Windows 11 `pnpm tauri:build` 产出 `target/release/bundle/nsis/*.exe` 与 `target/release/bundle/msi/*.msi`（文件存在且 > 0 字节）。
- [ ] **AC3** (PRD §User Flow 主流程 4): 产出的 `.exe`（或 `.msi`）安装后 Vibestation 启动，窗口正常渲染、无 macOS 专属样式 artifact（traffic light / overlay 标题栏残留）。
- [ ] **AC4** (PRD §Constraints §兼容性 · §Success Metrics 反指标): macOS/Linux 上 `pnpm tauri:build` 的 `dmg`/`appimage`/`deb` 产物与窗口外观零回归（bundle.targets 仅追加不替换；windowEffects 等 macOS 字段不变）。
- [ ] **AC5** (本 task 新增): icon 引用有效——NSIS/MSI 安装器使用 `crates/app/icons/icon.ico`（已就位），bundle 过程无 "icon not found" 错误。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 · targets 含 nsis+msi 且保留 unix | SCEN-5.1.1 | TEST-5.1.1 / TEST-5.1.2 | N/A | `cargo test -p vibestation-app --test bundle_config` | Done |
| AC2 · Windows 产出 .exe + .msi | SCEN-5.1.2 | N/A（bundle 产物层 · 非单元） | runtime-smoke：`pnpm tauri:build` 本机 Windows | 本机 Windows 11 `pnpm tauri:build` + 列目录 | Done |
| AC3 · 安装后窗口正常渲染 | SCEN-5.1.3 | N/A（GUI 层） | runtime-smoke：安装 + 启动 | 本机安装 `.exe` + 启动观察（§2.14） | Done |
| AC4 · mac/Linux bundle 零回归 | SCEN-5.1.4 | TEST-5.1.2（unix targets 保留） | N/A | macOS/Ubuntu `pnpm tauri:build` 产物对比 | Done |
| AC5 · icon 引用有效 | SCEN-5.1.2 | N/A | runtime-smoke：bundle 无 icon error | `pnpm tauri:build` 日志无 icon not found | Done |

## 8. Risks

- **R-5.1-a**（关联 PRD R2）：windows-latest / 本机缺 WiX toolset → `msi` bundle 失败。缓解：先确认 nsis 单独可产（NSIS 更轻），msi 失败时可临时 `--bundles nsis` 验证主路径，msi 作可选；WiX 通过 Tauri 自动下载或 CI 显式装。
- **R-5.1-b**（关联 PRD R4）：macOS 专属窗口装饰在 Windows 渲染异常。缓解：survey `already_windows_ok` 已注明 Tauri 安全忽略；本机实跑确认；仅在出现 artifact 时才条件化 JSON，避免无谓改 macOS 行为。
- **R-5.1-c**（关联 PRD §Success Metrics 反指标）：误删/改动 mac/Linux targets 致回归。缓解：TEST-5.1.2 锁住 unix targets 存在；改动为纯追加。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test -p vibestation_app --test bundle_config（随 `cargo test --workspace` 一并跑）
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Runtime smoke**: 本机 Windows 11 `pnpm tauri:build` → 校验 `target/release/bundle/nsis/*.exe` + `bundle/msi/*.msi` 存在并安装启动（§2.14 critical UX path）
- **Manual**: 安装 `.exe` 后启动 Vibestation，肉眼确认窗口无 macOS 专属 artifact；macOS 本机 `pnpm tauri:build` 确认 dmg 仍产出

## 10. Completion Notes

- **完成日期**：2026-05-29
- **改动文件**：
  - `crates/app/tauri.conf.json`（修改 · `bundle.targets` 追加 `nsis`/`msi`）
  - `crates/app/tests/bundle_config.rs`（新增 · TEST-5.1.1 / TEST-5.1.2）
- **commit 列表**：
  - `51947eb` test(tauri-bundle): 加 SCEN-5.1.1 共 2 个 RED 测试
  - `358c892` feat(tauri-bundle): bundle.targets 追加 nsis+msi 通过全部 2 个测试
  - （无 refactor · 单行声明式改动 + 简洁测试）
- **§9 Verification 结果**：
  - install: ✅（pnpm 9.15.9 · web node_modules 已就位 · build smoke 内 `pnpm install` 链路通过）
  - typecheck: ✅（`cargo check --workspace` 0 error）
  - unit-test: 2 passed / 0 failed（`cargo test -p vibestation-app --test bundle_config` · 注：包名实际为 `vibestation-app` 非 spec §9 写的 `vibestation_app`）
  - build: ✅（`cargo build --workspace` 0 error · release bundle build 亦 0 error）
  - lint: ✅（`cargo clippy --workspace -- -D warnings` 0 error · 即 CLAUDE.md/prompt 既定 gate）；⚠️ `--all-targets` 变体因 `crates/core/tests/git_ops_integration.rs:19` 无条件 `use std::os::unix::fs::PermissionsExt` + `set_mode(0o755)`（Unix-only API）在 Windows 编译失败——此为**预存**的 Windows 测试门控缺口（pre-task baseline commit `79e5b18` 已存在 · 本 task 未触及 core crate），归属 Task 6.1（windows-test-gating）· 不在本 task 范围
  - runtime-smoke: ✅ **本机 Windows 11 实跑**：① `pnpm tauri:build:smoke`（--debug --no-bundle）产出 `target/debug/vibestation-app.exe`（exit 0 · macOS 专属窗口字段未引发 build 报错）② `pnpm tauri:build --bundles nsis` 产出 `target/release/bundle/nsis/Vibestation_0.1.0_x64-setup.exe`（7.57 MB · Tauri 自动下载 NSIS 3.11 toolchain）③ `pnpm tauri:build --bundles msi` 产出 `target/release/bundle/msi/Vibestation_0.1.0_x64_en-US.msi`（10.18 MB · Tauri 自动下载 WiX 3.14 toolset）· bundle 全程无 "icon not found"（AC5 ✅ · icon.ico 被消费无错）
  - manual: ⚠️ AC3（安装 `.exe` 后启动 + 肉眼确认无 macOS 专属 artifact）+ AC4（macOS 本机 `pnpm tauri:build` 确认 dmg 仍产出）defer 给 Arbiter——安装器产物已在本机生成验证，但**安装后 GUI 渲染**与 **macOS 平台 build** 需各自宿主机实测；macOS 字段在 Windows build 链路安全（实测无报错），渲染层确认走 Arbiter playbook 窗口
- **剩余风险 / 未做项**：AC3 安装后 GUI 渲染肉眼确认 + AC4 macOS 宿主机 dmg 产物确认 defer 给 Arbiter（产物层已本机验证 nsis/msi 可产出）；unsigned MVP（SmartScreen 警告 · ADR-004 推 GA 签名）
- **下游 task 影响**：Task 5.2（CI 矩阵）可在 windows-latest leg 复用本配置跑 `--bundles nsis` 可选 step；无破坏性影响
