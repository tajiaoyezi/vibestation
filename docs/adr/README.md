# Architecture Decision Records (ADR)

> 本目录存放 **Vibestation 的架构决策记录**。每条重大决策（=影响多模块 / 破坏向后兼容 / 跨 phase 影响 / 代价显著）必须对应一个 ADR。
> 格式参考 [MADR 4.0](https://adr.github.io/madr/)，中文化精简。

---

## 📂 ADR 索引

| ID | 标题 | 状态 | 决策表 # | Spike / PR |
|----|------|------|---------|------------|
| [ADR-001](./ADR-001-license-apache-2.0.md) | 许可证 = Apache License 2.0（不签 CLA）| **accepted** | #1 | 锁定 @ Phase 1 |
| [ADR-002](./ADR-002-mvp-scope-b-compromise.md) | MVP 范围 = B 折中方案 | **accepted** | #2 | 锁定 @ Phase 1 |
| [ADR-003](./ADR-003-pty-architecture.md) | PTY 架构 = portable-pty + 共享读线程 + mpsc | **proposed** | #15 | Pending [SPIKE-05](../tasks/SPIKE-05-pty-multi-tab.md) |
| [ADR-004](./ADR-004-frontend-stack.md) | 前端栈 = SolidJS + TypeScript + Vite + xterm.js | **accepted** | #6 | 锁定 @ Phase 1 |
| [ADR-005](./ADR-005-local-storage.md) | 本地存储 = redb 2（fallback: rusqlite）| **proposed** | #14 | Pending [SPIKE-04](../tasks/SPIKE-04-storage-benchmark.md) |
| [ADR-006](./ADR-006-desktop-framework.md) | 桌面框架 = Tauri 2（fallback: Electron 28+）| **proposed** | #12 | Pending [SPIKE-02](../tasks/SPIKE-02-tauri-hard-pass-matrix.md) |
| [ADR-007](./ADR-007-git-stack.md) | Git 栈 = git2 0.20（写）+ gix 0.70（读优化）| **proposed** | #13 | Pending [SPIKE-03](../tasks/SPIKE-03-git2-gix-read-benchmark.md) |
| [ADR-008](./ADR-008-diff-renderer-custom.md) | Diff 渲染 = 自建（非 Monaco）| **accepted** | #7 | 锁定 @ Phase 1 |
| [ADR-009](./ADR-009-ai-aware-v1-vision.md) | AI-Aware Pane 联动 = v1.0 vision（对外不宣传）| **accepted** | #3 | 锁定 @ Phase 1 |
| [ADR-010](./ADR-010-cargo-workspace-2-crate.md) | Cargo workspace = 2 crate（app + core）| **accepted** | #5 | 锁定 @ Phase 1 |

---

## 📝 状态定义

| 状态 | 含义 |
|------|------|
| **proposed** | 决策草案 · 依赖 Spike 验证或 Arbiter 仲裁 · 当前 `CLAUDE.md` 决策表 B 栏 |
| **accepted** | 已锁定 · 当前 `CLAUDE.md` 决策表 A 栏 · 除非写新 ADR 推翻否则不得讨论 |
| **rejected** | 曾考虑但拒绝 · 保留做历史记录 |
| **deprecated** | 曾 accepted · 后续因新决策不再适用 · 通常指向 superseded by |
| **superseded by** | 被新 ADR 取代 · 见链接 |

**重要**：`proposed → accepted` 的触发是"Spike 通过 + 独立评审 + 用户拍板"三件套。缺一不可。

---

## 🚀 新增 ADR 的流程

当出现一个**新决策**（同时满足：影响多模块 OR 破坏向后兼容 OR 跨 phase）时：

```bash
# 1. 从模板创建新文件
cp docs/adr/_template.md docs/adr/ADR-011-<slug>.md

# 2. 填写 ADR（见下方 section 结构）
#    - 必须有"考虑的选项"（至少 2 个）
#    - 必须有"后果"（正面 + 负面都列）

# 3. 开 PR（conventional commit + trailer）
git checkout -b docs/adr-011-<slug>
git add docs/adr/ADR-011-<slug>.md docs/adr/README.md
git commit -m "docs(adr): 新增 ADR-011 <中文标题>

Co-authored-by: <agent> via <email>"
gh pr create --title "docs(adr): 新增 ADR-011 <标题>"

# 4. 独立评审（≠ 原作者）· 确认 Spike / 讨论依据充分 · 过"翻转 gate"
# 5. merge 后更新 CLAUDE.md 决策表（若对应 B 栏 → A 栏）
```

---

## 📝 ADR Section 结构（MADR 4.0 中文化）

每个 ADR 必含以下章节，顺序固定：

```markdown
# ADR-NNN: <中文标题>

**状态**：proposed | accepted | rejected | deprecated | superseded by [ADR-XXX]
**日期**：YYYY-MM-DD
**决策者**：<作者 agent-id + 独立评审者 + 用户>
**对应 `CLAUDE.md` 决策表**：# N
**对应 Spike**：[SPIKE-NN](../tasks/SPIKE-NN-<slug>.md)（若 Spike 依赖）

## 背景与问题（Context and Problem Statement）

## 决策驱动因素（Decision Drivers）

## 考虑的选项（Considered Options）

## 决策（Decision Outcome）

## 后果（Consequences）

### 正面

### 负面

### 风险

## 与 `implementation-plan.md` 的映射

## 相关（Links）
```

---

## 🔗 与其他治理文档的关系

- `CLAUDE.md §决策状态表`：**快速索引 + 锁定状态** · 每条对应一个 ADR 文件
- `implementation-plan.md`：**战略依据** · ADR 的"背景与问题"通常引用此文具体章节
- `docs/tasks/SPIKE-*`：**验证依据** · `proposed` ADR 依赖 Spike 结论 · Spike 通过后更新 ADR 状态
- `docs/tasks/README.md`：**任务流程** · 实施 ADR 决策的具体 task spec

---

**本目录 Phase 3 建立（2026-04-18）· 10 个 ADR 覆盖 `CLAUDE.md` 决策表所有 A + B 档。**
