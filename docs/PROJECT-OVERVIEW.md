# Project Overview · 项目概览

> 开发者向项目结构、规划成果、锁定决策、非目标与贡献入口。
> 本文件从 README 迁移而来，保留原文措辞；仅调整了 markdown 链接的相对路径。

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

## 规划成果（规划期交付物）

| 文档                                                           | 内容                                                                                                         | 行数         |
| -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ------------ |
| [`implementation-plan.md`](implementation-plan.md)             | 产品定位 / 4 crate → 2 crate 架构 / 数据模型 / IPC / 30 风险 / 降级树 / 终端正确性矩阵 / 安全边界 / 分发运营 | 14 章 + 附录 |
| [`codex-review-and-response.md`](codex-review-and-response.md) | Codex 独立评审（7 CRITICAL · 12 HIGH · 5 MEDIUM · 13 强烈反对）+ Claude 元评论 + 用户 4 项决策               | 157          |
| [`tech-research.md`](tech-research.md)                         | 三项目深度预研与可借鉴点（PTY 多会话表 / AsyncLog 双速率 / logwalker 时间堆 / workspace 配置）               | —            |

## 当前已锁定决策

| 维度     | 值                                                                |
| -------- | ----------------------------------------------------------------- |
| 许可证   | **Apache License 2.0**（无 CLA）                                  |
| 技术栈   | **Tauri 2 + Rust + SolidJS + xterm.js**                           |
| 持久化   | **rusqlite 0.31+ + r2d2_sqlite**                                  |
| Git 栈   | **git2 0.20 写 + gix 0.70 读**                                    |
| 平台     | **macOS-first**；Ubuntu 24 为低优先级补测项；Windows 11 推到 v0.4 |
| 视觉方向 | Calm Studio（柔和 oklch + Inter + JetBrains Mono 双字体）         |

## 非目标

MVP 明确不做：Windows · 云同步 · 团队协作 · 插件市场 · 远程 / SSH / devcontainer · Git worktree/submodule/LFS 的高级支持。详见实施计划 §1.4。

## 贡献者与 Agent 入口

贡献流程已就绪。详见：

- [`AGENTS.md`](../AGENTS.md) · 任意 agent CLI 通用入口
- [`CLAUDE.md`](../CLAUDE.md) · 项目权威单文件入口（规则 / 决策 / 禁区 / 5 步 PR 流程）
- [`docs/PROGRESS.md`](PROGRESS.md) · 当前进度 / 下一步 / 滚动窗口
- [`docs/tasks/`](tasks/) · task spec 索引与实施规格
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) · 详细贡献指南
- [`CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md) · Contributor Covenant 2.1 中文版

**不要求贡献者签署 CLA**（Apache 2.0 本身已含 patent grant）。
