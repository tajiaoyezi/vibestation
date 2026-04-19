# SPIKE-04 · redb vs rusqlite benchmark + safety 源码（v2 accepted 版）

对应 report：[`docs/spikes/SPIKE-04-report.md`](../../SPIKE-04-report.md)
对应 ADR：[`docs/adr/ADR-005-local-storage.md`](../../../adr/ADR-005-local-storage.md)（accepted · supersede redb → rusqlite）
对应 spec：[`docs/tasks/SPIKE-04-storage-benchmark.md`](../../../tasks/SPIKE-04-storage-benchmark.md)
后续 Spike：[`SPIKE-04.5`](../../../tasks/SPIKE-04.5-rusqlite-safety-verification.md)（rusqlite B.1-5 on-rusqlite 补齐）

## 来源

- **交付 agent**：OpenCode agent（2026-04-19）
- **主 agent review**：Claude Code Sonnet 4.6
- **版本**：v2 accepted（v1 因 4 个 CRITICAL 问题被 BLOCK · v2 补做后 accept · 详见 report v1→v2 追溯段）
- **原始归档**：
  - `spike-tmp/archive/spike-04-review-v2/`（gitignored · v2 交付 tarball 冷备 · 148 KB）
  - `spike-tmp/archive/spike-04-work/`（gitignored · v1 含完整 target · 472 MB · 留存证据链）
- **本目录**：从 v2 review accepted 版直接归档 · 约 130 KB

## 结构

```
SPIKE-04/
├── Cargo.toml · Cargo.lock
├── src/
│   ├── main.rs              # bench 入口
│   ├── redb_bench.rs        # §A redb 性能 + §B.1-5 redb safety
│   ├── rusqlite_bench.rs    # §A rusqlite 性能
│   ├── safety.rs            # 674 行 · B.1-5 safety 测试全量（redb 视角）
│   ├── git2_smoke.rs        # §C git2 write smoke
│   └── stats.rs             # P50/P99 统计工具
└── benches/
    ├── storage.rs           # Criterion storage bench harness
    └── git2_smoke.rs        # Criterion git2 write smoke
```

## 关键文件说明

| 文件 | 作用 | 备注 |
|---|---|---|
| `src/safety.rs` (674 行) | B.1-5 全量实现 | **⚠️ 全部针对 redb** · rusqlite 版见 SPIKE-04.5 |
| `src/redb_bench.rs` | A 性能 + B.2 关键证据 | 测出 redb 2.6.3 B.2 坏库检测 silent FAIL（致命缺陷） |
| `src/rusqlite_bench.rs` | A 性能 | 证明 rusqlite A 过关 · 但 B.1-5 on-rusqlite 未测 |

## 如何复现

```bash
cd docs/spikes/code/SPIKE-04
cargo build --release
# A 性能（10M 行 · 多次迭代）
./target/release/spike-04-storage-bench --mode perf
# B.1-5 safety（redb 视角 · 重点看 B.2 silent FAIL）
./target/release/spike-04-storage-bench --mode safety
# Criterion bench
cargo bench
```

## SPIKE-04 已知瑕疵（留给 SPIKE-04.5）

本 Spike 只证明 **redb 2.6.3 B.2 FAIL → 锁 rusqlite** · **rusqlite B.1-5 应用侧安全防护未实测**。

留给 SPIKE-04.5 补齐的 4 项（代码要重写 · 基于本目录 safety.rs 做增量）：

- H1：B.3 旧版读新 DB 无实际 error assert
- H2：B.4 `auto-backup on target-exists`（spec §87）未测
- H3：B.5 op-log 简化版（1 byte phase） · 本 Spike 要 production 级（per-tx_id + manifest）
- H4：range scan 测试场景 1M 行 vs spec 字面 100 行歧义

## raw 数据

`docs/spikes/raw/SPIKE-04/`：
- `full-run-output.txt` · `run2-output.txt`：v2 bench 输出
- `git2-smoke-log.txt`：§C git2 write smoke log（hash `bbaee4d...`）

## 注意

- 本目录代码**不进生产**（生产由 MVP-02/06/10/19 独立实施）
- 决策结论已锁 rusqlite · 本目录源码是**决策证据链** · 不是实现参考
- SPIKE-04.5 交付后 · 本目录保持不动（历史快照） · 新 safety 代码归档到 `docs/spikes/code/SPIKE-04.5/`
