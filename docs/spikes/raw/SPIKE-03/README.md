# SPIKE-03 · benchmark raw 数据

对应 report：[`docs/spikes/SPIKE-03-report.md`](../../SPIKE-03-report.md)
对应源码：[`docs/spikes/code/SPIKE-03/`](../../code/SPIKE-03/)

## 来源

- **产生时间**：2026-04-19（opencode agent 实测当日）
- **测试机**：macOS · Apple Silicon · SPIKE-01 Phase A 同机环境
- **目标 repo**：opencode 选定的样本 git repo（≥ 10000 commits · 详见文件内 repo 字段）

## 文件

| 文件 | 内容 |
|---|---|
| `measurements.json` (12 KB) | 完整 benchmark raw · git2 + gix × Scenario × 10 iterations · report §B 数据源头 |
| `smoke.json` (3 KB) | Smoke 测试初步 pass/fail |
| `smoke-time-only.json` (3 KB) | Smoke 时间单项 |

## 如何引用

Report 里 "gix log -100 warm P99 12.65ms vs git2 24964ms（1973× 快）" 等具体数字都能在 `measurements.json` 对应 scenario + engine 字段溯源。JSON 格式遵循 `code/SPIKE-03/src/lib.rs` 中 `Measurement` struct 定义。

## 注意

- 这些数据是**决策依据快照** · 不要修改
- 若未来 gix/git2 bump 版本需要重测 · 走 SPIKE-03 spec 流程 · 新 raw 归档到同目录的新子目录（如 `retest-YYYY-MM-DD/`）
