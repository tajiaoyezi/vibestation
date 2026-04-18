# Vibestation · 新接手 Agent 介入开发可行性复核

> 评估时间：2026-04-18
> 评估目标：判断一个首次进入仓库、没有历史上下文的 agent，是否能快速介入当前项目的任务开发
> 评估范围：`CLAUDE.md` / `docs/PROGRESS.md` / `docs/SESSION-STARTUP.md` / `docs/tasks/` / `docs/adr/` / `docs/implementation-plan.md` / `README.md` / 视觉原型 / `.github/workflows/`

---

## 背景

仓库当前仍处于 **pre-code** 阶段，尚未进入实际 Rust + SolidJS 代码实现，但已经沉淀了较完整的规划、任务、ADR、原型和流程文档。

本复核不回答“文档多不多”，而是回答更具体的问题：

1. 新 agent 能否在较短时间内建立正确上下文
2. 新 agent 能否按仓库现有流程直接认领并推进任务
3. 文档之间是否一致，是否会把接手者带偏

---

## 结论

**结论分两层：**

- **如果标准是“快速理解项目并知道下一步做什么”**：**满足**
- **如果标准是“严格按当前仓库规则，零摩擦地直接认领并开始开发”**：**暂不完全满足**

综合判断：**项目具备较强的上下文接手能力，但还没有达到“无歧义开发就绪”**。  
建议评分：**7/10**。

更准确地说，这个仓库已经足以让新 agent 在 **30-60 分钟内建立可靠上下文**，但还存在几个会明显拖慢开工、甚至导致误判的缺口。

---

## 正向证据

### 1. 核心规划资产完整

以下材料已经足够支撑新 agent 快速理解产品与技术方向：

- `docs/implementation-plan.md`：有完整产品定位、技术架构、风险、Milestone、验收标准
- `docs/adr/`：10 份 ADR 已把永久锁定和待 Spike 锁定的关键决策固化
- `design/directions/1-calm-studio.html`：主视觉原型可直接帮助理解布局和交互意图
- `docs/tasks/`：SPIKE 和 MVP 任务已经拆到可执行 spec 粒度

### 2. 任务文档不是空壳

抽查 `SPIKE-01`、`MVP-01`、`SPIKE-07`、`MVP-15` 后可以确认：

- 目标、范围、验收、依赖、风险都已写出
- 不是只有标题级占位，很多任务已经具备实施骨架
- `node scripts/validate-task-spec.mjs` 已通过，**27 个 task spec frontmatter 全部合法**

这说明任务体系本身是可用的，不是摆设。

### 3. 基础治理已经具备雏形

仓库已经有：

- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `.github/PULL_REQUEST_TEMPLATE.md`
- `.github/workflows/task-spec-validator.yml`
- `.github/workflows/secret-scan.yml`
- `docs/BRANCH-PROTECTION.md`

从“项目治理基础设施是否存在”这个角度看，准备度是够的。

---

## 关键阻碍

### 1. 入口文档彼此冲突，首轮 onboarding 会被带偏

这是当前最大的实际问题。

`docs/PROGRESS.md` 已明确写出：

- Phase 1-4 **全部完成**
- Spike W0 **可启动**
- 下一步是 **SPIKE-01**

但 `docs/SESSION-STARTUP.md` 和 `CLAUDE.md` 仍保留旧阶段表述，例如：

- 仍写“仓库还没有代码，`docs/tasks/` / `docs/adr/` / `.github/` 待建立”
- 仍把“Phase 2 / 3 / 4 文档补齐”写成当前可执行动作

这意味着一个新 agent 即使按“先读入口文档”的路径进入，也会在最开始接收到**过期指令**，然后再靠 `PROGRESS.md` 去纠偏。

**影响**：

- 首次接手成本被抬高
- 容易误以为当前还不能进入 Spike
- 容易重复做已经完成的文档工作

### 2. 当前没有任何 `ready` 任务，无法按既定流程直接认领开发

仓库自己的流程定义是：

`draft -> ready -> in-progress -> done`

而当前实际状态是：

- `ready = 0`
- `draft = 27`

这意味着：

- 新 agent **可以快速理解任务**
- 但**不能严格按流程直接开始实施**
- 还需要先做一次 spec 独立评审 / 状态翻转，才能合规认领

所以当前更准确的状态不是“可直接开发”，而是：

**“可快速进入上下文，但正式开工前还差一步 task 准入动作。”**

### 3. 部分任务 spec 的战略引用失真，会削弱可追溯性

抽查后发现，多个任务引用了并不存在的 `implementation-plan.md §5.4`，例如：

- `docs/tasks/MVP-11-git-push-pull-fetch.md`
- `docs/tasks/MVP-12-commit-rail-graph.md`
- `docs/tasks/MVP-13-branch-crud.md`
- `docs/tasks/MVP-16-rebase-merge-cherrypick.md`

但 `implementation-plan.md` 在 `5.3.9` 之后直接进入 `6`，并没有 `5.4`。

另有个别引用明显像笔误，例如：

- `docs/tasks/SPIKE-04-storage-benchmark.md` 中的 `§512`

这类问题不影响“知道要做什么”，但会影响：

- 新 agent 回溯战略依据时的可信度
- 文档间的导航效率
- 后续 review / 追责 / scope 校验

### 4. README 和既有 readiness 评估偏乐观，不能再当权威结论使用

`README.md` 仍写：

- “即将进入 Spike Week 0”
- “正式进入 Spike 后再开放贡献，届时 `CONTRIBUTING.md` + `CODE_OF_CONDUCT.md` 会补齐”

但这两份文件实际上已经存在。

此外，此前同主题的 readiness 评估稿结论偏乐观，未充分考虑：

- 入口文档过期冲突
- `ready=0`
- `§5.4` 断链问题

因此旧结论不宜继续当作当前事实引用，本次文档已按实际仓库状态重写。

### 5. 分支保护仍未实际应用，流程约束还停留在“文档层”

`docs/BRANCH-PROTECTION.md` 已提供配置方案，但 `docs/PROGRESS.md` 也明确写了 **分支保护未应用**。

这会带来一个现实结果：

- 流程规则存在
- 但 repo 还没有完全进入 enforced 状态

对单人或少量 agent 协作，这不是立刻阻断开发的因素；但对“新 agent 是否能放心介入”而言，它会降低流程的可靠性。

---

## 判断拆分

### A. 是否能快速读懂项目

**能。**

原因：

- 有总计划
- 有 ADR
- 有任务 spec
- 有设计原型
- 有进度面板

新 agent 即使没有历史上下文，也能比较快判断：

- 项目做什么
- 不做什么
- 当前阶段在哪
- 下一个关键动作是什么

### B. 是否能快速按流程进入任务开发

**部分能，但不够顺滑。**

原因：

- 没有 `ready` task
- 入口文档存在旧状态残留
- 部分 strategic mapping 断链

所以当前最准确的描述应是：

**“可以快速接手项目认知和任务评审，但还不算零摩擦开发就绪。”**

---

## 建议的最小修补动作

如果目标是把当前状态从“基本可接手”提升到“新 agent 可直接开工”，最优先做下面 4 件事：

### P0 · 同步入口文档

至少同步以下文件的阶段描述：

- `CLAUDE.md`
- `docs/SESSION-STARTUP.md`
- `README.md`

统一到与 `docs/PROGRESS.md` 一致的状态：

- Phase 1-4 已完成
- 当前进入 Spike W0 准备阶段
- 不是再补 Phase 2/3/4 文档

### P0 · 把首批可执行任务翻转为 `ready`

最少应处理：

- `SPIKE-01`
- `SPIKE-02`
- 如有需要，再补 `MVP-01`

这样新 agent 才能真正按项目自己的流程直接认领。

### P1 · 修正失效的 `plan_ref` / 正文章节引用

重点处理：

- `§5.4` 相关任务
- `SPIKE-04` 中的 `§512`

目标不是美化，而是恢复“文档可追溯性”。

### P1 · 应用 main 分支保护

把流程从“文档约定”升级为“平台约束”，避免后续 Spike 阶段开始写代码后仍靠人工守门。

---

## 总结

Vibestation 现在的状态，不是“文档不足，无法接手”；恰恰相反，**文档资产已经很多，而且多数质量不低**。

真正的问题是：

- 文档更新不同步
- 流程准入未完成
- 少量关键引用断链

所以最终判断不是简单的“行”或“不行”，而是：

**新 agent 已经可以较快介入项目，但还不应该被描述为“完全开发就绪”。**

更稳妥的说法是：

**“上下文接手就绪，高于平均；直接开发就绪，中等偏上，但还差一次入口文档清理和首批 task 放行。”**

---

## 后续事项

- 若要把结论升级为“完全满足快速介入开发”，建议先完成上面的 P0/P1 修补
- 若下一步就是启动 Spike W0，建议把 `SPIKE-01` 和 `SPIKE-02` 先翻转为 `ready`
- 若后续继续保留多份 onboarding 文档，必须建立一个单一权威入口，避免阶段状态漂移
