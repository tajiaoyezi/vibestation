# Architecture Decision Records (ADR)

> 本目录存放 **Vibestation 的架构决策记录**。每条重大决策（=影响多模块 / 破坏向后兼容 / 跨 phase 影响 / 代价显著）必须对应一个 ADR。
> 格式参考 [MADR 4.0](https://adr.github.io/madr/)，中文化精简。

---

## 📂 ADR 索引

| ID | 标题 | 状态 | 决策表 # | Spike / PR |
|----|------|------|---------|------------|
| [ADR-001](./ADR-001-license-apache-2.0.md) | 许可证 = Apache License 2.0（不签 CLA）| **accepted** | #1 | 锁定 @ Phase 1 |
| [ADR-002](./ADR-002-mvp-scope-b-compromise.md) | MVP 范围 = B 折中方案 | **accepted** | #2 | 锁定 @ Phase 1 |
| [ADR-003](./ADR-003-pty-architecture.md) | PTY 架构 = portable-pty + 共享读线程 + mpsc | **accepted** | #15 | [SPIKE-05.5](../spikes/SPIKE-05.5-report.md) accepted @ 2026-04-19 |
| [ADR-004](./ADR-004-frontend-stack.md) | 前端栈 = SolidJS + TypeScript + Vite + xterm.js | **accepted** | #6 | 锁定 @ Phase 1 |
| [ADR-005](./ADR-005-local-storage.md) | 本地存储 = rusqlite（redb 因坏库检测 FAIL superseded）| **accepted** | #14 | [SPIKE-04.5](../spikes/SPIKE-04.5-report.md) accepted @ 2026-04-19 |
| [ADR-006](./ADR-006-desktop-framework.md) | 桌面框架 = Tauri 2（fallback: Electron 28+ · Ubuntu Phase B pending caveat） | **accepted** | A 栏 #19 | macOS Phase A 强 PASS · SPIKE-01/02 Ubuntu 待环境 · 不阻塞锁定 · PR #50 @ 2026-04-19 |
| [ADR-007](./ADR-007-git-stack.md) | Git 栈 = git2 0.20（写）+ gix 0.70（读优化）| **accepted** | #13 | [SPIKE-03](../spikes/SPIKE-03-report.md) accepted @ 2026-04-19 |
| [ADR-008](./ADR-008-diff-renderer-custom.md) | Diff 渲染 = 自建（非 Monaco）| **accepted** | #7 | 锁定 @ Phase 1 |
| [ADR-009](./ADR-009-ai-aware-v1-vision.md) | AI-Aware Pane 联动 = v1.0 vision（对外不宣传）| **accepted** | #3 | 锁定 @ Phase 1 |
| [ADR-010](./ADR-010-cargo-workspace-2-crate.md) | Cargo workspace = 2 crate（app + core）| **accepted** | #5 | 锁定 @ Phase 1 |
| [ADR-011](./ADR-011-runtime-evidence-location.md) | Runtime evidence 路径锁 `docs/runtime-evidence/<task-id>/` | **accepted** | #18 | Session 10 FU-2 · PR #44/#45 @ 2026-04-19 |
| [ADR-012](./ADR-012-v2d1-arbiter-approval-simplification.md) | v2-D → v2-D.1 · 单人项目 Arbiter approval 规则简化（删 24h 补 comment 硬要求）| **accepted** | —（治理规则）| Session 13 开场 · session 12 audit H1 根因 @ 2026-04-21 |
| [ADR-013](./ADR-013-spike-cold-backup-degradation.md) | Spike 冷备归档 v1 强制 → v2 推荐（22% 合规率实证 · 3 场景判断清单保留特殊情况）| **accepted** | —（项目规则）| Session 13 中 · audit M-1 根因 @ 2026-04-21 |

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
cp docs/adr/_template.md docs/adr/ADR-012-<slug>.md

# 2. 填写 ADR（见下方 section 结构）
#    - 必须有"考虑的选项"（至少 2 个）
#    - 必须有"后果"（正面 + 负面都列）

# 3. 开 PR（conventional commit + trailer）
git checkout -b docs/adr-012-<slug>
git add docs/adr/ADR-012-<slug>.md docs/adr/README.md
git commit -m "docs(adr): 新增 ADR-012 <中文标题>

Co-authored-by: <Agent Name> <email>"
gh pr create --title "docs(adr): 新增 ADR-012 <标题>"

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

- `CLAUDE.md §决策状态表`：**快速索引 + 锁定状态** · 本目录的 10 个 ADR 覆盖其中 10 条（见下方精确清单）· 其余条款仍由 `implementation-plan.md` / 原型文件锁定
- `implementation-plan.md`：**战略依据** · ADR 的"背景与问题"通常引用此文具体章节
- `docs/tasks/SPIKE-*`：**验证依据** · `proposed` ADR 依赖 Spike 结论 · Spike 通过后更新 ADR 状态
- `docs/tasks/README.md`：**任务流程** · 实施 ADR 决策的具体 task spec

---

## 📊 ADR 覆盖范围（Codex PR #12 review F1 复核精确描述 · Session 10 accepted 扩充）

**已 ADR 化（11 条 · 11 accepted · 0 proposed · session 10 末 ADR-006 升级后全 accepted）**：
- A 档 · 11 条 accepted：
  - Phase 1 锁定 6 条：`#1` License · `#2` MVP 范围 · `#3` AI-Aware v1.0 vision · `#5` Cargo workspace · `#6` 前端栈 · `#7` Diff 自建
  - Session 8-10 Spike 通过 3 条：`#13` Git 栈（SPIKE-03 @ 2026-04-19）· `#14` 本地存储 rusqlite（SPIKE-04.5 @ 2026-04-19）· `#15` PTY 方案（SPIKE-05.5 @ 2026-04-19）
  - Session 10 FU-2 新增 1 条：`#18` Runtime evidence 路径锁（PR #44/#45 @ 2026-04-19）
  - Session 10 末 B→A 升级 1 条：`#19` 桌面框架 Tauri 2（macOS Phase A 强 PASS · Ubuntu Phase B pending caveat · PR #50 @ 2026-04-19）
- B 档 · 当前空（原 #12 桌面框架 session 10 末升级到 A 栏 #19 · B 栏 header 保留作未来类似决策载体）

**A 档未 ADR 化**（5 条 · 已在其他文档锁定 · 需要改变决策时才补 ADR）：
- `#4` Calm Studio 视觉方向 → 锁定 @ `design/directions/1-calm-studio.html`
- `#8` 平台 MVP = macOS + Ubuntu → 锁定 @ `implementation-plan.md §3.1`
- `#9` Tool Windows 默认状态 → 锁定 @ 原型 JS `DEFAULT_STATE`
- `#10` Telemetry 默认关 + opt-in → 锁定 @ `implementation-plan.md §5.1` + R30
- `#11` Landing page 栈 = Astro → 锁定 @ `implementation-plan.md §12`

**C 档（2 条 · 时间锁定结果开放）**：不建 ADR · 由时间节点触发决策并直接更新 `CLAUDE.md` 决策表。
- `#16` 项目域名 TLD（W10 附近）
- `#17` Logo 最终定稿（v0.1 发布前）

---

**本目录 Phase 3 建立（2026-04-18）· 当前 11 个 ADR（11 accepted · 0 proposed · session 10 末全收敛）· 覆盖范围如上精确描述。**
