---
id: SPIKE-02
type: spike
title: Tauri 硬通过矩阵 + Electron fallback 验证
status: draft
owner:
phase: W0-D2
depends_on: ["SPIKE-01"]
blocks: ["SPIKE-03", "SPIKE-04", "SPIKE-05", "SPIKE-06"]
estimate: 1d
plan_ref: implementation-plan.md §3.1.1 · §附录 A D2
risk_ref: R12
reviewer:
---

# SPIKE-02: Tauri 硬通过矩阵 + Electron fallback 验证

> **状态**：`draft`
> **依赖**：SPIKE-01（空壳启动通过）/ **阻塞**：SPIKE-03..06（桌面框架锁定后才能做 git / storage / PTY / CLI 验证）
> **战略依据**：[`implementation-plan.md §3.1.1 Tauri Spike 硬通过判据`](../implementation-plan.md) · [`§附录 A D2`](../implementation-plan.md)

---

## 🎯 目标（Goal）

完成 `implementation-plan.md §3.1.1` 全部 6 项硬通过判据的三平台验收；若 SPIKE-01 失败则本 Spike 切换为 Electron 28+ fallback 验证。Day 2 结束必须**锁定桌面框架并写入 ADR**。

## 📖 背景（Context）

- SPIKE-01 只做了"启动 + 渲染 + IME 初测"3 项基础验收
- §3.1.1 还有"10 次零失败、剪贴板、bundle 大小、plugin smoke test"4 项判据未验
- **W0 结束前必须锁定桌面框架**（`CLAUDE.md` 决策表 #12 从 B 栏 → A 栏），否则 W1 MVP 开发无法启动
- 对应 `implementation-plan.md §9 R12`：CRITICAL 风险消除最后一关

---

## ✅ 通过标准（Pass Criteria · Tauri 路径）

**前提**：SPIKE-01 三平台空壳启动通过。

- [ ] **连续启动 10 次零失败**（三平台各 10 次，共 30 次，0 次黑屏/白屏/崩溃）
- [ ] **剪贴板 copy/paste** 在 macOS + Ubuntu Wayland + Ubuntu X11 全部工作（包含中文字符）
- [ ] **IME 完整测试**：中文拼音 + 日文罗马字 双语种，不丢字，光标位置正确（三平台各录屏 3 个样例）
- [ ] **Bundle 大小**：macOS dmg < 30MB、Ubuntu AppImage < 40MB、deb < 40MB
- [ ] **Tauri plugin smoke test** 三个 plugin 在三平台均通过：
  - [ ] `tauri-plugin-clipboard-manager`（读写 + 中文字符）
  - [ ] `tauri-plugin-fs`（读写用户目录文件）
  - [ ] `tauri-plugin-updater`（假 update manifest URL 能发起请求并解析响应）
- [ ] **ADR 草稿**：`docs/adr/ADR-002-desktop-framework.md`（Phase 3 建立 ADR 目录前，暂以 `implementation-plan.md §3.1` 内嵌 changelog 记录）
- [ ] **`CLAUDE.md` 决策表 #12 更新**：B 栏 → A 栏，注明 "Spike W0 D2 hard-pass 通过"

## 🔀 Electron Fallback 路径

**如果 SPIKE-01 任一判据失败 → 本 Spike 转为 Electron 28+ fallback 验证**：

- [ ] Electron 28+ 在三平台空壳启动成功
- [ ] 冷启动 < 5s（Electron 允许比 Tauri 宽松）
- [ ] Bundle 大小 < 120MB（Electron 基线）
- [ ] IME + 剪贴板 在三平台工作
- [ ] 决策：`CLAUDE.md` #12 移入 A 栏，锁定 Electron 28+

**如果 Tauri 和 Electron 双失败** → 升级为 session-level 阻塞，通知 Arbiter（用户）决策

## ❌ 失败信号（Fail Signals）

Tauri 路径：

- 连续 10 次启动出现 ≥ 1 次黑屏 / 闪退 → Fail
- Bundle 大小超标（mac > 30MB 或 linux > 40MB）→ Fail
- 任一 plugin 在任一平台 smoke test 失败 → Fail

Electron 路径：

- 冷启动 > 5s → Fail（两种框架都不行时进入 Arbiter 仲裁）

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-02-tauri/` 完整硬通过测试代码 + 脚本（`.gitignore` 排除）
- [ ] **`docs/spikes/SPIKE-02-report.md`**（per-task；Phase 3 建立 `docs/spikes/` 目录后填写）
- [ ] 三平台 30 次启动日志汇总（csv / markdown 表）
- [ ] IME 录屏 × 6（3 平台 × 2 语种）
- [ ] Bundle 产物体积截图
- [ ] **ADR-002 草稿**：桌面框架锁定依据（Tauri 或 Electron）
- [ ] **`CLAUDE.md` 更新 PR**：决策表 #12 B → A

## 🛠 依赖资源（Resources Needed）

- SPIKE-01 产出的空壳项目作为起点
- `tauri-cli` 2.x + `tauri-plugin-*` 2.x
- （fallback 需要时）Electron 28+ + electron-builder
- 三平台截图 / 录屏工具

## ⚠️ 已知风险

- **R12**（CRITICAL）：至此消除
- **Plugin 兼容性**：Tauri 2 plugin 生态比 1.x 小，三选一 plugin 若在 Wayland 下有 bug 可能触发降级决策
- **Electron 包体积**：120MB 是底线，超过需重新评估（可能牺牲 auto-update 体积换取启动速度）

---

## 📝 Notes / 讨论

- 10 次启动测试必须在"冷启动"场景（关闭 app + 清 cache 后重开），不是"热启动"
- Plugin smoke test 用最小闭环：写一个 `fs.writeTextFile` + `clipboard.writeText` + `updater.check` 的组合
- 如果 Wayland 下 IME 在 `ibus` 失败、`fcitx5` 通过，仍算通过（Linux 用户可自由切换 IME 框架），但需在 ADR 里注明"推荐 fcitx5"

## 🔗 相关

- ADR：`docs/adr/ADR-002-desktop-framework.md`
- 对应 `CLAUDE.md` 决策表：**#12 桌面框架**（Day 2 结束后移入 A 栏）
- `implementation-plan.md` 章节：§3.1.1 · §附录 A D1-D2 · §9 R12
- 上游：SPIKE-01
- 下游：SPIKE-03..06

---

**填写完毕后自审**：

1. **递归完备性**：6 项 §3.1.1 判据 + 通过/失败双路径 + ADR 产出 ✅
2. **反向场景**：Tauri fail → Electron；Electron fail → Arbiter 仲裁 ✅
3. **边界适用性**：三平台显式测试 ✅
4. **YAGNI**：不加任何业务代码，只验证框架基线能力 ✅
