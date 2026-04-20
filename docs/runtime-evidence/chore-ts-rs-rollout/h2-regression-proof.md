# H2 compile-time regression proof · ts-rs rollout

本文档证明：ts-rs 落地到 MVP-02 IPC contract 后，H2 类 `camelCase` / `snake_case` drift
事故会在 **tsc compile time** 前移 catch，不再需要等到 runtime 用户点 Delete 才暴露。

## 背景

MVP-02 H2 事故（PR #47 修复 · 见 `docs/PROGRESS.md`）：Rust `#[serde(rename_all =
"camelCase")]` 输出 `workspaceId`，但前端 `interface WorkspaceMetadata` 误声明
`workspace_id`，两端 interface 分离维护，`tsc` / CI 全部放行，直到用户真 click
Delete 才 runtime 报 "missing required key id"。

SPIKE-08 §A 的选型结论：ts-rs 作为 source of truth，禁止手写 TS interface。
本 PR 把选型落地到 MVP-02 现有两个 IPC contract struct（`WorkspaceMetadata` /
`LayoutState`），并做以下回归实验证明 compile-time drift catching 生效。

## 实验步骤

在 `crates/core/src/workspace.rs` `WorkspaceMetadata` 的 `workspace_id` field 上
加 `#[ts(rename = "workspaceIdRenamedForProof")]` · 模拟 Rust 端单独改了 TS 字段
名而忘了同步前端的场景。Rust field 本身不动 · 所有 Rust 代码继续 compile。

```rust
pub struct WorkspaceMetadata {
    #[ts(rename = "workspaceIdRenamedForProof")]
    pub workspace_id: String,
    ...
}
```

然后按标准流程重新生成 bindings + 跑前端 typecheck：

```
$ cargo build -p vibestation-app
$ pnpm -C web typecheck
```

## 实验结果（2026-04-20）

### 1. `cargo build` 成功（Rust 端不感知 drift · 符合预期）

```
   Compiling vibestation-core v0.1.0
   Compiling vibestation-app v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.38s
```

### 2. Bindings 被重新生成（verified by file hash）

`web/src/bindings/WorkspaceMetadata.ts` 的 `workspaceId: string` 变成
`workspaceIdRenamedForProof: string`。

### 3. `pnpm typecheck` **FAIL** · 10 处引用全被抓到

```
src/App.tsx(77,27): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/App.tsx(276,42): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/App.tsx(353,50): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/App.tsx(365,52): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/App.tsx(368,48): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/App.tsx(369,56): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/components/PrimarySidebar.tsx(51,46): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/components/PrimarySidebar.tsx(51,65): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/components/PrimarySidebar.tsx(53,48): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
src/components/PrimarySidebar.tsx(67,39): error TS2339: Property 'workspaceId' does not exist on type 'WorkspaceMetadata'.
 ELIFECYCLE  Command failed with exit code 2.
```

## 结论

**PASS**：H2 类 Rust ↔ TS drift 在 ts-rs 生态下 **必然** 被 `tsc` catch · 无法
进到 `pnpm build` / CI · 更无法到 runtime。根因制度化成立。

对 CI 的影响：未来任何 IPC contract 改动必须同时改 Rust struct 和前端 import /
usage · 任一漏改 `pnpm typecheck` 立即 FAIL · rule 15 "CI 绿 ≠ runtime 过" 的盲
区在 IPC contract 维度被填补。

## 回滚

实验结束后 `#[ts(rename = "workspaceIdRenamedForProof")]` annotation 已回滚。
本 PR 最终 commit 不含 proof 改动 · 只含 ts-rs 集成本身。

## 相关

- SPIKE-08 报告：`docs/spikes/SPIKE-08-report.md`
- H2 根因记录：`docs/PROGRESS.md` · session 10 终极末段
- build.rs 实现：`crates/app/build.rs`
- ADR-011 runtime evidence location：本目录结构 `docs/runtime-evidence/<task-id>/`
