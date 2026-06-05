---
id: FEAT-01
type: feat
title: Windows 平台 parity 收尾（ADR-024 提前立项后剩余项）
status: draft
owner:
phase: 当前 active scope（ADR-024）
depends_on: []
blocks: []
estimate: 3d
plan_ref: implementation-plan.md §1.4 / §3.1
risk_ref: R12
reviewer:
---

# FEAT-01: Windows 平台 parity 收尾（ADR-024 提前立项后剩余项）

> **状态**：`draft` → `ready` → `in-progress` → `done`
> **战略依据**：[ADR-024](../adr/ADR-024-windows-platform-pull-forward.md)（Windows 从 v0.4 提前到当前 active scope）· CLAUDE.md 决策表 #8

---

## 🎯 目标（Goal）

收敛 Windows 11（x64 MSVC）平台 parity 到与 macOS / Ubuntu 同等水平 —— 追踪 [ADR-024](../adr/ADR-024-windows-platform-pull-forward.md) 提前立项后、session 34 #431 + session 37 #452 中被 defer 或未覆盖的剩余 Windows 项。

## 📖 背景（Context）

- **session 34 · PR #431**：完成 Windows 适配主体 —— `pty.rs` ConPTY reader（修 reader 死锁 / 自然退出漏检）· shell 探测链 `pwsh→powershell→cmd` · external_term / config_import / keybinding / fs_watch 全平台分支 · CI 矩阵 ubuntu+windows 实跑 · 真实 `.exe`（NSIS）+ `.msi`（WiX）。
- **session 37 · PR #452**：Windows GUI 层 —— 无边框窗口 + 自绘深色标题栏 + WebView2 配色 + 字体 latin bundle + 关闭确认模态框 + pane 焦点切换。其中 app-menu 快捷键 fallback 被 Arbiter defer（lib.rs 注释记录）。
- **ADR-024（2026-06-04 accepted）**：Arbiter 选 (a) 正式提前立项 · 修订决策表 #8（macOS + Ubuntu + Windows 三平台并列当前 active scope）。本 spec 是 ADR-024 实施 §2 钦定的「Windows 剩余项 task spec」。

---

## 🎨 功能范围（Scope）

**Do**：

- **app-menu 快捷键 keydown fallback（Windows）** ✅ 已实现（#456）：menu 在 Windows 关闭后键盘 accelerator 失去来源（#452 defer）。前端加 capture-phase keydown → `emit("menu:action")` 复用现有监听（后端无需改）。**键位 `Ctrl+Shift+T`（新标签）/ `Ctrl+Shift+W`（关标签）/ `Ctrl+,`（设置）** —— 不用裸 `Ctrl+T/W/D`（撞 shell readline transpose / kill-word / EOF）· 采 Windows Terminal / VS Code 约定。**split（`Cmd+D`/`Cmd+Shift+D`）的 Windows fallback 暂不做** —— split-via-`menu:action` 当前是「即将推出」placeholder（`Terminal.tsx:1426-1429` · 所有平台都只弹 toast）· Windows 补 fallback 会 wire 到 stub 无实义 · 待 split-via-menu 真实现后同步补。
- **GUI critical UX path runtime 验证（§2.14）**：Windows 自绘窗口 min/max/restore/close · 字体/字号实时切换 · split 达 MAX_PANES=16 禁用 · pane mousedown 焦点切换 · Escape 关闭确认弹窗 · color-scheme 暗/浅主题原生控件配色。
- **macOS 回归策略（R1）**：项目无 mac CI leg · 三平台并列后 Windows/Linux 改动可能静默破 mac → 评估补 mac runner 或定期 Arbiter 本机回归窗口（二选一并记录）。
- **v0.1 GA Windows parity gate 决策**：Windows 是否 block v0.1 GA · 含签名（Windows code signing）/ 安装包（NSIS/MSI 分发）/ QA 矩阵口径。
- 其余跨平台 parity 项（随发现补 · 如 IME / 高 DPI / 多显示器）。

**Don't**（显式排除）：

- 不重做 #431/#452 已落地的 Windows 适配核心（ConPTY/pty/shell/GUI）。
- 不做 Windows-only 新功能 —— 保持三平台 parity（区别于 ADR-024 否决的 Windows-first 选项 c）。
- 不做 mac/Ubuntu 降级（ADR-024 = + Windows · 不 − mac/ubuntu）。

## ✅ Acceptance

> ⚠️ draft 阶段 · 部分项待打磨到 ready 时量化（自审四问 §1 递归完备性）。

- [x] Windows app-menu 快捷键 fallback：**`Ctrl+Shift+T` / `Ctrl+Shift+W` / `Ctrl+,`** 触发 new_tab / close_tab / preferences（前端 `emit("menu:action")` 复用既有 wiring · 后端不改 · 代码 + typecheck/build 过 · #456）· ⏳ Windows dev-mode 运行时验证（xterm capture 拦截）待 Arbiter
- [ ] GUI critical UX path 在 Windows `pnpm tauri:dev` 实跑验证通过（§2.14 · 上列各路径目视确认）
- [ ] macOS 回归策略落地并记录（补 mac CI leg · 或 Arbiter 定期回归窗口 · 二选一）
- [ ] v0.1 GA Windows parity gate 决策记录（block / 不 block · 含签名 / 安装包 / QA 矩阵口径）
- [x] 三平台 `#[cfg(target_os)]` parity 审计（本 PR · 整体健康 · shell/PTY/配置导入/外部终端/标题栏/窗口控件/路径 七大子系统 Windows 分支均非 stub · **0 真 high gap** · 2 tech-debt follow-up〔见 Notes 审计结论〕）

## 🧪 测试策略

| 层次      | 范围                                | 覆盖路径                                                                                              |
| --------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------- |
| 单元/集成 | `core/` Rust（含 Windows cfg 分支） | `cargo test --workspace`（windows-latest CI leg）                                                     |
| 前端      | TS/组件                             | `pnpm typecheck` + `pnpm vitest run`（Linux CI 为准 · Windows 本机有 CRLF/@solid-refresh 已知假失败） |
| Runtime   | GUI critical UX path                | §2.14 `pnpm tauri:dev`（Windows + macOS 各一遍 · 防三平台漂移）                                       |

## 📝 Notes / 讨论

- mac CI gap（R1）是三平台并列后最大的回归风险源 —— Windows/Linux 改动当前无 mac 自动兜底。GA 前必须定策略。
- app-menu fallback（#452 唯一显式 defer 项）已实现（#456）· 键位 Ctrl+Shift+T/W + Ctrl+,（裸 Ctrl+T/W/D 撞 readline · 故采 Windows Terminal 约定）。

### 📊 三平台 cfg parity 审计结论（item 5 · 本 PR）

全仓 `#[cfg(target_os)]`（Rust）+ 前端平台判断只读审计。**整体健康 · 0 真 high gap**：shell（app_settings / pty 探测链）、PTY（ConPTY）、配置导入（ghostty / alacritty / iterm2 / keybinding）、外部终端、标题栏（#452）、窗口控件（#456 权限齐）、路径解析 七大子系统的 Windows 分支均实打实（非 stub）· mac/linux 路径无回归。审计同时澄清 / 记录：

- **split 快捷键 fallback = 非真 gap**：split-via-`menu:action` 全平台都是「即将推出」placeholder（`Terminal.tsx:1426-1429`）· Windows 与 mac 同样「无可用键盘 split」· 非 Windows 独缺 · 待 split 真实现后同步补 Windows 键位。
- **🟡 tech-debt #1 · Windows live-cwd 缺失**（`crates/core/src/pty.rs:1685` `detect_process_cwd` 恒 `None` · mac=lsof / linux=/proc）：`working_directory()` 在 Windows 永远回落 spawn-time cwd → 「新 tab 继承当前目录」/「在此处开外部终端」拿到旧目录。已知 ADR-001 OQ3 deferred · 补全需 Windows API（`NtQueryInformationProcess`）· 不阻塞 · follow-up。
- **🟡 tech-debt #2 · 前端平台检测双实现**：canonical `detectPlatform()` / `isMacPlatform()`（`navigator.platform` + `userAgentData` 双信号）vs 7 处内联裸 `navigator.platform.includes("mac")`（`hooks.ts:58` · `pane-keyboard.ts:149` · `usePaneShortcuts.ts:32` · `usePaneNavigation.ts:94` · `mvp17-keyboard.ts:20/35` · `Terminal.tsx:956` · `index.tsx:15`）。`navigator.platform` 渐废 · 内联点在其返回空值时可能误判 → Ctrl 修饰键映射错。当前 Tauri webview 下仍可用故未爆 · follow-up：统一到 `isMacPlatform()`。**本 PR 未盲改**（跨平台键盘逻辑 · 本机无法验 mac/linux）。
- **🟢 已文档化非问题**：ConPTY signal 退化为 kill（`pty.rs:365` · ConPTY 硬限制 · Ctrl+C 走 PTY 输入正常）· GCM 凭据测试 Windows ignore（仅测试门控）。

## 🔗 相关

- ADR：[ADR-024](../adr/ADR-024-windows-platform-pull-forward.md)（Windows 提前立项）
- 对应 `CLAUDE.md` 决策表：#8（三平台并列）
- 相关 PR：#431（Windows 适配）· #452（Windows GUI）· #456（快捷键 fallback · item 1）
- 风险：R12（跨平台 · `implementation-plan.md §9`）· ADR-024 R1（无 mac CI leg）/ R2（三平台同演进复杂度）

---

**填写完毕后自审**（CLAUDE.md "📝 写规则/清单前的自审四问"）：

1. **递归完备性**：frontmatter 字段齐 · 部分 Acceptance 待 ready 量化（draft 阶段允许 · 见顶部 ⚠️）
2. **反向场景**：若不收 parity · 三平台漂移 + mac 静默回归 → 本 spec 正为追踪此
3. **边界适用性**：三平台均适用 · `#[cfg(target_os)]` 隔离 · mac 回归策略是边界补强
4. **YAGNI**：剩余项均来自 #431/#452 实际 defer / R1 实际 gap · 非投机
