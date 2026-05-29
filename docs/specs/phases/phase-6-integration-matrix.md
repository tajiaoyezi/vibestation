# Phase 6: integration-matrix

> Phase Spec（S2V §8.2 八项结构）· Windows 适配工作流 `feat/windows-support`。
> 本 Phase 是整条 Windows 适配链路的收口阶段：把前 5 个 Phase 落地的 `#[cfg(target_os)]` 分支拼起来做端到端验证，并锁住 mac/Linux 零回归。

**Status**: Ready

---

## 1. 阶段目标

让 Vibestation 在 Windows 11 上跑通端到端 critical UX path（pwsh/cmd 经 ConPTY 拉起 + Git status 实时刷新），同时把所有 Unix-only 行为测试按 `#[cfg(target_os)]` / `#[cfg_attr(windows, ignore)]` 门控，使 `cargo test --workspace` 在三平台都不出现 `panicked`，并验证 macOS/Ubuntu 现有测试矩阵零回归。

具体落地为两件事：

1. **测试门控（Task 6.1）**：cfg-gate 现存 Unix-only 测试（`/bin/sh`、`chmod 0o755`、`#!/bin/sh` hook、`/tmp`、`HOME`、`which`、`/etc/shells` 等硬假设），并按需补 Windows 专属测试，使 Windows 上 `cargo test --workspace` 显示 `ignored` 而非 `panicked`。
2. **端到端 smoke + 三平台矩阵（Task 6.2）**：Windows 本机 `pnpm tauri:dev` 跑通 Tab 创建 / ConPTY / Git status 刷新 critical UX path；校验 mac/Linux `cargo test` + `vitest` 仍全绿。

---

## 2. 业务价值

- 对 **Windows 11 上的 AI-agent 开发者**：第一次能在本机原生 Vibestation 完整跑通"多 Tab 终端 + 实时 Git 工作台"——`cargo test --workspace` 不再因 Unix 硬假设而 `panicked`，开发者可信地在 Windows 本机自测和贡献。
- 对 **项目维护者 / CI**：windows-latest 矩阵能稳定跑 build+test（Unix-only 测试自动 skip 而非崩），三平台 CI 全绿成为合入主线的可信门禁（PRD §Success Metrics 主要指标）。
- 对 **整条 Windows 适配工作流**：本 Phase 是唯一的集成兜底层——前 5 个 Phase 各自隔离严格 TDD，"拼起来能否集成"只在这里被端到端验证一次（S2V §8.2 C1）。

---

## 3. 涉及模块

| 模块 | 文件 | 本 Phase 改动 |
|---|---|---|
| tests（Unix-only 门控）| `crates/app/tests/shell_compat.rs` | `which` / `/bin/zsh` 候选路径 `#[cfg(unix)]` 或 `#[cfg_attr(windows, ignore)]` |
| tests | `crates/core/tests/git_ops_integration.rs` | `chmod 0o755` + `#!/bin/sh` hook 走 `#[cfg(unix)]`，或 Windows `.bat` 分支 |
| tests | `crates/core/tests/pty_pool_bench.rs` | `/tmp` → `std::env::temp_dir()`；`HOME` → `USERPROFILE` 兜底 |
| tests | `crates/core/tests/pty_scrollback_integration.rs` | `/bin/sh` shell 参数 `#[cfg]` 分离 |
| tests | `crates/core/src/external_term/env_filter.rs`（`#[cfg(test)] mod tests`）| fixture `/usr/bin` / `/bin/zsh` → Windows `C:\Windows\System32` / `cmd.exe` cfg 分离 |
| integration（汇总）| `crates/app/tests/`（新增 Windows smoke 汇总断言测试）+ `crates/app/tauri.conf.json`（验证侧只读断言）| 三平台矩阵零回归汇总 + Windows 端到端 smoke checklist |

> 本 Phase 不新增生产代码逻辑（生产侧 Windows 分支由 Phase 1-5 落地）；本 Phase 聚焦测试层门控 + 端到端集成验证。所有改动走 `#[cfg(target_os)]` / `#[cfg_attr]`，mac/Linux 测试行为零回归。

---

## 4. 任务清单

| Task | 模块 | 描述 | Spec | 依赖 |
|---|---|---|---|---|
| 6.1 | tests | cfg-gate Unix-only 测试 + 补 Windows 专测 · `cargo test --workspace` 在 Windows 显示 ignored 而非 panicked | [../tasks/task-6.1-windows-test-gating.md](../tasks/task-6.1-windows-test-gating.md) | Phase 2.x / 3.x |
| 6.2 | integration | Windows 端到端 smoke（ConPTY pwsh/cmd + Git status 200ms 刷新）+ 三平台测试矩阵零回归校验 | [../tasks/task-6.2-windows-smoke-matrix.md](../tasks/task-6.2-windows-smoke-matrix.md) | 全部（Phase 1-5）|

---

## 5. 依赖关系

- **上游（Phase 依赖）**：Phase 2（shell-runtime · ConPTY spawn/IO）+ Phase 3（terminal-integration · fs_watch Windows backend）+ Phase 4（frontend-platform · platform-windows class）+ Phase 5（build-package-ci · windows-latest 矩阵）必须全部 Done。
  - Task 6.1 依赖 Phase 2.x / 3.x（待门控的测试覆盖 PTY shell / external_term / config_import 行为，须先有对应生产分支）。
  - Task 6.2 依赖全部（端到端 smoke 串起 PTY + shell + fs_watch + 前端 + bundle 全链）。
- **Phase 内顺序**：6.1 先（门控使 `cargo test --workspace` 在 Windows 干净通过）→ 6.2 后（在干净测试基线上跑端到端 smoke + 三平台矩阵校验）。
- **下游**：本 Phase 是工作流终点。Done 后整条 `feat/windows-support` 分支作为一个 PR 合入 main（遵守 Vibestation 禁止直推 main）。

---

## 6. 阶段级验收标准

- [ ] **AC-P6.1**：Windows 上 `cargo test --workspace` 0 编译错误、0 `panicked`；Unix-only 测试显示为 `ignored`（而非 fail / panic）。
- [ ] **AC-P6.2**：Windows 上 `cargo build --workspace` 0 错误；Windows 专属 / Windows-gated 测试全绿。
- [ ] **AC-P6.3**：Windows 本机 `pnpm tauri:dev` 起窗，新建 Tab 经 ConPTY 拉起 pwsh（或回落 cmd）看到 prompt，`git --version` 回显正确；改 workspace 文件后 Git status 徽章 200ms 内刷新（PRD §Success Metrics 次要指标 · §2.14 本机实跑）。
- [ ] **AC-P6.4**：mac/Linux 三平台测试矩阵零回归——macOS + Ubuntu `cargo test --workspace` + `pnpm --filter @vibestation/web exec vitest run` 仍 100% 绿（PRD §Success Metrics 反指标）。
- [ ] **AC-P6.5**：本 Phase 全部 task §10 Completion Notes 已回填；本 Phase Status 翻 Done。

### 端到端 Smoke（C1 集成兜底 · 具体可执行）

> 本 Phase 是 Windows 适配工作流的最后阶段，§6 smoke 是整条链路唯一集成兜底，不是说明文字。Phase 收尾前（Task 6.2 完工前）必须实跑下列三条并记录证据。

1. **Windows 编译 + 测试 smoke**（windows-latest CI + 本机）：
   ```
   cargo build --workspace           # 期望 0 错误
   cargo test --workspace            # 期望 0 panicked · Unix-only 测试显示 ignored
   ```
   断言：build exit 0；test 输出含 `test result: ok` 且 ignored 计数 > 0（Unix-only 测试被门控），无 `panicked`。

2. **三平台零回归 smoke**（mac/Linux · reviewer / CI 矩阵）：
   ```
   # macOS + Ubuntu runner
   cargo test --workspace
   pnpm --filter @vibestation/web exec vitest run
   ```
   断言：两平台 cargo test 全绿（含原 `#[cfg_attr(target_os = "linux", ignore)]` 行为不变）；vitest 全绿。

3. **Windows 端到端 GUI smoke**（本机 H:\ Windows 11 · §2.14 critical UX path）：
   ```
   pnpm tauri:dev
   ```
   手动 path：起窗 → 新建 Tab → 观察 pwsh/cmd prompt → 输入 `git --version` 看回显 → 在 workspace 改一个文件 → 观察 Git status 徽章 200ms 内刷新。证据：截图 / 录屏 / 控制台日志记入 Task 6.2 §10 runtime-smoke。

---

## 7. 阶段级风险

| # | 风险 | 缓解（关联 PRD §Technical Risks）|
|---|---|---|
| RP6-1 | 门控写错把 Unix 路径也 skip 掉，导致 mac/Linux 测试覆盖静默缩水 | 门控只对 Windows 加 cfg/ignore；mac/Linux 测试计数前后比对（零回归校验）；reviewer 三平台意识（PRD R3）|
| RP6-2 | ConPTY 端到端 smoke 在本机 hang / 漏 exit / 读不到尾部输出 | Task 6.2 smoke 用 `child.try_wait()` 退出语义；本机 Windows 11 实跑验证（PRD R1）|
| RP6-3 | windows-latest headless 跑不了完整 GUI smoke | CI 限 build + 单元/集成测试；GUI critical UX path 靠开发者本机 §2.14 实跑并记录证据（PRD R5）|
| RP6-4 | 路径分隔符 / `/tmp` → `temp_dir()` 改写引入 Windows 路径 round-trip 偏差 | 用 `std::env::temp_dir()` 跨平台 API；含反斜杠 fixture 走文件而非 inline（PRD R4）|

---

## 8. 阶段级 Definition of Done

- [ ] Task 6.1 + Task 6.2 均 Status: Done（§10 Completion Notes 已回填）。
- [ ] §6 阶段级验收标准 AC-P6.1 ~ AC-P6.5 全部勾选。
- [ ] §6 端到端 smoke 三条全部实跑并记录证据（Windows 编译+测试 / 三平台零回归 / Windows GUI critical UX path）。
- [ ] Windows 上 `cargo test --workspace` 0 panicked；Unix-only 测试 ignored。
- [ ] mac/Linux `cargo test --workspace` + `vitest run` 零回归（与 Phase 前基线一致）。
- [ ] 所有改动走 `#[cfg(target_os)]` / `#[cfg_attr]`，无 Unix 行为变更。
- [ ] adapter §Phase 状态索引 + §Task 总索引中 Phase 6 / Task 6.1 / 6.2 Status 同步更新。
- [ ] 无无主 TODO / placeholder 留在主线；剩余风险已记录。
