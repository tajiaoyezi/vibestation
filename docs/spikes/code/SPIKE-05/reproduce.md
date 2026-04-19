# reproduce

> 在 `spike-tmp/spike-05-pty/` 目录执行。

## 准备

```bash
./scripts/bench-single-tab.sh 3
./scripts/bench-4tab.sh 3
./scripts/check-correctness.sh 3
./scripts/hol-frontend-slow.sh 3
./scripts/hol-ipc-saturated.sh 1
./scripts/hol-hidden-throttle.sh 1
./scripts/soak-10min.sh 1 1
```

## 产物对应

- A 单 Tab：`raw-data/single-yes/run-*/summary.{md,json}`
- A 交互 TUI：`raw-data/interactive-top/run-*/summary.{md,json}`
- A 4 Tab：`raw-data/four-yes/run-*/summary.{md,json}`
- B.1 soak：`raw-data/soak-10min/run-1/`
- B.2 hidden：`raw-data/hidden-5min/run-1/`
- B.4.1 render slow：`raw-data/hol-frontend-slow/run-*/`
- B.4.2 IPC queue 满：`raw-data/hol-ipc-saturated/run-1/`
- B.4.3 hidden throttle：`raw-data/hol-hidden-throttle/run-1/`
- C correctness：`raw-data/correctness/run-*/`

## 聚合报告

- `reports/A-short-burst.md`
- `reports/B1-soak.md`
- `reports/B2-hidden.md`
- `reports/B3-channel-arch.md`
- `reports/B4-hol-blocking.md`
- `reports/C-correctness.md`
- `SUMMARY.md`
