# ADR-003: PTY 架构 = portable-pty + 共享读线程 + mpsc（pending SPIKE-05）

**状态**：**proposed**（pending [SPIKE-05](../tasks/SPIKE-05-pty-multi-tab.md) 通过后升级为 accepted）
**日期**：2026-04-18（Phase 1 默认选 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审 · 待 SPIKE-05 Arbiter 仲裁（若失败）
**对应 `CLAUDE.md` 决策表**：#15（B 档，Spike 后锁定）
**对应 Spike**：[SPIKE-05](../tasks/SPIKE-05-pty-multi-tab.md)

---

## 背景与问题

多 Tab 终端是 Vibestation 核心卖点之一（vs Claude Desktop 单窗口）。MVP 目标 4 Tab 并存 · v0.2+ 20 Tab。

核心技术选择：
- **PTY 库**：哪个 Rust crate 支持 fork + openpty + SIGWINCH + 跨平台（mac + linux）
- **读取架构**：每 PTY 一个 OS 线程 · vs · 共享读线程（mio / epoll）poll 所有 PTY fd

坏的失败模式：
- **全局 stall**：单慢消费者让共享读线程卡住 · 所有 Tab 卡顿（Codex PR #10 F2 警告）
- **Head-of-line blocking**：后端 IPC queue 满 / hidden tab throttle → cascade 所有 Tab（Codex PR #10 F3）
- **Unbounded memory**：mpsc 无背压 · 慢消费者导致 OOM

## 决策驱动因素

- **D1 · 多 Tab 性能**：4 Tab 并存 20 MB/s 总吞吐 · 主线程阻塞 ≤ 16ms（60fps）
- **D2 · 内存有界**：一个 Tab 的慢消费者不能让总 RSS 无界增长
- **D3 · 架构隔离**：一个 Tab 的问题不能 cascade 其他 Tab（HOL blocking）
- **D4 · 跨平台**：mac Darwin PTY + Linux glibc/musl PTY + Wayland/X11 兼容

## 考虑的选项

### PTY 库

- **portable-pty**（wezterm 维护）：成熟 · 三平台支持 · 被 zed / wezterm 实战使用
- **pty-process**：纯 async · 较新 · 生态较小
- **alacritty_terminal**：含 terminal emulator · overkill（我们用 xterm.js 做 emulator）

### 读取架构

- **A · 每 session 一 OS 线程**：简单 · 线程开销 2MB × 20 Tab = 40MB（可接受）· 不存在 HOL
- **B · 共享读线程（mio/epoll）+ mpsc**：线程数少（1）· 可能 HOL（必须验证）· 主流实现
- **C · async runtime（tokio）**：更灵活 · 但 PTY 的阻塞 I/O 语义不好与 tokio 协作

## 决策

**选择（proposed · pending SPIKE-05）**：
- **PTY 库**：`portable-pty 0.8+`
- **读取架构**：**B · 共享读线程 + mpsc bounded channel**（drop-oldest / drop-newest 策略二选一，**禁止 block-producer**）
- **Fallback**：若 SPIKE-05 的 B.4 一慢拖全部测试失败 → 切到 **A · 每 session 一线程**

**理由**：
1. portable-pty 在 wezterm / zed 实战验证 · 三平台兼容度最高
2. 共享读线程资源消耗低 · 若能过 HOL 测试则是首选架构
3. **必须 Spike 验证**的项：Codex PR #10 F2/F3 指出共享读线程架构下 HOL 风险实际存在 · 不允许"文档层通过"

## 后果

### 正面

- **资源消耗低**：共享读线程架构 · 1 个 OS 线程 poll 所有 PTY fd · 比每 session 一线程省 40MB（20 Tab 场景）
- **生态成熟**：portable-pty 跨平台 · SIGWINCH / 窗口 resize 直接可用
- **可降级**：若共享读线程失败 → per-session 架构是明确的 fallback · 用户几乎无感（仅内存 +40MB）

### 负面

- **Spike 复杂度**：SPIKE-05 B.4 必须覆盖 3 HOL 场景（前端 render 慢 · 后端 IPC queue 满 · hidden-tab throttle）· 测试用例非琐碎
- **drop 策略权衡**：drop-oldest 保最新输出 · drop-newest 保历史 · 必须代码层显式常量标注

### 风险

- **SPIKE-05 任一 HOL 子场景失败** → 切 per-session（fallback 明确）
- **portable-pty 平台 bug**（如 macOS 特殊 PTY 行为）→ 调查修复 · 或换 `pty-process`（成本：不大）

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.1（技术栈选型）· §附录 A D5（Spike 计划）
- 对应风险：无专属 R 号 · 但共享读线程失败会触发 §3.1 的 fallback 路径

## 相关

- `CLAUDE.md` 决策表：#15
- Spike：[SPIKE-05 portable-pty 单读线程 + mpsc + xterm 4-Tab 压测](../tasks/SPIKE-05-pty-multi-tab.md)
- Codex 对抗性教训：PR #3 Round 1 F2 · PR #7 F2 · PR #10 F2/F3

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code · status: proposed · 等 SPIKE-05 通过后改为 accepted
