# Project Development Adapter

> S2V Development 项目适配层。AI agent 进入项目的第一份必读文件。
> 一旦项目结构、命令、约束发生变化，立即更新本文件（s2v §4.4）。
>
> ⚠️ **范围说明**：本 adapter 由 `/s2v-init` 为 **Windows 适配工作流（feat/windows-support 分支）** 生成，与项目既有治理（`CLAUDE.md` 决策表 + `docs/tasks/` MVP 体系）**并存**。本 adapter 的 §Commands / §Source And Test Areas 服务于 Windows 适配 task 的 SDD→BDD→TDD→Verification 闭环；不取代 `CLAUDE.md` 的锁定决策表与禁区。
>
> 与 AGENTS.md 关系：本文件定义"项目结构与命令规范"，AGENTS.md 定义"协作行为约束"。加载顺序：AGENTS.md（协作）→ 本文件（结构）→ task spec（业务）。

---

## Project

- **Name**: `Vibestation Windows 适配`
- **Type**: `Desktop` <!-- Tauri 2 桌面终端 + Git 工作台 -->
- **Primary users / actors**: `Windows 11 上用 Claude/Codex CLI 的 AI-agent 开发者 + 项目 Windows 贡献者 + windows-latest CI`
- **Critical workflows**: `多 Tab 终端经 ConPTY 拉起 pwsh/cmd / JetBrains 级 Git 工作台(Log/Diff/实时 status) / 终端配置导入(%APPDATA%) / 多 agent 并行会话`

---

## Specification Locations

- **SDD home**: `docs/specs/`
- **Master spec**: `docs/prds/windows-support.prd.md`
- **Phase spec pattern**: `docs/specs/phases/phase-{N}-{name}.md`
- **Task spec pattern**: `docs/specs/tasks/task-{phase}.{seq}-{name}.md`
- **BDD acceptance home**: `test/features/*.feature`
- **ADR home**: `docs/decisions/adr-{N}-{title}.md`

---

## Source And Test Areas

> **路径 list 格式**：每行一个 git pathspec。下游 `/s2v-implement` 把整个 list 展开为 `git add` 多参数。
>
> **混合栈说明**：本项目 Windows 适配 ~85% 是 Rust（crates/core + crates/app），~15% 前端（web/src）。§Commands 以 Rust 为主槽位；前端（Phase 4）task 的 §9 Verification 直接列 pnpm 命令（lint/typecheck/vitest），实施时按 task §9 实际命令跑。

### Source areas

- `crates/core/src`
- `crates/app/src`
- `web/src`

### Unit test areas

- `crates/core/src`
- `crates/core/tests`
- `crates/app/tests`
- `web/src`

### Integration test areas

- `crates/core/tests`
- `crates/app/tests`

### E2E test areas

- `N/A: 无独立 e2e 框架（GUI critical path 走 runtime-smoke：本机 Windows pnpm tauri:dev）`

### Other locations

- **BDD feature**: `test/features/*.feature`（Windows 适配场景文档，Scenario ID 映射到 Rust/vitest 测试）
- **Fixture areas**: 见下方 §Fixture 约定

### Test File Naming（本项目实际约定 · 覆盖默认 TS profile）

> Vibestation 是 Rust 为主 + SolidJS 前端的混合栈，覆盖 adapter 默认的 TS 命名建议。

| 测试类型 | 文件名 | 示例 |
|---|---|---|
| Rust 单元测试 | `#[cfg(test)] mod tests` 同源文件 | `crates/core/src/pty.rs` 内嵌 `mod tests` |
| Rust 集成测试 | `crates/<crate>/tests/<scenario>.rs` | `crates/core/tests/pty_scrollback_integration.rs` |
| 前端单元测试 | `<module>.test.ts` / `tests/**/*.test.ts` | `web/src/lib/pane-keyboard.test.ts` |
| BDD feature | `<module>.feature` | `test/features/pty.feature` |

### Fixture 约定

| Fixture 大小 / 用途 | 落地位置 | 示例 |
|---|---|---|
| 小 (<20 行) | inline（Rust `const` / TS literal） | `let cfg = "[font]\nsize=12";` |
| 中 (20-100 行) | `crates/core/tests/fixtures/<module>/<case>.<ext>` 或 `test/fixtures/<module>/` | `crates/core/tests/fixtures/config/alacritty-win.toml` |
| 大 (>100 行 / 跨 task) | `test/fixtures/shared/<purpose>.<ext>` | — |

**约束**：含 unicode / 反斜杠路径的 fixture 一律走文件；fixture 文件名 kebab-case 描述性。

### TEST-ID 落地约定

task spec §7 追踪表写 `TEST-X.Y.Z`，代码层落地：

```text
// SCEN-X.Y.Z / AC<N>
Rust:  #[test] fn test_x_y_z_<desc>()  // 注释含 TEST-X.Y.Z
TS:    it("TEST-X.Y.Z: <描述>", ...)
```

要求 TEST-ID 能被 grep 精确匹配。

---

## Commands

> 所有命令在项目根目录运行。字段名加粗、裸值、不加反引号 / 行尾注释。
>
> **混合栈**：以下是 Rust 主槽位。前端（Phase 4）task §9 直接列 `pnpm lint` / `pnpm typecheck` / `pnpm --filter @vibestation/web exec vitest run`，实施 agent 按 task §9 实际命令执行并记录到 §10。
>
> **平台说明**：Windows 适配的实施与验证在 Windows 11 本机进行（开发机即 Windows）；mac/Linux 回归由 reviewer / CI 矩阵保证。

- **Install**: pnpm install --frozen-lockfile
- **Lint**: cargo clippy --workspace --all-targets -- -D warnings
- **Typecheck**: cargo check --workspace
- **Unit Test**: cargo test --workspace
- **Integration tests**: N/A: 集成测试随 cargo test --workspace 一起跑（crates/*/tests/）
- **E2E tests**: N/A: 无独立 e2e 框架
- **Build**: cargo build --workspace
- **Coverage**: N/A: MVP 不强制覆盖率阈值
- **Runtime smoke**: pnpm tauri:dev

<!-- 字段顺序与 s2v_extract_verify_keys 固定执行序一致 -->

### Coverage 判读规则

本适配 MVP 阶段不强制覆盖率阈值（Coverage = N/A）。若未来引入，Rust 用 `cargo tarpaulin` 判读 `Coverage Results: X/Y (Z%)`，前端用 `vitest --coverage` 的 `% Lines` 列。

---

## Constraints

- **Runtime target**: `Rust stable ≥ 1.95 / Node 20+ / pnpm 9.15 / Tauri CLI 2.x；Windows 侧需 MSVC toolchain (x86_64-pc-windows-msvc) + WebView2 Runtime (evergreen)`
- **Supported platforms**: `Windows 11 x64 (MSVC) · macOS arm64/x64 · Ubuntu 24 LTS (X11/Wayland)`
- **Security requirements**: `Tauri ACL/CSP 不放松（沿用 capabilities/default.json + 现有 CSP）；shell 拉起过 sanitize::is_denied_windows_path；禁硬编码凭证/路径`
- **Performance requirements**: `Windows 冷启动 < 500ms（mac 基线 202ms / Ubuntu ~108ms）；Git status watch debounce 200ms；PTY visible throughput 不显著低于 Unix`
- **Compatibility requirements**: `所有 Windows 改动走 #[cfg(target_os)] 分支或运行期平台判断，mac/Linux 行为零回归；DB schema 不变（app_local_data_dir 已跨平台）；config import 向后兼容已有 mac/Linux 路径`
- **Release constraints**: `新增 Windows .msi(WiX)+.exe(NSIS)，先 unsigned（对齐 macOS alpha，签名推 Windows GA）；保持 mac .dmg + Linux .deb/.AppImage；CI windows-latest 矩阵`

---

## Workflow

- **Collaboration Tier**: `solo`
  <!-- solo：单分支(feat/windows-support)无人值守 · 主 agent 兼 Arbiter + 调度 subagent 实施 · 直接在分支三段 commit，不开 per-task worktree/PR。整个分支最终作为一个 PR 合入 main（遵守 Vibestation 禁止直推 main）。 -->
  Overrides:
    - unattended: true   <!-- /goal 无人值守模式：主 agent 自答 Draft→Ready 审核 GATE（以 Arbiter 身份，基于 Windows 缺口调研证据填实业务字段） -->
    - main-branch: feat/windows-support   <!-- solo 的"main"在本工作流 = 适配 feature 分支，非 repo main -->

---

## Phase 状态索引

> 与 Master Spec §Implementation Phases 同步。Status 取值：`Draft / Ready / In Progress / Done / Blocked / Waived`。

| # | Phase | Phase Spec | Status | Tasks | Worktree（仅 team）|
|---|---|---|---|---|---|
| 1 | `foundation-build` | `docs/specs/phases/phase-1-foundation-build.md` | Draft | 3 | `-` |
| 2 | `shell-runtime` | `docs/specs/phases/phase-2-shell-runtime.md` | Draft | 2 | `-` |
| 3 | `terminal-integration` | `docs/specs/phases/phase-3-terminal-integration.md` | Draft | 4 | `-` |
| 4 | `frontend-platform` | `docs/specs/phases/phase-4-frontend-platform.md` | Draft | 2 | `-` |
| 5 | `build-package-ci` | `docs/specs/phases/phase-5-build-package-ci.md` | Draft | 3 | `-` |
| 6 | `integration-matrix` | `docs/specs/phases/phase-6-integration-matrix.md` | Draft | 2 | `-` |

## Task 总索引

> Status 取值：`Draft / Ready / In Progress / Done / Blocked / Waived`。

| Task | 模块 | Spec 文件 | Status | 依赖 / Phase 内顺序 | Worktree（仅 team）|
|---|---|---|---|---|---|
| 1.1 | pty | docs/specs/tasks/task-1.1-pty-platform-split.md | Done | Phase 1 首 · 解锁编译 | `-` |
| 1.2 | app-home | docs/specs/tasks/task-1.2-home-dir-helper.md | Done | 可与 1.1 并行（不同文件域）| `-` |
| 1.3 | app-settings | docs/specs/tasks/task-1.3-shell-default-setting.md | Done | 依赖 1.1 编译 | `-` |
| 2.1 | pty-shell | docs/specs/tasks/task-2.1-windows-shell-detection.md | Done | 依赖 1.1 | `-` |
| 2.2 | pty-conpty | docs/specs/tasks/task-2.2-conpty-spawn-io.md | Done | 依赖 2.1 | `-` |
| 3.1 | external_term | docs/specs/tasks/task-3.1-external-term-windows.md | Draft | 依赖 1.1 | `-` |
| 3.2 | config_import | docs/specs/tasks/task-3.2-config-import-paths.md | Draft | 依赖 1.2 | `-` |
| 3.3 | keybinding | docs/specs/tasks/task-3.3-keybinding-platform.md | Draft | 依赖 1.1 | `-` |
| 3.4 | fs_watch | docs/specs/tasks/task-3.4-fs-watch-windows.md | Draft | 依赖 1.1 | `-` |
| 4.1 | web-platform | docs/specs/tasks/task-4.1-platform-windows-class.md | Draft | 纯前端 · 无 Rust 依赖 | `-` |
| 4.2 | web-shortcuts | docs/specs/tasks/task-4.2-shortcut-display.md | Draft | 依赖 4.1 | `-` |
| 5.1 | tauri-bundle | docs/specs/tasks/task-5.1-windows-bundle.md | Draft | 依赖 1.1 | `-` |
| 5.2 | ci | docs/specs/tasks/task-5.2-windows-ci-matrix.md | Draft | 依赖 1.1, 2.1 | `-` |
| 5.3 | app-window | docs/specs/tasks/task-5.3-prepare-titlebar.md | Draft | 依赖 1.1 | `-` |
| 6.1 | tests | docs/specs/tasks/task-6.1-windows-test-gating.md | Draft | 依赖 2.x/3.x | `-` |
| 6.2 | integration | docs/specs/tasks/task-6.2-windows-smoke-matrix.md | Draft | 依赖全部 | `-` |

## ADR 索引

> ADR 状态机：`Proposed / Accepted / Deprecated / Superseded`。

| # | Title | Status | File |
|---|---|---|---|
| 001 | PTY Windows 适配用 cfg 分离 + portable-pty ConPTY | Accepted | docs/decisions/adr-001-pty-windows-cfg-separation.md |
| 002 | 跨平台家目录用 dirs crate | Accepted | docs/decisions/adr-002-cross-platform-home-dir-dirs.md |
| 003 | Windows 默认 shell 探测链 pwsh→powershell→cmd | Accepted | docs/decisions/adr-003-windows-default-shell-probe-chain.md |
| 004 | Windows 安装包 NSIS 主 + MSI 辅（unsigned MVP）| Accepted | docs/decisions/adr-004-windows-installer-nsis-msi.md |
| 005 | Windows 测试门控策略（cfg + ignore + 专测）| Accepted | docs/decisions/adr-005-windows-test-gating-strategy.md |
| 006 | fs_watch Windows 启用 notify backend | Accepted | docs/decisions/adr-006-fs-watch-windows-notify-backend.md |

## BDD Feature 索引

> 轻量 BDD（s2v §9.2）：`.feature` 作业务可读场景文档。

| Task(s) | Feature 文件 |
|---|---|
| 1.1, 2.1, 2.2 | test/features/pty.feature |
| 1.2, 1.3 | test/features/app-foundation.feature |
| 3.1 | test/features/external-term.feature |
| 3.2, 3.3 | test/features/config-import.feature |
| 3.4 | test/features/fs-watch.feature |
| 4.1, 4.2 | test/features/frontend-platform.feature |
| 5.1, 5.2, 5.3, 6.1, 6.2 | test/features/build-and-integration.feature |
