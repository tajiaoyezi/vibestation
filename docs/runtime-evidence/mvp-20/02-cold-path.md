# 02 · Cold path with pool disabled（A2 等价性验证）

> 测量日期：2026-04-30
> 环境：macOS 15.x · zsh + oh-my-zsh
> 源码：`crates/core/tests/pty_pool_bench.rs::cold_path_with_pool_disabled`

## 测量定义

PtyPool 配置 `enabled=false` · `pool.take` 立即返回 `TakeResult::Cold` · 上层 fallback 到 `manager.spawn` 走原 cold spawn 路径。验证 spec A2："pool disable 时延迟 P99 与 MVP-04 baseline 差异 ≤ 10%"。

## 原始数据（10 样本 · 已排序）

| # | 延迟 (ms) |
|---|-----------|
|  1 |   775.51 |
|  2 |   789.24 |
|  3 |   790.70 |
|  4 |   794.11 |
|  5 |   794.59 |
|  6 |   794.70 |
|  7 |   800.87 |
|  8 |   981.56 |
|  9 |  1017.40 |
| 10 |  1035.73 |

## 统计

| 指标 | 值 |
|---|---|
| min | 775.51 ms |
| **P50** | **794.65 ms** |
| mean | 857.44 ms |
| P90 | 1017.40 ms |
| max | 1035.73 ms |
| stddev | 101.83 ms |

## 与 cold_spawn_baseline 对比

跑同环境的 backend `cold_spawn_baseline`（pool 完全不存在）数据：

| 指标 | cold_spawn_baseline | cold_path_pool_disabled | 差异 | 容差 ±10% |
|---|---|---|---|---|
| P50 | 795.59 ms | 794.65 ms | **0.12%** | ✅ |
| mean | 853.62 ms | 857.44 ms | 0.45% | ✅ |
| P90 | 991.87 ms | 1017.40 ms | 2.57% | ✅ |
| max | 1102.35 ms | 1035.73 ms | -6.04% | ✅ |
| stddev | 104.92 ms | 101.83 ms | -2.95% | ✅ |

**所有指标差异 < 7%** · 远在 A2 容差 ±10% 内。

## A2 验证

| | 目标 | 实测 | 通过 |
|---|---|---|---|
| P99 差异（用 max 近似） | ≤ ±10% | -6.04% | ✅ |
| P50 差异 | n/a | 0.12% | ✅ |
| 行为等价 | 视觉无回归 | take 立即 Cold · fallback `manager.spawn` 路径完全一致 | ✅ |

**结论**：pool disable 时 PTY pool 系统**完全透明** · 不引入额外开销 · 也不破坏既有 cold spawn 行为。用户怀疑 pool 异常时一键 toggle off 即可回到 MVP-04 baseline 体验。

## 与 frontend baseline 交叉验证

frontend baseline（[00-baseline-cold-spawn.md](./00-baseline-cold-spawn.md)）：

- frontend cold P50 = 830 ms
- backend cold P50 = 795 ms（baseline）/ 794 ms（pool disabled）
- 差异（IPC overhead）：~35 ms（稳定）

frontend pool-disabled 实测虽未做（避免重复用户操作）· 但根据 backend pool-disabled 与 baseline 的近似性（P50 差异 0.12%）· 推断 frontend pool-disabled 也应在 [747, 913] ms 范围（baseline 830 ±10%）· **A2 frontend 等价性预期成立**。
