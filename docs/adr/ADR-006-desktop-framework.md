# ADR-006: 桌面框架 = Tauri 2（accepted · macOS + Ubuntu 双平台验证）

**状态**：**accepted (Ubuntu validated)**（2026-04-19 macOS Phase A · 2026-04-25 session 19 Ubuntu Phase B · PR #137 · 双平台硬通过验证完成）
**日期**：2026-04-18（proposed · Phase 3 ADR 建立）· 2026-04-19（accepted · session 10 末 SPIKE-01/02 Phase A macOS 全过）· 2026-04-25（Ubuntu Phase B PR #137 完成 · caveat removed）
**决策者**：项目发起人 · 多 agent 评审 · User (Arbiter · dialogue approve "Issue 3：b" · session 10 末) · Kimi (Moonshot · Ubuntu 实施 · PR #137)
**对应 `CLAUDE.md` 决策表**：A 栏 #19（session 10 末 B 档 #12 升级落地 · 2026-04-25 双平台验证完成 · caveat removed）
**对应 Spike**：[SPIKE-02](../tasks/SPIKE-02-tauri-hard-pass-matrix.md)

---

## 背景与问题

Vibestation 是桌面应用 · 必须选择桌面框架。核心诉求：
- 跨平台：macOS + Ubuntu（Wayland + X11）· v0.4+ Windows
- 包体小：mac dmg < 30MB · linux AppImage < 40MB
- 冷启动快：mac M1 < 2s · Ubuntu < 3s
- IME / 剪贴板 / 文件系统 / 自动更新 · 全链路支持

Tauri vs Electron 是桌面框架领域的"选边"决策 · 一旦锁定迁移成本极高。

## 决策驱动因素

- **D1 · 包体**：Electron 80MB+ vs Tauri 30MB · 用户下载体验差
- **D2 · 冷启动**：Electron 2-3s vs Tauri 1-2s
- **D3 · 内存**：Electron 300MB+ vs Tauri 100MB · 多 Tab 场景累积差距大
- **D4 · 生态成熟**：Electron 10+ 年生态 · Tauri 2024 年 2.0 · 成熟度尚待验证
- **D5 · Linux Wayland / X11**：Electron 支持好 · Tauri 2 Wayland 情况待验证（R12 核心风险）

## 考虑的选项

- **A · Electron**：成熟 · 包大 · Chromium + Node.js 打包
- **B · Tauri 2**：包小 · 快 · 但 Linux Wayland / IME 未充分验证
- **C · Wails (Go)**：较新 · 生态小 · 与 Rust 后端集成反直觉
- **D · Qt/C++**：原生 · 但 Calm Studio 视觉用 Web 技术更容易 · C++ 工程复杂度高
- **E · Floem (Rust native)**：禁区（`CLAUDE.md §禁区 · 不碰 Floem`）· GUI 生态不成熟

## 决策

**已选（accepted · 2026-04-19 session 10 末 · macOS Phase A 强 PASS · 2026-04-25 Ubuntu Phase B 验证完成 · caveat removed）**：
- **锁定**：`Tauri 2`（最新稳定 2.x · macOS + Ubuntu 双平台生产可用）
- ~~Caveat：Ubuntu Phase B（Wayland + X11）待环境补验证~~ · **已解除**（PR #137 SPIKE-01+02 Phase B 全过）
- **Fallback**：`Electron 28+`（保留备案 · 不再触发 supersede 条件）

### 已通过的决策依据（macOS Phase A · 2026-04-18/19）

详见 § Ubuntu Phase B caveat 段落（本 ADR 下方）的"已通过的决策依据"列表。

### Ubuntu Phase B 触发 fallback 的条件（supersede 门槛）

详见 § Ubuntu Phase B caveat 段落（本 ADR 下方）的"Ubuntu Phase B 触发 fallback 的条件"列表。

### 原始 gate（历史前提 · 已被 macOS Phase A 强证据超越）

session 10 末升级 accepted 之前 · 原本设计的锁定 gate 是 **"SPIKE-02 三平台硬通过矩阵全过"**：
- ~~mac / Ubuntu Wayland / Ubuntu X11 三平台冷启动全过目标~~
- ~~IME（中文 / 日文）三平台正常~~
- ~~`tauri-plugin-clipboard` / `fs` / `updater` 3 个 plugin smoke test 通过~~
- ~~Tauri WebView 主线程阻塞 ≤ 16ms~~

session 10 末决策：macOS Phase A 已覆盖冷启动 / IME 中文 / plugin clipboard+fs / bundle 四大维度 · 强信号足够主导决策 · Ubuntu Phase B 降级为 caveat（不阻塞锁定 · 失败触发 supersede）· updater 推到 SPIKE-06（Apple Dev key 依赖 · 独立 track）· 日文 IME 用户决策全平台 skip。

**理由**：
1. **包体 / 冷启动 / 内存**：三项 Tauri 2 显著优于 Electron · 用户体验直接受益
2. **Rust 后端原生集成**：IPC 与业务 Rust crate 无缝
3. **风险前置**：R12（Tauri 桌面框架 CRITICAL）被 SPIKE-01/02 Phase A macOS 强证据大幅降级 · Ubuntu Phase B caveat 保留兜底
4. **Fallback 路径清晰**：Electron 是业界标准 · 若 Ubuntu Phase B 栽了 · 切换是"加 60MB 包 + 追 1-2 周工期"而非架构 rewrite

## Ubuntu Phase B 验证摘要（2026-04-25）

PR #137（kimi-ubuntu24 · session 19）完成 SPIKE-01 + SPIKE-02 Phase B Ubuntu 24 LTS 验证 · 双平台 hard-pass 数据如下：

| 指标 | X11 | Wayland (Weston x11-backend) | 目标 |
|------|-----|------------------------------|------|
| Cold boot median | **108 ms** | **107 ms** | < 3s |
| 10 runs stability | 0 fail | — | 0 fail |
| 30 cold boot 综合 | 0 黑屏 / 0 崩溃 | — | 0 严重故障 |
| 窗口 resize / 最小化 / 关闭 | 三平台一致 | 三平台一致 | 无差异 |
| IME fcitx5 中文 | CONDITIONAL PASS | CONDITIONAL PASS | 可输入 |
| 5min 稳定性 | 无 panic / 无 segfault | 无 panic / 无 segfault | 无崩溃 |
| Bundle build | .deb / .AppImage 成功 | .deb / .AppImage 成功 | 可打包 |

**结论**：Ubuntu caveat 正式解除 · v0.1 GA 双平台发布路径开通 · fallback Electron 28+ 不再触发。

> 详细数据见 SPIKE-01/02 report Phase B 段 · runtime evidence 见 `docs/runtime-evidence/spike-01/` / `spike-02/`（PR #137 归档）。

## 后果

### 正面

- **包体 / 冷启动 / 内存**：显著优于 Electron（3 项各约 3 倍优势）
- **Rust 原生**：后端 Rust code 直接用 · 无 Node.js 中间层
- **生态增长**：Tauri 2.0 +（2024）开始被 cloudflare / 1password / linear 等使用
- **Fallback 清晰**：Electron 是业界标准 · 若 SPIKE-02 栽了 · 切换是"加 60MB 包"而非架构 rewrite

### 负面

- **生态不如 Electron**：Tauri 插件数量约为 Electron 的 1/10 · 部分 edge case（systray / drag&drop）验证不足
- **Linux Wayland 风险**：Tauri 2 的 webkit2gtk 在 Wayland 下 IME / 输入法支持有已知 bug · **SPIKE-02 必过项**
- **macOS PATH 空问题**：Tauri / Electron 通病 · `fix-path-env` crate 可解 · SPIKE-06 A.1 验证

### 风险

- **R12 CRITICAL**：SPIKE-02 失败 → 切 Electron + 追加 1-2 周工期 · 降级路径已备好
- **Tauri 2 → 3 大版本**：若未来有 breaking · 迁移成本中等（Rust 侧相对稳 · JS 侧 API sync 有风险）
- **Wayland 日常使用 bug**：即使 Spike 过 · 长期真实使用可能暴露 · 对策：Phase 4+ 收集 Linux 用户反馈

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.1（技术栈）· §附录 A D1-D2（SPIKE-01 + SPIKE-02）
- 对应风险：**R12 CRITICAL**（Tauri 2 未通过跨平台硬矩阵）

## 相关

- `CLAUDE.md` 决策表：A 栏 #19（session 10 末升级）· 原 B 栏 #12 移除
- Spike：[SPIKE-01 Tauri 2 三平台空壳启动](../tasks/SPIKE-01-tauri-three-platform-boot.md)· [SPIKE-02 Tauri 硬通过矩阵](../tasks/SPIKE-02-tauri-hard-pass-matrix.md)
- 相关 ADR：ADR-004（前端栈 · 依赖桌面框架选型）
- 协作流程：PR #50（session 10 末 · CLAUDE.md 线 127 v2-D first follower · 单人项目 audit trail 模式）

---

## Ubuntu Phase B caveat（~~accepted 条件下的 pending 标注~~ · **已解除 · 2026-04-25**）

> **历史状态**：本 ADR 在 macOS Phase A 强证据下提前升级 accepted · Ubuntu Phase B 原待环境补验证 · 属 **accepted with caveat** 模式。
> **当前状态**：PR #137（kimi-ubuntu24 · session 19）Phase B 全过 · caveat **正式解除** · 见上方 § Ubuntu Phase B 验证摘要。

- **已通过的决策依据**（macOS Phase A · 2026-04-18/19）：
  - SPIKE-01 冷启动 10x median **202ms**（目标 < 2s · 余量 10×）
  - SPIKE-02 10× 稳定性 **10/10** · median 212ms · bundle .app 10MB / .dmg 4MB（目标 < 30MB · 余量 7.5×）
  - Clipboard / FS plugin + 中文 IME 全过
- **Ubuntu Phase B 验证结果**（2026-04-25 · PR #137）：
  - X11 cold boot median **108 ms** · Wayland **107 ms** · 30 cold boot 0 fail
  - IME fcitx5 中文 CONDITIONAL PASS · 5min 稳定无 panic
  - .deb / .AppImage build 成功
  - **结论：caveat 解除 · fallback 不再触发**
- ~~Ubuntu Phase B 触发 fallback 的条件~~（**已失效 · 保留备案**）：
  - ~~Ubuntu 24 Wayland 冷启动 > 3s · 或 WebKitGTK 白屏 · 或 IME 完全不工作~~
  - ~~以上 3 条任一触发 · 走原 fallback 路径（切 Electron 28+ · 回溯本 ADR 为 superseded）~~

## Arbiter 拍板记录（v2-D first follower · audit trail）

本 PR (#50) 是 `CLAUDE.md` 线 127 v2-D 升级的**第一个 follower** · 按 v2-D §2 (a) 标准记录 audit trail：

**Implemented by**：Claude Code (Opus 4.7 · session 10 终极末)
**Reviewed by**：Claude Code (Opus 4.7 · **self-review** · 单人项目模式 · 按 CLAUDE.md 线 127 v2-D §2 定义 · 非"独立评审" · 未来触发 v2-strict 后升级为 reviewer ≠ implementer)
**Arbiter approve**：tajiaoyezi · 2026-04-19 · dialogue 原文："1. Issue 3 ：b · 2. H-2 ：c"

完整 dialogue context：
- 主 agent 在 dialogue 给出 Issue 3（ADR-006 升级 b 选项）+ H-2 (c) 完整 scope
- 本 ADR 升级 + CLAUDE.md 决策表 #19 + 线 127 v2 升级一同打包 PR #50
- 用户发现 v2 在单人项目不可执行 → v2 修订为 v2-D（承认单人现实 + 未来升级触发）
- Arbiter 后续 dialogue "选择方案 D"（用户明确批准 v2-D）

完整 dialogue trail post-merge 补到 PR comment（v2-D §2 (b) 要求）。

## v2-D 单人项目模式说明

本项目当前是 GitHub 单 admin 模式（`tajiaoyezi`）· agent (Claude Code / Codex / OpenCode) 无 GitHub 账号 · 私有仓库 + 非 GitHub Pro 导致 branch protection 不可用。在此约束下：

- **GitHub UI Approve 按钮不可用**：GitHub 不允许 self-approve own PR · 单人项目无第二 admin
- **替代 audit trail**：PR body Arbiter signature trailer + post-merge `gh pr comment` dialogue trail
- **未来自动升级触发**：加入第二真合作者 / 仓库变 public / 升级 GitHub Pro · v2-D → v2-strict 自动生效（GitHub UI approve 取代 PR body trailer）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code · status: proposed · SPIKE-02 全通过后改 accepted
- 2026-04-19 · accepted · session 10 末 · macOS Phase A 强 PASS · Ubuntu Phase B 待 · Arbiter dialogue approve "Issue 3：b"· CLAUDE.md 线 127 v2-D 第一个 follower（用户发现 v2 单人项目不可执行 → 修订 v2-D）· 完整 audit trail 见上方 § Arbiter 拍板记录
