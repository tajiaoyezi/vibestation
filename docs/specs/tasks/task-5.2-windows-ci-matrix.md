# Task `5.2`: `windows-ci-matrix`

**Status**: Ready

> Allowed values: `Draft` · `Ready` · `In Progress` · `Blocked` · `Waived` · `Done`。
> 本项目无人值守 solo 模式：主 agent 兼 Arbiter，业务字段已据 Windows 缺口调研（`spike-tmp/win-survey.json`）+ 实际 `.github/workflows/ci.yml` 填实，非编造，故初始即 Ready。

**Priority**: P1
**Owner**: 主 agent
**Related Phase**: Phase 5 · build-package-ci
**Dependencies**: 依赖 1.1（pty-platform-split · Windows 编译通过）、2.1（windows-shell-detection · windows-latest `cargo test --workspace` 跑 shell 相关测试才有意义）

## 1. Background

`.github/workflows/ci.yml` 所有 job 当前 `runs-on: ubuntu-latest`（`rust-build` line 53、`frontend-build` line 117、`runtime-evidence-validator`、`pre-code-status`）。**没有 windows runner**，Windows 编译错误"永远不会在合入前被发现"（PRD §Problem Statement / survey ci.yml finding · severity high）。

工作流当前仅 `on: workflow_dispatch:` 触发（ADR-021 既定运营模型：私有仓 + 非 Pro，为省 Actions 分钟关 PR/push 触发，唯一有效质量门 = 本地 gate + reviewer 复跑）。本 task **保持 `workflow_dispatch` 触发不变**，仅把 `rust-build` job 升级为跨平台矩阵——让"需要时手动 dispatch"能覆盖 Windows leg。

ADR-005 已定 Windows 测试门控策略：Unix-only 行为测试用 `#[cfg(unix)]` / `#[cfg_attr(windows, ignore)]`，补 Windows 专测；CI windows-latest 跑 `cargo test --workspace` 自动跳过 Unix-only 用例。

## 2. Goal

任务完成后应成立的事实：

1. `rust-build` job 升级为 `strategy.matrix.os=[ubuntu-latest, windows-latest]`，`runs-on: ${{ matrix.os }}`。
2. Windows leg 跳过 Ubuntu 专属 step（`apt-get install` Tauri deps、任何 xvfb/X11 相关），依赖 windows-latest runner 预装的 MSVC toolchain + WebView2 Runtime。
3. Windows leg 跑 `cargo build --workspace` + `cargo test --workspace`（Windows-gated 测试集，Unix-only 用例 ignored 而非 panicked）+ Tauri `--no-bundle` 编译 smoke；nsis bundle 作为可选 step（失败不阻断 build+test gate，对齐 PRD R2）。
4. `on: workflow_dispatch:` 触发保持不变（ADR-021）；macOS/Ubuntu 现有 leg 的所有既有 step（apt deps / corepack / clippy / fmt / test / tauri smoke）零回归。

## 3. Scope

### In Scope

- `.github/workflows/ci.yml`：`rust-build` job 加 `strategy.matrix.os: [ubuntu-latest, windows-latest]` + `runs-on: ${{ matrix.os }}`。
- `.github/workflows/ci.yml`：Ubuntu 专属 step（`Install system deps (Tauri on Ubuntu)` apt-get）加 `if: runner.os == 'Linux'` 条件门控。
- `.github/workflows/ci.yml`：Windows leg 验编译——`cargo build --workspace` + `cargo test --workspace` + Tauri `--no-bundle`（`pnpm tauri:build:smoke`）；可选 `pnpm tauri:build --bundles nsis` step 带 `continue-on-error: true`（ADR-021 不阻断）。
- 轻量单元测试（s2v unit-test 强制 · CI/bundle task 也要有 Unit）：解析 `ci.yml`，断言 `rust-build` job 的 matrix 含 `windows-latest`。
- 跨平台 cache key 已用 `${{ runner.os }}-cargo-...`（line 81），矩阵化后天然按 OS 分桶——确认无需改。

### Out Of Scope

- 改触发条件（恢复 `push` / `pull_request` 触发）——ADR-021 决议，需另开 ADR（PRD 约束 + ADR-021 明确）。
- `frontend-build` / `runtime-evidence-validator` / `pre-code-status` job 矩阵化（前端 lint/typecheck 平台无关，无需 Windows leg）。
- bundle 产物格式定义（属 Task 5.1）。
- Windows 测试代码本身的 `#[cfg]` / `ignore` 标注（属 Phase 6 / Task 6.1 windows-test-gating；本 task 仅消费已门控的测试集跑矩阵）。
- 代码签名 / Authenticode（PRD Out of Scope）。

## 4. Users / Actors

- **项目维护者 / CI**：手动 `gh workflow run ci.yml` 后，在 Actions 看到 windows-latest leg 跑 build+test，合入前拦截 Windows 编译错误。
- **Windows 贡献者**：PR 改动经维护者 dispatch 验证 Windows leg 绿，才合入 `feat/windows-support`。

## 5. Behavior Contract

GitHub Actions matrix：`strategy.matrix.os` 把单 job 展开为多 leg（每 OS 一个），`runs-on: ${{ matrix.os }}` 选 runner，step 用 `if: runner.os == '<OS>'` 做平台分支。Windows runner（`windows-latest`）预装 MSVC + WebView2 + Node，无需 apt。`cargo test --workspace` 在 Windows 上自动跳过 `#[cfg(unix)]` 测试、`#[cfg_attr(windows, ignore)]` 标记的测试显示 `ignored`。

### 5.1 Required Reading

- [Phase 5 spec](../phases/phase-5-build-package-ci.md)
- [task-1.1-pty-platform-split.md](./task-1.1-pty-platform-split.md)（上游：Windows 编译通过）
- [task-2.1-windows-shell-detection.md](./task-2.1-windows-shell-detection.md)（上游：windows-latest test 跑 shell 探测）
- [ADR-005 Windows 测试门控策略（cfg + ignore + 专测）](../../decisions/adr-005-windows-test-gating-strategy.md)
- BDD：[test/features/build-and-integration.feature](../../../test/features/build-and-integration.feature)
- 参考现状：`.github/workflows/ci.yml`（`rust-build` job line 51-112 · 触发 line 15-16 `workflow_dispatch`）

### 5.2 Imports

- 测试侧（解析 `ci.yml` 的轻量单元测试，落 `crates/app/tests/`）：
  - `serde_yaml`（若 workspace 未引入，则用纯字符串断言兜底——见 §5.3 备选）；优先 `use serde_yaml::Value;`
  - `std::fs`（读 workflow 文件）、`std::path::PathBuf`（相对 `env!("CARGO_MANIFEST_DIR")` 上溯到 repo 根 `.github/workflows/ci.yml`）
- 生产侧：YAML workflow 改动，无 Rust import 新增。

### 5.3 函数签名

本 task 主体是 `ci.yml` 声明式 matrix 改动，无生产函数签名变更。新增轻量单元测试骨架（平台无关 · 验证 workflow 文件内容）：

```rust
// crates/app/tests/ci_matrix.rs（新增）
// SCEN-5.2.1 / AC1 · 断言 rust-build job 含 windows-latest 矩阵

use std::path::PathBuf;

/// 定位 repo 根的 .github/workflows/ci.yml
/// CARGO_MANIFEST_DIR = crates/app → 上溯两级到 repo 根
fn read_ci_workflow() -> String {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let path = repo_root.join(".github").join("workflows").join("ci.yml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn test_5_2_1_rust_build_has_windows_matrix() {
    // TEST-5.2.1 · 字符串断言兜底（不强依赖 serde_yaml）：
    // matrix 化后 ci.yml 必含 "windows-latest" 与矩阵占位 "${{ matrix.os }}"
    let yaml = read_ci_workflow();
    assert!(
        yaml.contains("windows-latest"),
        "ci.yml rust-build job must include windows-latest in matrix"
    );
    assert!(
        yaml.contains("matrix.os"),
        "ci.yml must use ${{{{ matrix.os }}}} runs-on for cross-platform leg"
    );
}

#[test]
fn test_5_2_2_workflow_dispatch_trigger_preserved() {
    // TEST-5.2.2 · ADR-021 零回归：触发仍是 workflow_dispatch（不恢复 push/pull_request）
    let yaml = read_ci_workflow();
    assert!(
        yaml.contains("workflow_dispatch"),
        "ci.yml must keep workflow_dispatch trigger (ADR-021)"
    );
}
```

> 若后续引入 `serde_yaml` 做结构化断言（精确定位 `jobs.rust-build.strategy.matrix.os` 数组），可升级 TEST-5.2.1；MVP 阶段字符串断言已满足 s2v unit-test 强制门槛。

## 6. Acceptance Criteria

- [ ] **AC1** (PRD §Core Capabilities 5 · §Success Metrics 主要指标): `.github/workflows/ci.yml` 的 `rust-build` job 含 `strategy.matrix.os: [ubuntu-latest, windows-latest]` 与 `runs-on: ${{ matrix.os }}`（轻量单元测试断言通过）。
- [ ] **AC2** (PRD §Decisions Log D5 · ADR-005): windows-latest leg `cargo test --workspace` 跑通——Unix-only 用例显示 `ignored` 而非 `panicked`，0 failed。
- [ ] **AC3** (PRD §Problem Statement · survey ci.yml finding): windows-latest leg `cargo build --workspace` 0 错误 + Tauri `--no-bundle` 编译 smoke（`pnpm tauri:build:smoke`）通过。
- [ ] **AC4** (PRD §Technical Risks R2): nsis bundle step 为可选（`continue-on-error: true` 或独立可选 job），其失败不阻断 build+test gate。
- [ ] **AC5** (ADR-021 · PRD §Constraints): `on: workflow_dispatch:` 触发保持不变；未恢复 `push`/`pull_request`（TEST-5.2.2 断言）。
- [ ] **AC6** (PRD §Success Metrics 反指标): macOS/Ubuntu leg 既有 step（apt deps via `if: runner.os == 'Linux'` / corepack / clippy / fmt / test / tauri smoke）零回归——Ubuntu leg 仍按改动前序列执行。

## 7. SDD / BDD / TDD Traceability

| Acceptance Criterion | BDD Scenario | TDD Test | Integration / E2E Test | Verification | Status |
|---|---|---|---|---|---|
| AC1 · rust-build 含 windows 矩阵 | SCEN-5.2.1 | TEST-5.2.1 | N/A | `cargo test -p vibestation_app --test ci_matrix` | Not Started |
| AC2 · windows-latest test 全绿（unix ignored）| SCEN-5.2.2 | N/A（CI leg 层） | CI：windows-latest `cargo test --workspace` | 手动 `gh workflow run ci.yml` → Actions windows leg | Not Started |
| AC3 · windows build + --no-bundle smoke | SCEN-5.2.2 | N/A | CI：windows-latest `cargo build` + `tauri:build:smoke` | Actions windows leg build step | Not Started |
| AC4 · bundle step 可选不阻断 | SCEN-5.2.3 | N/A | CI：bundle step `continue-on-error` | Actions：bundle step fail 时 job 仍绿 | Not Started |
| AC5 · workflow_dispatch 触发保留 | SCEN-5.2.4 | TEST-5.2.2 | N/A | `cargo test -p vibestation_app --test ci_matrix` | Not Started |
| AC6 · mac/Linux leg 零回归 | SCEN-5.2.5 | N/A | CI：ubuntu-latest leg 全绿 | 手动 dispatch → ubuntu leg step 序列对比 | Not Started |

## 8. Risks

- **R-5.2-a**（关联 PRD R2）：windows-latest runner 缺 WiX / WebView2 致 `tauri build` bundle 失败。缓解：先 `--no-bundle` 验编译链路（必过 gate），bundle step `continue-on-error: true` 隔离（AC4）；WiX 由 Tauri 自动下载或后续显式装。
- **R-5.2-b**（关联 PRD R3）：matrix 化误伤 ubuntu leg（误把 apt step 对 Windows 也跑 / 漏 `if` 条件）。缓解：apt step 显式 `if: runner.os == 'Linux'`；改动前后 ubuntu leg step diff 审计（AC6）。
- **R-5.2-c**（关联 PRD R1）：Windows 上某些 PTY 集成测试 timing 不稳 hang。缓解：依赖 Task 6.1 测试门控（`#[cfg_attr(windows, ignore)]` 标 timing-sensitive 用例）；CI job 已有 `timeout-minutes: 25` 兜底。
- **R-5.2-d**（关联 PRD R5）：windows-latest headless 跑不了 GUI smoke。缓解：CI 限 build + unit/integration + `--no-bundle`；GUI critical path 靠本机 Windows（§2.14）。

## 9. Verification Plan

- **Install**: pnpm install --frozen-lockfile
- **Typecheck**: cargo check --workspace
- **Unit**: cargo test -p vibestation_app --test ci_matrix（随 `cargo test --workspace` 一并跑 · s2v unit-test 强制）
- **Build**: cargo build --workspace
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Runtime smoke**: 手动 `gh workflow run ci.yml` → GitHub Actions 观察 windows-latest leg `cargo build --workspace` + `cargo test --workspace` + `--no-bundle` smoke 全绿（raw log 截取记入 §10）
- **Manual**: Actions UI 核对：windows-latest leg 跑了 build/test、未跑 apt（`if: runner.os == 'Linux'` 跳过）；ubuntu-latest leg step 序列与改动前一致；触发仍是 workflow_dispatch

## 10. Completion Notes

<TBD-after-impl>
