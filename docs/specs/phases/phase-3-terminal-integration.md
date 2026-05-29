# Phase 3: terminal-integration

**Status**: Done

> Phase Spec 按 S2V §8.2 八项结构。本 phase 是 Windows 适配的「外设与集成」层：把外部终端检测/启动、终端配置导入、keybinding 规范化、文件系统监视四个纯后端子系统补齐 Windows 分支，让 Windows 用户拿到与 mac/Linux 对等的集成体验。
>
> **Master Spec**：[`../../prds/windows-support.prd.md`](../../prds/windows-support.prd.md)

---

## 1. 阶段目标

让 Windows 11 上 Vibestation 的「终端外设集成」全链路可用：

- 外部终端检测/启动支持 Windows Terminal / conhost / pwsh（`external_term` 模块加 `Windows` 平台变体 + 启动配方）。
- 终端配置导入走 `%APPDATA%` 路径（Ghostty / Alacritty 加 Windows 路径分支；iTerm2 在 Windows 直接返回 not found）。
- keybinding 规范化在 Windows 上把 `win`/`super`/`meta` 映射到 `Ctrl`（而非 `Cmd`），内置快捷键返回 `Ctrl+X`，冲突检测正确匹配 `Ctrl+T`。
- Git status 在 Windows 实时刷新（`fs_watch` 移除 `UnsupportedPlatform` 短路，启用 `notify` 的 `ReadDirectoryChangesW` backend）。

所有改动走 `#[cfg(target_os)]` / 运行期平台判断分支，mac/Linux 行为零回归。

## 2. 业务价值

- **配置迁移无门槛**：Windows 用户从已装的原生 Alacritty / Ghostty（配置在 `%APPDATA%`）导入字体/快捷键，`Ctrl+T` 被正确识别为与内置冲突，而非被误规范化成 `Cmd+T` 后冲突检测静默失效（PRD §Users 场景 3）。
- **Git 工作台实时性对等**：在 pwsh Tab 里 `git commit` / 改文件后，Git status 徽章 200ms 内刷新（PRD §Success Metrics 次要指标「Git status 实时刷新」、PRD §Users 场景 2）。
- **外部终端可弹出**：装了 Windows Terminal / Alacritty 时 `external_term_list()` 返回非空且 `launch` 能拉起（PRD §Success Metrics 次要指标「外部终端列表非空」）。
- **零回归承诺**：四个子系统都是纯后端 cfg 分支叠加，不动 Unix 路径，保住 PRD §Success Metrics 反指标「不能为 Windows 适配牺牲 mac/Linux 现有行为」。

## 3. 涉及模块

| 模块 | 文件 | Windows 改动要点 |
|---|---|---|
| `external_term` | `crates/core/src/external_term/detect.rs` · `launch.rs` · `env_filter.rs` | `DetectionPlatform::Windows` + `Platform::Windows` 变体 · TERMINALS Windows 条目 · 启动配方 · `where.exe` 探测 · WHITELIST 加 `COMSPEC`/`PATHEXT` |
| `config_import` | `crates/core/src/config_import/ghostty.rs` · `alacritty.rs` · `iterm2.rs` · `ipc.rs` · `crates/app/src/lib.rs` | `%APPDATA%` 路径分支（保 `.config` fallback）· iTerm2 Windows not found · `prettify_home_path` 用 `USERPROFILE` · `home_dir_or_root` 复用 1.2 helper |
| `keybinding` | `crates/core/src/config_import/keybinding.rs` | `classify_token`/`canonicalize_keybinding`/`vibestation_builtins` 平台感知 · Windows 上 win/super/meta → Ctrl · 内置 `Ctrl+X` |
| `fs_watch` | `crates/core/src/fs_watch.rs` | 移除 Windows `UnsupportedPlatform` 短路 · 启用 notify Windows backend · 保持 200ms debounce + `.git/index.lock` 排除契约 |

## 4. 任务清单

| Task | 模块 | Spec | 依赖 | Status |
|---|---|---|---|---|
| 3.1 | external_term | [`../tasks/task-3.1-external-term-windows.md`](../tasks/task-3.1-external-term-windows.md) | 1.1 | Ready |
| 3.2 | config_import | [`../tasks/task-3.2-config-import-paths.md`](../tasks/task-3.2-config-import-paths.md) | 1.2 | Ready |
| 3.3 | keybinding | [`../tasks/task-3.3-keybinding-platform.md`](../tasks/task-3.3-keybinding-platform.md) | 1.1 | Ready |
| 3.4 | fs_watch | [`../tasks/task-3.4-fs-watch-windows.md`](../tasks/task-3.4-fs-watch-windows.md) | 1.1 | Ready |

## 5. 依赖关系

- **上游**：Phase 1 `foundation-build`（Task 1.1 解锁 Windows 编译 + 提供 `#[cfg(target_os)]` 分支范式；Task 1.2 提供跨平台 `home_dir()` helper）。本 phase 全部 4 task 依赖 Phase 1 编译通过。
  - Task 3.1 / 3.3 / 3.4 依赖 1.1（编译基线）。
  - Task 3.2 依赖 1.2（home_dir helper 复用）。
- **Phase 内**：4 个 task 文件域互不交叠（external_term / config_import / keybinding / fs_watch 各自独立），可并行实施。
- **可并行**：本 phase 整体可与 Phase 4 `frontend-platform` 并行（纯后端 Rust，与前端 TS 文件域不交叠 — PRD §Implementation Phases 标注）。
- **下游**：Phase 6 `integration-matrix` 依赖本 phase（Windows-gated 测试集含 `env_filter` cfg-gate / 外部终端 / config import / fs_watch 集成）。

## 6. 阶段级验收标准 + 端到端 smoke

阶段级验收（4 个 task 全 Done 后成立）：

- [ ] **P3-AC1**：Windows 上 `cargo build --workspace` 0 错误，本 phase 改动的 4 个模块全部编译通过。
- [ ] **P3-AC2**：Windows 上 `cargo test --workspace` 全绿 — Windows 专属新增测试（external_term Windows 启动配方 / config_import `%APPDATA%` 路径 / keybinding Ctrl 映射 / fs_watch Windows backend）通过，Unix-only 测试按 `#[cfg(unix)]` 正确跳过。
- [ ] **P3-AC3**：mac/Linux 上 `cargo test --workspace` 零回归（所有 Windows 改动走 cfg 分支，Unix 路径行为字节级不变）。
- [ ] **P3-AC4**：`cargo clippy --workspace --all-targets -- -D warnings` 0 warning（含 Windows cfg 分支代码）。

端到端 smoke（Windows 11 本机，PRD §Success Metrics 次要指标）：

- **windows**：`cargo build --workspace` 0 错误 + `cargo test --workspace` Windows-gated 测试集全绿。
- **windows**：`cargo test -p vibestation_core external_term::` 通过；`build_launch_command("windows-terminal", cwd, "pwsh.exe", Platform::Windows)` 返回含 `wt.exe` 的命令。
- **windows**：`cargo test -p vibestation_core config_import::` 通过；构造 `%APPDATA%/alacritty/alacritty.toml` 后 `scan_all_sources()` 返回该源 `path_exists=true`。
- **windows**：`cargo test -p vibestation_core keybinding::` 通过；`canonicalize_keybinding("win+t")` 在 Windows 返回 `Ctrl+T`、在 macOS 返回 `Cmd+T`。
- **windows（runtime · §2.14 本机实跑）**：`pnpm tauri:dev` 打开 workspace（git repo），改一个文件，Git status 徽章 200ms 内更新（fs_watch Windows backend 生效，PRD §Success Metrics「Git status 实时刷新」）。
- **mac/Linux（回归）**：`cargo test --workspace` 仍 100% 绿（CI 矩阵 + reviewer 本机）。

## 7. 阶段级风险

- **R-P3-1（来自 PRD §Technical Risks R4 · 路径/UNC/编码）**：Windows 反斜杠 / `%APPDATA%` / UNC 路径在 config import 扫描时拼错或 round-trip 不一致。缓解：config import 路径用 `Path::join` + 复用 1.2 `home_dir()`（已含 `dirs` fallback）；含 unicode/反斜杠的 fixture 走文件而非 inline（adapter §Fixture 约定）。
- **R-P3-2（来自 PRD §Technical Risks R3 · mac/Linux 回归）**：keybinding `classify_token` 平台分支写错，把 Unix 上 `win/super/meta → Cmd` 的现有语义也改坏。缓解：TDD 先写 RED 锁住 macOS 上 `Meta+t → Cmd+T` 不变（已有测试 `canonical_meta_alias_for_cmd`），再加 Windows 分支；mac/Linux 全量 `cargo test` 必绿。
- **R-P3-3（fs_watch ReadDirectoryChangesW · 来自 PRD §Technical Risks R1 旁系）**：notify Windows backend 事件粒度 / debounce 行为与 Unix 不同，可能漏事件或重复触发。缓解：保持 200ms debounce + `.git/index.lock` 排除契约不变；先写 Windows backend 启动/回调集成测试再实现；§2.14 本机 runtime 实跑验证 200ms 刷新。
- **R-P3-4（external_term 启动副作用）**：`build_launch_command` 是纯函数（只构造命令不执行），但 detect 的 `where.exe` 探测在 CI headless 环境可能行为不稳。缓解：检测/构造逻辑用注入 context（`TerminalDetectionContext`）做单元测试，避免依赖真实 PATH；实际 launch 在 §2.14 本机手动验。

## 8. 阶段级 Definition of Done

- [ ] Task 3.1 / 3.2 / 3.3 / 3.4 全部 Status = Done（§10 Completion Notes 已回填）。
- [ ] P3-AC1 ~ P3-AC4 阶段级验收标准逐项确认（raw output 留痕）。
- [ ] 端到端 smoke 全部执行：Windows build + test 全绿；§2.14 本机 runtime 验证 Git status 200ms 刷新有证据。
- [ ] mac/Linux 回归：`cargo test --workspace` 100% 绿（反指标硬约束）。
- [ ] 4 个 task 的 BDD feature（`test/features/external-term.feature` · `config-import.feature` · `fs-watch.feature`）场景已映射到 Rust 测试。
- [ ] 关联 ADR（adr-006 fs_watch Windows backend）状态正确；keybinding / external_term 的平台分支决策无新增 ADR 触发（复用 1.1 cfg 范式）。
- [ ] 无无主 TODO / placeholder 留在主线；`preflight` 检查 §6 已填实。
