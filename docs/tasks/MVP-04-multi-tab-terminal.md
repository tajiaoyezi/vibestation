---
id: MVP-04
type: mvp
title: 多 Tab 终端（PTY + xterm.js + Shell/CLI 兼容）
status: draft
owner:
phase: W4-W6
depends_on: ["MVP-03", "SPIKE-05", "SPIKE-06"]
blocks: ["MVP-05", "MVP-06"]
blocked_by: []
blocked_note:
estimate: 8d
plan_ref: implementation-plan.md §10.1 · §10.6（终端正确性矩阵）· §附录 A D5
risk_ref:
reviewer:
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

## 🖼 UI 引用

- 主区 Tab bar：`design/directions/1-calm-studio.html` 主内容区顶部
- Tab 样式：紧凑，带 close X，active tab 用主色下边框
- 字体：JetBrains Mono（原型定义）

## ✅ Acceptance

### A. Tab 基础

- [ ] 新 workspace 打开默认创建 1 个 Tab（运行默认 shell）
- [ ] `⌘T` 新建 Tab → 新 PTY 进程 + 新 xterm 实例
- [ ] `⌘W` 关 Tab（最后一个 Tab 关闭 → 询问"关闭 workspace?"）
- [ ] 双击 Tab 标题 → 重命名输入框
- [ ] Tab 切换 100ms 内完成（对齐 `§10.2` 性能目标）

### B. PTY 正确性

- [ ] 每 Tab 独立 PTY（不共享）
- [ ] Shell 启动：macOS zsh / Linux bash（从设置读取）
- [ ] 环境变量：继承 user shell env（`fix-path-env` 解决 macOS GUI app PATH 问题）
- [ ] 信号传递：Ctrl+C 中断、Ctrl+D EOF、Ctrl+Z 暂停
- [ ] resize：窗口尺寸变化 → PTY SIGWINCH，`htop` / `vim` 即时重排

### C. 程序兼容矩阵（`§10.6`）

- [ ] zsh 交互：Tab 补全、历史、ANSI 颜色
- [ ] vim：基础编辑 + `/`搜索 + `:wq` 保存退出
- [ ] htop：UI 渲染正常（5Hz+ 刷新）
- [ ] yes：10s 连续输出不卡顿、不丢帧（对齐 SPIKE-05 A.1 但 MVP 场景）
- [ ] tmux：基础 session 创建 + 切 window
- [ ] Claude CLI：启动 + 登录 + 对话（SPIKE-06 已验）
- [ ] Codex CLI：启动 + 登录 + 对话（SPIKE-06 已验）

### D. 粘贴保护

- [ ] 粘贴内容含换行（多行命令）→ 弹出确认对话框
- [ ] 对话框显示将要粘贴的前 5 行预览
- [ ] 可选"不再提示本 session"

### E. 性能（对齐 `§10.2`）

- [ ] 10 Tab 并存，总内存 < 500MB（Activity Monitor 测）
- [ ] 切 Tab 延迟 < 50ms（Playwright 记录）
- [ ] 单 Tab 吞吐 ≥ 20MB/s（`yes | pv` 测）
- [ ] 主线程阻塞 ≤ 16ms（60FPS 达标）

### F. 错误处理

- [ ] Shell 进程异常退出 → Tab 显示"Process exited (code X). Press Enter to restart"
- [ ] PTY open 失败 → 明确报错，不崩溃
- [ ] xterm renderer fallback：webgl → canvas → dom（逐级降级）

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
struct TabState {
    tab_id: String,              // UUID
    workspace_id: String,        // FK
    name: String,                // 用户可改
    shell: String,               // "zsh" / "bash" / etc
    cwd: String,                 // 当前工作目录
    scroll_back: Vec<String>,    // 最多保留 10k 行
    created_at: i64,
}
```

## ⚠️ 已知风险

- **PTY 架构 fallback（SPIKE-05 B.3）**：若单读线程失败 → 改为一 session 一线程 → 10 Tab 资源上升 ~40MB
- **Wayland IME**：Wayland 下 IME 切换可能和 xterm focus 冲突 → 三平台分开测
- **CLI 中断残帧（SPIKE-06 A.2）**：Ctrl+C Claude CLI 流式输出中途 → 检查残帧是否污染下条 prompt

## 📝 Notes

- MVP-04 不实现 tmux control mode（看 tmux 作为普通程序跑即可）
- Claude/Codex CLI 的协议解析留给 v1.0 AI-Aware（SPIKE-07 parser spike）

## 🔗 相关

- `CLAUDE.md` #15 · #6 · ⚠️ CLI 警告（R1）
- SPIKE-05（PTY 架构）· SPIKE-06（CLI 实机 + fix-path-env）
- `implementation-plan.md` §10.6 终端正确性矩阵
- 上游：MVP-03 · SPIKE-05 · SPIKE-06
- 下游：MVP-05 · MVP-06

---

**自审四问**：1. 矩阵覆盖 ✅ · 2. PTY fallback 已定 ✅ · 3. 三平台显式 ✅ · 4. tmux control mode / AI 联动 都推后 ✅
