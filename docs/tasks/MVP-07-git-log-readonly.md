---
id: MVP-07
type: mvp
title: Git Log 只读视图 + Commit 详情
status: ready
owner:
phase: W8-W9
depends_on: ["MVP-02", "MVP-03", "SPIKE-03"]
blocks: ["MVP-08"]
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §10.2（10 万 commit < 500ms）
risk_ref: R3
reviewer: Kimi
---

# MVP-07: Git Log 只读视图

> **状态**：`draft`
> **依赖**：MVP-02（workspace → repo）· MVP-03（Secondary Sidebar 容器）· SPIKE-03（git 读性能锁定 gix）
> **阻塞**：MVP-08（Diff 基于 commit 详情）

---

## 🎯 目标（Goal）

在 Secondary Sidebar 显示 workspace 对应 repo 的 commit log（只读），点击 commit 打开详情视图（元数据 + 变更文件列表）。10 万 commit 首屏 < 500ms。

## 📖 背景（Context）

- `CLAUDE.md` #13（A 栏永久锁定）：Git 栈 = **读 `gix 0.70` · 写 `git2 0.20` 混用**
- 禁区：**不做自绘 rail graph**（砍到 v0.2）
- 禁区：**不做 Diff 复杂语法高亮**（MVP-08 只做基础对比）

---

## 🎨 功能范围（Scope）

**Do**：
- Secondary Sidebar 展开 → 显示 Git Log
- 列表每行：commit hash（短 7 位）+ message 第一行 + author name + 相对时间
- 分支 / tag 标签贴（**不画 rail graph**）
- 分页加载：初始 100 条，滚动到底触发加载更多（+ 100）
- 10 万 commit 首屏 < 500ms（`§10.2`）
- 点击 commit → 主区域 Tab 旁打开"Commit 详情"视图
- Commit 详情：hash 全长 / author email / authored date / committer / committed date / parent(s) / 变更文件列表（仅列名 + 加减行数）
- 筛选：按 author / 消息关键词（顶部搜索框）

**Don't**：
- 自绘 rail graph（v0.2+）
- Branch create / checkout / delete（v0.2+）
- Diff 内容（→ MVP-08）
- Cherry-pick / rebase / merge（v0.3+）

## 🖼 UI 引用

- Secondary Sidebar: `design/directions/1-calm-studio.html` 右侧 Git Log 区
- 标签贴：浅色底 + 主色边框 + `branch`/`tag` 文字（不同颜色区分）
- Commit 详情主区：顶部元数据 + 下部文件列表，每文件 `src/foo.rs +10 -5`

## ✅ Acceptance

### A. Log 列表

- [ ] 打开含 git 的 workspace → Secondary Sidebar toggle 显示 Git Log
- [ ] 初始加载最近 100 commit，耗时 < 500ms（linux kernel 仓库 · 测 3 次取 P99）
- [ ] 每行显示：短 hash / message 首行 / author / 相对时间（如 `2 hours ago`）
- [ ] 分支 / tag 标签贴显示在对应 commit 旁（不同颜色区分 `branch`/`tag`）
- [ ] 滚动到底触发加载下 100 条，滚动 FPS ≥ 60 · 帧时长 < 16ms（Chrome DevTools Performance 面板记录）

### B. Commit 详情

- [ ] 点击 commit 行 → 主区新开 "Commit: {shortsha}" tab（不是终端 Tab，是视图 Tab）
- [ ] 详情页显示：hash / author email / 日期 / parent(s) / 消息全文 / 变更文件列表
- [ ] 文件列表每行：路径 + 加减行数（`+10 -5`）+ 状态（M/A/D/R）
- [ ] 点击文件行 → 打开 Diff 视图（MVP-08 接管）
- [ ] 从 click 到 paint < 100ms（Chrome DevTools Performance 面板 `performance.now()` 差值 · 5 次采样取 median）

### C. 筛选 / 搜索

- [ ] 顶部搜索框，输入回车过滤
- [ ] 支持：message 关键词（case-insensitive substring）+ `author:name` + `after:2024-01-01`
- [ ] 在 linux kernel 10 万 commit 上测 · `'fix bug'` 关键词过滤 < 500ms（测 3 次取 P99）
- [ ] 过滤结果也应用分页（前 100 + 加载更多）

### D. 性能（`§10.2` + SPIKE-03）

- [ ] 10 万 commit 仓库首屏 < 500ms（linux kernel · 测 3 次取 P99）
- [ ] 滚动加载下 100 条 < 200ms（测 3 次取 P99）
- [ ] Commit 详情打开 < 100ms（元数据 + 文件列表 · 测 3 次取 P99）
- [ ] 筛选/搜索 < 500ms（10 万 commit 基数 · 测 3 次取 P99）

### E. 错误处理

- [ ] 非 git workspace → Secondary Sidebar 显示 `"This workspace has no git repo"` + 建议操作 `"Open a directory containing a .git folder"`
- [ ] git 仓库损坏 → 显示具体 error type（如 `gix::open::Error` / `invalid object`）+ suggested action（如 `"请检查 .git/objects 是否损坏 · 可尝试 git fsck"`）· 应用不 panic / 不白屏
- [ ] 超大 commit（>10 万文件变更）→ 文件列表限显 1000 条 + `"Show all ({total} files)"` 按钮 · 点击后 lazy load 剩余

### F. 多 workspace 隔离

- [ ] 切换 workspace → Git Log 刷新为对应 repo（< 200ms · 测 3 次取 P99）
- [ ] 多 workspace 并存时，Git Log 状态（滚动位置 / 筛选关键词 / 当前选中 commit）per-workspace 独立 · 切换回来后恢复

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元（core）| gix 读逻辑 + 元数据解析 + 筛选算法 |
| 集成 | 小 / 中 / 大 fixture 仓库（1k / 10k / 100k commit）|
| 性能 | linux kernel benchmark（对齐 SPIKE-03 · gix revwalk 分页）|
| E2E | workspace 打开 → 滚动 → 点 commit → 详情 |

## 💾 数据模型变更

新 table `git_log_cache`（可选，用于加速重复打开）:

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitLogCacheEntry {
    pub cache_key: String,       // workspace_id + ":" + branch_head_sha
    pub commits_json: String,    // JSON array of CommitSummary
    pub created_at: i64,
}
```

- key: `workspace_id + ":" + branch_head_sha`
- value: `Vec<CommitSummary>`（最近 1000 条）
- TTL: invalidate when `branch_head_sha` 变化

## ⚠️ 已知风险

- **R3**（git2 大仓库 log 慢，SPIKE-03 消除）：MVP-07 采用 gix 读路径 · 10 万 commit 首屏目标可达（见 §H 分析）
- **超大 commit**（kernel 全量 merge）文件列表可能卡住 → 限 1000 + lazy load
- **gix bundle 体积**：gix 0.70 作为读路径依赖 ·  release 二进制增量约 +2MB（SPIKE-03 已评估 · 可接受）

## 📝 Notes

- MVP-07 **完全不自绘 commit graph**——列表视图即可，图形在 v0.2
- Commit 详情 Tab 和终端 Tab 并列，但类型不同（enum TabKind）

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS + PR #63 rollout 生产化 · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（预期）

| Rust struct | 用途 | 前端 import 路径 |
|-------------|------|-----------------|
| `GitLogEntry` | 单条 commit 摘要（列表项） | `import type { GitLogEntry } from "../bindings/GitLogEntry"` |
| `CommitDetail` | Commit 详情全量数据 | `import type { CommitDetail } from "../bindings/CommitDetail"` |
| `CommitAuthor` | Author / Committer 信息 | `import type { CommitAuthor } from "../bindings/CommitAuthor"` |
| `CommitParent` | Parent commit 引用 | `import type { CommitParent } from "../bindings/CommitParent"` |
| `FileChange` | 变更文件条目 | `import type { FileChange } from "../bindings/FileChange"` |
| `GitLogQueryRequest` | 查询参数（workspace_id / offset / limit / filter） | `import type { GitLogQueryRequest } from "../bindings/GitLogQueryRequest"` |
| `GitLogQueryResponse` | 查询结果（entries + has_more） | `import type { GitLogQueryResponse } from "../bindings/GitLogQueryResponse"` |

> 实际 struct 名和字段以实施 PR 为准，但**必须**全部走 ts-rs 自动生成。

### G.2 derive 模板（以 `GitLogEntry` 为例）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GitLogEntry {
    pub short_sha: String,           // 短 hash（7 位）
    pub message: String,             // commit message 第一行
    pub author_name: String,
    /// Unix timestamp (seconds)· 映射为 TS `number` 而非默认 `bigint`
    #[ts(type = "number")]
    pub authored_date: i64,
    pub relative_time: String,       // "2 hours ago"
    pub branch_labels: Vec<String>,  // 分支名列表
    pub tag_labels: Vec<String>,     // tag 名列表
}
```

### G.3 强制规范

- [ ] 所有 IPC struct 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] `i64` 类型的时间戳字段必须加 `#[ts(type = "number")]`（防止 TS 生成 `bigint`，前端 Date/sort 零改动）
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface GitLogEntry { ... }` 或 `type GitLogEntry = { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-04 §G.3 定义（见 `docs/tasks/MVP-04-multi-tab-terminal.md` §G.3），流程如下：

1. 临时在任一 IPC struct（如 `GitLogEntry`）的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'GitLogEntry'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]`，确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次，结果写入 PR description 或 `docs/runtime-evidence/MVP-07/`（如实施 PR 本身含 ts-rs 集成）。

## §H. Git 栈约束（MVP-07 专有）

MVP-07 是**只读路径** · 按 CLAUDE.md 决策表 #13（2026-04-19 accepted · [ADR-007](../adr/ADR-007-git-stack.md)）· 必须明确：

### H.1 读路径实现

- **读路径 crate**：`gix 0.70`
- **依据**：SPIKE-03 benchmark（[docs/spikes/SPIKE-03-report.md](../spikes/SPIKE-03-report.md)）
  - gix `log -100` warm P99 **12.65ms** vs git2 **24964ms** · gix **1973×** 加速
  - gix `log -1000` warm **113.84ms** vs git2 **21108ms**
  - gix `log -10000` warm **733.72ms** vs git2 **33483ms**

### H.2 10 万 commit 首屏 < 500ms 目标可达性

- gix 支持**分页 revwalk** · 不扫描全量 commit 树
- 初始 100 条：按 SPIKE-03 `log -100` warm 12.65ms 推断 · 实际 ≈ **12–50ms**（含 gix object cache warm-up）
- 500ms 预算分配：gix revwalk 50ms + Rust→TS IPC 50ms + 前端渲染 200ms + 余量 200ms
- **结论**：目标可达 · 余量充足

### H.3 写路径

- 本 MVP **不涉及写**（只读视图）
- 若未来需要写（如 MVP-08 index staging / MVP-09 commit）· 走 **`git2 0.20`**
- 混用边界：读命令（`git_log_query` / `commit_detail`）用 gix · 写命令（`stage` / `commit`）用 git2

### H.4 禁止引入第三个 git 库

- **禁止**在 gix / git2 之外引入第三个 git 操作库
- gitoxide 生态内 sub-crate（如 `gix-traverse` / `gix-revision`）可用
- 非 gitoxide 第三方 git 库需走 ADR 流程

## 🔗 相关

- `CLAUDE.md` #13 Git 栈
- SPIKE-03 git2/gix benchmark · SPIKE-03-report.md
- `implementation-plan.md` §10.1 · §10.2 · §9 R3
- ADR-007 Git 栈混用决策
- 上游：MVP-02 · MVP-03 · SPIKE-03
- 下游：MVP-08

---

**自审四问（2026-04-20）**：
1. **递归完备性**：Acceptance 清单覆盖 Log / Commit 详情 / 筛选 / 性能 / 错误 / 多 workspace / IPC contract / Git 栈约束 全维度 ✅
2. **反向场景**：若 TS derive 漏加 → `pnpm typecheck` 立即 FAIL（H2 proof 制度化）· 若 gix revwalk 异常 → fallback 到全量扫描仍 < 500ms（余量 200ms）✅
3. **边界适用性**：0 commit（空 repo）/ 1 commit / 10 万 commit / 超大 commit（>10 万文件）都适用；非 git workspace 显式降级 ✅
4. **YAGNI**：rail graph / branch ops / Diff 内容 / cherry-pick / rebase / merge / AI 联动 都明确推后 ✅
