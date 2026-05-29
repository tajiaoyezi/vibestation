# Vibestation Windows 适配 · 产品需求文档（PRD）

> 本 PRD 由 `/s2v-prd` 生成。
>
> ⚠️ **不要手工重命名章节标题**（`/s2v-init` 按"中文｜English"双语锚点解析）。修改章节内容随时可以；改章节名会让 init 漏读字段。
>
> 解析逻辑：`## Vision` 或 `## 愿景` 任一命中即认；但 `## 产品愿景` / `## Product Vision` 会失败（不在双语模板内）。

**生成日期**：2026-05-29
**作者**：leafiellune
**版本**：v1.0

---

## Vision｜愿景

让 Windows 11 上用 Claude CLI / Codex CLI 等 AI agent 的开发者，能在本机原生运行 Vibestation——拿到与 macOS / Ubuntu 对等的"多 Tab 终端 + JetBrains 级 Git 工作台"体验：3-6 个月后，Windows 不再是 `cargo build` 直接编译失败的二等公民，而是 CI 矩阵里持续验证、能产出可安装 `.msi/.exe`、能跑 pwsh/cmd、能看实时 Git status 的一等平台。关键差异是把"AI agent 终端"与"Git 工作台"合一、开源（Apache 2.0）、本地优先、无账号绑定——这正是 Windows 上现有竞品（Windows Terminal 无 Git 工作台、Warp 闭源绑账号）留下的空白。

---

## Problem Statement｜问题陈述

**谁有这个问题**：
把 Windows 11 当主力开发机、希望在本机用多 Tab 终端 + Git 工作台驱动 AI coding agent（Claude CLI / Codex CLI / Cursor 等）的个人开发者与小团队；以及项目自身的 Windows 贡献者与未来的 Windows CI。

**痛点**（能观察到的具体痛苦，非假设）：

- `crates/core/src/pty.rs` 直接 `use mio::unix::SourceFd` + `libc::fcntl/kill/tcgetpgrp` + `std::os::unix::fs::PermissionsExt`，这些是 Unix-only API——**项目在 Windows 上 `cargo build --workspace` 直接编译失败**，连跑都跑不起来。
- 即便绕过编译：默认 shell 写死 `/bin/zsh`/`/bin/bash`（`pty.rs::default_shell_path` + `app_settings.rs:69-72`），`list_available_shells()` 读 `/etc/shells`；`crates/app/src/lib.rs:336/637` 用 `std::env::var("HOME")` 缺失时回落 `/`，导致 workspace 初始化 / config import / PTY pool refill 全部拿到错误路径。
- `fs_watch.rs:51-59` 在 Windows 直接返回 `UnsupportedPlatform`——Git status 永不实时刷新。
- 外部终端检测（`external_term/detect.rs`）`DetectionPlatform` 枚举无 Windows 变体、返回空列表；`launch.rs` 把 Windows 当 Linux 回落，拉起命令全错。
- 前端 `index.tsx` 只发 `platform-macos`/`platform-linux` class（无 `platform-windows`）；TopBar/ActivityStrip/PaneTerminal/CommitBar 等处快捷键提示硬编码 `⌘`/`⇧`/`⌥` 符号，Windows 用户看到 `⌘B` 却要按 `Ctrl+B`（键盘**事件**处理已正确，只是**显示**误导）。
- `tauri.conf.json` bundle targets 只有 `dmg`/`appimage`/`deb`，**无 `msi`/`nsis`**——产不出 Windows 安装包；`.github/workflows/ci.yml` 所有 job `runs-on: ubuntu-latest`，没有 windows runner，Windows 编译错误永远不会在合入前被发现。

**现状**（现有方案 + 为什么不够）：

- **WSL 跑**：能跑，但那是 Linux 体验，失去 Windows 原生窗口/路径/shell 集成，等于"在 Windows 上装个 Linux 再用 Linux 版"。
- **Windows Terminal / VS Code 集成终端**：有多 Tab，但没有 JetBrains 级 Git 工作台、没有 AI session 感知。
- **Warp（已上 Windows）**：闭源、绑账号、非本地优先。
- **项目内部决策表 #8 / ADR-006**：明确把 Windows "推到 v0.4"，至今未启动——前提条件（mac+Linux 主线打磨透）现已满足，但适配工作零进展。

**为什么是现在**：

- v0.3 sprint（MVP-12~17）+ v1.0 vision 代码侧已收口，mac/Linux 主线打磨透了——ADR-006 / implementation-plan §74 当初"先打磨透 mac+linux、不同步踩 ConPTY 和 Wayland/X11 两套坑"的前提已成立。
- 关键依赖本就为 Windows 埋了底：`portable-pty` 统一 ConPTY/Unix PTY 抽象（tech-research.md:74）、`gix` 纯 Rust 利于 Windows（ADR-007）、`dunce` 已在 workspace 依赖里处理 Windows UNC、`rusqlite` bundled 无需系统 sqlite、`main.rs` 已设 `windows_subsystem="windows"`、`icons/icon.ico` 已就位。
- 开发者本机即 Windows 11（工作目录 `H:\`），适配后可本机自测 §2.14 critical UX path。

---

## Users & Context｜用户与场景

**主要用户**：

- **Windows 11 上的 AI-agent 开发者**：用 Claude CLI / Codex CLI 在本机跑 coding agent，需要多 Tab 终端把多个 agent 会话并排、并在同一窗口看 Git Log/Diff/status。接触点 = 创建 Tab、跑 shell 命令、看 Git 工作台。
- **项目 Windows 贡献者**：在 Windows 本机 `cargo build/test` + `pnpm tauri:dev` 验证改动，目前因编译失败而无法贡献。

**次要用户 / 利益相关者**：

- **项目维护者 / CI**：多一个平台 = 新增 windows-latest CI 成本 + 两套 PTY 维护负担；被本适配直接影响（决策表 #8 当初因此推迟）。
- **小团队 IT / 部署方**：需要 `.msi`（适合 GPO 企业部署）或 `.exe`（NSIS，适合个人安装）分发渠道。

**关键使用场景**：

1. **多 agent 并行**：开发者在 Windows 上开 3 个 Tab 分别跑 Claude CLI / Codex / pwsh，切换观察各 agent 输出，同时底部看当前 repo 的 Git status 实时变化。
2. **Git 工作台日常**：在 pwsh Tab 里 `git commit` 后，Git Log 面板立即反映新 commit，Diff 面板看改动；改文件 200ms 内 Git status 徽章刷新（依赖 fs_watch）。
3. **配置迁移**：从已装的 Alacritty / Ghostty（Windows 原生版，配置在 `%APPDATA%`）导入字体/快捷键，keybinding 的 `Ctrl+T` 正确识别（而非被规范化成 `Cmd+T` 后冲突检测失效）。
4. **首次安装**：下载 `.exe`/`.msi` → 安装 → 启动自动检测 pwsh/cmd → 立即可用，无需手配 PATH。

---

## Core Capabilities｜核心能力

> ≤ 5 条。多于 5 条说明范围还没收敛。

1. **Windows 上可编译可运行**：把 `crates/core/src/pty.rs` 的 Unix-only 内核（`mio::unix` reader loop / `libc` 信号 / `fcntl` 非阻塞 / `PermissionsExt`）按 `#[cfg(target_os)]` 分离，Windows 走 `portable-pty` 的 ConPTY 阻塞读路径，使 `cargo build --workspace` + `cargo test --workspace` 在 Windows 通过。例：在 windows-latest CI 上 `cargo build -p vibestation_core` 零错误。
2. **Windows 原生 shell 体验**：默认 shell 探测链 `pwsh.exe → powershell.exe → cmd.exe`，shell 枚举走 PATH/`where`，PTY 能拉起并正常读写。例：新建 Tab 在 pwsh 看到 prompt 并 `git --version` 回显。
3. **跨平台路径与家目录**：统一 `home_dir()` 助手（`USERPROFILE`/`dirs` crate），让 workspace / config-import / pty-pool 在 Windows 正确解析；启用 `fs_watch` 的 Windows backend（notify `ReadDirectoryChangesW`）使 Git status 实时刷新。例：`%USERPROFILE%` 下的 repo 改文件，状态 200ms 内更新。
4. **Windows UI 与外设集成**：前端补 `platform-windows` class + 平台感知的快捷键显示助手（显示 `Ctrl+B` 而非 `⌘B`）；外部终端检测/启动支持 Windows Terminal / conhost / pwsh；config import 支持 `%APPDATA%` 路径 + keybinding `Cmd↔Ctrl` 平台映射。例：右键 Pane 菜单显示 `Ctrl+Shift+O`。
5. **Windows 构建 / 打包 / CI**：`tauri.conf.json` 增 `msi`+`nsis` bundle；CI 加 `windows-latest` 矩阵；`package.json` `prepare` 脚本跨平台化；窗口装饰（hudWindow/trafficLight 等 macOS 专属）平台条件化。例：`pnpm tauri:build --bundles nsis` 产出可安装 `.exe`。

**明确不做（Out of Scope，至少列 3 项）**：

- ARM64 Windows（MVP 仅 x64 MSVC）。
- WSL 专属深度集成 / WSL distro 选择 UI（Windows 原生优先，WSL 用户继续用 Linux 构建）。
- Windows 代码签名 / Authenticode 证书（先 unsigned 分发，对齐 macOS alpha 的 unsigned 策略；签名推后到 Windows GA）。
- Microsoft Store / MSIX 分发。
- `detect_process_cwd` 的 Windows 精确实现（用 spawn-time 缓存的 `initial_cwd` 兜底即可）。
- git hook 脚本除测试层 `cfg` 处理外的深度 Windows 化（`#!/bin/sh` hook 生产逻辑不在本适配范围）。

---

## User Flow｜用户流程

**主流程（happy path）**：

1. Windows 用户下载并安装 `.exe`（NSIS）或 `.msi`，启动 Vibestation。
2. 应用启动时 `fix_path_env` 在 Windows 安全 no-op；`home_dir()` 解析到 `C:\Users\<user>`；shell 探测链选中 `pwsh.exe`（或回落 cmd.exe）。
3. 用户新建 Tab → PTY 经 ConPTY 拉起 pwsh → 看到 prompt → 跑 AI agent / git 命令，输出正常回显。
4. 终态：用户在 Windows 上拿到多 Tab 终端 + 实时 Git 工作台，与 mac/Linux 对等。

**异常流（≥ 2 项）**：

- **未装 PowerShell 7**：探测链回落 `powershell.exe`（5.1 内置）再回落 `cmd.exe`——绝不因找不到 pwsh 而崩溃或拉起不存在的 `/bin/bash`。
- **`HOME` 环境变量未设**：`home_dir()` 用 `USERPROFILE` 兜底，config import 不会去扫 `/Library/Preferences/...` 等虚假 Unix 路径，workspace 初始化拿到正确根目录。
- **路径含空格 / 中文 / 反斜杠 / UNC**：`dunce::canonicalize` 规范化，DB round-trip 一致，git 仓库检测沿 `Path::parent()` 正常上溯。

**边界场景（≥ 1 项）**：

- **UNC / 网络盘 workspace**（`\\?\C:\...` 或 `\\server\share\repo`）：`dunce` 去 verbatim 前缀用于显示，git 检测仍工作，存库为规范化字符串。

---

## Technical Approach｜技术方案

- **项目类型**：Desktop（Tauri 2 桌面应用 · macOS+Linux 已支持，新增 Windows 11）
- **技术栈**：Rust（`crates/core` 业务 + `crates/app` Tauri 启动层）+ SolidJS/TypeScript（`web`）；关键库 `portable-pty`（ConPTY/Unix PTY 统一）/ `git2`+`gix` / `rusqlite`(bundled) / `notify`(fs watch) / `dunce`(Windows 路径)；拟新增 `dirs`（跨平台家目录，纯 Rust 无 C 依赖）。硬约束：优先 `#[cfg(target_os)]` 分支复用现有抽象，不引入与现有锁定栈冲突的重依赖。
- **关键模块边界**（≥ 3 个）：
  - `crates/core/src/pty.rs`：PTY 会话 / 读线程 / 信号 / shell 检测——核心 cfg 分离点（Windows ConPTY vs Unix mio）。
  - `crates/core/src/external_term/{detect,launch,env_filter}.rs`：外部终端检测/启动/环境过滤——加 Windows 平台变体与启动配方。
  - `crates/core/src/config_import/{iterm2,ghostty,alacritty,keybinding,ipc}.rs`：终端配置导入 + keybinding 规范化——`%APPDATA%` 路径 + Cmd/Ctrl 平台映射。
  - `crates/core/src/{app_settings,fs_watch,workspace}.rs`：shell 默认 / 文件监视 / 路径规范化。
  - `crates/app/src/lib.rs`：Tauri 启动层 `home_dir()` 助手 / shell 解析 / 窗口装饰条件化。
  - `crates/app/tauri.conf.json` + `.github/workflows/ci.yml` + `package.json`：构建 / 打包 / CI。
  - `web/src/{index.tsx,lib,components,panels}`：前端 `platform-windows` class + 平台感知快捷键显示。
- **架构风格**：模块化单体（Tauri 桌面应用 · 前后端经 IPC）。Windows 适配以"运行期平台判断 + 编译期 `#[cfg(target_os)]` 分支"叠加在现有抽象上，不重写架构。
- **数据流（如适用）**：用户输入（键盘/IPC 命令）→ Tauri command handler（`crates/app`）→ 业务逻辑（`crates/core`：PTY/git/config）→ SQLite 持久化（`app_local_data_dir`，Windows = `%APPDATA%`）+ 事件回推前端。平台分支集中在 PTY 拉起 / shell 检测 / 路径解析 / 文件监视四处。

---

## Constraints｜约束

- **运行时**：Rust stable（≥ 1.95，对齐项目 toolchain）+ Node 20+ / pnpm 9.15 + Tauri CLI 2.x；Windows 侧需 MSVC toolchain（`x86_64-pc-windows-msvc`）+ WebView2 Runtime（evergreen，系统自带或随包）。
- **平台**：新增 `Windows 11 x64 (MSVC)`；保持 `macOS arm64/x64` + `Ubuntu 24 LTS (X11/Wayland)` 零回归。
- **性能**：Windows 冷启动 < 500ms（mac 基线 202ms / Ubuntu ~108ms，ADR-006）；PTY visible throughput 不显著低于 Unix；Git status watch debounce 维持 200ms（`GIT_STATUS_WATCH_DEBOUNCE`）。
- **安全**：无新增敏感数据；Tauri ACL/CSP 不放松（沿用 `default.json` capability + 现有 CSP）；shell 拉起仍过 `sanitize::is_denied_windows_path`（已存在 `#[cfg(windows)]`）；禁硬编码凭证/路径。
- **兼容性**：所有 Windows 改动走 `#[cfg(target_os)]` 分支或运行期平台判断，不破坏 mac/Linux 行为；DB schema 不变（`app_local_data_dir` 已跨平台）；config import 向后兼容已有 mac/Linux 路径（Windows 路径为新增分支，非替换）。
- **发布**：新增 Windows `.msi`（WiX）+ `.exe`（NSIS）bundle，GitHub Release 附 Windows 产物；CI `windows-latest` 矩阵跑 build+test；保持 mac `.dmg` + Linux `.deb/.AppImage`。先 unsigned（出问题在 Release notes 写 SmartScreen bypass 指引，签名推 Windows GA）。

---

## Implementation Phases｜实施阶段

> `/s2v-init` 会读这张表批量生成 phase spec 和 task spec。
> - `description` 列写"完成后能做什么"，不写 TODO 风格
> - `scope` 列要列出**具体模块名 / 文件名**
> - `depends_on` 用 phase 编号；零依赖写 `-`
> - `parallel` 标"是 / 否"；写"是"时必须说明"可与谁并行"

| # | Phase 名称（kebab）| 描述（完成后能做什么）| 范围（涉及模块 / 文件）| 依赖 | 可并行 |
|---|---|---|---|---|---|
| 1 | foundation-build | Windows 上 `cargo build --workspace` 编译通过；跨平台家目录/shell-默认基元就位 | crates/core/src/pty.rs（cfg 分离 mio/libc/signal/fcntl/PermissionsExt + ConPTY reader 路径）+ crates/core/src/app_settings.rs（default_shell Windows 分支）+ crates/app/src/lib.rs（home_dir() 助手 + dirs 依赖）| - | 否（基础设施 · 解锁全部后续）|
| 2 | shell-runtime | Windows 上能探测/枚举 shell 并经 ConPTY 拉起 pwsh/cmd 正常读写 | crates/core/src/pty.rs（default_shell_path/list_available_shells/resolve_shell Windows 探测链 + detect_process_cwd 缓存兜底）+ 相关单元测试 cfg-gate | 1 | 否 |
| 3 | terminal-integration | Windows 外部终端检测/启动可用；config import 走 %APPDATA%；keybinding Ctrl 正确；Git status 实时刷新 | crates/core/src/external_term/{detect,launch,env_filter}.rs（Windows 平台变体+启动配方）+ config_import/{ghostty,alacritty,iterm2,keybinding,ipc}.rs + fs_watch.rs（启用 notify Windows backend）| 1 | 是（可与 Phase 4 并行 · 纯后端，与前端文件域不交叠）|
| 4 | frontend-platform | 前端发 platform-windows class；所有快捷键提示按平台显示 Ctrl/⌘ | web/src/index.tsx（platform 检测）+ components/{TopBar,ActivityStrip} + panels/{Terminal,CommitBar} + styles.css（平台感知快捷键显示助手）| - | 是（可与 Phase 3 并行 · 纯前端 TS，独立于 Rust）|
| 5 | build-package-ci | 产出可安装 Windows `.exe/.msi`；windows-latest CI 矩阵跑 build+test；prepare 脚本跨平台；窗口装饰条件化 | crates/app/tauri.conf.json（msi/nsis targets + 窗口装饰条件化）+ .github/workflows/ci.yml（windows-latest 矩阵）+ package.json（prepare 跨平台）+ crates/app/src/lib.rs（configure_title_bar Windows 分支）| 1, 2 | 否 |
| 6 | integration-matrix | Windows 端到端 smoke 通过；三平台测试矩阵零回归；CI 全绿 | 全量 cargo test（Windows-gated 测试集：shell_compat/git_ops_integration/pty_*_integration/env_filter cfg-gate）+ vitest + 三平台 CI 验证 | 2, 3, 4, 5 | 否 |

---

## Decisions Log｜决策日志

> `/s2v-init` 阶段 9.1 会把每条决策转成一份 ADR（默认 Status=Accepted）。
> 至少 3 条；至少覆盖 S2V 8 类决策中的任 3 类。
> **`类别`列取值约束**：从 8 个字面值中选其一——`架构` / `依赖` / `数据持久化` / `协议接口` / `安全` / `测试工具链` / `部署发布` / `兼容性`（逐字照抄）。

| ID (D1, D2...) | 类别 | 决策（一句话）| 选择 | 候选方案 | 拒绝候选的理由 |
|---|---|---|---|---|---|
| D1 | 架构 | PTY 的 Windows 适配怎么落地 | 在 `pty.rs` 内用 `#[cfg(target_os)]` 分离 Unix（mio+libc）与 Windows（portable-pty ConPTY 阻塞读 + `child.try_wait` 退出检测）两条 reader 路径，复用同一 `PtySession` 抽象 + 局部 helper | (a) 整模块 Windows 全新实现 / (b) 抽象 `PtyBackend` trait 双实现 / (c) cfg 分支 + 局部 helper | (a) 重复维护两份会话逻辑，易漂移；(b) trait 化是过度设计（YAGNI · 唯一 Unix-bound 处是 reader loop），portable-pty 本已统一 ConPTY/Unix PTY 抽象（tech-research:74）；(c) 最小改动 + 零 Unix 回归 |
| D2 | 依赖 | 跨平台家目录怎么解析 | 新增 `dirs` crate 做 `home_dir()`；应用数据目录继续用 Tauri `app_local_data_dir()`（已正确解析 %APPDATA%）| (a) 手写 `cfg` 读 USERPROFILE/HOME / (b) `dirs` crate / (c) `directories` crate (ProjectDirs) | (a) 易漏 `HOMEDRIVE`+`HOMEPATH` 组合等边界；(c) ProjectDirs 偏重，而 app data dir 已由 Tauri path API 覆盖；(b) `dirs` 纯 Rust、无 C 依赖、事实标准、最小 |
| D3 | 兼容性 | Windows 默认 shell 选哪个 | 探测链 `pwsh.exe → powershell.exe → cmd.exe`，取第一个可用 | (a) 固定 cmd.exe / (b) 固定 powershell.exe / (c) pwsh→powershell→cmd 探测 | (a) cmd.exe 体验过于基础；(b) 纯 5.1 powershell 不如 pwsh 现代；(c) "有啥用啥"且 cmd.exe 永远保底，不会拉起不存在的 shell |
| D4 | 部署发布 | Windows 安装包用什么格式 | bundle 同产 `nsis`（.exe，主）+ `msi`（WiX，辅），先 unsigned | (a) 仅 msi / (b) 仅 nsis / (c) 两者 | (a) MSI 体积大、对未签名场景不友好；(b) 仅 NSIS 缺企业 GPO 部署能力；(c) NSIS 小巧友好作主、MSI 供企业部署，Tauri 都原生支持，CI 同跑 |
| D5 | 测试工具链 | Windows 测试怎么门控 | Unix-only 行为测试用 `#[cfg(unix)]` 或 `#[cfg_attr(windows, ignore="...")]`，并补 Windows 专属测试；CI windows-latest 跑 `cargo test --workspace` 自动跳过 | (a) 全部测试强制双平台 / (b) cfg 分离 + Windows ignore + 补 Windows 专测 / (c) Windows 只 build 不 test | (a) 不现实（/etc/shells、chmod 0o755、#!/bin/sh hook 在 Windows 无意义）；(c) 放过 Windows 行为回归；(b) 既保留 Unix 真测又给 Windows 真覆盖 |
| D6 | 协议接口 | fs_watch 在 Windows 启用还是禁用 | 移除 `fs_watch.rs` 的 Windows `UnsupportedPlatform` 短路，启用 `notify` 的 Windows backend（ReadDirectoryChangesW），保持 200ms debounce + `.git/index.lock` 排除契约不变 | (a) 保持 Windows 禁用 / (b) 启用 notify Windows backend / (c) 自写 ReadDirectoryChangesW 包装 | (a) Git status 不实时 = Windows 体验降级；(c) 重复造轮子（notify 已跨平台）；(b) 复用成熟 backend，契约不变 |

---

## Success Metrics｜成功指标

**主要指标**（Primary，≥ 1 个，必须可测量）：

- **三平台 CI 全绿**：windows-latest 上 `cargo build --workspace` + `cargo test --workspace`（Windows-gated 测试集）+ `pnpm tauri:build --bundles nsis` 全部成功；macOS/Ubuntu 现有 job 零回归。冷启动 < 500ms（Windows 实测）。

**次要指标**（Secondary，≥ 2 个）：

- **PTY runtime smoke**：Windows 上新建 Tab，pwsh/cmd 显示 prompt，跑 `git --version` / `echo` 命令有正确回显（§2.14 本机实跑）。
- **Git status 实时刷新**：Windows workspace 内改文件，Git status 徽章 200ms 内更新（fs_watch Windows backend 生效）。
- **外部终端列表非空**：装了 Windows Terminal / Alacritty 时 `external_term_list()` 返回非空，且 `launch` 能拉起。

**反指标**（Anti-metrics，≥ 1 项）：

- **不能为 Windows 适配牺牲 mac/Linux 现有行为**：macOS + Ubuntu 全量 `cargo test` / vitest 必须仍 100% 绿（所有 Windows 改动走 cfg 分支，Unix 路径行为零变化）。
- **不能为"编译过"而阉割 Unix PTY 功能**：Unix 的信号/前台进程组/cwd 检测/非阻塞 fd 行为零回归。

---

## Open Questions｜开放问题

> ≥ 1 项。

- [ ] OQ1：Windows 代码签名 / Authenticode 何时做（影响 SmartScreen 警告）——MVP 先 unsigned（对齐 macOS alpha），推 Windows GA；需 Arbiter 在 GA 前决定证书来源。
- [ ] OQ2：是否需要单实例（single-instance）强制（NSIS / 命名管道）——MVP 不做，待用户反馈决定。
- [ ] OQ3：`detect_process_cwd` 的 Windows 精确实现是否必要——MVP 用 spawn-time 缓存 cwd 兜底，按反馈决定是否升级到 Windows API 查询。
- [ ] OQ4：WebView2 Runtime 随包分发（fixed version，体积大）还是依赖系统 evergreen——默认 evergreen，待最低支持的 Windows 版本确认。

---

## Technical Risks｜技术风险

> ≥ 3 项。

| # | 风险 | 概率 | 影响 | 缓解策略 |
|---|---|---|---|---|
| R1 | ConPTY 行为差异：portable-pty 的 ConPTY reader 退出检测 / 信号语义与 Unix PTY 不同，可能 hang 或漏 exit、读不到尾部输出 | 高 | 高 | Windows reader 用 `child.try_wait()` + 阻塞读循环；先写 ConPTY spawn/exit/echo 集成测试再实现；在本机 Windows 11 实跑 smoke 验证退出与回显；保留 Unix mio 路径不动 |
| R2 | MSVC + WiX/NSIS + WebView2 的 CI 环境：windows-latest runner 需 MSVC toolchain + WebView2 + WiX，`tauri build` 可能因缺依赖或 WiX 配置失败 | 中 | 高 | 先用 `--no-bundle` 验证编译链路，再分步加 nsis、msi；用官方 tauri-action 或显式装 WiX；bundle 失败时先只 gate build+test，bundle 作为单独可选 job |
| R3 | mac/Linux 回归：cfg 分支写错把 Unix 路径也改坏 | 中 | 中 | 所有 Windows 改动严格走 `#[cfg(target_os)]` 分支；mac/Linux 全量 `cargo test` 必须本地+CI 仍绿（反指标硬约束）；reviewer 三平台意识；TDD 先 RED 锁住 Unix 行为不变 |
| R4 | 路径分隔符 / UNC / 编码：Windows 反斜杠、UNC、中文路径在 DB round-trip / git 检测 / 前端显示出错 | 中 | 中 | 统一 `dunce::canonicalize`；前端 `prettyPath` 的 Unix-only regex 已对 Windows 路径安全（只匹配 `/` 前缀，不误伤 `C:\`）；补 Windows 路径 round-trip 单元测试 |
| R5 | CI 无法做 GUI runtime 验证：windows-latest headless 跑不了完整 GUI smoke | 低 | 中 | CI 限 build + 单元/集成测试 + bundle 产物校验；GUI critical UX path 靠开发者本机（H:\ Windows 11）按 §2.14 实跑并记录证据 |

---

## Next Steps｜后续步骤

1. **审本 PRD 内容**（重点：阶段表 / 决策日志 / 风险）——无人值守模式下由主 agent 以 Arbiter 身份完成对抗性自审。
2. **后续路径**：跑 `/s2v-init`（tier=solo · AUTO 拆分）一次性产出全套 S2V 文档（adapter / AGENTS.md / phase spec / task spec / ADR / BDD feature），再逐 task `/s2v-implement`。

> ⚠️ task spec 实施完后留在原地不归档（SDD 单一事实源核心要求）。
