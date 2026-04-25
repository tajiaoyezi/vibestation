---
id: MVP-08
type: mvp
title: Diff 基础视图（自绘）+ Git Status 只读面板
status: ready
owner: Codex CLI
phase: W9-W10
depends_on: ["MVP-07"]
blocks: ["MVP-09"]
blocked_by: []
blocked_note:
estimate: 5d
plan_ref: implementation-plan.md §10.1 · §3.1（Diff 自建）
risk_ref:
reviewer: Kimi
---

# MVP-08: Diff 基础视图 + Git Status 只读

> **状态**：`ready`
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

## 🛠 实施进度

MVP-08 估时 5d，拆 5 Phase 串行实施：

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · diff 算法 + IPC 后端 | `similar` crate 接入 + git2 `statuses()` + gix blob 读取 + 6 个 IPC commands（`diff_compute` / `git_status_query` / `git_status_subscribe` / `git_status_unsubscribe` / `git_status_refresh` / `diff_get_settings`）+ 8 个 ts-rs binding 生成 + 单元测试（diff 算法 + binary 检测 + 大文件 fallback）| ✅ done | [#100](https://github.com/tajiaoyezi/vibestation/pull/100) |
| Phase B · Status 面板前端 | SolidJS 组件 `web/src/panels/GitStatus/` + 3 分组折叠 + 文件 icon + 加减行数 + 持久化（rusqlite）| ✅ done | [#101](https://github.com/tajiaoyezi/vibestation/pull/101) |
| Phase C · Diff 视图前端 | SolidJS 组件 `web/src/panels/Diff/` + split/unified 切换 + 行号 + 大文件 lazy load + binary 提示 + 帧时长 < 16ms 验证 + Git Status/Git Log → Diff 接通 + view mode 持久化（rusqlite） | ✅ done | [#105](https://github.com/tajiaoyezi/vibestation/pull/105) |
| Phase D · fs watch 自动刷新 | `notify` 6.x crate 集成 + 三平台测试（macOS FSEvents · Linux inotify · Windows ReadDirectoryChangesW skip）+ 200ms debounce + IPC event 推送前端 + `.git/index.lock` 排除 + 4 测试覆盖（2 单元 + 2 集成）| ✅ done | 本 PR |
| Phase E · runtime 证据 + 性能量化 | Criterion bench 2 个（git_status_bench + diff_bench）+ F.1 17ms ✅ + F.2 55µs ✅ + F.4 1.07ms ✅ + F.5 39.2ms ✅ + E.3 100k 行硬 stop ✅ + 截图 4/5（第 5 张 fs watch 实时刷新待 Phase D done 后补）+ metrics-phase-e.md · 放 `docs/runtime-evidence/mvp-08/phase-e/` | 🟡 4/5 done（第 5 张截图 + A.6/F.3 DevTools 测量待 follow-up）| [#109](https://github.com/tajiaoyezi/vibestation/pull/109) |

**下次 agent 起点**：Phase E 收尾 · Phase D 已落地（`notify` 6.1.1 + 200ms debounce + IPC event 推送 · 后续不要恢复 polling）· 继续补 ≥ 5 张截图（含 fs watch 实时刷新录屏 · 现 Phase D done 可拍）+ A.2 端到端 < 200ms / A.6 帧时长 < 16ms / F.3 1k 文件前端渲染 < 70ms 等 DevTools 测量 · 放 `docs/runtime-evidence/mvp-08/phase-e/`。Phase D runtime 证据（`docs/runtime-evidence/mvp-08/phase-d/` · 7 张 `current-screen*.png` 自动命名）建议 follow-up 重命名为 ADR-011 R3 语义化命名。

**依赖关系说明**：
- MVP-08 整体依赖 MVP-07 done（已满足 · PR #83）
- MVP-08 Phase A-D 内部串行（Phase B/C 共用 Phase A 的 IPC + binding · Phase D 增强 · Phase E 收尾）
- MVP-08 和 MVP-04 Phase E/F · MVP-05/06/09/10 文件域**完全隔离** · 可并行（MVP-08 只动 `crates/core/src/{diff,git_status}.rs` + `crates/app/src/lib.rs` 注册 + `web/src/panels/{GitStatus,Diff}/`）

## 🖼 UI 引用

- Bottom Panel Status：`design/directions/1-calm-studio.html` 底部面板区
- Diff 视图：主区 Tab（类型 `diff`），split 视图左右 50/50，unified 视图单列

## ✅ Acceptance

### A. Diff 渲染

- [ ] Diff 算法用 `similar` crate（Myers / Patience）· **不**走 git2/gix diff（避免 IPC 层多余序列化）
- [ ] 渲染方案：**HTML 优先** · 1k 行 diff **纯渲染** < 50ms（Chrome DevTools Performance 面板 `performance.now()` 差值 · 测起点 = `DiffResponse` 数据已在前端 · 终点 = DOM commit 完成 · 不含 IPC roundtrip 和 Rust 侧 similar 计算 · 5 次采样取 median）· 不达标才 fallback Canvas
- [ ] **新增**：1k 行 diff **端到端**（用户点 Status 文件到 DOM commit）< 200ms（含 IPC + Rust similar + 渲染 · 测 3 次取 P99）· 这是用户感知门槛
- [ ] 添加行：绿底黑字（`--color-diff-added-bg` token）· 删除行：红底黑字（`--color-diff-removed-bg`）· 未变上下文：默认色
- [ ] 左右 split 和 unified 切换 toggle · 状态持久化到 rusqlite（per-workspace）
- [ ] 行号列显示原文 / 新文行号，对齐（font-variant-numeric: tabular-nums）
- [ ] 大文件（>10k 行）可流畅滚动 · 帧时长 < 16ms（Chrome DevTools Performance 面板记录 · 测 3 次取 P99）

### B. Diff 来源

- [ ] Commit 详情点文件 → gix 读取 blob + parent blob → `similar` 计算 diff（reviewer 对照 Rust 侧 `gix::object::Blob` → `similar::TextDiff` 调用链）
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

- [ ] Status 面板列出 1000 文件 · 后端 git2 `statuses()` < 100ms（Criterion bench 用 1k 文件 fixture repo · 测 3 次取 P99）
- [ ] Status 面板列出 1000 文件 · IPC 序列化 + 反序列化 < 30ms（含 `Vec<FileChange>` JSON encode/decode · Tauri IPC bench）
- [ ] Status 面板列出 1000 文件 · 前端列表渲染 < 70ms（virtualized list · Chrome DevTools 测 · 总和 < 200ms 端到端）
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
- **fs watch macOS FSEvents 延迟**：macOS FSEvents 系统 API 有 2s 延迟下限 · 实测可能无法达到 §D < 500ms 目标 → 备选：(a) macOS 目标放宽到 < 2s（见 §H.6）或 (b) polling 1s fallback（CPU 上升）
- **gix/git2 混用 bundle 体积**：git2 + gix + similar = 推算 +5-7MB（已在 MVP-07 §H 预算内）· 若 release 超限 → fallback single lib（rename detection 质量可能下降）

## 📝 Notes

- MVP-08 用 `similar` crate（Rust）计算 diff · 前端只做渲染
- 选 HTML 渲染优先（开发快 + a11y 好）· 若性能不足再切 Canvas（记录到 ADR）
- Diff 计算不经过 git2/gix 原生 diff 输出（序列化开销大）· 直接读 blob content → `similar::TextDiff` → 行级结果

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS + PR #63 rollout 生产化 · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

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
| `FileStatus` | 每文件 · 含 path / status / 加减行数 | ~~`import type { FileStatus } from "../bindings/FileStatus"`~~ → **见 §G.5 复用决策** |

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

### G.5 · 与 MVP-07 已落地 binding 的复用决策

MVP-07 已生成以下 7 个 ts-rs binding（位于 `web/src/bindings/`）· MVP-08 实施前必须明确复用 / 新增 / 扩展策略：

| MVP-07 已有 binding | MVP-08 §G.1 预期 | 决策 | 理由 |
|---|---|---|---|
| `FileChange { path, status: string, additions, deletions }` | §G.1 `FileChange` 同名 | **复用** · 不在 MVP-08 重新定义 | 字段完全覆盖 MVP-08 需求 · status 是 string 当前足够（M/A/D/R/? 字符值 · enum 化是 G.5.2 升级路径） |
| `FileStatus`（不存在）| §G.1 `FileStatus` 新 enum | **不新增独立 binding** · MVP-08 内部用 string 即可 | 新建 `FileStatus` enum 会和 MVP-07 `FileChange.status: string` 双轨 · 触发 H2 类前端漂移风险 |
| `CommitAuthor` / `CommitDetail` / `CommitParent` | 不需要（MVP-08 只关心文件级 diff · 不展示 commit 元数据 · 那是 MVP-07 已做的） | **不引入** | 范围隔离 |
| `GitLogEntry` / `GitLogQueryRequest` / `GitLogQueryResponse` | 不需要 | **不引入** | 同上 |

#### G.5.1 MVP-08 实际新增的 binding 清单（8 个 · 替换原 §G.1）

| Rust struct / enum | 用途 | 前端 import 路径 |
|---|---|---|
| `DiffRequest` | 触发 diff 计算 · 含 source（commit hash / `"unstaged"` / `"staged"`）+ file path | `import type { DiffRequest } from "../bindings/DiffRequest"` |
| `DiffResponse` | diff 计算结果 · 含 hunks + binary flag + truncated flag | `import type { DiffResponse } from "../bindings/DiffResponse"` |
| `DiffHunk` | 单个 hunk · 含 old_start / new_start / lines | `import type { DiffHunk } from "../bindings/DiffHunk"` |
| `DiffLine` | 每行 · 含 line_type / content / line numbers | `import type { DiffLine } from "../bindings/DiffLine"` |
| `DiffLineType` | enum：Added / Removed / Context | `import type { DiffLineType } from "../bindings/DiffLineType"` |
| `GitStatusRequest` | 查 workspace 状态 · 含 workspace_id | `import type { GitStatusRequest } from "../bindings/GitStatusRequest"` |
| `GitStatusResponse` | 含 staged / unstaged / untracked 3 组 `Vec<FileChange>`（**复用** MVP-07 `FileChange`）| `import type { GitStatusResponse } from "../bindings/GitStatusResponse"` |
| `FileStatusEvent` | fs watch 触发的状态变化推送 event payload | `import type { FileStatusEvent } from "../bindings/FileStatusEvent"` |

> 原 §G.1 `FileStatus` enum **不新增独立 binding**（MVP-08 内部用 string 即可 · 和 MVP-07 `FileChange.status` 一致 · 避免双轨）· 升级到 enum 留 G.5.2

#### G.5.2 升级路径（v0.2 / 触发条件）

未来若发现 `FileChange.status: string` 在 UI 渲染 / 测试 / 类型检查中频繁踩坑（如 typo `"modifed"` · status 字符集合不收敛）· 触发以下升级：

1. 在 MVP-07 spec 加 ADR · 把 `FileChange.status: string` 改为 `FileStatus` enum · 同步 ts-rs binding regenerate
2. MVP-08 frontend 同步用新 enum
3. 当前 MVP-08 实施**不**做这个升级（避免 MVP-08 PR 改 MVP-07 接口 · 范围爬升）

#### G.5.3 实施约定（MVP-08 Phase A）

- `crates/core/src/diff.rs` 新建（含 `DiffRequest` / `DiffResponse` / `DiffHunk` / `DiffLine` / `DiffLineType`）
- `crates/core/src/git_status.rs` 新建（含 `GitStatusRequest` / `GitStatusResponse` / `FileStatusEvent` · **复用** `crates/core/src/git_log.rs` 已 export 的 `FileChange`）
- 6 IPC commands：`diff_compute` / `git_status_query` / `git_status_subscribe` / `git_status_unsubscribe` / `git_status_refresh` / `diff_get_settings`（split/unified 持久化）
- 6 Tauri permissions：`allow-diff-compute` / `allow-git-status-*` 5 条 · 新建 `crates/app/permissions/diff.toml` + `crates/app/permissions/git-status.toml`
- ts-rs bindings 自动生成到 `web/src/bindings/`（含上述 8 个新 + 复用 MVP-07 的 `FileChange`）

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

### H.4 Bundle size 估算（更新版 · 2026-04-22）

| 库 | 引入 MVP | 增量 | 累计 |
|---|---|---|---|
| gix 0.70 | MVP-07（已 done · PR #83） | +X MB（实测 · 待 MVP-08 实施 agent 用 `cargo bloat --release` 量化补） | +X MB |
| git2 0.20 | MVP-08 Phase A | +Y MB（推算 ~2MB） | +X+Y MB |
| similar | MVP-08 Phase A | +Z MB（推算 ~0.5MB · pure Rust 算法） | +X+Y+Z MB |
| notify 6.x | MVP-08 Phase D | +W MB（推算 ~0.3MB） | total |

**实施约定**（Phase A）：
- 跑 `cargo bloat --release --crates -n 30` · 对比 MVP-07 done 后 vs MVP-08 Phase A 实施后的 release binary 体积 · 实际增量写到 PR body
- 若 total > 30MB（`implementation-plan.md` §3.1 bundle 预算上限） · 触发 §H.4.1 fallback 决策

### H.4.1 Bundle 超限 fallback 决策树（不变 · 重申）

1. 保留 git2（状态查询刚需）
2. 保留 gix（读性能刚需 · MVP-07 已上）
3. 替换 similar → `diff` crate（体积更小 · 算法降级为 Myers only · 失去 Patience）
4. 替换 notify → 自写 polling 1s（失去 fs watch 实时性 · macOS 反正 FSEvents 也是 2s 下限 · 影响有限）

### H.5 禁止

- **禁止**引入 Monaco（CLAUDE.md #7 禁区 · 3MB bundle 超限）
- **禁止**在 gix / git2 / similar 之外引入第四个 git 操作库
- gitoxide 生态内 sub-crate（如 `gix-object` / `gix-traverse`）可用

### H.6 · fs watch 跨平台实现选型

**主路径**：`notify` crate 6.x（推荐 ^6.1）

| 平台 | notify 后端 | 已知特性 |
|---|---|---|
| macOS | FSEvents | 2s 延迟下限（系统 API · 无法绕过 · §D 性能门槛 < 500ms 实测可能用 polling fallback）· 跨 fork/move 稳定 |
| Linux | inotify | 实时（< 50ms）· 注意 fd 上限（ulimit · 默认 8192）· 大 repo（10 万文件）可能爆 fd |
| Windows | ReadDirectoryChangesW | 实时 · 注意 buffer overflow（高频改动 batch loss）· MVP 阶段 v0.4 才覆盖 |

**fallback 策略**：

- macOS 实测 fs watch 延迟 > 500ms（FSEvents 系统限制） → 实施 agent 评估两选项：
  - (a) 接受 macOS-only 弱化目标到 `< 2s`（FSEvents 真实下限）· 在 spec §D 注明
  - (b) macOS 走 polling 1s（覆盖 < 500ms 目标 · 但 CPU 上升）
- Linux fd 爆（10 万文件 repo）→ 自动降级 polling 2s（spec §⚠️ 已知风险 增条目）

**实施约定**（Phase D）：
- `crates/core/src/fs_watch.rs` 新建 · 封装 notify crate · 暴露 `subscribe(workspace_id, callback)` API
- IPC event：`git_status_changed { workspaceId: string }`（防抖 200ms · 避免 vim swap 触发风暴）
- 测试：mock `notify::Event` · 验证防抖 + 路径 filter（忽略 `.git/` 内部 · `node_modules/` · `target/`）

### H.7 · fs watch 测试策略

| 层次 | 范围 |
|---|---|
| 单元 | 防抖逻辑 + 路径 filter |
| 集成 | 真 fs · `tempfile` repo · `fs::write()` 触发 → 等 IPC event · 跨平台 macOS/Linux 各跑一遍（Windows 留 v0.4） |
| Soak | 10k 文件改动 / s 持续 1 min · 验证 IPC event rate 不爆（防抖收敛） |

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
5. **对齐 MVP-07 已落地 binding**：`FileChange` 复用决策已明确（§G.5 锁 (a)）· 避免 MVP-08 实施时范围爬升改 MVP-07 接口 · 升级路径留 G.5.2 ✅
