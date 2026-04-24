---
id: MVP-09
type: mvp
title: Stage/Unstage + Commit 操作（git2 写）
status: ready
owner:
phase: W10-W11
depends_on: ["MVP-08", "SPIKE-04"]
blocks: []
blocked_by: []
blocked_note:
estimate: 4d
plan_ref: implementation-plan.md §10.1
risk_ref:
reviewer: Kimi
---

# MVP-09: Stage/Unstage + Commit 操作

> **状态**：`ready`
> **依赖**：MVP-08（Status 面板）· SPIKE-04 §C（git2 写路径 smoke test）

---

## 🎯 目标（Goal）

在 Git Status 面板上支持 Stage / Unstage 单文件或整体操作，+ Commit UI（勾文件 + 输入 message + 可选 amend）。**不含 push/pull/fetch**（v0.2）。

## 📖 背景（Context）

- `implementation-plan.md §10.1` MVP B 折中方案：**保留 commit，砍 push/pull/fetch**
- `CLAUDE.md` #13（A 栏永久锁定）：Git 栈 = **读 `gix 0.70` · 写 `git2 0.20` 混用**
- 本 MVP 是**纯写路径** · 全用 git2 0.20 · 不碰 gix

---

## 🎨 功能范围（Scope）

**Do**：
- Stage 操作：
  - 单文件 Stage：Status 面板每行 ✓ 按钮
  - Stage All Unstaged：组标题 "Stage All" 按钮
  - Stage All Untracked：同上
- Unstage 操作：
  - 单文件 Unstage：Staged 组每行 ✗ 按钮
  - Unstage All Staged：组标题 "Unstage All" 按钮
- Commit UI：
  - Status 面板底部：消息框 + "Commit" 主 CTA + "Amend" 复选框
  - 多行消息支持（subject + body 分离，blank line 自动插入）
  - Commit 成功后 Status 刷新 + toast 提示 + Git Log 刷新
- Author 信息：从 git config 读取 `user.name` / `user.email`
- 快捷键：`⌘↵`（mac）/ `Ctrl+↵` 提交

**Don't**：
- Push / Pull / Fetch（v0.2）
- Branch operations（v0.2）
- Rebase / Merge / Cherry-pick（v0.3）
- Commit signing（GPG）（v0.2+）
- Partial staging（stage hunks）（v0.2）

## 🛠 实施进度

MVP-09 估时 4d，拆 4 Phase 串行实施：

| Phase | 范围 | 状态 | PR |
|-------|------|------|----|
| Phase A · git2 写路径后端 + IPC | stage / unstage / commit / amend 后端封装 + IPC commands + ts-rs bindings + 单元 / 集成测试 | ⏳ todo | — |
| Phase B · Status 面板操作接线 | 复用 MVP-08 Status 面板，接单文件/批量 stage/unstage、乐观 UI、刷新链路 | ⏳ todo | — |
| Phase C · Commit UI + 错误流 | message composer / amend / identity dialog / detached HEAD / pre-commit hook stderr / Git Log refresh | ⏳ todo | — |
| Phase D · runtime 证据 + 性能量化 | 截图 / 录屏 + Stage/Commit 性能量化 + 放 `docs/runtime-evidence/mvp-09/` | ⏳ todo | — |

**Phase A 实施起点 checklist**（让 agent 接 spec 后 5 min 内启动）：

- [ ] `crates/core/Cargo.toml` 加 `git2` 已存在（继承 MVP-07）· 不需要新增依赖
- [ ] 新建 `crates/core/src/git_ops.rs`（不和 `git_status.rs` 混 · 写路径独立模块）
- [ ] git2 API 调用链 ready-to-use（参考 §H.4 表）：
  - Stage：`Repository::index()` → `Index::add_path()` → `Index::write()`
  - Unstage：`Repository::head()` → tree 对应 path → `Index::remove_path()` / `add_tree_entry()` → `Index::write()`
  - Commit：`Repository::signature_default()` → `Repository::commit(parents, author, committer, message, tree, ...)`
  - Amend：`Commit::amend(Some("HEAD"), None, None, None, None, None, Some(message))`
- [ ] IPC commands 注册顺序（`crates/app/src/lib.rs` `invoke_handler!`）：
  - `stage` / `unstage` / `commit` / `amend` / `get_git_identity` / `set_git_identity`
- [ ] permission toml：`crates/app/permissions/git_ops.toml` 新建 · 含 6 个 `allow-{name}`
- [ ] capability `default.json` 引用上述 permission
- [ ] ts-rs binding 自动生成到 `web/src/bindings/`（`build.rs` 触发）
- [ ] fixture：`tests/fixtures/mvp-09/` 用 `tempfile` crate 运行时生成（不要硬编码本地路径）

**下次 agent 起点**：Phase A

**依赖关系说明**：MVP-09 依赖 MVP-08 Status 面板存在；自身四个 phase 内部串行。MVP-09 文件域与 MVP-04 Phase F / MVP-08 实施 **完全隔离** · 可并行（MVP-09 只动 `crates/core/src/git_ops.rs` + `crates/app/src/lib.rs` 注册 + `web/src/panels/CommitBar/`）。

## 🖼 UI 引用

- Status 面板底部：消息框 + CTA 按钮（参考 `design/directions/1-calm-studio.html` Bottom Panel）
- Commit 消息框：等宽字体（JetBrains Mono），72 字符宽度提示线

## ✅ Acceptance

### A. Stage / Unstage

- [ ] Unstaged 组每行有 ✓ 按钮 → 点击 stage 该文件
- [ ] Staged 组每行有 ✗ 按钮 → 点击 unstage 该文件
- [ ] 组标题有 "Stage All" / "Unstage All" 批量按钮
- [ ] 操作后 Status 面板立即刷新：点击到 UI 反馈 < 50ms（乐观 UI）· git call 后 < 100ms 校正完成（测 3 次取 P99）· 失败 revert + toast error 文案（如 `"无法 stage：{file} 已被删除"`）
- [ ] Stage All 批量操作显示 spinner / progress indicator · 1000 文件场景不阻塞 UI（参考 D 段 Stage All < 2s 目标）

### B. Commit

- [ ] Status 面板底部有消息框（多行 · JetBrains Mono 等宽字体）
- [ ] "Commit" 按钮 disabled 状态：
  - No staged files → disabled + tooltip `"无暂存变更"`
  - Empty message → disabled + tooltip `"需要提交信息"`
- [ ] 点击 Commit：
  - 调用 git2 创建 commit 使用 staged tree
  - Author / committer 从 `git config` 读（`user.name` / `user.email`）
  - Message 规范：第一行 subject（< 72 字符建议 · 等宽区显示 `|` 提示线）+ blank line 自动插入 + body
  - 支持中文 message（UTF-8）· 测样本：message 含中文 + emoji + 长 subject（100 字符）· `git log --format=%s <sha>` 显示一致 · 编码测 3 次
- [ ] "Amend" 勾选：Commit 修改最后一个 commit（`git commit --amend`）· message 自动 pre-fill 上一个 commit 的 message · 用户修改 message 后取消 Amend → message **保留不清空**
- [ ] Commit 成功：toast `"已提交 {shortsha}"`（shortsha 长度 7 · 格式 `abc1234`）· toast 持续 3s · 点击 toast 跳转到 Git Log 定位该 commit（MVP-07 联动）

### C. 错误处理

- [ ] git2 调用失败 → 明确错误提示 + **保留消息框内容不清空**
- [ ] 没有 identity（`user.name` 未设）→ 弹对话框：字段 Name + Email + `"保存到 local git config"` 复选框（默认勾选）· 取消 → 退出 commit · 确认 → 写入 `<repo>/.git/config` · **不污染 `~/.gitconfig`**
- [ ] Detached HEAD → 弹确认对话框：`"当前处于 detached HEAD 状态 · commit 将不会关联到任何分支 · 继续？"` · 二选一（取消 / 继续提交）
- [ ] Pre-commit hook 失败（exit code != 0）→ commit 回退 · 显示 hook stderr **最后 20 行** · 可复制 · 消息框保留

### D. 性能

- [ ] Stage 单文件 < 100ms（测 3 次取 P99 · Criterion bench 或 `performance.now()`）
- [ ] Commit < 500ms（典型仓库 · 测 3 次取 P99 · fixture：vibestation 自身 repo）
- [ ] Stage All 1000 文件 < 2s（测 3 次取 P99 · fixture：linux kernel 复制 1000 文件变更）

### E. 测试 fixture

- [ ] 正常 commit（单文件 / 多文件）
- [ ] 空 staged 试 commit → 拒绝（Commit 按钮 disabled + tooltip）
- [ ] Amend（message pre-fill · 取消 Amend 保留用户修改）
- [ ] 中文 message + 中文文件名（UTF-8 一致性）
- [ ] `.gitignore` 外的 untracked 文件 stage
- [ ] 已 staged 后 working tree 又改 → Status 正确显示两份（staged 和 unstaged 同文件）：测样本为 stage 后再改同文件 · `git status --porcelain` 输出 `'MM path'` · MVP-09 Status 面板应在 Staged 和 Unstaged 两组都显示该文件
- [ ] Fixture 管理：`tests/fixtures/mvp-09/` 下准备 6 个小 fixture repo（.git 含）· 或用 `tempfile` crate 运行时创建 · 测试结束清理

## 🧪 测试策略

| 层次 | 范围 |
|------|------|
| 单元 | git2 wrapper（Repository / Index / Tree / Signature）|
| 集成 | Stage → Commit → Log 读新 commit 链路 |
| E2E | 完整 flow：改文件 → Status → Stage → Commit → 验证新 commit |
| 手动 QA | Amend / detached HEAD / 权限问题 / pre-commit hook |

## 💾 数据模型变更

无新 table。所有变更落在 git repo 本身。

Commit UI 状态（message 草稿 / Amend 勾选态）持久化到现有 `app_settings` 表（MVP-03 已建）：

```rust
// key 示例："commit_draft:{workspace_id}" → 用户未提交的 message 草稿
// key 示例："commit_amend:{workspace_id}" → "true" | "false"
```

> 切换 workspace 时恢复对应草稿 · 避免误跨 workspace。

## ⚠️ 已知风险

- **用户 git config 不完整**（user.name/email 未设）：弹窗引导用户填，写入 `<repo>/.git/config` local 而非 global（避免污染全局）
- **中文 commit message 编码**：SPIKE-04 §C 已验证 git2 0.20 UTF-8 支持（见 `docs/spikes/SPIKE-04-report.md`）
- **Pre-commit hooks**：若 repo 有 pre-commit hook 可能拖慢 commit → 不改 git2 行为 · UI 显示 `"提交中…"` 转圈 · hook 失败显示 stderr 最后 20 行
- **Detached HEAD commit**：允许但弹警告（见 Acceptance C）· commit 后 HEAD 指向新 commit · 不关联任何分支

## 📝 Notes

- MVP-09 的 commit **不签 GPG**（keychain 集成复杂，v0.2+）
- 不做 "Commit Verification"（v0.3+）
- 未来 push 按钮会出现在 Commit 成功 toast 上（v0.2 接入）

## §G. IPC Contract（ts-rs）

> **依据**：[ADR-014 · IPC contract source of truth = Rust struct + ts-rs codegen](../adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md)（H2 根因消除 · SPIKE-08 §A PASS + PR #63 rollout 生产化 · 规范源头）。所有 IPC struct 必须遵循 ADR-014 §规范 5 条 + H2 regression proof 6 步。

本 MVP 所有 IPC struct 必须单点维护——**Rust struct 为 source of truth**，禁止前端手写对偶 TypeScript interface。

### G.1 本 MVP 涉及的 IPC struct 清单（预期）

| Rust struct | 用途 | 前端 import 路径 |
|-------------|------|-----------------|
| `StageRequest` | 批量 stage · 含 workspace_id + file_paths | `import type { StageRequest } from "../bindings/StageRequest"` |
| `UnstageRequest` | 批量 unstage · 含 workspace_id + file_paths | `import type { UnstageRequest } from "../bindings/UnstageRequest"` |
| `CommitRequest` | 提交参数 · 含 workspace_id + message + amend | `import type { CommitRequest } from "../bindings/CommitRequest"` |
| `CommitResponse` | 提交结果 · 含 sha / short_sha / message / author / timestamp | `import type { CommitResponse } from "../bindings/CommitResponse"` |
| `CommitAuthor` | Author / Committer 信息（name + email + timestamp） | `import type { CommitAuthor } from "../bindings/CommitAuthor"` · **复用** MVP-07 已有 binding · 见 §G.5 |
| `GitConfigIdentity` | 读出的 user.name / user.email | `import type { GitConfigIdentity } from "../bindings/GitConfigIdentity"` |
| `SetGitIdentityRequest` | 设置 identity · name + email + scope | `import type { SetGitIdentityRequest } from "../bindings/SetGitIdentityRequest"` |
| `StageResult` | 批量 stage 结果 · staged_count + failed 列表 | `import type { StageResult } from "../bindings/StageResult"` |
| `CommitError` | 提交失败 enum · 含 payload | `import type { CommitError } from "../bindings/CommitError"` |

> 实际 struct 名和字段以实施 PR 为准，但**必须**全部走 ts-rs 自动生成。

### G.2 derive 模板（以 `CommitRequest` + `CommitError` 为例）

```rust
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    pub workspace_id: String,
    pub message: String,
    pub amend: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CommitError {
    NoStagedFiles,
    IdentityMissing,
    HookFailed { stderr: String, exit_code: i32 },
    DetachedHead,
    Git2Error { message: String },
}
```

> `CommitError` 因含 payload（stderr / exit_code / message）必须用 tagged union（`#[serde(tag = "kind")]`）· 前端 TS 生成 discriminated union。

### G.3 强制规范

- [ ] 所有 IPC struct + enum 必须 `#[derive(Debug, Clone, Serialize, Deserialize, TS)]` + `#[ts(export)]` + `#[serde(rename_all = "camelCase")]`
- [ ] 简单无 payload enum 用 string union（`rename_all` + 无 tag）· 含 payload enum 用 tagged union（`#[serde(tag = "kind")]`）
- [ ] bindings 由 `crates/app/build.rs` 在 `cargo build` 时自动生成到 `web/src/bindings/`
- [ ] 前端**禁止**手写 `interface CommitRequest { ... }` 或 `type CommitError = { ... }`——所有类型必须从 `./bindings/*` import
- [ ] `.prettierignore` 已排除 `web/src/bindings/`（防止 prettier 与生成格式冲突）

### G.4 H2 类 regression proof

复用 MVP-04 §G.3 定义（见 `docs/tasks/MVP-04-multi-tab-terminal.md` §G.3），流程如下：

1. 临时在任一 IPC struct（如 `CommitRequest`）的某个字段上加 `#[ts(rename = "xxxProof")]`
2. 运行 `cargo build -p vibestation-app`（Rust 端编译通过）
3. 运行 `pnpm -C web typecheck`
4. **预期**：`tsc` 报 `TS2339: Property 'xxx' does not exist on type 'CommitRequest'`——FAIL 证明防御生效
5. **回滚**：撤销 `#[ts(rename = ...)]`，确认 `pnpm typecheck` 恢复 PASS

> 本 proof 只需做一次，结果写入 PR description 或 `docs/runtime-evidence/MVP-09/`（如实施 PR 本身含 ts-rs 集成）。

### G.5 · 与上游已落地 binding 的复用决策

MVP-09 实施前必须明确复用 / 新增边界，避免和 MVP-07 / MVP-08 已生成 binding 冲突：

| 已有 binding | MVP-09 §G.1 涉及 | 决策 | 理由 |
|---|---|---|---|
| `CommitAuthor { name, email, timestamp }`（MVP-07 已生成）| §G.1 `CommitAuthor` | **复用** · 不在 MVP-09 重新定义 | `CommitResponse.author / committer` 直接引用现有 binding；字段含 `timestamp: number`（对齐 MVP-07 derive 模式 `#[ts(type = "number")]`） |
| `FileChange { path, status, additions, deletions }`（MVP-07 已生成 · MVP-08 §G.5 已锁复用）| Commit 面板输入来源 | **复用** · 不造平行类型 | MVP-09 Stage/Unstage 操作的对象来自 MVP-08 Status 面板；禁止新建 `StagedFile` / `StatusFile` / `GitStatusItem` 等平行 struct |
| `GitStatusResponse`（MVP-08 将生成）| Commit 面板 staged 文件列表来源 | **复用** · 不重新定义 | MVP-09 只消费 `GitStatusResponse.staged` 数组，不重新查询 status |

#### G.5.1 保持独立的类型

以下类型因语义不同，**不**复用现有 binding，保持为 MVP-09 新增：

| Rust struct | 独立理由 |
|---|---|
| `GitConfigIdentity` | 表示 "identity 对话框可编辑表单 / git config 读取结果"，只含 `name + email`（无 timestamp）· 与 `CommitAuthor`（含 timestamp）语义不同 |
| `SetGitIdentityRequest` | 写操作请求体 · 含 `scope` 字段（local / global）· 上游无对应类型 |
| `StageResult` | MVP-09 专属批量操作结果 · 含 `staged_count + failed` 列表 · 上游无对应类型 |
| `CommitError` | MVP-09 专属错误枚举 · 含 payload（stderr / exit_code / message）· 上游无对应类型 |

> **核心原则**：MVP-09 是 "写操作消费上游读结果" · 输入侧全部复用（`FileChange` / `GitStatusResponse` / `CommitAuthor`）· 输出侧和写请求侧保持独立（`CommitRequest` / `StageResult` / `CommitError` / `GitConfigIdentity`）。

## §H. Git 栈约束（MVP-09 专有 · 纯写路径）

MVP-09 是**纯写路径** · 对齐 CLAUDE.md 决策表 #13（2026-04-19 accepted · [ADR-007](../adr/ADR-007-git-stack.md)）· 必须明确：

### H.1 本 MVP Git 栈

- **主路径 crate**：`git2 0.20`
- **场景**：Stage / Unstage / Commit / Amend / config 读写 / index / tree 操作
- **依据**：
  - SPIKE-04 §C 已验证 git2 0.20 写路径 smoke test 通过（UTF-8 commit / detached HEAD / config 读写 · 见 `docs/spikes/SPIKE-04-report.md`）
  - gix 0.70 的 index / commit 写 API 尚不成熟（参考 MVP-08 §H 判断）· MVP-09 **绝不试水 gix 写路径**

### H.2 不碰的 crate

- **不碰 gix**：gix 0.70 只读路径成熟 · 写路径（index / commit / tree builder）API 不完整
- **不碰 similar / diff crate**：MVP-09 不涉及 diff 算法 · 那是 MVP-08 范围
- **禁止引入第四个 git 库**：如独立 gitoxide sub-crate 若涉及写操作也需 review

### H.3 Bundle size 影响

- MVP-09 不加新 crate · 仅深度使用已有 git2
- 累计 bundle（MVP-07 gix + MVP-08 git2/similar + MVP-09 git2）仍在 MVP-08 §H 预算 **+5-7MB** 内

### H.4 git2 0.20 API 使用要点（实施参考）

| 操作 | git2 API 调用链 |
|------|----------------|
| Stage 单文件 | `Repository::index()` → `Index::add_path(path)` → `Index::write()` |
| Unstage 单文件 | `Repository::head()` → `HEAD tree 对应 path` → `Index::remove_path(path)` 或 `add_tree_entry()` → `Index::write()` |
| Commit | `Repository::signature_default()` 读 config → `Repository::commit(parents, author, committer, message, tree, ...)` |
| Amend | `Commit::amend(Some("HEAD"), None, None, None, None, None, Some(message))` |
| 错误分诊 | `git2::Error::class()` / `git2::Error::code()` → 映射到 `CommitError` enum |

> MVP-09 不新增 git crate，只消费 MVP-08 Status 结果并对 git2 写路径落地。

## 🔗 相关

- `CLAUDE.md` #7 Diff 自建 · #13 Git 栈
- ADR-007 Git 栈混用决策
- SPIKE-04 §C git2 写 smoke test
- 上游：MVP-08 · SPIKE-04
- 下游：v0.2 push/pull

---

**自审四问（2026-04-20）**：
1. **递归完备性**：Acceptance 清单覆盖 Stage/Unstage / Commit / 错误 / 性能 / fixture / IPC contract / Git 栈约束 全维度 ✅
2. **反向场景**：若 TS derive 漏加 → `pnpm typecheck` 立即 FAIL（H2 proof 制度化）· 若 git2 stage 失败 → 乐观 UI revert + toast error（Acceptance A）· 若 identity 缺失 → 弹窗引导不阻塞（Acceptance C）· 若 hook 失败 → 保留 message + 显示 stderr（Acceptance C）✅
3. **边界适用性**：0 staged / 1 staged / 1000 staged · 空 message / 长 message / 中文 message · detached HEAD / 正常 branch · 有 hook / 无 hook · 全适用 ✅
4. **YAGNI**：push/pull/fetch / branch ops / rebase / merge / cherry-pick / GPG signing / partial staging / AI 联动 都明确推后 ✅ · 新增关注点：§G ts-rs contract 覆盖 + §H git2 单路径锁定已显式文档化 ✅
5. **对齐上游 binding**：`CommitAuthor` / `FileChange` / `GitStatusResponse` 复用决策已明确（§G.5）· 避免 MVP-09 实施时造平行类型 · 升级路径留 G.5.1 ✅
