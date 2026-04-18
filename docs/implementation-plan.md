# Vibestation 完整实施计划

> 版本：v2.0（2026-04-18 · 整合 Codex 评审与用户决策）
> 前序：v1.0（2026-04-17 规划定稿）
> 作者：项目规划师
> 技术栈：**默认 Tauri 2 + Rust + SolidJS + xterm.js + git2 + portable-pty**（Spike Week 0 Day 2 硬通过后锁定，失败回退 Electron 28+）
> 许可证：**Apache 2.0**（含 NOTICE、patent grant；不签 CLA）
> 调研依据：`/Users/leaf/CodeWorkSpace/PersonalWorkspace/docs/terminal-git-workbench-tech-research.md`
> 评审依据：`/Users/leaf/CodeWorkSpace/PersonalWorkspace/docs/vibestation-codex-review-and-response.md`

---

## v2 变更概要

相比 v1，本版做了结构性升级而非小修补。核心修改如下：

| 维度 | v1 | v2 |
|------|----|----|
| 许可证 | MIT | **Apache 2.0**（无 CLA）|
| Tauri 决策 | "已敲定" | **默认 + Spike Day 2 硬通过 + Electron 28+ 保底** |
| Cargo workspace | 4 crate | **2 crate（app + core）**，v0.2 再拆 |
| git2 + gix 混用 | 同时上 | **默认 git2，gix 在 Spike Day 3 benchmark 后评估读路径再引入** |
| redb 持久化 | "已敲定" | **默认 redb，Spike 后 benchmark 对比 rusqlite 再锁** |
| AI-Aware Pane 叙事 | 核心卖点 | **v1.0 vision；README / landing 完全移除，MVP 不宣传** |
| MVP 工期 | 10 周 | **12 周**（20% buffer） |
| v1.0 总工期 | 24-25 周 | **28-30 周** |
| 风险登记 | 20 条 | **30 条**（新增 R21-R30）|
| MVP 功能范围 | 全套 | **B 折中**：砍 Push/Pull/Fetch + 自绘 rail graph + 复杂 Diff |
| 社区 KPI | stars 数字 | **信号指标**（HN 首页 / r/rust 100 upvotes / 博客提及）|
| 章节数量 | 12 | **14**（新增 §13 安全边界、§14 分发运营）|

---

## 1. 产品定位一页纸

### 1.1 一句话卖点

**"Claude CLI / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台"**

> v2 调整：删除 "Mission Control" / "AI session 感知" 对外叙事。产品对外只承诺"多 Tab 终端 + Git 工作台"。AI session 感知能力作为 v1.0 vision 内部保留，README / landing 页面不宣传，直到 v1.0 真实落地再作为升级故事推出。

### 1.2 目标用户画像

**Persona A：独立开发者 Alice（35 岁，全栈）**
- 日常同时跑 3-5 个副业项目，每个项目开 1-2 个 Claude CLI 窗口做 pair coding
- 痛点：iTerm2 的 Tab 切来切去，忘了哪个窗口在改哪个仓库；Claude 改完代码要手动切 IDE 看 diff
- 使用场景：早上打开 Vibestation，工作区里列着 5 个项目，每个项目的当前分支、未提交文件数一目了然。点进某个项目看到 Git Status 面板列着已修改文件，审阅 Diff 后勾选文件一键 commit。

**Persona B：团队技术负责人 Bob（42 岁，架构师）**
- 维护 monorepo（单仓 50 万 commit），同时带 3 个团队成员
- 痛点：JetBrains 的 Git 面板好用但启动慢 30 秒，终端不在同一个窗口；code review 时在 Claude CLI 跑完修改又要切 GitKraken 看提交图
- 使用场景：在 Vibestation 里打开 monorepo，Git Log 列表瞬时加载（MVP 无 rail 自绘，分支信息用标签贴呈现），右侧多 Tab 终端同时跑 `cargo test` 和 Claude CLI 重构助手。

**Persona C：重度 vibe coder Carol（28 岁，前端）**
- 全程 CLI 派，不用 IDE，偏好 Ghostty + tmux，要求键盘驱动
- 痛点：桌面 Git 客户端都太 GUI 化（SourceTree、Fork），命令行版 gitui 又没有多 Tab 终端集成
- 使用场景：Vibestation 导入 Ghostty 配置（字体/主题/按键），键盘快捷键覆盖 95% 操作，多 Tab 里 Claude + Codex + zsh + vim 并行工作。

### 1.3 核心价值主张

1. **多项目 × 多终端的统一工作台**：一个窗口承载 N 个项目 Tab，每个项目独立的终端 session 和 Git 状态，切换成本归零。
2. **JetBrains 级 Git 视图 + CLI 级响应速度**：读路径优先用 git2；若 Spike Day 3 benchmark 证明 gix 在大仓库上显著更快，再在读侧引入 gix。MVP 用 Log 列表 + 分支标签贴（无自绘 rail），10 万 commit 仓库首屏 <500ms。Diff 自绘避开 Monaco 3MB 包体积。
3. **（v1.0 vision，不对外宣传）AI session 感知的版本控制**：把 Claude / Codex CLI 的一次对话识别为一个 session，自动聚合 AI 改动、一键 diff 审阅、一键回滚。此能力作为 v1.0 的升级故事，MVP 和 README 均不提及。

### 1.4 Non-goals（明确不做）

1. **不做通用 IDE**：不内置语法高亮/代码补全/LSP 集成。
2. **不 Fork Ghostty 源码**：Ghostty 是 Zig，技术栈冲突，只做配置兼容。
3. **不做云同步**：workspace/配置纯本地 redb（或 Spike 后替换为 rusqlite）。
4. **不做协作/团队功能**：不做 PR review、不做评论、不做 issue 追踪。
5. **不支持 Windows（v1.0 前）**：macOS + Ubuntu 24 先打磨透；ConPTY 和 Wayland/X11 两套坑不同步踩。
6. **不做 Git Flow / GitHub Flow 工作流教条**。
7. **不做插件市场（v1.0 前）**。
8. **（v2 新增）不支持远程 / SSH / devcontainer 场景**（v1.x 考虑）。
9. **（v2 新增）不支持企业代理 / 离线安装环境**（需单独做 air-gapped 打包，v1.x 评估）。
10. **（v2 新增）不保证超大仓库（1M+ commit 或 >10GB）最佳体验**，只承诺不崩溃。
11. **（v2 新增）Git worktree / submodule / LFS / partial clone 支持是 v0.3-v1.0 渐进范围**，MVP 遇到只保证不崩、不提供专门 UI。

---

## 2. 竞品对比矩阵

| 能力 | Warp | Ghostty | iTerm2 | CodexMonitor | Claude Desktop | Codex Desktop | JetBrains | Zed | Cursor | **Vibestation MVP** |
|------|------|---------|--------|--------------|----------------|---------------|-----------|-----|--------|---------------------|
| 多项目 Tab | 块级 Tab | 标签 | 标签 | Workspace | 无 | 无 | Project 窗口 | Worktree | Project 窗口 | ✅ 工作区 + 多 Tab |
| 多 CLI 支持 | ✅ | ✅ | ✅ | 仅 Codex | ❌ | 仅 Codex | 嵌入式终端 | 嵌入式终端 | 嵌入式终端 | ✅ zsh/bash + Claude/Codex |
| Git Log 视图 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ 强 | ✅ 中 | ✅ 中 | ✅ 列表 + 分支标签（MVP 无 rail 图） |
| Git Commit 操作 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ 强 | ✅ 中 | ✅ 弱 | ✅ 简化版（勾文件 + 消息 + amend）|
| Push/Pull/Fetch | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ 强 | ✅ 中 | ✅ 中 | ❌ v0.2 |
| Diff 渲染 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ 强 | ✅ 中 | ✅ 中 | ✅ 基础行对比（v0.3 升级）|
| 终端配置导入 | 自有 | 自有 | 自有 | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ Ghostty/iTerm2/Alacritty |
| Pane 分屏 | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ 最多 1 层嵌套（4 Pane）|
| 跨平台 | mac+linux+win | mac+linux | mac | 全平台 | 全平台 | mac+linux | 全平台 | 全平台 | 全平台 | mac+ubuntu（v1.0 前）|
| 包体积 | ~200MB | ~50MB | ~30MB | ~40MB | ~150MB | ~120MB | ~1GB+ | ~150MB | ~300MB+ | <30MB（目标，Tauri）/ <80MB（Electron fallback）|
| 开源 | ❌ 商业 | ✅ MIT | 免费闭源 | ✅ MIT | ❌ 商业 | ✅ Apache | ❌ 商业 | ✅ GPL | ❌ 商业 | ✅ **Apache 2.0** |

**象限定位**：横轴「Git 深度」、纵轴「终端整合度」。

- Vibestation MVP 的差异化锚点：**"多 Tab 终端 + 只读 Git 工作台 + 配置导入"三点闭环**，不依赖 AI session 感知叙事即可成立。
- 与 CodexMonitor 的关键差异：多 Tab 通用终端（不限 Codex）+ Git Log/Status/Diff/Commit UI。CodexMonitor 没有 Git 工作台。
- 长期卖点（v1.0）：**JetBrains 级 Git 工作台 + 多 Tab 终端 + AI session 感知**，但 MVP / v0.2 不对外用 AI session 叙事。

---

## 3. 技术架构

### 3.1 技术栈决策

| 层 | 决策 | 锁定状态 | 备选 / fallback |
|----|------|----------|------------------|
| 桌面框架 | **Tauri 2** | **默认选用**，Spike Week 0 Day 1-2 硬通过后锁定 | Electron 28+（Spike 失败切回）|
| 前端 | SolidJS + TypeScript + Vite | 锁定 | - |
| 终端渲染 | xterm.js 5.5 | 锁定 | alacritty_terminal（v1.x 评估）|
| PTY | portable-pty | 锁定 | - |
| Git 写 | git2 (vendored libgit2) | 锁定 | - |
| Git 读 | **默认 git2**；Spike Day 3 benchmark 后若 gix 有显著增益再引入 gix | 未锁定 | 纯 git2 保底 |
| 持久化 | **默认 redb**；Spike 后用真实数据量 benchmark 对比 rusqlite 再锁 | 未锁定 | rusqlite 保底 |
| CSS | 原生 CSS + oklch token | 锁定 | - |
| 构建 | Cargo workspace + pnpm | 锁定 | - |
| 许可证 | **Apache 2.0** | 锁定 | - |

#### 3.1.1 Tauri Spike 硬通过判据（Week 0 Day 1-2）

必须在 Ubuntu 24 Wayland + X11 + macOS 15 三台机器上全部通过，否则 Day 3 切 Electron 28+：

| 判据 | 通过条件 |
|------|----------|
| 冷启动窗口显示 | < 2s（mac）/ < 3s（linux）|
| 窗口不白屏、不闪退 | 连续启动 10 次零失败 |
| IME 输入（中文/日文）| 不丢字、光标位置正确 |
| 剪贴板 copy/paste | Wayland + X11 均工作 |
| WebView bundle 大小 | dmg < 30MB / AppImage < 40MB |
| 关键 Tauri plugin 可用 | `tauri-plugin-clipboard-manager`、`tauri-plugin-fs`、`tauri-plugin-updater`（Day 2 至少 smoke test）|

任何一项失败 → 启动 Day 3 Electron 28+ spike（1 天），通过则切 Electron，文档 §3-4 全部回退。

### 3.1.2 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                  Frontend (SolidJS + TypeScript)                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────────┐ │
│  │ TerminalView │ │  GitLogView  │ │  DiffView / CommitView   │ │
│  │ (xterm.js)   │ │ (virtualized)│ │    (自绘 HTML / Canvas)  │ │
│  └──────────────┘ └──────────────┘ └──────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────────┐│
│  │        Store (Context + createStore per workspace)           ││
│  └──────────────────────────────────────────────────────────────┘│
└─────────────────────────────┬───────────────────────────────────┘
                              │ Tauri IPC (invoke + emit)
┌─────────────────────────────▼───────────────────────────────────┐
│                      Tauri Host (app crate)                     │
│  - #[tauri::command] 入口路由                                    │
│  - Emitter 事件广播                                              │
│  - AppState { workspaces, terminal_sessions, git_repos }        │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                   ┌──────────▼──────────┐
                   │   core crate        │
                   │                     │
                   │  git:    git2 / gix │
                   │  pty:    portable-pty│
                   │  store:  redb       │
                   │  watch:  notify     │
                   │  config: toml       │
                   └─────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                          System Layer                           │
│   libgit2 (vendored) │ OS PTY (Unix) / ConPTY │ notify (FS)     │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Cargo Workspace（v2 收紧为 2 crate）

v1 的 4-crate（app / core / git / pty-proxy）对单人项目过度工程。v2 合并为 **2 crate**：

| Crate | 职责 | 关键依赖 | 禁止 |
|-------|------|----------|------|
| `vibestation-app` | Tauri 入口、IPC 路由、事件广播、AppState 组装 | `tauri`、`tokio`、`tracing` | - |
| `vibestation-core` | 业务模型 + Git + PTY + 持久化 + 事件总线一体 | `serde`、`redb`（待定）、`git2`、`gix`（待定）、`portable-pty`、`notify` | `tauri`（纯逻辑层）|

**拆分触发条件**：当以下任一情况发生时，把 `core` 拆为 `core / git / pty-proxy`：

- `core` 单 crate 行数超过 8000 行
- `cargo test` 全跑时间超过 60s
- CI 上 incremental rebuild 开始超过 2 分钟
- 引入独立进程隔离（目前不在 v1.0 计划里）

预计触发时点：v0.2 W15-W18 区间。

核心原则：**`app` 依赖 `core`，`core` 不感知 Tauri**。单测不需要启动 Tauri。

### 3.3 关键数据流（AI 改代码 → UI 刷新）

以 Persona A 场景为例："用户在 Tab 1 的 Claude CLI 里让 AI 改了 src/main.rs"：

```
┌──────────────┐                                      ┌──────────────────┐
│ User 输入指令 │──①──▶ xterm.js (Tab 1 前端)          │ FS Watcher (notify)│
│  "改 main.rs"│       │                              │                  │
└──────────────┘       ▼                              └─────────┬────────┘
                ② TerminalProxy.write (IPC)                    │ ③ 文件写入
                       │                                        ▼
                       ▼                              ┌──────────────────┐
              ┌──────────────────┐                    │ src/main.rs 变化 │
              │ core::pty       │                    └─────────┬────────┘
              │  Claude CLI 进程 │                              │ ④ debounce 300ms
              └────────┬─────────┘                              ▼
                       │ ⑤ stdout 输出                 ┌──────────────────┐
                       ▼                               │ core: FsEvent    │
          ⑥ emit("terminal:output", tab_id, chunk)    │  → dirty_repo(id)│
                       │                               └─────────┬────────┘
                       ▼                                         │ ⑦
              ┌──────────────────┐                     ┌─────────▼────────┐
              │ xterm.js write   │                     │ core::git:      │
              │ (Tab 1 显示改动) │                     │ status/diff 刷新 │
              └──────────────────┘                     └─────────┬────────┘
                                                                 │ ⑧
                                                      emit("git:status-changed")
                                                                 │
                                           ┌─────────────────────▼───────────┐
                                           │ 前端 workspace store 更新       │
                                           │ GitStatusPanel 响应式重渲染     │
                                           │ （未提交文件数 +1，main.rs 红） │
                                           └─────────────────────────────────┘
```

关键节拍：
- ①-⑥ 是终端路径（毫秒级，xterm 直接 write）
- ③-⑧ 是 Git 路径（debounce 300ms，避免 AI 连续改 10 个文件刷屏）
- 两条路径独立不阻塞，即使 git status 慢也不影响终端流畅

---

## 4. 代码仓库目录结构

```
vibestation/
├── Cargo.toml                       # workspace 根（2 crate）
├── Cargo.lock
├── package.json                     # pnpm workspace 根
├── pnpm-workspace.yaml
├── .github/                         # CI/模板
│   ├── workflows/
│   │   ├── ci.yml                   # lint/test/build 矩阵 (mac+ubuntu)
│   │   ├── release.yml              # release-please + artifact 上传
│   │   ├── notarize.yml             # macOS 公证专职流水线（v2 新增）
│   │   ├── appimage.yml             # Linux AppImage 专职流水线（v2 新增）
│   │   └── cache-warm.yml           # 缓存 target/ 预热
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
├── LICENSE                          # Apache-2.0
├── NOTICE                           # Apache-2.0 NOTICE（v2 新增）
├── README.md                        # 英文主
├── README.zh-CN.md                  # 中文副
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── CHANGELOG.md                     # release-please 自动维护
├── SECURITY.md                      # 安全报告邮箱 + 披露流程（v2 新增）
├── crates/
│   ├── vibestation-app/             # Tauri 入口 crate
│   │   ├── Cargo.toml
│   │   ├── build.rs                 # Tauri build hook
│   │   ├── tauri.conf.json
│   │   ├── Entitlements.plist       # macOS Hardened Runtime + PATH 继承
│   │   ├── icons/                   # 多尺寸 icon
│   │   └── src/
│   │       ├── main.rs              # tauri::Builder 入口
│   │       ├── commands/            # #[tauri::command] 路由
│   │       │   ├── mod.rs
│   │       │   ├── terminal.rs
│   │       │   ├── git.rs
│   │       │   ├── workspace.rs
│   │       │   ├── pane.rs          # Pane 系统命令（v2 新增）
│   │       │   └── config.rs
│   │       ├── events.rs            # Emitter 事件定义
│   │       └── state.rs             # AppState 组装
│   └── vibestation-core/            # 一体化业务 + Git + PTY
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── workspace.rs         # Workspace 结构
│           ├── tab.rs               # Tab / Pane / LayoutNode
│           ├── session.rs           # TerminalSession/UserSession
│           ├── config.rs            # AppConfig + TOML 读写
│           ├── profile.rs           # TerminalProfile + 导入
│           ├── persistence.rs       # redb 封装（未锁定）
│           ├── events.rs            # 内部事件总线
│           ├── watcher.rs           # notify + debouncer-mini 封装
│           ├── paths.rs             # directories::ProjectDirs 封装
│           ├── git/                 # Git 子模块
│           │   ├── mod.rs
│           │   ├── repo.rs          # git2::Repository（+ 可选 gix）
│           │   ├── sync/            # 同步 git 操作
│           │   │   ├── mod.rs
│           │   │   ├── logwalker.rs # TimeOrderedCommit + BinaryHeap
│           │   │   ├── commit.rs
│           │   │   ├── branches.rs
│           │   │   ├── diff.rs
│           │   │   └── status.rs
│           │   ├── revlog.rs        # AsyncLog 分批 + 双速率
│           │   ├── async_ops.rs     # 异步操作 + AtomicBool 取消
│           │   └── types.rs         # CommitNode / BranchInfo / FileDiff
│           ├── pty/                 # PTY 子模块
│           │   ├── mod.rs
│           │   ├── session.rs       # TerminalSession
│           │   ├── reader.rs        # 单读线程 + mpsc 分发
│           │   ├── shell.rs         # zsh/bash 启动参数
│           │   └── env_fix.rs       # fix-path-env wrapper
│           └── security/            # v2 新增：TaskRunner 白名单 / 命令校验
│               ├── mod.rs
│               ├── allowlist.rs
│               └── sanitize.rs
├── frontend/
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── terminal/
│   │   │   │   ├── Terminal.tsx
│   │   │   │   ├── TerminalTabs.tsx
│   │   │   │   └── terminal.css
│   │   │   ├── git/
│   │   │   │   ├── GitLogView.tsx      # 虚拟化列表
│   │   │   │   ├── CommitList.tsx      # v2：MVP 用列表 + 标签贴，无 rail 自绘
│   │   │   │   ├── CommitDetail.tsx
│   │   │   │   ├── DiffView.tsx        # v2：MVP 基础行对比
│   │   │   │   ├── BranchList.tsx
│   │   │   │   └── StatusPanel.tsx
│   │   │   ├── pane/
│   │   │   │   ├── PaneContainer.tsx
│   │   │   │   ├── SplitLayout.tsx
│   │   │   │   └── SmartLayoutPicker.tsx
│   │   │   ├── workspace/
│   │   │   │   ├── WorkspaceSwitcher.tsx
│   │   │   │   └── ProjectList.tsx
│   │   │   └── ui/
│   │   │       ├── Button.tsx
│   │   │       ├── Splitter.tsx
│   │   │       └── ContextMenu.tsx
│   │   ├── stores/
│   │   │   ├── workspace.ts
│   │   │   ├── git.ts
│   │   │   └── terminal.ts
│   │   ├── lib/
│   │   │   ├── ipc.ts
│   │   │   ├── events.ts
│   │   │   └── diff-parser.ts
│   │   ├── hooks/
│   │   │   ├── useGitLog.ts
│   │   │   ├── useTerminal.ts
│   │   │   └── useKeybindings.ts
│   │   └── styles/
│   │       ├── tokens.css
│   │       ├── typography.css
│   │       └── global.css
│   ├── public/
│   ├── index.html
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── package.json
├── docs/
│   ├── architecture.md
│   ├── ipc-contract.md
│   ├── terminal-correctness-matrix.md  # v2 新增
│   ├── security-model.md               # v2 新增
│   ├── distribution.md                 # v2 新增：公证 / AppImage / updater
│   └── release-notes/
├── scripts/
│   ├── bootstrap.sh
│   ├── benchmark-gitlog.sh
│   ├── notarize-macos.sh               # v2 新增
│   └── verify-appimage.sh              # v2 新增
└── tests/
    ├── e2e/
    └── fixtures/
```

---

## 5. 数据模型

### 5.1 核心实体（Rust struct 签名）

```rust
// crates/vibestation-core/src/workspace.rs
pub struct Workspace {
    pub id: WorkspaceId,              // UUID v7（时间可排序）
    pub name: String,
    pub root_path: PathBuf,
    pub repo_id: Option<RepoId>,      // 若是 git 仓库
    pub terminal_ids: Vec<TerminalSessionId>,
    pub active_terminal: Option<TerminalSessionId>,
    pub layout: LayoutPreset,         // 默认 Terminal Only
    pub last_opened_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

// crates/vibestation-core/src/pty/session.rs
pub struct TerminalSession {
    pub id: TerminalSessionId,
    pub workspace_id: WorkspaceId,
    pub title: String,                // 自动从 cwd/shell 派生
    pub cwd: PathBuf,
    pub shell: ShellKind,             // Zsh | Bash | Fish(v0.3) | Nushell(v0.3)
    pub profile_id: Option<ProfileId>,
    pub pid: Option<u32>,
    pub state: SessionState,          // Running | Exited(code) | Crashed
    // 运行时字段不序列化
    pub master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    pub writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub child: Arc<Mutex<Box<dyn Child + Send + Sync>>>,
}

// crates/vibestation-core/src/profile.rs
pub struct TerminalProfile {
    pub id: ProfileId,
    pub name: String,
    pub source: ProfileSource,        // Ghostty | ITerm2 | Alacritty | Custom
    pub font_family: String,
    pub font_size: f32,
    pub theme: ThemeColors,           // fg/bg/cursor/ansi16
    pub cursor_style: CursorStyle,
    pub keybindings: Vec<Keybinding>,
    pub env: HashMap<String, String>,
    pub source_file: Option<PathBuf>,
}

// crates/vibestation-core/src/git/types.rs
pub struct GitRepository {
    pub id: RepoId,
    pub root_path: PathBuf,
    pub current_branch: Option<BranchInfo>,
    pub remote_url: Option<String>,
    pub last_fetched_at: Option<OffsetDateTime>,
    // 运行时（v2：默认只持 git2；gix 句柄在 Spike 后才引入）
    pub git2_repo: git2::Repository,
    pub gix_repo: Option<gix::Repository>,  // v2：None 直到 Spike 确认
}

pub struct CommitNode {
    pub oid: String,
    pub short_oid: String,
    pub summary: String,
    pub body: Option<String>,
    pub author: Signature,
    pub committer: Signature,
    pub parents: Vec<String>,
    pub time: OffsetDateTime,
    pub refs: Vec<RefLabel>,          // 分支/tag 贴标签（MVP 主要展示方式）
    pub rail: Option<u16>,            // v2：MVP 不用，v0.2 rail 图启用
}

pub struct FileDiff {
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub status: DiffStatus,
    pub hunks: Vec<Hunk>,
    pub binary: bool,
    pub stats: DiffStats,
}

pub struct BranchInfo {
    pub name: String,
    pub full_ref: String,
    pub kind: BranchKind,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub head_commit: String,
}

// crates/vibestation-core/src/config.rs
pub struct AppConfig {
    pub schema_version: u32,
    pub default_shell: ShellKind,
    pub default_profile_id: Option<ProfileId>,
    pub recent_workspaces: Vec<WorkspaceId>,
    pub telemetry_enabled: bool,      // 默认 false
    pub keybindings: Vec<Keybinding>,
    pub ui_theme: UiTheme,
}

pub struct UserSession {
    pub active_workspace: Option<WorkspaceId>,
    pub window_bounds: WindowBounds,
    pub last_crash: Option<CrashReport>,
}
```

> **v2 变更说明**：
> - `GitRepository.gix_repo` 改为 `Option<gix::Repository>`，Spike Day 3 benchmark 后决定是否引入。MVP 默认 `None`，所有读路径走 `git2`。
> - `CommitNode.rail` 改为 `Option<u16>`，MVP 不计算 rail 索引（无自绘 rail 图）。

### 5.2 持久化分类

| 数据 | 存储 | 理由 |
|------|------|------|
| `AppConfig` | TOML 文件（`~/.config/vibestation/config.toml`）| 用户可编辑 |
| `Workspace` 列表 | **redb（默认）/ rusqlite（Spike 后 benchmark 决定）** | 高频读写 |
| `TerminalProfile` | 同上 + 源文件路径引用 | 支持重新同步 |
| `UserSession` | 单行 | 崩溃恢复 |
| `TerminalSession` 运行时 | 纯内存 `HashMap<Id, Arc<TerminalSession>>` | PTY 资源不可序列化 |
| `GitRepository` 运行时 | 纯内存 `HashMap<RepoId, GitRepository>` | 句柄不可序列化 |
| `CommitNode` 缓存 | 纯内存 LRU（容量 10k）| 切 Tab 时避免重算 |
| `FileDiff` | 不缓存 | 每次按需计算 |

**redb vs rusqlite 决策（v2 未锁定）**：
- Spike Week 0 Day 4 追加半天：分别用 redb 和 rusqlite 跑相同场景（10 个 workspace × 100 个 profile × 10k terminal 快照）
- 对比指标：写入延迟 P99、读取延迟 P99、磁盘占用、损坏恢复能力、crash-consistency
- redb 有显著优势（P99 优于 20%）才锁定 redb；否则用 rusqlite（更成熟、更多工具链）
- 决策在 Spike Week 结束前写入 `docs/adr/0001-storage-choice.md`

### 5.3 窗口 / Tab / Pane 系统

> **v2 说明**：Pane 系统的三层模型保持 v1 设计。但 (a) 默认布局改为 **Primary Sidebar 展开（workspace 切换即时可见）+ Secondary Sidebar（Git Log）+ Bottom Panel 收起**（对齐原型 `design/directions/1-calm-studio.html` 的 `DEFAULT_STATE = { primary:true, secondary:false, bottom:false }`）—— Codex 评审（2026-04-18）建议"全收起"，但权衡后仍保留 Primary 展开，因为 workspace 切换是核心卖点，首次打开应即时可见；(b) Smart Layouts 合并 AI+Watch 和 AI+Test 为单一 "AI + Runner"；(c) AI-Aware Pane 联动降级为 v1.0 vision，MVP / README 不宣传。

#### 5.3.1 三层模型

```
Workspace（项目容器）
  └─ Tab（独立的"桌面"，同一窗口可多 Tab）
      └─ Pane（Tab 内的窗格，可嵌套分屏）
          └─ PaneContent（ClaudeCli / CodexCli / Shell / TaskRunner / DiffViewer / LogFollower）
              └─ TerminalSession（仅 CLI/Shell 类绑定 PTY）
```

类比 tmux：**Workspace = session** / **Tab = window** / **Pane = pane**。

#### 5.3.2 默认布局（v2 调整）

**初次打开应用 / 新建 Workspace 的默认视图**（与原型 `design/directions/1-calm-studio.html` 的 `DEFAULT_STATE` 一致）：

```
┌──────────────────────────────────────────────────────────┐
│  Tab bar + Workspace 名 + Theme toggle                   │
├────────┬──────────────────────────────────────────┬──────┤
│Primary │                                          │ AS-R │
│Sidebar │                                          │(细条)│
│ (236w) │     单 Pane 终端（充满剩余宽度）          │ (36w)│
│        │                                          │      │
│ WS 列表│                                          │ [G]  │
│ 分支树 │                                          │ [S]  │
│        │                                          │ [R]  │
├────────┴──────────────────────────────────────────┴──────┤
│  Status bar: branch · unstaged · Claude status · ctx     │
└──────────────────────────────────────────────────────────┘
```

**展开**：Primary Sidebar（236px，显 Workspace 列表 + 分支树）· Right Activity Strip（36px 细条，Git/Commit/Diff 图标按钮）
**收起**：Secondary Sidebar（Git Log 400px）· Bottom Panel（Problems/Output/Diff 240px）

**切换快捷键**：
- `⌘B` — toggle Primary Sidebar（左栏）
- `⌘9` — toggle Secondary Sidebar（Git Log，**致敬 JetBrains ⌘9**）
- `⌘J` — toggle Bottom Panel（VSCode 风）

> **设计依据与权衡**：
> - Primary 展开 → 用户首次打开就看到 "workspace 切换" 这个核心卖点；否则面对纯终端不知道怎么多项目管理
> - Secondary + Bottom 收起 → 保持视觉干净，避免 Codex 批评的 "UI chrome 过重"
> - Codex 2026-04-18 评审建议"全收起"，但权衡后保留此折中（workspace 即时可见 > 绝对干净）
> - 对齐 Calm Studio 定稿视觉方向

#### 5.3.3 Rust 数据模型

```rust
pub struct Tab {
    pub id: TabId,
    pub workspace_id: WorkspaceId,
    pub layout: LayoutNode,
    pub active_pane_id: PaneId,
    pub title: String,
}

pub enum LayoutNode {
    Leaf(PaneId),
    Split {
        direction: SplitDir,
        ratio: f32,                  // 0.0..1.0
        left: Box<LayoutNode>,
        right: Box<LayoutNode>,
    },
}

pub enum SplitDir { Horizontal, Vertical }

pub struct Pane {
    pub id: PaneId,
    pub tab_id: TabId,
    pub content: PaneContent,
    pub title: String,
}

pub enum PaneContent {
    ClaudeCli(TerminalSessionId),
    CodexCli(TerminalSessionId),
    Shell(TerminalSessionId),
    TaskRunner { command: String, pane_a_link: Option<PaneId> },
    DiffViewer { file_path: PathBuf },
    LogFollower { file_path: PathBuf, tail_lines: usize },
}
```

#### 5.3.4 Smart Layouts 预设库（v2 合并）

| 布局名 | 结构 | 用途 | MVP |
|-------|------|------|-----|
| **Solo**（默认）| 单 Leaf | 单终端，Git 面板收起 | ✅ |
| **AI + Runner**（v2 合并 Watch/Test）| H(0.55, ClaudeCli \| TaskRunner[用户自定义 cmd]) | 左 Claude、右运行任意命令（cargo watch / pytest / npm run dev / `pnpm test -- --watch`）| ✅ |
| **Dual AI** | H(0.5, ClaudeCli \| CodexCli) | 对比双 agent | v0.2 |
| **Triple Review** | H(0.5, ClaudeCli \| V(0.5, TaskRunner \| LogFollower)) | 左 AI、右上 run、右下 log | v0.2 |
| **Quad** | 2×2 | 完全自定义 | v0.2 |

> v2 去除了 v1 的 "AI + Watch" / "AI + Test" 两个预设（codex 指出这只是同一 H-split + 不同命令，不是两种布局）。统一为 "AI + Runner"，命令由用户自行输入（含常见一键模板：cargo watch、pytest、npm run dev、jest --watch 等）。

自定义预设存为 TOML：

```toml
# ~/.config/vibestation/layouts.toml
[[layout]]
name = "AI + Runner"
key = "cmd+1"
root.split = { dir = "horizontal", ratio = 0.55 }
root.left.leaf = { type = "claude-cli" }
root.right.leaf = { type = "task-runner", cmd = "cargo watch -c -x test" }
```

#### 5.3.5 快捷键（v2 修订，解决 v1 冲突）

v1 原型在三处快捷键冲突上被 codex 指出问题：⌘K 同时被标 Command Palette 和 Git Commit；⌘D 同时代表 Pane Split 和 Diff；⌘W 抢占 macOS 关闭标签。v2 修订：

| 动作 | 快捷键 | 备注 |
|------|--------|------|
| Command Palette | `⌘K` | 唯一用途（v1 冲突解决）|
| Git Commit 面板 | `⌘⇧K` | 从 ⌘K 改到 ⌘⇧K |
| 分屏（右）| `⌘\` | 对齐 iTerm2，不占用 ⌘D |
| 分屏（下）| `⌘⇧\` | 同上 |
| Diff 视图 | `⌘D` | 回归 macOS 常用 |
| 关闭 Pane | `⌘⌃W` | 避开 macOS 默认 ⌘W 关窗 |
| 关闭 Tab | `⌘W` | 保留 macOS 默认语义 |
| 跳邻居 | `⌘⌥ ←/→/↑/↓` | 方向键切焦点（v0.2）|
| 最大化 | `⌘Enter` | 临时全屏，再按恢复（v0.2）|
| 调整大小 | `⌘⌃ ←→↑↓` | 10% 步进 |
| 弹为独立窗口 | `⌘⇧O` | Detach（v0.3+）|
| Smart Layouts | `⌘0…⌘5` | 快速套预设 |
| 唤起 Git 面板 | `⌘1` | v2 新增（默认收起后需要快捷键唤出）|
| 唤起 Workspace 面板 | `⌘2` | 同上 |

#### 5.3.6 AI-Aware Pane 联动（v2：v1.0 vision，MVP 不实现、README 不宣传）

AI-Aware Pane 联动作为 v1.0 的升级故事设计保留，但：

- **MVP（v0.1）、v0.2、v0.3 不实现**
- **README / landing page / Product Hunt / Show HN 文案都不提及**
- **v1.0 正式实现前必须有 spike 验证**：用真实 Claude CLI transcript + rustc/tsc/gcc 至少 3 种编译器输出样本，证明 `parsed_issues` 字段的解析可行且稳定

内部设计（保留，不对外）：

1. Pane A 跑 Claude CLI，Pane B 跑 `cargo watch` / `pytest` / 任意命令
2. `notify-debouncer-mini` 监听 workspace 文件
3. Pane B 的 `TaskRunner { pane_a_link: Some(pane_a_id) }` 订阅 Pane A 的文件修改事件
4. 失败反哺：若 Pane B 退出码 ≠ 0，解析错误输出，Pane A 顶部出现"[点此发给 Claude]" 提示
5. 点击后把错误粘贴为新 prompt

> **为什么对外叙事降级**：CodexMonitor、VSCode、JetBrains 已经把 Problems 面板做得很好，Vibestation 如果把"反哺给 AI"写成核心卖点但实现不稳定，只会被当作"又一个 AI 噱头"。先把基础打稳，v1.0 阶段用真实效果说话。

#### 5.3.7 IPC 补充命令 / 事件

新增命名空间 `pane:*`（§6.2 会补入）：

```typescript
// 命令
"pane:split"       ({ pane_id, direction, content_type }) -> PaneId
"pane:close"       ({ pane_id }) -> ()
"pane:resize"      ({ pane_id, ratio: f32 }) -> ()
"pane:focus"       ({ pane_id }) -> ()
"pane:maximize"    ({ pane_id, toggle: bool }) -> ()       // v0.2
"pane:detach"      ({ pane_id }) -> WindowId               // v0.3+
"pane:link"        ({ parent, child, kind }) -> ()         // v1.0
"layout:apply"     ({ tab_id, preset_name }) -> ()
"layout:save"      ({ name, root }) -> ()

// 事件
"pane:created"     { tab_id, pane_id, content }
"pane:closed"      { pane_id }
"pane:focused"     { pane_id }
"pane:linked"      { parent_pane, child_pane, kind }       // v1.0
"pane:trigger"     { pane_id, reason, files? }             // v1.0
"pane:build-failed"{ pane_id, exit_code, error_text, parsed_issues }  // v1.0
```

#### 5.3.8 MVP 范围（v2 收紧）

**MVP 必做**：
- Tab 基础（创建 / 关闭 / 切换）
- 单 Pane（Solo 布局，默认）
- 水平 / 垂直分屏，**最多 1 层嵌套**（最多 4 个 Pane）
- 快捷键：`⌘\` / `⌘⇧\` / `⌘⌃W` / `⌘W` 四个
- 分隔条拖拽调整比例
- 预设布局 2 种：**Solo / AI + Runner**

**推到 v0.2**：
- 任意嵌套（2 层以上）
- Dual AI / Triple Review / Quad 预设
- 方向键跳邻居、最大化（⌘Enter）

**推到 v0.3**：
- 弹为独立窗口（Detach）
- 非 CLI 类 Pane 内容：DiffViewer / LogFollower
- 自定义 Smart Layouts 保存到 TOML

**推到 v1.0**：
- **AI-Aware Pane 联动（订阅 + 失败反哺）**

#### 5.3.9 实现要点与风险

- **布局引擎**：Rust 持 `LayoutNode` tree，前端接收 JSON 后用递归 SolidJS 组件渲染
- **分隔条拖拽**：纯前端 state，拖完 debounce 200ms 发 `pane:resize` 命令持久化
- **PTY 生命周期**：关闭 Pane 必须 `portable-pty::MasterPty::kill()` 子进程，释放 HashMap 条目
- **焦点**：`active_pane_id` 独立于布局树存储，Tab 切换时焦点不丢
- **性能风险**：嵌套超过 3 层时 SolidJS 递归组件可能触发不必要的重渲染；MVP 1 层嵌套回避
- **相关风险**：见 §9 R19（xterm 频繁 fit）、R20（zombie 累积）、R24（终端正确性）

---

## 6. IPC 接口设计

### 6.1 命名原则

遵循调研 §4.4 五条原则：
- **命名空间**：`terminal:*` / `git:*` / `workspace:*` / `pane:*` / `layout:*` / `config:*` / `profile:*`
- **命令 = 请求-响应**，**事件 = 单向广播**
- **错误统一 `Result<T, String>`**
- **长任务返回 `task_id`**，通过 `cancel_task(task_id)` 取消
- **本地/远端抽象预留**

### 6.2 核心命令清单

#### Workspace（7 个）
```rust
workspace_list() -> Result<Vec<Workspace>, String>
workspace_create(name, root_path) -> Result<Workspace, String>
workspace_delete(id) -> Result<(), String>
workspace_rename(id, new_name) -> Result<(), String>
workspace_open(id) -> Result<Workspace, String>
workspace_close(id) -> Result<(), String>
workspace_set_layout(id, layout) -> Result<(), String>
```

#### Terminal（9 个）
```rust
terminal_create(workspace_id, profile_id, cwd, shell) -> Result<TerminalSessionId, String>
terminal_write(id, data) -> Result<(), String>
terminal_resize(id, cols, rows) -> Result<(), String>
terminal_kill(id) -> Result<(), String>
terminal_list(workspace_id) -> Result<Vec<TerminalSession>, String>
terminal_set_title(id, title) -> Result<(), String>
terminal_pop_external(id, target) -> Result<(), String>   // v0.3
terminal_serialize_buffer(id) -> Result<String, String>
terminal_restore_buffer(id, snapshot) -> Result<(), String>
```

#### Git（MVP 10 个 + v0.2 扩展 + v0.3 扩展）

**MVP（v0.1）**：
```rust
git_open_repo(path) -> Result<RepoId, String>
git_close_repo(id) -> Result<(), String>
git_status(id) -> Result<RepoStatus, String>
git_log(id, params) -> Result<TaskId, String>            // 异步 + 事件回推
git_commit_detail(id, oid) -> Result<CommitNode, String>
git_diff_commit(id, oid) -> Result<Vec<FileDiff>, String>
git_diff_workspace(id) -> Result<Vec<FileDiff>, String>
git_stage(id, paths) -> Result<(), String>
git_unstage(id, paths) -> Result<(), String>
git_commit(id, message, amend) -> Result<String, String>
git_branch_list(id) -> Result<Vec<BranchInfo>, String>    // 只读列表
```

**v0.2 新增**：
```rust
git_branch_create(id, name, from) -> Result<(), String>
git_branch_checkout(id, name) -> Result<(), String>
git_push(id, remote, branch) -> Result<TaskId, String>
git_pull(id, remote, branch, strategy) -> Result<TaskId, String>
git_fetch(id, remote) -> Result<TaskId, String>
```

**v0.3 新增**：
```rust
git_reset(id, oid, mode) -> Result<(), String>
git_rebase(id, onto, interactive) -> Result<TaskId, String>
git_merge(id, branch) -> Result<TaskId, String>
git_cherrypick(id, oid) -> Result<(), String>
```

> **v2 MVP 调整**：v1 把 Push/Pull/Fetch 塞进 MVP，Codex 指出这会让 MVP 工期失控。v2 MVP 只保留只读 Git + Commit，写远端操作推迟到 v0.2。

#### Pane / Layout（10 个）
```rust
pane_split(pane_id, direction, content_type) -> Result<PaneId, String>
pane_close(pane_id) -> Result<(), String>
pane_resize(pane_id, ratio) -> Result<(), String>
pane_focus(pane_id) -> Result<(), String>
pane_maximize(pane_id, toggle) -> Result<(), String>     // v0.2
pane_detach(pane_id) -> Result<WindowId, String>         // v0.3+
pane_link(parent, child, kind) -> Result<(), String>     // v1.0
layout_apply(tab_id, preset_name) -> Result<(), String>
layout_save(name, root) -> Result<(), String>
layout_list() -> Result<Vec<LayoutPreset>, String>
```

#### Config / Profile（6 个）
```rust
config_get() -> Result<AppConfig, String>
config_set(config) -> Result<(), String>
profile_list() -> Result<Vec<TerminalProfile>, String>
profile_import(source, file_path) -> Result<Vec<TerminalProfile>, String>
profile_delete(id) -> Result<(), String>
profile_set_default(id) -> Result<(), String>
```

#### 通用（2 个）
```rust
cancel_task(task_id) -> Result<(), String>
app_version() -> Result<AppVersion, String>
```

**小计**：
- **MVP**：Workspace 7 + Terminal 9 + Git 11 + Pane 4 + Config 6 + 通用 2 = **39 个**
- **v0.2 增量**：Git 5 + Pane 2 = 7 个 → 累计 46
- **v0.3 增量**：Git 4 + Pane 1 = 5 个 → 累计 51
- **v1.0 增量**：Pane 1 = 1 个 → 累计 52

### 6.3 核心事件清单

```
terminal:output           { id, chunk }
terminal:exited           { id, code }
terminal:title-changed    { id, title }
git:log-loaded            { repo_id, task_id, commits, has_more }
git:log-progress          { repo_id, task_id, loaded, total_estimate }
git:status-changed        { repo_id, status }
git:branch-changed        { repo_id, branches }
git:fetch-progress        { repo_id, task_id, stage, percent }   // v0.2
git:push-progress         { repo_id, task_id, stage, percent }   // v0.2
git:operation-done        { repo_id, task_id, outcome }
workspace:opened          { workspace }
workspace:closed          { workspace_id }
workspace:list-changed    { workspaces }
pane:created              { tab_id, pane_id, content }
pane:closed               { pane_id }
pane:focused              { pane_id }
pane:linked               { parent_pane, child_pane, kind }      // v1.0
pane:trigger              { pane_id, reason, files? }            // v1.0
pane:build-failed         { pane_id, exit_code, error_text, parsed_issues }  // v1.0
config:changed            { config }
profile:imported          { profiles, source }
fs:dirty                  { repo_id, paths }
app:error                 { code, message, context }
app:crash                 { report }
```

**小计：22 个事件**（MVP 阶段 16 个）

所有事件 payload 均为 `serde_json` 可序列化 struct，类型定义放在 `crates/vibestation-core/src/events.rs`，前端通过 `pnpm run gen:types`（基于 `ts-rs`）自动生成 TS 类型。

---

## 7. 周级 Milestone 甘特图（v2 加宽）

> v2 调整：
> - MVP 由 10 周加到 **12 周**（20% buffer）
> - v0.2 由 4 周加到 **5 周**
> - v0.3 由 4 周加到 **5 周**
> - v1.0 总工期由 24-25 周加到 **28-30 周**
> - 每个阶段最后一周留 buffer，不安排新 feature
> - 假设：每周 20-25 小时个人投入（比 v1 的 20-30 收紧下限）

| 周 | 阶段 | 交付物 | 验收标准 | 关键依赖 |
|---|------|--------|----------|----------|
| **W0** | Spike | 5-6 天技术验证 | Tauri Pass/Fail、PTY 吞吐、git2 vs gix、redb vs rusqlite、Claude CLI 录样 | 下文 §附录 A |
| **W1** | v0.1 基建 | Cargo workspace 2 crate 脚手架、CI（lint/test/build）、icon、`tauri.conf.json` 初版、LICENSE（Apache-2.0）+ NOTICE | `cargo test` 绿、GitHub Actions 通过、dmg/AppImage 可产出 | Spike 结论 |
| **W2** | v0.1 PTY | `vibestation-core/pty` 完整：单读线程 + mpsc、zsh/bash 启动、fix-path-env | 单 workspace 单 Tab 在 mac 上可跑 Claude CLI 不丢字符，10 秒 `yes` 不卡 | 参考 CodexMonitor terminal.rs |
| **W3** | v0.1 前端终端 | `Terminal.tsx` + `TerminalTabs.tsx`，xterm 5.5 + fit/web-links/serialize addon | 5 个 Tab 并存切换，每个保留独立 buffer，切换延迟 <50ms | W2 |
| **W4** | v0.1 Pane 系统 | `PaneContainer.tsx` + `SplitLayout.tsx`，1 层嵌套（4 Pane），快捷键 `⌘\` `⌘⇧\` `⌘⌃W` | 4 Pane 并存，拖拽调整比例持久化 | W3 |
| **W5** | v0.1 Git 读 | `vibestation-core/git/sync/logwalker.rs` + `revlog.rs`（AsyncLog 分批）| 10 万 commit 仓库 Log 首屏 <500ms，滚动不卡 | 参考 gitui/asyncgit |
| **W6** | v0.1 Git Log UI | `GitLogView.tsx` + `CommitList.tsx`（列表 + 分支标签贴，**无自绘 rail**）+ `@tanstack/solid-virtual` | 虚拟化滚动 60fps、分支/tag 标签贴对、merge commit 正确显示双 parent | W5 |
| **W7** | v0.1 Diff 基础 | `DiffView.tsx` 基础行对比（+/-），**无 Monaco、无复杂语法高亮**；大文件降级提示 | 100KB 文件 diff 渲染 <100ms；10MB 以上降级为"预览前 200 行" | W5 |
| **W8** | v0.1 Commit 视图 | 右侧 `CommitDetail.tsx`（元数据 + 文件列表 + 跳转 Diff）、`StatusPanel.tsx`（stage/unstage/untracked 分组） | 任意 commit 点进详情 <100ms，stage/unstage 操作正确 | W6, W7 |
| **W9** | v0.1 Commit 操作 | `git_commit`（勾文件 + 消息 + amend），无远端操作 | 可在本地 repo 成功产出 commit，能被 `git log` 看到 | W8 |
| **W10** | v0.1 配置导入 | Ghostty（mac/linux）+ iTerm2（mac）+ Alacritty（linux）解析器；`profile_import` 命令 | 3 种配置各至少 10 条真实用户配置导入不报错 | `toml_edit` + `plist` |
| **W11** | v0.1 终端正确性矩阵 | §10.6 矩阵逐项过一遍；崩溃恢复基础；macOS 公证流水线搭建 | 矩阵 100% 通过；notarization 可在 CI 上跑一轮 | `docs/terminal-correctness-matrix.md` |
| **W12** | v0.1 发布 ★ + buffer | 文档、landing page v0、首个 GitHub Release v0.1.0；跨平台 QA | 见 §10 MVP 验收；Show HN + r/rust 发布；<30MB（Tauri）或 <80MB（Electron）包体积 | 全部前置周 |
| **W13** | v0.2 分支 | `git_branch_create/checkout/delete` + UI、`BranchList.tsx` | 分支切换 <500ms、脏工作区切换有警告 | W12 |
| **W14** | v0.2 Push/Pull/Fetch | `git_push` / `git_pull` / `git_fetch` 带进度条 | 在真实 GitHub repo 上双向同步，SSH/HTTPS 均工作 | `vibestation-core/git/sync/push_pull.rs` |
| **W15** | v0.2 Pane 扩展 | 任意嵌套、方向键跳邻居、`⌘Enter` 最大化、Dual AI / Triple Review / Quad 预设 | 3 层嵌套不卡 60fps；预设切换 <100ms | W4 |
| **W16** | v0.2 自绘 rail graph | `CommitGraph.tsx`（Canvas）+ rail 算法（抄 gitui）| linux kernel 仓库 rail 图正确 | W6 |
| **W17** | v0.2 发布 ★ + buffer | v0.2.0 Release；信号指标观察 | 信号指标：3 篇独立博客提及、10+ alpha 测试者 | W13-W16 |
| **W18** | v0.3 Rebase | `git_rebase` + 交互式 rebase UI、冲突标记 | pick/squash/edit/drop 可用 | - |
| **W19** | v0.3 Merge/Cherry-pick | `git_merge` / `git_cherrypick` + 冲突解决面板 | 三路 diff 展示，手动解决后可 continue | W18 |
| **W20** | v0.3 Pop to External | `terminal_pop_external`，mac 支持 Ghostty/iTerm2，linux 支持 Ghostty/Alacritty | 点击一键弹到外部终端，cwd/env/命令历史保留 | - |
| **W21** | v0.3 Diff 高级 | `DiffView` 支持语法高亮（shiki lazy load）、大文件流式加载 | 1MB 文件 diff <300ms | - |
| **W22** | v0.3 发布 ★ + buffer | v0.3.0 Release | - | - |
| **W23** | v1.0 AI-Aware Spike | 用真实 Claude CLI transcript 验证 `parsed_issues` 解析；多语言编译器（rustc/tsc/gcc）诊断样本收集 | Spike 结论：可行 / 不可行；若不可行则 descope | - |
| **W24** | v1.0 Claude 深集成 | session ↔ commit 自动绑定、"用 Claude 生成 commit message" | 每个 Claude session 产生的 commit 带 session 元数据 | W23 |
| **W25** | v1.0 AI-Aware Pane 联动 | Pane A-B 订阅 + 失败反哺 | Claude CLI 改代码后 `cargo watch` 失败可一键回填 | W23 |
| **W26** | v1.0 一键回滚 | 回滚整个 AI session 或单文件 | 回滚后工作区干净、可复原 | W24 |
| **W27** | v1.0 性能 | 基准测试固化：启动 <2s、Log <500ms、切 Tab <50ms、内存 <300MB；benchmark CI 回归报警 | 指标达标 | W5-W26 |
| **W28** | v1.0 Polish | 键盘快捷键覆盖 95% 操作、主题系统、错误上报完善 | 键盘用户不摸鼠标可完成全流程 | - |
| **W29** | v1.0 文档 + 安全审计 | 用户手册、开发者指南、API ref、3 篇 tutorial、外部安全审计过一轮 | docs 上线 | - |
| **W30** | v1.0 发布 ★ + buffer | v1.0.0 Release、Product Hunt、Show HN 二发 | 信号指标：HN 首页再进一次、1 家媒体报道、Sponsors 开通 | 全部 |

**里程碑汇总**：
- ★ W12 — v0.1 MVP（多 Tab 终端 + 只读 Git + Commit + Pane + 配置导入）
- ★ W17 — v0.2（分支 + Push/Pull/Fetch + rail 图 + Pane 扩展）
- ★ W22 — v0.3（完整 Git workflow + 外部终端 + 高级 Diff）
- ★ W30 — v1.0 GA（AI-Aware 首次对外亮相）

---

## 8. 测试策略

### 8.1 单元测试

| Crate / 模块 | 覆盖率目标 | 关键测试 |
|-------|-----------|----------|
| `vibestation-core::config` | 85% | config 解析、persistence 迁移、profile 导入边界（坏 TOML、空 plist）|
| `vibestation-core::git` | 80% | logwalker 排序稳定性、diff hunks 边界、branch ahead/behind |
| `vibestation-core::pty` | 70% | PTY session 创建销毁、resize、mpsc 分发无丢失 |
| `vibestation-core::security` | 90% | TaskRunner 白名单、命令 sanitize、路径注入 |
| `vibestation-app` | 50% | IPC command 参数校验、error 序列化 |

工具：`cargo nextest` + `cargo tarpaulin`（mac/linux 都支持），CI 矩阵跑。

### 8.2 集成测试

- **Tauri E2E**：`webdriver` + `tauri-driver`（或 Playwright + headless Tauri），跑关键用户流程：
  - 创建 workspace → 开 Tab → 跑 `ls` → 确认输出
  - 打开 git 仓库 → Log 加载 → 点击 commit → 看 Diff
  - 修改文件 → stage → commit
- **Git 集成**：`tests/fixtures/` 用 tar.zst 存预构造的 .git 目录（10k / 100k / 1M commit 三档）
- **Git edge-case 专项测试（v2 新增）**：worktree、submodule、LFS、nested repo、bare repo。每种场景验证 MVP "不崩溃"承诺。

### 8.3 手动 QA 清单（每次发布前必跑）

跨平台矩阵：**macOS 15 + Ubuntu 24（Wayland）+ Ubuntu 24（X11）**

- [ ] 冷启动 < 2s（mac）/ < 3s（linux）
- [ ] 单 Tab Claude CLI 无丢字
- [ ] 单 Tab Codex CLI 无丢字
- [ ] 10 个 Tab 同时运行（5 个 `yes`、5 个 Claude）内存 <500MB
- [ ] 导入 Ghostty 配置（真实用户样本）字体/主题渲染一致
- [ ] 10 万 commit 仓库（linux kernel）Log 首屏 <500ms
- [ ] Diff 打开 10MB 大文件不卡（降级为"文件过大，预览前 200 行"提示）
- [ ] Push/Pull 网络中断时错误可恢复（v0.2）
- [ ] 剪贴板复制（Wayland + X11 都测）
- [ ] OSC52 剪贴板转发（tmux over SSH 场景）
- [ ] IME 中文 / 日文输入正常
- [ ] 键盘快捷键全覆盖跑一遍
- [ ] 崩溃恢复：kill -9 再打开，tab 列表和 workspace 状态保留
- [ ] notarization 后 dmg 在干净 mac（Gatekeeper 启用）可直接打开
- [ ] AppImage 在干净 Ubuntu 24 可执行（不需额外依赖）

### 8.4 性能基准

| 指标 | 目标 | 测试方法 |
|------|------|----------|
| 冷启动到可用 | <2s（mac M1）/ <3s（linux）| `scripts/benchmark-startup.sh` |
| 10 万 commit Log 首屏 | <500ms | `scripts/benchmark-gitlog.sh`（linux kernel）|
| 切 Tab 延迟 | <50ms | Playwright 记录 |
| 终端输出吞吐 | >10MB/s（`yes \| head -c 10M`）| 手动计时 |
| 内存占用（5 workspace × 10 tab）| <500MB | Activity Monitor / `ps` |
| 安装包大小（Tauri） | <30MB（mac dmg）/ <40MB（linux AppImage）| `ls -lh target/release/bundle/` |
| 安装包大小（Electron fallback） | <80MB（mac dmg）/ <100MB（linux AppImage）| 同上 |

benchmark 结果写入 `docs/benchmarks/`，CI 跑完自动 PR 更新，回归超 20% 报警。

---

## 9. 风险登记册（v2 扩充至 30 条）

> v2 调整说明：
> - R12 Wayland 升级至 CRITICAL（MVP 验收门槛）
> - R13 commit graph 降级至 低/低（MVP 不做 rail 图，改列表 + 分支标签贴）
> - R17 单人耗尽改实质缓解（明确"停摆触发条件"）
> - 新增 R21-R30 共 10 条（notarization、Linux 分发、auto-updater、终端正确性、TaskRunner 安全、Git edge-case、状态恢复、商标、AI API 变更、telemetry 合规）

| # | 风险 | 概率 | 影响 | 对策 | Owner | 触发时机 |
|---|------|------|------|------|-------|----------|
| R1 | Claude CLI 输出协议与 Codex 不同，解析失败 | 高 | 高 | Spike Day 5 实机录制样本；v1.0 W23 单独 spike 前不锁定实现 | 核心作者 | Spike Day 5 |
| R2 | portable-pty 多 Tab Mutex 瓶颈 | 中 | 高 | 单读线程 + mpsc 分发 | 核心作者 | W2 |
| R3 | git2 大仓库 log 慢到不可用 | 中 | 高 | Spike Day 3 benchmark；若慢则引入 gix 做读路径 | 核心作者 | W5 |
| R4 | macOS GUI 启动 PATH 为空，CLI 找不到 | 高 | 中 | `fix-path-env` crate 必装 | 核心作者 | W1 |
| R5 | Wayland 剪贴板 / 窗口管理 API 差异 | 中 | 中 | 使用 `tauri-plugin-clipboard-manager`，Wayland fallback 到 `wl-copy` | 核心作者 | W3 |
| R6 | Monaco 体积爆炸拖累冷启动 | 高 | 中 | 自建 diff（diff npm + HTML 行对照），MVP 基础版 | 核心作者 | W7 |
| R7 | Floem 诱惑（Rust 原生）走弯路 | 低 | 高 | 坚持 SolidJS 不动摇 | 核心作者 | 持续 |
| R8 | npm postinstall 链过复杂 | 中 | 低 | 禁用 predev/prebuild 链，只保留 `gen:types` | 核心作者 | W1 |
| R9 | git2 vendored-libgit2 首次编译慢（3-5 分钟）| 高 | 低 | CI 缓存 target/ + ~/.cargo/registry | DevOps | W1 |
| R10 | alacritty_terminal 只发 git | 中 | 低 | MVP 不用 alacritty_terminal，只用 xterm.js | 核心作者 | 决策时 |
| R11 | 多 workspace 文件监听爆炸（node_modules）| 高 | 中 | `notify-debouncer-mini` + .gitignore 模式忽略 | 核心作者 | W2 |
| R12 | **Tauri 2 在 Ubuntu 24 Wayland 下不稳定** | **中** | **CRITICAL** | **Spike Week 0 Day 1-2 硬验证，失败回退 Electron 28+；该风险是 MVP 验收门槛** | 核心作者 | Spike Day 1-2 |
| R13 | commit graph rail 算法复杂度低估 | 低 | 低 | **MVP 不做 rail，用列表 + 分支标签贴**；v0.2 W16 单独一周做 rail | 核心作者 | v0.2 W16 |
| R14 | CLI session 边界判定不稳定 | 高 | 高 | v1.0 vision，MVP 不做；W23 专项 spike；三档策略：显式标记 > 时间窗口 > 手动 | 核心作者 | W23 |
| R15 | 开源社区冷启动，无人 star/反馈 | 高 | 中 | 首发 Show HN + r/rust + r/selfhosted + HN；信号指标非数字 KPI | 核心作者 | W12 |
| R16 | Apache-2.0 下有公司抄袭闭源 | 中 | 低 | Apache-2.0 已含 patent grant；README 强调原创性；关键 demo 视频留存 | 核心作者 | v1.0 后 |
| R17 | 单人维护精力耗尽 | 高 | 高 | **停摆触发**：连续 2 周 < 5 小时投入 → 进入 §10.5 hibernation 模式，README 坦诚公开；v0.3 后主动招 1-2 个 co-maintainer | 核心作者 | 持续 |
| R18 | 依赖大版本升级（gix 0.70+）破坏 API | 中 | 中 | Dependabot 开通 + 每月人工 review；锁 `Cargo.lock` | 核心作者 | 持续 |
| R19 | Pane 嵌套布局触发 xterm.js 频繁 `fit` 调用导致性能抖动 | 中 | 中 | `ResizeObserver` + debounce 50ms；MVP 限制 1 层嵌套 | 核心作者 | W4 |
| R20 | Pane 关闭时 PTY 子进程未 kill 造成 zombie | 高 | 中 | 关 Pane 必调 `MasterPty::kill()`；测试套件专门验证 PID 回收 | 核心作者 | W2 |
| **R21** | **macOS notarization + Hardened Runtime 配置错误导致 dmg 在用户机打不开** | **高** | **CRITICAL** | W11 搭建 `notarize.yml` 流水线；Apple Developer ID 申请（需 1-2 周审核）；Entitlements.plist 准确配置；scripts/notarize-macos.sh 自动化；真机测试 Gatekeeper 启用场景 | 核心作者 | W11 |
| R22 | Linux 分发碎片化（AppImage vs deb vs Flatpak vs Snap）| 中 | 中 | **AppImage 优先**（签名 + sha256 分发）；deb 次之（Debian/Ubuntu 官方风格）；Flatpak / Snap 交给社区贡献；文档明确支持分级 | 核心作者 | W11 |
| R23 | auto-updater 错误导致用户无法回滚或收不到更新 | 中 | 高 | Tauri updater 签名验证；staged rollout（10% → 50% → 100%）；回滚机制（保留前一版本）；fail-closed 默认 | 核心作者 | v0.2+ |
| **R24** | **终端正确性问题（IME/CJK/OSC52/mouse/alt-screen/tmux 兼容）** | **高** | **CRITICAL** | W11 专项验收矩阵（§10.6）；每项可 demo；不通过不 release；OSC52 剪贴板转发、bracketed paste、mouse reporting、alt-screen 切换、tmux 嵌套全测 | 核心作者 | W11 |
| R25 | TaskRunner 任意命令执行被注入 | 中 | 高 | §13 安全边界：白名单机制 + 二次确认 + 最小权限；AI 回填 prompt sanitize；不开 sudo；不写系统目录 | 核心作者 | v0.1（基础） / v1.0（AI 回填） |
| R26 | Git edge-case（worktree/submodule/LFS/nested repo/partial clone）行为异常 | 高 | 中 | Non-goals 声明 MVP 仅保证"不崩溃"；集成测试覆盖 5 种场景；v0.3-v1.0 渐进支持 | 核心作者 | W5 起 |
| R27 | 本地状态损坏（redb 文件损坏 / 升级迁移失败）导致用户数据丢失 | 中 | 高 | schema_version 字段 + 迁移测试；备份（`~/.config/vibestation/backups/`）；崩溃后启动时自检 + 可选回滚；提供手动导出/导入命令 | 核心作者 | v0.1 |
| R28 | 商标 / 项目名冲突（"vibestation" 被他人注册）| 中 | 中 | v0.1 发布前做商标搜索（USPTO / EUIPO / CNIPA）；域名推到 W10 附近再决定（候选 `.app` / `.dev` / `.io`）；GitHub organization 预注册 | 核心作者 | W10-W11 |
| R29 | AI 提供商 API 变更（Anthropic / OpenAI 折腾）破坏 CLI 集成 | 中 | 中 | 不直接调 API（让用户自己跑 Claude CLI / Codex CLI）；只解析输出；CLI 协议变化在 v1.0 AI-Aware 时才敏感，届时 W23 spike 验证 | 核心作者 | W23 |
| R30 | 崩溃上报 / telemetry 合规（GDPR / CCPA）| 中 | 中 | 默认 `telemetry_enabled = false`；首次启动显式询问；仅收集匿名 crash report + 版本号；有明确 privacy policy；用户可随时导出 / 删除 | 核心作者 | W11 |

---

## 10. MVP 验收标准

### 10.1 功能清单（v2 收紧为 B 折中方案）

**保留（MVP v0.1 必做）**：

- [ ] 启动应用，欢迎页可创建第一个 workspace
- [ ] 选择项目目录，自动识别是否 git 仓库
- [ ] 打开 workspace，**默认视图**：Primary Sidebar 展开（workspace 列表 + 分支树）· Right Activity Strip 细条可见 · Secondary Sidebar（Git Log）+ Bottom Panel 收起（与原型 `design/directions/1-calm-studio.html` DEFAULT_STATE 一致）
- [ ] 终端 Tab：新建/关闭/重命名，切换不丢 buffer
- [ ] **终端可运行 zsh/bash、Claude CLI、Codex CLI、vim、htop、yes、tmux**
- [ ] **Pane 分屏**：`⌘\` 右分屏、`⌘⇧\` 下分屏、`⌘⌃W` 关 Pane，**最多 1 层嵌套（4 Pane）**
- [ ] **Smart Layouts**：一键切换 Solo / AI + Runner 两种预设
- [ ] **Pane 分隔条**：拖拽调整比例，双击复位 50/50，比例持久化
- [ ] **配置导入**：Ghostty（mac/linux）、iTerm2（mac）、Alacritty（linux）
- [ ] **Git Log 只读视图**：commit 列表 + 作者 + 时间 + **分支/tag 标签贴**（**无自绘 rail graph**）
- [ ] 点击 commit 打开详情视图（元数据 + 变更文件列表）
- [ ] **Diff 基础视图**（自绘，非 Monaco，**基础行对比、无复杂语法高亮**）
- [ ] **Git Status 只读面板**：staged / unstaged / untracked 分组
- [ ] **Stage / Unstage** 单文件或整体
- [ ] **Commit 操作**（勾文件 + 写 message + 可勾 amend；**不含 push/pull/fetch**）
- [ ] 多 workspace 同时打开，Tab 切换状态独立
- [ ] 崩溃恢复（基础）：重启后 Tab 和 workspace 状态恢复
- [ ] 双平台打包 + 签名：macOS dmg 经 notarization、Linux AppImage 签名 + sha256
- [ ] **终端正确性矩阵**（§10.6）100% 通过

**砍到 v0.2**：
- [ ] Push / Pull / Fetch
- [ ] 自绘 commit rail graph
- [ ] 分支 create / checkout / delete
- [ ] Pane 任意嵌套 / Dual AI / Triple Review / Quad 预设
- [ ] 方向键跳邻居 / ⌘Enter 最大化

**砍到 v0.3**：
- [ ] Diff 复杂语法高亮
- [ ] Rebase / Merge / Cherry-pick
- [ ] Pop to External Terminal
- [ ] Pane Detach

**砍到 v1.0**：
- [ ] AI-Aware Pane 联动
- [ ] session ↔ commit 自动绑定
- [ ] AI 一键回滚

### 10.2 性能指标

| 指标 | 目标 | 验证方法 |
|------|------|----------|
| 冷启动（mac M1）| <2s（Tauri）/ <3s（Electron fallback）| 三次取均值 |
| 冷启动（Ubuntu 24 x86）| <3s（Tauri）/ <4s（Electron fallback）| 同上 |
| 10 万 commit Log 首屏 | <500ms | 用 linux kernel 仓库 |
| 切 Tab 延迟 | <50ms | Playwright 记录 |
| 10 Tab 并存内存 | <500MB | Activity Monitor |
| 安装包（Tauri mac dmg）| <30MB | ls -lh |
| 安装包（Tauri linux AppImage）| <40MB | ls -lh |
| 安装包（Electron fallback mac dmg）| <80MB | ls -lh |

### 10.3 跨平台验收

**macOS 15（Intel + Apple Silicon）**：全部功能清单通过，dmg 通过 notarization 在 Gatekeeper 启用的干净机上可直接打开。
**Ubuntu 24 Wayland**：全部功能清单通过（剪贴板经 xdg-desktop-portal）。
**Ubuntu 24 X11**：全部功能清单通过。

每个平台单独过一遍 §8.3 手动 QA 清单 + §10.6 终端正确性矩阵。任何一项失败则不发布。

### 10.4 非功能

- [ ] LICENSE 文件 **Apache-2.0** 清晰，含 NOTICE 文件
- [ ] **不签 CLA**（README 贡献段说明）
- [ ] README 双语（英/中）首屏即懂能做什么
- [ ] **README 不提 AI-Aware / Mission Control 叙事**（保留给 v1.0）
- [ ] CONTRIBUTING.md + CoC 就位
- [ ] CHANGELOG 遵循 Keep a Changelog
- [ ] v0.1.0 GitHub Release 上传 mac dmg + linux AppImage（x86_64 + aarch64）
- [ ] SECURITY.md 有效，含安全报告邮箱
- [ ] privacy policy 公开（§9 R30）

### 10.5 降级树（descoping tree，v2 新增）

> 当个人投入时间出现下降，按以下树依次砍：

**每周投入 ≤ 15 小时（正常 → 节能）**：
- 砍配置导入 iTerm2 + Alacritty（只留 Ghostty；覆盖 Persona C，其他两个 persona 可手动配置）
- 砍 Pane 分屏（只保留单 Pane；高级用户可用 tmux 在终端内分屏）
- 双平台 → 仅 macOS（Linux 延后到 v0.2）

**每周投入 ≤ 10 小时（节能 → 紧缩）**：
- 仅保留核心三件套：多 Tab 终端 + Git Log 只读 + Git Status 只读
- 砍 Commit UI（用户用终端 `git commit`）
- 砍配置导入（用户手动编辑 TOML）
- 砍 Diff 视图（用户用终端 `git diff`）

**连续 2 周 < 5 小时投入（停摆触发）**：
- 进入 **hibernation 模式**
- README 顶部加明确 banner："项目维护者暂停中，v0.1 完成日期不定；欢迎 fork"
- 不假装还在活跃开发
- Discord / Twitter 公告一致

> 为什么需要这棵树：v1 的 "v0.3 后招 co-maintainer" 不是真缓解——那时已经太晚。descoping tree 把"砍什么"前置到计划里，避免临场慌乱或鸽掉承诺。

### 10.6 终端正确性验收矩阵（v2 新增）

> 终端产品的"及格线"，每项必须可 demo 才能 release v0.1。

| # | 项 | 验收动作 | 通过标准 |
|---|----|---------|----------|
| T01 | IME 中文输入 | 在终端输入 `echo "你好世界"`（搜狗输入法）| 字符显示完整、光标位置正确、无候选框错位 |
| T02 | IME 日文输入 | 输入假名转汉字 | 同上，Kotoeri / macOS 原生 IME |
| T03 | CJK 双宽字符 | 输入 `echo "中文" \| wc -c` | 列宽计算正确（一个中文占 2 列）|
| T04 | Bracketed paste | 粘贴多行 shell 脚本 | 不被当成每行 Enter 执行；`vim` / `zsh` / `fish` 均识别 |
| T05 | OSC52 剪贴板转发 | tmux over SSH，远端 vim `:"*y`（yank to clipboard via OSC52）| 本地剪贴板收到内容 |
| T06 | Mouse reporting | `vim` 鼠标滚动 / `htop` 鼠标点击 | 滚动平滑、点击响应 |
| T07 | Alt-screen 切换 | 打开 `vim` / `less` / `htop` 再退出 | 退出后主屏 buffer 恢复、无残留 |
| T08 | tmux 嵌套 | 在终端内跑 `tmux`，再开多 pane | 分屏/快捷键/滚动均正常 |
| T09 | ANSI 256 / truecolor | `echo $'\e[38;2;255;100;50mhello\e[0m'` | 显示指定 RGB |
| T10 | ANSI 边界 | 超长单行（10k 字符）输出 | 不卡顿、不崩溃 |
| T11 | resize 响应 | 窗口大小变化 | xterm 正确 fit，无错位 |
| T12 | 快速大量输出 | `yes \| head -c 100M` | 不丢字、CPU < 单核 80% |
| T13 | UTF-8 emoji | `echo "😀 🎉 👨‍💻"` | 宽度正确、组合字符合并显示 |
| T14 | Shell 启动环境 | macOS GUI 启动后 `echo $PATH` | 包含 brew / asdf / mise 等（fix-path-env 生效）|
| T15 | 崩溃后子进程回收 | kill -9 应用 | `ps aux` 无 zombie 子进程 |

---

## 11. 开源治理

### 11.1 License 与贡献框架

- **LICENSE**：**Apache 2.0**（v2 调整）
  - 含 patent grant，比 MIT 更自洽地处理专利问题
  - NOTICE 文件列出依赖的 attribution 要求
- **不签 CLA**（v2 明确）
  - 小型 OSS 项目 CLA 只增加贡献摩擦
  - 除非 v2.0+ 走商业化/双许可证再评估
- **CoC**：Contributor Covenant 2.1，中英双语
- **CONTRIBUTING.md** 要求：
  - 必须通过 `cargo fmt` + `cargo clippy -- -D warnings` + `pnpm lint` + `pnpm typecheck`
  - 新增 public API 必须带 rustdoc + 单测
  - 大改动（>200 行）先开 issue 讨论
  - commit 遵循 Conventional Commits；PR 描述要包含 test plan

### 11.2 README 结构（v2 调整）

**英文主 README**（MVP 发布时）：
1. 一行 slogan：**"Multi-tab terminal + JetBrains-grade Git workbench for Claude CLI / Codex CLI users"**（**无 "Mission Control" / "AI session aware"**）
2. 一张动图 demo：多 Tab 终端 + Git Log 列表 + Commit
3. Why Vibestation?（3 段故事化，聚焦终端 + Git 工作台场景，不讲 AI session）
4. Features（功能清单，对齐 §10.1 保留项）
5. Install（mac dmg / linux AppImage / 编译）
6. Quick Start（5 步上手）
7. Roadmap（列 v0.1 / v0.2 / v0.3 / v1.0 已承诺项；AI session 能力作为 v1.0 单列一行，不展开）
8. Community（Discord / Twitter / Sponsors）
9. Contributing
10. License（**Apache 2.0**）

**中文副 README**：同结构，放在 `README.zh-CN.md`。

**Landing page（vibestation.<tld>）**：同样不提 AI-Aware / Mission Control。

### 11.3 Release 策略

- **semver**：v0.x 期可做 breaking change；v1.0 后严格遵守
- **release-please**：GitHub Actions 集成，从 Conventional Commits 自动生成 CHANGELOG 和 tag
- **发版节奏**：v0.1 后每 2-3 周一版 feature release，每周 patch release（按需）
- **Release artifact**：
  - macOS: Universal dmg（Intel + Apple Silicon），通过 Apple notarization
  - Linux: AppImage（x86_64 + aarch64）+ tar.xz + deb
  - 所有产物 sha256 校验 + GitHub Attestation
  - auto-updater 签名（§14）

### 11.4 模板与自动化

- `.github/ISSUE_TEMPLATE/bug_report.md`：必填 OS / version / 重现步骤 / 日志
- `.github/ISSUE_TEMPLATE/feature_request.md`：必填问题场景 / 期望行为 / 备选方案
- `.github/PULL_REQUEST_TEMPLATE.md`：变更摘要 / 测试计划 / 截图（UI 变更必填）
- **Dependabot**：开 cargo + npm + github-actions 三类，每周聚合 PR
- **CODEOWNERS**：核心作者 + v0.3 后增选 1-2 人
- **SECURITY.md**：披露流程 + 邮箱 + 90 天 coordinated disclosure

### 11.5 CI 流水线（GitHub Actions）

```yaml
# .github/workflows/ci.yml 概要
jobs:
  lint:           # cargo fmt / clippy / pnpm eslint / pnpm stylelint
  test-rust:      # matrix: [ubuntu-24.04, macos-14], cargo nextest
  test-frontend:  # vitest + playwright(headed=false)
  build:          # matrix 构建 dmg/AppImage，产物上传
  benchmark:      # 跑 scripts/benchmark-*.sh，结果存 artifact
  security:       # cargo audit + pnpm audit + CodeQL
  terminal-matrix: # v2 新增：运行 §10.6 自动化子集
```

关键配置：**CI 缓存 `target/` 和 `~/.cargo/registry/`**，否则每次 CI 都要 3-5 分钟编译 vendored-libgit2。

---

## 12. 推广与社区建设（v2 改为信号指标）

### 12.1 v0.1 发布前（W10-W12）

- **landing page**：`vibestation.<tld>`（Astro + 自建动效，对齐 anti-template policy）
- **Twitter/X 账号**：`@vibestation`，W10 开始每周一条开发进度
- **Discord**：开 `#announcements` / `#help` / `#feedback` / `#dev` 四频道
- **demo 视频**：3 分钟产品故事视频，YouTube + B 站同步

### 12.2 v0.1 发布日（W12）

**上午**（UTC+8）：
- Show HN: "Show HN: Vibestation – A multi-tab terminal with JetBrains-grade Git workbench"（**不用 Mission Control 字样**）
- r/rust、r/commandline、r/linux、r/macapps
- V2EX、即刻

**下午**：
- Product Hunt（需要准备好 product image、gallery、maker comment）
- Twitter 发布动图 demo

### 12.3 信号指标（v2 替换 v1 的 stars 数字）

> v1 的 "W14 1k stars / W25 2k stars" 是拍脑袋数字。v2 改为定性信号：

**v0.1 发布 72 小时内（理想场景）**：
- Show HN 首页（front page）至少 4 小时
- r/rust 或 r/commandline 100+ upvotes
- 3 篇独立开发者博客提及（非付费）
- 10+ alpha 测试者主动提供反馈（issue / Discord / 邮件）

**v0.2（W17）**：
- GitHub Issues 有健康活跃度（单周 >3 issue 创建、triage 及时）
- 第一个外部 PR 合并（无论大小）
- 至少 1 个独立用户录制使用视频

**v0.3（W22）**：
- 月度 HN 二次曝光（如新 feature 帖）
- Discord 社区 ≥ 50 人
- 至少 1 家媒体报道（Console Weekly / This Week in Rust / 少数派 / Sspai）

**v1.0（W30）**：
- Sponsors 开通，首月 >5 人赞助（无论金额）
- ≥ 3 位 co-maintainer 候选（核心作者邀请 + 反向申请）

> **为什么不用数字 stars**：GitHub stars 受首日流量影响大、易被操纵、与产品价值关系弱。信号指标看的是"真实使用者 + 真实讨论"。

### 12.4 长期运营

- **双周 release + changelog 博客**：每次发版配一篇 blog
- **YouTube demo 视频**：每月 1 条新功能深挖
- **社区贡献激励**：首 100 个有效 PR 贡献者上 README 头像墙
- **赞助通道**：v0.3 后开 GitHub Sponsors（不强求，不放广告）
- **v1.0 发布后**：考虑投稿 Console、JetBrains 社区媒介、Rust Weekly、This Week in Rust

---

## 13. 安全边界设计（v2 新增）

Vibestation 允许执行任意 shell 命令（终端本身就是），但 TaskRunner、AI 回填 prompt、自定义 layout 执行的命令需要安全边界。本章节定义最小权限模型。

### 13.1 威胁模型

| 威胁 | 场景 | 严重度 |
|------|------|--------|
| 恶意 TaskRunner 命令注入 | Smart Layout TOML 包含 `rm -rf $HOME` 类命令 | 高 |
| AI 回填 prompt 注入（v1.0）| Claude 输出被 attacker 控制（如 git log 中的 commit message），解析后自动执行 | 高 |
| 配置文件污染 | 用户导入的 Ghostty 配置包含恶意 `env` | 中 |
| 钓鱼 workspace 路径 | 用户打开第三方项目，里面 `.git/hooks` 被触发 | 中 |
| 剪贴板嗅探 | OSC52 剪贴板写入被恶意程序监听 | 低 |

### 13.2 原则

1. **最小权限**：Vibestation 自身不开 sudo、不写系统目录（`/etc`、`/usr`、`/System`）；所有持久化限定在 `~/.config/vibestation/` 和 `~/.cache/vibestation/`
2. **二次确认**：所有破坏性操作（TaskRunner 首次运行非白名单命令、git reset --hard、git push --force、layout 导入外部 TOML）必须弹确认框，用户明示同意
3. **白名单优先**：TaskRunner 内置常见命令白名单（下表），白名单内无需确认
4. **不信任 AI 输出**：v1.0 AI 回填 prompt 必须 sanitize（shell 特殊字符 escape、路径规范化、拒绝绝对路径到系统目录）

### 13.3 TaskRunner 命令白名单

首批白名单（`crates/vibestation-core/src/security/allowlist.rs`）：

| 命令前缀 | 说明 |
|---------|------|
| `cargo {build,test,check,clippy,watch,run,fmt}` | Rust 工具链 |
| `npm {install,run,test,build,ci}` / `pnpm *` / `yarn *` | JS 包管理 |
| `pytest` / `python -m pytest` / `uv run *` / `poetry run *` | Python 测试/执行 |
| `go {build,test,run,vet,mod}` | Go 工具链 |
| `make {,build,test,clean,install}` | make 常用目标（带参数） |
| `git {status,log,diff,branch,add,commit}` | Git 只读 + 基础写 |
| `docker {compose,ps,logs}` | Docker 常用只读 |
| `node` / `python` / `ruby` / `deno` | 脚本解释器（无 -c 参数注入） |

不在白名单的命令 → **首次运行弹确认框**（"Run `xxx`? Not in allowlist"），用户勾选"为此 workspace 记住"后下次不再提示。

### 13.4 AI 回填 prompt sanitization（v1.0）

```rust
pub fn sanitize_ai_prompt(raw: &str) -> String {
    // 1. 去除 ANSI 控制序列
    // 2. 限长 8K 字符（防止 prompt 爆炸）
    // 3. shell meta 字符 escape（如果准备作为命令参数）
    // 4. 拒绝包含 `\x00` / `\x1b]52` (OSC52) 等异常控制
    // 5. 路径规范化后，拒绝绝对路径指向 /etc /System /usr /bin /sbin
}
```

v1.0 实现前必须经过一轮外部安全审计（W29）。

### 13.5 敏感操作清单

以下操作必须二次确认（即使白名单）：

- `git reset --hard` / `git push --force` / `git clean -fd`
- 删除 workspace（含或不含 `.git`）
- 从外部 TOML 导入 layout
- 导入 Ghostty / iTerm2 配置文件（首次）
- TaskRunner 执行非白名单命令（首次）
- AI 回填 prompt 自动执行（v1.0，默认关闭）

### 13.6 审计 trail

所有"敏感操作" 记录到 `~/.cache/vibestation/audit.log`（JSONL），保留 90 天。用户可随时查看/导出/删除。字段：`{timestamp, action, workspace, details, result}`。

---

## 14. 分发运营（v2 新增）

### 14.1 macOS notarization 流水线

#### 14.1.1 前置条件（v0.1 发布前 2 周完成）

- [ ] Apple Developer Program 账号（$99/年，审核 1-2 周）
- [ ] Developer ID Application 证书（用于 notarization）
- [ ] App-specific password（用于 notarytool）
- [ ] Entitlements.plist 配置：
  ```xml
  <key>com.apple.security.cs.allow-jit</key>           <true/>
  <key>com.apple.security.cs.allow-unsigned-executable-memory</key> <true/>
  <key>com.apple.security.inherit</key>                <true/>
  <key>com.apple.security.cs.disable-library-validation</key> <true/>  <!-- fix-path-env 需要 -->
  ```

#### 14.1.2 CI 流水线（`.github/workflows/notarize.yml`）

```yaml
# 伪代码
jobs:
  notarize:
    needs: build-macos
    steps:
      - Import Developer ID certificate (secrets)
      - codesign --deep --force --options runtime --entitlements Entitlements.plist Vibestation.app
      - Create dmg (create-dmg)
      - codesign --deep --force --options runtime Vibestation.dmg
      - xcrun notarytool submit Vibestation.dmg --apple-id --password --team-id --wait
      - xcrun stapler staple Vibestation.dmg
      - Verify: spctl -a -t open --context context:primary-signature -vv Vibestation.dmg
```

#### 14.1.3 公证周期

- 正常：5-30 分钟
- 高峰：最长 24 小时
- **Release 流程上游要留 buffer**：v0.1 发布前 2 天就要 notarize，不能发布当天才开始

### 14.2 Linux 分发策略

| 格式 | 优先级 | 适用 | 投入 |
|------|--------|------|------|
| **AppImage** | 优先 | Ubuntu 22/24、Fedora、Arch | v0.1 必做 |
| **.tar.xz** | 次之 | 手动安装、Docker 内 | v0.1 必做 |
| **.deb** | 次之 | Debian / Ubuntu 官方风格 | v0.2 |
| **Flatpak** | 社区驱动 | GNOME/KDE 软件中心 | v0.3+（欢迎 PR）|
| **Snap** | 社区驱动 | Ubuntu Store | v0.3+（欢迎 PR）|
| **AUR (Arch)** | 社区驱动 | Arch 用户 | v0.2+（欢迎 PR）|

#### 14.2.1 AppImage 签名与验证

- 使用 `appimagetool` + GPG 签名 AppImage
- 公钥发布到 `vibestation.<tld>/.well-known/gpg-key.asc` 和 GitHub Release
- 每个 release 附 `sha256sums.txt` + `sha256sums.txt.asc`（detached signature）
- `scripts/verify-appimage.sh` 示例脚本供用户校验

### 14.3 auto-updater 策略

Tauri 内置 updater，Electron fallback 用 `electron-updater`。共同要求：

1. **签名验证**：下载的 update 包必须通过公钥签名校验，失败 fail-closed
2. **Staged rollout**：
   - Day 0：10% 用户
   - Day 2：50% 用户（若 crash 率无显著上升）
   - Day 4：100% 用户
3. **回滚机制**：
   - 本地保留前一版本的二进制（`~/.cache/vibestation/previous/`）
   - 用户可通过 `vibestation --rollback` 切回
   - auto-updater 检测到连续 3 次启动失败 → 自动回滚
4. **用户控制**：
   - 默认行为：通知 + 用户确认后下载
   - 可选：`auto_update: true`（安静更新，下次启动生效）
   - 可选：`auto_update: false`（完全手动）

### 14.4 telemetry / crash 合规

- **默认值**：`telemetry_enabled = false`
- **首次启动**：弹窗明示询问（不是预选）
- **收集内容**（仅启用后）：
  - 版本号
  - 平台 / OS 版本
  - 匿名 crash stack（去除文件路径中的用户名）
  - **不收集**：命令行内容、git repo 路径、file path、IP 地址
- **存储**：用户自行持有（本地文件）+ 若启用上报，走 Sentry 或自建端点
- **隐私政策**：`vibestation.<tld>/privacy`，明确 GDPR / CCPA 合规
- **用户权利**：随时导出、删除、撤回同意；`vibestation telemetry --export` / `--purge`

### 14.5 发行 checklist

每次 release 前依次确认：

- [ ] CHANGELOG 更新并人审
- [ ] §8.3 手动 QA 清单全过
- [ ] §10.6 终端正确性矩阵全过（v0.1 必需）
- [ ] macOS dmg 通过 notarization（`stapler validate` 确认）
- [ ] Linux AppImage GPG 签名 + sha256 校验
- [ ] auto-updater 在 staged rollout 模式
- [ ] 版本 tag + GitHub Release + 官网发行说明同步
- [ ] Twitter / Discord / Blog 发布日协同

---

## 附录 A：Spike 周详细计划（W0，v2 扩展）

v2 由 5 天扩展为 **5-6 天**，新增 redb vs rusqlite benchmark、Tauri Pass/Fail 更细判据。

| Day | 任务 | 产出 | 通过/失败后续 |
|-----|------|------|---------------|
| D1 | **Tauri 2 在 mac + Ubuntu Wayland + Ubuntu X11 启动** | 空壳三平台启动、冷启动 < 2s / 3s / 3s、IME 初测 | 通过 → D2 继续 Tauri；**失败 → D2 切 Electron 28+ spike** |
| D2 | **Tauri 硬通过矩阵** + **Electron fallback 验证（若 D1 失败）** | §3.1.1 判据表填写完毕；锁定 Tauri 或 Electron；写入 ADR | 选定框架 → D3 |
| D3 | **git2 读 commit log + gix 对比 benchmark** | linux kernel 仓库（100 万 commit）log benchmark 数据表；决定是否引入 gix 做读路径；写入 ADR | git2 足够 → MVP 用纯 git2；gix 显著更快 → 引入 gix |
| D4 | **redb vs rusqlite benchmark** + **git2 写 commit 打通** | 10 workspace × 100 profile × 10k 快照读写 P99 对比；ADR 锁定持久化；本地 git commit 成功 | 选定持久化 → D5 |
| D5 | **portable-pty 单读 + mpsc + xterm 5.5**；多 Tab 容器 | 单 Tab `yes`/`htop` 10 秒不卡；4 Tab 并存；PTY benchmark 吞吐 | 通过 → D6 |
| D6 | **实机跑 Claude CLI + Codex CLI**；macOS Developer 账号申请启动 | Claude CLI / Codex CLI 输出样本录制；协议初探；Apple Dev Program 申请提交 | - |

### Spike 验收表（W1 首日交付）

| 风险点 | 结论（通过/有坑/需换方案）| 证据 |
|--------|---------------------------|------|
| Tauri 2 on Wayland | | 截图 + 启动耗时 + IME 视频 |
| Tauri 2 on macOS | | 同上 |
| Electron 28+ fallback（若触发）| | 构建产物 + 冷启动数据 |
| PTY 多 Tab 吞吐 | | benchmark 数据 |
| git2 大仓库 log | | benchmark 数据 |
| gix vs git2（若评估）| | benchmark 数据 |
| redb vs rusqlite | | benchmark 数据 |
| Claude CLI 在 PTY 里 | | 录屏 + 输出样本 |
| macOS PATH 空问题 | | fix-path-env 验证结果 |
| Apple Developer Program | | 申请提交日期、预计审核完成日期 |

---

**文档完**

本文档（v2）为 Vibestation 项目从 Spike（2026-04-20 周）到 v1.0 GA（2026-11 下旬）的完整实施指南，共 14 章 + 附录，预计总投入 **28-30 周 × 20-25 小时 ≈ 600-750 小时**（带 20% buffer）。所有技术决策可追溯到 `terminal-git-workbench-tech-research.md` 调研报告、`vibestation-codex-review-and-response.md` 评审与应对，或本文第 9 节风险登记册。

