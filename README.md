<!--
  README — v0.0.0（规划期）
  v0.1 发布时会由 design + implementation-plan 的产出派生出正式 README。
  当前版本仅记录项目存在、定位与仓库结构，不对外宣传。
-->

# Vibestation

> 给 Claude CLI / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台。
> 一个窗口管多项目，每个 Tab 一个 CLI 会话，右栏看 Git，不再为了看 commit 打开一堆 IDE。

**项目状态**：规划期 · v0.0.0 · 尚未发布可用二进制 · **Pre-code Phase 1-4 已全部交付**（27 task spec + 10 ADR + 全套治理），等待启动 Spike Week 0 Day 1（SPIKE-01 Tauri 三平台空壳）。

---

## 仓库结构

```
vibestation/
├── LICENSE                 Apache License 2.0
├── NOTICE                  Apache 2.0 归属声明
├── README.md               本文件
├── .gitignore              Rust / Node / Tauri / OS
├── docs/
│   ├── implementation-plan.md          v2 实施计划（14 章 + 附录）
│   ├── codex-review-and-response.md    Codex 独立评审与应对
│   └── tech-research.md                 CodexMonitor / lapce / gitui 预研
└── design/
    ├── index.html          视觉方向总览（4 个方向）
    ├── directions/
    │   ├── 1-calm-studio.html      主风格（定稿）
    │   ├── 2-terminal-native.html
    │   ├── 3-codex-inspired.html
    │   └── 4-vscode-dense.html
    └── logos/
        ├── wordmark-a.svg
        └── mark.svg
```

## 快速查看当前设计

```bash
open design/index.html
```

打开 `design/directions/1-calm-studio.html` 可直接体验定稿方向的完整原型（双主题切换、可 toggle 工具窗口、Pane 分屏演示）。

## 规划成果（规划期交付物）

| 文档 | 内容 | 行数 |
|------|------|------|
| [`docs/implementation-plan.md`](docs/implementation-plan.md) | 产品定位 / 4 crate → 2 crate 架构 / 数据模型 / IPC / 30 风险 / 降级树 / 终端正确性矩阵 / 安全边界 / 分发运营 | 14 章 + 附录 |
| [`docs/codex-review-and-response.md`](docs/codex-review-and-response.md) | Codex 独立评审（7 CRITICAL · 12 HIGH · 5 MEDIUM · 13 强烈反对）+ Claude 元评论 + 用户 4 项决策 | 157 |
| [`docs/tech-research.md`](docs/tech-research.md) | 三项目深度预研与可借鉴点（PTY 多会话表 / AsyncLog 双速率 / logwalker 时间堆 / workspace 配置）| — |

## 当前已锁定决策

| 维度 | 值 |
|------|---|
| 许可证 | **Apache License 2.0**（无 CLA）|
| 技术栈 | Tauri 2 + Rust + SolidJS + xterm.js（Spike Day 2 硬通过后锁；失败回退 Electron 28+）|
| 持久化 | 默认 redb，Spike 后对比 rusqlite 再锁 |
| Git 栈 | 默认 git2；读路径性能不足时引入 gix |
| 平台 | MVP：macOS + Ubuntu 24（Wayland 必过）；Windows 11 推到 v0.4 |
| 视觉方向 | Calm Studio（柔和 oklch + Inter + JetBrains Mono 双字体）|

## 路线图（高层）

| 里程碑 | 周 | 内容 |
|--------|----|------|
| Spike W0 | 1 周 | Tauri Pass/Fail · PTY · 多 Tab · Claude CLI 实机 · git2 读 · git2 写 · 存储对比 benchmark |
| v0.1 MVP | +12 周 | 多 Tab 终端 · Git log/status 只读 · Commit · 基础 Diff · 单层 Pane · 配置导入 · 崩溃恢复 · macOS + Linux 签名打包 |
| v0.2 | +5 周 | Push/Pull/Fetch · Rail graph · 分支管理 · Pane 任意嵌套 |
| v0.3 | +5 周 | Rebase/Merge/Cherry-pick · 冲突解决 · Pop to External |
| v1.0 | +6-8 周 | 高级工作流能力（范围详见 [`implementation-plan.md`](docs/implementation-plan.md)）|

**总预算**：28-30 周 × 20-25 小时 ≈ 600-750 小时（含 20% buffer）。若投入减半，触发 [`docs/implementation-plan.md#105-降级树`](docs/implementation-plan.md) 降级策略。

## 贡献

贡献流程已就绪。详见：
- [`AGENTS.md`](AGENTS.md) · 任意 agent CLI 通用入口
- [`CLAUDE.md`](CLAUDE.md) · 项目权威单文件入口（规则 / 决策 / 禁区 / 5 步 PR 流程）
- [`CONTRIBUTING.md`](CONTRIBUTING.md) · 详细贡献指南
- [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) · Contributor Covenant 2.1 中文版

**不要求贡献者签署 CLA**（Apache 2.0 本身已含 patent grant）。

## 非目标

MVP 明确不做：Windows · 云同步 · 团队协作 · 插件市场 · 远程 / SSH / devcontainer · Git worktree/submodule/LFS 的高级支持。详见实施计划 §1.4。

## 许可证

Apache License 2.0 — 详见 [`LICENSE`](LICENSE)。
