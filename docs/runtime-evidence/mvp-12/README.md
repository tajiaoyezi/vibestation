# MVP-12 Phase A · Runtime Evidence

**Task**: MVP-12 Phase A · Rail graph 数据层 + 4 ts-rs binding + 21 单测  
**Date**: 2026-05-07 · Session 25  
**Author**: Droid (Factory.ai)

## Phase A 范围

数据层 + IPC contract · **无 GUI** · 无 Canvas · 无截图。  
Runtime evidence = 数据 / IPC / 单测输出。

## Evidence Files

| 文件 | 内容 |
|------|------|
| `01-vitest-output.txt` | `pnpm vitest run panels/GitLog/RailGraph` 完整输出 · 21 PASS |
| `02-cargo-test-output.txt` | `cargo build -p vibestation-app` 输出 · 含 ts-rs 生成的 4 个 RailGraph*.ts |
| `03-h2-regression-proof.txt` | H2 regression proof · 改字段名 typecheck fail · 回滚后 PASS |
| `04-ts-rs-bindings-list.txt` | `ls RailGraph*.ts` · 4 文件 + 内容 |

## Key Metrics

- vitest: **21 tests PASS** (4 test files)
- cargo build: **4 RailGraph*.ts** auto-generated via ts-rs
- cargo test --workspace: **596 tests** (prior total) + new infra, 0 failures
- H2 proof: rename `refs_hash` → typecheck fails with TS2339, rollback → PASS

## Phase A Acceptance Coverage

- A.1: buildRailGraphInputFromGitLog consumes MVP-07 GitLogEntry data ✓
- A.2: Valid output on linear_20 / branchy_1k / kernel_100k fixtures ✓
- A.3: Deterministic (10-run stability test for linear_20) ✓
- A.4: Root commit laneIndex >= 0 ✓
- A.5: Merge commit (2+ parents) data recorded correctly ✓
- A.6: Detached HEAD isHead by OID prefix match ✓
- A.7: refs normalization zero-loss count ✓
- A.8: 6 Phase A snapshots (light/dark × 3 fixture) ✓
- G.1-G.4: 4 ts-rs payload structs + H2 regression proof ✓
- G.5: Reuses GitLogEntry / BranchInfo bindings ✓
- G.6: Exactly 4 new bindings ✓
