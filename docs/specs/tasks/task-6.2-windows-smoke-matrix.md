# Task `6.2`: `windows-smoke-matrix`

> Task Spec（S2V §8.3）· Windows 适配工作流 `feat/windows-support`。
> 本 task 是整条 Windows 适配链路的端到端收口：Windows 本机跑通 critical UX path（ConPTY pwsh/cmd + Git status 200ms 刷新），并校验三平台测试矩阵零回归。

**Status**: Ready

> 无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 PRD §Success Metrics + §2.14 critical UX path + Windows 缺口调研填实，非编造。

**Priority**: P0
**Owner**: 主 agent（solo · 本机 H:\ Windows 11 实跑 + 调度）
**Related Phase**: Phase 6 · integration-matrix
**Dependencies**: 全部（Phase 1 foundation-build · Phase 2 shell-runtime · Phase 3 terminal-integration · Phase 4 frontend-platform · Phase 5 build-package-ci · Task 6.1 windows-test-gating 须全部 Done）

## 1. Background

前 5 个 Phase 各自隔离严格 TDD：Phase 1 让 `cargo build --workspace` 在 Windows 编译通过；Phase 2 落地 ConPTY spawn/IO 与 shell 探测链；Phase 3 启用 fs_watch Windows backend + 外部终端 + config import；Phase 4 补前端 `platform-windows` class + 快捷键显示；Phase 5 产出 `.exe/.msi` bundle + windows-latest CI。Task 6.1 把 Unix-only 测试门控干净。

但 S2V §8.2（C1）指出："task 拼起来能否集成"只有 Phase 级 §6 这一层兜底。本 task 是 Windows 适配工作流唯一的端到端集成验证点：在 Windows 本机 `pnpm tauri:dev` 跑通 PRD §User Flow happy path（新建 Tab → ConPTY 拉起 pwsh → prompt → `git --version` 回显 → 改文件 → Git status 200ms 刷新），并校验 macOS/Ubuntu 现有测试矩阵零回归（PRD §Success Metrics 反指标）。CI 无法做 headless GUI runtime 验证（PRD R5），故 GUI critical UX path 靠开发者本机 §2.14 实跑并记录证据；可机械断言的部分（bundle 产物 / 三平台 cargo test）落 CI + 汇总单元测试。

## 2. Goal

任务完成后：

- Windows 本机 `pnpm tauri:dev` 起窗，新建 Tab 经 ConPTY 拉起 pwsh（或回落 cmd）看到 prompt，`git --version` 回显正确，改 workspace 文件后 Git status 徽章 200ms 内刷新（fs_watch Windows backend 生效）。
- 三平台测试矩阵零回归：macOS + Ubuntu `cargo test --workspace` + `pnpm --filter @vibestation/web exec vitest run` 仍 100% 绿；Windows `cargo test --workspace` 0 panicked。
- 有一个轻量汇总断言测试（smoke checklist / 配置断言）保证 §9 unit-test 强制门槛满足，且 CI 可机械校验 Windows 链路前置条件（如 `tauri.conf.json` 含 nsis/msi target、三平台测试可枚举）。

## 3. Scope

### In Scope

- **Windows 端到端 GUI smoke**（本机 H:\ Windows 11 · §2.14 critical UX path）：`pnpm tauri:dev` 起窗 → 新建 Tab → ConPTY 拉起 pwsh/cmd → prompt 可见 → `git --version` / `echo` 回显 → workspace 改文件 → Git status 200ms 刷新。截图 / 录屏 / 日志归档到 §10 runtime-smoke。
- **三平台测试矩阵零回归校验**：在 macOS + Ubuntu 复跑 `cargo test --workspace`（reviewer / CI 矩阵）+ `pnpm --filter @vibestation/web exec vitest run`，与 Phase 前基线比对；Windows 复跑 `cargo test --workspace` 断言 0 panicked。
- **汇总断言测试**（满足 §9 unit-test 强制 · 新增 `crates/app/tests/windows_smoke_matrix.rs`）：轻量、跨平台可跑的机械断言——解析 `crates/app/tauri.conf.json` 断言 `bundle.targets` 含 `nsis` + `msi`（Windows bundle 前置条件）；断言 Windows smoke checklist 文档存在且非空（避免空 smoke）。

### Out Of Scope

- 生产代码 Windows 分支实现（PTY / shell / fs_watch / bundle / CI）——Phase 1-5 落地。
- Unix-only 测试门控（Task 6.1 负责）。
- Windows 代码签名 / Authenticode（PRD Out of Scope · 推 Windows GA）。
- 单实例强制（PRD §Open Questions OQ2 · MVP 不做）。
- `detect_process_cwd` Windows 精确实现（PRD Out of Scope · 缓存 `initial_cwd` 兜底）。
- 自动化 GUI E2E 框架（adapter §E2E test areas = N/A · GUI critical path 走 runtime-smoke 手动）。

## 4. Users / Actors

- **主 agent / 项目 Windows 贡献者（本机 H:\ Windows 11）**：`pnpm tauri:dev` 本机实跑 critical UX path，观察 ConPTY prompt + Git status 刷新，记录截图 / 录屏证据（§2.14）。
- **Windows 11 上的 AI-agent 开发者**：最终受益者——本 task 验证其 happy path（PRD §User Flow）在 Windows 成立。
- **windows-latest / macOS / Ubuntu CI runner**：跑汇总断言测试 + 三平台 `cargo test`；Windows headless 不跑 GUI（PRD R5），只跑 build + 单元/集成 + bundle 产物校验。

## 5. Behavior Contract

Windows 本机 critical UX path 与 mac/Linux 对等：新建 Tab → ConPTY 拉起默认 shell（pwsh→powershell→cmd 探测链，PRD D3）→ prompt 显示 → 命令回显正常 → 文件改动触发 Git status 200ms debounce 刷新（`GIT_STATUS_WATCH_DEBOUNCE`，PRD §Constraints）。三平台 `cargo test` / `vitest` 行为一致（Unix 路径零回归，Windows 路径走 cfg 分支）。汇总断言测试机械校验 Windows bundle 前置条件不漂移。

### 5.1 Required Reading

- [`docs/specs/phases/phase-6-integration-matrix.md`](../phases/phase-6-integration-matrix.md)（本 Phase §6 端到端 smoke 三条具体步骤）
- 上游 task spec（依赖全部 · 重点）：`docs/specs/tasks/task-2.2-conpty-spawn-io.md`（ConPTY spawn/IO · smoke 拉起依据）；`task-2.1-windows-shell-detection.md`（pwsh→cmd 探测链）；`task-3.4-fs-watch-windows.md`（Git status 200ms 刷新）；`task-5.1-windows-bundle.md`（nsis/msi target · 汇总断言依据）；`task-5.2-windows-ci-matrix.md`（windows-latest 矩阵）；`task-4.1-platform-windows-class.md` / `task-4.2-shortcut-display.md`（前端 platform）；`task-6.1-windows-test-gating.md`（测试门控 · smoke 前置）
- ADR：[`docs/decisions/adr-001-pty-windows-cfg-separation.md`](../../decisions/adr-001-pty-windows-cfg-separation.md) · [`adr-003-windows-default-shell-probe-chain.md`](../../decisions/adr-003-windows-default-shell-probe-chain.md) · [`adr-004-windows-installer-nsis-msi.md`](../../decisions/adr-004-windows-installer-nsis-msi.md) · [`adr-006-fs-watch-windows-notify-backend.md`](../../decisions/adr-006-fs-watch-windows-notify-backend.md)
- BDD：[`test/features/build-and-integration.feature`](../../../test/features/build-and-integration.feature)
- 实际源码 / 配置：`crates/app/tauri.conf.json`（`bundle.targets`）· `.github/workflows/ci.yml`（windows-latest 矩阵）

### 5.2 Imports

汇总断言测试 `crates/app/tests/windows_smoke_matrix.rs`（跨平台可跑）：

```rust
use std::fs;                 // 读 tauri.conf.json
use std::path::Path;
use serde_json::Value;       // 解析 tauri.conf.json（serde_json 已在 workspace dev/build 依赖中）
```

> 若 `serde_json` 不在 test 依赖范围，退化为字符串包含断言（`config.contains("\"nsis\"")`），不引入新 crate。GUI smoke 无 Rust import（手动 `pnpm tauri:dev`）。

### 5.3 函数签名

> 汇总断言测试骨架（跨平台 · 满足 §9 unit-test 强制）。GUI critical UX path 是手动 runtime-smoke，无函数签名。

```rust
// ── crates/app/tests/windows_smoke_matrix.rs ──

/// 读取 tauri.conf.json 内容（相对 crate manifest 定位）。
fn read_tauri_conf() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    fs::read_to_string(path).expect("tauri.conf.json 必须存在")
}

/// AC1 前置：Windows bundle targets 含 nsis + msi。
// SCEN-6.2.1 / AC1
#[test]
fn test_6_2_1_tauri_conf_has_windows_bundle_targets() {
    let conf = read_tauri_conf();
    let v: Value = serde_json::from_str(&conf).expect("tauri.conf.json 应为合法 JSON");
    let targets = v["bundle"]["targets"].as_array()
        .expect("bundle.targets 应为数组");
    let names: Vec<&str> = targets.iter().filter_map(|t| t.as_str()).collect();
    assert!(names.contains(&"nsis"), "Windows 安装包前置：targets 须含 nsis · 实际 {names:?}");
    assert!(names.contains(&"msi"), "Windows 安装包前置：targets 须含 msi · 实际 {names:?}");
}

/// AC4 前置：Windows smoke checklist 文档存在且非空（防空 smoke）。
// SCEN-6.2.4 / AC4
#[test]
fn test_6_2_2_windows_smoke_checklist_present() {
    // 断言本 task spec §6 smoke checklist 或 runtime-evidence 索引存在且非空
    // （路径在实施时按实际归档位置确定 · 保证 GUI smoke 有可追溯 checklist）
}
```

GUI critical UX path（手动 · `pnpm tauri:dev`）的"签名"是 §6 端到端 smoke 三步（起窗 / Tab+ConPTY / Git status 刷新）。

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Success Metrics 主要指标 / §Decisions Log D4): `crates/app/tauri.conf.json` 的 `bundle.targets` 含 `nsis` + `msi`（Windows 安装包前置条件，由汇总断言测试机械校验）。
- [ ] **AC2** (PRD §Success Metrics 次要指标 PTY runtime smoke / §User Flow happy path): Windows 本机 `pnpm tauri:dev` 起窗，新建 Tab 经 ConPTY 拉起 pwsh（或回落 cmd）看到 prompt，`git --version` / `echo` 命令回显正确（§2.14 本机实跑 · 截图证据）。
- [ ] **AC3** (PRD §Success Metrics 次要指标 Git status 实时刷新 / §Constraints 性能): Windows workspace 内改文件后，Git status 徽章 200ms 内刷新（fs_watch Windows backend 生效 · 本机实跑 · 截图/录屏证据）。
- [ ] **AC4** (PRD §Success Metrics 反指标): 三平台测试矩阵零回归——macOS + Ubuntu `cargo test --workspace` + `pnpm --filter @vibestation/web exec vitest run` 仍 100% 绿；Windows `cargo test --workspace` 0 panicked（与 Task 6.1 门控基线一致）。
- [ ] **AC5** (本 task 新增): 新增汇总断言测试 `crates/app/tests/windows_smoke_matrix.rs` 跨平台可跑并通过（满足 §9 unit-test 强制 + 机械校验 Windows 链路前置条件不漂移）。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1: tauri.conf nsis+msi target | SCEN-6.2.1 | TEST-6.2.1 `test_6_2_1_tauri_conf_has_windows_bundle_targets` | N/A | `cargo test --test windows_smoke_matrix`（断言 nsis+msi）| Not Started |
| AC2: ConPTY pwsh/cmd prompt + 回显 | SCEN-6.2.2 | N/A（GUI runtime · 无法稳定单测）| Windows runtime-smoke（手动 critical UX path）| `pnpm tauri:dev`（本机 · 新建 Tab + `git --version` 回显 · 截图）| Not Started |
| AC3: Git status 200ms 刷新 | SCEN-6.2.3 | N/A（GUI runtime）| Windows runtime-smoke（手动）| `pnpm tauri:dev`（本机 · 改文件观察徽章 200ms 刷新 · 录屏）| Not Started |
| AC4: 三平台矩阵零回归 | SCEN-6.2.4 | TEST-6.2.4 既有三平台测试集复跑 | 三平台 CI 矩阵 | `cargo test --workspace`（mac/Ubuntu/Win）+ `vitest run`（mac/Ubuntu）| Not Started |
| AC5: 汇总断言测试可跑 | SCEN-6.2.5 | TEST-6.2.5 `windows_smoke_matrix.rs`（`test_6_2_1_*` + `test_6_2_2_*`）| N/A | `cargo test --test windows_smoke_matrix`（全 passed）| Not Started |

## 8. Risks

- ConPTY 端到端 smoke 在本机 hang / 漏 exit / 读不到尾部输出（PRD §Technical Risks R1）。缓解：Windows reader 用 `child.try_wait()` + 阻塞读循环（Phase 2 落地）；本机 Windows 11 实跑验证退出与回显；保留 Unix mio 路径不动。
- CI 无法做 GUI runtime 验证（PRD R5）。缓解：CI 限 build + 单元/集成 + bundle 产物校验（AC1/AC5 机械断言）；GUI critical UX path（AC2/AC3）靠本机 §2.14 实跑 + 截图证据，CI 不门禁 GUI。
- bundle / WiX / WebView2 CI 环境缺依赖致 `tauri build` 失败（PRD R2）。缓解：本 task 只机械断言 `tauri.conf.json` 含 nsis/msi target（不实际 bundle）；实际 bundle 产物校验在 Phase 5 / 可选 CI job。
- 三平台零回归校验遗漏（只在 Windows 跑没在 mac/Linux 复跑）→ Unix 回归漏网（PRD R3）。缓解：AC4 强制 reviewer / CI 矩阵在 macOS + Ubuntu 复跑并与基线比对。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile  <!-- 与 adapter §Commands Install 一致 -->
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test --workspace  <!-- 强制：含新增 crates/app/tests/windows_smoke_matrix.rs（跨平台可跑）；三平台复跑 -->
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Runtime smoke**: pnpm tauri:dev  <!-- 本机 Windows 11 · critical UX path：起窗 + Tab ConPTY pwsh/cmd prompt + git --version 回显 + 改文件 Git status 200ms 刷新 · 截图/录屏/日志证据归档 §10 -->

> 三平台零回归（AC4）：reviewer / CI 矩阵在 macOS + Ubuntu 复跑 `cargo test --workspace` + `pnpm --filter @vibestation/web exec vitest run`，与 Phase 前基线比对。adapter Coverage = N/A，无 E2E 框架（GUI 走 runtime-smoke 手动）。

## 10. Completion Notes

- **完成日期**：<TBD-after-impl>
- **改动文件**：<TBD-after-impl>
- **commit 列表**：<TBD-after-impl>
- **§9 Verification 结果**：
  - install: <TBD-after-impl>
  - typecheck: <TBD-after-impl>
  - unit-test: <TBD-after-impl>
  - build: <TBD-after-impl>
  - lint: <TBD-after-impl>
  - runtime-smoke: <TBD-after-impl>
- **剩余风险 / 未做项**：<TBD-after-impl>
- **下游 task 影响**：<TBD-after-impl>
