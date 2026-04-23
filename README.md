<!--
  README — 开发中（未发布）
  当前版本记录仓库现状与高层定位，不展开 session 级执行细节。
-->

# Vibestation

> 给 Claude CLI / Codex CLI 用户的多 Tab 终端 + JetBrains 级 Git 工作台。
> 一个窗口管多项目，每个 Tab 一个 CLI 会话，右栏看 Git，不再为了看 commit 打开一堆 IDE。

**项目状态**：开发中 · 尚未发布可用二进制 · 终端主链已完成 `MVP-04` Phase A/B/C/E/F，Git 读链已完成 `MVP-07` + `MVP-08` Phase A/B，当前主线为 `MVP-08` Phase C（Diff 视图前端）。

---

## 仓库结构

```
vibestation/
├── LICENSE                 Apache License 2.0
├── NOTICE                  Apache 2.0 归属声明
├── README.md               本文件
├── .gitignore              Rust / Node / Tauri / OS
├── crates/
│   ├── app/                            Tauri 启动层 / IPC / permissions / capabilities
│   └── core/                           业务核心（workspace / PTY / git / diff / layout）
├── web/
│   ├── src/                            SolidJS 前端（Terminal / Git Log / Git Status）
│   └── package.json
├── docs/
│   ├── PROGRESS.md                     当前进度 / 下一步 / 滚动窗口
│   ├── SESSION-STARTUP.md              人类启动手册
│   ├── tasks/                          task spec 索引与实施规格
│   ├── adr/                            accepted ADR
│   ├── runtime-evidence/               runtime 截图 / 指标记录
│   ├── implementation-plan.md          v2 实施计划（14 章 + 附录）
│   ├── codex-review-and-response.md    Codex 独立评审与应对
│   └── tech-research.md                CodexMonitor / lapce / gitui 预研
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
| 技术栈 | **Tauri 2 + Rust + SolidJS + xterm.js** |
| 持久化 | **rusqlite 0.31+ + r2d2_sqlite** |
| Git 栈 | **git2 0.20 写 + gix 0.70 读** |
| 平台 | **macOS-first**；Ubuntu 24 为低优先级补测项；Windows 11 推到 v0.4 |
| 视觉方向 | Calm Studio（柔和 oklch + Inter + JetBrains Mono 双字体）|

## 路线图（高层）

| 里程碑 | 周 | 内容 |
|--------|----|------|
| Spike W0（已完成） | 1 周 | Tauri / PTY / Git / 存储 / CLI 实机验证与 ADR 锁定 |
| v0.1 MVP（进行中） | +12 周 | 多 Tab 终端 · Git log/status 只读 · Commit · 基础 Diff · 单层 Pane · 配置导入 · 崩溃恢复 · macOS-first 打包发布 |
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
