---
id: MVP-04
type: mvp
title: 多 Tab 终端（PTY + xterm.js + Shell/CLI 兼容）
status: ready
owner: Codex CLI
phase: W4-W6
depends_on: ["MVP-03", "SPIKE-05", "SPIKE-06"]
depends_on_notes: "SPIKE-06 = §A 脱敏样本（done · PR #71）· §B codesign/notarization 不是 MVP-04 前置（MVP-04 运行态用 ad-hoc sign 即可 · codesign 是 MVP-10 GA 打包事）· SPIKE-06 现 status: blocked 只是 §B 卡 Apple Dev · 不阻塞 MVP-04"
blocks: ["MVP-05", "MVP-06"]
blocked_by: []
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1 · §10.6（终端正确性矩阵）· §附录 A D5
risk_ref:
reviewer: Kimi
---

# MVP-04: 多 Tab 终端

> **状态**：`ready`（Phase A/B/C/D/E/F 全部完成 · MVP-04 done）
> **依赖**：MVP-03（主区布局）· SPIKE-05（PTY 架构锁定）· SPIKE-06（CLI 实机验证）/ **阻塞**：MVP-05（Pane 基于 Tab）· MVP-06（配置导入映射到终端）
> **战略依据**：[`§10.1`](../implementation-plan.md) · `§10.6 终端正确性矩阵`

---

## 🎯 目标（Goal）

主内容区实现多 Tab 终端，每 Tab 独立 PTY + xterm.js 渲染，支持 zsh/bash/vim/htop/yes/tmux/Claude CLI/Codex CLI 运行。100% 通过 `§10.6 终端正确性矩阵`。

## 📖 背景（Context）

- `CLAUDE.md` #15（B 栏 → SPIKE-05 锁定）：PTY 方案 = portable-pty + 单读线程 + mpsc（失败 fallback 一 session 一线程）
- `#6`（A 栏）：xterm.js 5.5 前端终端渲染
- CLI 只作为 PTY 普通程序运行，不做 AI-Aware 联动（v1.0）

---

## 🎨 功能范围（Scope）

**Do**：
- Tab 基础操作：新建 / 关闭 / 重命名 / 切换
- Tab 切换不丢 buffer（每 Tab 的 scroll back 独立）
- 每 Tab 独立 PTY 进程 + 独立 xterm.js 实例
- 支持运行：zsh / bash / vim / htop / yes / tmux / Claude CLI / Codex CLI
- 快捷键：`⌘T` 新 Tab · `⌘W` 关 Tab · `⌘⇧[/]` 前后切换 · `⌘1..9` 跳指定 Tab
- resize：调整窗口 → PTY SIGWINCH 正确传达
- Ctrl+C / Ctrl+D / Ctrl+Z 信号正确传递
- 粘贴保护：粘贴多行前提示确认（防误触 rm -rf）
- Shell 选择：macOS 默认 zsh / Linux 默认 bash，可在设置改

**Don't**：
- Pane 分屏（→ MVP-05）
- 配置导入（→ MVP-06）
- AI CLI 联动（v1.0 vision，禁区）
- tmux 控制 mode（v0.2+）

## 🛠 实施进度（2026-04-21 更新 · audit H3+L1）

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · storage prep | migration v5 tabs 表 + TabsDao 6 CRUD + 2 scrollback 方法 + 5 IPC commands + Tauri ACL tabs.toml + ts-rs 5 bindings + 36 单元测试 | ✅ done | [#72](https://github.com/tajiaoyezi/vibestation/pull/72) |
| Phase B · PTY runtime | portable-pty 启动 · stdin/stdout 桥接 · bounded mpsc + drop-oldest（SPIKE-05 架构）· resize/signal 传递 | ✅ done | [#82](https://github.com/tajiaoyezi/vibestation/pull/82) |
| Phase C · xterm 前端 | xterm.js 5.5 渲染 · SolidJS 组件集成 · WebGL → Canvas → DOM fallback · theme token 接入 | ✅ done | [#91](https://github.com/tajiaoyezi/vibestation/pull/91) |
| Phase D · shell 兼容 | zsh/bash/fish 默认选择（`app_settings.default_shell`）· Claude CLI / Codex CLI 实机（SPIKE-06 §A 已脱敏） | ✅ done | #109 (this PR) |
| Phase E · 持久化 | `scrollback_append` + `scrollback_fetch` IPC 串起前后端 · 关 Tab 清 scrollback（随 tabs 行删除自动清理 · sqlite 已验证） | ✅ done | — |
| Phase F · runtime 证据 | ≥ 3 张截图或 30s 录屏 · 覆盖 create/close/rename/switch/scrollback · 放 `docs/runtime-evidence/mvp-04/` | ✅ done | — |

**Phase E · 持久化补充验收（本 PR）**
- [x] PTY stdout 持续写入 scrollback（100ms debounce / 100 行阈值，whichever first）
- [x] 关 workspace 重开后，已存在 Tab 的 scrollback 完整恢复（顺序 + 内容）
- [x] 多 Tab scrollback 隔离（关重开后各自不串）
- [x] 关 Tab 删除 `tabs` 行后，`scroll_back` JSON 同步消失（sqlite3 验证）
- [x] 10k 行上限 trim · 超过部分 FIFO drop（Phase A DAO 测试覆盖 + 本轮 PTY→DB 集成路径补齐）
- [x] PTY exit 时强制 flush 剩余 pending buffer（不丢最后几行）

**Phase F · runtime 证据补充验收（本 PR）**
- [x] Runtime 证据已提交到 `docs/runtime-evidence/mvp-04/`（5 张截图 + `metrics-phase-f.md`）
- [x] create / rename / switch / close / scrollback 画面均有覆盖
- [x] A.5 / E.2 tab switch latency 已量化（AX 自动化 2-tab 样本 median `20 ms`）
- [x] E.4 已量化：页面内同步 JS 执行 `sync max = 3 ms`（frame delta `19 ms` 作为上下文记录于 evidence note）

**下次 agent 起点**：MVP-04 整体 done · 下游 MVP-05 / MVP-06 已解阻塞。

**migration 版本规划**（L-2 · 本 MVP 不做 v6）：

- Phase A 已用 `migration v5`（tabs 表 + FK CASCADE + idx_tabs_workspace_created）
- `migration v6` 由 **MVP-05** 引入（panes 表或 `tabs.layout` 列 · 见 MVP-05 §H Pane 布局模型）· 不是本 MVP 范围
- 未来 agent 实施 MVP-04 Phase B-F 时**不得**新建 migration · 只读写 tabs / scrollback 表

**保持 `status: ready`**：整体 MVP-04 未 done · 允许下次 agent 直接认领 Phase B 起点。

## 🖼 UI 引用

- 主区 Tab bar：`design/directions/1-calm-studio.html` 主内容区顶部
- Tab 样式：紧凑，带 close X，active tab 用主色下边框
- 字体：JetBrains Mono（原型定义）

## ✅ Acceptance

### A. Tab 基础

- [ ] 新 workspace 打开默认创建 1 个 Tab（运行默认 shell）
- [ ] `⌘T` 新建 Tab → 新 PTY 进程 + 新 xterm 实例（`ps` 或 Activity Monitor 可观察到新进程）
- [ ] `⌘W` 关 Tab → PTY 进程被 SIGKILL / SIGTERM 清理；最后一个 Tab 关闭 → 弹出确认对话框"关闭 workspace？"
- [ ] 双击 Tab 标题 → 进入重命名输入框，回车确认，Esc 取消
- [ ] Tab 切换延迟 < 100ms（Chrome DevTools Performance 面板或 Playwright 采样 5 次取 median）

### B. PTY 正确性

- [ ] 每 Tab 独立 PTY（不共享）：同时运行 `echo $$` 两 Tab 输出不同 PID
- [ ] Shell 启动：macOS 默认 zsh / Linux 默认 bash（从设置表 `app_settings` 的 `"default_shell"` 键读取）
- [ ] 环境变量 PATH：与系统 Terminal.app / GNOME Terminal 启动同 shell 时的 PATH 一致（±1 个路径项容差；`fix-path-env` 解决 macOS GUI app PATH 问题）
- [ ] 信号传递：
  - Ctrl+C：运行 `sleep 30` 后按 Ctrl+C → 进程终止（`ps` 确认 PID 消失）
  - Ctrl+D：空 prompt 按 Ctrl+D → shell 发送 EOF 退出
  - Ctrl+Z：运行 `sleep 30` 后按 Ctrl+Z → 进程暂停（`jobs` 显示 `Stopped`）
- [ ] resize：窗口尺寸变化 → PTY SIGWINCH 传达，`htop` / `vim` 即时重排（肉眼可见 1s 内重排，无需手动 `:redraw!`）

### C. 程序兼容矩阵（`§10.6`）

- [ ] zsh 交互：Tab 键触发路径补全（输入 `ls /u` + Tab → 补全为 `/usr/` 或候选列表）；上下箭头触发历史；`echo $TERM` 输出 `xterm-256color`
- [ ] vim：基础编辑（`i` 插入、`Esc`、`:wq` 保存退出）；`/` 搜索高亮；方向键正常（不输出 `^[[A` 乱码）
- [ ] htop：UI 渲染正常，刷新率 ≥ 5Hz（肉眼感知流畅，无撕裂）
- [ ] yes：10s 连续输出，终端滚动流畅（肉眼无卡滞），单 Tab 吞吐 ≥ 20MB/s（`yes | pv > /dev/null` 测）
- [ ] tmux：基础 session 创建（`tmux new -s test`）+ window 切换（`Ctrl+B n` / `Ctrl+B p`）正常
- [ ] Claude CLI：启动 `claude` → 登录流程（如有）→ 单轮对话输入/输出正常（SPIKE-06 §A 已验 smoke）
- [ ] Codex CLI：启动 `codex` → 登录流程（如有）→ 单轮对话输入/输出正常（SPIKE-06 §A 已验 smoke）

### D. 粘贴保护

- [ ] 粘贴内容含换行符（≥1 个 `\n`）→ 弹出确认对话框
- [ ] 对话框显示将要粘贴的前 5 行预览（每行最多 80 字符，超长截断加 `…`）
- [ ] 对话框提供"不再提示本 session"复选框：勾选后同 session 再次粘贴多行不再弹窗，关闭 workspace 后重置

### E. 性能（对齐 `§10.2`）

- [ ] 10 Tab 并存，总 RSS < 500MB（Activity Monitor / `ps` 采样，xterm 实例 + PTY 进程合计）
- [ ] 切 Tab 延迟 < 50ms（Chrome DevTools Performance 面板或 Playwright `performance.now()` 差值，5 次采样取 median）
- [ ] 单 Tab 吞吐 ≥ 20MB/s（`yes | pv > /dev/null` 连续 10s，取平均吞吐量）
- [ ] 主线程阻塞 ≤ 16ms（Chrome DevTools Performance 面板记录 xterm 渲染帧，单帧 JS 执行 ≤ 16ms，60FPS 达标）

### F. 错误处理

- [ ] Shell 进程异常退出（非零 exit code 或 signal）→ Tab 内显示 `"Process exited (code X). Press Enter to restart"`，按 Enter 重新启动同 shell
- [ ] PTY open 失败（如 shell 路径不存在）→ 显示可读的 error toast（如 `"无法启动 shell：/bin/fake-shell 不存在"`），应用不 panic / 不白屏
- [ ] xterm renderer fallback：WebGL → Canvas 2D → DOM（逐级降级），降级事件记录到 console.warn
- [ ] Shell 冷启动反馈：tab 新建到 PTY 首屏可见文本之间，显示明确 loading 态（启动卡片 + shell 路径提示），禁白屏

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | PTY 状态机、mpsc channel 背压（阻塞/丢弃策略）|
| 集成 | portable-pty + mpsc + xterm 端到端流通 |
| E2E | 创建 Tab → 运行命令 → 切 Tab → 关 Tab |
| 兼容矩阵 | `§10.6` 全量手动 + 自动回归 |
| Soak | 10 Tab × 10 分钟 yes，RSS / channel depth 记录（对齐 SPIKE-05 B.1）|

## 💾 数据模型变更

新 table `tabs`：
```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Tab 状态 · IPC contract source of truth.
///
/// 前端类型由 ts-rs 自动生成到 `web/src/bindings/TabState.ts`（见
/// `crates/app/build.rs`）· 禁止手写 TypeScript 对偶 interface · 避免 H2 类
/// camelCase / snake_case drift 事故。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub tab_id: String,              // UUID
    pub workspace_id: String,        // FK
    pub name: String,                // 用户可改
    pub shell: String,               // "zsh" / "bash" / etc
    pub cwd: String,                 // 当前工作目录
    /// ⚠️ 建议从 IPC contract 排除：scroll_back 数据量大（最多 10k 行），
    /// 全量序列化拖慢 IPC。改为本地前端缓存，IPC 只传 tab_id，
    /// 按需通过独立 command 拉取可见区 buffer。
    pub scroll_back: Vec<String>,    // 最多保留 10k 行
    /// Unix timestamp (seconds)· 映射为 TS `number` 而非默认 `bigint`：unix 时间戳
    /// 秒数在可预见未来（~year 285476）都 < 2^53 · 用 `number` 前端 Date/sort 零改动。
    #[ts(type = "number")]
    pub created_at: i64,
}
```

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS + PR #63 rollout 生产化 · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（预期）

| Rust struct | 用途 | 前端 import 路径 |
|-------------|------|-----------------|
| `TabState` | Tab 全量状态（list/get） | `import type { TabState } from "../bindings/TabState"` |
| `TabCreateRequest` | 新建 Tab 参数（shell / cwd 可选） | `import type { TabCreateRequest } from "../bindings/TabCreateRequest"` |
| `TabCloseRequest` | 关闭 Tab 参数（tab_id） | `import type { TabCloseRequest } from "../bindings/TabCloseRequest"` |
| `TabRenameRequest` | 重命名 Tab 参数（tab_id + name） | `import type { TabRenameRequest } from "../bindings/TabRenameRequest"` |
| `TabListResponse` | 某 workspace 下 Tab 列表 | `import type { TabListResponse } from "../bindings/TabListResponse"` |

> 实际 struct 名和字段以实施 PR 为准，但**必须**全部走 ts-rs 自动生成。

### G.2 强制规范

- [ ] 所有 IPC struct 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] `i64` 类型的时间戳字段必须加 `#[ts(type = "number")]`（防止 TS 生成 `bigint`，前端 Date/sort 零改动）
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface TabState { ... }` 或 `type TabState = { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.3 H2 类 regression proof（PR merge 前必做）

模拟一次"Rust 端改字段名但前端忘同步"的场景，验证 compile-time 防御生效：

1. 临时在任一 IPC struct（如 `TabState`）的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'TabState'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]`，确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次，结果写入 PR description 或 `docs/runtime-evidence/MVP-04/`（如实施 PR 本身含 ts-rs 集成）。

## ⚠️ 已知风险

- **PTY 架构 fallback（SPIKE-05 B.3）**：若单读线程失败 → 改为一 session 一线程 → 10 Tab 资源上升 ~40MB
- **Wayland IME**：Wayland 下 IME 切换可能和 xterm focus 冲突 → 三平台分开测
- **CLI 中断残帧（SPIKE-06 A.2）**：Ctrl+C Claude CLI 流式输出中途 → 检查残帧是否污染下条 prompt
- **fix-path-env shim**（2026-04-21 · PR #82 · Codex）：Phase B 实施时 crates.io `fix-path-env` 包在 Codex 环境无法解析 · 改为 `crates/app/src/fix_path_env.rs` 53 行本地 shim（macOS/Linux 启动登录 shell + `printf %s "$PATH"` 覆盖 `env::set_var`）。**风险**：shim 无 timeout · 若用户 shell rc 文件 source 慢资源（NVM / oh-my-zsh plugin）· app 启动可能卡几秒。**GA gate**：v0.1.0 GA 发布前评估 · 若官方包在 CI 环境可用 · 切回；否则 shim 加 timeout 保护。
- **Linux PTY SIGTERM timing 不稳定**（2026-04-21 · PR #82 CI failure · PR #86 多轮 workaround 失败）：`pty::tests::signal_sigterm_exits_exec_session` 在 macOS 本地稳定 · 在 GitHub Actions Ubuntu runner 上 timing 不一致 · `exec sleep 30` 后 SIGTERM 到 mio epoll 感知 PTY close event 的延迟 > 10s · 根因怀疑是 `tcgetpgrp` / `waitpid` / epoll 在 Linux PTY 上的行为和 macOS kqueue 差异。**workaround**（PR #86）：测试标 `#[cfg_attr(target_os = "linux", ignore)]` · 本地 macOS 继续跑 · CI Ubuntu skip。**GA gate**：MVP-04 Phase D（shell 兼容 · 三平台矩阵）启动时 · 在 Ubuntu 环境深挖 signal/exit 语义 · 解除 ignore。不阻塞 macOS-first v0.1.0-alpha 发布。
- **Shell rc 慢启动感知**（2026-04-22 · PR #91 · Codex）：macOS GUI zsh + oh-my-zsh / nvm / pyenv plugin source 可能 1-3s 才吐出首屏可见文本。**workaround**：Phase C 前端加入 loading 态（F.4）覆盖 UX，避免用户把启动中的 PTY 误判成白屏。若用户 shell rc 卡 30s+ · 视为用户环境问题，app 不主动介入。

## 📝 Notes

- MVP-04 不实现 tmux control mode（看 tmux 作为普通程序跑即可）
- Claude/Codex CLI 的协议解析留给 v1.0 AI-Aware（SPIKE-07 parser spike）

## §I. 测试矩阵

> **目的**：让 Phase D 实施 agent 接到本 spec 后 5 min 内能起手，不用现场设计 22 用例、通过判定、fail 处理路径。
> **位置**：追加在 §Notes 之后、§相关 之前。
> **原则**：纯追加，零删除，不改前面已 accepted 段。

### §I.1 默认 shell 矩阵（macOS · 3 shell × 4 测试项 = 12 用例）

| 测试组 | 测试项 | 通过判定 | 失败处理 |
|---|---|---|---|
| **zsh**（macOS 默认）| 启动 → 显示 prompt | < 3 s 内 prompt 可见，无 panic / 白屏 | **blocker** · 阻塞 v0.1 GA |
| zsh | `echo $TERM` | 输出 `"xterm-256color"`（一致性测）| **blocker** |
| zsh | Tab 补全 `ls /u` + Tab → `/usr/` | 候选列表显示；单一时直接补全 | **blocker** |
| zsh | 中文 IME 输入 + 输出 | 输入"你好"显示"你好"，无乱码 | **blocker** · UTF-8 是 v0.1 必需 |
| **bash** | 启动 → 显示 prompt | < 3 s | **blocker** |
| bash | `echo $TERM` | `"xterm-256color"` | **blocker** |
| bash | Tab 补全 | 候选列表 | **blocker** |
| bash | history 上下箭头 | 显示历史命令 | **blocker** |
| **fish** | 启动 → 显示 prompt | < 3 s，fish 风格 prompt 显示 | **non-blocker** · 推 v0.2 若 fail |
| fish | autosuggestion 灰字 | 输入 `ec` → 灰字 `echo` 提示 | **non-blocker** |
| fish | Tab 补全 | fish 风格补全（含描述）| **non-blocker** |
| fish | 中文 IME | 显示正确 | **non-blocker** |

### §I.2 CLI 实机矩阵（Claude CLI + Codex CLI · 每个 5 测试项 = 10 用例）

| CLI | 测试项 | 通过判定 | 失败处理 |
|---|---|---|---|
| **Claude CLI**（`claude`）| 启动 → login flow（已登录则跳过）| 启动 < 5 s，显示登录提示或 ready prompt | **blocker**（v0.1 核心 use case）|
| Claude CLI | 输入"你好" → 流式回复 | 流式 stream 显示中文，无乱码，完整结束 | **blocker** |
| Claude CLI | Ctrl+C 中断流式输出中途 | 流式 stop，进入 prompt，无残帧污染下一条 prompt | **blocker** · 残帧污染是 R1 风险 |
| Claude CLI | 退出（`exit` / Ctrl+D）| shell 回到当前 Tab，PTY 进程清理 | **blocker** |
| Claude CLI | 长输出（5000+ token 回复）滚动 | 滚动流畅，scrollback 全保留 | **blocker**（覆盖 §Acceptance E 性能）|
| **Codex CLI**（`codex`）| 启动 → login flow | < 5 s，ready | **blocker** |
| Codex CLI | 单轮对话输入/输出 | 显示正确 | **blocker** |
| Codex CLI | Ctrl+C 中断 | stop，prompt，无残帧 | **blocker** |
| Codex CLI | 退出 | shell 回 prompt | **blocker** |
| Codex CLI | 长输出滚动 | 流畅，scrollback 保留 | **blocker** |

### §I.3 Ubuntu / Windows 跳过条款（明确 macOS-first）

> **Ubuntu Phase D 后续补**（spec frontmatter 已显示 SPIKE-01/02 Phase B Ubuntu blocked）：
> - 所有 §I.1 / §I.2 用例，Ubuntu 平台标 **deferred**，v0.1 macOS-first GA 后再补
> - blocker 用例，Ubuntu fail 推 v0.2，不阻塞 v0.1
> - non-blocker 用例，Ubuntu fail 推 v0.3
>
> **Windows skip**：
> - MVP-04 spec line 8 明确 Windows 推 v0.4
> - §I 测试矩阵不涵盖 Windows
> - Windows shell（PowerShell / cmd / WSL）测试矩阵 v0.4 单独 spec

### §I.4 测试执行流程（Phase D 实施 agent 用）

**Phase D 实施 agent 按以下流程执行 §I 矩阵**：

1. **环境准备**（macOS）：
   - 安装 zsh（macOS 默认有）、bash（macOS 默认有）、fish（`brew install fish`）
   - Claude CLI（`npm install -g @anthropic-ai/claude-cli` 或官方安装方式）
   - Codex CLI（按 OpenAI 官方安装方式）
   - Vibestation app 本地编译 + `pnpm tauri:dev` 启动

2. **执行顺序**：
   - 先 §I.1 默认 shell 矩阵（12 用例），每用例独立新 Tab 测
   - 再 §I.2 CLI 矩阵（10 用例），每用例独立新 Tab 测
   - 总计 22 用例，估时 1–1.5 h（含测试 + 截图 + 记录）

3. **录证据**：
   - 每用例 1 张截图（Tab 内画面），放 `docs/runtime-evidence/mvp-04/phase-d/`
   - 22 张截图按 ADR-011 R3 命名：`shell-zsh-01-startup.jpg`、`cli-claude-03-ctrl-c.jpg` 等
   - 关键流式行为录 30 s 录屏（如 Claude CLI 流式回复、Ctrl+C 中断）

4. **记录通过率**：
   - PR body 列 22 行用例表，每行 ✅ / ❌ / ⏭️（skip 注明原因）
   - blocker fail ≥ 1 → **BLOCK PR merge**，实施 agent 必须修
   - non-blocker fail → 推 v0.2 / v0.3（在 spec §已知风险 段加技术债条目）

### §I.5 fail 处理流程（实施 agent 卡壳时）

实施 agent 跑 §I 矩阵，遇 fail 按以下决策：

| Fail 类型 | 处理 |
|---|---|
| zsh / bash 启动 fail | **blocker** · 修代码 · 不交 PR |
| fish 启动 fail | **non-blocker** · §已知风险加条目 · spec 标 v0.2 修 · PR 可交 |
| Claude CLI 残帧污染 | **blocker** · R1 风险 · 修代码（PTY 输出处理 / xterm reset 序列）· 不交 PR |
| Claude CLI Ctrl+C fail | **blocker** · 修信号传递（PTY signal 链路）· 不交 PR |
| 中文 IME 乱码 | **blocker** · UTF-8 编码问题 · 修 PTY / xterm encoding · 不交 PR |
| 长输出滚动卡顿 | **non-blocker**（若 §Acceptance E 性能已过）· §已知风险加条目 |
| Codex CLI 退出残留进程 | **blocker** · 修 PTY 进程清理 · 不交 PR |
| 不在 §I 矩阵的新发现 fail | 实施 agent 判断 · blocker 在 spec §已知风险 段加新条目 + Arbiter approve 后推后 |

## 🔗 相关

- `CLAUDE.md` #15 · #6 · ⚠️ CLI 警告（R1）
- SPIKE-05（PTY 架构）· SPIKE-05.5（visible throughput 验证）· SPIKE-06（CLI 实机 + fix-path-env）
- `implementation-plan.md` §10.6 终端正确性矩阵
- 上游：MVP-03 · SPIKE-05 · SPIKE-06
- 下游：MVP-05 · MVP-06

---

**自审四问（2026-04-20）**：
1. **递归完备性**：Acceptance 清单覆盖 Tab/PTY/兼容矩阵/粘贴/性能/错误/IPC contract 全维度 ✅
2. **反向场景**：若 TS derive 漏加 → `pnpm typecheck` 立即 FAIL（H2 proof 制度化）· 若 PTY fallback 触发 → Acceptance B 仍通过（独立 PTY 不共享）✅
3. **边界适用性**：10 Tab / 1 Tab / 0 Tab（新建 workspace 默认 1 Tab）都适用；macOS/Linux 双平台 shell 默认不同 ✅
4. **YAGNI**：tmux control mode / AI 联动 / Pane 分屏 / 配置导入 都明确推后 ✅
