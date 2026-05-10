# MVP-16 Phase D · Criterion bench baseline

> Spec acceptance §A.9 / §B.9 / §C.9 / §D.9 / §F.5 时间预算量化 · 全部远低于阈值。
> Bench 源码：[`crates/core/benches/rebase_bench.rs`](../../../../crates/core/benches/rebase_bench.rs)
> 复跑命令：`cargo bench --bench rebase_bench`

## 运行平台 baseline

| 字段 | 值 |
|---|---|
| OS | macOS 26.3.1 (Darwin 25.3.0) |
| Arch | arm64 (Apple Silicon) |
| CPU | Apple M2 Max |
| Rust | toolchain stable（workspace `rust-toolchain.toml`） |
| Criterion | 0.5 |
| git2 | 0.20 (ADR-007 写路径) |
| 跑时间 | 2026-05-10 session 27 · ~1 min 内 7 bench 全收 |

⚠️ Linux x86_64 + Linux arm64 baseline 推 v0.2 W17 跨平台 capture · 由 Arbiter dev VM 跑 · 数据归档同目录 `linux-x64.log` / `linux-arm64.log`。

## Bench 数据 vs spec 时间预算

数字格式：`[lower_bound · estimate · upper_bound]`（criterion 95% confidence interval）。

| Bench | Spec 目标 | 实测 estimate | 倍数余裕 | 验证项 |
|---|---|---|---|---|
| `rebase_10_commits_clean` | < 1s | **54.6 ms** [53.6, 54.6, 55.9] | 18.3× | spec §A.9 |
| `rebase_100_commits_clean` | < 5s | **608 ms** [592.6, 608.0, 629.6] | 8.2× | spec §A.9 |
| `merge_no_ff_50_commits` | < 3s | **35.4 ms** [34.8, 35.4, 36.0] | 84.7× | spec §B.9 |
| `cherrypick_single` | < 1s | **5.9 ms** [5.83, 5.89, 5.95] | 169.5× | spec §C.9 |
| `cherrypick_range_10` | < 5s | **36.2 ms** [35.6, 36.2, 36.8] | 138.1× | spec §C.9 |
| `conflict_3way_50_files_status` | < 2s（后端检测） | **20.4 ms** [20.03, 20.4, 20.83] | 98.0× | spec §D.9 后端部分 |
| `crash_recovery_detection_clean` | < 200ms | **1.9 ms** [1.88, 1.90, 1.92] | 105.3× | spec §F.5 |

**全部通过 · 全部远低于阈值 · macOS arm64 baseline confirmed。**

## Raw output

完整 criterion 输出见 [`rebase_bench-raw.log`](./rebase_bench-raw.log) · 含每个 bench 的 warm-up / sample / outlier / analysis 段。

复跑后的 HTML 报告位于 `target/criterion/` 下（gitignored · 不入 repo · 跑过的人本地查看）。

## 关键观察

1. **rebase_100 vs rebase_10 比率 ~11×**（54.6 ms → 608 ms）· 接近 commit 数线性 · 符合 git2 顺序 cherry-pick plan 状态机的 O(n) 复杂度
2. **merge_no_ff 在 50 commit 下仅 35 ms**（比 rebase_10 还快）· 因为 merge 单次 tree merge + 1 commit · 不像 rebase 要 10 次 cherry-pick replay
3. **cherrypick_range_10 (36 ms) ≈ rebase_10 (54.6 ms) · 比 rebase 略快**· cherry-pick plan 不必 hard-reset 到 onto · 少一次 reset 开销
4. **conflict_3way_50_files 后端 status 仅 20 ms** · git2 `index.conflicts()` 迭代 50 entry 极快 · 前端 UI 渲染才是真正的 bottleneck（Phase D part B GUI 量化范围）
5. **crash recovery clean repo 1.9 ms** · 检测 `.git/MERGE_HEAD` / `.git/CHERRY_PICK_HEAD` / SQLite operation_state 三路 OR · 干净仓库直接 short-circuit

## Phase D 全貌

本 README 覆盖 Phase D 的 **bench 部分**。完整 Phase D 还需：

- 🟡 GUI screenshot baseline（rebase editor / 3-way conflict view / continue/abort/skip 按钮 · 5-7 PNG）· 推 Phase D part B · Arbiter capture
- 🟡 Linux 跨平台 bench 复跑（x86_64 + arm64）· 推 v0.2 W17 dev VM · 数据归档 `linux-*.log`
- 🟡 真实工程仓库 100k commit 历史的 rebase smoke（验证 git2 在大库不退化）· 可选 · 不阻塞 GA

bench 部分单独 PR · spec 仅翻 Phase D 行的"bench done · GUI/跨平台 deferred"状态字段（不翻 Phase D 整体 done）。

## 复跑

```bash
cd crates/core
cargo bench --bench rebase_bench
# 报告位置：target/criterion/<bench-name>/report/index.html
```

跑一次约 50-60 秒 · CPU 单核占用 · 不耗 RAM。
