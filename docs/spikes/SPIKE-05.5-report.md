# SPIKE-05.5 · PTY visible throughput + per-session fallback 对照

> **Task spec**：[`docs/tasks/SPIKE-05.5-pty-visible-throughput-fallback.md`](../tasks/SPIKE-05.5-pty-visible-throughput-fallback.md)
> **执行者**：Codex CLI（2026-04-19）
> **结论**：**shared-reader 不是 visible throughput 瓶颈 · ADR-003 可 accepted · per-session 不值得作为默认降级**

---

## 1 · 比对结果总表

| 场景 | 策略 | read p50 | read p99 | drain p50 | drain p99 | invoke p50 | invoke p99 | reader threads |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 单 Tab yes | shared-reader | 32.14 | 45.04 | 7.79 | 8.77 | 8.00 | 15.00 | 1 |
| 单 Tab yes | per-session-reader | 24.70 | 25.47 | 8.42 | 8.80 | 8.00 | 180.00 | 1 |
| 4 Tab yes | shared-reader | 43.48 | 52.32 | 14.58 | 14.73 | 22.00 | 345.00 | 1 |
| 4 Tab yes | per-session-reader | 61.47 | 61.49 | 12.86 | 14.93 | 22.00 | 499.00 | 4 |

## 2 · Profiling 结论

### 2.1 shared-reader 是否是瓶颈？

不是。

证据：

- 4 Tab 场景下，**per-session read-path p50 = 61.47 MB/s**，显著高于 shared 的 **43.48 MB/s**。
- 但 UI drain p50 并没有更好：shared **14.58 MB/s**，per-session **12.86 MB/s**。
- 单 Tab 也类似：drain p50 只从 **7.79 → 8.42 MB/s**，差异极小。

这意味着：**reader 侧更快 ≠ UI drain 更快**。瓶颈在 reader 之后。

### 2.2 当前真正的热点

shared / per-session 两种策略在 4 Tab 场景下的 **invoke latency p50 都约 22ms**，而 polling cadence 只有 4ms。

这说明前端每次 drain 的往返时间已经远大于 polling 间隔，JS loop 与 Tauri invoke 本身已经饱和。shared-reader 只负责把数据放进 bounded queue；真正卡住的是：

1. `drain_session` invoke 往返
2. JS 侧拼接 payload / 更新状态
3. xterm `write()` 消费链路

## 3 · Pass / Fail 对照 Acceptance

| Acceptance | 结果 | 说明 |
|---|---|---|
| 定位 shared-reader 瓶颈 | ✅ | 已证明瓶颈不在 shared-reader，而在 invoke/JS/xterm downstream |
| 实现 per-session reader 对照 | ✅ | `SPIKE055_STRATEGY=per-session` |
| 单 Tab / 4 Tab 各跑 3 次 | ✅ | raw-data 下 2 × 2 × 3 全量产物齐全 |
| 给出翻 A / 留 B / 降级建议 | ✅ | 建议 **翻 A**，不采用 per-session 默认降级 |

## 4 · correctness

- shared correctness：resize `40 100` · fd delta 0
- per-session correctness：resize `40 100` · fd delta 0

## 5 · 推荐决策

**推荐：把 `CLAUDE.md` #15 从 B 翻到 A，锁定 shared-reader。**

理由：

- shared-reader 的原始担忧（HOL / 无界内存 / 多 Tab cascade）已在 SPIKE-05 解决。
- SPIKE-05.5 进一步证明，即使换成 per-session reader，visible throughput 仍不过线。
- 因此，继续把 #15 维持在 B 只会误导后续工作，把 IPC/xterm 问题误判成 PTY 架构问题。

## 6 · 原始数据索引

- Compare 表：`docs/spikes/raw/SPIKE-05.5/compare-table.csv`
- shared 单 Tab：`docs/spikes/raw/SPIKE-05.5/shared/single-yes/`
- shared 4 Tab：`docs/spikes/raw/SPIKE-05.5/shared/four-yes/`
- per-session 单 Tab：`docs/spikes/raw/SPIKE-05.5/per-session/single-yes/`
- per-session 4 Tab：`docs/spikes/raw/SPIKE-05.5/per-session/four-yes/`
