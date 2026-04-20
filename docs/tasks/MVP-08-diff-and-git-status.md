---
id: MVP-08
type: mvp
title: Diff 基础视图（自绘）+ Git Status 只读面板
status: draft
owner:
phase: W9-W10
depends_on: ["MVP-07"]
blocks: ["MVP-09"]
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §3.1（Diff 自建）
risk_ref:
reviewer:
---

# MVP-08: Diff 基础视图 + Git Status 只读

> **状态**：`draft`
> **依赖**：MVP-07（commit 详情触发 Diff）
> **阻塞**：MVP-09（Stage/Unstage 基于 Status 面板）

---

## 🎯 目标（Goal）

实现**自绘** Diff 视图（基础行对比，**不用 Monaco**），+ Bottom Panel 的 Git Status 只读面板（staged / unstaged / untracked 分组）。

## 📖 背景（Context）

- `CLAUDE.md` #7（A 栏永久锁定）：Diff 渲染 = 自建（`similar` crate + HTML/Canvas），**不用 Monaco**（Monaco 3MB 会爆 bundle size 预算）
- `CLAUDE.md` #13（A 栏永久锁定）：Git 栈 = **读 `gix 0.70` · 写 `git2 0.20` 混用**
- MVP 不做复杂语法高亮（v0.3+），只做 added/removed/modified 3 色行对比
- Status 面板基于 git2 `statuses()` API

---

## 🎨 功能范围（Scope）

**Do**：
- Diff 视图打开触发源：
  - MVP-07 Commit 详情点文件 → 该 commit vs parent 的 diff
  - MVP-08 Status 面板点文件 → working tree vs index（unstaged）或 index vs HEAD（staged）
- Diff 视图渲染：
  - 左右 split（old / new）或 unified（单列标 +/-）
  - 用户可切换 split / unified
  - 行号 + 颜色（绿 added / 红 removed / 灰 unchanged context）
  - **无语法高亮**（纯行对比）
  - 二进制文件 → 显示 "Binary file, X bytes changed"
  - 大文件（>1MB）→ 提示 + 按钮 "Show anyway"
- Git Status Bottom Panel：
  - 分组：staged / unstaged / untracked
  - 每行：文件路径 + 状态 icon（M/A/D/R/?）+ 加减行数（staged 有，untracked 无）
  - 点击文件 → 打开对应 Diff 视图
  - 刷新按钮 + 自动监听（fs watch 或 polling 2s）

**Don't**：
- 语法高亮（v0.3+）
- Diff 编辑功能（Stage hunk / Stash）（v0.2/v0.3+）
- 3-way merge 视图（v0.3+）
- Rename 检测高级 UI（基础检测可做，复杂 UI v0.3+）

## 🖼 UI 引用

- Bottom Panel Status：`design/directions/1-calm-studio.html` 底部面板区
- Diff 视图：主区 Tab（类型 `diff`），split 视图左右 50/50，unified 视图单列

## ✅ Acceptance

### A. Diff 渲染

- [ ] Diff 算法用 `similar` crate（Myers / Patience）· **不**走 git2/gix diff（避免 IPC 层多余序列化）
- [ ] 渲染方案：**HTML 优先** · 1k 行 diff render < 50ms（Chrome DevTools Performance 面板 `performance.now()` 差值 · 5 次采样取 median）· 不达标才 fallback Canvas
- [ ] 添加行：绿底黑字（`--color-diff-added-bg` token）· 删除行：红底黑字（`--color-diff-removed-bg`）· 未变上下文：默认色
- [ ] 左右 split 和 unified 切换 toggle · 状态持久化到 rusqlite（per-workspace）
- [ ] 行号列显示原文 / 新文行号，对齐（font-variant-numeric: tabular-nums）
- [ ] 大文件（>10k 行）可流畅滚动 · 帧时长 < 16ms（Chrome DevTools Performance 面板记录 · 测 3 次取 P99）

### B. Diff 来源

- [ ] Commit 详情点文件 → gix 读取 blob + parent blob → `similar` 计算 diff（ reviewer 对照 Rust 侧 `gix::object::Blob` → `similar::TextDiff` 调用链）
- [ ] Unstaged 点文件 → git2 `diff_index_to_workdir()` → `similar` 计算 diff（`git2::Diff` 原生输出不够细 → 转交 `similar` 做行级 diff）
- [ ] Staged 点文件 → git2 `diff_tree_to_index()` → `similar` 计算 diff

### C. Git Status 面板

- [ ] Bottom Panel toggle 显示 Git Status
- [ ] 3 分组标题：Staged (N) / Unstaged (N) / Untracked (N)
- [ ] 每组可折叠 · 折叠状态持久化到 rusqlite（per-workspace）
- [ ] 每行：icon + 文件路径（相对 repo root）+ 加减行数（staged/unstaged 有）
  - icon 采用 VS Code 语义：M=Modified（🟡 `#F5A623`）/ A=Added（🟢 `#3FB950`）/ D=Deleted（🔴 `#F85149`）/ R=Renamed（⚪ 灰）/ ?=Untracked（⚪ 灰）
- [ ] 点击文件 → 主区新开 Diff Tab

### D. 刷新

- [ ] 刷新按钮手动触发
- [ ] fs watch 或 polling 2s（三平台差异性处理：Linux inotify / macOS FSEvents）
- [ ] 刷新期间不阻塞 UI（refresh 在独立 Rust task，结果通过 IPC event 推前端）
- [ ] fs watch 延迟 < 500ms（测样本：在 repo 内 `touch` 1 个文件 → Bottom Panel 刷新到显示新状态的时延 · 测 3 次取 P99）

### E. 边界 / 错误

- [ ] 二进制文件：Diff 视图显示 `"Binary file, X bytes changed"` · 不尝试文本 diff
- [ ] 大文件 > 1MB：显示 `"Large file ({size}), click to load"` · 点击后全量加载
- [ ] 文件超 10 万行：禁止加载 + 提示 `"File too large ({n} lines). Use CLI: git diff <file>"` · 应用不崩溃
- [ ] git repo 破损 → Status 面板显示 error 状态：`"Git repository corrupted: <具体原因>"`（如 `.git/objects 损坏` / `index 冲突`）+ suggested action（如 `"请尝试 git fsck --full"`）· 应用不 panic / 不白屏

### F. 性能

- [ ] Status 面板列出 1000 文件 < 200ms（测 3 次取 P99 · Criterion bench 用 fixture repo）
- [ ] Diff 打开 1k 行文件 < 200ms（测 3 次取 P99 · `performance.now()` 前端 timing）
- [ ] Diff 打开 10k 行文件 < 1s（测 3 次取 P99 · Criterion bench 大文件 fixture）
- [ ] fs watch 延迟 < 500ms（测 3 次取 P99 · 见 D 段测法）

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | `similar` diff 算法 + 文件类型判定（binary / text）+ `DiffLine` 解析 |
| 集成 | git2 `statuses()` API + gix blob 读取 + fs watch（`notify` crate）|
| E2E | 改文件 → Status 刷新 → 点开 Diff → split/unified 切换 |
| 性能 | 大文件 fixture（1k / 10k / 100k 行）· Criterion bench |
| 视觉回归 | Diff 三色样式 split / unified · Playwright screenshot 对比 |

## 💾 数据模型变更

无新 table。Diff 结果不缓存（每次实时计算）。

Status 面板折叠态 + Diff split/unified 偏好持久化到现有 `app_settings` 表（MVP-03 已建）：

```rust
// key 示例："diff_view_mode:{workspace_id}" → "split" | "unified"
// key 示例："status_panel_collapsed:{workspace_id}:staged" → "true" | "false"
```

## ⚠️ 已知风险

- **大文件性能**：HTML 渲染优先 · 若 10k 行帧时长 > 16ms → fallback Canvas（记录决策到 ADR）
- **Rename 检测**：git2 `statuses()` 支持 rename detection 但结果可能误判 · UI 保守显示 `old_name → new_name` · 不保证 100% 准确
- **fs watch 跨平台**：`notify` crate 抽象三平台 · macOS FSEvents 有 2s 延迟下限 · Linux inotify 实时 · 差异在 E 段可接受范围
- **gix/git2 混用 bundle 体积**：git2 + gix + similar = 推算 +5-7MB（已在 MVP-07 §H 预算内）· 若 release 超限 → fallback single lib（rename detection 质量可能下降）

## 📝 Notes

- MVP-08 用 `similar` crate（Rust）计算 diff · 前端只做渲染
- 选 HTML 渲染优先（开发快 + a11y 好）· 若性能不足再切 Canvas（记录到 ADR）
- Diff 计算不经过 git2/gix 原生 diff 输出（序列化开销大）· 直接读 blob content → `similar::TextDiff` → 行级结果

## §G. IPC Contract（ts-rs）

> 依据：PR #63 ts-rs rollout 确立的 IPC contract 规范（见 `docs/runtime-evidence/chore-ts-rs-rollout/h2-regression-proof.md`）。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（预期）

| Rust struct | 用途 | 前端 import 路径 |
|-------------|------|-----------------|
| `DiffRequest` | 触发 diff 计算 · 含 commit hash / file path / diff type | `import type { DiffRequest } from "../bindings/DiffRequest"` |
| `DiffResponse` | diff 计算结果 · 含 hunks | `import type { DiffResponse } from "../bindings/DiffResponse"` |
| `DiffHunk` | 单个 hunk · 含 old_start / new_start / lines | `import type { DiffHunk } from "../bindings/DiffHunk"` |
| `DiffLine` | 每行 · 含 line_type / content / line numbers | `import type { DiffLine } from "../bindings/DiffLine"` |
| `DiffLineType` | enum：Added / Removed / Context | `import type { DiffLineType } from "../bindings/DiffLineType"` |
| `GitStatusRequest` | 查 workspace 状态 | `import type { GitStatusRequest } from "../bindings/GitStatusRequest"` |
| `GitStatusResponse` | 含 staged / unstaged / untracked 3 组 | `import type { GitStatusResponse } from "../bindings/GitStatusResponse"` |
| `FileStatus` | 每文件 · 含 path / status / 加减行数 | `import type { FileStatus } from "../bindings/FileStatus"` |

> 实际 struct 名和字段以实施 PR 为准，但**必须**全部走 ts-rs 自动生成。

### G.2 derive 模板（以 `DiffLine` + `DiffLineType` 为例）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineType {
    Added,
    Removed,
    Context,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub line_type: DiffLineType,      // Added / Removed / Context
    pub content: String,              // 行内容（不含换行符）
    pub old_line_num: Option<u32>,    // 原文行号 · None 表示新增行
    pub new_line_num: Option<u32>,    // 新文行号 · None 表示删除行
}
```

### G.3 强制规范

- [ ] 所有 IPC struct + enum 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] enum 变体走 ts-rs 生成 TS string union（如 `type DiffLineType = "added" | "removed" | "context"`）· 不要 tagged union / discriminated union
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface DiffLine { ... }` 或 `type DiffLine = { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-04 §G.3 定义（见 `docs/tasks/MVP-04-multi-tab-terminal.md` §G.3），流程如下：

1. 临时在任一 IPC struct（如 `DiffLine`）的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'DiffLine'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]`，确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次，结果写入 PR description 或 `docs/runtime-evidence/MVP-08/`（如实施 PR 本身含 ts-rs 集成）。

## §H. Git 栈约束（MVP-08 专有 · 读 + 写 + 算法三分工）

MVP-08 涉及**读 + 状态查询 + diff 算法**混合路径 · 按 CLAUDE.md 决策表 #13（2026-04-19 accepted · [ADR-007](../adr/ADR-007-git-stack.md)）· 必须明确三分工：

### H.1 读路径（commit blob / tree 读取）

- **读路径 crate**：`gix 0.70`
- **场景**：Commit 详情 diff → 读取 commit blob + parent blob content
- **依据**：SPIKE-03 benchmark · gix 读性能 1973× 于 git2（12.65ms vs 24964ms）

### H.2 写路径 / 状态查询（index / working tree）

- **状态查询 crate**：`git2 0.20`
- **场景**：
  - `statuses()` → staged / unstaged / untracked 分组
  - `diff_index_to_workdir()` → unstaged diff 的原始 blob
  - `diff_tree_to_index()` → staged diff 的原始 blob
- **依据**：gix 0.70 的 `status` / `index` API 尚不成熟 · git2 `statuses()` 生产验证充分

### H.3 Diff 计算算法（独立层）

- **Diff 算法 crate**：`similar`（推荐）或 `diff` crate
- **场景**：行级 diff 计算（Myers / Patience）
- **依据**：
  - `similar` 是 Rust 生态标准 diff 库（Myers + Patience + LCS）· 纯算法 · 零 git 依赖
  - 不走 git2/gix 原生 diff（输出格式粗 · 序列化开销大）· 直接读 blob content → `similar::TextDiff` → 行级结果
- **替代**：若 `similar` 依赖过重 → fallback `diff` crate（更简单 · 只有 Myers）

### H.4 Bundle size 估算

- git2 + gix + similar 合计 ≈ **+5-7MB**（已在 MVP-07 §H 预算内）
- 若 release bundle 超过目标（如 > 30MB）· 优先级：
  1. 保留 git2（状态查询刚需）
  2. 保留 gix（读性能刚需）
  3. 替换 similar → `diff` crate（体积更小 · 算法降级为 Myers only）

### H.5 禁止

- **禁止**引入 Monaco（CLAUDE.md #7 禁区 · 3MB bundle 超限）
- **禁止**在 gix / git2 / similar 之外引入第四个 git 操作库
- gitoxide 生态内 sub-crate（如 `gix-object` / `gix-traverse`）可用

## 🔗 相关

- `CLAUDE.md` #7 Diff 自建 · #13 Git 栈
- ADR-007 Git 栈混用决策
- 上游：MVP-07
- 下游：MVP-09
- `implementation-plan.md` §10.1 · §3.1

---

**自审四问（2026-04-20）**：
1. **递归完备性**：Acceptance 清单覆盖 Diff 渲染 / 来源 / Status 面板 / 刷新 / 边界 / 性能 / IPC contract / Git 栈约束 全维度 ✅
2. **反向场景**：若 TS derive 漏加 → `pnpm typecheck` 立即 FAIL（H2 proof 制度化）· 若 HTML 渲染帧时长 > 16ms → fallback Canvas（Acceptance A 硬要求）· 若 git2 `statuses()` 失败 → 显示 error toast（不 panic）✅
3. **边界适用性**：0 文件 / 1 文件 / 1000 文件 / 大文件（>1MB / >10万行）/ 二进制 / 破损 repo 全适用；split / unified 双模式；三平台 fs watch 差异化 ✅
4. **YAGNI**：语法高亮 / Stage hunk / Stash / 3-way merge / Rename 高级 UI / AI 联动 都明确推后 ✅ · 新增关注点：ts-rs contract（§G）· gix/git2/similar 三分工（§H）已显式文档化
