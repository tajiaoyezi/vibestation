---
id: SPIKE-01
type: spike
title: Tauri 2 三平台空壳启动（mac + Ubuntu Wayland + X11）
status: blocked
owner: Claude Code (Sonnet 4.6)
phase: W0-D1
depends_on: []
blocks: ["SPIKE-02", "SPIKE-03", "SPIKE-04", "SPIKE-05", "SPIKE-06"]
blocked_by: ["ubuntu-24-environment"]
blocked_from: in-progress
blocked_note: Phase A macOS PASS（PR #20 · 冷启动 202ms median · ADR-006 accepted）· Phase B Ubuntu 等用户搭 Ubuntu 24 LTS 环境 · Ubuntu 已降为 v0.1 GA 最低优先（见 PROGRESS "S-3 Ubuntu 降级"）· 不是等 agent 认领 · session 13 horizontal scan @ 2026-04-21
estimate: 1d
plan_ref: implementation-plan.md §3.1.1 · §附录 A D1
risk_ref: R12
reviewer: User (Arbiter · GitHub PR approve)
---

# SPIKE-01: Tauri 2 三平台空壳启动

> **状态**：`in-progress`（macOS Phase A PR #28 PASS · Ubuntu Phase C 待 Linux 环境 · 关联 MVP-01 PR #28/#33）
> **依赖**：— / **阻塞**：SPIKE-02..06 全部（Tauri 是所有后续 Spike 的载体）
> **战略依据**：[`implementation-plan.md §3.1.1 Tauri Spike 硬通过判据`](../implementation-plan.md) · [`§附录 A D1`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 macOS 15 + Ubuntu 24 Wayland + Ubuntu 24 X11 三台机器上，让 Tauri 2 空壳 app（"Hello Vibestation"）成功启动，并采集冷启动耗时与 IME 初测数据。

## 📖 背景（Context）

- **Tauri 2 是 `CLAUDE.md` 决策表 #12 的默认选择**，但未在 Ubuntu 24 Wayland 上亲自验证过——这是 `implementation-plan.md §9 R12`（CRITICAL 级别）核心风险
- **如果 D1 三平台启动失败 → D2 切 Electron 28+ fallback spike**（`implementation-plan.md §3.1.1`）
- 所有后续 Spike（git2 / redb / PTY / CLI 实机）都依赖这个空壳作为运行环境，所以 SPIKE-01 是 W0 第一天的必过门

---

## ✅ 通过标准（Pass Criteria）

全部勾选为通过，任一项失败触发 Fail Signals：

- [ ] macOS 15 启动成功，冷启动耗时 **< 2s**（多次测量取中位数）
- [ ] Ubuntu 24 Wayland 启动成功，冷启动耗时 **< 3s**
- [ ] Ubuntu 24 X11 启动成功，冷启动耗时 **< 3s**
- [ ] 三平台窗口均能正常渲染 "Hello Vibestation" 文字，无黑屏 / 白屏
- [ ] 窗口 resize / 最小化 / 关闭在三平台均正常
- [ ] IME（中文拼音）初测：在三平台输入框里能打中文字（录屏作证）
- [ ] 单次启动 → 5 分钟内无 panic / 崩溃日志（`tauri dev` console）

## ❌ 失败信号（Fail Signals）

any of：

- 冷启动 > 5s（三平台任一）
- Wayland 下窗口黑屏 / 白屏 / 无法渲染
- IME 在 Wayland 下输入中文无反应或崩溃
- 启动 5 分钟内出现 panic / segfault

## 🔀 Fallback 方案

**通过** → Day 2 继续 Tauri 硬通过矩阵（SPIKE-02）
**历史设计（session 10 末前 · 已被 macOS Phase A 强证据超越）**：失败 → Day 2 启动 **Electron 28+ fallback spike**（1 天），通过则锁定 Electron 并：
- ~~更新 `CLAUDE.md` 决策表 #12 从 B 栏移入 A 栏~~（session 10 末已升级 · 见 A 栏 #19）
- 若 Ubuntu Phase B 失败 → 新开 ADR 记录 [ADR-006](../adr/ADR-006-desktop-framework.md) supersede · Electron 成为新默认
- `implementation-plan.md §3.1` 全章回退为 Electron

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-01-tauri/`：最小 Tauri 2 项目骨架代码（不进 main 仓库，`.gitignore` 已排除）
- [ ] 三平台启动录屏（`spike-artifacts/SPIKE-01/` Phase 3 后建立）
- [ ] 冷启动耗时表（**`docs/spikes/SPIKE-01-report.md`**，per-task；Phase 3 建立 `docs/spikes/` 目录）
- [ ] IME 测试录屏
- [ ] `CLAUDE.md` 决策表 #12 状态更新（通过 → 锁定；失败 → Electron）

## 🛠 依赖资源（Resources Needed）

- macOS 15 开发机 × 1
- Ubuntu 24 LTS 机器 × 1（需要能切换 Wayland 和 X11 会话）
- Rust toolchain（rustup 2024.x+）
- Node 20 LTS + pnpm
- `tauri-cli` 2.x

## ⚠️ 已知风险

- **R12**（`implementation-plan.md §9`，CRITICAL）：Tauri 2 在 Ubuntu 24 Wayland 下可能不稳定——这正是 SPIKE-01 要消除的风险
- **未知风险**：macOS PATH 空问题（`fix-path-env`，D6 也验证）、Wayland 下 IME 框架差异（fcitx5 vs ibus）

---

## 📝 Notes / 讨论

- 本 Spike 故意只做"空壳"——不加入 xterm / git2 / redb，避免被其他依赖问题干扰判断
- IME 初测用 fcitx5（Ubuntu 默认）；如 fcitx5 失败可补测 ibus 但不算通过标准
- 三平台测试顺序建议：macOS → X11 → Wayland（Wayland 是最大风险，放最后深入调查）

## 🔗 相关

- ADR：[`docs/adr/ADR-006-desktop-framework.md`](../adr/ADR-006-desktop-framework.md)（accepted with Ubuntu caveat @ 2026-04-19 · session 10 末升级）
- 对应 `CLAUDE.md` 决策表：**A 栏 #19 桌面框架 = Tauri 2**（原 B 栏 #12 session 10 末升级）
- `implementation-plan.md` 章节：§3.1.1（硬通过判据表）· §附录 A D1 · §9 R12
- 下游：SPIKE-02（硬通过矩阵完整验收）

---

**填写完毕后自审**：

1. **递归完备性**：7 条 Pass Criteria 覆盖了"启动 + 渲染 + 交互 + 输入 + 稳定" 5 个维度 ✅
2. **反向场景**：失败明确走 Electron fallback，不留悬念 ✅
3. **边界适用性**：三平台显式列出，非"所有 Linux 都要过" ✅
4. **YAGNI**：不加任何业务代码，只做空壳 ✅
