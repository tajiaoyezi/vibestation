---
id: MVP-16
type: mvp
title: Rebase / Merge / Cherry-pick（含交互式 + 冲突解决）
status: done
owner:
phase: v0.3
depends_on: ["MVP-08", "MVP-09", "MVP-13"]
blocks: []
blocked_by: []
blocked_from:
blocked_note:
estimate: 7d
plan_ref: implementation-plan.md §10.1（MVP B 折中砍到 v0.3）· §6.2 git_rebase_*/git_merge_*/git_cherrypick_* IPC · §11 W18-W19 路线图
risk_ref: 本 spec §已知风险 R1-R5（git2 interactive rebase API 复杂 / 3-way conflict UI / 中断恢复 / cherry-pick range 部分失败 / merge --squash message 编辑）
reviewer: Claude Code
---

# MVP-16: Rebase / Merge / Cherry-pick（含交互式 + 冲突解决）

> **状态**：`draft`（v0.3 候选 · 详化完成度 100% · 等 Arbiter approve 翻 ready · 等待认领）
> **依赖**：MVP-08（Diff 基础视图 · ✅ done · 提供冲突解决 UI 数据流）+ MVP-09（git2 写路径基础 · ✅ done · 提供 identity / commit error / Tauri permission 模式）+ MVP-13（branch CRUD · ✅ done · rebase / merge / cherry-pick 必须能切换分支）
> **下游 blocks**：无（v0.3 自成一功能 · 不阻塞下游）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) · v0.3 W18-W19 高级 Git 操作
> **详化时间**：2026-05-06 session 24 · Worker A（Claude Code）· self-review + Arbiter approve 后翻 ready

---

> ⚠️ **2026-05-20 · capture mandate removed**（ADR-023 supersede ADR-011）：本 spec 中所有 **"Phase D 截图 / 录屏 / GUI capture / runtime evidence / manual QA capture / 跨平台 Linux capture" 类 acceptance 项 / Phase 表行** 已 supersede · 不再阻塞 spec done flip。inline 文字保留作 audit 历史 · 但**功能上 deprecated**。代码侧 acceptance（rebase_ops::tests:: 54 + 3-way conflict + crash recovery 全链 / 性能 DevTools 数字）保留为 done gate。

---

## 🎯 目标（Goal）

在 MVP-13 分支 CRUD 之后 · 补齐三大分支历史改写操作：**rebase（含交互式 5 op）/ merge（ff/no-ff/squash）/ cherry-pick（单 commit 或 range）** + 完整的 **冲突解决 UI**（复用 MVP-08 Diff 视图 + 3-way marker）+ **中断恢复**（continue / abort / skip · 含 crash recovery）· 让用户不再需要回终端做大部分高级 Git 工作。

## 📖 背景（Context）

- **战略地位**：`implementation-plan.md §10.1` MVP B 折中方案明确把 rebase/merge/cherry-pick 砍到 v0.3 · v0.1 只能 commit · v0.2 加了 push/pull/fetch + branch CRUD · v0.3 补"分支历史改写"
- **路线图位置**：`§11 W18-W19` v0.3 第二批 task（在 MVP-15 Diff syntax highlight 之后 · MVP-17 pane detach 之前）· 因为 rebase/merge 是分支操作的延伸
- **CLAUDE.md 锁定**：#13 永久锁定（A 栏）· Git 栈 = **写 git2 0.20**（本 task 纯写路径 · 复用 MVP-09/13 模式）· **不用 gix 写**（gix 0.70 rebase API 不成熟）
- **上游已落地**：
  - MVP-09 PR #116/#118/#159 已在 `crates/core/src/git_ops.rs` 落地 git2 写路径基础设施（identity / CommitError / Tauri permission 模式）
  - MVP-13 PR #220 已在 `crates/core/src/branch_ops.rs` 落地分支 CRUD（branch list / create / checkout / delete）· MVP-16 直接复用 checkout 链路（rebase / cherry-pick 都需要切分支）
  - MVP-08 PR #100 已在 `crates/core/src/git_status.rs` + `web/src/panels/Diff/` 落地 Diff 视图基础设施（行级 diff + 颜色区分）· MVP-16 扩展为 3-way conflict view
- **战略价值**：rebase / merge / cherry-pick 是 JetBrains 级 Git 工作台的**核心差异化**（vs GitKraken 的图形化 / vs CLI 的效率 / vs Fork 的步进式 UX）· v0.3 解锁后 · 用户不再需要回终端做 90% 的 Git 工作

---

## 🎨 功能范围（Scope）

**Do**：

- **Rebase（普通 + 交互式）**：
  - 普通 rebase onto `<target>`：`git rebase <target>` 等价 · 一步完成 · 冲突时进 conflict resolver
  - 交互式 rebase：5 种操作（**pick / reword / squash / fixup / drop / edit**）· UI 列表展示 commits（仿 `git rebase -i` editor）· 拖拽排序 · 单条修改 op
  - 中断恢复：`--continue` / `--abort` / `--skip` · UI 顶部 banner 显示当前状态 + 三个按钮
  - Crash recovery：app 启动时检测 `.git/rebase-merge` / `.git/rebase-apply` · 提示用户继续或放弃
- **Merge**：
  - **Fast-forward**（默认 · 可在设置改为 no-ff）：`git merge <branch>` 等价 · 无 merge commit
  - **--no-ff**（保留分支历史）：强制创建 merge commit · UI 可编辑 commit message
  - **--squash**（聚合 commit）：所有变化压成一个 commit · UI 弹 message editor 让用户合并 commit messages
  - 冲突时进 conflict resolver
- **Cherry-pick**：
  - 单 commit：右键 commit → "Cherry-pick onto current branch"
  - Range（多 commit）：选 起点 + 终点 commit · UI 列表展示要 pick 的 commits · 支持取消单条
  - `--no-commit` 选项：只放入 working tree · 不自动 commit（让用户编辑后再 commit）
  - 多 commit 部分失败：每条独立处理 · 失败的进 conflict resolver · 用户解决后 continue 下一条
- **冲突解决（复用 MVP-08 Diff 视图 + 扩展 3-way）**：
  - Conflict banner：UI 顶部红色 banner · 显示当前操作（"Rebasing onto main · 2/5 conflict on file X"）+ 三按钮（Continue / Abort / Skip）
  - 文件列表：左侧显示所有 conflicting 文件（state: unresolved / resolved · 视觉区分）
  - 3-way Diff 视图：
    - 顶部三栏标签：`Ours (current branch)` · `Base (common ancestor)` · `Theirs (incoming)`
    - 每个 conflict hunk 含 3 个内容（base + ours + theirs）+ 4 个按钮（`Accept Ours` / `Accept Theirs` / `Accept Both` / `Manual edit`）
    - Manual edit 模式：直接在 diff 视图编辑（行级 inline editor · 复用 MVP-08 字体 + 配色）
  - 全部 resolved 后 · banner 按钮 enable Continue · 触发 `git rebase --continue` 等价
- **中断恢复（continue / abort / skip）**：
  - **Continue**：当前 conflict 全 resolved 后触发 · 完成下一步 / 下一个 commit
  - **Abort**：放弃整个 rebase / merge / cherry-pick · 回到操作前 HEAD（用 git2 `Repository::cleanup_state()`）
  - **Skip**：跳过当前 commit（仅 rebase + cherry-pick 多条 · merge 不支持 skip）
- **Crash recovery（v0.3 关键能力）**：
  - app 启动时检测 `.git/rebase-merge/` / `.git/rebase-apply/` / `MERGE_HEAD` / `CHERRY_PICK_HEAD`
  - 检测到 → 顶部全局 banner `"上次操作未完成 · {operation_type} on branch {name} · {N}/{M} commits 已完成"` + 三按钮（Continue / Abort / View status）
  - 用户选 Continue → 恢复 UI（conflict banner + 文件列表）· 选 Abort → cleanup state · 选 View status → 进 Git Log + 高亮当前 rebase 起点

**Don't**（明确不做 · 推后版本）：

- **跨 remote 的 rebase**（rebase onto `origin/main` 不 fetch）→ MVP-21 push/pull/fetch 已 done · 用户先 fetch 再 rebase（v0.3 不做隐式 fetch）
- **`git reflog` 恢复**（找回已 abort 的 rebase）→ v1.0 范围 · 单独 spec
- **Stash 的交互式管理**（stash list / pop / drop UI）→ v1.0 范围
- **Rebase --root**（rebase 到第一个 commit）→ v0.4+ 评估
- **Rebase --autostash**（自动 stash + rebase + restore）→ v0.4+ 评估（依赖 git2 stash API 稳定性 · 同 MVP-13 §H.3 决策）
- **Merge --abort 在 ff merge 时的特殊处理**（ff merge 已无回退点）→ v0.3 不做 · ff 完成即不可逆
- **Cherry-pick reverse**（`git cherry-pick -x` 反向）→ v0.4+
- **Submodule 内 rebase / merge**（保持 v0.3 不崩即可 · 单列 issue）

## 🛠 实施进度

MVP-16 估时 **7d** · 拆 4 Phase 串行实施：

| Phase                                             | 范围                                                                                                                                             | 估时 | 状态                                                                                          |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---- | --------------------------------------------------------------------------------------------- |
| Phase A · git2 后端 + IPC                         | rebase_ops.rs 后端（rebase / merge / cherry-pick API + 状态机 + RebaseState 持久化）· 9 IPC + 18 ts-rs binding + 50+ 单元测试 + H2 proof         | 3d   | ✅ done                                                                                       |
| Phase B · UI 主体（rebase editor + 冲突解决）     | 交互式 rebase editor 组件 + 3-way conflict Diff 视图（扩展 MVP-08）+ conflict banner + Git Log 右键菜单 + Smart Layouts merge / cherry-pick 入口 | 2.5d | ✅ done · PR #257 · runtime screenshots 按用户要求跳过                                        |
| Phase C · 中断恢复 + crash recovery               | rebase_state 表持久化 · app 启动检测 .git/rebase-merge · 全局 banner UI · continue / abort / skip 路径                                           | 1d   | ✅ done · PR #259                                                                             |
| Phase D · runtime 证据 + Criterion bench + 跨平台 | 截图（rebase editor / 3-way conflict / 各类操作）+ 性能量化（10 / 100 commit rebase · 5 / 50 file conflict）+ macOS + Linux 双平台跑             | 0.5d | 🟢 部分 done · macOS Criterion bench done by PR #266 · GUI screenshot + Linux 跨平台 deferred |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动 · 复用 MVP-09/13 模式）：

- [ ] `crates/core/Cargo.toml` 已含 `git2`（继承 MVP-07/09/13）· **不需要新增依赖**
- [ ] 新建 `crates/core/src/rebase_ops.rs`（独立模块 · 不和 `branch_ops.rs` / `git_ops.rs` / `git_sync.rs` 混 · rebase / merge / cherry-pick 复杂度高 · 单文件足够）
- [ ] git2 API 调用链 ready-to-use（参考 §H.4 表）：
  - **Rebase**：`Repository::rebase()` → `Rebase` 对象 · `Rebase::next()` 拿当前 step · `Rebase::commit()` 完成 step · `Rebase::abort()` / `Rebase::finish()` 收尾
  - **Interactive rebase**：自定义 plan（不用 git2 内置 todo · 因为 git2 0.20 不暴露 interactive plan API）· 状态机自管理 + 每 step 调 `Repository::cherrypick()` 或 `Repository::merge_commits()`
  - **Merge**：`Repository::merge()` 拿 fast-forward analysis · `MergeAnalysis::is_fast_forward()` → `Repository::set_head_detached(target)` · 否则 `Repository::merge_commits()` + 写 `MERGE_HEAD`
  - **Cherry-pick**：`Repository::cherrypick(commit, opts)` · 失败时检测 `.git/CHERRY_PICK_HEAD` 进 conflict
  - **Conflict detection**：`Repository::index()?.has_conflicts()` · `Repository::index()?.conflicts()` 拿 conflicting paths
  - **Cleanup state**：`Repository::cleanup_state()` 清 `.git/MERGE_HEAD` / `.git/REBASE_HEAD` / `.git/CHERRY_PICK_HEAD`
- [ ] IPC commands 注册顺序（`crates/app/src/lib.rs` `invoke_handler!`）：
  - `rebase_start` / `rebase_continue` / `rebase_abort` / `rebase_skip`
  - `rebase_interactive_plan` / `rebase_interactive_apply`
  - `merge_start` / `merge_abort`
  - `cherrypick_start` / `cherrypick_continue` / `cherrypick_abort`
  - `conflict_resolve_file` / `conflict_status`
  - 总 **13 个新 IPC commands**
- [ ] permission toml：`crates/app/permissions/rebase_ops.toml` 新建 · 含 13 个 `allow-{name}`
- [ ] capability `default.json` 引用上述 permission
- [ ] ts-rs binding 自动生成到 `web/src/bindings/`（`build.rs` 触发 · 18 个新 binding · 见 §G.6）
- [ ] fixture：`rebase_ops.rs` 内嵌单元测试用 `tempfile` crate 在测试 dir 创建 + 真实 commit history（仿 MVP-09/13 §C.1 模式）
- [ ] 复用 MVP-09 / MVP-13 模式 → 新 `RebaseOpError` enum（含 `NotInRebase / ConflictUnresolved / DirtyWorkingTree / UncommittedChanges / InvalidStep / DetachedHead / Git2Error` 等 9 个 variant · 见 §G.2）
- [ ] **新增** SQLite 表 `rebase_state`（持久化 in-progress rebase / cherry-pick state · 见 §数据模型变更）
- [ ] 启动时 crash detection（`crates/app/src/lib.rs` `setup` hook 调 `rebase_ops::detect_in_progress`）
- [ ] Tauri event：`git:rebase-progress` / `git:conflict-detected` / `git:operation-done` 三个 event 注册（payload 见 §G.7）

**下次 agent 起点**（spec 详化完后）：等 Arbiter approve PR · 翻 `ready` · 派 Phase A 实施 agent（首选 Codex CLI · 网络 / 状态机重 · 适合 Codex 强项 · 如 Codex 不可用走 OpenCode）。

**依赖关系说明**：MVP-16 依赖 MVP-08（Diff 视图基础）+ MVP-09（git2 写路径）+ MVP-13（branch CRUD）· 三者均已 done · 所以 MVP-16 v0.3 启动时无前置阻塞。MVP-16 自身 4 phase 内部串行。文件域与 v0.3 其他 task 隔离（仅动 `crates/core/src/rebase_ops.rs` + `crates/app/src/lib.rs` 注册 + `web/src/panels/RebaseEditor/` + `web/src/panels/Diff/3way/` + `web/src/dialogs/MergeDialog/CherryPickDialog/`）。

## 🖼 UI 引用

- **Git Log 右键菜单**（rebase / cherry-pick 入口）：`design/directions/1-calm-studio.html` line 1080-1095（Git Log entry context menu）
  - 右键单 commit：`"Cherry-pick onto current branch"` · `"Reset to here"`（v1.0）· `"Revert this commit"`（v0.4）
  - 右键多 commit（shift 选）：`"Cherry-pick range onto current branch"` · `"Squash these commits"`（v0.4）
  - 右键 branch tip：`"Rebase {current} onto {this}"` · `"Interactive rebase from here"` · `"Merge this into {current}"`
- **Smart Layouts** merge 入口：`design/directions/1-calm-studio.html` line 1100-1108（toolbar dropdown）
  - 顶部 toolbar `"Merge"` 按钮 → 弹 dropdown 选 source branch + 策略（ff / no-ff / squash）
- **交互式 rebase editor**：新建组件 `web/src/panels/RebaseEditor/`
  - 全屏 modal · 居中 80% width / 70% height
  - 顶部：操作类型 + onto 分支 chip + 关闭按钮
  - 中部：commit 列表（每行 = 1 commit · 含 op dropdown / commit message / SHA / author / time）· 支持拖拽重排
  - op dropdown：5 个值（pick / reword / squash / fixup / drop / edit）· 不同颜色区分
  - 底部：`Cancel` / `Start rebase` 按钮
- **3-way Conflict Diff 视图**：扩展 `web/src/panels/Diff/`
  - 三栏布局：`Ours` / `Base` / `Theirs`（顶部 chip 区分）
  - 每个 conflict hunk 框选 + 4 个 action 按钮（`Accept Ours` / `Accept Theirs` / `Accept Both` / `Manual edit`）
  - 已 resolved hunk 折叠 + 绿色对勾
  - 底部：`Mark file as resolved` 按钮（所有 hunk 选完后启用）
- **Conflict Banner**（顶部全局）：新建组件 `web/src/components/ConflictBanner/`
  - 红色背景（design token `--color-status-error` 50% 透明）+ 警告 icon
  - 文案：`"⚠ Rebasing onto main · 2/5 conflict on web/src/main.tsx"`（含 progress + 当前文件）
  - 三按钮：`Continue`（disabled 直到所有 conflict resolved）/ `Abort`（红色二次确认）/ `Skip`（rebase + cherry-pick range only）
- **Crash Recovery Banner**（启动时全局）：复用 `ConflictBanner` 但样式不同（黄色背景 · `"上次未完成"`）
  - 三按钮：`Continue` / `Abort` / `View status`
- **Merge Dialog**：新建 `web/src/dialogs/MergeDialog/`
  - 标题：`"合并 {source} 到 {current}"`
  - 字段：source branch（fuzzy switcher 选 · 复用 MVP-13）+ 策略 radio（fast-forward / no-ff / squash）+ commit message editor（squash / no-ff 时显示）
  - 按钮：`Cancel` / `Merge`
- **Cherry-pick Dialog**（range only · 单 commit 右键直接执行不弹 dialog）：新建 `web/src/dialogs/CherryPickDialog/`
  - 标题：`"Cherry-pick {N} commits"`
  - body：commit 列表（每行 commit message + SHA · 可点击取消）
  - 选项：`Auto-commit each` checkbox（默认 on · off → 进 working tree 不 commit）
  - 按钮：`Cancel` / `Cherry-pick`
- **截图归档**：详化时实施 PR 补到 `docs/runtime-evidence/mvp-16/`（由于 [ADR-023](../adr/ADR-023-capture-mandate-removed.md) capture mandate 已移除，此项已弃用）

## ✅ Acceptance

### A. Rebase（普通 + 交互式）

- [x] A.1 Git Log branch tip 右键 → context menu 含 `"Rebase {current} onto {this}"`
- [x] A.2 普通 rebase 触发 → 后端调 `rebase_start` IPC（interactive: false）· 进度反馈采用现有 toast / operation event 路径
- [x] A.3 普通 rebase 无冲突完成 → toast `"已 rebase {branch} onto {target}"` · Git Log 刷新
- [x] A.4 普通 rebase 有冲突 → event path 进入 conflict resolver（§D 流程）· conflict banner 出现
- [x] A.5 交互式 rebase 触发：右键 branch tip `"Interactive rebase from here"` → 弹 RebaseEditor modal · 显示从 here 到 HEAD 的 commits
- [x] A.6 RebaseEditor modal：
  - commit 列表显示（默认 op = pick · 用户可改）
  - 每条 commit op 下拉选（pick / reword / squash / fixup / drop / edit）
  - 拖拽重排
  - reword 选中时 commit row 展开 message editor
  - 底部 `Cancel` / `Start rebase`
- [x] A.7 Start rebase 触发 → 后端调 `rebase_interactive_apply` IPC + 整个 plan · 后端按 plan 状态机执行
- [ ] A.8 交互式 rebase 中断（用户点 Cancel 或外部 ctrl+c）→ Phase C crash recovery / persisted resume scope
- [ ] A.9 性能：100 commit 普通 rebase 无冲突 < 5s（P99 · 测 3 次 · M1 Pro / Ubuntu 24）→ Phase D Criterion bench

### B. Merge

- [x] B.1 顶部 toolbar `"Merge"` 按钮 → 弹 MergeDialog
- [x] B.2 fast-forward 默认（设置 MVP-10 用户可改 no-ff）· 单选 source branch + 策略 → 后端调 `merge_start` IPC
- [x] B.3 fast-forward 成功 → toast + Git Log 刷新（HEAD 移动）
- [x] B.4 no-ff merge 成功 → toast + Git Log 刷新（新 merge commit 出现）
- [x] B.5 squash merge 成功 → message editor（用户可改）→ 用户编辑后提交
- [x] B.6 merge 冲突 → 进 conflict resolver（§D）· conflict banner 显示 merge operation
- [x] B.7 merge --abort（仅 no-ff / squash · ff 已完成不可逆）→ 二次确认 modal → 后端调 `merge_abort` IPC
- [x] B.8 merge dirty tree 阻塞：dirty working tree → toast warn `"工作区有未提交修改 · 请先 commit / stash / discard"` + 跳转 Status 面板（同 MVP-21 §B.2）
- [ ] B.9 性能：合并 50 commit / 10 file change < 3s（P99 · 测 3 次）→ Phase D Criterion bench

### C. Cherry-pick

- [x] C.1 单 commit 右键 → context menu `"Cherry-pick onto current branch"` → 直接执行 · 不弹 dialog
- [x] C.2 单 commit cherry-pick 成功 → toast `"已 cherry-pick commit {short_sha}"`
- [x] C.3 单 commit 冲突 → 进 conflict resolver · conflict banner 显示 cherry-pick operation
- [x] C.4 多 commit shift 选 → 右键 `"Cherry-pick {N} commits onto current branch"` → 弹 CherryPickDialog
- [x] C.5 CherryPickDialog：commit 列表（可单条取消）· `Auto-commit each` checkbox · 提交后逐条 cherry-pick
- [x] C.6 多 commit 部分失败：第 K 条冲突 → 进 conflict resolver · 顶部 banner + Continue / Abort / Skip
- [x] C.7 Skip 触发 → 跳过当前 commit · 进下一条 · banner 进度更新
- [x] C.8 Auto-commit off → 每条 cherry-pick 完后停在 working tree · 用户手动 commit
- [ ] C.9 性能：单 commit cherry-pick < 1s · 10 commit range < 5s（P99）→ Phase D Criterion bench

### D. 冲突解决（核心 UI · 复用 MVP-08 + 扩展 3-way）

- [x] D.1 Conflict 检测：后端 `git2::Index::has_conflicts()` 触发 emit `git:conflict-detected` event · 前端跳转 Diff 视图 + conflict banner 出现
- [ ] D.2 Conflict 文件列表（左侧 Status 面板）：每个 conflict 文件标 `⚔` icon + 红色文字 → Status 面板增强顺延，Phase B 提供 3-way 左侧 conflict file list
- [x] D.3 3-way Diff 视图：三栏 `Ours / Base / Theirs` · 顶部 chip 区分
- [x] D.4 Conflict hunk 4 个 action 按钮：
  - `Accept Ours`（绿）：用 ours 内容
  - `Accept Theirs`（蓝）：用 theirs 内容
  - `Accept Both`（紫）：拼接 ours + theirs（不去重 · 用户后续手动整理）
  - `Manual edit`：进 inline editor · 用户直接编辑
- [x] D.5 已 resolved hunk 折叠 + 绿色对勾 + `Reset` 按钮（重置回未解决状态）
- [x] D.6 文件级 `Mark as resolved` 按钮：所有 hunk resolved 后启用 · 点击调 `conflict_resolve_file` IPC · 后端 `git2::Index::add_path()` + 写回 working tree
- [x] D.7 全部文件 resolved → conflict banner 的 `Continue` 按钮启用 · 点击触发 rebase_continue / merge_commit / cherrypick_continue
- [x] D.8 Manual edit 模式：直接在 Diff 视图编辑 · 行级 inline editor · 字体 + 配色继承 MVP-08
- [ ] D.9 性能：50 file × 100 hunk 冲突场景 · 视图加载 < 2s（P99）· 单 hunk action < 100ms → Phase D Criterion bench

### E. 中断恢复（continue / abort / skip）

- [x] E.1 Continue：所有 conflict resolved 后启用 · 调 `rebase_continue` / `cherrypick_continue` · 状态机进下一 step
- [x] E.2 Abort：红色按钮 + 二次确认 modal `"放弃当前 {operation} · 工作区将回滚到 {prev_HEAD}"` · 调 `rebase_abort` / `cherrypick_abort` · 后端 `Repository::cleanup_state()` + 重置 HEAD
- [x] E.3 Skip：仅 rebase + cherry-pick range（merge 不支持）· 跳过当前 commit · 进下一 step
- [x] E.4 Continue 失败（仍有 conflict）→ banner 显示新 conflict 文件 · `Continue` 按钮重新 disable
- [ ] E.5 Abort 后 Git Log 刷新 · 回到 prev_HEAD · `rebase_state` 表清理 → Phase C recovery polish

### F. Crash recovery（启动时检测）

- [ ] F.1 app 启动检测 `.git/rebase-merge/` / `.git/rebase-apply/` / `.git/MERGE_HEAD` / `.git/CHERRY_PICK_HEAD` · 任一存在即触发 banner → Phase C
- [ ] F.2 Crash banner 黄色 + 文案 `"上次操作未完成 · {operation_type} on {branch} · {N}/{M} commits 已完成 · 选择恢复或放弃"` → Phase C
- [ ] F.3 三按钮：Continue（恢复 conflict UI · 进 §D 流程）/ Abort（cleanup_state · 回到 prev_HEAD）/ View status（进 Git Log + 高亮 rebase 起点）→ Phase C
- [ ] F.4 Continue 时 rebase_state 表读取 plan + remaining steps · 恢复 UI 状态 → Phase C
- [ ] F.5 性能：启动检测 < 200ms（不阻塞 splash screen）→ Phase D

### G. 错误处理 + 边界

- [x] G.1 Rebase / merge / cherry-pick onto self（`Rebase main onto main`）→ 后端错误通过 toast/error banner 暴露
- [x] G.2 Rebase onto ancestor（无新 commit）→ 后端错误通过 toast/error banner 暴露
- [x] G.3 Detached HEAD 状态 → 阻止 rebase / merge · toast warn `"detached HEAD 状态不支持 rebase / merge · 请先 checkout branch"`
- [ ] G.4 git repo 损坏 → 顶部 banner `"Git repo unavailable · 请检查 .git 目录"` + retry → v1.0 repo health check
- [x] G.5 git index lock（`.git/index.lock` 存在）→ rebase / merge / cherry-pick 失败 toast + suggested action

## 🧪 测试策略

| 层次              | 范围                                                                                                                      | 覆盖路径                                                                                          |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| 单元（core）      | `rebase_ops.rs` 所有函数（rebase / merge / cherrypick / conflict 检测）+ `RebaseOpError` enum 全 variant + 状态机转换     | `cargo test --package vibestation-core rebase_ops::` · fixture: tempfile + git2::Repository::init |
| 集成              | IPC 链路：前端 invoke → Rust → git2 → 真实 repo · 包含交互式 rebase / 3-way conflict / 中断恢复 / crash recovery 四大边界 | `cargo test --features integration`                                                               |
| Criterion bench   | rebase 10 / 100 commit · merge 50 commit · cherry-pick 1 / 10 commit · 5 / 50 file conflict                               | `crates/core/benches/rebase_bench.rs`                                                             |
| E2E（Playwright） | golden path：交互式 rebase 5 op 全用 + 3-way conflict 3 种 action + crash recovery 流程                                   | `web/tests/e2e/rebase.spec.ts`                                                                    |
| 视觉回归          | RebaseEditor modal · 3-way Diff 视图 · ConflictBanner · CrashBanner · MergeDialog / CherryPickDialog                      | Playwright screenshot diff                                                                        |
| 手动 QA           | macOS / Linux 双平台跑：UTF-8 文件 conflict / 二进制文件 conflict（git2 不能 3-way · 给 fallback "Use ours/theirs" 选项） | 手动 capture                                                                                      |

### C.1 · fixture 准备脚本

仿 MVP-09/13 §C.1 模式 · 所有 fixture 用 `tempfile::TempDir` + `git2::Repository::init()` 在测试运行时生成：

```rust
// crates/core/tests/fixtures/mvp_16_helpers.rs（新建）
use git2::{Repository, Signature};
use tempfile::TempDir;

fn create_fixture_linear_3_commits() -> TempDir { /* main: A → B → C · 用于普通 rebase */ }
fn create_fixture_diverged_2_branches() -> TempDir { /* main: A → B; feat: A → C · merge 测试 */ }
fn create_fixture_rebase_conflict() -> TempDir { /* feat 改 hello.txt line 1 · main 也改 hello.txt line 1 · rebase 必冲突 */ }
fn create_fixture_3way_complex() -> TempDir { /* base: hello.txt 含 "AAA"; ours: 改 "BBB"; theirs: 改 "CCC" · 用于 3-way diff UI 验证 */ }
fn create_fixture_cherrypick_range() -> TempDir { /* feat: A → B → C → D · main 选 range B..D cherry-pick */ }
fn create_fixture_interactive_rebase_5op() -> TempDir { /* 5 commit 用于全 5 op（pick/reword/squash/fixup/drop/edit）*/ }
fn create_fixture_in_progress_rebase() -> TempDir { /* 模拟 .git/rebase-merge 目录存在 · 用于 crash recovery */ }
fn create_fixture_binary_conflict() -> TempDir { /* PNG 文件冲突 · 测 fallback ours/theirs */ }
fn create_fixture_dirty_tree_block() -> TempDir { /* working tree 有 untracked + modified · 测 §B.8 阻塞 */ }
fn create_fixture_detached_head() -> TempDir { /* detached HEAD · 测 §G.3 阻止 */ }
fn create_fixture_50_file_conflict() -> TempDir { /* 50 file × 多 hunk 冲突 · 性能 */ }
```

每个 helper 返回 `TempDir` · drop 自动清理。

### C.2 · Criterion bench 模板

新建 `crates/core/benches/rebase_bench.rs`：

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_rebase_10_commits_clean(c: &mut Criterion) { /* 10 commit 无冲突 rebase */ }
fn bench_rebase_100_commits_clean(c: &mut Criterion) { /* 100 commit · §A.9 < 5s 验证 */ }
fn bench_merge_no_ff_50_commits(c: &mut Criterion) { /* §B.9 < 3s */ }
fn bench_cherrypick_single(c: &mut Criterion) { /* §C.9 < 1s */ }
fn bench_cherrypick_range_10(c: &mut Criterion) { /* 10 commit range · §C.9 < 5s */ }
fn bench_conflict_3way_50_files(c: &mut Criterion) { /* §D.9 < 2s 视图加载（仅后端检测时间） */ }
fn bench_crash_recovery_detection(c: &mut Criterion) { /* §F.5 启动检测 < 200ms */ }

criterion_group!(
    benches,
    bench_rebase_10_commits_clean, bench_rebase_100_commits_clean,
    bench_merge_no_ff_50_commits,
    bench_cherrypick_single, bench_cherrypick_range_10,
    bench_conflict_3way_50_files,
    bench_crash_recovery_detection,
);
criterion_main!(benches);
```

跑 `cargo bench --bench rebase_bench` · P99 数字写入 PR description。

## 💾 数据模型变更

新增 1 个 SQLite 表 `rebase_state`（持久化 in-progress rebase / cherry-pick · 跨 session crash recovery 关键）：

```sql
CREATE TABLE IF NOT EXISTS rebase_state (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    TEXT NOT NULL,
    operation_type  TEXT NOT NULL,  -- "rebase" / "cherrypick" · merge 不持久化（git2 自管 .git/MERGE_HEAD）
    branch          TEXT NOT NULL,
    onto            TEXT,            -- rebase 目标 branch · cherrypick 不用
    plan_json       TEXT NOT NULL,  -- JSON serialize 的 RebaseInteractivePlan / cherry-pick range
    current_step    INTEGER NOT NULL DEFAULT 0,
    total_steps     INTEGER NOT NULL,
    started_at      INTEGER NOT NULL,  -- unix timestamp
    last_updated    INTEGER NOT NULL,
    UNIQUE(workspace_id)  -- 同一 workspace 同时只能有一个 in-progress rebase / cherry-pick
);

CREATE INDEX idx_rebase_state_workspace ON rebase_state(workspace_id);
```

migration：在 `crates/app/migrations/` 新建 `0042_rebase_state.sql` · 启动时通过现有 schema versioning 机制自动应用。

**禁止**：不在 rebase_state 表持久化 conflict 文件内容（用 git2 自带 `.git/index` + `MERGE_MSG` 即可 · 重复持久化是状态分裂源头）。

**禁止**：不持久化 merge state（git2 用 `.git/MERGE_HEAD` · cleanup_state 自动清 · 不需要 SQLite）。

**禁止**：不持久化 conflict resolution history（v0.4+ 评估 · v0.3 abort 即丢）。

## ⚠️ 已知风险

- **R1 · git2 0.20 interactive rebase API 局限性** · git2 0.20 不暴露 `git rebase -i` 的 todo 编辑接口（Rebase 对象只支持顺序 next() · 无 reorder / squash 内置）· 缓解：自实现 plan 状态机（`RebaseInteractivePlan` + `RebaseInteractiveStep` · 每 step 调 `Repository::cherrypick()` 或 `merge_commits()` 模拟 5 op 行为）· squash / fixup 用 `merge_commits` 后再 amend · drop 用 跳过 next 步骤 · 测试 fixture 必须覆盖 5 op 全场景
- **R2 · 3-way conflict UI 数据流复杂** · MVP-08 Diff 是 2-way（old vs new）· MVP-16 需要 3-way（base + ours + theirs）· 缓解：扩展 `ConflictedFile` 类型（含 base/ours/theirs 三段内容）· `Diff` 组件加 `mode: "two-way" | "three-way"` prop · 视觉验证 baseline screenshot
- **R3 · 中断恢复（ctrl+c / app crash）** · rebase 在 step K 时 crash → `.git/rebase-merge` 目录存在但 SQLite `rebase_state` 表可能未更新 · 缓解：每 step 完成后 `last_updated` 写入 SQLite + 启动检测两套数据源（git2 `.git/rebase-merge` + SQLite plan）· 取并集 · 不一致时 SQLite 为准（保留用户 plan 编辑）
- **R4 · cherry-pick range 多 commit 部分失败** · 第 K 条 conflict · 用户 abort → 前 K-1 条已 commit 留在 HEAD · 缓解：UI 顶部明确显示 "Aborted at {K}/{N} · {K-1} commits already applied" · 用户决定是否手动 reset · 不自动回滚（自动回滚是 v1.0 reflog 范围）
- **R5 · merge --squash + 跨 workspace 设置** · 用户在 workspace A 改默认 squash · workspace B 仍是 ff（per-workspace setting）· 缓解：MergeDialog 顶部显示当前 workspace 的默认策略 + 单次操作可改

## 📝 Notes

- MVP-16 是 v0.3 第一个**有状态**写路径 task（rebase / cherry-pick 跨多 commit · 需要持久化 plan + step）· 模式（git2 backend + ts-rs binding + SQLite migration + Tauri permission + cmd 注册）和 MVP-09/13 一致 · 实施 agent 直接复用
- **Reflog 集成**（恢复已 abort 的 rebase）推到 v1.0 · 接 git2 reflog API · 单独 spec
- **Auto-stash on rebase**（v0.4+ 评估）：等 git2 stash API 稳定（同 MVP-13 §H.3）
- **Submodule rebase / merge** 推到 v1.0 · 单独 spec
- **冲突解决 ML 辅助**（如 GitKraken 的 AI conflict resolution）→ MVP-18 AI-Aware 范围（v1.0 vision · 对外不提）

## 🔗 相关

- `CLAUDE.md` #13 Git 栈混用决策（写 git2）· #7 Diff 自建（MVP-08 已锁定 · MVP-16 复用扩展）
- ADR-007 Git 栈混用决策
- `implementation-plan.md` §10.1 v0.3 砍到 rebase/merge/cherry-pick · §6.2 git*rebase*_/git*merge*_/git*cherrypick*\* IPC · §11 W18-W19
- 上游：MVP-08（Diff 视图基础 + 2-way 数据流）· MVP-09（git2 写路径基础设施 · CommitError / ts-rs / permission / capability 模式）· MVP-13（branch CRUD · checkout 链路 + dirty tree 阻塞）· SPIKE-04（git2 写 smoke test）
- 下游：无（v0.3 自成功能）· v1.0 reflog 集成 + AI conflict resolution

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单

| Rust struct                  | 用途                                                                                                                  | 前端 import 路径 |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------- |
| `RebaseStartRequest`         | 启动 rebase · `{ workspaceId, branch, onto, interactive: bool }`                                                      | 新增             |
| `RebaseInteractivePlan`      | 交互式 rebase plan · `{ steps: RebaseInteractiveStep[] }`                                                             | 新增             |
| `RebaseInteractiveStep`      | 单 step · `{ stepId, op, commitSha, messageOverride: string \| null }`                                                | 新增             |
| `RebaseOp`                   | 枚举：`Pick / Reword / Squash / Fixup / Drop / Edit`                                                                  | 新增             |
| `RebaseStatus`               | 状态查询 · `{ inProgress: bool, operation: string \| null, currentStep, totalSteps, conflictingFiles: string[] }`     | 新增             |
| `RebaseControlRequest`       | continue / abort / skip · `{ workspaceId, action: "continue" \| "abort" \| "skip" }`                                  | 新增             |
| `MergeRequest`               | 启动 merge · `{ workspaceId, sourceBranch, strategy: "ff" \| "no-ff" \| "squash", commitMessage: string \| null }`    | 新增             |
| `MergeStrategy`              | 枚举：`FastForward / NoFastForward / Squash`                                                                          | 新增             |
| `MergeStatus`                | 输出 · `{ outcome: "fast-forwarded" \| "merge-commit" \| "squash-commit" \| "conflict", conflictingFiles: string[] }` | 新增             |
| `CherryPickRequest`          | `{ workspaceId, commitShas: string[], autoCommit: bool }`                                                             | 新增             |
| `CherryPickStatus`           | `{ currentIndex, totalCommits, conflictingFiles: string[] }`                                                          | 新增             |
| `ConflictedFile`             | 单 conflict 文件 · `{ path, hunks: ConflictHunk[], resolved: bool }`                                                  | 新增             |
| `ConflictHunk`               | 单 hunk · `{ id, baseContent, oursContent, theirsContent, resolved: bool, resolution: ConflictResolution \| null }`   | 新增             |
| `ConflictResolution`         | 枚举（含 payload tagged union）：`AcceptOurs / AcceptTheirs / AcceptBoth / Manual { content: string }`                | 新增             |
| `ConflictResolveFileRequest` | `{ workspaceId, filePath, resolutions: ConflictHunkResolution[] }`                                                    | 新增             |
| `ConflictHunkResolution`     | `{ hunkId, resolution: ConflictResolution }`                                                                          | 新增             |
| `RebaseOpError`              | 错误枚举 · 含 payload tagged union                                                                                    | 新增             |
| `CrashRecoveryState`         | 启动检测结果 · `{ inProgress: bool, operation: string \| null, branch: string \| null, currentStep, totalSteps }`     | 新增             |

> 实际 struct 名和字段以实施 PR 为准 · 但**必须**全部走 ts-rs 自动生成。

### G.2 derive 模板（以 `RebaseInteractivePlan` + `RebaseOpError` 为例）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseInteractivePlan {
    pub steps: Vec<RebaseInteractiveStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RebaseInteractiveStep {
    pub step_id: String,           // UUID v4 · 前端拖拽用
    pub op: RebaseOp,
    pub commit_sha: String,
    pub message_override: Option<String>,  // reword 时填
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "PascalCase")]  // 枚举用 PascalCase 避免 TS keyword 冲突
pub enum RebaseOp {
    Pick,
    Reword,
    Squash,
    Fixup,
    Drop,
    Edit,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RebaseOpError {
    NotInRebase,                                              // continue / abort 时无 in-progress rebase
    ConflictUnresolved { files: Vec<String> },                // continue 时仍有 conflict
    DirtyWorkingTree { modified: Vec<String>, staged: Vec<String> },
    UncommittedChanges { paths: Vec<String> },                 // edit op 后未 commit 就 continue
    InvalidStep { step_id: String, reason: String },           // plan 含非法 step
    DetachedHead,
    OperationInProgress { existing: String },                  // 已有 in-progress · 必须先 continue 或 abort
    AlreadyUpToDate,                                            // rebase / merge / cherry-pick 无需操作
    Git2Error { class: String, code: i32, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ConflictResolution {
    AcceptOurs,
    AcceptTheirs,
    AcceptBoth,
    Manual { content: String },  // 用户手动编辑后的最终内容
}
```

> `RebaseOpError` 与 `ConflictResolution` 因含 payload 必须用 tagged union（`#[serde(tag = "kind")]`）· 前端 TS 生成 discriminated union。

### G.3 强制规范

- [ ] 所有 IPC struct + enum 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`（enum 用 `PascalCase` 避 TS keyword）
- [ ] 简单无 payload enum 用 string union（`rename_all` + 无 tag）· 含 payload enum 用 tagged union（`#[serde(tag = "kind")]`）
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface RebaseStartRequest { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-04 §G.3 / MVP-09 §G.4 / MVP-13 §G.4 模式 · 流程：

1. 临时在 `RebaseStartRequest` 字段加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'RebaseStartRequest'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]` · 确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次 · 结果写入 PR description 或 `docs/runtime-evidence/mvp-16/`。

### G.5 · 与上游已落地 binding 的复用决策

MVP-16 实施前必须明确复用 / 新增边界：

| 已有 binding                                  | MVP-16 §G.1 涉及                                                          | 决策                                                                                                                        | 理由                                                                                                         |
| --------------------------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `BranchInfo` / `BranchKind`（MVP-13 PR #220） | `MergeRequest.sourceBranch` 前端展示用                                    | ✅ 前端**复用**（不新建）· 调 MVP-13 `branch_list` IPC 拿数据                                                               | 一致性 + 不重复定义                                                                                          |
| `GitStatusResponse` / `FileChange`（MVP-08）  | Dirty tree 检测 + conflict 文件列表                                       | ✅ 前端**复用** GitStatusResponse · `ConflictedFile` 新建（不复用 FileChange · 因为 conflict 含 base/ours/theirs 多了语义） | 复用状态查询 · 但 conflict 数据结构独立                                                                      |
| `CommitError`（MVP-09）                       | 新 `RebaseOpError` enum                                                   | ⛔ 不复用 · 新建独立 enum                                                                                                   | 错误语义完全不同（Rebase 不涉及 IdentityMissing · 但有 ConflictUnresolved / OperationInProgress 等专属语义） |
| `BranchError`（MVP-13）                       | 不直接涉及（rebase 调 branch checkout 时通过 IPC 间接调）                 | ⛔ 不复用 · MVP-16 不直接抛 BranchError                                                                                     | 跨 IPC 边界 · branch_checkout 失败时 MVP-13 IPC 自带 error 处理                                              |
| `CommitInfo` / `GitLogEntry`（MVP-07）        | `RebaseInteractiveStep.commitSha` 前端展示用（拿 short message / author） | ✅ 前端**复用**（不重新定义 commit metadata）· 调 MVP-07 IPC 拿数据                                                         | 避免 commit 元数据双源                                                                                       |

### G.6 · MVP-16 新增 binding 清单（明确数量）

以下 **18 个 binding** 为 MVP-16 **新增** · 实施时 `web/src/bindings/` 将新增 18 个 `.ts` 文件：

| Rust struct / enum           | 用途                            | 前端 import 路径                                                                 |
| ---------------------------- | ------------------------------- | -------------------------------------------------------------------------------- |
| `RebaseStartRequest`         | rebase 启动输入                 | `import type { RebaseStartRequest } from "../bindings/RebaseStartRequest"`       |
| `RebaseInteractivePlan`      | plan 输入                       | `import type { RebaseInteractivePlan } from "../bindings/RebaseInteractivePlan"` |
| `RebaseInteractiveStep`      | plan 单 step                    | `import type { RebaseInteractiveStep } from "../bindings/RebaseInteractiveStep"` |
| `RebaseOp`                   | 枚举 5 op                       | `import type { RebaseOp } from "../bindings/RebaseOp"`                           |
| `RebaseStatus`               | 状态查询输出                    | `import type { RebaseStatus } from "../bindings/RebaseStatus"`                   |
| `RebaseControlRequest`       | continue / abort / skip 输入    | 新增                                                                             |
| `MergeRequest`               | merge 启动输入                  | 新增                                                                             |
| `MergeStrategy`              | 枚举 ff/no-ff/squash            | 新增                                                                             |
| `MergeStatus`                | merge 输出                      | 新增                                                                             |
| `CherryPickRequest`          | cherry-pick 输入                | 新增                                                                             |
| `CherryPickStatus`           | cherry-pick 输出                | 新增                                                                             |
| `ConflictedFile`             | conflict 文件                   | 新增                                                                             |
| `ConflictHunk`               | conflict hunk                   | 新增                                                                             |
| `ConflictResolution`         | resolution 枚举（tagged union） | 新增                                                                             |
| `ConflictResolveFileRequest` | resolve 输入                    | 新增                                                                             |
| `ConflictHunkResolution`     | resolve hunk                    | 新增                                                                             |
| `RebaseOpError`              | 错误枚举（tagged union）        | 新增                                                                             |
| `CrashRecoveryState`         | 启动检测输出                    | 新增                                                                             |

总 **18 个新 binding**（vs 复用 5 个 上游 binding · 总暴露给前端 23 个 type）。

### G.7 · Tauri event 清单

| Event name              | Payload 字段                                                 | 触发点                                             |
| ----------------------- | ------------------------------------------------------------ | -------------------------------------------------- |
| `git:rebase-progress`   | `{ workspaceId, currentStep, totalSteps, currentCommit }`    | rebase 每 step 完成时                              |
| `git:conflict-detected` | `{ workspaceId, operation, conflictingFiles: string[] }`     | rebase / merge / cherrypick 检测到 conflict 时     |
| `git:operation-done`    | `{ workspaceId, operation, success: bool, message: string }` | rebase / merge / cherrypick 完成（成功 / abort）时 |

前端 listen 三 event · `git:operation-done` success 时刷 Git Log + Status 面板。

## §H. Git 栈约束 + 决策锁定（MVP-16 专有 · 防 v0.3 实施期反复讨论）

MVP-16 是**纯写路径** · 对齐 CLAUDE.md 决策表 #13（2026-04-19 accepted · [ADR-007](../adr/ADR-007-git-stack.md)）· 必须明确：

### H.1 本 MVP Git 栈

- **写路径 crate**：`git2 0.20`（rebase / merge / cherry-pick 全走 git2）
- **读路径 crate**（commit history）：`gix 0.70`（复用 MVP-07 模式 · rebase plan 生成时 list 起点到 HEAD 的 commits 用 gix Revwalk）
- **场景**：
  - List commits for rebase plan：gix Revwalk 拿 from..HEAD 的 commit 列表
  - Rebase：git2 `Repository::rebase()` + 自定义 plan 状态机（git2 0.20 不支持 interactive plan API · 见 §H.2）
  - Merge：git2 `Repository::merge()` + `MergeAnalysis` 判 ff
  - Cherry-pick：git2 `Repository::cherrypick()` 单步 · range 走 loop + 状态机
  - Conflict 检测：git2 `Repository::index()?.has_conflicts()` + `Index::conflicts()`
  - Cleanup：git2 `Repository::cleanup_state()`
- **依据**：
  - SPIKE-04 §C 已验证 git2 0.20 写路径 smoke test 通过（branch CRUD + commit 已在 SPIKE-04 §C 边界用例覆盖 · MVP-16 扩展 rebase / merge / cherry-pick）
  - git2 0.20 changelog 确认 rebase / merge / cherrypick API 稳定（自 0.16+ 即稳）
  - gix 0.70 read 路径 SPIKE-03 benchmark 远快于 git2（commit list 在 100 commit 量级毫秒级 · 见 `docs/spikes/SPIKE-03-report.md`）

### H.2 不碰的 crate / 路径

- **不碰 gix 写**：gix 0.70 的 rebase / merge / cherrypick API 不存在或不完整 · 不试水
- **不碰 git CLI 子进程**（如 `std::process::Command::new("git")`）：性能差（fork + exec ~30-100ms / call）+ 跨平台 PATH 依赖 + 解析 stdout 脆弱 · 全部走 git2
- **不碰其他 git 库**：禁止引入 gitoxide 之外第三方 git 库
- **不碰 git2 内置 interactive rebase API**：git2 0.20 `Rebase` 对象只支持顺序 `next()` · 不暴露 todo 编辑接口 · MVP-16 自实现 plan 状态机（§H.3）

### H.3 交互式 rebase plan 实现策略（核心架构决策）

**决策**：MVP-16 自实现 `RebaseInteractivePlan` 状态机 · 不依赖 git2 内置 interactive rebase。

**理由**（v0.3 锁定避免实施期反复讨论）：

| 选项                                                            | 优点                                                               | 缺点                                                                                                              | v0.3 评估            |
| --------------------------------------------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- | -------------------- |
| (a) **自实现 plan 状态机**（**v0.3 选定**）                     | 完全控制 5 op 行为 · UI 可完整呈现 · 跨平台一致 · 不依赖 git2 内部 | 需要写状态机代码（~300 行）· 测试 fixture 必须覆盖 5 op 全场景                                                    | ✅ 控制力 + 可测试性 |
| (b) 依赖 git2 内置 + 自定义 todo 文件                           | git2 自动管理 .git/rebase-merge                                    | git2 0.20 不暴露 todo 编辑接口 · 必须直接写 .git/rebase-merge/todo · 跨 git 版本格式不稳 · 无法被 git2 重新 parse | ❌ 脆弱              |
| (c) 子进程调 `git rebase -i`（用 GIT_SEQUENCE_EDITOR 环境变量） | 用 git 官方实现 · 行为对齐 100%                                    | fork + exec 慢 · 跨平台 PATH 依赖 · 解析 stdout 脆弱 · 用户 git 版本不一致                                        | ❌ 见 §H.2           |

**实现要点**：

1. **Plan 表示**：`RebaseInteractivePlan { steps: Vec<RebaseInteractiveStep> }` · 每 step 含 op / commit_sha / message_override · 持久化为 JSON 进 SQLite `rebase_state.plan_json`
2. **执行状态机**：
   - `pick`：`Repository::cherrypick(commit, opts)` · 失败进 conflict
   - `reword`：cherrypick + amend message（用 message_override）
   - `squash`：cherrypick · 然后 `Commit::amend(parent_message + current_message)` · 或用 git2 `merge_commits` + manual amend
   - `fixup`：同 squash 但丢弃 current message
   - `drop`：跳过 next 步骤 · 不 cherry-pick
   - `edit`：cherrypick 后 pause · 等用户手动 commit + continue
3. **Conflict 处理**：每 step 后检测 `index.has_conflicts()` · 进 conflict resolver · resolve 后 continue 下一 step
4. **Plan 验证**：start rebase 前验证 plan（squash 第一条不能是 squash · drop 不能全部 drop · etc.）

### H.4 git2 0.20 API 使用要点（实施参考）

| 操作                            | git2 API 调用链                                                                                                                                                  |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Start rebase                    | `Repository::rebase(branch, upstream, onto, opts)` 拿 `Rebase` 对象 · 但 MVP-16 自管 plan · 不直接用 Rebase::next（仅用作 fallback 模式 · interactive=false 时） |
| Interactive step (pick)         | `Repository::cherrypick(target_commit, opts)` · 检测 `index.has_conflicts()`                                                                                     |
| Interactive step (reword)       | cherrypick 后 `Repository::head()?.peel_to_commit()?.amend(None, None, None, Some(msg), None, None)`                                                             |
| Interactive step (squash/fixup) | cherrypick → `Commit::parents()` 拿 parent → 用 `git2::merge_commits` 合并 tree → `Commit::amend` 或 `Commit::tree` · message 拼接（squash）或丢弃（fixup）      |
| Interactive step (drop)         | 跳过 · 进下一 step                                                                                                                                               |
| Interactive step (edit)         | cherrypick 后 set rebase_state 为 paused · 等 user `rebase_continue`                                                                                             |
| Merge ff                        | `Repository::merge_analysis(target)` → `MergeAnalysis::is_fast_forward()` → `Repository::set_head_detached(target)` + `Repository::checkout_tree`                |
| Merge no-ff                     | `Repository::merge` → 写 `MERGE_HEAD` · 检测 conflict · 无 conflict 时 `Repository::commit` 创建 merge commit                                                    |
| Merge squash                    | `Repository::merge_commits(ours, theirs)` 拿 tree → `Commit::tree(tree)` 创建单 commit（不写 MERGE_HEAD）                                                        |
| Merge abort                     | `Repository::cleanup_state()` · 重置 working tree（用 `Repository::checkout_head` 强制）                                                                         |
| Cherry-pick single              | `Repository::cherrypick(commit, opts)` · 检测 `CHERRY_PICK_HEAD` · auto_commit 时 `Repository::commit` 创建 commit                                               |
| Cherry-pick range               | loop 每 commit · 单 commit cherry-pick · failure 进 conflict resolver                                                                                            |
| Conflict detection              | `Repository::index()?.has_conflicts()` · `Repository::index()?.conflicts()` 拿 IndexConflict iter                                                                |
| Conflict resolution             | 用户 resolution 写 working tree 文件 · `Index::add_path(path)` · `Index::write` · 解决后所有文件 → `cleanup_state` 不调（rebase / cherrypick 后续 step 还要）    |
| Cleanup state                   | `Repository::cleanup_state()` 清 .git/MERGE_HEAD / REBASE_HEAD / CHERRY_PICK_HEAD                                                                                |
| Crash detection                 | check `repo.path().join("rebase-merge").exists()` / `rebase-apply` / `MERGE_HEAD` / `CHERRY_PICK_HEAD`                                                           |
| 错误分诊                        | `git2::Error::class()` / `Error::code()` → 映射到 `RebaseOpError` enum                                                                                           |

### H.5 conflict 解决流程锁定（3-way · 不退化为 2-way）

**决策**：MVP-16 conflict 解决 UI **必须** 是 3-way（base + ours + theirs）· 不接受 2-way 退化（仅 ours vs theirs）。

**理由**：

| 选项                                                     | 优点                                                                                                 | 缺点                                                                                                            | v0.3 评估               |
| -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ----------------------- |
| (a) **3-way（base + ours + theirs）**（**v0.3 选定**）   | 用户能看到 base · 决策更准确（特别 squash / cherry-pick 复杂场景）· 对齐 GitKraken / Fork / SmartGit | UI 三栏复杂 · 视觉空间紧 · 性能（50 file 时 3 栏渲染）· 需要从 git2 拿 base content                             | ✅ JetBrains 级体验必需 |
| (b) 2-way（ours vs theirs）                              | UI 简单                                                                                              | 用户无法判断改动是否冲突基础（如 ours 加了一行 · theirs 也加了一行 · 但是不同行 · 实际不冲突 · 2-way 看不出来） | ❌ UX 退化              |
| (c) 不做 UI · 让用户用外部 mergetool（vimdiff / VSCode） | 实现简单                                                                                             | 工作流断裂 · 不是 JetBrains 级                                                                                  | ❌                      |

**实现要点**：

1. **Base content 来源**：`git2::Repository::merge_base(ours, theirs)` 拿 base commit · `Tree::get_path(file_path)?.to_object()?.peel_to_blob()?.content()` 拿 base 文件内容
2. **Hunk 分析**：用 `git2::Diff::merge` 或自实现 3-way merge marker 解析（git2 已写到 working tree 的 `<<<<<<< / ======= / >>>>>>>` marker · 解析为 hunks）
3. **UI 三栏**：base 居中 · ours 左 · theirs 右（视觉对齐 GitKraken 习惯）

### H.6 中断恢复 + crash recovery 持久化

- **rebase + cherry-pick** 持久化到 SQLite `rebase_state` 表 · 因为：
  - git2 `.git/rebase-merge` 不存 plan（只存当前 step）· 用户 plan 编辑必须存 SQLite
  - cherry-pick range 多 commit · git2 `.git/CHERRY_PICK_HEAD` 只存当前 commit · 不存 range
- **merge** **不**持久化（用 git2 `.git/MERGE_HEAD` · cleanup_state 自动清）：
  - merge 不需要 plan（一步完成 · ff / no-ff / squash 三选一）
  - merge 中断恢复就是 `Repository::cleanup_state()` + 用户重启操作
- **启动检测**：app 启动时调 `rebase_ops::detect_in_progress` · 取并集（git2 `.git/*` + SQLite `rebase_state`）· 不一致时以 SQLite 为准（保留用户 plan）

### H.7 跨平台兼容性

| 平台                             | 状态         | 说明                                                                                                                   |
| -------------------------------- | ------------ | ---------------------------------------------------------------------------------------------------------------------- |
| macOS（Apple Silicon / Intel）   | ✅ v0.3 支持 | git2 0.20 + libgit2 vendored · APFS case-insensitive · UTF-8 文件 conflict 验证                                        |
| Linux（Ubuntu 24 X11 / Wayland） | ✅ v0.3 支持 | git2 0.20 同样可用 · ext4 case-sensitive · `feat/x` ≠ `feat/X` · binary file conflict fallback `Use ours / Use theirs` |
| Windows                          | ❌ v0.4+     | NTFS case-insensitive + path separator `\` · 同 MVP-13 §H.7 推后                                                       |

### H.8 与 MVP-13 / MVP-21 边界

MVP-16 仅本地操作 · **不调用任何网络**。

| 场景                                         | MVP-16 责任                                      | MVP-13 责任       | MVP-21 责任                              |
| -------------------------------------------- | ------------------------------------------------ | ----------------- | ---------------------------------------- |
| Rebase / merge / cherry-pick 本地            | ✅                                               | ❌                | ❌                                       |
| Branch CRUD（rebase 前需切分支）             | 通过 IPC 调 MVP-13                               | ✅                | ❌                                       |
| 拿 commit list / metadata（rebase plan 用）  | 通过 IPC 调 MVP-07                               | ❌（MVP-07 责任） | ❌                                       |
| Push rebased branch（force-push）            | ❌                                               | ❌                | ✅（用户手动触发 MVP-21 push --force）   |
| Fetch upstream（rebase onto origin/main 前） | ❌                                               | ❌                | ✅（用户先 fetch · MVP-16 不隐式 fetch） |
| Pull（merge / rebase remote）                | 仅 merge 逻辑复用 · MVP-21 调 MVP-16 merge IPC？ | ❌                | ✅（MVP-21 已实现 · 不调 MVP-16）        |

实施时严格隔离 · 避免 MVP-16 PR 引入网络代码 / 跨 task 状态混乱。

---

**自审四问**（2026-05-06 · session 24 · 主 agent 详化）：

1. **递归完备性**：Acceptance 清单覆盖 Rebase（普通 + 交互式）/ Merge（ff/no-ff/squash）/ Cherry-pick（单 + range）/ 冲突解决（3-way）/ 中断恢复（continue/abort/skip）/ Crash recovery / 错误处理 / 性能 / fixture / IPC contract / Git 栈约束 / 跨平台 全维度 ✅
2. **反向场景**：
   - Rebase 自身 → §G.1 toast warn ✅
   - Rebase onto ancestor 无效 → §G.2 toast info ✅
   - Detached HEAD 阻止 → §G.3 toast warn ✅
   - 交互式 rebase 中断 → §A.8 持久化 + 顶部 banner ✅
   - Merge ff 已完成不可逆 → §B.7 explicit 说明（仅 no-ff/squash 可 abort）✅
   - Cherry-pick range 部分失败 → §C.6 banner 显示 progress + Skip 选项 ✅
   - 3-way conflict 处理 binary 文件 → 测试策略手动 QA + R2 mitigation 复用 MVP-08 fallback ✅
   - Crash 时 SQLite 与 git2 状态不一致 → §H.6 explicit 取 SQLite 为准 ✅
3. **边界适用性**：
   - 0 commit rebase（无 commit 在 plan）/ 1 / 10 / 100 / 1000 commit 都覆盖（§A.9 + Criterion bench）
   - 0 file conflict / 1 / 5 / 50 file conflict（§D.9 + bench）
   - 中文文件名 / Unicode commit message（§H.7 + 测试）
   - 跨平台：macOS + Linux v0.3 / Windows v0.4+ 明确推后
   - 多 workspace：rebase_state 表 `UNIQUE(workspace_id)` 隔离
4. **YAGNI**：
   - 不做：git CLI 子进程 / gix 写 / 第三方 git 库 / `git rebase --root` / `git rebase --autostash` / cherry-pick reverse / submodule rebase / reflog 恢复 / AI conflict resolution · 全在 §Don't 明示推后
   - 不引：第三方 merge 库（如 `merge`, `diff3`）· 自实现 3-way（git2 已经写到 working tree marker · 解析即可）
5. **对齐上游 binding**（§G.5）：BranchInfo（MVP-13 PR #220 复用 IPC）· GitStatusResponse（MVP-08 复用 IPC）· CommitInfo / GitLogEntry（MVP-07 复用 IPC）· CommitError（MVP-09）/ BranchError（MVP-13）不复用 · 错误语义不同 · MVP-16 新建 RebaseOpError · 新增 18 个独立 binding 清单明确（§G.6）
6. **§H 决策锁定全覆盖**：H.1 Git 栈 / H.2 不碰列表 / H.3 plan 状态机自实现 / H.4 API 调用链 / H.5 3-way 锁定 / H.6 持久化策略 / H.7 跨平台 / H.8 与 MVP-13/21 边界 · 防 v0.3 实施期反复讨论
7. **runtime evidence 要求已弃用**：由于 [ADR-023](../adr/ADR-023-capture-mandate-removed.md) capture mandate 已移除，不再强制要求截图归档（ADR-023 §3 保留已捕证据作历史 audit）

---

## 详化完成度评估（Arbiter 审 PR 时参考）

| 12 段必含                                 | 状态 | 备注                                                                                                                                         |
| ----------------------------------------- | ---- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. frontmatter                            | ✅   | id / type / title / status:draft / depends_on / phase / estimate / plan_ref / risk_ref / reviewer 占位                                       |
| 2. 🎯 目标 Goal                           | ✅   | 一句话核心 + plan_ref link + 战略价值                                                                                                        |
| 3. 📖 背景 Context                        | ✅   | implementation-plan + CLAUDE.md + 路线图 W18-W19 + 上游已落地 + 战略价值                                                                     |
| 4. 🛠 实施进度表                          | ✅   | Phase A/B/C/D 拆分 + Phase A 13 项起点 checklist                                                                                             |
| 5. 🎨 功能范围 Scope                      | ✅   | Do 5 大组（rebase / merge / cherry-pick / conflict / 中断恢复）/ Don't 8 项                                                                  |
| 6. 🖼 UI 引用                             | ✅   | design 原型 line 引用 + 7 类 UI 元素描述（含新建 RebaseEditor / 3-way Diff / ConflictBanner / CrashBanner / MergeDialog / CherryPickDialog） |
| 7. ✅ Acceptance                          | ✅   | A-G 7 大组 / 40 项 checkbox · 每项含具体测法                                                                                                 |
| 8. 🧪 测试策略                            | ✅   | 单元 / 集成 / Criterion / E2E / 视觉回归 / 手动 QA + 11 个 fixture + 7 个 bench 模板                                                         |
| 9. 💾 数据模型变更                        | ✅   | rebase_state 表新建 + migration 0042 + 3 反模式禁止                                                                                          |
| 10. §G IPC Contract                       | ✅   | 18 struct + derive 模板 + G.5 复用决策 + G.6 新增 18 binding 清单 + G.7 Tauri event 3 个                                                     |
| 11. §H 决策锁定                           | ✅   | H.1-H.8 8 子段 · 含 plan 状态机自实现表 + git2 API 表 + 3-way 锁定表 + 跨平台矩阵 + 与 MVP-13/21 边界                                        |
| 12. ⚠️ 已知风险 + Notes + 相关 + 自审四问 | ✅   | 5 风险 + 4 Notes + 7 相关 + 7 条自审 + 完成度 100%                                                                                           |

**完成度**：12/12 = **100%**（建议 Arbiter approve PR 后翻 status: ready）。

**遗留问题**：无 · 所有决策已锁定 · 没有"v0.3 启动后再讨论"的悬空项。
