# Session 启动手册

> 给**人类**看的阶段感知版本。Agent 入口在根目录 `CLAUDE.md`（纯规则）。
> 本文件讲"当前阶段怎么接手 + 具体 Playbook + 常见问题"，不重复 CLAUDE.md 的规则 / 决策 / 禁区。

---

## ⚠️ 当前阶段：Pre-code Phase 1-4 全交付 · Spike W0 可启动（2026-04-18 之后）

**仓库仍无代码**（第一行代码在 SPIKE-01 启动时产生），但 Phase 1-4 全套规划 / spec / 治理 / CI 基础设施**已完整交付**：

- ✅ `docs/tasks/` × 27（7 SPIKE + 20 MVP，全 frontmatter 合法）
- ✅ `docs/adr/` × 10（6 accepted + 4 proposed pending Spike）
- ✅ `CONTRIBUTING.md` / `CODE_OF_CONDUCT.md` / `CHANGELOG.md`
- ✅ `.github/` 全套（4 issue 模板 + PR 模板 + 3 workflows + dependabot）
- ✅ `docs/spikes/` + `docs/spike-artifacts/` + `docs/session-history/` 三个 per-task 目录
- ⚠️ `pnpm tauri dev` / `cargo build` 仍会失败（SPIKE-01 启动后才有骨架）
- ⚠️ main 分支保护**未应用**（用户暂缓 · 见 `docs/BRANCH-PROTECTION.md`）

**先读 `docs/PROGRESS.md` 的"📍 当前位置"和"🔜 下一步"获取最新状态。**

---

## 🏁 今天立即能做的（3 选 1）

### 选项 A · 启动 Spike W0 Day 1（推荐）
认领 `docs/tasks/SPIKE-01-tauri-three-platform-boot.md`：
- 在 mac + Ubuntu 24 Wayland + X11 三平台跑 Tauri 2 空壳启动
- 量化测冷启动耗时（mac < 2s · Linux < 3s）+ IME 录屏
- 产出 `docs/spikes/SPIKE-01-report.md` + `docs/spike-artifacts/SPIKE-01/*.mp4`
- **如果只有单平台**：先做 mac 半边，标记 Ubuntu 部分 `pending-cross-platform` 等接力

### 选项 B · 帮 draft spec 升级 ready（不在 W0 关键路径上）
当前 27 个 spec 大部分仍是 draft。挑 v0.2/v0.3/v1.0 的 MVP（如 MVP-12..20）做独立评审：
- 走 `docs/tasks/README.md` 第 7 步流程
- 演练 `draft → ready` 翻转 gate（reviewer push 翻转 commit 推荐）

### 选项 C · 遇到新决策时提议 ADR-011+
按 `docs/adr/_template.md`：
- ≥ 2 候选选项 + 正面/负面/风险
- 独立评审 + **用户拍板 gate**（B → A 升级硬阻塞）

---

## 📖 上手流程（phase-aware）

### Pre-code · Spike W0 阶段（当前 · 2026-04-18 之后）

**权威流程在 `CLAUDE.md` "🚀 新 Agent 首次启动（5 步）"**。本文件不复述，只补当前阶段的具体动作（认领 Spike / 写 spec PR / 翻转 gate）。

```
阅读（对齐 AGENTS.md / CLAUDE.md 5 跳 onboarding）：
  1. AGENTS.md                    (1 分钟) — 工具无关入口 · 路由
  2. CLAUDE.md                    (3 分钟) — 项目权威规则 / 决策表 / 禁区
  3. docs/PROGRESS.md             (2 分钟) — 当前位置 + 下一步
  4. docs/tasks/README.md         (3 分钟) — 任务索引 + 状态流转 + 翻转 gate

动作（按 CLAUDE.md 第 5 步导游）：
  5. 三选一：
     A. 启动 SPIKE-01 Tauri 三平台空壳（status: ready · 直接 claim · PR #18 已翻转）
     B. 帮某个 v0.2/v0.3/v1.0 draft spec（MVP-12..20）走独立评审升 ready
     C. 提议新 ADR（按 docs/adr/_template.md · 含用户拍板 gate）

  6. 实施：
     a. git checkout -b <scope>/<task-id>      # spike/SPIKE-01 / docs/spec-flip-MVP-15 / 等
     b. 首 commit 改 owner + status: in-progress（如认领 task）
     c. 实施 commits（Conventional Commits + 中文 + Co-authored-by）
     d. push + gh pr create（PR body 必填 Implemented by / Reviewed by）
     e. 独立评审（≠ 原作者）approve
     f. 翻转 gate（二选一）：
        (a) Reviewer 自己 push 翻转 commit（推荐 · 防作者私自改 spec）
        (b) Author push 翻转 + Reviewer re-approve 最新 HEAD
     g. merge

收尾：
  7. Session end 前更新 PROGRESS.md（Active branch / Latest commit / Next action）
```

> ⚠️ **任何 commit 都走 PR，不直接 push main**。claim / 状态变更 / stale 释放等都走 PR。
> ⚠️ 流程细节（状态机、blocked 恢复规则）以 `docs/tasks/README.md` "🔄 状态流转"为权威。

### Code-ready 阶段（Spike W0 完成后）

```
1. 读 CLAUDE.md + docs/PROGRESS.md
2. 环境自检：rustc --version / node --version / pnpm --version
3. pnpm install && pnpm tauri dev
4. 去 docs/tasks/README.md 挑 status: ready 的任务
5. feat/<task-id> 分支 + 按 spec Acceptance 开发 → PR
```

---

## 📁 当前仓库结构（Pre-code Phase 1-4 全交付）

```
vibestation/
├── AGENTS.md                     工具无关 agent 入口（路由到 CLAUDE.md）
├── CLAUDE.md                     项目权威单文件入口（规则 / 决策 / 禁区）
├── README.md                     对外（规划期）
├── LICENSE / NOTICE              Apache 2.0
├── CONTRIBUTING.md               贡献指南 + 用户拍板 gate
├── CODE_OF_CONDUCT.md            Contributor Covenant 2.1 中文
├── CHANGELOG.md                  Keep a Changelog（release-please 维护）
├── .gitignore
├── .github/
│   ├── ISSUE_TEMPLATE/           4 模板（config / bug / feature / task_spec_proposal）
│   ├── PULL_REQUEST_TEMPLATE.md  强制 Implemented by / Reviewed by / 翻转 gate
│   ├── dependabot.yml            cargo + npm + github-actions 周更
│   └── workflows/                ci.yml · secret-scan.yml · task-spec-validator.yml
├── scripts/
│   └── validate-task-spec.mjs    frontmatter validator + 9 条 self-test
├── docs/
│   ├── SESSION-STARTUP.md        本文件 · 人类启动手册
│   ├── PROGRESS.md               滚动进度快照
│   ├── BRANCH-PROTECTION.md      admin 分支保护 checklist
│   ├── implementation-plan.md    v2 战略计划（14 章 + 附录）
│   ├── codex-review-and-response.md
│   ├── tech-research.md
│   ├── tasks/                    27 task spec（3 ready: SPIKE-01/02/MVP-01 · 24 draft 按需翻转）
│   │   ├── README.md · _template.md
│   │   ├── SPIKE-01..07-*.md
│   │   └── MVP-01..20-*.md
│   ├── adr/                      10 ADR（6 accepted + 4 proposed pending Spike）
│   │   ├── README.md · _template.md
│   │   └── ADR-001..010-*.md
│   ├── spikes/                   per-task SPIKE 报告目录（SPIKE-NN-report.md · 待 Spike 启动后产出）
│   ├── spike-artifacts/          per-task 录屏/截图目录（<SPIKE-NN>/*.png/mp4 · 待 Spike 启动后产出）
│   └── session-history/          session 归档（Phase 3 建立）
└── design/
    ├── index.html · directions/ · logos/   Calm Studio 视觉定稿
```

**Spike W0 启动后会新增**：`src-tauri/` + `web/` + `crates/`（实际代码骨架 · 由 SPIKE-01 / MVP-01 创建）+ `docs/ENV-SETUP.md`。

---

## 🎯 常见任务 Playbook

### 修一个 Bug

```
1. 确认可复现 → 最小复现脚本
2. docs/tasks/ 找 spec；没有就新建 BUG-NNN-<slug>.md
3. git checkout -b fix/<slug>
4. TDD：先写失败测试
5. 修代码 → 测试过
6. 本地：cargo test + cargo clippy + pnpm lint
7. commit: fix(<scope>): 中文描述（带 Co-authored-by trailer）
8. gh pr create
```

### 加一个 Feature

```
1. 写 spec（_template.md）放 docs/tasks/
2. 跨 2+ crate / 新 IPC / 架构变动 → 先写 ADR 放 docs/adr/
3. git checkout -b feat/<task-id>
4. 按 Acceptance 开发
5. 更新 docs/PROGRESS.md（session end 前）
6. gh pr create 并 PR body 引用 task spec 路径
```

### Review 一个 PR

```
1. 读 PR 描述里的 task spec / ADR
2. 按 Acceptance 逐项对照 diff
3. 检查 CI：cargo test / clippy / rustfmt / pnpm lint / typecheck
4. 检查 unsafe / unwrap（需要注释理由）
5. 检查硬编码密钥 / console.log / 无 issue 的 TODO
6. commit 是否符合 Conventional Commits + 带 Co-authored-by trailer
7. 评论分级：Critical（阻塞）/ Warning（建议修）/ Info（可忽略）
```

### 处理 Merge Conflict

```
1. git fetch origin && git rebase origin/main
2. 冲突逐个解决：
   - 普通文件 → 保留目标意图
   - append-only 列表（风险登记 / task 列表）→ 保留两方改动
   - scalar 字段（PROGRESS Active branch / 决策表单值）→ Arbiter（用户）仲裁写新值
3. git rebase --continue
4. git push --force-with-lease（仅对自己的 feature 分支）
```

特殊：`Cargo.lock` / `pnpm-lock.yaml` 冲突 → `rm *.lock && cargo build` 或 `pnpm install`。

### 回滚错误 Commit

```
本地未 push：
  git reset --soft HEAD~1   # 保留修改
  git reset --hard HEAD~1   # 丢弃修改

已 push feature 分支：
  git revert <commit-hash> + git push

已合并 main：
  gh pr create 一个 revert PR，git revert -m 1 <merge-commit-hash>

redb 数据损坏：
  cp ~/.config/vibestation/backups/*.redb ~/.config/vibestation/data.redb
```

### Spike Fail 切换 Fallback

```
Tauri on Wayland 失败（Day 2）：
  1. docs/spikes/SPIKE-02-report.md 标红 R12
  2. docs/adr/ADR-002 supersede：Tauri 2 → Electron 28+
  3. 更新 CLAUDE.md 决策表 #12 从 B 栏移 A 栏
  4. 更新 implementation-plan.md §3.1

redb benchmark 输给 rusqlite（Day 6）：
  1. docs/spikes/SPIKE-04-report.md 标 R27
  2. ADR-005 supersede → rusqlite
  3. 更新 CLAUDE.md #14

git2 读慢（Day 4）：
  1. 引入 gix 0.70 混用
  2. ADR-004 改为"git2 写 + gix 读"
```

### 多 Agent 并发协作

本项目不绑定具体 agent 工具。多个 agent 并发时按 `CLAUDE.md` **"🤝 多 Agent 协作（简版）"**的 5 条规则即可（禁 push main / trailer 标身份 / PR 双填 / 独立评审 / scalar 冲突找 Arbiter）。

**复杂治理（任务 claim / stale 释放 / Authority files 严格序列化）**：Phase 2-4 真实遇到并发问题时再加，现在不设计。

---

## ❓ FAQ（阶段导向）

### Q1：为什么 `pnpm tauri dev` 会失败？

仍处 **pre-code 阶段**（无 `package.json` / `src-tauri/` / `crates/`）。SPIKE-01 启动后会在 `spike-tmp/spike-01-tauri/` 建第一个 Tauri 骨架（`.gitignore` 已排除，作者本地 scratchpad），MVP-01 后才把生产骨架并入主仓库。

### Q2：我今天能做的第一件事是什么？

三选一，见本文件顶部 **"🏁 今天立即能做的"**：启动 SPIKE-01 / 帮 draft spec 升 ready / 提议新 ADR。

### Q3：`CLAUDE.md` 里引用的 `docs/adr/ADR-NNN` 在哪？

`docs/adr/` 目录已在 Phase 3 建立，当前 10 个 ADR（ADR-001..010）齐全。索引见 `docs/adr/README.md`。

### Q4：我要修一个已锁决策（A 栏）怎么办？

`CLAUDE.md` "🚫 禁区" 末尾 ⚠️ 条款：新开 ADR → 独立评审（不同 agent + 用户）→ 同步 CLAUDE.md + implementation-plan.md。

### Q5：为什么有 `AGENTS.md` / `CLAUDE.md` / 本文件三份？

| 文件 | 读者 | 内容 |
|------|------|------|
| `AGENTS.md` | **任意 agent CLI**（Codex / Cursor / Aider / OpenCode / 自建 …）| 工具无关入口 · 极简 · 路由到 CLAUDE.md |
| `CLAUDE.md` | **Agent**（Claude Code 自动加载 · 也是项目权威单文件入口）| 稳定规则 + 决策 + 禁区 + 命令速查 |
| `SESSION-STARTUP.md` | **人类** | 当前阶段状态 + Playbook + FAQ |

### Q6：每周投入 < 10 小时怎么办？

触发 `implementation-plan.md` §10.5 **降级树**：
- ≤ 15h：砍 iTerm2/Alacritty 配置导入（只留 Ghostty）、砍 Pane 分屏
- ≤ 10h：仅保留多 Tab + Git Log/Status 只读
- 连续 2 周 < 5h：hibernation，README 公开项目节奏

---

## 📮 贡献入口

- Bug：GitHub Issue `bug_report` 模板（已建 · `.github/ISSUE_TEMPLATE/bug_report.yml`）
- Feature：GitHub Issue `feature_request` 模板（已建）→ 写 spec 放 `docs/tasks/` → PR
- Task spec 提议：GitHub Issue `task_spec_proposal` 模板（已建）
- 代码规范：[`CONTRIBUTING.md`](../CONTRIBUTING.md)（已建）
- 行为准则：[`CODE_OF_CONDUCT.md`](../CODE_OF_CONDUCT.md)（Contributor Covenant 2.1 中文 · 已建）
- 不签 CLA（Apache 2.0 本身有 patent grant）

---

**本文件每次 Phase 完成或阶段切换时更新。当前版本对应 Phase 1 v4 simplified（2026-04-18，砍掉过度设计，只保留 Git 普世最佳实践）。**
