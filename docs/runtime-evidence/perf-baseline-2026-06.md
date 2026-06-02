# Perf 基线 · 2026-06-02 · gix 0.84 / 低风险批后

> **基线 commit**：`cc7ff3f`（Merge PR #438 · gix 0.84 migration）
>
> **采样**：Criterion 默认（release · 100 samples / 5s target，个别 bench 自动降 sample）
>
> **Raw**：`H:\devlopment\code\perf-criterion-raw.txt` · `H:\devlopment\code\perf-pty-raw.txt`（PR body 全文粘贴）

| bench | 指标 | 数值（Criterion estimate 或 P50/P99） | 备注 |
|-------|------|--------------------------------------|------|
| **workspace_query** | snapshots_warm/no_index（run0） | 245.06 ms [241.11 · **245.06** · 249.04] | 无复合索引 · 1M rows |
| workspace_query | snapshots_warm/with_index（run0） | 10.766 ms [10.575 · **10.766** · 10.957] | 有 `idx_snapshots_ws_profile_snap` |
| workspace_query | workspace_list_100/no_index | 20.334 µs [19.867 · **20.334** · 20.832] | 100 行 list |
| workspace_query | workspace_list_100/with_index | 14.528 µs [14.198 · **14.528** · 14.876] | `idx_workspaces_last_opened` |
| **git_status_bench** | git_status_query_1k/statuses_query | 157.43 ms [142.37 · **157.43** · 173.83] | 1k 文件 status |
| git_status_bench | git_status_ipc_1k/serde_json_roundtrip/300 | 50.581 µs [50.195 · **50.581** · 50.980] | IPC 序列化 roundtrip |
| **diff_bench** | diff_compute/similar_1k_lines/1000 | 2.6098 ms [2.5155 · **2.6098** · 2.7203] | **gix 0.84 读路径** |
| diff_bench | diff_compute/similar_10k_lines/10000 | 42.235 ms [40.324 · **42.235** · 43.551] | **gix 0.84 读路径** |
| diff_bench | diff_pure_similar/lines/1000 | 643.20 µs [631.98 · **643.20** · 653.77] | 纯算法对照 |
| diff_bench | diff_truncation/100k_lines_reject | 7.9928 ms [7.8309 · **7.9928** · 8.1271] | 截断拒绝 |
| **git_ops_bench** | stage_single_file | 16.769 ms [16.347 · **16.769** · 17.247] | |
| git_ops_bench | commit_typical | 19.859 ms [19.276 · **19.859** · 20.698] | |
| git_ops_bench | stage_all_1000_files | 2.4277 s [2.3693 · **2.4277** · 2.4879] | 1000 文件 stage |
| **pane_layout_bench** | split_layout · solo→horizontal_2pane | 356.84 ns [346.47 · **356.84** · 368.16] | 内存布局 · 无 I/O |
| pane_layout_bench | apply_smart_layout · 2x2→AiAndRunner | 469.32 ns [460.98 · **469.32** · 477.82] | |
| **branch_bench** | branch_list_10/list | 5.2609 ms [4.7679 · **5.2609** · 5.8875] | |
| branch_bench | branch_list_1000/list | 193.37 ms [187.37 · **193.37** · 199.23] | |
| branch_bench | branch_checkout_clean/checkout | 22.759 ms [21.154 · **22.759** · 24.865] | |
| **git_sync_bench** | git_sync_push_1mb_100commits/push | 217.77 ms [202.91 · **217.77** · 236.55] | 已完成 |
| git_sync_bench | git_sync_pull_ff_1mb_100commits/pull_ff | 327.44 ms [297.54 · **327.44** · 364.14] | 已完成 |
| git_sync_bench | git_sync_pull_conflict_abort/pull_conflict_abort | **skipped** | `cargo bench -p vibestation-core` 在此 panic：`local-conflict\r\n` vs `\n`（Windows CRLF · `git_sync_bench.rs:275`） |
| **rebase_bench** | rebase_10_commits_clean | 212.75 ms [207.95 · **212.75** · 217.85] | |
| rebase_bench | rebase_100_commits_clean | 2.2257 s [2.1500 · **2.2257** · 2.3088] | |
| rebase_bench | cherrypick_single | 29.831 ms [29.155 · **29.831** · 30.576] | |
| rebase_bench | crash_recovery_detection_clean | 10.941 ms [10.683 · **10.941** · 11.209] | |
| **pty_pool_bench** | cold_spawn_baseline · spawn→first_stdout | P50 **104.31 ms** · P90 113.02 ms · min 92.19 ms | n=10 · `cargo test --test pty_pool_bench -- --ignored` |
| pty_pool_bench | cold_path_with_pool_disabled | P50 **105.70 ms** · P90 112.55 ms · min 89.38 ms | pool 禁用 · 等价 Cold |
| pty_pool_bench | warm_hit_with_pool | **skipped** | iter 0 panic：`expected warm hit … but got Cold`（`pty_pool_bench.rs:188`）；进程挂起 >60s 后人工终止 |

## 环境（Windows 本机 · CPU / RAM / rustc 版本）

| 项 | 值 |
|----|-----|
| OS | Microsoft Windows NT 10.0.26200.0（x64） |
| CPU | AMD Ryzen 7 9800X3D 8-Core Processor |
| RAM | 47.1 GiB |
| rustc | `rustc 1.95.0 (59807616e 2026-04-14)` |
| cargo | `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| 工作目录 | `H:\devlopment\code\wt-perf`（worktree `docs/perf-baseline-2026-06` @ `cc7ff3f`） |

## 执行说明

1. **首轮**：`cargo bench -p vibestation-core 2>&1 | tee ..\perf-criterion-raw.txt`
   - 顺序跑至 `git_sync_bench` 时 `pull_conflict_abort` CRLF 断言失败，后续 `pane_layout_bench` / `rebase_bench` / `workspace_query` 未执行。
2. **补跑**：`cargo bench -p vibestation-core --bench workspace_query --bench pane_layout_bench --bench rebase_bench`（结果追加至同一 raw 文件，标记 `RERUN`）。
3. **pty_pool**：`cargo test -p vibestation-core --test pty_pool_bench -- --ignored --nocapture` → `perf-pty-raw.txt`；`warm_hit_with_pool` 失败后测试挂起，已 `Stop-Process` 清理，无残留 cargo/pty 进程。

## Raw 输出（贴关键摘要 · 或注明完整 raw 在 PR）

### `cargo bench` 失败点（git_sync）

```
thread 'main' panicked at crates\core\benches\git_sync_bench.rs:275:17:
assertion `left == right` failed
  left: "local-conflict\r\n"
 right: "local-conflict\n"
error: bench failed, to rerun pass `-p vibestation-core --bench git_sync_bench`
```

### diff_bench 代表行（gix 0.84）

```
diff_compute/similar_1k_lines/1000    time:   [2.5155 ms 2.6098 ms 2.7203 ms]
diff_compute/similar_10k_lines/10000  time:   [40.324 ms 42.235 ms 43.551 ms]
```

### pty_pool · cold_spawn 统计块

```
=== cold_spawn_baseline · backend spawn→first_stdout (n = 10) ===
  P50:       104.31 ms
  P90:       113.02 ms
  max:       116.40 ms
```

完整 Criterion / test 输出见 PR body（`perf-criterion-raw.txt` ~100 KB · `perf-pty-raw.txt` ~14 KB）。