# Session 启动手册

> 给**人类**看的阶段感知版本。Agent 入口在根目录 `CLAUDE.md`（纯规则）。
> 本文件讲"当前阶段怎么接手 + 具体 Playbook + 常见问题"，不重复 CLAUDE.md 的规则 / 决策 / 禁区。

---

## ⚠️ 当前阶段：Pre-code（2026-04-18）

**仓库还没有代码**，只有规划文档（1600+ 行）+ 视觉原型（4 份 HTML）。

- `pnpm tauri dev` 会失败（没 package.json 没 src-tauri）
- `cargo build` 会失败（没 Cargo.toml）
- 没有 `docs/tasks/` / `docs/adr/` / `CHANGELOG.md` / `.github/`（Phase 2-4 待建立）

**不要按"开发期 Playbook"操作。先读 `docs/PROGRESS.md` 的"📍 当前位置"和"🔜 下一步"。**

---

## 🏁 今天立即能做的（3 选 1）

### 选项 A：Phase 2 文档升级
补 `docs/tasks/` 框架（task schema + _template + 索引）+ 6 个 Spike task spec（SPIKE-01 到 SPIKE-06）+ MVP 前 10 个详细 spec。参考 `docs/implementation-plan.md` §7 + §10.1。

### 选项 B：Phase 3 文档升级
补 `docs/adr/` × 10（对应 CLAUDE.md 锁定表 A 栏 11 条 + B 栏 4 条）+ `CONTRIBUTING.md` + `CHANGELOG.md`（Keep a Changelog）+ `docs/spikes/`（per-task SPIKE 报告目录）+ `docs/spike-artifacts/`（per-task 录屏/截图目录）。

### 选项 C：Phase 4 基础设施
补 `.github/ISSUE_TEMPLATE/` + `.github/PULL_REQUEST_TEMPLATE.md` + `.github/workflows/ci.yml` 骨架 + `CODE_OF_CONDUCT.md` + `.github/dependabot.yml`。

**Phase 1-4 全部完成后**才启动 Spike Week 0 Day 1（Tauri 骨架搭建）。

---

## 📖 上手流程（phase-aware）

### Pre-code 阶段（当前）

**权威流程在 `CLAUDE.md` "🚀 新 Agent 首次启动（5 步）"**。本文件不复述，只补 Pre-code 阶段的具体动作。

```
阅读（对齐 CLAUDE.md 第 1-3 步）：
  1. CLAUDE.md                    (3 分钟)
  2. docs/PROGRESS.md             (2 分钟)
  3. docs/tasks/README.md         (3 分钟) — Pre-code 期间选文档动作，非业务 task

动作（对齐 CLAUDE.md 第 4-5 步）：
  4. 挑选 Phase 2/3/4 文档升级动作
  5. 按 CLAUDE.md 第 5 步流程（建分支 → 认领 commit → 开工 commits → 收尾 commit）：
     a. git checkout -b docs/phase-<N>-<slug>
     b. 如动作对应某个 task spec → 首 commit 把 task 改 owner/status: in-progress
        如动作是"修文档"无 task → 跳过 claim
     c. 修改文档 → commit（Conventional Commits + 中文描述 + Co-authored-by trailer）
     d. git push -u origin <branch> → gh pr create（body 写 "Implemented by: <agent-id>"）
     e. 独立评审（Codex/Claude 另一实例/人类）approve
     f. 如对应 task → merge 前最后 commit 改 reviewer/status: done
     g. merge

收尾：
  6. Session end 前更新 PROGRESS.md 的 Next concrete action
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

## 📁 当前仓库结构

### Current（已存在）

```
vibestation/
├── CLAUDE.md                     agent 总入口
├── README.md                     对外（规划期最小版）
├── LICENSE                       Apache 2.0
├── NOTICE
├── .gitignore
├── docs/
│   ├── SESSION-STARTUP.md        本文件
│   ├── PROGRESS.md               滚动进度快照
│   ├── implementation-plan.md    v2 战略计划（14 章 1473 行）
│   ├── codex-review-and-response.md
│   └── tech-research.md
└── design/
    ├── index.html · directions/ · logos/
```

### Planned（Phase 2-4 陆续建立）

```
├── CONTRIBUTING.md               (Phase 3)
├── CHANGELOG.md                  (Phase 3)  Keep a Changelog
├── CODE_OF_CONDUCT.md            (Phase 4)  Contributor Covenant 2.1
├── .github/                      (Phase 4)
│   ├── ISSUE_TEMPLATE/*.yml
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── workflows/ci.yml
│   └── dependabot.yml
├── docs/
│   ├── tasks/                    (Phase 2)
│   │   ├── README.md · _template.md · SPIKE-NN-*.md · MVP-NN-*.md
│   ├── adr/                      (Phase 3)  ADR-NNN-<slug>.md
│   ├── spikes/                   (Phase 3)  per-task SPIKE 报告（SPIKE-NN-report.md）
│   ├── spike-artifacts/          (Phase 3)  per-task 录屏/截图（<SPIKE-NN>/*.png/mp4）
│   ├── session-history/          (Phase 3)  session 归档
│   └── ENV-SETUP.md              (Spike W0)
└── src-tauri/ + web/ + crates/   (Spike W0)
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

当前 **pre-code 阶段**，没有 `package.json` / `src-tauri/` / `crates/`。Spike W0 Day 1 后才有骨架。

### Q2：我今天能做的第一件事是什么？

三选一，见本文件顶部 **"🏁 今天立即能做的"**：Phase 2 / 3 / 4 文档升级。

### Q3：`CLAUDE.md` 里引用的 `docs/adr/ADR-NNN` 为什么打不开？

ADR 文件在 Phase 3 才建立。当前锁定依据指向 `docs/implementation-plan.md` 章节。

### Q4：我要修一个已锁决策（A 栏）怎么办？

`CLAUDE.md` "🚫 禁区" 末尾 ⚠️ 条款：新开 ADR → 独立评审（不同 agent + 用户）→ 同步 CLAUDE.md + implementation-plan.md。

### Q5：为什么 `CLAUDE.md` 和本文件两份？

| 文件 | 读者 | 内容 |
|------|------|------|
| `CLAUDE.md` | **Agent**（Claude Code 自动加载）| 稳定规则 + 决策 + 禁区 + 命令速查 |
| `SESSION-STARTUP.md` | **人类** | 当前阶段状态 + Playbook + FAQ |

### Q6：每周投入 < 10 小时怎么办？

触发 `implementation-plan.md` §10.5 **降级树**：
- ≤ 15h：砍 iTerm2/Alacritty 配置导入（只留 Ghostty）、砍 Pane 分屏
- ≤ 10h：仅保留多 Tab + Git Log/Status 只读
- 连续 2 周 < 5h：hibernation，README 公开项目节奏

---

## 📮 贡献入口

- Bug：GitHub Issue `bug_report` 模板（Phase 4 建立）
- Feature：先开 Issue 讨论 → 写 spec 放 `docs/tasks/` → PR
- 代码规范：`CONTRIBUTING.md`（Phase 3 建立）
- 不签 CLA（Apache 2.0 本身有 patent grant）

---

**本文件每次 Phase 完成或阶段切换时更新。当前版本对应 Phase 1 v4 simplified（2026-04-18，砍掉过度设计，只保留 Git 普世最佳实践）。**
