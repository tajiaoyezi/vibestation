# SPIKE-05 · raw 数据

对应 report：[`docs/spikes/SPIKE-05-report.md`](../../SPIKE-05-report.md)
对应源码：[`docs/spikes/code/SPIKE-05/`](../../code/SPIKE-05/)

## 目录索引

- `single-yes/run-1..3/`：单 Tab `yes` 10s
- `interactive-top/run-1..3/`：5Hz synthetic TUI（`clear + date + ps`）
- `four-yes/run-1..3/`：4 Tab `yes` 10s
- `soak-10min/run-1/`：B.1 10 分钟慢消费者 soak
- `hidden-5min/run-1/`：B.2 hidden tab
- `hol-frontend-slow/run-1..3/`：B.4.1
- `hol-ipc-saturated/run-1/`：B.4.2
- `hol-hidden-throttle/run-1/`：B.4.3
- `correctness/run-1..3/`：resize + cleanup

每个 run 目录包含 `summary.json` / `summary.md` / `queue-depth.csv` / `rss-over-time.csv` / `session-previews.txt` / `app.log`。
