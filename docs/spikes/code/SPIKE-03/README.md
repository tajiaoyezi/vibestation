# SPIKE-03 · git2 vs gix benchmark 源码

对应 report：[`docs/spikes/SPIKE-03-report.md`](../../SPIKE-03-report.md)
对应 ADR：[`docs/adr/ADR-007-git-stack.md`](../../../adr/ADR-007-git-stack.md)（accepted）
对应 spec：[`docs/tasks/SPIKE-03-git2-gix-read-benchmark.md`](../../../tasks/SPIKE-03-git2-gix-read-benchmark.md)

## 来源

- **交付 agent**：OpenCode agent（2026-04-19）
- **主 agent review**：Claude Code Sonnet 4.6 · 对照 spec §B.6 accept（一次通过 · 未退回）
- **原始归档**：`spike-tmp/archive/spike-03-work/`（gitignored · 含完整 target/ build 冷备 · 约 1.4 GB）
- **本目录**：仅源码 + benches + Cargo manifest · 约 100 KB · 可 clone 后直接复现

## 结构

```
SPIKE-03/
├── Cargo.toml · Cargo.lock
├── src/
│   ├── lib.rs           # Engine/Scenario 枚举 + 通用测量逻辑
│   └── bin/
│       ├── measure.rs   # git2 / gix 双栈测量入口
│       └── report.rs    # P50/P99 统计 + JSON 输出
└── benches/
    └── git_read.rs      # Criterion benchmark harness
```

## 如何复现

准备：Rust stable toolchain · 本地某个 git repo 路径（建议用 vibestation 本身 · 或任何有 ≥ 10000 commits 的 repo）

```bash
cd docs/spikes/code/SPIKE-03
cargo build --release
# 实测入口：measure 输出 raw · report 聚合 P99
./target/release/measure --repo /path/to/target-repo --iterations 10 --output /tmp/run.json
./target/release/report --input /tmp/run.json
# 或直接 Criterion bench
cargo bench
```

## 关键结论（对照 raw 数据）

- Raw 数据：[`docs/spikes/raw/SPIKE-03/`](../../raw/SPIKE-03/)
- `measurements.json` · `smoke.json` 是 opencode 实测当日产出
- Report §B 引用的 "gix log -100 warm P99 12.65ms vs git2 24964ms" 等数据直接可在 `measurements.json` 溯源

## 注意

- 本目录代码作为**决策依据归档** · 不直接进生产
- 生产实现由 MVP-08 (`git log`) / MVP-07 (`git status`) 等独立 spec 负责
- 若未来需要回归验证 gix/git2 新版本 · 在此基础上 bump deps 重跑即可
