# A · 单 Tab yes 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **24.70 MB/s**
- UI drain throughput: **8.42 MB/s**
- total drop: **162.66 MB**
- peak RSS: **117024 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 24.70 MB/s
- throughput(drain): 8.42 MB/s
- queue depth avg/max: 233.55 / 256
- drop: 170710 chunks / 162.66 MB
- read calls: 259234 · avgReadBytes=998.98 · readSyscall=37.51µs · enqueue=1.03µs
- invoke latency p50/p99: 8.00 / 10.00 ms
- preview:
```text
<headless>
```

## Notes

- shared-reader vs per-session-reader 对照基线。
