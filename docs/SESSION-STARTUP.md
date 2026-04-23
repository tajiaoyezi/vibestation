# Session 启动手册

> 给**人类**看的阶段感知版本。Agent 入口在根目录 `CLAUDE.md`（纯规则）。
> 本文件讲"当前阶段怎么接手 + 具体 Playbook + 常见问题"，不重复 CLAUDE.md 的规则 / 决策 / 禁区。

---

## ⚠️ 当前阶段：session 17 · MVP-04 Phase F 收口 + MVP-08 Phase A/B 已落地（2026-04-23）

**仓库已进入代码实施中段**，`main` 当前已合入 `PR #99/#100/#101/#102`：

- ✅ `crates/app/` + `crates/core/` + `web/` 生产代码已在仓库中
- ✅ `pnpm tauri:dev` / `cargo build` / `pnpm typecheck` 为可执行路径，不再是 pre-code 空仓库
- ✅ `MVP-02/03/07` 已 done
- ✅ `MVP-04` 已完成 Phase A/B/C/E/F，仅剩 Phase D shell 兼容
- ✅ `MVP-08` 已完成 Phase A/B，当前主线进入 Phase C（Diff 视图前端）
- ⚠️ 外部阻塞仍是 Ubuntu 24 环境与 Apple Developer Program；不阻塞当前主线
- ⚠️ main 分支保护仍未应用（用户暂缓 · 见 `docs/BRANCH-PROTECTION.md`）

**先读 `docs/PROGRESS.md` 的"📍 当前位置"和"🔜 下一步"获取最新状态。**

---

## 🏁 今天立即能做的（按优先级）

### 选项 A · 继续主线：MVP-08 Phase C（推荐）
认领 `docs/tasks/MVP-08-diff-and-git-status.md`：
- 复用已落地 `diff_compute` / `diff_get_settings` / Git Status 面板 contract
- 主区接入 Diff 视图前端：split/unified 切换、行号、binary 提示、大文件 fallback
- 把 Git Status / Git Log 的文件点击真正接到 Diff 视图

### 选项 B · 收口主线：MVP-08 Phase D/E
- `notify` / polling 刷新方案
- runtime 证据 + 性能量化
- 让 MVP-08 达到可交付状态，为 MVP-09 写路径铺路

### 选项 C · 低优先级收尾：MVP-04 Phase D
- 默认 shell / Claude CLI / Codex CLI 实机兼容
- 这是当前终端主链唯一剩余 Phase，但优先级低于 MVP-08 / MVP-09

---

## 📖 上手流程（phase-aware）

### 当前阶段（代码已落地 · 2026-04-23）

**权威流程在 `CLAUDE.md` "🚀 新 Agent 首次启动（5 步）"**。本文件不复述，只补当前阶段的具体动作（认领主线 task / 实施 / 收尾）。

```
阅读（对齐 AGENTS.md / CLAUDE.md 5 跳 onboarding）：
  1. AGENTS.md                    (1 分钟) — 工具无关入口 · 路由
  2. CLAUDE.md                    (3 分钟) — 项目权威规则 / 决策表 / 禁区
  3. docs/PROGRESS.md             (2 分钟) — 当前位置 + 下一步
  4. docs/tasks/README.md         (3 分钟) — 任务索引 + 状态流转 + 翻转 gate

动作（按 CLAUDE.md 第 5 步导游）：
  5. 按当前主线顺序挑任务：
     A. MVP-08 Phase C（Diff 视图前端）
     B. MVP-08 Phase D/E（刷新 + 证据量化）
     C. MVP-09（写路径）或 MVP-04 Phase D（低优先）

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

```
1. 读 CLAUDE.md + docs/PROGRESS.md
2. 环境自检：rustc --version / node --version / pnpm --version
3. 必要时：pnpm install && pnpm tauri:dev
4. 去 docs/tasks/README.md 按 ready task + depends_on 已完成筛主线
5. feat/<task-id> 分支 + 按 spec Acceptance 开发 → PR
```

---

## 📁 当前仓库结构（代码已落地）

```
vibestation/
├── AGENTS.md                     工具无关 agent 入口（路由到 CLAUDE.md）
├── CLAUDE.md                     项目权威单文件入口（规则 / 决策 / 禁区）
├── README.md                     仓库首页状态说明
├── LICENSE / NOTICE              Apache 2.0
├── CONTRIBUTING.md               贡献指南 + 用户拍板 gate
├── CODE_OF_CONDUCT.md            Contributor Covenant 2.1 中文
├── CHANGELOG.md                  Keep a Changelog（release-please 维护）
├── .gitignore
├── crates/
│   ├── app/                      Tauri 启动层 / IPC / permissions / capabilities
│   └── core/                     业务核心（workspace / PTY / git / diff / layout）
├── web/
│   ├── src/                      SolidJS 前端（Terminal / Git Log / Git Status）
│   └── package.json
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
│   ├── tasks/                    task spec 索引（当前共 30 个 task）
│   │   ├── README.md · _template.md
│   │   ├── SPIKE-01..07-*.md
│   │   └── MVP-01..20-*.md
│   ├── adr/                      accepted ADR + 模板
│   │   ├── README.md · _template.md
│   │   └── ADR-001..014-*.md
│   ├── spikes/                   Spike 报告归档
│   ├── spike-artifacts/          Spike 录屏 / 截图归档
│   ├── runtime-evidence/         MVP / feature runtime 证据
│   └── session-history/          session 归档（Phase 3 建立）
└── design/
    ├── index.html · directions/ · logos/   Calm Studio 视觉定稿
```

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

数据库异常：
  先备份当前 app data 目录中的 SQLite 数据库，再新开 BUG task；当前存储已锁定为 rusqlite，不再使用 redb
```

### Spike Fail 切换 Fallback

```
Tauri on Wayland 失败（Day 2）：
  1. docs/spikes/SPIKE-02-report.md 标红 R12
  2. docs/adr/ADR-002 supersede：Tauri 2 → Electron 28+
  3. 更新 CLAUDE.md 决策表 #12 从 B 栏移 A 栏
  4. 更新 implementation-plan.md §3.1

rusqlite / git / PTY 这类 Spike 结论已锁定：
  1. 查对应 SPIKE report 与 ADR
  2. 不再把已 accepted ADR 当成待验证选项
  3. 新变更走 ADR supersede，而不是重跑已完成 Spike

git2 读慢（Day 4）：
  1. 引入 gix 0.70 混用
  2. ADR-004 改为"git2 写 + gix 读"
```

### 多 Agent 并发协作

本项目不绑定具体 agent 工具。多个 agent 并发时按 `CLAUDE.md` **"🤝 多 Agent 协作（简版）"**的 5 条规则即可（禁 push main / trailer 标身份 / PR 双填 / 独立评审 / scalar 冲突找 Arbiter）。

**复杂治理（任务 claim / stale 释放 / Authority files 严格序列化）**：Phase 2-4 真实遇到并发问题时再加，现在不设计。

---

## ❓ FAQ（阶段导向）

### Q1：为什么 `pnpm tauri:dev` 可能会失败？

当前已经不是 pre-code 阶段；若失败，通常是本地环境问题（Node / pnpm / Rust / Tauri 依赖）或运行时配置问题，而不是“仓库还没有代码”。先按 `CLAUDE.md` 的常用命令做环境自检，再看具体报错。

### Q2：我今天能做的第一件事是什么？

默认先接主线：`MVP-08 Phase C`。若不做主线，再看本文件顶部 **"🏁 今天立即能做的"** 里的 Phase D/E 或 MVP-04 Phase D。

### Q3：`CLAUDE.md` 里引用的 `docs/adr/ADR-NNN` 在哪？

`docs/adr/` 目录已建立，当前 accepted ADR 已扩展到 `ADR-014`。索引见 `docs/adr/README.md`。

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
