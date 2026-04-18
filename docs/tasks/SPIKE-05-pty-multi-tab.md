---
id: SPIKE-05
type: spike
title: portable-pty 单读线程 + mpsc + xterm 4-Tab 压测
status: draft
owner:
phase: W0-D5
depends_on: ["SPIKE-02"]
blocks: ["SPIKE-06"]
estimate: 1d
plan_ref: implementation-plan.md §附录 A D5 · §3.1
risk_ref:
reviewer:
---

# SPIKE-05: portable-pty + xterm 多 Tab 压测

> **状态**：`draft`
> **依赖**：SPIKE-02（桌面框架已锁定）/ **阻塞**：SPIKE-06（Claude CLI 实机要跑在 PTY 里）
> **战略依据**：[`implementation-plan.md §附录 A D5`](../implementation-plan.md)

---

## 🎯 目标（Goal）

验证 **portable-pty + 单读线程 + mpsc 分发 + xterm.js 5.5** 组合在 4 Tab 并存 + 高吞吐场景下的表现，消除"多 Tab 瓶颈"风险，确认 PTY 架构选型。

## 📖 背景（Context）

- `CLAUDE.md` 决策表 **#15 PTY 方案 = portable-pty + 单读线程 + mpsc（默认）**，Spike W0 D3→D5 验证（v2 附录 A 改到 D5）
- **多 Tab 是 Vibestation 核心卖点之一**（vs Claude Desktop 单窗口）
- 常见多 Tab 失败模式：单线程读多个 PTY 互相阻塞、mpsc channel 满、xterm 渲染卡顿
- **Fallback**：如单读线程瓶颈 → 每个 session 一线程（成本 × 4，但简化实现）

---

## ✅ 通过标准（Pass Criteria）

> ⚠️ **PTY 架构真正锁定 = A（短时压测）+ B（长时/背压/内存）都通过**。只过 A 不能锁定——慢消费者场景未验证时 mpsc unbounded channel 会在真实使用里 OOM。Codex PR #3 Round 1 Finding 2 教训。

### A. 短时压测（当前·阻塞项）

- [ ] **单 Tab 高吞吐压测**：
  - [ ] Tab 里 `yes`（无限输出）连续运行 10s 不卡顿、不丢帧
  - [ ] Tab 里 `htop` 运行 10s，UI 更新正常（5Hz+）
  - [ ] 主线程（Tauri webview）event loop 阻塞 ≤ 16ms（60 FPS 达标）
- [ ] **4 Tab 并存压测**：
  - [ ] 4 个 Tab 同时 `yes` 10s，每个 Tab 都流畅，无互相拖慢
  - [ ] 切换 Tab 时当前 Tab 继续滚动（不冻结）
  - [ ] 4 个 Tab 各自 scroll back 独立，不串数据
- [ ] **PTY 吞吐 benchmark**：
  - [ ] 单 Tab 吞吐 ≥ 20 MB/s（`yes | pv` 或等价测试）
  - [ ] 4 Tab 并存总吞吐 ≥ 40 MB/s（不必线性，但不可 < 2 Tab 总吞吐）

### B. 长时 Soak + 背压 + 内存有界（阻塞项 · Codex PR #3 加入）

> 本 Spike 文档自述"mpsc unbounded channel 满时需要 bounded + back-pressure 设计"。该风险必须在锁定前验证：任一 Tab/renderer 消费慢于 PTY 产出时，**输出不可无界堆积**。

**B.1 · 10 分钟慢消费者 soak test**
- [ ] 4 Tab 全部 `yes` 持续 10 分钟
- [ ] **人为制造慢消费者**：在 xterm 前端加 50ms 人工延迟（模拟前端卡顿）
- [ ] 期间记录：
  - [ ] mpsc channel 队列深度 over time（最大值 + 均值）
  - [ ] RSS 内存 over time（每 10s 采样）
  - [ ] PTY 数据丢弃量 / 丢弃比例（如启用了 bounded + drop）
- [ ] **通过门槛**：
  - [ ] channel 队列深度上限可证有界（如 ≤ 10,000 条或等价上限）
  - [ ] RSS 增长 ≤ 100MB（10 分钟内不可无界）
  - [ ] 总 RSS ≤ 500MB（4 Tab + yes）

**B.2 · 隐藏 tab 场景**（常见失效模式）
- [ ] 4 Tab 都 `yes`，3 个 Tab 隐藏（不显示 xterm），只 1 个 active
- [ ] 持续 5 分钟
- [ ] 隐藏 tab 的 PTY 数据按**明确策略**处理（可选之一）：
  - (a) 持续渲染到 off-screen buffer（bounded size）
  - (b) 限制 scroll back 长度
  - (c) bounded channel + 显式丢弃（可接受 · 需在 UI 提示用户）
- [ ] **不允许**：unbounded 内存堆积（即使是"后台 tab"）

**B.3 · 架构必备条件**（阻塞锁定的硬要求）
- [ ] mpsc channel **必须是 bounded**（非 unbounded）
- [ ] 队列满时有明确策略：阻塞生产者 / 丢弃最老 / 丢弃最新——**必须显式记录所选策略**
- [ ] 实现代码里 channel 容量是配置项（不是 hardcoded）

### C. 正确性验证（当前）

- [ ] **resize 正确**：调整 Tab 大小后，PTY `SIGWINCH` 正确传达，`htop` / `vim` 即时重排
- [ ] **进程退出清理**：Tab 内 `exit` 后 PTY 资源（fd + thread）释放，无泄漏
- [ ] 结论写入 **ADR-003 草稿**（Phase 3 后建立）

## ❌ 失败信号（Fail Signals）

短时（A）：

- 单 Tab `yes` 卡顿（主线程阻塞 > 50ms）→ mpsc / 渲染路径有瓶颈
- 4 Tab 互相拖慢（总吞吐 < 2 Tab 总吞吐）→ 单读线程瓶颈触发 fallback
- 切换 Tab 冻结其他 Tab → 单读线程实现错误（scroll back 可能串流）
- `SIGWINCH` 不传达 → portable-pty 在目标平台有 bug

**长时/背压（B · Codex 加入）**：

- **B.1 soak 期间 RSS 增长 > 200MB** → mpsc unbounded 或 scroll back 无界，不可锁定
- **B.1 channel 深度单调增长到 > 100,000 条** → 生产快于消费且无背压
- **B.2 隐藏 tab 5 分钟 RSS 增长 > 50MB** → off-screen 处理策略缺失
- **B.3 channel 仍是 unbounded** → **硬拒绝锁定**，即使性能看似通过

## 🔀 Fallback 方案

**通过** → `CLAUDE.md` #15 B → A，锁定 portable-pty + 单读线程 + mpsc
**单读瓶颈失败** → 切换到"每 session 一线程"，评估 Tab 数上限（MVP 用户很少开 8+ Tab，20 线程可接受）
**portable-pty 平台 bug** → 调查能否修 / 换 `pty-process` 或 `alacritty_terminal` crate

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-05-pty/`：完整 PTY + xterm 集成 demo
- [ ] **`docs/spikes/SPIKE-05-report.md`** PTY benchmark 数据表（per-task）
- [ ] 4 Tab 并存压测录屏（`yes` × 4 + htop × 1）
- [ ] 主线程阻塞时间火焰图
- [ ] **ADR-003 草稿**：PTY 架构决策
- [ ] `CLAUDE.md` 决策表 #15 更新 PR（通过后）

## 🛠 依赖资源（Resources Needed）

- SPIKE-02 产出的空壳 Tauri 项目
- `portable-pty` 0.8+ + `xterm` 5.5+ + `xterm-addon-*`（fit / web-links）
- 工具：`pv` / `htop`（benchmark）
- 测试机三平台各跑一次（macOS + Wayland + X11）

## ⚠️ 已知风险

- **单读线程 + mpsc 架构** 如果 mpsc unbounded channel 满（极端吞吐）→ 需要 bounded + back-pressure 设计
- **xterm.js 5.5 的 webgl addon**：renderer 选择（canvas vs webgl vs dom）在三平台表现可能不同
- **Wayland 下 xterm 字体渲染**：subpixel antialiasing 可能有差异（视觉问题不算通过障碍）

---

## 📝 Notes / 讨论

- 单读线程架构：**1 个 native thread 负责 poll 所有 PTY 的 fd**（`mio` 或等价），数据通过 mpsc 分发到前端 channel
- Fallback 的"每 session 一线程"成本：Rust 线程初始栈 2MB，20 Tab 约 40MB 额外内存（可接受）
- `yes` 压测虽然不真实，但能暴露 channel 饱和问题；实际用户负载多是 interactive（htop / vim），本 Spike 两类都测

## 🔗 相关

- ADR：`docs/adr/ADR-003-pty-architecture.md`
- 对应 `CLAUDE.md` 决策表：**#15 PTY 方案**
- `implementation-plan.md` 章节：§附录 A D5 · §3.1
- 上游：SPIKE-02
- 下游：SPIKE-06（Claude CLI 要跑在本 Spike 验证的 PTY 里）

---

**填写完毕后自审**：

1. **递归完备性**：单 Tab + 4 Tab + resize + 清理 全覆盖 ✅
2. **反向场景**：单读瓶颈 → 一 session 一线程；portable-pty bug → 换 crate ✅
3. **边界适用性**：三平台各跑一次 ✅
4. **YAGNI**：不做 16 Tab 压测（MVP 用户典型 3-4 Tab） ✅
