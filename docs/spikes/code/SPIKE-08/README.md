# SPIKE-08 · IPC contract + runtime 双层防御 POC

对应 report：[`docs/spikes/SPIKE-08-report.md`](../../SPIKE-08-report.md)
对应 spec：[`docs/tasks/SPIKE-08-e2e-and-contract-harness.md`](../../../tasks/SPIKE-08-e2e-and-contract-harness.md)

## 来源

- **交付 agent**：Codex CLI（2026-04-20）
- **原始工作目录**：`/private/tmp/spike-08-work/docs/spikes/code/SPIKE-08`
- **冷备**：`spike-tmp/archive/SPIKE-08/`（gitignored · 含 `node_modules` / `target` / trace / Docker 产物）

## 目标

用一个独立的 mini Tauri 2 + SolidJS app 验证两层防御：

1. **Contract layer**：Rust `serde(rename_all = "camelCase")` + `ts-rs` 自动生成 TS bindings，替换手写 interface。
2. **Runtime layer**：E2E 覆盖 `create/list/delete mock-workspace` golden path，补上 compile-time 之外的交互回归。

## 目录

- `src/`：SolidJS 前端，直接 import `src/bindings/*.ts`
- `src-tauri/src/contract.rs`：5 个 IPC contract struct（Rust source of truth）
- `src-tauri/src/lib.rs`：3 个 command（`create_workspace` / `list_workspaces` / `delete_workspace`）
- `src-tauri/build.rs`：`cargo build` 时生成 TS bindings + `src/bindings/index.ts`
- `src/bindings/`：生成产物（必须 committed，CI 用于防漏跑 codegen）
- `scripts/`：contract/E2E/replay/cleanup 辅助脚本

## 复现

```bash
cd docs/spikes/code/SPIKE-08
pnpm install

# 生成 bindings + 前端 typecheck/build
pnpm build

# 浏览器层 smoke（B.2）
pnpm dev

# Tauri 本地开发（用于手工 smoke 或 B.3 实验）
pnpm tauri:dev

# 仅构建 debug app（给 Linux tauri-driver / contract check 用）
pnpm tauri:build:smoke

# 检查 bindings 是否 committed 且最新
pnpm contract:check
```

## 关键结论溯源

- contract 导出：`../../raw/SPIKE-08/contract-export.log`
- Rust rename 导致 `tsc` 失败：`../../raw/SPIKE-08/h2-contract-regression.log`
- Playwright browser trace：`../../raw/SPIKE-08/playwright-browser-trace.zip`
- Linux tauri-driver smoke：`../../raw/SPIKE-08/tauri-driver-linux-smoke.log`
- runtime H2 回归失败：`../../raw/SPIKE-08/h2-runtime-regression.log`
