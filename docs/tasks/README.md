# Tasks · 任务索引

> 本目录存放所有**可执行任务的详细规格（task spec）**：SPIKE（技术验证）/ MVP（MVP 功能）/ BUG（缺陷）/ FEAT（v0.2+ 功能）。
> 每个 task spec 是**一个 PR 的验收依据**——评审者按 spec 的 Acceptance 逐项对照 diff。

---

## 📂 命名规范

```
<TYPE>-<编号>-<英文 slug>.md
```

| TYPE | 用途 | 编号规则 |
|------|------|---------|
| `SPIKE` | 技术验证 / benchmark / 风险消除 | 按 Spike 天数顺序 `SPIKE-01..06` |
| `MVP` | MVP B 折中方案范围内的功能（`implementation-plan.md §10.1`）| 按模块顺序 `MVP-01..20` |
| `BUG` | 缺陷修复 | 按发现顺序 `BUG-001..` |
| `FEAT` | v0.2+ 新功能 | 按路线图顺序 `FEAT-01..` |

**示例**：`SPIKE-01-tauri-three-platform-boot.md`、`MVP-03-terminal-multi-tab.md`、`BUG-001-pty-resize-crash.md`

**slug 要求**：小写英文 + 连字符，3-5 个词，语义清晰。

---

## 🔄 状态流转

```
draft ──┐
        ├──► ready ──► in-progress ──► done
        │        ▲           │
        │        └──────────┤
        │                    ▼
        └──────────────► blocked ──► ready（blocker 解除）
```

| 状态 | 含义 | 进入条件 |
|------|------|---------|
| `draft` | 草稿，字段未填完 | 新建 |
| `ready` | 可被认领，字段完整，Acceptance 明确 | 作者自审 + 独立评审通过 |
| `in-progress` | 已被某 agent/人类认领实施 | PR 打开且分支存在 |
| `blocked` | 被依赖项或外部资源阻塞 | `blocks_by` 字段填写原因 |
| `done` | PR 已 merge 到 main，Acceptance 全过 | Acceptance 逐项勾完 + merge |

**规则**：
- 状态字段**必须**与 `PROGRESS.md`、PR description 一致
- `done` 状态的 task 文件**不删除**，作为历史留档（Phase 3 可选归档到 `docs/session-history/`）

---

## 📋 字段说明（common schema，所有 TYPE 共享）

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | ✅ | `SPIKE-01` / `MVP-03` / `BUG-001` |
| `type` | enum | ✅ | `spike` / `mvp` / `bug` / `feat` |
| `title` | string | ✅ | 中文简述（≤ 30 字）|
| `status` | enum | ✅ | 见上表 |
| `owner` | string | ⛔ 留空 = 未认领 | 认领者标识（PR `Implemented by` 填写的 agent/人类 ID）|
| `phase` | string | ✅ | `W0-D1` / `W1` / `W5` / `v0.2` |
| `depends_on` | list | ✅（可空 `[]`）| 任务依赖的 task id（必须全 `done` 才能 `ready`）|
| `blocks` | list | ✅（可空 `[]`）| 该 task 完成后解锁的 task id |
| `writes_to` | list | ✅（可空 `[]`）| **并发声明**：本 task 会修改/创建的文件或目录。不得包含 `CLAUDE.md` 决策表（锁定走独立 PR）|
| `reads_from` | list | ✅（可空 `[]`）| 依赖的其他 task 产出，格式 `<task-id>:<path>`，如 `"SPIKE-01:spike-tmp/spike-01-tauri/"` |
| `estimate` | string | ✅ | `0.5d` / `1d` / `3d` |
| `plan_ref` | string | ✅ | `implementation-plan.md` 章节 `§3.1.1` |
| `risk_ref` | string | ⛔ 可选 | `R1` / `R12` / `R27` 等 `implementation-plan §9` 风险 ID |
| `reviewer` | string | ⛔ 默认填 PR review 时 | 独立评审者（≠ owner）|

**YAML frontmatter 示例**：详见 [`_template.md`](./_template.md)。

---

## 📝 正文 Section（按 TYPE 差异化）

### SPIKE 必填

- **目标**（Goal）：一句话描述要验证什么
- **背景**（Context）：为什么现在做这个 Spike
- **通过标准**（Pass Criteria）：**可量化**的判据（P99 延迟 / 冷启动时间 / 错误率 …）
- **失败信号**（Fail Signals）：触发 fallback 的具体条件
- **Fallback 方案**：通过 / 失败后的分支决策（对应 `CLAUDE.md` 决策表 B 栏）
- **产出**（Deliverables）：benchmark 数据表 / 录屏 / ADR 草稿 / 代码 proof
- **依赖资源**：硬件 / 账号 / 数据集（如 linux kernel 仓库）

### MVP / FEAT 必填

- **功能范围**（Scope）：什么做，什么不做
- **UI 引用**（UI Reference）：`design/directions/1-calm-studio.html` 对应区块 / 截图
- **Acceptance**（验收清单）：勾选式，evaluator 按条对照
- **测试策略**：单元 / 集成 / E2E 覆盖哪些路径
- **数据模型变更**（如有）：redb key schema / redb table 变化

### BUG 必填

- **复现步骤**（Reproduction Steps）
- **期望行为** vs **实际行为**
- **根因分析**（Root Cause）
- **修复验证**（Fix Verification）：回归测试

> BUG 和 FEAT 模板在真实需要时 Phase 3 补；当前 Phase 2 只定 SPIKE + MVP。

---

## 🗂 当前索引

### SPIKE（W0 周，硬依赖 Spike W0 启动）

| ID | 标题 | 状态 | 估时 | 依赖 | 风险 |
|----|------|------|------|------|------|
| [SPIKE-01](./SPIKE-01-tauri-three-platform-boot.md) | Tauri 2 三平台空壳启动（mac + Ubuntu Wayland + X11）| draft | 1d | — | R12 |
| [SPIKE-02](./SPIKE-02-tauri-hard-pass-matrix.md) | Tauri 硬通过矩阵 + Electron fallback（若 D1 失败）| draft | 1d | SPIKE-01 | **R12 CRITICAL** |
| [SPIKE-03](./SPIKE-03-git2-gix-read-benchmark.md) | git2 读 log + gix 对比 benchmark（linux kernel）| draft | 1d | SPIKE-02 | R3 |
| [SPIKE-04](./SPIKE-04-storage-benchmark.md) | redb 2 vs rusqlite benchmark + git2 写 commit | draft | 1d | SPIKE-02 | R27 |
| [SPIKE-05](./SPIKE-05-pty-multi-tab.md) | portable-pty 单读 + mpsc + xterm 4-Tab 压测 | draft | 1d | SPIKE-02 | — |
| [SPIKE-06](./SPIKE-06-cli-protocol-and-codesign.md) | Claude CLI / Codex CLI 实机 + macOS Dev Program | draft | 1d | SPIKE-05 | R1 |

### MVP

| ID | 标题 | 状态 | 估时 |
|----|------|------|------|
| MVP-01..10 | 详细 spec | 下一 PR | — |
| MVP-11..20 | 占位 spec（最小骨架）| 下下 PR | — |

### BUG / FEAT

当前无。

---

## 🔀 并发安全（多 agent 并行执行时读）

本项目欢迎多个 agent 实例 / 人类同时推进不同 task。**并发安全靠两个机制**：

### 1. `writes_to` 声明（显式 · 认领前检查）

每个 task 的 frontmatter 必须声明 `writes_to`，列出本 task 会修改/创建的文件或目录。认领 task 前：

```bash
gh pr list --state open                           # 查所有 open PR
# 对每个 open PR 查看其引用 task 的 writes_to
# 检查本 task 的 writes_to 是否与任何 open PR 的 writes_to 有交集
```

**有交集** → 选其他 task 或等对方 merge
**无交集** → 认领（改 status: in-progress + owner + 开分支）

### 2. 锁定表更新独立化（强制 · 防 scalar 冲突）

`CLAUDE.md` 决策表 A/B/C 档是 **scalar 字段**（单值），多 SPIKE 同时触发锁定会并发冲突。规则：

- ❌ **spec PR 不得在同一 PR 里修改 `CLAUDE.md` 决策表**
- ✅ **spec merge 后开独立 PR**，标题 `chore(decision): 锁定 #N <决策>`
- ✅ SPIKE spec 的 Deliverables 里应注明 "锁定 #N 走独立 PR"

### 3. 冲突升级路径

| 冲突类型 | 处理 |
|---------|------|
| 两个 task 的 `writes_to` 有交集（认领前未发现）| 后认领者 rebase + 保留两方意图；冲突字段由 Arbiter（用户）仲裁 |
| scalar 字段冲突（如 PROGRESS `Active branch`、决策表某行）| 不 merge 两方，Arbiter 写新值 |
| 依赖链破坏（reads_from 的上游 task 被 supersede）| 触发 task 重新 ready：`status: blocked` → 等上游更新 → 本 task spec 按新 schema 重写 |

---

## 🚀 新建 task 的流程

```bash
# 1. 复制模板
cp docs/tasks/_template.md docs/tasks/SPIKE-07-<slug>.md

# 2. 填写 frontmatter + 正文 section

# 3. 开 feature 分支
git checkout -b docs/tasks/SPIKE-07-<slug>

# 4. 自审四问（CLAUDE.md "📝 写规则/清单前的自审四问"）
#    - 递归完备性 / 反向场景 / 边界适用性 / YAGNI

# 5. commit + push + PR（Conventional Commits + 中文描述 + trailer）
git commit -m "docs(tasks): 新增 SPIKE-07 <中文描述>

Co-authored-by: <Agent Identity> via <email>"

# 6. PR description 写 "Implemented by: X · Reviewed by: Y"

# 7. 独立评审（≠ 原作者）→ merge
```

---

## ⚠️ 原则（不要重演 Phase 1 过度设计）

1. **不做 claim 机制 / 自动状态流转 / CI 校验脚本**——当前靠**声明式字段 `writes_to` / `reads_from`** + agent 自觉；CI 校验 Phase 4 真遇到冲突再加
2. **状态字段靠 PR description 和 commit 同步**，不在文件里搞复杂的锁
3. **锁定表更新独立化**（见上面「并发安全 §2」）：spec PR 不动 `CLAUDE.md` 决策表
4. **task spec 冲突**：同一 task 两人同时动 → PR 冲突时 rebase + 保留两方意图 + scalar 冲突找 Arbiter（用户）
5. **task spec 是"一个 PR 一个逻辑单元"的依据**——评审者按 Acceptance 逐项对照

---

**本目录 Phase 2 建立（2026-04-18）。SPIKE-01..06 作为 Spike W0 启动的硬依赖。**
