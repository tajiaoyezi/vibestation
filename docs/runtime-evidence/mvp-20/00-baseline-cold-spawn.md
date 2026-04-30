# 00 · Frontend cold spawn baseline

> 测量日期：2026-04-30
> 测量者：Arbiter（用户实测）+ Claude Code（数据整理）
> 环境：macOS 15.x · zsh + oh-my-zsh
> Vibestation：main @ 099095f（PR #189+#190+#191+#192 已合 · Phase B 接入 IPC 前）
> 测量方法：临时改 `web/src/panels/Terminal/PaneTerminal.tsx` 加 `console.time`（紧贴 `invoke("pane_pty_spawn")` 之前）/ `console.timeEnd`（`pane_pty_stdout` 首次回调时）· 测完 git checkout 还原

## 用途

提供 **frontend e2e cold spawn baseline**（IPC → xterm onData）· 用于：

- 推导 **A1b** end-to-end 目标值（≤ baseline P50 × 0.5 = **415 ms**）
- 推导 **A2** cold 兜底容差（P99 ±10% = **[1099, 1343] ms**）
- 跟 backend cold spawn benchmark 交叉验证 IPC overhead

## 原始数据（10 样本 · 已排序）

| # | 延迟 (ms) |
|---|-----------|
|  1 |   792.10 |
|  2 |   800.95 |
|  3 |   808.70 |
|  4 |   810.31 |
|  5 |   811.33 |
|  6 |   848.79 |
|  7 |  1096.15 |
|  8 |  1110.92 |
|  9 |  1160.47 |
| 10 |  1220.70 |

## 统计

| 指标 | 值 |
|---|---|
| min | 792 ms |
| **P50** | **830 ms** |
| mean | 946 ms |
| P90 | 1160 ms |
| **max (≈P99)** | **1221 ms** |
| stddev | 168 ms |

## 双峰分布观察

- ~800ms 段：5 次 · 推测 omz 加载缓存命中
- ~1100ms 段：4 次 · cold load
- 1221ms：1 次 · 系统抖动

## 与 backend baseline 对比

backend benchmark `cold_spawn_baseline`（同环境 · 10 样本）：

| 指标 | Frontend (IPC→onData) | Backend (spawn→stdout) | 差异（IPC overhead） |
|---|---|---|---|
| P50 | 830 ms | 796 ms | **34 ms** |
| mean | 946 ms | 854 ms | 92 ms |
| max | 1221 ms | 1102 ms | 119 ms |

**结论**：frontend 比 backend 多 ~30-100ms（Tauri IPC roundtrip + Vite event listener dispatch + xterm onData callback）。这是 A1b 必须计入的 e2e 开销。

## 推导的 acceptance 目标

| Acceptance | 公式 | 目标值 |
|---|---|---|
| A1a warm hit IPC→onData | spec 锁定 | ≤ **200 ms** |
| A1b warm e2e | baseline P50 × 0.5 | ≤ **415 ms** |
| A2 cold P99 ±10% | baseline P99 ± 10% | ∈ **[1099, 1343] ms** |

实测见同目录 `01-warm-hit.md` 和 `02-cold-path.md`。
