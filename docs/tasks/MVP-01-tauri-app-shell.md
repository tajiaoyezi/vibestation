---
id: MVP-01
type: mvp
title: Tauri 应用骨架 + 启动流程 + 基础崩溃恢复
status: ready
owner: Claude Code (Sonnet 4.6 · Phase A PR #28 · Phase B PR #33 · macOS 交付完成 · Phase C Ubuntu 待环境)
phase: W1-W2
depends_on: ["SPIKE-02"]
blocks: ["MVP-02", "MVP-03", "MVP-04", "MVP-10"]
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §3.1 · §3.2
risk_ref:
reviewer: User (Arbiter · GitHub PR approve)
---

# MVP-01: Tauri 应用骨架 + 启动流程

> **状态**：`ready`（spec PR `docs/spec-flip-spike-01-spike-02-mvp-01` · 走 docs/tasks/README.md 第 7 步 (b) 路径变种 · 分支保护暂缓 · 用户在 GitHub UI 正式 approve 后 merge）
> ⚠️ 实施前提：SPIKE-02 必须先 done（桌面框架已锁定 · 决定 Tauri 还是 Electron fallback）
> **依赖**：SPIKE-02（桌面框架已锁定）/ **阻塞**：MVP-02..04 · MVP-10
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) MVP B 折中方案 · §3.1 架构 · §3.2 Cargo workspace

---

## 🎯 目标（Goal）

建立 Tauri 2 + SolidJS + Cargo workspace（2 crate：`app` + `core`）基础应用骨架，包含启动流程、欢迎页、基础崩溃恢复、双平台打包配置。所有后续 MVP 功能都运行在本骨架上。

## 📖 背景（Context）

- SPIKE-02 已锁定 Tauri 2 作为桌面框架（若 Spike fail 则锁定 Electron 28+，本 spec 的具体实现技术对应选型）
- `implementation-plan.md §10.1` 明确 MVP 必须有"启动应用 + 欢迎页"+"崩溃恢复（基础）"
- Cargo workspace **2 crate**（`app` + `core`）是 `CLAUDE.md` 决策表 #5 锁定的 A 栏决策，v0.2 前不拆
- 前端栈 `SolidJS + TypeScript + xterm.js` 是决策表 #6 A 栏

---

## 🎨 功能范围（Scope）

**Do（MVP-01 范围内）**：
- Cargo workspace 骨架：`app`（Tauri 启动层） + `core`（业务逻辑，Rust 纯库）
- 前端项目：SolidJS + TypeScript + Vite（对齐 Tauri 2 默认 template）
- 应用启动：窗口显示 "Vibestation" 欢迎页
- 欢迎页内容：Logo + "Create first workspace" 按钮 + 版本号（从 `Cargo.toml` 读）
- 基础崩溃恢复：重启后恢复上次打开的 workspace 列表 + 每个 workspace 的 tab 状态
- 双平台打包配置：`tauri.conf.json` 配置 mac dmg + linux AppImage 构建目标
- 应用图标：使用 `design/logos/mark.svg`（Calm Studio 定稿版本）

**Don't（推到后续 MVP）**：
- Workspace 创建 / 项目识别逻辑（→ MVP-02）
- Tool Windows 布局 / Primary Sidebar（→ MVP-03）
- 终端 Tab（→ MVP-04）
- Telemetry opt-in 对话框（→ MVP-10）
- macOS notarization / Linux AppImage 签名（→ MVP-10）

## 🖼 UI 引用（UI Reference）

- 欢迎页：`design/directions/1-calm-studio.html` 第一屏（无 workspace 时的空状态）
- 应用图标：`design/logos/mark.svg`（主图标）+ `design/logos/wordmark-a.svg`（可选用于 `about` 对话框）
- 色彩 token：从 `design/directions/1-calm-studio.html` CSS 变量继承（`--color-bg` / `--color-fg` / etc.）

## ✅ Acceptance

> evaluator 按此逐项对照 diff + 手动 QA

### A. Workspace 骨架

- [ ] `Cargo.toml` workspace 定义 2 crate：`crates/app` + `crates/core`
- [ ] `crates/app/Cargo.toml` 依赖 `tauri` 2.x，入口 `main.rs`
- [ ] `crates/core/Cargo.toml` 为纯 Rust 库（无 Tauri 依赖），可独立 `cargo test`
- [ ] `cargo build --workspace` 成功
- [ ] `cargo test --workspace` 至少运行 1 个 dummy 单元测试成功

### B. 前端项目

- [ ] `web/` 目录下 SolidJS + TypeScript + Vite 项目
- [ ] `web/package.json` 依赖 `solid-js` + `typescript` + `vite-plugin-solid`
- [ ] `web/tsconfig.json` strict 模式开启
- [ ] `pnpm install && pnpm --filter web dev` 成功启动 dev server
- [ ] Prettier + ESLint 配置就位（对齐 `CLAUDE.md` 代码风格）

### C. Tauri 集成

- [ ] `src-tauri/tauri.conf.json` 配置完整（bundle targets、window size、icon）
- [ ] `pnpm tauri dev` 启动应用，欢迎页正常显示
- [ ] 窗口默认尺寸 1280×800，最小 800×600
- [ ] 应用图标在 mac Dock / linux taskbar 显示 `mark.svg`

### D. 欢迎页

- [ ] 应用首次启动显示 "Vibestation" 欢迎页
- [ ] 欢迎页显示：Logo（mark.svg）+ 标题 + 版本号 + "Create first workspace" 主 CTA
- [ ] 版本号从 `Cargo.toml` `[package] version` 读取（运行时通过 Tauri API）
- [ ] 欢迎页 CTA 点击后暂时 `console.log('MVP-02 will implement')`（MVP-02 接管）
- [ ] 欢迎页 a11y：所有交互元素有 aria-label，键盘可达（Tab/Enter）

### E. 基础崩溃恢复

- [ ] 应用退出时，通过 Tauri IPC 把"打开的 workspace 列表"写入本地存储（rusqlite · ADR-005 锁定）
- [ ] 应用启动时，读取存储的状态：
  - 无状态 → 显示欢迎页
  - 有状态 → 恢复上次打开的 workspace 列表（但 workspace 内容由 MVP-02 填充）
- [ ] 存储路径：macOS `~/Library/Application Support/Vibestation/state.db` · Linux `~/.config/vibestation/state.db`
- [ ] 手动测试：启动 → 模拟崩溃（kill -9）→ 重启，验证状态恢复

### F. 双平台打包

- [ ] `tauri.conf.json` bundle targets 包含 `dmg`（macOS）+ `appimage`（Linux）
- [ ] `pnpm tauri build` 在 macOS 成功产出 `*.dmg`
- [ ] `pnpm tauri build` 在 Ubuntu 24 成功产出 `*.AppImage`
- [ ] 产物大小：mac dmg < 30MB · Linux AppImage < 40MB（对齐 `implementation-plan.md §10.2`）
- [ ] 应用启动耗时：mac < 2s · Linux Wayland/X11 < 3s

### G. CI 必过项（`CLAUDE.md` 禁区）

- [ ] `cargo clippy --workspace -- -D warnings` 零警告
- [ ] `cargo fmt --all -- --check` 零格式差异
- [ ] `pnpm lint` 前端零错误
- [ ] `pnpm typecheck` 前端零错误

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元（Rust）| `core/` 逻辑（崩溃恢复 state 序列化）| `cargo test -p vibestation-core` |
| 集成（Rust + Tauri）| IPC + 本地存储读写 | `cargo test -p vibestation-app --features integration` |
| E2E（前端）| 启动 → 欢迎页渲染 → CTA 点击 | Playwright + Tauri webdriver |
| 手动 QA | 双平台打包 + 崩溃恢复 | `implementation-plan.md §8.3` QA 清单 |

**覆盖率目标**：`core/` crate **≥ 80%**（`CLAUDE.md` 全局规则）。`app/` 因含大量 Tauri IPC boilerplate 可放宽到 **≥ 60%**。

## 💾 数据模型变更

**首次建立 `state.db`（rusqlite · SPIKE-04 + SPIKE-04.5 全链路 accept · ADR-005 A 栏锁定）**：

- Table `workspaces`：
  - key: `workspace_id: String`（UUID v4）
  - value: `WorkspaceMetadata { path: String, name: String, last_opened: i64 }`
- Table `app_state`：
  - key: `"last_opened_workspaces"`
  - value: `Vec<workspace_id>`（Workspace 顺序）

`schema_version = 1`（对应 SPIKE-04 §B.3 migration 要求）。

## ⚠️ 已知风险

- **SPIKE-02 fallback**：若 Spike 决定切 Electron 28+，本 spec 的 Tauri API 调用需改为 Electron API，部分实现细节要重写（但整体架构不变）
- **macOS PATH 空问题**（SPIKE-06 已知风险）：启动子进程时需要 `fix-path-env` crate 或等价方案
- **崩溃恢复的边缘情况**：状态文件损坏时需 fallback 到欢迎页（不能 crash）

## 📝 Notes / 讨论

- Cargo workspace 的 2-crate 划分：`app` 只放 Tauri 启动 + IPC adapter；`core` 放所有业务逻辑（workspace 管理、git、PTY 都在 core），便于独立测试
- 欢迎页设计故意简洁：MVP-01 只做"最小可启动"，装饰性动效留到 Phase 3 完成 Landing page（Astro）时补进
- Tauri 2 启用 `updater` plugin 配置但不实装 update 服务端 —— MVP-10 打包阶段再接

## 🔗 相关

- 对应 `CLAUDE.md` 决策表：**#5 Cargo workspace 2 crate**（A 栏）· **#6 前端栈 SolidJS**（A 栏）· **#12 桌面框架**（B 栏 → SPIKE-02 锁定）
- `implementation-plan.md` 章节：§10.1 MVP 范围 · §3.1 架构 · §3.2 Cargo workspace · §10.2 性能
- 上游：SPIKE-02（Tauri 硬通过矩阵）
- 下游：MVP-02..04 · MVP-10

---

**填写完毕后自审**（CLAUDE.md "📝 写规则/清单前的自审四问"）：

1. **递归完备性**：7 类 Acceptance（骨架/前端/Tauri/欢迎页/恢复/打包/CI）覆盖 ✅
2. **反向场景**：失败 → 无法作为后续 MVP 的载体；崩溃恢复失败 → 用户状态丢失 ✅
3. **边界适用性**：三平台都在 Acceptance 里显式要求验证 ✅
4. **YAGNI**：Telemetry / notarization / workspace 创建逻辑都推后，不在本 spec 里做 ✅
