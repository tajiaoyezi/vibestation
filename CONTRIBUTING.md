# Contributing to Vibestation

> 欢迎为 Vibestation 贡献代码、文档、bug 报告、功能建议。本项目 **Apache 2.0 许可**、**不签 CLA**（贡献默认 Apache 2.0 授权 · 见 [ADR-001](docs/adr/ADR-001-license-apache-2.0.md)）。

---

## 📌 贡献者类型

本项目欢迎任何 agent 工具（Claude Code / Codex CLI / Cursor / Aider / OpenCode / Windsurf / Gemini / 人类开发者 / 自建 agent）参与。**不绑定具体 agent 身份**。

---

## 🚀 第一次贡献

### Step 1 · 读入门三件套

1. **`CLAUDE.md`**（5 步机器可读 checklist · 锁定决策 · 禁区 · 代码风格 · 自审四问）
2. **`docs/PROGRESS.md`**（当前阶段 / 上次 session / 卡点 / 下一步）
3. **`docs/tasks/README.md`**（任务索引 + 状态流转 + 新建流程）

### Step 2 · 选择贡献方式

| 方式 | 入口 | 模板 |
|------|------|------|
| **报告 bug** | GitHub Issues · `🐛 Bug Report` | `.github/ISSUE_TEMPLATE/bug_report.yml` |
| **建议功能** | GitHub Issues · `✨ Feature Request` | 含 AI-Aware 叙事合规勾选 |
| **提议 task spec** | GitHub Issues · `📋 Task Spec Proposal` | 后续可正式 PR |
| **讨论 / 问问题** | GitHub Discussions | 不走 issue |
| **安全漏洞** | GitHub Security Advisory（私密） | **不要开 issue** |
| **改代码 / 文档** | Fork + feature branch + PR | 见下方 [PR 流程](#-pr-流程) |

### Step 3 · 环境准备（代码贡献者）

**当前状态**（Phase 1-4 · pre-code）：**仓库无代码**。
- `pnpm tauri dev` / `cargo build` 等会失败（Spike W0 后才有代码）
- Phase 1-4 贡献主要是**文档 / spec / 流程**
- Spike W0 启动后更新本节

---

## 🔄 PR 流程

### A · 实施现有 task（`ready` → `done`）

按 **`CLAUDE.md` 第 5 步导游**（4 小步）：

```bash
# 1. 建分支（先于所有 commit）
git checkout -b <scope>/<task-id>         # 例：feat/MVP-01-tauri-app-shell

# 2. 认领（单独的 claim commit）
# 编辑 task spec · 改 owner + status: in-progress
git commit -m "chore(MVP-01): claim"

# 3. 开工（后续 commits · Conventional Commits + 中文 + trailer）
git commit -m "feat(MVP-01): 实现 Tauri 应用骨架

Co-authored-by: <Agent Identity> via <email>"
git push -u origin feat/MVP-01-tauri-app-shell
gh pr create

# 4. 收尾（独立评审 ≠ 原实现者 approve 后 · merge 前最后一个 commit）
# 走翻转 gate 二选一（CLAUDE.md 5.4 步）：
#   (a) Reviewer 自己 push 翻转 commit（推荐）
#   (b) Author push 翻转 + Reviewer re-approve 最新 HEAD
# 编辑 task spec · 改 reviewer + status: done
git commit -m "chore(MVP-01): done"
git push
# merge
```

### B · 新建 task spec（`draft` → `ready`）

按 **`docs/tasks/README.md` 第 7 步流程**：

```bash
# 1. 复制模板
cp docs/tasks/_template.md docs/tasks/SPIKE-07-<slug>.md

# 2. 填 frontmatter（默认 status: draft）+ 正文 section
# 3. 自审四问过关（CLAUDE.md）
# 4. 开 feature 分支 + commit + push + PR
# 5. 独立评审（≠ 原作者）approve 后走翻转 gate 改 status: ready
```

### C · 新增 ADR（架构决策）

按 **`docs/adr/README.md` 流程**：

```bash
# 1. 从模板创建
cp docs/adr/_template.md docs/adr/ADR-011-<slug>.md

# 2. 填写决策 · 必须有 ≥ 2 个候选选项 + 正面/负面/风险
# 3. 开 PR · Conventional Commits + 中文 + trailer
# 4. 独立评审（≠ 原作者 · 任意 agent / 人类）
# 5. **（B → A 升级专属 gate · Codex PR #12 F2 教训）** · 用户（Arbiter · @leaf）
#    必须在 PR 里明确 approve 后才能 merge · 三件套缺一不可：
#    (a) Spike 通过（对应 SPIKE-NN 有 passing benchmark 数据 / 结论写入 report）
#    (b) 独立评审通过（非作者 agent / 人类）
#    (c) **用户拍板**（Arbiter approve · PR 评论中明确"agree to promote #<N>
#        from B → A" 或同等声明）
#    **任一缺失 → 不得 merge** · 不得代替用户决策（即使 reviewer 是另一个 agent）
# 6. Merge 后同步更新 `CLAUDE.md` 决策表（B 栏 → A 栏 + 锁定依据 ADR 路径）
# 7. 同步更新 `docs/adr/README.md` 索引状态（proposed → accepted）
```

**A 档决策的修改流程**（推翻已 accepted ADR）：
1. 新建 `docs/adr/ADR-XXX-*.md`（替代原 ADR）· 或修改原 ADR 加 `status: deprecated · superseded by ADR-XXX`
2. 同样走 **Spike（若需要） + 独立评审 + 用户拍板** 三件套
3. 用户拍板是硬 gate · 即使只是"微调"也不能绕过

### D · 其他（基础设施 / 文档小改）

直接开 PR · PR description 说明"无关联 task spec · 属于 <基础设施 / 文档修复 / ...>"。

---

## 📝 Commit 规范

**Conventional Commits + 中文描述 + trailer**：

```
<type>(<scope>): <中文描述>

<可选正文 · 解释 why 而非 what>

Co-authored-by: <Agent Identity> via <email>
```

**type 枚举**：`feat` / `fix` / `docs` / `refactor` / `chore` / `test` / `perf` / `ci`

**scope 示例**：`MVP-01` / `SPIKE-03` / `tasks` / `adr` / `phase-4` / `terminal` / `git-log`

**trailer**：
- Claude Code: `Co-authored-by: Claude Code <noreply@anthropic.com>`
- Codex CLI: `Co-authored-by: Codex CLI <noreply@openai.com>`
- 其他 agent: `Co-authored-by: <Agent Name> <email>`
- 人类: `Co-authored-by: <Name> <email>`

---

## 🔒 PR description 必填字段

`.github/PULL_REQUEST_TEMPLATE.md` 已提供结构化模板，**以下字段必填**：

- **Summary**（1-3 句 · 做了什么）
- **Linked Task / Issue**（`docs/tasks/<TYPE-NN>.md` 或 `closes #NN`）
- **Implemented by**（作者 agent-id · 对应 commit trailer）
- **Reviewed by**（merge 前填 · 必须 ≠ Implemented by）
- **Task Status Transition** 勾选（翻转 gate 二选一）
- **Test Plan**（勾选式 · 按 task Acceptance）
- **Self Review · 自审四问**（规则 / 文档 / spec / 流程 PR 必答）

---

## ⚠️ 禁区（硬约束）

以下禁区来自 `CLAUDE.md §禁区` · **任何 PR 违反则硬拒绝**：

- ❌ **禁止 push 到 main**（走 feature branch + PR + 独立评审）
- ❌ **禁止对外文案提及** `AI-Aware Pane` / `Mission Control` / `AI session aware`（v1.0 vision · `ADR-009`）
- ❌ **禁止硬编码** API Key / 密码 / Token / 个人邮箱 / 生产域名（用 `.env.local`）
- ❌ **禁止跳过 CI 必过项**（`cargo clippy -D warnings` / `cargo fmt --check` / `pnpm lint` / `pnpm typecheck` / `gitleaks` / `task-spec-validator`）
- ❌ **禁止重排 `docs/implementation-plan.md`** 的章节结构（允许章末追加 changelog / v2.x 子节）
- ❌ **禁止修改 `design/directions/1-calm-studio.html`** 的布局结构 / 色彩 token / 字体选择（允许 token 数值微调 / bug 修复 / a11y 补强）
- ⚠️ **改锁定表 A 栏前必须**：(1) 开新 ADR · (2) 独立评审通过 · (3) 同步 `CLAUDE.md` + `implementation-plan.md`

---

## 🧪 自审四问（规则 / 文档 / spec / 流程 PR 必答）

`CLAUDE.md "📝 写规则/清单前的自审四问"`：

1. **递归完备性**：清单自己在清单里吗？规则适用于定义规则的文档自己吗？
2. **反向场景**：规则不遵守会怎样？有没有违规激励？
3. **边界适用性**：规则对所有数据形态 / 并发数 / 阶段适用吗？
4. **YAGNI**：当前阶段真需要这条吗？还是 Phase N 真遇到问题再加？

任一条答不清楚 → **删该规则，或标记 `[planned - 真实需要时加]`**。

---

## 🏛️ 治理与行为准则

- **Code of Conduct**：[`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md)（Contributor Covenant 2.1 中文版）
- **Branch protection**：[`docs/BRANCH-PROTECTION.md`](./docs/BRANCH-PROTECTION.md)
- **Changelog**：[`CHANGELOG.md`](./CHANGELOG.md)（Keep a Changelog 格式 · 版本 release 时更新）

---

## 🔗 相关文档

- 战略计划：`docs/implementation-plan.md`
- 架构决策：`docs/adr/`
- Task 框架：`docs/tasks/README.md`
- Spike 报告：`docs/spikes/` · Phase 3 建立
- Session 历史：`docs/session-history/` · Phase 3 建立

---

**本文件 Phase 3 建立（2026-04-18）· 随项目演进更新**
