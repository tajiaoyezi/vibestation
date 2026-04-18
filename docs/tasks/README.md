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
      (新建)          (spec 过独立评审)    (认领)         (Acceptance 全过)
draft ────────► ready ──────────────► in-progress ────────────► done
                  ▲ │                      ▲ │
                  │ │ (外部阻塞)           │ │ (外部阻塞)
                  │ └────► blocked ◄──────┘ │
                  │   (blocked_by 填原因)   │
                  └──── (阻塞解除恢复原状态)─┘
```

| 状态 | 含义 | 进入条件 | 出口条件 |
|------|------|---------|---------|
| `draft` | 草稿，字段未填完 / 未评审 | 新建 | 作者自审 + 独立评审通过 → `ready` |
| `ready` | 可被认领，字段完整，Acceptance 明确 | spec PR 评审通过（同 PR 最后一个 commit 改 `status: ready`）| 某 agent 认领 → `in-progress` · 或遇外部阻塞 → `blocked` |
| `in-progress` | 已被认领并实施中 | 实施 PR 首个 commit 改 `owner` + `status: in-progress` | Acceptance 全过 → `done` · 或遇外部阻塞 → `blocked` |
| `blocked` | 被依赖项或外部资源阻塞 | 从 `ready` 或 `in-progress` 进入；必填 `blocked_by`（上游 task-id 或外部资源名）；可选 `blocked_note`（人类可读原因）| 阻塞解除 → **恢复到进入前的状态**（见下方规则）|
| `done` | PR 已 merge 到 main，Acceptance 全过 | 实施 PR merge 前最后一个 commit 改 `reviewer` + `status: done` → merge | 终态（不删文件，作为历史留档）|

**`blocked` 状态恢复规则**（解除阻塞时执行）：

- 从 `in-progress` 进入 `blocked`：解除后**回到 `in-progress`**，`owner` **保留**，对应 branch 和 open PR **不动**（agent 继续原工作）
- 从 `ready` 进入 `blocked`：解除后**回到 `ready`**，`owner` 保持空（等待新 agent 认领）
- 解除时必做：清空 `blocked_by` 和 `blocked_note` 字段

**其他规则**：
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
| `depends_on` | list | ✅（可空 `[]`）| 依赖的 task id |
| `blocks` | list | ✅（可空 `[]`）| 该 task 完成后解锁的 task id |
| `blocked_by` | list | ⛔（仅 `status: blocked` 时必填）| 阻塞源：task-id（如 `["SPIKE-02"]`）或外部资源（如 `["apple-dev-program-approval"]`）|
| `blocked_note` | string | ⛔ 可选 | 人类可读的阻塞原因说明（1-2 句）|
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

### MVP（v0.1 范围 · B 折中方案）

| ID | 标题 | 状态 | 估时 | 依赖 |
|----|------|------|------|------|
| [MVP-01](./MVP-01-tauri-app-shell.md) | Tauri 应用骨架 + 启动流程 + 基础崩溃恢复 | draft | 5d | SPIKE-02 |
| [MVP-02](./MVP-02-workspace-management.md) | Workspace 管理 + 项目识别 + 多 workspace 并存 | draft | 4d | MVP-01 |
| [MVP-03](./MVP-03-tool-windows-layout.md) | Tool Windows 布局（Primary/Secondary/Bottom + Activity Strip）| draft | 4d | MVP-01/02 |
| [MVP-04](./MVP-04-multi-tab-terminal.md) | 多 Tab 终端（PTY + xterm + Shell/CLI 兼容）| draft | 8d | MVP-03 · SPIKE-05/06 |
| [MVP-05](./MVP-05-pane-split-single-level.md) | Pane 分屏（单层 · 最多 4 Pane · Smart Layouts）| draft | 4d | MVP-04 |
| [MVP-06](./MVP-06-config-import.md) | 配置导入（Ghostty + iTerm2 + Alacritty）| draft | 3d | MVP-04 |
| [MVP-07](./MVP-07-git-log-readonly.md) | Git Log 只读视图 + Commit 详情 | draft | 5d | MVP-02/03 · SPIKE-03 |
| [MVP-08](./MVP-08-diff-and-git-status.md) | Diff 基础视图（自绘）+ Git Status 只读面板 | draft | 5d | MVP-07 |
| [MVP-09](./MVP-09-stage-unstage-commit.md) | Stage/Unstage + Commit 操作（git2 写）| draft | 4d | MVP-08 · SPIKE-04 |
| [MVP-10](./MVP-10-settings-telemetry-packaging.md) | 设置面板 + Telemetry opt-in + 打包发布（v0.1 GA）| draft | 5d | MVP-01..09 全部 |

**占位 spec（v0.2 / v0.3 / v1.0 范围 · `implementation-plan.md §10.1` 砍到后续版本）**：

| ID | 标题 | 状态 | 目标版本 | 估时 | 依赖 |
|----|------|------|---------|------|------|
| [MVP-11](./MVP-11-git-push-pull-fetch.md) | Git Push / Pull / Fetch（远端同步）| draft | v0.2 | 5d | MVP-09 |
| [MVP-12](./MVP-12-commit-rail-graph.md) | 自绘 commit rail graph | draft | v0.2 | 8d | MVP-07 |
| [MVP-13](./MVP-13-branch-crud.md) | 分支 create / checkout / delete | draft | v0.2 | 4d | MVP-07/09 |
| [MVP-14](./MVP-14-pane-advanced-layout.md) | Pane 高级布局（任意嵌套 + 3 预设 + 导航 + 最大化）| draft | v0.2 | 7d | MVP-05 |
| [MVP-15](./MVP-15-diff-syntax-highlight.md) | Diff 复杂语法高亮（tree-sitter）| draft | v0.3 | 6d | MVP-08 |
| [MVP-16](./MVP-16-rebase-merge-cherrypick.md) | Rebase / Merge / Cherry-pick | draft | v0.3 | 7d | MVP-13 |
| [MVP-17](./MVP-17-external-terminal-pane-detach.md) | Pop to External + Pane Detach | draft | v0.3 | 4d | MVP-14 |
| [MVP-18](./MVP-18-ai-aware-pane-linking.md) | **AI-Aware Pane 联动**（v1.0 vision · 对外禁提）| draft | v1.0 | 15d | MVP-14 · SPIKE-07 |
| [MVP-19](./MVP-19-session-commit-binding.md) | **AI session ↔ commit 自动绑定**（v1.0 vision）| draft | v1.0 | 8d | MVP-18 |
| [MVP-20](./MVP-20-ai-one-click-rollback.md) | **AI 一键回滚（session 级 revert）**（v1.0 vision）| draft | v1.0 | 6d | MVP-19 |

> 占位 spec 用途：在 `<TYPE>-NN-<slug>` 编号连续性 + 依赖可视化上提前占位，v0.2 / v0.3 / v1.0 启动时按 kickoff 详化到实施 spec（补具体 UI 截图 / Acceptance 可量化门槛 / 数据模型细节）。

### BUG / FEAT

当前无。

---

## 🚀 新建 task 的流程（spec 创建 PR · `draft → ready` 落盘）

> 本流程用于**创建新的 task spec**（从无到 `status: ready`）。实施 task 的流程见 `CLAUDE.md` "🚀 新 Agent 首次启动" 第 5 步（`ready → in-progress → done`）。

```bash
# 1. 复制模板
cp docs/tasks/_template.md docs/tasks/SPIKE-07-<slug>.md

# 2. 填写 frontmatter（默认 status: draft）+ 正文 section

# 3. 开 feature 分支
git checkout -b docs/tasks/SPIKE-07-<slug>

# 4. 自审四问（CLAUDE.md "📝 写规则/清单前的自审四问"）
#    - 递归完备性 / 反向场景 / 边界适用性 / YAGNI

# 5. commit + push + PR（Conventional Commits + 中文描述 + trailer）
git commit -m "docs(tasks): 新增 SPIKE-07 <中文描述>

Co-authored-by: <Agent Identity> via <email>"
git push -u origin docs/tasks/SPIKE-07-<slug>
gh pr create

# 6. PR description 必填：
#    - Author: <作者 agent-id>
#    - Spec Reviewed by: <待评审>（和实施 task 的 Reviewer 不同）

# 7. 独立评审（≠ 原作者）approve 后，在 merge 前最后一个 commit
#    把 task status 从 draft 改为 ready：
git commit -m "chore(tasks/SPIKE-07): spec reviewed, status: ready"
git push

# 8. merge → 此后其他 agent 可从 status: ready 认领
#    （走 CLAUDE.md 5 步导游的"认领 → 开工 → 收尾"流程）
```

---

## ⚠️ 原则（不要重演 Phase 1 过度设计）

1. **不做 claim 机制 / 自动状态流转 / CI 校验脚本**——Phase 2 真遇到并发问题再加（CLAUDE.md "📝 写规则/清单前的自审四问" 第 4 条 YAGNI）
2. **状态字段靠 PR description 和 commit 同步**，不在文件里搞复杂的锁
3. **task spec 冲突**：同一 task 两人同时动 → PR 冲突时 rebase + 保留两方意图 + scalar 冲突找 Arbiter（用户）
4. **task spec 是"一个 PR 一个逻辑单元"的依据**——评审者按 Acceptance 逐项对照
5. **Deliverables 用 per-task 文件，不用共享文件**：每个 task 写自己的 `docs/spikes/<id>-report.md` / `docs/adr/ADR-NNN-<slug>.md`，**不要**多个 task 都往 `docs/SPIKE-REPORT.md` 写——物理隔离比"声明式并发治理"更可靠（详见 `docs/session-history/` Phase 3 后的 PR #4 close 反思）
6. **`spike-tmp/` 是作者本地 scratchpad**（`.gitignore` 已排除），**不得作为其他 task 的依赖源**：跨 task 交接只能基于 committed / versioned 产物

---

**本目录 Phase 2 建立（2026-04-18）。SPIKE-01..06 作为 Spike W0 启动的硬依赖。**
