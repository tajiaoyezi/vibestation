# Internal · 内部协作流程归档

> 本目录存放**仅供项目所有者 + AI agent 协作消费**的内部工作流文档。
> 外部贡献者**无需阅读**，对理解项目代码 / 路线图 / 决策**没有帮助**。

## 为什么单独一个 `internal/` 目录

vibestation 是 AI-native 开发项目（主 agent 驱动 · 多模型协作 · session-based ledger）。日常运营产生大量**只对 agent 协作有意义的元信息**：

- **session ledger**：每次工作 session 的 PR 时间线 / dispatch 决策 / 临时卡点 / 失败案例追溯
- **agent 协作失败档案**：哪个 agent 在什么场景下违反了什么硬约束 / 怎么修
- **新 agent 接手手册**：首次进项目的 5 步 onboarding / 常用命令 / 协作 FAQ
- **一次性 audit 报告**：某个时间点的横向项目状态盘点

这些文档**信息密度极高**但**外部读者完全看不懂上下文**——比如 "PR #410 self-review nit 闭合 · §F.3 fixture smoke" 这类术语对外部访客是噪音。

把它们集中到 `docs/internal/` 而不是直接删，是因为：

1. **agent 协作离不开**：下一个 session 启动 agent 需要读 ledger 续上下文
2. **决策追溯需要**：未来回顾"为什么 ADR-NNN 这么决策"时这些 ledger 是 audit trail
3. **不公开发布**但**保留 git history**

## 目录清单

| 文件 / 子目录                                | 用途                                                                       |
| -------------------------------------------- | -------------------------------------------------------------------------- |
| `SESSION-STARTUP.md`                         | 人类启动手册 · 新 agent 首次进项目的详细 onboarding（5 步 + Playbook + FAQ） |
| `dispatch-incidents.md`                      | dispatch prompt 16 条硬约束的事件源 · 反模式表 · 失败案例追溯              |
| `session-history/`                           | 每个工作 session 的 PR 时间线 / dispatch 决策 / 卡点归档（按 session-NN.md 切片） |
| `project-status-2026-04-22-session-16.md`    | session 16 的横向项目状态 audit 报告（一次性）                             |
| `session-12-audit-report-2026-04-20.md`      | session 12 audit 报告（一次性 · 含 M-1/M-2/M-3 findings）                  |

## 外部读者应该读哪里

如果你是来了解项目的：

- 项目当前状态 → [`docs/PROGRESS.md`](../PROGRESS.md)
- 项目概览 / 仓库结构 → [`docs/PROJECT-OVERVIEW.md`](../PROJECT-OVERVIEW.md)
- 快速上手 → [`docs/QUICKSTART.md`](../QUICKSTART.md)
- 架构决策 → [`docs/adr/`](../adr/)
- 任务索引 → [`docs/tasks/`](../tasks/)
- 贡献指南 → [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md)

## 维护约定

- 新增内部协作流程文档默认放本目录
- 不在 README.md / PROJECT-OVERVIEW.md / QUICKSTART.md 等开源向文档里反向引用本目录
- agent 协作文档（CLAUDE.md / .claude/rules/*）可以引用本目录
