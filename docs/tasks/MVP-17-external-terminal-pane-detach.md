---
id: MVP-17
type: mvp
title: 外部终端弹出（Pop to External）+ Pane Detach
status: done
owner:
phase: v0.3
depends_on: ["MVP-04", "MVP-14"]
depends_on_notes: "MVP-04（PTY runtime + xterm.js terminal · 必须能够取当前 cwd + shell · Phase D done @ PR #82-#86）· MVP-14（LayoutNode tree · Pane Detach 需复用递归 layout 数据结构做关闭还原 · Phase B done @ PR #262 · Phase C done @ PR #264）。无 MVP-15/16 依赖（与 syntax highlight / rebase 无关）。"
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5.3
risk_ref: R1
reviewer: Claude Code · self-review（单人项目 v2-D.2 模式 · session 29 详化 PR #283 + fixup PR #284 + Phase B skeleton PR #285 · Phase A/C dispatch 后由实施者 self-review · 主 agent cross-agent review）
---

# MVP-17: Pop to External + Pane Detach

> **状态**：`ready`（v0.3 phase · 2026-05-12 session 29 详化完成 · 主 agent 单人详化 · self-review v2-D.2 模式）
> **依赖**：MVP-04（终端 PTY + xterm.js · done @ PR #82-#86）· MVP-14（Pane LayoutNode tree · done @ PR #262 + #264）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.3`](../implementation-plan.md) · [`§5.3`](../implementation-plan.md)

---

> ⚠️ **2026-05-20 · capture mandate removed**（ADR-023 supersede ADR-011）：本 spec 中所有 **"Phase D macOS + Linux 双平台 5 张截图 + 30s 录屏 + 内存释放量化 / §H.5 runtime evidence / Linux 跨平台 capture / manual QA" 类 acceptance 项 / Phase 表行** 已 supersede · 不再阻塞 spec done flip。inline 文字保留作 audit 历史 · 但**功能上 deprecated**。代码侧 acceptance（external_term 45 + pane_detach::tests:: 19 + 6 重写 vitest 33 全过）保留为 done gate。已捕 `docs/runtime-evidence/mvp-17/phase-{b-lifecycle,e4}/` evidence 保留作 audit。

---

## 🎯 目标（Goal）

两个独立但相邻的 "脱离主窗口" 操作 · 共享 Pane 上下文逻辑：

1. **Pop to External**：把当前 Pane / Tab 的工作上下文（cwd + shell + 选择性 env）一键弹到用户系统已装的外部终端（Ghostty / iTerm2 / Terminal.app / gnome-terminal / Alacritty）继续操作 · 原 Pane 保留 / 不复制 scrollback。
2. **Pane Detach**：把当前 Pane 弹为独立 Tauri WebviewWindow（仍在同一进程 · 共享 PTY backend）· 关闭 detached window 时 Pane 回到原 LayoutNode 位置。

两个操作 UX 路径独立 · 但底层 Pane 引用语义统一（按 `pane_id` 处理）· 复用 MVP-14 LayoutNode tree。

## 📖 背景（Context）

### 真实用户 pain point

- **Pop to External**（场景 A）：用户跑长 build / debug 时想锁定一个终端用本机 tmux + 系统 IME · 不希望被 Vibestation 主窗口 IPC 抖动影响；或临时打开 zsh-with-instant-prompt / oh-my-zsh / starship 的完整体验跑命令
- **Pane Detach**（场景 B）：多显示器用户希望 Git Log 在左屏 · 终端在右屏 · 不依赖 OS 多窗口拖拽（Vibestation 自己管理多窗口集合 · 关闭主窗口时 detached 也跟随退出）

### 与已有 MVP 协同

- 复用 MVP-04 PTY runtime（pane_id → session → PtySession 映射 · 不新增 PTY 抽象）
- 复用 MVP-14 LayoutNode tree 的 pane_id 引用语义（detach 时前端 `detachedPanes: Map<PaneId, WindowLabel>` runtime signal 标记 · LayoutNode 不动 · 关闭时移除 map 项即复原）
- 复用 MVP-10 settings 面板（外部终端偏好 / env 白名单可配置）

### 占位 spec 升级

原 105 行占位 spec → 本次详化（session 29 · 2026-05-12）按 MVP-14/16 模板补 Phase 拆分 + Acceptance A-H + IPC contract + 风险表 + 测试矩阵 · 满足"ready"翻转标准。

---

## 🛠 实施进度

| Phase                                            | 范围                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | 文件域                                                                                                                                                                                                                                                                                                                                               | 依赖                   | 状态                                                                                                                                                                                                                                                               | PR                                                                                                                      |
| ------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| **Phase A · Pop to External 终端识别 + 启动**    | `crates/core/src/external_term/` 新模块：`detect.rs`（按 `TERM_PROGRAM` + `which` 探测 5 种终端）+ `launch.rs`（`open -a <App>` macOS / `gnome-terminal --` Linux · cwd + shell 传参）+ `env_filter.rs`（白名单 env 过滤 · 防 API key 泄漏）· 单元测试 13 detect + 10 env_filter + launch fixture · `crates/core/src/pty.rs` 加 `PtyManager::working_directory()` + `environment()` API（B.1 cwd + B.4 env preview 实施所需）                                                                                                                                                                                                                                                                        | `crates/core/src/external_term/*` · `crates/core/src/lib.rs` · `crates/core/src/pty.rs` · `crates/app/src/lib.rs` · `crates/app/permissions/external_term.toml` · `crates/app/capabilities/default.json` · `crates/app/build.rs`                                                                                                                     | MVP-04 done            | ✅ done @ PR #291（Codex CLI · 3 commits · +1430/-1 · 16 文件 · macOS runtime dry-run 验证 · 11 ts-rs binding 完整落地）                                                                                                                                           | [#291](https://github.com/tajiaoyezi/vibestation/pull/291)                                                              |
| **Phase B · Pane Detach WebviewWindow 生命周期** | `crates/core/src/pane_detach.rs`（业务逻辑 · IPC binding · 不依赖 Tauri）+ `crates/app/src/pane_detach/` 新模块：`window_manager.rs`（新建 WebviewWindow · label `pane-detach-<uuid-hex>` · close listener）· `DetachedPaneMap` runtime-only HashMap · 不持久化 · IPC `pane_detach_open` / `_close` / `_list` + **6 ts-rs binding 显式 export**（PaneDetachOpenRequest / Result / CloseRequest / Result / ListEntry / StateEvent）· 实际生成 **8 .ts 文件**（自动含 PaneDetachAction enum + DetachedWindowBounds nested struct）· 30 单测（18 core state machine + 6 app skeleton + 6 session 30 lifecycle）· **不动 `crates/core/src/panes.rs`**（LayoutNode schema 0 侵入 · 见 §数据模型修订记录） | `crates/core/src/pane_detach.rs` · `crates/app/src/pane_detach/*` · `crates/app/src/lib.rs` · `crates/app/permissions/pane_detach.toml` · `crates/app/capabilities/default.json` · `crates/app/build.rs`                                                                                                                                             | MVP-14 done            | 🟡 skeleton done @ PR #285 · **session 30 in-progress**（Codex CLI dispatch · worktree `/private/tmp/MVP-17-phase-B-work` HEAD `55b1642` · 待补 state.rs + window_manager close listener + 5 integration tests + 3 张 runtime evidence）                           | [#285](https://github.com/tajiaoyezi/vibestation/pull/285) skeleton                                                     |
| **Phase C · UI + 快捷键 + 右键菜单 + 集成**      | 右键菜单 "Pop to External" / "Detach Pane" 两项 · `⌘⇧O` (Pop) + `⌘⇧D` (Detach) 全局快捷键 · detached pane 在原位置显示 "Detached · click to bring back" placeholder · 第二 WebviewWindow 自带 mini-toolbar（pane_id + workspace 名 + "Reattach" 按钮）· 关闭 detached 重新挂载 PaneTerminal                                                                                                                                                                                                                                                                                                                                                                                                          | `web/src/panels/Terminal/PaneContextMenu.tsx`（扩）· `web/src/panels/Terminal/DetachedPlaceholder.tsx`（新建）· `web/src/dialogs/PopToExternal/PopToExternalDialog.tsx`（新建 · 终端选择 + env 预览）· `web/src/lib/pane-detach.ts`（IPC 接通）· `web/src/lib/external-term.ts`（IPC 接通）· `web/src/styles.css`（placeholder 样式 + mini-toolbar） | Phase A + Phase B done | 🟡 partial done @ PR #292（OpenCode · 源码 UI + IPC wrapper · runtime OK）+ PR #294（主 agent fix-up · 6 test files `describe.skip` · 33 vitest tests 待 Cursor session 30 重写）· **OpenCode N=3 §2.10 violation 实证 · Arbiter 推翻永久转出 · N=4 触发条件激活** | [#292](https://github.com/tajiaoyezi/vibestation/pull/292) + [#294](https://github.com/tajiaoyezi/vibestation/pull/294) |
| **Phase D · runtime 证据 + GUI capture**         | macOS + Linux 双平台 5+5 截图 + 30s 录屏 + 内存释放量化（capture 要求 deprecated 2026-05-20 · ADR-023）                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | `docs/runtime-evidence/mvp-17/phase-d/`                                                                                                                                                                                                                                                                                                              | Phase C done           | ✅ done（capture 要求 supersede）                                                                                                                                                                                                                                  | —                                                                                                                       |

---

## 🎨 功能范围（Scope）

### Pop to External · Do

- 识别用户系统装的终端 · 优先级（macOS）：Ghostty > iTerm2 > Terminal.app > Alacritty · Linux：Ghostty > Alacritty > gnome-terminal > Konsole
- 检测策略二选一同时跑（并取并集）：
  1. `$TERM_PROGRAM` 环境变量（用户当前 launching shell 的 hint）
  2. `which <app>` / `ls /Applications/<App>.app` 物理探测（macOS）/ `xdg-mime query default x-scheme-handler/terminal` 默认终端（Linux）
- 弹出时构造启动命令：
  - macOS Ghostty：`open -a Ghostty --args --working-directory=<cwd> -e <shell>`
  - macOS iTerm2：`open -a iTerm.app <cwd>`（iTerm2 自动 cd · 不接 shell · 后续靠 user profile）
  - macOS Terminal.app：`open -a Terminal <cwd>`
  - Linux Ghostty：`ghostty --working-directory=<cwd> -e <shell>`
  - Linux gnome-terminal：`gnome-terminal --working-directory=<cwd> -- <shell>`
  - Linux Alacritty：`alacritty --working-directory <cwd> -e <shell>`
- env 白名单（默认）：`PATH` / `HOME` / `LANG` / `TERM` / `SHELL` / `USER`
- env 黑名单（绝不传）：任何含 `KEY` / `TOKEN` / `SECRET` / `PASSWORD` 字段名（大小写不敏感）· 任何含 `OPENAI` / `ANTHROPIC` / `CLAUDE` / `GITHUB_TOKEN` 字段名 · 任何含 `_PAT` / `_API` 后缀字段
- 弹出后用户选择记忆（按 workspace 维度 · 写入 `app_settings`：`external_term_preferred = "ghostty" | "iterm" | ...`）
- 显示对话框预览将传递的 cwd + shell + env subset（让用户确认）· "Don't ask again" 选项保存到 settings

### Pop to External · Don't

- 不传 scrollback buffer（技术限制：iTerm2 API 受限 · Ghostty stdin pipe 不接 raw escape sequences）· 明确在对话框告知"原 Pane 输出不会跟随"
- 不传 Tauri 进程的 PTY fd（外部终端用自己的 PTY · 不复用）
- 不做反向同步（外部终端关闭 → Vibestation 收回 · 技术上无 hook 点）
- 不做"持续同步"（双端实时镜像 · 推 v1.0+ · 不在本 MVP）

### Pane Detach · Do

- 右键菜单 "Detach Pane" 触发 · 或快捷键 `⌘⇧D`
- 创建新 `WebviewWindow` · label = `pane-detach-<uuid-v4>` · 复用主窗口的 CSP + capabilities（minimal subset · 不再多授权）
- 新窗口大小默认 800×600 · 位置基于主窗口 offset (40, 40) · 之后用户拖动位置不记忆（每次 detach 都默认 offset · 简化 v0.3）
- 新窗口内只渲染当前 Pane（无 sidebar / no tool windows · `?mode=detached&pane=<id>` URL param）
- 主窗口原位置显示 "Detached · click to bring back" placeholder（灰底 + 中央 icon + 计数 "1 of 1 detached"）
- 第二窗口 mini-toolbar（顶部）：workspace 名 + pane label + "Reattach" 按钮
- 关闭 detached window（点 ✕ 或主窗口退出）→ Pane 自动 reattach 到原 LayoutNode 位置 · placeholder 消失 · xterm scrollback 保留（共享 PTY backend）
- 多 detached window 可以并存（不限数 · 但 memory budget 警告 > 4 时 toast）
- detached window 内的 keyboard shortcut 走主窗口 IPC 转发（避免 detached 独立 binding 维护 · 复用 MVP-14 pane-keyboard.ts）

### Pane Detach · Don't

- detached window 间互拖 Pane（v1.0 · 需要 cross-window drag protocol · 当前 Tauri 2 不直接支持）
- global system-wide 快捷键（v1.0 · 需要 Tauri global shortcut plugin · 当前不引入新 plugin）
- detached window 状态持久化（关闭 Vibestation 重启后不恢复 detached · 全部回主窗口 · 简化 v0.3）
- detached window 独立 menubar / dock icon（macOS · 推 v0.4 · 当前共享主窗口的）
- detached window 拥有自己的 layout sub-tree（detached 一定是单 Pane · 不能在 detached 里再分屏 · 简化 state machine）

---

## 🖼 UI 引用（UI Reference）

### Pop to External 对话框

- 模态对话框（Calm Studio 风格 · 复用 MVP-06 ConfigImport 对话框样式 base）
- 4 段：
  1. **选择终端**：4-5 个识别到的终端 icon + 名 · 单选 · default 标 "preferred"（settings 记忆值）· "Don't ask again" checkbox
  2. **预览 cwd + shell + env**：3 列表（cwd 一行 · shell 一行 · env 6-10 项可滚动 · 黑名单字段不显示但有 "X items filtered for security" 提示）
  3. **确认**：底部 [Cancel] [Open in {terminal}]
- 错误状态：终端不在系统 / 路径权限不足 / launching 命令失败 → 顶部红条 + suggested fallback

### Pane Detach 操作

- 右键菜单（PaneTerminal）新增条目："Detach Pane" + 快捷键提示 `⌘⇧D`
- 主窗口原 Pane 位置 placeholder：
  - 灰底（`oklch(--surface-2)`）+ 中央 icon（external-link · 复用 Lucide）+ "Pane detached" 文字 + 提示行 "Detached window is open. Close to bring back."
  - 整块可点 · 点击 → focus 切到 detached window
- detached window 内部：
  - 顶部 mini-toolbar 高度 32px · 字号 13px
  - 内容：`<workspace-name> · pane-<id>` 左侧 / Reattach + ✕ 右侧
  - 主体：PaneTerminal 100% 填充

详化时实施 PR 补截图到 `docs/runtime-evidence/mvp-17/phase-d/`（按 `.claude/rules/runtime-evidence-location.md` R1 命名）。

---

## ✅ Acceptance

### A. Pop to External · 终端识别（Phase A）

- [ ] A.1 macOS 上能检测 Ghostty / iTerm2 / Terminal.app / Alacritty 中实际安装的子集 · 顺序遵循上述优先级 · 通过 `ls /Applications/<App>.app` + `which <bin>` 双路径探测 · 任一通过即视为"已装"
- [ ] A.2 Linux 上能检测 Ghostty / Alacritty / gnome-terminal / Konsole 中实际安装的子集 · 顺序遵循上述优先级 · 通过 `which <bin>` + `xdg-mime query default x-scheme-handler/terminal` 探测
- [ ] A.3 `$TERM_PROGRAM` 提供的 hint 加分（若 `TERM_PROGRAM=iTerm.app` 则 iTerm2 排到首位 · 即使 Ghostty 已装）· `$TERM_PROGRAM` 值映射表测试至少 5 个
- [ ] A.4 0 个终端检测到时 · 返回 `Err(NoExternalTerminalFound)` · 不 panic · IPC 返回 user-friendly error string
- [ ] A.5 单元测试覆盖：5 检测路径 × 2 平台 + `$TERM_PROGRAM` 加分逻辑 + 全空场景 = ≥ 12 单测

### B. Pop to External · cwd + shell + env 传递（Phase A）

- [ ] B.1 cwd 取自当前 Pane 的 PTY session（MVP-04 提供的 `session.working_directory()` · 通过 OSC 7 或 `cwd` symlink 实时获取）
- [ ] B.2 shell 取自 settings 默认 shell（`/bin/zsh` · `/bin/bash` · 用户配置）· 不取 PTY 当前 shell（可能跑 vim / 子 shell · 不准）
- [ ] B.3 env 白名单 / 黑名单按上述 Scope 段实现 · `env_filter.rs` 含 ≥ 8 单测：白名单 pass · 黑名单字段名匹配（KEY / TOKEN / SECRET / PASSWORD · 大小写 / OPENAI / ANTHROPIC / CLAUDE / GITHUB_TOKEN / `_PAT` / `_API` 后缀）
- [ ] B.4 env subset 在对话框预览（≤ 10 项可见 + 计数 "X items filtered for security"）
- [ ] B.5 命令构造（5 终端 × 2 平台）有单测 fixture 验证 escape 正确性（路径含空格 / 引号 / 非 ASCII）

### C. Pane Detach · WebviewWindow 创建（Phase B）

- [ ] C.1 右键菜单点击 "Detach Pane" → 创建新 WebviewWindow · label `pane-detach-<uuid>` · 大小 800×600 · 位置主窗口 offset (40, 40)
- [ ] C.2 detached window URL = `index.html?mode=detached&pane=<id>` · 前端 `App.tsx` 根据 `mode` query 渲染 minimal layout（只 PaneTerminal + mini-toolbar）
- [ ] C.3 主窗口原位置前端 `detachedPanes` map 标记该 pane_id · LayoutNode tree 不动 · 显示 placeholder（点击 focus detached window）
- [ ] C.4 detached window 共享主窗口 PTY backend（同 `pane_id` → 同 PtySession）· xterm.js 在 detached 内 attach 现有 stream · scrollback 保留
- [ ] C.5 detached window 数量无硬上限 · 当前实施测 ≥ 3 个并存稳定

### D. Pane Detach · 关闭恢复（Phase B）

- [ ] D.1 点 detached window ✕ → triggers `pane_detach_close` IPC · backend 从 `DetachedPaneMap` 移除 · emit `pane_detach_state_changed { action: "attached" }` 事件 · 前端 `detachedPanes` map 移除 + placeholder 消失 · PaneTerminal 在原位置 re-mount · xterm 流不断
- [ ] D.2 主窗口退出（quit）时所有 detached window 自动关闭 · 状态全部回主窗口（虽然进程也退 · 但 quit 前的 state save 保持 attached 状态 · 不存 detached 中间态）
- [ ] D.3 detached window 关闭后内存释放 ≤ 10MB 残留（measured by `ps -o rss=` 对比 detach 前 / 关闭后稳态 · 间隔 5s 取均值）
- [ ] D.4 reattach 后 ⌘Enter maximize / 方向键 navigation 等 MVP-14 Phase C 功能正常（regression test）
- [ ] D.5 异常关闭路径（kill -9 detached pid · crash · network loss IPC 断）→ 主窗口检测 IPC channel close 事件 → 自动还原 LayoutNode

### E. 快捷键 + 右键菜单 + 集成（Phase C）

- [ ] E.1 `⌘⇧O` (macOS) / `Ctrl⇧O` (Linux) 触发 Pop to External 对话框
- [ ] E.2 `⌘⇧D` (macOS) / `Ctrl⇧D` (Linux) 触发当前 focused Pane 的 Detach
- [ ] E.3 右键菜单 "Pop to External" + "Detach Pane" 两项可见 · 已 detached 的 Pane（placeholder）右键菜单显示 "Reattach" 替代 "Detach"
- [x] E.4 设置面板 (MVP-10) Appearance 段 / 新增 Pane 段加 "External Terminal" subsection：preferred terminal 下拉 + "Don't ask again" toggle + env 白名单可视化（read-only · v0.3 不允许编辑 · v0.4+ 加自定义）

### F. 跨平台（Phase C）

- [ ] F.1 macOS（13+ · Apple Silicon + Intel）：5 终端检测 + 启动全过
- [ ] F.2 Ubuntu 24 LTS（X11 + Wayland）：4 终端检测 + 启动全过 · Wayland 下 `xdg-mime` fallback
- [ ] F.3 Windows：本 MVP **不实施**（v0.4 跟随 SPIKE-01 Phase C Windows）· spec 明确标 `[platform: macos + linux only]`

### G. 错误处理 / 边界

- [ ] G.1 Pop to External 启动命令失败（exit code ≠ 0）· 不留 zombie process · IPC 返回 stderr 内容 · UI toast 显示 "Failed to open in {terminal}: <reason>"
- [ ] G.2 Pane Detach 时该 Pane 已 maximized（MVP-14 ⌘Enter）· 先取消 maximize · 再 detach（保证 detach state + maximize state 不同时存在）
- [ ] G.3 Pane Detach 时该 Pane 已 detached（重复触发）· no-op + toast "Already detached"
- [ ] G.4 detached window 触发 Detach（嵌套 detach）· no-op + toast "Cannot detach a detached pane"
- [ ] G.5 同一 workspace 内 detach 全部 Pane（极端）· 主窗口可能空 layout · 仍可见 sidebar + tool windows · 右下 toast "All panes detached. Reattach to continue."

### H. 性能 / runtime evidence（Phase D · deferred）

- [ ] H.1 Pop to External 弹出对话框 ≤ 80ms（按 MVP-09 dialog open 标准 · Phase D capture）
- [ ] H.2 Pane Detach window 创建 ≤ 200ms（含 Tauri WebviewWindow init + xterm re-mount）· P99 from console.time 实测
- [ ] H.3 detached window 关闭还原 ≤ 100ms（IPC close → LayoutNode restore → placeholder remove）
- [ ] H.4 内存：单 detached window 增量 RSS ≤ 60MB（含 webview + xterm canvas + shiki cache miss）· 3 detached 并存稳态 ≤ 240MB total
- [ ] H.5 runtime evidence：5 PNG + 1 MP4 30s 录屏 · 放 `docs/runtime-evidence/mvp-17/phase-d/`（验证按 `validate-runtime-evidence.mjs` R1-R5）

---

## 🧪 测试策略

| 层次             | 范围                                                                                                                                                                                                                 | 工具                                | 覆盖率目标                                                   |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------- | ------------------------------------------------------------ |
| 单元（Rust）     | 终端检测 5 case + 命令构造 5 终端 × 2 平台 fixture + env 过滤 ≥ 8 case + WebviewWindow lifecycle state machine                                                                                                       | `cargo test --workspace`            | `external_term` mod ≥ 90% line cov · `pane_detach` mod ≥ 85% |
| 单元（Frontend） | pane-detach.ts / external-term.ts IPC wrapper 错误路径 + DetachedPlaceholder 渲染 + PopToExternalDialog form state                                                                                                   | `vitest`                            | 新增 ≥ 18 单测                                               |
| 集成             | IPC contract pane_detach_open → close → reattach 状态机 · 含异常关闭                                                                                                                                                 | `cargo test --features integration` | 5 integration test                                           |
| E2E              | Playwright 模拟："detach pane → 拖窗口 → 关闭 → 验证主窗口 placeholder 消失 + PaneTerminal 在原位置 re-mount" 一气呵成 · "Pop to External" 弹对话框 + 选 Ghostty + 验证 spawn 命令（不真启 · mock 拦截 `open` 命令） | Playwright                          | 2 E2E（cross-platform）                                      |
| Runtime evidence | 5 PNG + 1 MP4（macOS） · Phase D capture · Linux Phase D part B 推 v0.4 跟 SPIKE-01 Phase C                                                                                                                          | manual                              | Arbiter 60-90 min                                            |

---

## 💾 数据模型变更

### app_settings 新增 KV（Phase C · 通过 MVP-10 settings UI 写）

| key                            | type           | default                      | 说明                                                                |
| ------------------------------ | -------------- | ---------------------------- | ------------------------------------------------------------------- |
| `external_term_preferred`      | string \| null | null                         | 用户选择的默认外部终端 · null = 每次问                              |
| `external_term_dont_ask_again` | bool           | false                        | "Don't ask again" 状态 · true 时跳过对话框                          |
| `pane_detach_default_size`     | json           | `{"width":800,"height":600}` | detach window 默认大小（v0.3 不允许 UI 改 · 仅 settings 文件 edit） |

### Detach state · runtime-only map（**不动 panes.rs schema** · Phase B）

> **本次详化修订（2026-05-12 session 29）**：原 §数据模型变更段写"扩展 LayoutNode + 新增 LayoutLeafState enum + struct PaneLeaf"是误读 · 实际 `crates/core/src/panes.rs` L24 是 `enum LayoutNode { Single { pane_id }, Split {...} }` · 无 `PaneLeaf` struct · 且本 spec D.2 + H.5 已明确"detached state 不持久化 · 重启回主窗口"。**正确实现路径 = runtime-only HashMap · 不动 LayoutNode schema · MVP-14 binding 0 侵入**。

**Phase B 实现**：

```rust
// crates/app/src/pane_detach/state.rs · 新文件 · app 层运行时状态
use std::collections::HashMap;
use std::sync::Mutex;

/// detached pane 运行时状态映射 · App 启动时空 · quit 时全清 · 不持久化
pub struct DetachedPaneMap {
    inner: Mutex<HashMap<PaneId, DetachedWindowInfo>>,
}

pub struct DetachedWindowInfo {
    pub window_label: String,      // tauri WebviewWindow label · "pane-detach-<uuid-v4>"
    pub workspace_id: String,
    pub created_at: SystemTime,
}

impl DetachedPaneMap {
    pub fn new() -> Self { /* ... */ }
    pub fn insert(&self, pane_id: PaneId, info: DetachedWindowInfo) -> Result<(), DetachError> { /* idempotent · 已存在返回 AlreadyDetached */ }
    pub fn remove(&self, pane_id: &PaneId) -> Option<DetachedWindowInfo> { /* 关闭时调用 */ }
    pub fn get(&self, pane_id: &PaneId) -> Option<DetachedWindowInfo> { /* clone · 不持锁 */ }
    pub fn list(&self) -> Vec<(PaneId, DetachedWindowInfo)> { /* 全量列举 · UI 显示 */ }
    pub fn clear(&self) { /* App quit 时调用 · 无副作用 · IPC 接收方负责发 close */ }
}
```

**注册为 Tauri state**：`tauri::Builder::default().manage(DetachedPaneMap::new())`。

**不改 `crates/core/src/panes.rs`**：MVP-14 已 done · LayoutNode schema 稳定 · 0 binding regression。

**前端如何感知 detached 状态**：

- 通过 `pane_detach_state_changed` 事件（payload: `{ pane_id, action: "detached" | "attached", window_label?: string }`）
- 前端 `App.tsx` 维护 `detachedPanes: Map<PaneId, WindowLabel>` solid signal · 渲染 placeholder 时查 map
- MVP-14 `PaneTerminal.tsx` props 加一行 `isDetached?: boolean` 决定渲染主体 vs placeholder（接收 caller 计算 · 不污染 LayoutNode tree）

### app_settings 新增 KV（Phase C · 通过 MVP-10 settings UI 写 · 唯一持久化项）

| key                            | type           | default                      | 说明                                                                |
| ------------------------------ | -------------- | ---------------------------- | ------------------------------------------------------------------- |
| `external_term_preferred`      | string \| null | null                         | 用户选择的默认外部终端 · null = 每次问                              |
| `external_term_dont_ask_again` | bool           | false                        | "Don't ask again" 状态 · true 时跳过对话框                          |
| `pane_detach_default_size`     | json           | `{"width":800,"height":600}` | detach window 默认大小（v0.3 不允许 UI 改 · 仅 settings 文件 edit） |

**总结**：本 MVP 唯一 schema 变更是 `app_settings` KV 加 3 行（既有表 + 既有 IPC · 复用 MVP-10 已有写路径）· 0 DB migration · 0 LayoutNode binding 变更 · detached state 全 runtime。

---

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)。所有 IPC struct 走 ts-rs 自动生成。

### G.1 本 MVP 新增 IPC binding 清单

| #   | Struct/Enum                     | 用途                                                                                                                             | Phase |
| --- | ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ----- |
| 1   | `ExternalTerminalInfo`          | Phase A `external_term_list` 返回单个终端 info（id / name / icon_path? / detected）                                              | A     |
| 2   | `ExternalTerminalLaunchRequest` | Phase A `external_term_launch` 入参（terminal_id + pane_id + override_env?）                                                     | A     |
| 3   | `ExternalTerminalLaunchResult`  | Phase A `external_term_launch` 返回（success / failed_reason）                                                                   | A     |
| 4   | `EnvPreview`                    | Phase A `external_term_preview_env` 返回（visible_entries: Vec\<EnvEntry\> + filtered_count: u32）                               | A     |
| 5   | `EnvEntry`                      | `EnvPreview` 内嵌（key + value_truncated: String 最长 40 char + is_sensitive_redacted: bool）                                    | A     |
| 6   | `PaneDetachOpenRequest`         | Phase B `pane_detach_open` 入参（pane_id）                                                                                       | B     |
| 7   | `PaneDetachOpenResult`          | Phase B `pane_detach_open` 返回（window_label + initial_bounds: Rect）                                                           | B     |
| 8   | `PaneDetachCloseRequest`        | Phase B `pane_detach_close` 入参（window_label）                                                                                 | B     |
| 9   | `PaneDetachCloseResult`         | Phase B `pane_detach_close` 返回（pane_id reattached）                                                                           | B     |
| 10  | `PaneDetachListEntry`           | Phase B `pane_detach_list` 返回的单项（window_label + pane_id + bounds）                                                         | B     |
| 11  | `PaneDetachStateEvent`          | Phase B 事件 payload（pane_detach_state_changed · pane_id + action: "detached" \| "attached" + window_label?: Option\<String\>） | B     |

共 **11 个新 binding**（原 12 · 修订移除 LayoutLeafState · 见 §数据模型变更修订记录 · detached state 改 runtime-only · 不扩 LayoutNode schema）。

### G.2 IPC command 清单

| #   | Command                     | 入参                            | 返回                           | Phase | Permission                        |
| --- | --------------------------- | ------------------------------- | ------------------------------ | ----- | --------------------------------- |
| 1   | `external_term_list`        | `()`                            | `Vec<ExternalTerminalInfo>`    | A     | `allow-external-term-list`        |
| 2   | `external_term_preview_env` | `pane_id: PaneId`               | `EnvPreview`                   | A     | `allow-external-term-preview-env` |
| 3   | `external_term_launch`      | `ExternalTerminalLaunchRequest` | `ExternalTerminalLaunchResult` | A     | `allow-external-term-launch`      |
| 4   | `pane_detach_open`          | `PaneDetachOpenRequest`         | `PaneDetachOpenResult`         | B     | `allow-pane-detach-open`          |
| 5   | `pane_detach_close`         | `PaneDetachCloseRequest`        | `PaneDetachCloseResult`        | B     | `allow-pane-detach-close`         |
| 6   | `pane_detach_list`          | `()`                            | `Vec<PaneDetachListEntry>`     | B     | `allow-pane-detach-list`          |

共 6 个新 IPC + 6 permission（`permissions/external_term.toml` 3 个 + `permissions/pane_detach.toml` 3 个）。

### G.3 事件清单

| #   | Event 名                    | Payload                                    | 触发时机                                 |
| --- | --------------------------- | ------------------------------------------ | ---------------------------------------- |
| 1   | `pane_detach_state_changed` | `PaneDetachStateEvent`                     | detached window 创建 / 关闭 / 异常 close |
| 2   | `external_term_launched`    | `{ terminal_id: String, pane_id: PaneId }` | 外部终端成功启动后 emit · UI 关闭对话框  |

### G.4 H2 regression proof（详化时实施 PR 验证）

按 ADR-014 §H2 标准 6 步：

1. 改 Rust struct 字段名 → 实施 PR cargo build 通过
2. cargo build (ts-rs export) → bindings/\*.ts 更新
3. 前端 caller 命中字段错 → pnpm typecheck fail
4. 修复前端 caller → typecheck pass
5. CI 验证：bindings/\*.ts 是 git tracked · diff 显示前后变化
6. 文档 link：实施 PR 在 spec §G.4 标记 "H2 proof done @ PR #XXX"

---

## §H. 决策锁定（MVP-17 专有）

### H.1 多窗口策略：每 detach 一个 WebviewWindow（不复用 / 不池化）

**决策**：每次 detach 创建独立 WebviewWindow · 关闭即销毁 · 不维护"detached window pool"。

**理由**：

- Tauri 2 WebviewWindow 创建 ≤ 200ms（H.2 acceptance）· 不需预热
- 池化方案在 detach 频次低（典型 1-3/session）下无收益 · 反而增加 state machine 复杂度
- 关闭即销毁让内存 budget 可预测（D.3 ≤ 10MB 残留）

**待 v0.4 评估**：若用户实测发现 detach 频次高（> 5/session）· 重新评估池化（预创建 1 个 hidden window · 复用 webview）。

### H.2 不碰列表：不引第三方多窗口管理库 / 不跨窗口 drag

- ❌ 不引 `tauri-plugin-window-state`（不持久化 detached window 位置 / 大小 · 简化 v0.3）
- ❌ 不实现 cross-window Pane drag（v1.0 需要 Tauri 2 IPC channel + canvas-level drag protocol）
- ❌ 不引 `tauri-plugin-global-shortcut`（detach / pop 快捷键限主窗口 focus 时生效 · 当前 KISS）
- ❌ 不做 detached window menubar 独立（macOS · v0.4 跟随 native menu MVP）

### H.3 安全：env 白名单 + 黑名单双层

**决策**：env 传递走"白名单 default + 黑名单 sanity check"双层模式。

**白名单层**（默认 6 项）：`PATH` / `HOME` / `LANG` / `TERM` / `SHELL` / `USER`

**黑名单层**（sanity · 即使白名单意外含 · 也 redact）：

- 字段名（大小写不敏感）含：`KEY` / `TOKEN` / `SECRET` / `PASSWORD` / `_PAT` / `_API`
- 字段名含 vendor：`OPENAI` / `ANTHROPIC` / `CLAUDE` / `GITHUB_TOKEN` / `AWS_` / `GCP_`

**实施**：`env_filter.rs::filter_env()` 单次 pass · O(n) · ≥ 8 单测 cover 黑名单边界（前缀 / 后缀 / 中段 / 大小写）。

**v0.4 评估**：若用户反馈白名单太严（例如 `LC_*` locale 系列）· 在 settings 增加 "Custom env include list" · 但保留黑名单强制层。

### H.4 Detach 不下钻：detached 内不再分屏

**决策**：detached window 内永远是单 Pane · 不允许在 detached 内再分屏 / 再 detach。

**理由**：

- 简化 detach state 状态机（一个 detached window 永远对应 LayoutNode 的一个 `Single { pane_id }` variant · 不会指向 `Split` 子树）
- 用户场景实际：detach 是为了"独立屏 / 独立窗口"· 而非"独立分屏组"
- v1.0+ 若有用户反馈 · 评估 detached 内分屏（但需要重写 LayoutNode 支持"sub-tree as root"）

### H.5 关闭主窗口时 detached 全部跟随

**决策**：主窗口 quit 时所有 detached window 自动关闭 · 不询问。

**理由**：

- Tauri 2 默认行为（主窗口关闭 → app 退出 → 所有窗口关闭）· 不 override
- 用户预期：主窗口是 anchor · 关掉就是关 Vibestation · detached 跟随是符合直觉的
- 反向场景"用户关主窗口但保留 detached"在 v0.3 范围外（需要 Vibestation 改为 "single-window-quit-keeps-others" 模式 · 复杂度过高）

---

## ⚠️ 风险（Risks）

| ID  | 风险                                  | 触发条件                                                           | 缓解                                                                                             | 严重度    |
| --- | ------------------------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ | --------- |
| R1  | env 泄漏 API key                      | 用户在 shell `export OPENAI_API_KEY=...` 后 Pop to External        | 黑名单层强制 redact `OPENAI` 字段名 · §H.3 + B.3 单测                                            | 🔴 high   |
| R2  | detached window 关闭 → Pane 状态丢失  | IPC channel race · close 事件先于 DetachedPaneMap 移除完成         | D.5 异常路径 IPC channel close listener + `DetachedPaneMap::remove` idempotent + 事件 retry once | 🟡 medium |
| R3  | Pane Detach + Maximized 状态冲突      | MVP-14 ⌘Enter maximize 后再 detach                                 | G.2 detach 前自动取消 maximize                                                                   | 🟡 medium |
| R4  | 多 detached window IPC 拥堵           | 5+ detached 并存 · 每个独立 IPC channel · main process 处理压力    | 当前 KISS · D.3 测 3 detached 稳态 OK · > 4 时 toast 警告 · v0.4 评估 channel multiplexing       | 🟢 low    |
| R5  | macOS Gatekeeper 阻止 detached window | unsigned alpha build · macOS 13+ Gatekeeper 对 sub-window 二次询问 | 复用 MVP-10 §I.D Gatekeeper bypass 指引 · README 已有                                            | 🟢 low    |
| R6  | Linux Wayland detached 窗口失焦       | Wayland security model 不允许 raise window programmatically        | C.1 仅默认 offset 创建 · 不强制 raise · 用户自己点 alt-tab                                       | 🟢 low    |

---

## 📝 Notes / 讨论

### 与 MVP-10 settings 的对接

- Phase C 在 MVP-10 AppearanceGroup.tsx 隔壁加新 group "External Terminal" · 包含：
  - preferred terminal 下拉（自动填充 `external_term_list` 结果）
  - "Don't ask again" toggle
  - env 白名单可视化（read-only · v0.3 不允许编辑 · v0.4+ 加自定义）
- 不修改 MVP-10 已有结构 · 仅新增字段

### 与 MVP-14 LayoutNode 的对接

- **不动 `crates/core/src/panes.rs`**（修订决策 · 详见 §数据模型变更）· LayoutNode schema 稳定 · MVP-14 13 个 binding 0 regression
- detached state 全 runtime（`DetachedPaneMap` HashMap in app state · 前端 `detachedPanes` Solid signal）· App 重启回到 attached（H.5 实施 · 不需特殊 migration）
- MVP-14 的 maximize state（session-level memory）不持久化 · 不影响 detach state

### 命名 / 文案

- 中文文档统一用 "Pop to External" / "外部终端弹出" · "Pane Detach" / "Pane 分离"
- 英文 UI 文案：菜单 "Pop to External…" / "Detach Pane" · placeholder "Pane detached. Close to bring back." · mini-toolbar "Reattach"
- 不使用 "Move to / 移动到"（暗示 cwd 实时同步 · 误导）
- 不使用 "Spawn / 派生"（技术术语 · 对终端用户不友好）

---

## 🔗 相关

- **上游**：MVP-04 (PTY runtime · cwd 获取) · MVP-14 (LayoutNode tree · pane_id 引用语义复用 · **不扩 schema**) · MVP-10 (settings UI 对接)
- **下游**：无（v1.0 cross-window drag / v1.0 global shortcut 都是独立 MVP · 不依赖本 MVP）
- **依据**：`implementation-plan.md` §10.1（v0.3 砍到名单）· §5.3（多窗口策略）
- **ADR 引用**：ADR-014（IPC contract via ts-rs）· ADR-006（Tauri 2 desktop framework）
- **相关 rule**：`.claude/rules/tauri-v2-patterns.md` §1 ACL permission · `.claude/rules/runtime-evidence-location.md` R1-R5

---

## 📝 详化完成度评估（详化 PR review 参考）

按 MVP-14 / MVP-16 详化标准 12 项：

| #   | 项                                          | 状态                                                                                        |
| --- | ------------------------------------------- | ------------------------------------------------------------------------------------------- |
| 1   | 目标 / 背景清晰                             | ✅                                                                                          |
| 2   | Phase 拆分（≥ 3 phase · 文件域 + 依赖明确） | ✅（A/B/C/D · 表内含文件域 + 依赖）                                                         |
| 3   | Scope Do / Don't 边界清晰                   | ✅                                                                                          |
| 4   | UI 引用（含截图归档目录）                   | ✅                                                                                          |
| 5   | Acceptance ≥ 25 项（A-H 8 节）              | ✅（A.1-A.5 / B.1-B.5 / C.1-C.5 / D.1-D.5 / E.1-E.4 / F.1-F.3 / G.1-G.5 / H.1-H.5 = 37 项） |
| 6   | 测试策略（单元 / 集成 / E2E / runtime）     | ✅                                                                                          |
| 7   | 数据模型变更（含迁移）                      | ✅（serde default 兼容 · 无 DB migration）                                                  |
| 8   | IPC contract（ts-rs binding ≥ 5）           | ✅（12 binding + 6 IPC + 2 event）                                                          |
| 9   | 决策锁定（≥ 3 H.x）                         | ✅（H.1-H.5 5 项）                                                                          |
| 10  | 风险表（R1-R5+）                            | ✅（R1-R6 6 项 · R1 high · R2/R3 medium · R4-R6 low）                                       |
| 11  | H2 regression proof 说明                    | ✅（§G.4 引用 ADR-014）                                                                     |
| 12  | 自审四问                                    | ✅（见下）                                                                                  |

**完成度**：12/12 = **100%**（建议 Arbiter approve PR 后翻 status: ready 翻转）。

---

## 🔍 自审四问

按 CLAUDE.md §第 8 节"自审四问"逐条：

1. **递归完备性**：Pop to External + Pane Detach 双操作各自完整覆盖（5+5 终端识别 / 启动 / 关闭恢复 / 异常路径）· spec 自己也按 12 项详化标准对照通过 ✅
2. **反向场景**：
   - 0 终端检测到 → A.4 IPC 返回 `NoExternalTerminalFound` 错误
   - detached window 异常关闭 → D.5 IPC channel close listener 自动还原
   - Pane 已 maximized → G.2 detach 前先取消 maximize
   - Pane 已 detached → G.3/G.4 no-op + toast
   - 全部 Pane 都 detached → G.5 主窗口空 layout · toast 提示
     ✅
3. **边界适用性**：
   - 平台：macOS 13+ + Ubuntu 24 (X11/Wayland) · Windows 推 v0.4（F.3 明示）
   - 终端：5 个（macOS 4 + Linux 4 · 交集 2）· 顺序明确 · 0 检测到也覆盖
   - 并发：detached 数 ≥ 3 稳定 · > 4 toast 警告（R4 缓解）
   - 状态机：Attached / Detached 2 态 · runtime-only `DetachedPaneMap` HashMap · 不持久化 · 0 LayoutNode schema 变更（D.2 + H.5 一致）
     ✅
4. **YAGNI**：
   - cross-window drag · global shortcut · detached menubar 独立 · detached 内分屏 · 双向同步 → 全部明示 don't do · 推 v0.4 / v1.0
   - state pool / persist detached state across restart → 明示 not now
   - 自定义 env 白名单 → 推 v0.4（read-only 显示 v0.3 已够用）
     ✅

---

**详化记录**：

- 2026-05-12 · session 29 · 主 agent 单人详化 · self-review v2-D.2 模式 · 占位 spec（105 行）→ 完整 spec（~480 行）
- 详化范围对齐 MVP-14 / MVP-16 / MVP-12 详化标准 12 项 · 100% 通过
- status 待 PR merge 后翻 `draft → ready`
