---
id: MVP-04
type: mvp
title: 多 Tab 终端（PTY + xterm.js + Shell/CLI 兼容）
status: ready
owner:
phase: W4-W6
depends_on: ["MVP-03", "SPIKE-05", "SPIKE-06"]
blocks: ["MVP-05", "MVP-06"]
blocked_by: []
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1 · §10.6（终端正确性矩阵）· §附录 A D5
risk_ref:
reviewer: Kimi
---

# MVP-04: 多 Tab 终端

> **状态**：`draft`
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
| Phase B · PTY runtime | portable-pty 启动 · stdin/stdout 桥接 · bounded mpsc + drop-oldest（SPIKE-05 架构）· resize/signal 传递 | ⏳ todo | — |
| Phase C · xterm 前端 | xterm.js 5.5 渲染 · SolidJS 组件集成 · WebGL → Canvas → DOM fallback · theme token 接入 | ⏳ todo | — |
| Phase D · shell 兼容 | zsh/bash/fish 默认选择（`app_settings.default_shell`）· Claude CLI / Codex CLI 实机（SPIKE-06 §A 已脱敏） | ⏳ todo | — |
| Phase E · 持久化 | `scrollback_append` + `scrollback_fetch` IPC 串起前后端 · 关 Tab 清 scrollback（FK CASCADE） | ⏳ todo | — |
| Phase F · runtime 证据 | ≥ 3 张截图或 30s 录屏 · 覆盖 create/close/rename/switch/scrollback · 放 `docs/runtime-evidence/mvp-04/` | ⏳ todo | — |

**下次 agent 起点**：Phase B · 依赖 Phase A 已落地的 `TabsDao::create_tab/list_tabs` + IPC `tabs.create/tabs.list/tabs.close/tabs.rename` + `scrollback_append/fetch`（见 PR #72）· 不要重写 storage 层。

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

> 依据：PR #63 ts-rs rollout 确立的 IPC contract 规范（见 `docs/runtime-evidence/chore-ts-rs-rollout/h2-regression-proof.md`）。

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

## 📝 Notes

- MVP-04 不实现 tmux control mode（看 tmux 作为普通程序跑即可）
- Claude/Codex CLI 的协议解析留给 v1.0 AI-Aware（SPIKE-07 parser spike）

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
