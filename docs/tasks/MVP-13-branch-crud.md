---
id: MVP-13
type: mvp
title: 分支 create / checkout / delete + Fuzzy Switcher
status: ready
owner:
phase: v0.2
depends_on: ["MVP-07", "MVP-09"]
blocks: ["MVP-16", "MVP-21"]
blocked_by: []
blocked_from:
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1（v0.2 砍到分支 CRUD）· §6.2 git_branch_* IPC · §11 W13 路线图
risk_ref: 本 spec §已知风险 R1-R5（git2 stash 不稳定 / undo 局限 / Unicode 兼容 / remote tracking 命名冲突 / force checkout 数据丢失）
reviewer: Claude Code
---

# MVP-13: 分支 create / checkout / delete + Fuzzy Switcher

> **状态**：`ready`（v0.2 候选 · 详化完成度 100% · 2026-05-03 Arbiter approve 翻 ready · 等待认领）
> **依赖**：MVP-07（Git Log 只读 · 分支标签贴 · done）+ MVP-09（git2 写路径已通 · done · 两者均已 done · 无开工阻塞）
> **下游 blocks**：MVP-16（v0.3 rebase/merge/cherry-pick）+ MVP-21（v0.2 push/pull/fetch · 同 sprint 顺序 W13 → W14）
> **战略依据**：[`implementation-plan.md §10.1`](../implementation-plan.md) · v0.2 W13 分支
> **详化时间**：2026-05-01 sprint vibe · Worker B（Claude Code）· 2026-05-03 self-review + frontmatter 翻 ready

---

## 🎯 目标（Goal）

在 Git Log 视图 + Primary Sidebar 分支树支持新建 / 切换 / 删除分支三个基础操作 · 同时提供 `⌘B` 触发的 fuzzy branch switcher · 让用户不再依赖 `git checkout` 终端命令。

## 📖 背景（Context）

- **战略地位**：`implementation-plan.md §10.1` MVP B 折中方案明确把分支 CRUD 砍到 v0.2 · v0.1 只有"看分支标签贴"· v0.2 补"改分支"
- **路线图位置**：`§11 W13` 第一个 v0.2 增量 task（在 W14 push/pull/fetch 之前）· 因为 push/pull 依赖能切分支
- **CLAUDE.md 锁定**：#13 永久锁定（A 栏）· Git 栈 = **写 git2 0.20**（本 task 纯写路径）+ **读 gix 0.70**（branch list 读路径仍走 gix · 复用 MVP-07 模式）
- **上游已落地**：MVP-09 PR #116/#118/#159 已在 `crates/core/src/git_ops.rs` 落地 git2 写路径基础设施（identity / commit error / Tauri permission 模式）· MVP-13 直接复用
- **历史尝试**：v0.1 阶段 OpenCode 曾在 spike-tmp 试 git2 branch API · 验证 `git2::Branch::create / Repository::set_head / git2::Branch::delete` 三组 API 稳定可用

---

## 🎨 功能范围（Scope）

**Do**：

- **Branch list（读）**：使用 MVP-13 PR #220 首次定义的 `BranchInfo` binding（spec §G.5 表 stale assumption 已修正 · 实际仓库 MVP-07 时未生成此 binding）· Primary Sidebar 分支树渲染本地 + remote + tag 三类
- **Branch create（写）**：
  - 默认从 `HEAD` 新建（`git branch <name>` 等价）· 不自动 checkout
  - 可选 `from <other-branch>` 起点（fuzzy switcher 选）
  - 可选 "create and checkout"（`git checkout -b <name>` 等价 · 一步完成）
  - 名字验证：用 `git2::Reference::is_valid_name` 同步校验 · 输入框红色边框 + tooltip 提示
- **Branch checkout（写）**：
  - Clean working tree → 直接切换 · 200ms 内完成
  - Dirty working tree → 弹"切换前需要 stash / discard / cancel"对话框 · 默认 cancel · v0.2 不自动 stash（见 §H.3）
  - Detached HEAD → checkout 已有 branch 后退出 detached 状态
  - Remote 分支 checkout：`origin/feat/x` → 自动建本地 tracking branch `feat/x`（`git2::Branch::set_upstream`）
- **Branch delete（写）**：
  - 安全删除 `git branch -d`：**已合并到当前 HEAD**（reachability check · 见 §H.4）+ 不是当前 branch + 不是 main/master 保护 → 通过；**未合并 → 拒绝 + 抛 `BranchError::Unmerged { name, missing_commits }`**（见 §G.2 enum · 必须先 force=true 才能删）
  - 强制删除 `git branch -D`：UI 二次确认 + 显式 "Force delete (data loss)" 红色按钮 · 5s undo toast（git2 `Reference::delete` 不可逆 · undo 通过缓存的 commit SHA 重建 ref）
- **Fuzzy Switcher**：`⌘B` 弹 modal · 输入过滤 · ↑↓ 选 · Enter checkout · Esc 取消 · 类似 VSCode `⌘P` Quick Open
- **分支树即时刷新**：CRUD 后通过 Tauri event `git:branch-changed` 推送 · 前端 store 增量更新 · 无需手动 refresh

**Don't**（明确不做 · 推后版本）：

- **Branch rename**（`git branch -m`）→ v0.3 MVP-16 同 PR
- **Auto-stash on checkout**（dirty tree 自动 stash 再 checkout）→ v0.2 后期 / v0.3 评估 · 见 §H.3 决策原因
- **Push deleted branch to origin**（`git push origin --delete`）→ MVP-21（push/pull/fetch）范围
- **Branch protection rules**（基于配置阻止删除特定 branch）→ v0.3+ 设置面板范围
- **Cherry-pick / merge / rebase 触发的分支创建**（`git rebase -i` 中创建 fixup branch）→ v0.3 MVP-16 范围
- **Submodule 内分支 CRUD**（保持 v0.2 不崩即可 · v0.3+ 单列 issue）

## 🛠 实施进度

MVP-13 估时 **4d** · 拆 4 Phase 串行实施：

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · git2 写路径后端 + IPC | branch_create / branch_checkout / branch_delete / branch_list 四组 IPC + ts-rs bindings + 单元测试（fixture: tempfile + git2::Repository::init）| ✅ done · PR #220 | #220 |
| Phase B · Primary Sidebar 分支树 UI + 右键菜单 | 复用 design/directions/1-calm-studio.html 分支树结构 · 接 IPC · 右键菜单（New / Checkout / Delete）· dirty tree 提示对话框 | ✅ done · PR #222 | #222 |
| Phase C · Fuzzy Switcher modal | `⌘B` keydown listener · BranchSwitcher 组件 · fuzzy 算法（subsequence 匹配 + 排序：current 置顶 / 最近 5 / 字母序）· Enter checkout 链路 | ✅ done · PR #224 | #224 |
| Phase D · runtime 证据 + 性能量化 | 截图（CRUD 三大操作 · dirty tree 对话框 · Fuzzy Switcher）+ 性能量化（10 / 100 / 1000 branch fixture）放 `docs/runtime-evidence/mvp-13/` | ⏳ todo | — |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动 · 复用 MVP-09 模式）：

- [ ] `crates/core/Cargo.toml` 已含 `git2`（继承 MVP-07/09）· **不需要新增依赖**
- [ ] 新建 `crates/core/src/branch_ops.rs`（不和 `git_status.rs` / `git_ops.rs` 混 · branch 操作独立模块）
- [ ] git2 API 调用链 ready-to-use（参考 §H.4 表）：
  - List：`Repository::branches(Some(BranchType::Local))` + `BranchType::Remote` 各跑一次
  - Create：`Repository::find_commit(target_oid)` → `Repository::branch(name, &commit, force)` → 可选 `Repository::set_head(...)`
  - Checkout：`Repository::find_branch(name, BranchType::Local)` → `Repository::set_head(branch.get().name().unwrap())` + `Repository::checkout_head(opts)`
  - Delete：`Repository::find_branch(name, t)` → `Branch::delete()`
  - Track upstream：`Branch::set_upstream(Some("origin/feat/x"))`
- [ ] IPC commands 注册顺序（`crates/app/src/lib.rs` `invoke_handler!`）：
  - `branch_list` / `branch_create` / `branch_checkout` / `branch_delete` / `branch_switcher_query`
  - 总 **5 个新 IPC commands**
- [ ] permission toml：`crates/app/permissions/branch_ops.toml` 新建 · 含 5 个 `allow-{name}`
- [ ] capability `default.json` 引用上述 permission
- [ ] ts-rs binding 自动生成到 `web/src/bindings/`（`build.rs` 触发 · 12 个新 binding · 含 BranchInfo / BranchKind 最小补齐 · 见 §G.6）
- [ ] fixture：`branch_ops.rs` 内嵌单元测试用 `tempfile` crate 运行时生成 · 不依赖本地物理目录（仿 MVP-09 §C.1）
- [ ] 复用 MVP-09 `CommitError::IdentityMissing` 模式 → 新 `BranchError` enum（含 `InvalidName / NotFound / Unmerged / ProtectedBranch / DetachedHead / DirtyWorkingTree / Git2Error` 7 个 variant）

**下次 agent 起点**（spec 详化完后）：等 Arbiter approve PR · 翻 `ready` · 派 Phase A 实施 agent（首选 OpenCode · 如 OpenCode 不可用走 Codex / Kimi）。

**依赖关系说明**：MVP-13 依赖 MVP-07 done（branch list 读路径） + MVP-09 done（git2 写路径基础）· 两者均已 done · 所以 MVP-13 v0.2 启动时无前置阻塞。MVP-13 自身 4 phase 内部串行。文件域与 MVP-21（push/pull/fetch）**完全隔离**（MVP-13 只动 `crates/core/src/branch_ops.rs` + `crates/app/src/lib.rs` 注册 + `web/src/panels/BranchTree/` + `web/src/dialogs/BranchSwitcher/`）· 可并行启动。

## 🖼 UI 引用

- **Primary Sidebar 分支树**：`design/directions/1-calm-studio.html` line 826-839
  - `<div class="sub-head">Branches <button class="add">+</button></div>`
  - `<div class="tree-row active">├─ <span class="ref-dot local"/> main <span class="badge">↑2</span></div>`
  - `local` / `remote` / `tag` 三种 ref-dot 颜色区分
  - `+` 按钮触发新建 modal · 点击 row 触发 checkout · 右键弹 context menu
- **Fuzzy Switcher modal**：参考 VSCode `⌘P` Quick Open + Linear 的 Branch Switcher
  - 居中浮层 · 600px 宽 · 暗色背景 50% 透明遮罩
  - 顶部 input · 下方列表 · ↑↓ 选 · Enter 确认 · Esc 取消
  - 列表项：分支名（match 字符高亮）+ 类型 chip（local / remote）+ 最近 commit message（截断）
  - 当前分支置顶 + 标记 `(current)`
- **Dirty Tree 对话框**：MVP-09 `CommitBar/IdentityDialog` 同款 modal 模板
  - 标题：`"切换分支前发现未提交修改"`
  - body：列出 N 个 modified / staged 文件（最多前 5 行 + "...还有 N 个"）
  - 三个按钮：`Stash & Switch`（v0.2 暂禁用 + tooltip "v0.2 不支持自动 stash · 请用终端"）/ `Discard & Switch`（红色 · 二次确认）/ `Cancel`（默认）
- **Force Delete 对话框**：
  - 标题：`"强制删除分支 {branch_name}"`
  - body：`"该分支含 N 个未合并 commit · 删除后**无法通过 UI 恢复**"`
  - 红色 `"Force delete (data loss)"` 按钮 + `Cancel` 默认
- **截图归档**：详化时实施 PR 补到 `docs/runtime-evidence/mvp-13/`（按 `.claude/rules/runtime-evidence-location.md` R1 命名）

## ✅ Acceptance

### A. Branch Create

- [ ] A.1 Primary Sidebar 分支树顶部 `+` 按钮点击 → 弹"新建分支" modal · 字段：name（必填）/ from（默认 HEAD · 可选 fuzzy 选其他 branch）/ "create and checkout" 复选框
- [ ] A.2 name 输入框实时校验：用 `git2::Reference::is_valid_name(format!("refs/heads/{name}"))` 同步检查
  - 合法 → 默认边框 · "确认" 按钮 enabled
  - 非法（含空格 / 控制字符 / 起始 `.` / `@{` 等）→ 红色边框 + tooltip `"分支名包含非法字符（不允许空格 · 控制字符 · 起始 .）"`
- [ ] A.3 提交后调用 `branch_create` IPC：
  - 成功 → toast `"分支 {name} 已创建"`（3s）· 分支树即时刷新
  - 失败（同名分支已存在 / git2 错误）→ toast error + 错误文案（如 `"分支 {name} 已存在"`）· modal 保留输入
- [ ] A.4 "create and checkout" 勾选 → 一步完成 create + checkout · 失败时如果 create 已成功但 checkout 失败 → toast warn `"分支已创建 · 但切换失败：{reason}"` + 分支树仍刷新（保留新 branch）
- [ ] A.5 性能：单次 create（< 1000 branch fixture）< 100ms（`performance.now()` 测点击 "确认" 到 toast 显示 · 测 3 次取 P99）

### B. Branch Checkout

- [ ] B.1 Primary Sidebar 点击 branch row → 触发 checkout
- [ ] B.2 Clean working tree（`git_status.staged + unstaged + untracked` 全为空）→ 直接 checkout · toast `"已切换到 {branch}"` + Git Log + Status 面板刷新 · 全程 < 500ms（10000 commit 仓库 · 测 3 次取 P99）
- [ ] B.3 Dirty working tree → 弹对话框（§UI 引用"Dirty Tree 对话框"）：
  - `Stash & Switch` 按钮在 v0.2 **禁用** + tooltip `"v0.2 不支持自动 stash · 请在终端执行 git stash 后重试"`（决策见 §H.3）
  - `Discard & Switch` 红色 → 二次确认 modal（"将丢弃以下文件的修改 · 不可恢复"）→ 用 `git2::CheckoutBuilder::force()` 强制 checkout
  - `Cancel` 默认 → 关闭对话框 · 不切分支
- [ ] B.4 Detached HEAD → checkout 任意已存在 branch 后退出 detached（`Repository::set_head(refs/heads/{name})`）· status bar 不再显示 `(detached)`
- [ ] B.5 Remote branch checkout：双击 `origin/feat/x` → 自动 `git2::Branch::create("feat/x", commit, false)` + `set_upstream("origin/feat/x")` + checkout · toast `"已基于 origin/feat/x 创建本地分支 feat/x 并切换"`
- [ ] B.6 已是当前 branch 再点击 → no-op · 不触发 IPC · 视觉上 row 闪一下表示 acknowledged（150ms transition）

### C. Branch Delete

- [ ] C.1 Primary Sidebar branch row 右键 → context menu 含 "Delete branch"（当前 branch 该项 disabled + tooltip `"无法删除当前分支 · 请先切换"`）
- [ ] C.2 main / master / trunk 三个名字保护：右键菜单该项 disabled + tooltip `"受保护分支 · 不允许删除"`（保护名单 hardcoded · v0.3 移到设置）
- [ ] C.3 安全删除（`-d` 等价）：
  - 已合并到 HEAD → 直接删除 · toast `"已删除分支 {name}"`（3s）· 同时显示 `Undo` 按钮（5s 内点击触发 `branch_create` 用缓存的 head SHA 重建 · 不带 upstream）
  - 未合并 → toast error `"分支 {name} 含未合并 commit · 强制删除？"` + 红色 `Force Delete` 按钮（点击进 §C.4 流程）
- [ ] C.4 强制删除（`-D` 等价）：
  - 弹 §UI 引用"Force Delete 对话框"· 列出 N 个未合并 commit 的 short SHA + message 第一行（最多前 3 个）
  - 点击 `Force delete (data loss)` → 调用 `branch_delete` IPC · `force: true` · 缓存被删 ref 的 head SHA · toast `"已强制删除 {name}"` + 5s undo
  - 点击 `Cancel` 默认 → 关闭对话框 · 分支保留
- [ ] C.5 Undo 5s window：toast 内 `Undo` 按钮 5s 后自动消失 · 期间点击触发恢复 · 恢复成功 toast `"已恢复分支 {name}"`（**不带原 upstream** · 用户需手动 set_upstream）
- [ ] C.6 性能：单次 delete < 50ms（`performance.now()` 测右键菜单点击到 toast · 测 3 次取 P99）

### D. Fuzzy Switcher

- [ ] D.1 `⌘B`（mac） / `Ctrl+B`（linux）触发 → modal 弹出 · 自动聚焦 input
- [ ] D.2 modal 关闭路径：Esc / 点击遮罩 / Enter checkout 后自动关 / 100ms inactive blur 不关闭（避免误触）
- [ ] D.3 输入过滤：subsequence fuzzy match（如 `fpt` 匹配 `feat/pty-pool`）· match 字符高亮（`<mark>` 包裹）· 大小写不敏感
- [ ] D.4 排序规则（输入为空时）：
  1. 当前 branch 置顶 + 标记 `(current)`
  2. 最近 5 个 checkout 历史（按 `app_settings` 表 `branch_recent_{workspace_id}` 时间倒序）
  3. 其他本地 branch 字母序
  4. Remote branches 单独分组在底部 · 字母序
- [ ] D.5 排序规则（有输入时）：fuzzy match score 降序 · score 算法：连续匹配位置加权（参考 fzf 简化版 · 不引第三方 crate · 自实现 30 行内）
- [ ] D.6 ↑↓ 键导航 + Enter checkout · 选中行高亮（主色背景 10% 透明）
- [ ] D.7 100 branch 场景过滤性能：输入每个字符到结果列表更新 < 16ms（DevTools Performance 录 input keypress 到 DOM commit · 测 3 次取 P99）
- [ ] D.8 1000 branch 极端场景：< 50ms（不阻塞 UI · 必要时加 debounce 50ms · 但不能掩盖性能问题）

### E. 分支树即时刷新

- [ ] E.1 任何 CRUD 后 backend emit `git:branch-changed` event · payload `{ workspace_id, branches: BranchInfo[], head: string }`
- [ ] E.2 前端 store 收到 event 增量更新（不 re-fetch 全量）· 分支树 DOM diff 200ms 内完成
- [ ] E.3 多 workspace 隔离：切 workspace 时分支树状态 per-workspace（fuzzy switcher 历史也是 per-workspace）

### F. 错误处理 + 边界

- [ ] F.1 git repo 损坏（`Repository::open` 失败）→ 分支树显示空状态 + `"Git repo unavailable · 请检查 .git 目录"` + retry 按钮
- [ ] F.2 非 git workspace → 分支树整块 hide · 不显示 sub-head
- [ ] F.3 Worktree 锁定（其他 git 进程持锁）→ create / delete 失败 toast `"Git index 被其他进程锁定 · 请稍后重试"`（3s）
- [ ] F.4 Permission denied（`.git/` 只读）→ toast error 含具体错误码 + suggested action

## 🧪 测试策略

| 层次 | 范围 | 覆盖路径 |
|------|------|---------|
| 单元（core） | `branch_ops.rs` 所有函数（list / create / checkout / delete / set_upstream） + `BranchError` enum 全 variant + name 校验函数 | `cargo test --package vibestation-core branch_ops::` · fixture: tempfile + git2::Repository::init |
| 集成 | IPC 链路：前端 invoke → Rust → git2 → 真实 repo · 包含 dirty tree / detached HEAD / unmerged delete 三大边界 | `cargo test --features integration` |
| Criterion bench | branch_list（10 / 100 / 1000 branch fixture）· branch_create / checkout / delete 单次 P99 | `crates/core/benches/branch_bench.rs` |
| E2E（Playwright） | golden path：创建 → 切换 → 删除 → 恢复（undo）· dirty tree 对话框流程 · fuzzy switcher 全键盘流程 | `web/tests/e2e/branch.spec.ts` |
| 视觉回归 | Primary Sidebar 分支树（10 branch · 100 branch · scrollable）· Fuzzy Switcher modal · Force Delete 对话框 | Playwright screenshot diff |
| 手动 QA | macOS 中文分支名（`feat/中文-test`） · Linux 不同 fs（ext4 / btrfs）的 case-sensitivity · Windows path（v0.3+） | 手动 capture |

### C.1 · fixture 准备脚本

仿 MVP-09 §C.1 模式 · 所有 fixture 用 `tempfile::TempDir` + `git2::Repository::init()` 在测试运行时生成：

```rust
// crates/core/tests/fixtures/mvp_13_helpers.rs（新建）
use git2::{Repository, Signature};
use tempfile::TempDir;

fn create_fixture_3_branches() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    // 1. config user.name + email
    // 2. write hello.txt + initial commit (master)
    // 3. branch feat/a + checkout · modify · commit
    // 4. branch feat/b 从 master · 留 unmerged
    dir
}

fn create_fixture_10_branches() -> TempDir { /* 10 local branch · 含 main / master / trunk 保护名 */ }
fn create_fixture_100_branches() -> TempDir { /* 用于 D.7 fuzzy switcher 100 branch 性能 */ }
fn create_fixture_1000_branches() -> TempDir { /* 用于 D.8 极端性能 */ }
fn create_fixture_dirty_tree() -> TempDir { /* 1 modified + 1 staged + 1 untracked */ }
fn create_fixture_detached_head() -> TempDir { /* checkout 一个 commit SHA 进入 detached */ }
fn create_fixture_unmerged_branch() -> TempDir { /* feat/x 含 main 没有的 commit */ }
fn create_fixture_remote_tracking() -> TempDir { /* 配 origin remote · 含 origin/feat/x ref */ }
fn create_fixture_chinese_branch_name() -> TempDir { /* 含 feat/中文-test 分支 */ }
```

每个 helper 返回 `TempDir` · 测试用 `let _dir = create_fixture_3_branches();` 持有 · `dir` drop 自动清理。

### C.2 · Criterion bench 模板

新建 `crates/core/benches/branch_bench.rs`：

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_branch_list_10(c: &mut Criterion) {
    c.bench_function("branch_list_10", |b| {
        b.iter(|| {
            let _dir = create_fixture_10_branches();
            // call vibestation_core::branch_ops::list(...)
        });
    });
}

fn bench_branch_list_1000(c: &mut Criterion) { /* 1000 branch */ }
fn bench_branch_create(c: &mut Criterion) { /* 单次 create */ }
fn bench_branch_checkout_clean(c: &mut Criterion) { /* clean tree checkout */ }
fn bench_branch_delete(c: &mut Criterion) { /* 单次 delete */ }
fn bench_fuzzy_filter_100(c: &mut Criterion) { /* 100 branch 输入 1 字符 filter */ }

criterion_group!(
    benches,
    bench_branch_list_10, bench_branch_list_1000,
    bench_branch_create, bench_branch_checkout_clean, bench_branch_delete,
    bench_fuzzy_filter_100
);
criterion_main!(benches);
```

跑 `cargo bench --bench branch_bench` · P99 数字写入 PR description。

## 💾 数据模型变更

新增 1 个 `app_settings` 表 key（不新建表 · 复用 MVP-03 已建 schema）：

```rust
// app_settings 表 key 示例：
// "branch_recent_{workspace_id}" → JSON array of { name: string, checked_out_at: i64 } · 最多 5 条 · LRU
```

切换 workspace 时恢复对应历史 · 用于 Fuzzy Switcher §D.4 排序。

**禁止**：不在 rusqlite 缓存 branch list（每次实时查 git2 · branch list 在 100 branch 量级毫秒级 · 缓存收益 < 维护成本）。

**禁止**：不持久化 force delete 的 cached SHA 跨 session（5s undo 是 in-memory 状态 · session 关掉即丢 · 这是有意识的取舍 · 强制删除本来就是 "data loss" 操作 · 不要给用户长时间 undo 错觉）。

## ⚠️ 已知风险

- **R1 · git2 stash API 不稳定** · v0.2 决策：**不做自动 stash**（§H.3）· 用户必须手动 `git stash` 或选 Discard · 缓解：UI tooltip 明确说明 + v0.3 评估 git2 stash API 稳定性后补
- **R2 · 强制删除 undo 局限性** · 5s 窗口 + in-memory · session 关掉即丢 · 缓解：UI 文案明确 `"删除后**无法通过 UI 恢复**"` + 用户教育 · v0.3 评估 reflog 集成
- **R3 · 中文 / Unicode 分支名兼容性** · git2 0.20 内部用 utf-8 byte sequences · 但 Windows NTFS / macOS HFS+ 大小写折叠会导致不同表现 · v0.2 macOS + Linux only · v0.3 Windows 单独 spike
- **R4 · Remote tracking 自动建本地 branch 的命名冲突** · `origin/main` checkout 时本地已有 `main` → 走 `git checkout main` 不重建 · 但 `origin/feat/x` 本地没有 `feat/x` 时新建 · 命名冲突场景（如本地 `feat/x` ≠ `origin/feat/x` head）需在 §B.5 文案明示 · 实施时单测覆盖
- **R5 · Dirty tree force checkout 数据丢失** · `CheckoutBuilder::force()` 会无声丢弃 untracked + modified 文件 · 缓解：二次确认 modal 必须显式列出 N 个文件 + `Discard & Switch` 红色按钮 + 二级确认（不能一键 force）

## 📝 Notes

- MVP-13 是 v0.2 第一个 git 写路径扩展 · 模式（git2 backend + ts-rs binding + Tauri permission + cmd 注册）和 MVP-09 完全一致 · 实施 agent 直接复用
- **Branch protection**（v0.3+）将基于设置面板 · 用户可自定义保护名单 · v0.2 hardcode `main / master / trunk` 即可
- **Reflog 集成**（恢复已删除 branch）推到 v0.3 · 接 git2 reflog API · 单独 spec
- **Auto-stash on checkout**（v0.3 评估）：等 v0.3 评估 git2 stash API 稳定性 + UX flow（自动 stash 后用户怎么找回）

## 🔗 相关

- `CLAUDE.md` #13 Git 栈混用决策（写 git2）· #7 Diff 自建（不影响本 task · 但同栈）
- ADR-007 Git 栈混用决策
- `implementation-plan.md` §10.1 v0.2 砍到分支 · §6.2 git_branch_* IPC · §11 W13
- 上游：MVP-07（git log 视图）· MVP-09（git2 写路径基础设施 + ts-rs / permission / capability 模式）· SPIKE-04（git2 写 smoke test）· **注**：BranchInfo / BranchKind binding 由 MVP-13 PR #220 首次定义（spec §G.5 stale assumption 已修正 · 详见下）
- 下游：MVP-21 push/pull/fetch（push deleted branch 联动）· MVP-16 rebase/merge/cherry-pick（v0.3）

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（预期）

| Rust struct | 用途 | 前端 import 路径 |
|-------------|------|-----------------|
| `BranchInfo` | 单条 branch 摘要（list 项）· **MVP-13 PR #220 首次定义**（原 spec 假设复用 MVP-07 已生成 · 实测仓库无此 binding · 已修正 · 详见 §G.5）| `import type { BranchInfo } from "../bindings/BranchInfo"` |
| `BranchKind` | 枚举：`Local` / `Remote` / `Tag` · **复用** MVP-07 | `import type { BranchKind } from "../bindings/BranchKind"` |
| `BranchListRequest` | 输入侧 · `{ workspace_id }` | 新增 |
| `BranchListResponse` | 输出侧 · `{ branches: BranchInfo[], head_name: string \| null, detached: boolean }` | 新增 |
| `BranchCreateRequest` | 输入侧 · `{ workspace_id, name, from_ref, checkout: bool }` | 新增 |
| `BranchCheckoutRequest` | 输入侧 · `{ workspace_id, name, force: bool }` · `force` 触发 `CheckoutBuilder::force()` | 新增 |
| `BranchDeleteRequest` | 输入侧 · `{ workspace_id, name, force: bool }` · `force` 触发 `-D` 等价 | 新增 |
| `BranchSwitchResult` | 输出侧 · `{ new_head: string, prev_head: string, dirty_files_dropped: number }` | 新增 |
| `BranchError` | 错误枚举 · 含 payload | 新增 |
| `SwitcherQueryRequest` | 输入侧 · `{ workspace_id, query: string, limit: number }` | 新增 |
| `SwitcherSearchResult` | 输出侧 · `{ matches: SwitcherMatch[] }` | 新增 |
| `SwitcherMatch` | 单条匹配 · `{ branch: BranchInfo, score: number, match_indices: number[] }` | 新增 |

> 实际 struct 名和字段以实施 PR 为准 · 但**必须**全部走 ts-rs 自动生成。

### G.2 derive 模板（以 `BranchCreateRequest` + `BranchError` 为例）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchCreateRequest {
    pub workspace_id: String,
    pub name: String,
    pub from_ref: Option<String>,  // None = HEAD · Some("origin/feat/x") = remote tracking
    pub checkout: bool,             // create and checkout
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BranchCheckoutRequest {
    pub workspace_id: String,
    pub name: String,
    pub force: bool,  // force checkout · drops uncommitted changes
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BranchError {
    InvalidName { reason: String },                // 含非法字符或保留前缀
    NotFound { name: String },                     // branch 不存在
    AlreadyExists { name: String },                // create 时同名已存在
    Unmerged { name: String, missing_commits: u32 }, // delete 未合并
    ProtectedBranch { name: String },              // main / master / trunk 保护
    DetachedHead,                                   // 当前 detached 不允许部分操作
    DirtyWorkingTree { modified: Vec<String>, staged: Vec<String>, untracked: Vec<String> },
    IndexLocked,                                    // .git/index.lock 存在
    Git2Error { class: String, code: i32, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SwitcherMatch {
    pub branch: BranchInfo,
    #[ts(type = "number")]
    pub score: f32,
    pub match_indices: Vec<usize>,  // input char index → branch.name char index 高亮用
}
```

> `BranchError` 因含 payload（reason / missing_commits / dirty 文件列表）必须用 tagged union（`#[serde(tag = "kind")]`）· 前端 TS 生成 discriminated union。

### G.3 强制规范

- [ ] 所有 IPC struct + enum 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] 简单无 payload enum 用 string union（`rename_all` + 无 tag）· 含 payload enum 用 tagged union（`#[serde(tag = "kind")]`）
- [ ] `f32` 类型字段（如 `SwitcherMatch.score`）必须加 `#[ts(type = "number")]`（防止 TS 生成 `bigint`）
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface BranchCreateRequest { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-04 §G.3 / MVP-09 §G.4 模式 · 流程：

1. 临时在任一 IPC struct（如 `BranchCreateRequest`）的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'BranchCreateRequest'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]` · 确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次 · 结果写入 PR description 或 `docs/runtime-evidence/MVP-13/`。

### G.5 · 与上游已落地 binding 的复用决策

> **⚠️ 修正说明（2026-05-03 · PR #220 实施事实）**：原 spec 此表假设 BranchInfo / BranchKind 已由 MVP-07 生成 ts-rs binding · **实测仓库无此 binding**（grep 验证 · `git log` 历史 · MVP-07 PR 仅生成 `GitLogEntry` / `GitLogQueryRequest/Response` 等 git log 相关 binding）。MVP-13 PR #220 已最小补齐定义 BranchInfo + BranchKind · 实际新增 binding 数 12（见 §G.6）· 不是原 spec 写的 9。下表已修正复用决策 · 反映 PR #220 实施事实。未来 MVP-07 spec 评估是否迁移这两个 binding 的 source of truth（low priority · 当前 MVP-13 + MVP-07 共享 binding 定义无冲突）。

MVP-13 实施前必须明确复用 / 新增边界：

| 已有 binding | MVP-13 §G.1 涉及 | 决策 | 理由 |
|---|---|---|---|
| `BranchInfo`（**MVP-13 PR #220 首次定义** · 含 `name / full_ref / kind / upstream / ahead / behind / head_commit`）| §G.1 `BranchInfo` 字段对齐 | ✅ **MVP-13 自定义** · ts-rs `#[ts(export)]` 生成 `BranchInfo.ts` | 仓库无 MVP-07 BranchInfo · 最小补齐 · 字段定义完全对齐 v0.2 需求 |
| `BranchKind`（**MVP-13 PR #220 首次定义**）| `BranchInfo.kind` | ✅ **MVP-13 自定义** · 枚举 `Local / Remote / Tag` | 同上 · 与 BranchInfo 配套定义 |
| `CommitAuthor`（MVP-07 已生成）| 不涉及 | ⛔ 不复用 · MVP-13 不涉及 commit 元数据 | branch_create 不带 author 信息 · 用 git2 自带 default signature |
| `FileChange`（MVP-07 / MVP-08 已生成）| `BranchError::DirtyWorkingTree` 的 modified / staged / untracked | ⛔ 不复用 · 用 `Vec<String>` 即可 | DirtyWorkingTree 只需要文件路径 · 不需要 additions/deletions/status · 复用 `FileChange` 是过度引入 |
| `GitStatusResponse`（MVP-08 已生成）| Dirty tree 检测可能消费此 binding | ✅ 前端**复用**（不新建） | MVP-13 前端检测 dirty 时调 MVP-08 已有 `git_status` IPC · 不重复实现状态查询 |
| `CommitError`（MVP-09 已生成）| `BranchError` enum | ⛔ 不复用 · 新建独立 `BranchError` | 错误语义不同（`HookFailed` vs `Unmerged` / `ProtectedBranch`）· 强行复用会让 union 膨胀 |

### G.6 · MVP-13 新增 binding 清单（明确数量）

> **修正（PR #220 实施事实）**：原 spec 写 9 个 · 实际 **12 个**（10 Branch* + Switcher* core binding + 2 BranchInfo / BranchKind 最小补齐）。

以下 **12 个 binding** 为 MVP-13 **新增** · 实施时 `web/src/bindings/` 已新增 12 个 `.ts` 文件（PR #220 落地）：

| Rust struct / enum | 用途 | 前端 import 路径 |
|---|---|---|
| `BranchListRequest` | list 输入 · `{ workspaceId }` | `import type { BranchListRequest } from "../bindings/BranchListRequest"` |
| `BranchListResponse` | list 输出 · `{ branches: BranchInfo[], headName: string \| null, detached: boolean }` | `import type { BranchListResponse } from "../bindings/BranchListResponse"` |
| `BranchCreateRequest` | create 输入 | `import type { BranchCreateRequest } from "../bindings/BranchCreateRequest"` |
| `BranchCheckoutRequest` | checkout 输入 | `import type { BranchCheckoutRequest } from "../bindings/BranchCheckoutRequest"` |
| `BranchDeleteRequest` | delete 输入 | `import type { BranchDeleteRequest } from "../bindings/BranchDeleteRequest"` |
| `BranchSwitchResult` | checkout 输出 | `import type { BranchSwitchResult } from "../bindings/BranchSwitchResult"` |
| `BranchError` | 错误枚举 · 含 payload tagged union | `import type { BranchError } from "../bindings/BranchError"` |
| `SwitcherQueryRequest` | fuzzy switcher 查询 · `{ workspaceId, query, limit }` | `import type { SwitcherQueryRequest } from "../bindings/SwitcherQueryRequest"` |
| `SwitcherMatch` + `SwitcherSearchResult` | fuzzy match 单条 + list · 2 binding | `import type { SwitcherMatch, SwitcherSearchResult } from "../bindings/..."` |

> **修正（PR #220 实施事实）**：原 spec 假设 BranchInfo / BranchKind 复用 MVP-07 · 实测仓库无此 binding · MVP-13 PR #220 最小补齐定义。实际新增 **12 个** `.ts` 文件（10 Branch+Switcher core + 2 BranchInfo/Kind 最小补齐）· 仅前端 GitStatusResponse 走 MVP-08 IPC 复用（不在 web/src/bindings/ 新增）。

## §H. Git 栈约束 + 决策锁定（MVP-13 专有 · 防 v0.2 实施期反复讨论）

MVP-13 是**纯写路径** · 对齐 CLAUDE.md 决策表 #13（2026-04-19 accepted · [ADR-007](../adr/ADR-007-git-stack.md)）· 必须明确：

### H.1 本 MVP Git 栈

- **写路径 crate**：`git2 0.20`
- **读路径 crate**（branch list）：`gix 0.70`（复用 MVP-07 模式 · 因为 list 不涉及修改 · gix 更快）
- **场景**：
  - List：gix `Repository::references()` → `Iterator<Result<Reference>>` 过滤 `refs/heads/*` + `refs/remotes/*` + `refs/tags/*`
  - Create / Checkout / Delete：git2（gix 0.70 写 API 不成熟）
  - Set upstream：git2 `Branch::set_upstream`
- **依据**：
  - SPIKE-03 benchmark：gix list refs 远快于 git2（因 zero-copy + parallel · 见 `docs/spikes/SPIKE-03-report.md`）
  - SPIKE-04 §C 已验证 git2 0.20 写路径 smoke test 通过（branch CRUD 在 SPIKE-04 §C 边界用例已覆盖）

### H.2 不碰的 crate

- **不碰 gix 写**：gix 0.70 的 branch / reference 写 API 不完整 · 不试水
- **不碰其他 git 库**：禁止引入 gitoxide 之外第三方 git 库
- **不碰 fzf 第三方 crate**（如 `fuzzy-matcher` / `skim`）：fuzzy 算法自实现 30 行内 · 避免依赖膨胀（v0.3+ 评估引入）

### H.3 stash 策略锁定（v0.2 不做自动 stash）

**决策**：v0.2 dirty working tree → checkout 前**不自动 stash** · 提示用户三选项（`Stash & Switch` 暂禁用 / `Discard & Switch` / `Cancel`）。

**理由**（v0.2 不做的取舍 · 锁定避免 v0.3 反复讨论）：

| 选项 | 优点 | 缺点 | v0.2 评估 |
|------|------|------|-----------|
| (a) **不做自动 stash · 提示用户**（**v0.2 选定**） | UX 简单 · stash 历史用户能在终端管理 · 无 git2 stash API 不稳定风险 | 用户必须切到终端做 stash · 工作流断裂 | ✅ MVP 复杂度优先 |
| (b) 自动 stash + checkout + restore | 工作流连续 · 用户感知"无缝切换" | git2 stash API 不稳定（实测 git2 0.18-0.20 stash 边界 case 多）· UX 复杂（用户找不回 stash 历史 · UI 需要 stash list 视图 · v0.2 工期不够） | ⏸ v0.3 评估 |
| (c) 阻止 dirty tree 切换 | 实现最简单 · 无数据丢失风险 | UX 太严苛 · 用户咒骂 | ❌ 不可接受 |

**v0.3 升级触发条件**（满足任一）：
1. git2 0.21+ stash API 稳定（边界 case 修复）
2. UX team 设计完整 stash list 视图（让用户能找回自动 stash 的内容）

### H.4 git2 0.20 API 使用要点（实施参考）

| 操作 | git2 API 调用链 |
|------|----------------|
| List local | gix（首选）：`Repository::references()` 过滤 `refs/heads/*` · fallback git2：`Repository::branches(Some(BranchType::Local))` |
| List remote | gix：过滤 `refs/remotes/*` · fallback git2：`Repository::branches(Some(BranchType::Remote))` |
| Create | `Repository::find_commit(target_oid)` → `Repository::branch(name, &commit, force)` |
| Checkout | `Repository::find_branch(name, BranchType::Local)` → `Branch::get().name()` → `Repository::set_head(...)` → `Repository::checkout_head(opts)` |
| Force checkout | `CheckoutBuilder::new().force()` |
| Delete (safe · `force=false`) | **必须先做 reachability check**：`Repository::find_branch(name, t)` → 取 branch tip OID → `Repository::head()` 取当前 HEAD OID → `Repository::merge_base(tip, head)` → 比较 `merge_base == tip`？是 → 已合并 · 安全 · `Branch::delete()` · 否 → 抛 `BranchError::Unmerged { name, missing_commits }`（variant 名对齐 §G.2 enum 定义 · `missing_commits` = `tip` 到 `merge_base` 的 commit 数 · 用 `Revwalk` 计数）· 让 UI 提示用户改用 force |
| Delete (force · `force=true`) | 跳过 reachability check · 直接 `Repository::find_branch(name, t)` → `Branch::delete()`（但保护分支名单 main/master/trunk 仍硬阻拦） |
| Set upstream | `Branch::set_upstream(Some("origin/feat/x"))` |
| Name validate | `git2::Reference::is_valid_name(format!("refs/heads/{name}").as_str())` |
| Detached HEAD detect | `Repository::head_detached()` |
| 错误分诊 | `git2::Error::class()` / `git2::Error::code()` → 映射到 `BranchError` enum |

### H.5 detached HEAD 处理

- **List**：detached 时 `BranchListResponse.detached: true` + `head_name: None` · 前端显示 status bar `(detached)` 标记
- **Create**：detached 时允许新建（从 detached 的 commit 起点 · 即新 branch 的 head = 当前 commit SHA）
- **Checkout**：必须有目标 branch · checkout 后退出 detached
- **Delete**：detached 不影响 delete 其他 branch

### H.6 branch name 验证规则

**层级 1（前端实时校验 · 用户体验）**：

```typescript
// web/src/utils/branchName.ts
const FORBIDDEN_CHARS = /[\s\x00-\x1f\x7f~^:?*\[\\]/;
const FORBIDDEN_PATTERNS = [/^[.\/]/, /\.\.+/, /@\{/, /\.lock$/, /\/$/];

export function validateBranchName(name: string): { valid: boolean; reason?: string } {
  if (!name) return { valid: false, reason: "分支名不能为空" };
  if (FORBIDDEN_CHARS.test(name)) return { valid: false, reason: "含非法字符（空格 / 控制字符 / ~^:?*[\\）" };
  for (const p of FORBIDDEN_PATTERNS) if (p.test(name)) return { valid: false, reason: "非法格式（起始 . / 含 .. / @{ / 结尾 .lock 或 /）" };
  return { valid: true };
}
```

**层级 2（后端权威校验 · 安全网）**：

```rust
// crates/core/src/branch_ops.rs
fn validate_name(name: &str) -> Result<(), BranchError> {
    let full_ref = format!("refs/heads/{}", name);
    if !git2::Reference::is_valid_name(&full_ref) {
        return Err(BranchError::InvalidName {
            reason: format!("git2 拒绝 ref name '{}'", name),
        });
    }
    Ok(())
}
```

两层校验 · 前端阻止常见错误 + 后端兜底 · 避免 git2 不同版本规则差异引发不一致。

### H.7 跨平台兼容性

| 平台 | 状态 | 说明 |
|------|------|------|
| macOS（Apple Silicon / Intel） | ✅ v0.2 支持 | git2 0.20 + libssh2 / libgit2 系统包 · APFS case-insensitive 但 git2 内部按 byte 处理 · 中文 branch 名 OK |
| Linux（Ubuntu 24 X11 / Wayland） | ✅ v0.2 支持 | git2 0.20 同样可用 · ext4 case-sensitive · 测试验证 `feat/x` ≠ `feat/X` 不视为同名（与 macOS 行为差异 · UI 不做隐藏） |
| Windows | ❌ v0.3+ | NTFS case-insensitive + path separator `\` · v0.3 单独 spike + Windows-specific 路径处理 |

### H.8 与 MVP-21 push/pull/fetch 的边界

MVP-13 仅本地 branch 操作 · **不调用任何网络**。

| 场景 | MVP-13 责任 | MVP-21 责任 |
|------|-------------|-------------|
| 本地 branch CRUD | ✅ | ❌ |
| Remote branch checkout（自动建本地 tracking） | ✅（local create + set_upstream · 不 fetch）| ❌ |
| Push deleted branch to origin（`git push origin --delete x`） | ❌ | ✅ |
| Fetch remote refs（更新 `origin/*`） | ❌ | ✅ |
| Pull（merge / rebase） | ❌ | ✅ |

实施时严格隔离 · 避免 MVP-13 PR 引入网络代码导致 review 复杂度爆炸。

---

**自审四问**（2026-05-01 · vibe sprint Worker B 详化）：

1. **递归完备性**：Acceptance 清单覆盖 Branch Create / Checkout / Delete / Fuzzy Switcher / 即时刷新 / 错误处理 / 性能 / fixture / IPC contract / Git 栈约束 / 跨平台 全维度 ✅
2. **反向场景**：
   - 名字非法 → 前端 + 后端两层校验 ✅
   - dirty tree → 三选项对话框 + Stash 暂禁用 ✅
   - 未合并 delete → toast 提示 + force flow ✅
   - undo 失败（5s 超时 / git2 重建失败）→ toast error · 不假装恢复成功 ✅
   - detached HEAD → §H.5 explicit 处理 ✅
   - git repo 损坏 / index lock → §F.1-F.3 错误码 + retry 引导 ✅
3. **边界适用性**：
   - 0 branch（空 repo · 仅 HEAD）/ 10 / 100 / 1000 / 1 万 branch 都覆盖（§D.7-D.8 性能 + §C.1 fixture）
   - 中文 / Unicode 分支名（§R3 + §H.7 macOS / Linux 验证）
   - 跨平台：macOS + Linux v0.2 / Windows v0.3+ 明确推后
   - 多 workspace：§E.3 隔离明确
4. **YAGNI**：
   - 不做：rename / auto-stash / push delete / branch protection rules / cherry-pick / merge / rebase / submodule branch / Windows · 全在 §Don't 明示推后
   - 不引入：fuzzy 第三方 crate（自实现 30 行内）/ 第三方 git 库
5. **对齐上游 binding**（§G.5）：BranchInfo / BranchKind **MVP-13 PR #220 首次定义**（修正 stale assumption · 详见 §G.5 修正说明）· MVP-08 `GitStatusResponse` 前端复用 IPC · 不造平行类型 · 新增 12 个独立 binding 清单明确（§G.6）
6. **§H 决策锁定全覆盖**：H.1 Git 栈 / H.2 不碰列表 / H.3 stash 策略 / H.4 API 调用链 / H.5 detached HEAD / H.6 name 校验 / H.7 跨平台 / H.8 与 MVP-21 边界 · 防 v0.2 实施期反复讨论
7. **runtime evidence 路径已锁定**：§Phase D 明确 `docs/runtime-evidence/mvp-13/`（按 `.claude/rules/runtime-evidence-location.md` R1）

---

## 详化完成度评估（Arbiter 审 PR 时参考）

| 12 段必含 | 状态 | 备注 |
|----------|------|------|
| 1. frontmatter | ✅ | id / type / title / status:draft / depends_on / phase / estimate / plan_ref / risk_ref / reviewer 占位 |
| 2. 🎯 目标 Goal | ✅ | 一句话核心 + plan_ref link |
| 3. 📖 背景 Context | ✅ | implementation-plan + CLAUDE.md + 路线图 W13 + 上游已落地 |
| 4. 🛠 实施进度表 | ✅ | Phase A/B/C/D 拆分 + Phase A 起点 checklist |
| 5. 🎨 功能范围 Scope | ✅ | Do 7 项 / Don't 6 项 |
| 6. 🖼 UI 引用 | ✅ | design 原型 line 引用 + 4 类 UI 元素描述 |
| 7. ✅ Acceptance | ✅ | A-F 6 大组 / 30 项 checkbox · 每项含具体测法 |
| 8. 🧪 测试策略 | ✅ | 单元 / 集成 / Criterion / E2E / 视觉回归 / 手动 QA + fixture + bench 模板 |
| 9. 💾 数据模型变更 | ✅ | 1 个 app_settings key · 不新建表 + 反模式禁止 |
| 10. §G IPC Contract | ✅ | 12 struct + derive 模板 + G.5 复用 + G.6 新增 9 binding 清单 |
| 11. §H 决策锁定 | ✅ | H.1-H.8 8 子段 · 含 stash 策略表 + git2 API 表 + 跨平台矩阵 |
| 12. ⚠️ 已知风险 + Notes + 相关 + 自审四问 | ✅ | 5 风险 + 4 Notes + 6 相关 + 7 条自审 |

**完成度**：12/12 = **100%**（建议 Arbiter approve PR 后翻 status: ready）。

**遗留问题**：无 · 所有决策已锁定 · 没有"v0.2 启动后再讨论"的悬空项。
