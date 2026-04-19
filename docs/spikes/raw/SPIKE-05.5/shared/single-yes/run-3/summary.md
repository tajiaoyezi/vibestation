# A · 单 Tab yes 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **32.14 MB/s**
- UI drain throughput: **6.79 MB/s**
- total drop: **253.36 MB**
- peak RSS: **117456 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 32.14 MB/s
- throughput(drain): 6.79 MB/s
- queue depth avg/max: 241.95 / 256
- drop: 261732 chunks / 253.36 MB
- read calls: 332001 · avgReadBytes=1014.97 · readSyscall=5.85µs · enqueue=0.86µs
- invoke latency p50/p99: 8.00 / 15.00 ms
- preview:
```text
<headless>
```

## Notes

- shared-reader vs per-session-reader 对照基线。
