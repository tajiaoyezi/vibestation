---
id: MVP-08
type: mvp
title: Diff 基础视图（自绘）+ Git Status 只读面板
status: draft
owner:
phase: W9-W10
depends_on: ["MVP-07"]
blocks: ["MVP-09"]
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §3.1（Diff 自建）
risk_ref:
reviewer:
---

# MVP-08: Diff 基础视图 + Git Status 只读

> **状态**：`draft`
> **依赖**：MVP-07（commit 详情触发 Diff）
> **阻塞**：MVP-09（Stage/Unstage 基于 Status 面板）

---

## 🎯 目标（Goal）

实现**自绘** Diff 视图（基础行对比，**不用 Monaco**），+ Bottom Panel 的 Git Status 只读面板（staged / unstaged / untracked 分组）。

## 📖 背景（Context）

- `CLAUDE.md` #7（A 栏）：Diff 渲染 = 自建（`diff` crate + Canvas/HTML），**不用 Monaco**（Monaco 3MB 会爆 bundle size 预算）
- MVP 不做复杂语法高亮（v0.3+），只做 added/removed/modified 3 色行对比
- Status 面板基于 git2 `statuses()` API

---

## 🎨 功能范围（Scope）

**Do**：
- Diff 视图打开触发源：
  - MVP-07 Commit 详情点文件 → 该 commit vs parent 的 diff
  - MVP-08 Status 面板点文件 → working tree vs index（unstaged）或 index vs HEAD（staged）
- Diff 视图渲染：
  - 左右 split（old / new）或 unified（单列标 +/-）
  - 用户可切换 split / unified
  - 行号 + 颜色（绿 added / 红 removed / 灰 unchanged context）
  - **无语法高亮**（纯行对比）
  - 二进制文件 → 显示 "Binary file, X bytes changed"
  - 大文件（>1MB）→ 提示 + 按钮 "Show anyway"
- Git Status Bottom Panel：
  - 分组：staged / unstaged / untracked
  - 每行：文件路径 + 状态 icon（M/A/D/R/?）+ 加减行数（staged 有，untracked 无）
  - 点击文件 → 打开对应 Diff 视图
  - 刷新按钮 + 自动监听（fs watch 或 polling 2s）

**Don't**：
- 语法高亮（v0.3+）
- Diff 编辑功能（Stage hunk / Stash）（v0.2/v0.3+）
- 3-way merge 视图（v0.3+）
- Rename 检测高级 UI（基础检测可做，复杂 UI v0.3+）

## 🖼 UI 引用

- Bottom Panel Status：`design/directions/1-calm-studio.html` 底部面板区
- Diff 视图：主区 Tab（类型 `diff`），split 视图左右 50/50，unified 视图单列

## ✅ Acceptance

### A. Diff 渲染

- [ ] `diff` crate 计算 line-level diff（Myers 算法）
- [ ] 渲染用 HTML（性能足够）或 Canvas（若 HTML 慢）
- [ ] 添加行：绿底黑字 · 删除行：红底黑字 · 未变上下文：默认色
- [ ] 左右 split 和 unified 切换 toggle
- [ ] 行号列显示原文 / 新文行号，对齐
- [ ] 大文件（>10k 行）可流畅滚动（60FPS）

### B. Diff 来源

- [ ] Commit 详情点文件 → `git diff <commit>^ <commit> -- <file>`
- [ ] Unstaged 点文件 → `git diff -- <file>`（working vs index）
- [ ] Staged 点文件 → `git diff --cached -- <file>`（index vs HEAD）

### C. Git Status 面板

- [ ] Bottom Panel toggle 显示 Git Status
- [ ] 3 分组标题：Staged (N) / Unstaged (N) / Untracked (N)
- [ ] 每组可折叠
- [ ] 每行：icon + 文件路径（相对 repo root）+ 加减行数（staged/unstaged 有）
- [ ] 点击文件 → 主区新开 Diff Tab

### D. 刷新

- [ ] 刷新按钮手动触发
- [ ] fs watch 或 polling 2s（三平台差异性处理，Linux inotify / macOS FSEvents）
- [ ] 刷新期间不阻塞 UI

### E. 边界 / 错误

- [ ] 二进制文件：Diff 视图显示 "Binary file, X bytes changed"
- [ ] 大文件 > 1MB：显示 "Large file, click to load"
- [ ] 文件超 10 万行：禁止加载 + 提示用户用 CLI
- [ ] git repo 破损 → Status 面板显示 error 状态

### F. 性能

- [ ] Status 面板列出 1000 文件 < 200ms
- [ ] Diff 打开 1k 行文件 < 200ms
- [ ] Diff 打开 10k 行文件 < 1s
- [ ] fs watch 延迟 < 500ms

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | diff 算法 + 文件类型判定（binary / text）|
| 集成 | git2 statuses API + fs watch |
| E2E | 改文件 → Status 刷新 → 点开 Diff |
| 性能 | 大文件 fixture（1k / 10k / 100k 行）|
| 视觉回归 | Diff 三色样式 split / unified |

## 💾 数据模型变更

无新 table。Diff 结果不缓存（每次实时计算）。

## ⚠️ 已知风险

- **大文件性能**：Canvas 渲染方案若需要则 fallback
- **Rename 检测**：git2 支持但结果可能误判，UI 上保守显示 `old_name → new_name`
- **fs watch 跨平台**：`notify` crate 抽象三平台，但 macOS FSEvents 有 2s 延迟下限

## 📝 Notes

- MVP-08 用 `similar` 或 `diff` crate（Rust）计算，前端只做渲染
- 选 HTML 渲染优先（开发快 + a11y 好），若性能不足再切 Canvas（记录到 ADR）

## 🔗 相关

- `CLAUDE.md` #7 Diff 自建
- 上游：MVP-07
- 下游：MVP-09
- `implementation-plan.md` §10.1 · §3.1

---

**自审四问**：1. Diff + Status + 刷新 + 边界覆盖 ✅ · 2. 大文件 / binary / 损坏 graceful ✅ · 3. 三平台 fs watch 差异化 ✅ · 4. 语法高亮 / 编辑 / merge UI 都推后 ✅
