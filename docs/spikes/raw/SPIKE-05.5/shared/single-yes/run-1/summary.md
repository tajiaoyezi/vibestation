# A · 单 Tab yes 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **23.34 MB/s**
- UI drain throughput: **7.79 MB/s**
- total drop: **155.33 MB**
- peak RSS: **120016 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 23.34 MB/s
- throughput(drain): 7.79 MB/s
- queue depth avg/max: 230.44 / 256
- drop: 160664 chunks / 155.33 MB
- read calls: 241344 · avgReadBytes=1013.88 · readSyscall=8.71µs · enqueue=0.97µs
- invoke latency p50/p99: 8.00 / 14.00 ms
- preview:
```text
<headless>
```

## Notes

- shared-reader vs per-session-reader 对照基线。
