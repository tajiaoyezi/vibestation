# Phase 5: build-package-ci

**Status**: Ready

> Phase Spec · 严格按 S2V standard §8.2 八项渲染。本 phase 让 Vibestation 在 Windows 上**产出可安装产物 + 进 CI 矩阵**，是 Windows 适配从"能编译能跑"走向"能分发能持续验证"的收口环节。
>
> 上游：依赖 Phase 1（foundation-build · `cargo build --workspace` 在 Windows 编译通过）与 Phase 2（shell-runtime · ConPTY 拉起 shell 可读写，CI 矩阵跑 test 才有意义）。

---

## 1. 阶段目标

在 Windows 11 (x64 MSVC) 上：

1. `pnpm tauri:build` 产出可安装的 `.exe`（NSIS，主）与 `.msi`（WiX，辅）安装包。
2. `.github/workflows/ci.yml` 的 `rust-build` job 升级为 `[ubuntu-latest, windows-latest]` 矩阵，在 windows-latest 上跑 `cargo build --workspace` + `cargo test --workspace`（Windows-gated 测试集自动跳过 Unix-only 用例）+ Tauri `--no-bundle` 编译 smoke。
3. `package.json` 的 `prepare` 脚本跨平台化——在 Windows PowerShell / cmd.exe 上也能正确配置 `git config core.hooksPath .githooks`，不再因 bash `2>/dev/null` 重定向语法静默失败。
4. 窗口装饰（`windowEffects`/`titleBarStyle`/`trafficLightPosition`/`macOSPrivateApi` 等 macOS 专属配置）在 Windows 上安全忽略或条件化，窗口正常渲染无 macOS 专属样式 artifact；`configure_title_bar` 的 Windows 分支安全（stub 或 Windows 原生 title bar）。

终态：Windows 用户能下载安装包安装运行；任何 PR 在合入前，Windows 编译错误能被 CI 矩阵拦截；Windows 开发者 clone 后 `pnpm install` 能正确装上 git hooks。

---

## 2. 业务价值

- **可分发**：没有 `.exe`/`.msi` bundle，Windows 用户拿不到安装包（PRD §Problem Statement：`tauri.conf.json` bundle targets 只有 `dmg`/`appimage`/`deb`，产不出 Windows 安装包）。本 phase 解锁 GitHub Release 附 Windows 产物（PRD §Constraints §发布）。
- **可持续验证**：没有 windows-latest CI runner，Windows 编译错误"永远不会在合入前被发现"（PRD §Problem Statement）。本 phase 让三平台 CI 全绿成为 PRD §Success Metrics 主要指标的机械门禁。
- **贡献者体验**：`prepare` 脚本失败 = Windows 开发者本地 commit 绕过 pre-push hook（PRD survey 中 medium 缺口）。修复后，Windows 贡献者 clone + `pnpm install` 即激活分支保护。
- **原生观感**：窗口装饰条件化让 Windows 窗口不残留 macOS 专属样式（PRD §Core Capabilities 5：窗口装饰平台条件化），对齐 mac/Linux 一等平台体验。

---

## 3. 涉及模块

| 模块 | 文件 | 本 phase 改动点 |
|---|---|---|
| tauri-bundle | `crates/app/tauri.conf.json` | `bundle.targets` 增 `nsis`+`msi`；确认/条件化 macOS 专属窗口装饰在 Windows 安全 |
| ci | `.github/workflows/ci.yml` | `rust-build` job 加 `strategy.matrix.os=[ubuntu-latest, windows-latest]`；Windows 跳过 apt deps + xvfb，加 MSVC/WebView2（runner 预装）；`--no-bundle` 编译 smoke + 可选 nsis bundle step |
| app-window | `package.json` + `crates/app/src/lib.rs` | `prepare` 脚本跨平台（node 一行 `child_process.execSync`）；`configure_title_bar` Windows 分支确认安全 |

> 已就位的 Windows 前提（survey `already_windows_ok`）：`crates/app/icons/icon.ico` 已存在；`crates/app/src/main.rs` 已设 `windows_subsystem="windows"`；`crates/app/Cargo.toml` 的 `tauri` feature `macos-private-api` 是 macOS 运行期 feature gate，不破坏 Windows build。

---

## 4. 任务清单

| Task | 模块 | Spec | 依赖 | 一句话范围 |
|---|---|---|---|---|
| 5.1 | tauri-bundle | [../tasks/task-5.1-windows-bundle.md](../tasks/task-5.1-windows-bundle.md) | 依赖 1.1 | `bundle.targets` 增 nsis+msi；macOS 专属窗口装饰在 Windows 安全忽略/条件化 |
| 5.2 | ci | [../tasks/task-5.2-windows-ci-matrix.md](../tasks/task-5.2-windows-ci-matrix.md) | 依赖 1.1, 2.1 | `rust-build` 升级 windows-latest 矩阵；`--no-bundle` 编译 smoke + 可选 bundle |
| 5.3 | app-window | [../tasks/task-5.3-prepare-titlebar.md](../tasks/task-5.3-prepare-titlebar.md) | 依赖 1.1 | `prepare` 脚本跨平台 node 化；`configure_title_bar` Windows 分支安全 |

---

## 5. 依赖关系

- **上游（必须先 done）**：
  - Phase 1 / Task 1.1（pty-platform-split）：`cargo build --workspace` 在 Windows 编译通过——否则 5.1 bundle 与 5.2 CI 矩阵都无法产出/验证。
  - Phase 2 / Task 2.1（windows-shell-detection）：Task 5.2 的 windows-latest `cargo test --workspace` 要跑到 shell 相关测试，依赖 2.1 的 Windows 探测链落地（否则测试集大面积红/无意义）。
- **Phase 内顺序**：5.1 / 5.2 / 5.3 文件域基本不交叠（`tauri.conf.json` / `ci.yml` / `package.json`+`lib.rs`），可并行；唯一交叠点是 5.1 与 5.3 都"触碰窗口装饰条目"——5.1 改 `tauri.conf.json` 的 windowEffects，5.3 改 `lib.rs` 的 `configure_title_bar`，二者通过文件边界自然隔离，无需串行。
- **下游**：Phase 6（integration-matrix）依赖本 phase 的 CI 矩阵（5.2）+ bundle 产物（5.1）作为三平台零回归验证的载体。

---

## 6. 阶段级验收标准

- [ ] **P5-AC1**：windows-latest 上 `cargo build --workspace` 0 错误（Task 5.2 矩阵跑通）。
- [ ] **P5-AC2**：windows-latest 上 `cargo test --workspace` 全绿——Windows-gated 测试集（Unix-only 用例经 `#[cfg(unix)]` / `#[cfg_attr(windows, ignore)]` 跳过）无 panic、无 fail（Task 5.2）。
- [ ] **P5-AC3**：本机 Windows 11 `pnpm tauri:build` 在 `src-tauri`/`target/release/bundle/nsis/` 产出 `.exe`、`bundle/msi/` 产出 `.msi`，安装包可安装并启动（Task 5.1）。
- [ ] **P5-AC4**：Windows 上 `pnpm install` 后 `git config core.hooksPath` == `.githooks`（`prepare` 脚本生效），且 macOS/Linux 上 `prepare` 行为零回归（Task 5.3）。
- [ ] **P5-AC5**：Windows 窗口启动后无 macOS 专属样式 artifact（无 traffic light 残留 / 标题栏正常），`configure_title_bar` Windows 分支不 panic（Task 5.3）。
- [ ] **P5-AC6（零回归）**：macOS/Ubuntu 现有 CI job 与 bundle（dmg/appimage/deb）行为零变化；所有改动走 `#[cfg(target_os)]` 分支或 matrix 条件 step（PRD §Success Metrics 反指标）。

**阶段级端到端 smoke**（phase 收口前必跑，证据记入末 task §10）：

1. **Windows 编译矩阵 smoke**：手动 `gh workflow run ci.yml`（或本机 `cargo build --workspace` + `cargo test --workspace`）→ windows-latest leg `cargo build --workspace` 0 错误 + `cargo test --workspace` 0 failed（Unix-only 用例 ignored 而非 panicked）。
2. **bundle 产物 smoke**：本机 Windows 11 `pnpm tauri:build` → `target/release/bundle/nsis/*.exe` + `target/release/bundle/msi/*.msi` 文件存在且 > 0 字节，双击 `.exe` 安装后 Vibestation 启动窗口正常渲染。
3. **prepare 跨平台 smoke**：Windows PowerShell 下 `pnpm install` → `git config --get core.hooksPath` 回显 `.githooks`，且无 `nul` 文件被误创建；macOS/Linux 复跑同命令行为不变。
4. **三平台不回归 smoke**：macOS + Ubuntu 现有 `cargo build --workspace` + `pnpm tauri:build`（dmg/appimage/deb）产物与改动前一致。

---

## 7. 阶段级风险

| # | 风险 | 关联 PRD §Technical Risks | 缓解 |
|---|---|---|---|
| P5-R1 | MSVC + WiX/NSIS + WebView2 的 CI 环境：windows-latest runner `tauri build` 可能因缺 WiX/WebView2 或配置失败 | R2 | 先 `--no-bundle` 验编译链路，再分步加 nsis / msi；用官方 tauri-action 或显式装 WiX；bundle 失败时先只 gate build+test，bundle 作可选 step |
| P5-R2 | macOS/Linux CI 回归：把 `rust-build` 改成 matrix 时误伤现有 ubuntu leg（apt deps / xvfb / corepack） | R3 | matrix 化保留 ubuntu leg 全部既有 step，Windows leg 用 `if: runner.os == 'Windows'` 条件分支跳过 apt/xvfb；改动前后 ubuntu job step 序列 diff 可审计 |
| P5-R3 | 窗口装饰差异：macOS 专属 `windowEffects`/`trafficLightPosition` 在 Windows 渲染异常或报错 | R4（路径/平台差异类） | 确认 Tauri 在 Windows 安全忽略 macOS-only 窗口配置（survey `already_windows_ok` 已注明）；如有 artifact 则条件化配置；`configure_title_bar` Windows 走 stub，已有 `#[cfg(not(target_os="macos"))]` 空实现兜底 |
| P5-R4 | CI 无法做 GUI runtime 验证：windows-latest headless 跑不了完整 GUI smoke | R5 | CI 限 build + 单元/集成测试 + bundle 产物校验；GUI critical path 靠开发者本机（H:\ Windows 11）按 §2.14 实跑并记录证据 |
| P5-R5 | prepare 脚本跨平台改写引入新失败：node 一行脚本在某些 git 客户端 / CI 环境 `execSync` 抛错阻断 install | R3 | `execSync` 包 try/catch 等价（失败不阻断 install，对齐原 `|| true` 语义）；mac/Linux/Windows 三平台各跑一次 `pnpm install` 验证 |

---

## 8. 阶段级 Definition of Done

- [ ] Task 5.1 / 5.2 / 5.3 全部 `Status: Done`，各自 §10 Completion Notes 回填。
- [ ] §6 全部 P5-AC 勾选；阶段级端到端 smoke 1-4 全部跑过并记录证据（末 task §10）。
- [ ] windows-latest CI leg 在一次手动 `gh workflow run ci.yml` 中 `cargo build --workspace` + `cargo test --workspace` 通过（raw log 链接/截取记入末 task §10）。
- [ ] 本机 Windows 11 产出 `.exe` + `.msi` 并安装启动成功（产物清单 + 安装 smoke 记入 Task 5.1 §10）。
- [ ] macOS/Ubuntu 现有 CI job 与 bundle 零回归（P5-AC6）；所有 Windows 改动走 `#[cfg(target_os)]` 分支或 CI matrix 条件 step。
- [ ] 涉及的 ADR（[ADR-004 Windows 安装包 NSIS 主+MSI 辅](../../decisions/adr-004-windows-installer-nsis-msi.md) · [ADR-005 Windows 测试门控策略](../../decisions/adr-005-windows-test-gating-strategy.md)）已被相关 task §5.1 Required Reading 引用。
- [ ] 本 phase 无遗留 `<TBD>` 占位；Phase 状态从 Ready → Done（末 task done 后翻转）。
