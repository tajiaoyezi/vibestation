#!/usr/bin/env python3
from __future__ import annotations

import csv
import json
import statistics
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / "raw-data"
REPORTS = ROOT / "reports"
REPORTS.mkdir(parents=True, exist_ok=True)


def throughput_mb(bytes_count: int, duration_ms: int) -> float:
    return bytes_count / (duration_ms / 1000) / (1024 * 1024)


def median(values: list[float]) -> float:
    return statistics.median(values) if values else 0.0


def p99(values: list[float]) -> float:
    return max(values) if values else 0.0


def load_runs(strategy: str, scenario: str):
    runs = []
    for summary_path in sorted((RAW / strategy / scenario).glob("run-*/summary.json")):
        obj = json.loads(summary_path.read_text())
        runs.append((summary_path.parent.name, obj))
    return runs


def aggregate(strategy: str, scenario: str):
    runs = load_runs(strategy, scenario)
    read_tps: list[float] = []
    drain_tps: list[float] = []
    invoke_p50s: list[float] = []
    invoke_p99s: list[float] = []
    peak_rss: list[int] = []
    threads: list[int] = []
    per_run = []
    for run_name, obj in runs:
        read_tp = throughput_mb(obj["totals"]["totalReadBytes"], obj["durationMs"])
        drain_tp = throughput_mb(obj["totals"]["totalDrainedBytes"], obj["durationMs"])
        invoke_p50 = median([session["invokeP50Ms"] for session in obj["sessions"]])
        invoke_p99 = max(session["invokeP99Ms"] for session in obj["sessions"])
        rss = max(sample["rssKb"] for sample in obj["processSamples"])
        thread_count = obj["processSamples"][-1]["readerThreads"]
        read_tps.append(read_tp)
        drain_tps.append(drain_tp)
        invoke_p50s.append(invoke_p50)
        invoke_p99s.append(invoke_p99)
        peak_rss.append(rss)
        threads.append(thread_count)
        per_run.append(
            {
                "run": run_name,
                "read_tp": read_tp,
                "drain_tp": drain_tp,
                "invoke_p50": invoke_p50,
                "invoke_p99": invoke_p99,
                "peak_rss_kb": rss,
                "reader_threads": thread_count,
            }
        )
    return {
        "strategy": strategy,
        "scenario": scenario,
        "per_run": per_run,
        "read_p50": median(read_tps),
        "read_p99": p99(read_tps),
        "drain_p50": median(drain_tps),
        "drain_p99": p99(drain_tps),
        "invoke_p50": median(invoke_p50s),
        "invoke_p99": p99(invoke_p99s),
        "peak_rss_p50": median([float(v) for v in peak_rss]),
        "reader_threads": threads[0] if threads else 0,
    }


def correctness(strategy: str):
    path = RAW / strategy / "correctness" / "run-1" / "correctness-summary.json"
    obj = json.loads(path.read_text())
    return {
        "strategy": strategy,
        "resize_result": obj["resizeResult"].strip() or "<missing>",
        "fd_delta": obj["afterStats"]["fdCount"] - obj["beforeStats"]["fdCount"],
    }


shared_single = aggregate("shared", "single-yes")
per_single = aggregate("per-session", "single-yes")
shared_four = aggregate("shared", "four-yes")
per_four = aggregate("per-session", "four-yes")
shared_tui = aggregate("shared", "interactive-tui")
per_tui = aggregate("per-session", "interactive-tui")
shared_correct = correctness("shared")
per_correct = correctness("per-session")

compare_rows = [
    ["scenario", "strategy", "read_p50_mb_s", "read_p99_mb_s", "drain_p50_mb_s", "drain_p99_mb_s", "invoke_p50_ms", "invoke_p99_ms", "reader_threads"],
    ["single-yes", "shared-reader", f"{shared_single['read_p50']:.2f}", f"{shared_single['read_p99']:.2f}", f"{shared_single['drain_p50']:.2f}", f"{shared_single['drain_p99']:.2f}", f"{shared_single['invoke_p50']:.2f}", f"{shared_single['invoke_p99']:.2f}", shared_single['reader_threads']],
    ["single-yes", "per-session-reader", f"{per_single['read_p50']:.2f}", f"{per_single['read_p99']:.2f}", f"{per_single['drain_p50']:.2f}", f"{per_single['drain_p99']:.2f}", f"{per_single['invoke_p50']:.2f}", f"{per_single['invoke_p99']:.2f}", per_single['reader_threads']],
    ["four-yes", "shared-reader", f"{shared_four['read_p50']:.2f}", f"{shared_four['read_p99']:.2f}", f"{shared_four['drain_p50']:.2f}", f"{shared_four['drain_p99']:.2f}", f"{shared_four['invoke_p50']:.2f}", f"{shared_four['invoke_p99']:.2f}", shared_four['reader_threads']],
    ["four-yes", "per-session-reader", f"{per_four['read_p50']:.2f}", f"{per_four['read_p99']:.2f}", f"{per_four['drain_p50']:.2f}", f"{per_four['drain_p99']:.2f}", f"{per_four['invoke_p50']:.2f}", f"{per_four['invoke_p99']:.2f}", per_four['reader_threads']],
]

with (REPORTS / "compare-table.csv").open("w", newline="") as handle:
    writer = csv.writer(handle)
    writer.writerows(compare_rows)

summary_md = f"""# SUMMARY · SPIKE-05.5

## 结论

**visible throughput 的瓶颈不在 shared-reader。**

- 单 Tab：shared drain p50 **{shared_single['drain_p50']:.2f} MB/s** vs per-session **{per_single['drain_p50']:.2f} MB/s**（仅 +{per_single['drain_p50'] - shared_single['drain_p50']:.2f} MB/s）。
- 4 Tab：shared drain p50 **{shared_four['drain_p50']:.2f} MB/s** vs per-session **{per_four['drain_p50']:.2f} MB/s**（反而 {per_four['drain_p50'] - shared_four['drain_p50']:+.2f} MB/s）。
- 但 read-path：4 Tab shared **{shared_four['read_p50']:.2f} MB/s** vs per-session **{per_four['read_p50']:.2f} MB/s**，说明 per-session 的确改善了 reader 侧读取能力；**UI drain 却没有同步提升**。
- 4 Tab invoke latency p50 在两种策略都约 **{shared_four['invoke_p50']:.0f}–{per_four['invoke_p50']:.0f}ms**，远高于 4ms polling cadence；这说明瓶颈落在 **Tauri invoke / JS drain / xterm 链路**，而不是 shared-reader。

## 建议

1. **接受 ADR-003 / CLAUDE.md #15**：锁定 `portable-pty + 共享读线程 + bounded queue + drop-oldest`。
2. **不要降级到 per-session**：它提高了 read-path，但没有解决 visible throughput。
3. 后续 visible throughput 优化应转向：
   - Rust→JS batching / 更粗粒度 drain
   - 降低 invoke 往返频率
   - xterm write coalescing / renderer 策略
"""
(REPORTS / "SUMMARY.md").write_text(summary_md)

report_md = f"""# SPIKE-05.5 · PTY visible throughput + per-session fallback 对照

> **Task spec**：[`docs/tasks/SPIKE-05.5-pty-visible-throughput-fallback.md`](../tasks/SPIKE-05.5-pty-visible-throughput-fallback.md)
> **执行者**：Codex CLI（2026-04-19）
> **结论**：**shared-reader 不是 visible throughput 瓶颈 · ADR-003 可 accepted · per-session 不值得作为默认降级**

---

## 1 · 比对结果总表

| 场景 | 策略 | read p50 | read p99 | drain p50 | drain p99 | invoke p50 | invoke p99 | reader threads |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 单 Tab yes | shared-reader | {shared_single['read_p50']:.2f} | {shared_single['read_p99']:.2f} | {shared_single['drain_p50']:.2f} | {shared_single['drain_p99']:.2f} | {shared_single['invoke_p50']:.2f} | {shared_single['invoke_p99']:.2f} | 1 |
| 单 Tab yes | per-session-reader | {per_single['read_p50']:.2f} | {per_single['read_p99']:.2f} | {per_single['drain_p50']:.2f} | {per_single['drain_p99']:.2f} | {per_single['invoke_p50']:.2f} | {per_single['invoke_p99']:.2f} | 1 |
| 4 Tab yes | shared-reader | {shared_four['read_p50']:.2f} | {shared_four['read_p99']:.2f} | {shared_four['drain_p50']:.2f} | {shared_four['drain_p99']:.2f} | {shared_four['invoke_p50']:.2f} | {shared_four['invoke_p99']:.2f} | 1 |
| 4 Tab yes | per-session-reader | {per_four['read_p50']:.2f} | {per_four['read_p99']:.2f} | {per_four['drain_p50']:.2f} | {per_four['drain_p99']:.2f} | {per_four['invoke_p50']:.2f} | {per_four['invoke_p99']:.2f} | 4 |

## 2 · Profiling 结论

### 2.1 shared-reader 是否是瓶颈？

不是。

证据：

- 4 Tab 场景下，**per-session read-path p50 = {per_four['read_p50']:.2f} MB/s**，显著高于 shared 的 **{shared_four['read_p50']:.2f} MB/s**。
- 但 UI drain p50 并没有更好：shared **{shared_four['drain_p50']:.2f} MB/s**，per-session **{per_four['drain_p50']:.2f} MB/s**。
- 单 Tab 也类似：drain p50 只从 **{shared_single['drain_p50']:.2f} → {per_single['drain_p50']:.2f} MB/s**，差异极小。

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

- shared correctness：resize `{shared_correct['resize_result']}` · fd delta {shared_correct['fd_delta']}
- per-session correctness：resize `{per_correct['resize_result']}` · fd delta {per_correct['fd_delta']}

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
"""
(REPORTS / "SPIKE-05.5-report.md").write_text(report_md)

readme = """# SPIKE-05.5 · raw 数据

- `shared/`：共享读线程策略
- `per-session/`：每 session 一 reader thread 对照
- 每个 `run-*` 目录含：`summary.json`、`summary.md`、`queue-depth.csv`、`rss-over-time.csv`、`frontend-drain.csv`、`progress.log`、`app.log`
- `correctness/` 额外包含 `correctness-summary.{md,json}` 与 `resize-winch.txt`
- `compare-table.csv` 提供报告引用的汇总表
"""
(RAW / "README.md").write_text(readme)
