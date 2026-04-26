# H2 compile-time regression proof · MVP-10 Phase B Telemetry

本文档证明：MVP-10 Phase B Telemetry / Settings 系列 IPC contract（`AppSettings` ·
`SettingsUpdateRequest` · `TelemetryStatus` · `TelemetryOptInRequest` · `CrashReportPayload` ·
`AppVersionInfo`）落地后，H2 类 `camelCase` / `snake_case` drift 事故会在 **tsc compile time**
前移 catch · 不再需要等到 runtime 用户改设置才暴露。

> **路径说明**：spec §G.4 原文要求 `h2-regression-proof.png`，与 `chore-ts-rs-rollout` ·
> `mvp-07` 两个先例一致 · 本目录用 `h2-regression-proof.md` 承载叙事 + 引用 raw log
> （`h2-regression-proof.log` · 字节级 tsc stderr · 比 PNG 更可 grep / 复现）。这两个先例
> 均为 PR review 通过的同款做法。

## 背景

MVP-02 H2 事故（PR #47 修复）：Rust `#[serde(rename_all = "camelCase")]` 输出 `workspaceId`，
但前端 `interface WorkspaceMetadata` 误声明 `workspace_id`，两端 interface 分离维护，`tsc` /
CI 全部放行 · 直到用户真 click Delete 才 runtime 报 "missing required key id"。

ts-rs 落地（PR #63 chore-ts-rs-rollout）后已对 `WorkspaceMetadata` / `LayoutState` 闭环。
本 PR 把同样的 compile-time gate 验到 MVP-10 Phase B 新引入的 6 个 IPC struct（其中 `AppSettings`
是公开切面最大的 · 选作 H2 proof 标的）。

## 实验步骤

在 `crates/core/src/app_settings.rs` `AppSettings` 的 `font_family` field 上加
`#[ts(rename = "fontNameRenamedForProof")]` · 模拟 Rust 端单独改了 TS 字段名而忘了同步前端
的场景。Rust field 本身不动 · 所有 Rust 代码继续 compile。

```rust
pub struct AppSettings {
    pub theme: String,
    #[ts(rename = "fontNameRenamedForProof")]
    pub font_family: String,
    pub font_size: u32,
    // ...
}
```

然后按标准流程重新生成 bindings + 跑前端 typecheck：

```
$ cargo build -p vibestation-app   # build.rs 会触发 ts-rs export_all()
$ pnpm typecheck                    # tsc --noEmit
```

## 实验结果（2026-04-26）

### 1. `cargo build` 成功（Rust 端不感知 drift · 符合预期）

```
   Compiling vibestation-core v0.1.0 (/Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation/crates/core)
   Compiling vibestation-app v0.1.0 (/Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation/crates/app)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.83s
```

### 2. Bindings 被重新生成（field name 已改）

`web/src/bindings/AppSettings.ts` line 3 ·
`fontFamily: string` 变成 `fontNameRenamedForProof: string`：

```
$ grep "font" web/src/bindings/AppSettings.ts
3:export type AppSettings = { theme: string, fontNameRenamedForProof: string, fontSize: number, ... };
```

### 3. `pnpm typecheck` **FAIL** · 6 处引用全被抓到

完整 stderr 见 [`h2-regression-proof.log`](./h2-regression-proof.log)。摘录：

```
src/panels/Settings/AppearanceGroup.tsx(61,27): error TS2339: Property 'fontFamily' does not exist on type 'AppSettings'.
src/panels/Settings/AppearanceGroup.tsx(63,30): error TS2353: Object literal may only specify known properties, and 'fontFamily' does not exist in type 'Partial<AppSettings>'.
src/stores/settings.ts(11,3): error TS2353: Object literal may only specify known properties, and 'fontFamily' does not exist in type 'AppSettings'.
src/stores/settings.ts(58,41): error TS2339: Property 'fontFamily' does not exist on type 'AppSettings'.
src/stores/settings.ts(78,19): error TS2339: Property 'fontFamily' does not exist on type 'Partial<AppSettings>'.
src/stores/settings.ts(78,70): error TS2339: Property 'fontFamily' does not exist on type 'Partial<AppSettings>'.
 ELIFECYCLE  Command failed with exit code 2.
```

## 结论

**PASS**：H2 类 Rust ↔ TS drift 在 ts-rs 生态下 **必然** 被 `tsc` catch · 无法进到 `pnpm build` /
CI · 更无法到 runtime。MVP-10 Phase B 的 6 个新 IPC contract 都享受同样的保护（任何一个 field
改名 / 删字段 · 前端引用方必然 compile-time FAIL）。

对 CI 的影响：未来 telemetry / settings 任何 IPC contract 改动必须同时改 Rust struct 和前端
import / usage · 任一漏改 `pnpm typecheck` 立即 FAIL · rule 15 "CI 绿 ≠ runtime 过" 的盲区在
MVP-10 维度被填补。

## 回滚

实验结束后 `#[ts(rename = "fontNameRenamedForProof")]` annotation 已回滚 · `cargo build` 重跑
让 bindings 恢复原状。本 PR 最终 commit 不含 proof 改动 · 只含 §C.4 endpoint UI + 本 evidence
文件本身。

## 相关

- 先例 1：`docs/runtime-evidence/chore-ts-rs-rollout/h2-regression-proof.md`（MVP-02 ts-rs 落地）
- 先例 2：`docs/runtime-evidence/mvp-07/H2-regression-proof.md`（MVP-07 Git Log）
- SPIKE-08 报告：`docs/spikes/SPIKE-08-report.md`
- ADR-014 IPC contract source of truth：`docs/adr/ADR-014-ipc-contract-source-of-truth-ts-rs.md`
- ADR-011 runtime evidence location：本目录结构 `docs/runtime-evidence/<task-id>/`
