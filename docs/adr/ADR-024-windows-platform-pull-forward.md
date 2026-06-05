# ADR-024: Windows 平台从 v0.4 提前到当前 scope（修订决策表 #8）

**状态**：proposed
**日期**：2026-06-04 proposed
**决策者**：Claude Code（主 agent · 提议）· tajiaoyezi（Arbiter · 2026-06-04 · 选 (a) 正式提前立项 · via dispatch dialogue）
**修订**：CLAUDE.md 决策表 **#8**（平台 MVP = macOS + Ubuntu 24 · Windows 推到 v0.4）· implementation-plan.md §3.1（含 line 74 "不支持 Windows（v1.0 前）" / line 96 跨平台 row "mac+ubuntu（v1.0 前）"）
**关联**：[ADR-006](./ADR-006-desktop-framework.md)（桌面框架 Tauri 2 · macOS + Ubuntu validated · 本 ADR 补 Windows 第三平台）· 决策表 #19（Tauri 2）

---

## 背景与问题（Context and Problem Statement）

决策表 #8（A 栏永久锁定 · 2026 早期 pre-code Phase 1 设定）锁「平台 MVP = macOS + Ubuntu 24 · Windows 推到 v0.4」，依据 implementation-plan §3.1（"不支持 Windows · v1.0 前先把 macOS + Ubuntu 打磨透 · ConPTY 和 Wayland/X11 两套坑不同步踩"）。

但实际开发已与该锁定背离 —— **Windows 产品代码已两批合入 main**：

1. **session 34 · PR #431**（2026-05-29）：为项目适配 Windows 11（x64 MSVC）· 全程 S2V 规格驱动 · `pty.rs` cfg 分离 Unix 内核 + Windows ConPTY reader（修 2 个真实运行期 bug）· shell 探测链 `pwsh→powershell→cmd` · external_term / config_import / keybinding / fs_watch 全平台分支 · 前端 `platform-windows` + `format-shortcut` 平台感知 · **CI 矩阵 ubuntu-latest + windows-latest 实跑全绿** · 真实产出 `.exe`（NSIS 7.57MB）+ `.msi`（WiX 10.18MB）· ConPTY 真 spawn 实证 · 全走 `#[cfg(target_os)]` 分支 · Unix 逻辑零回归
2. **session 37 · PR #452**（2026-06-04）：Windows GUI 层 —— 无边框窗口 + 前端自绘深色标题栏 + WebView2 配色统一 + 字体 latin 子集 bundle + 终端关闭确认自绘模态框 + pane 分屏焦点切换修复 · 质量门全过 + 4 维对抗评审 2 high 修

且 **Arbiter 的主开发 / 日常使用平台就是 Windows 11**（本仓 `git config` 身份在 Windows 机器 · #452 即 daily-driver 分支）。

session 37 账本对账时把这一「锁定决策 vs 现实」漂移作为待决治理项标出（PROGRESS 卡点段 + Session 37 条目）。Arbiter 2026-06-04 拍板 **(a) 正式提前立项 Windows**。

**问题**：#8 是 A 栏永久锁定项 · 不能静默改 · 必须走 ADR proposed→accepted + 同步 CLAUDE.md + implementation-plan + Arbiter 拍板（CLAUDE.md「改锁定表 A 栏前必须」）。本 ADR 承载该修订。

## 决策驱动因素（Decision Drivers）

- **D1 · 账本必须反映现实**：码已在 main · Arbiter 日常在 Windows 开发 · 锁定决策却说「Windows 推 v0.4」· 漂移会让未来 agent 困惑「为何 main 有不在 scope 的 Windows 码」。
- **D2 · Windows 已有 CI 兜底**：`ci.yml` 已含 windows-latest leg（session 34 落地）· ConPTY / bundle / pty 进程级已自动化实证 · 不是「没验证的探索」。
- **D3 · 原 deferral 理由部分已被消化**：「ConPTY 与 Wayland/X11 两套坑不同步踩」—— ConPTY 坑 session 34 已踩平（reader 死锁 / 自然退出漏检已修）· 三平台分支用 `#[cfg(target_os)]` 隔离 · Unix 零回归实证。
- **D4 · 与 ADR-006 协同**：ADR-006 已 validate macOS + Ubuntu（Tauri 2）· 本 ADR 把 Windows 加为第三 validated 平台（session 34 windows-latest 实跑 + 真实 .exe/.msi）。

## 考虑的选项（Considered Options）

### 选项 (a) · 正式提前立项 Windows（chosen）

修订 #8：Windows 从 v0.4 提前到**当前 active scope** · 与 macOS + Ubuntu 24 并列为受支持平台。开 Windows task spec 收纳剩余项（前端快捷键 fallback / GUI runtime 验证 / mac 回归覆盖）。同步 CLAUDE.md #8 + implementation-plan §3.1。

### 选项 (b) · 当探索性适配 · 保 #8 不变

保 #8 文字 + 加注记「Windows 码已落地为探索性适配 · 非正式范围变更」。低流程 · 不承诺 3 平台维护。被否：账本继续留「码在 main 但不在 scope」张力 · 剩余 Windows 项无正式追踪 · 与 Arbiter「日常就在 Windows 上」的现实不符。

### 选项 (c) · Windows-first 重定向

把 Windows 升为 v0.1 主平台、macOS/Ubuntu 降级。被否：当前无证据表明 mac/Ubuntu 该降级（ADR-006 双平台 validated 仍有效）· 战略级 pivot 超出当前需要（YAGNI）。

## 决策（Decision Outcome）

**选择（proposed）**：选项 (a)。修订决策表 #8 为：

> **平台 = macOS 15 + Ubuntu 24 + Windows 11（x64 MSVC）· 三平台并列当前 active scope**（Windows 从原 v0.4 提前 · ADR-024）。

**配套约束**：

1. **三平台均走 `#[cfg(target_os)]` 隔离 + CI 矩阵兜底**（macOS leg 仍缺 · 见 R1）· Unix 逻辑改动不得回归 Windows · 反之亦然。
2. **Windows 剩余项进新 Windows task spec**（docs/tasks/ · 本 ADR accepted 后随附 PR 创建 draft）：收纳 (i) app-menu 快捷键 keydown fallback（Ctrl+T/W/, · #452 defer）(ii) GUI critical UX path runtime 验证（§2.14 · Arbiter 窗口）(iii) 其余 §2.14 / 跨平台 parity 项。
3. **v0.1 GA 平台清单**：Windows 加入受支持平台集 · 但**与 mac/Ubuntu 的 GA gate parity（签名 / 安装包 / QA 矩阵）由 Windows task spec 细化**（本 ADR 不钦定 Windows 必须 block v0.1 GA · 留 spec + Arbiter 定）。
4. **不降级 mac/Ubuntu**：ADR-006 双平台 validated 不动 · 本 ADR 只做 + Windows · 不做 − mac/ubuntu（与选项 c 区分）。

## 后果（Consequences）

### 正面

- 账本与现实对齐 · 未来 agent 明确 Windows 在 scope · 剩余 Windows 项有正式追踪 home。
- session 34 / 37 的 Windows 工作获正式承认 · 不再是「孤儿码」。
- Windows CI leg（已存在）的价值被制度化（回归保护正式纳入门）。

### 负面

- **承诺 3 平台维护负担**：每个 cross-platform 改动需考虑三平台 · review/测试面增大。
- **GA 范围问题**：Windows 是否 block v0.1 GA 需 Windows task spec + Arbiter 进一步定（本 ADR 不钦定）。

### 风险

- **R1 · 无 macOS CI leg**：项目当前无 mac runner（session 34 已记）· 三平台并列后 mac 回归靠本机/手动 · Windows/Linux 改动可能静默破 mac。缓解：Windows task spec 记此 gap + GA 前评估补 mac CI（或 Arbiter 本机回归窗口）。
- **R2 · #8 原 "ConPTY 与 Wayland/X11 不同步踩" 顾虑**：ConPTY 已踩平 · 但三平台同时演进仍增复杂度 · 缓解 = `#[cfg]` 隔离 + CI 矩阵 + 每平台 parity 在 spec 内逐项追踪。

## 实施（Implementation · proposed→accepted 两 PR）

| PR | 内容 |
| --- | --- |
| PR 1（本 PR · proposed） | 新增本 ADR-024（status: proposed）· 仅此一文件 · 不动 CLAUDE.md / implementation-plan / spec |
| PR 2（accepted 翻转 · 待 Arbiter 确认本 ADR 措辞后） | ADR-024 status proposed→accepted · 同步 CLAUDE.md 决策表 #8（依据加 ADR-024）· implementation-plan §3.1（line 74 / line 96 校直 Windows 提前）· 新建 Windows task spec（draft · 收纳剩余项）|

## 关联

- **修订**：CLAUDE.md 决策表 #8 · implementation-plan §3.1
- **协同**：[ADR-006](./ADR-006-desktop-framework.md)（Tauri 2 · 本 ADR 补 Windows 第三 validated 平台）· 决策表 #19
- **触发来源**：session 34 PR #431（Windows 适配）· session 37 PR #452（Windows GUI）+ 账本漂移对账
- **后续**：Windows task spec（accepted 翻转 PR 随附 · 收纳快捷键 fallback / GUI 验证 / mac 回归 / GA parity）

## 自审四问

1. **递归完备性**：本 ADR 自己走 proposed→accepted 两 PR · 修订 A 栏 #8 按「改锁定表」流程（ADR + 同步双文件 + Arbiter 拍板）· 未来若 Windows 再 deferral 走新 ADR supersede ✅
2. **反向场景**：若不立项（保 #8）· 账本「码在 main 不在 scope」张力持续 + 剩余 Windows 项无追踪 → 本 ADR 正是消除该反向场景 ✅
3. **边界适用性**：仅 + Windows · 不动 mac/Ubuntu（区分选项 c）· GA gate parity 留 spec 细化不在本 ADR 钦定（避免越界）· 三平台均 `#[cfg]` 隔离适用 ✅
4. **YAGNI**：Windows 码已两批在 main + Arbiter 日常用 · 立项是对既有现实的承认 · 非投机为未来 Windows 铺路 ✅
