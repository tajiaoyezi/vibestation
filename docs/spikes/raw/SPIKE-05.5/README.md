# SPIKE-05.5 · raw 数据

对应 report：[`docs/spikes/SPIKE-05.5-report.md`](../../SPIKE-05.5-report.md)
对应源码：[`docs/spikes/code/SPIKE-05.5/`](../../code/SPIKE-05.5/)

## 目录索引

- `compare-table.csv`：报告里的最终比对表
- `shared/`：共享读线程策略（single-yes / four-yes / interactive-tui / correctness）
- `per-session/`：每 session 一 reader thread 对照（single-yes / four-yes / interactive-tui / correctness）

每个 `run-*` 目录至少包含：
- `summary.json` / `summary.md`
- `queue-depth.csv`
- `rss-over-time.csv`
- `frontend-drain.csv`
- `progress.log`
- `app.log`

`correctness` 目录额外包含 `correctness-summary.{md,json}` 与 `resize-winch.txt`。
