---
id: MVP-17
type: mvp
title: 外部终端弹出（Pop to External）+ Pane Detach
status: draft
owner:
phase: v0.3
depends_on: ["MVP-14"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1 · §5.3
risk_ref:
reviewer:
---

# MVP-17: Pop to External + Pane Detach

> **状态**：`draft`（v0.3 · 占位 spec，非 MVP v0.1 范围）
> **依赖**：MVP-14（Pane 高级布局已就绪）
> **战略依据**：[`implementation-plan.md §10.1 砍到 v0.3`](../implementation-plan.md) · [`§5.3`](../implementation-plan.md)

---

## 🎯 目标（Goal）

两个"分离"操作：
1. **Pop to External**：把某个 Tab 的终端内容弹到用户系统的外部终端（Ghostty / iTerm2 / Terminal.app / Alacritty）继续操作
2. **Pane Detach**：把某个 Pane 弹为独立应用窗口（仍在 Vibestation 进程内）

## 📖 背景（Context）

- Pop to External 需求：有时用户想用外部 IME / 外部 tmux / 外部特殊配置，一键弹出节省时间
- Pane Detach：多屏用户想把 Git Log 放到左屏，终端放到右屏（不依赖 OS 多窗口管理）
- 占位 spec

---

## 🎨 功能范围（Scope）

**Do**（v0.3 启动后详化）：
- **Pop to External**：
  - 识别用户系统装的终端（按 `TERM_PROGRAM` 或配置表）
  - 用 `open -a Ghostty` / `gnome-terminal --` 等命令启动
  - 传递当前 workspace 的 cwd + shell + env
  - 当前 Tab 的 scrollback buffer 不跟随（技术限制，明确告知用户）
- **Pane Detach**：
  - 右键菜单 "Detach Pane"
  - 生成新 Tauri WebviewWindow，内容为该 Pane
  - 关闭 detached window 时内容重新吸回原 Pane 位置
  - 多 detached window 间可拖回原窗口

**Don't**（明确不做）：
- Detached window 间的 Pane 互拖（v1.0）
- 跨窗口的 global 快捷键（v1.0）

## 🖼 UI 引用（UI Reference）

- 原型：`design/directions/1-calm-studio.html` Pane 右键菜单
- 详化时补截图到 `docs/tasks/assets/MVP-17/`

## ✅ Acceptance（v0.3 启动后详化）

骨架：
- [ ] Pop to External 支持 3 种终端（Ghostty / iTerm2 / gnome-terminal）
- [ ] Detached window 关闭后 Pane 恢复到原位置（layout tree 还原）
- [ ] 两个操作有清晰的快捷键（`⌘⇧O` Pop / `⌘⇧D` Detach）
- [ ] Detached window 的关闭时内存释放 ≤ 10MB 残留
- [ ] 跨平台（macOS + Ubuntu）均可用

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元 | 外部终端 command 构造 | `cargo test` |
| 集成 | Tauri WebviewWindow 生命周期 | `cargo test --features integration` |
| E2E | Playwright 模拟 detach / re-attach | Playwright |

## 💾 数据模型变更

- 无新表；detached window 状态运行时维护

---

## 📝 Notes / 讨论

- Pop to External 的 env 传递要避免泄漏 API key（只传安全白名单）
- 占位 spec

## 🔗 相关

- `implementation-plan.md` §10.1 · §5.3
- 上游：MVP-14
- 下游：无

---

**自审**（占位 spec 对应简化版）：

1. **递归完备性**：Pop + Detach 双操作齐全 ✅
2. **反向场景**：scrollback 不跟随 / detach 关闭恢复已说 ✅
3. **边界适用性**：3 种终端 × 2 平台 ✅
4. **YAGNI**：跨窗口互拖 / global 快捷键留给 v1.0 ✅
