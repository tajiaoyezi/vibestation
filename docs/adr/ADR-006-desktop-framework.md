# ADR-006: 桌面框架 = Tauri 2（accepted · macOS 强 PASS · Ubuntu pending）

**状态**：**accepted**（2026-04-19 session 10 末 · macOS Phase A 强 PASS · Ubuntu Phase B 待环境补验证 · 不阻塞锁定）
**日期**：2026-04-18（proposed · Phase 3 ADR 建立）· 2026-04-19（accepted · session 10 末 SPIKE-01/02 Phase A macOS 全过）
**决策者**：项目发起人 · 多 agent 评审 · User (Arbiter · dialogue approve "Issue 3：b" · session 10 末)
**对应 `CLAUDE.md` 决策表**：A 栏 #19（session 10 末 B 档 #12 升级落地 · accepted with macOS-only caveat）
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

**选择（proposed · pending SPIKE-02）**：
- **默认**：`Tauri 2`（最新稳定 2.x）
- **Fallback**：`Electron 28+`（SPIKE-02 硬通过矩阵任一失败则切换）

**SPIKE-02 硬通过矩阵**（必须全过才锁定 Tauri 2）：
- [ ] mac / Ubuntu Wayland / Ubuntu X11 三平台冷启动全过目标
- [ ] IME（中文 / 日文）三平台正常
- [ ] `tauri-plugin-clipboard` / `fs` / `updater` 3 个 plugin smoke test 通过
- [ ] Tauri WebView 主线程阻塞 ≤ 16ms（60fps · 多 Tab `yes` 压测）
- 任一失败 → fallback Electron 28+

**理由**：
1. **包体 / 冷启动 / 内存**：三项 Tauri 2 显著优于 Electron · 用户体验直接受益
2. **Rust 后端原生集成**：IPC 与业务 Rust crate 无缝
3. **风险前置**：R12（Tauri 桌面框架 CRITICAL）被 SPIKE-02 硬验证消除 · fallback Electron 成熟稳妥

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
- 协作流程：PR #50（session 10 末 · H-2 (c) 过渡末班车）

---

## Ubuntu Phase B caveat（accepted 条件下的 pending 标注）

本 ADR 在 macOS Phase A 强证据下提前升级 accepted · 但 Ubuntu Phase B 仍待环境补验证 · 属于 **accepted with caveat** 模式：

- **已通过的决策依据**（macOS Phase A · 2026-04-18/19）：
  - SPIKE-01 冷启动 10x median **202ms**（目标 < 2s · 余量 10×）
  - SPIKE-02 10× 稳定性 **10/10** · median 212ms · bundle .app 10MB / .dmg 4MB（目标 < 30MB · 余量 7.5×）
  - Clipboard / FS plugin + 中文 IME 全过
- **Ubuntu Phase B 触发 fallback 的条件**：
  - Ubuntu 24 Wayland 冷启动 > 3s · 或 WebKitGTK 白屏 · 或 IME 完全不工作
  - 以上 3 条任一触发 · 走原 fallback 路径（切 Electron 28+ · 回溯本 ADR 为 superseded）
- **Ubuntu Phase B 非 blocker**：
  - macOS 强信号足以主导决策 · 开发阶段优先
  - Ubuntu 环境就绪后补测 · 结果回填 SPIKE-01/02 Phase B report
  - 若 Ubuntu 通过 → caveat 移除 · ADR 标注完整 3 平台过

## Arbiter 拍板记录（H-2 (c) 过渡模式 audit trail）

本 PR (#50) 属于 `CLAUDE.md` 线 127 升级前最后一批 dialogue + proxy merge PR（"末班车"）· 按 H-2 (c) 过渡模式 · Arbiter approve 留痕通过 PR comment + 本段 audit trail：

- **Arbiter (User · `tajiaoyezi@gmail.com`)** · 2026-04-19 session 10 末
- dialogue approve 原文："1. Issue 3 ：b · 2. H-2 ：c"
- 主 agent 在 dialogue 中给出的完整 scope（含本 ADR 升级 + CLAUDE.md 决策表改动 + 线 127 升级）
- Arbiter 后续回复 "合" 作为 merge 授权（若 PR merge 时此 comment 存在）
- 协作规则升级生效后（CLAUDE.md 线 127 v2）· 未来 A 栏改动**不再接受** dialogue approve · 必须 GitHub PR UI `Approve` + `gh pr review --approve`

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code · status: proposed · SPIKE-02 全通过后改 accepted
- 2026-04-19 · accepted · session 10 末 · macOS Phase A 强 PASS · Ubuntu Phase B 待 · Arbiter dialogue approve "Issue 3：b"· 走 H-2 (c) 过渡模式（PR #50）
