# A · 单 Tab yes 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **21.36 MB/s**
- UI drain throughput: **6.55 MB/s**
- total drop: **148.03 MB**
- peak RSS: **118880 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 21.36 MB/s
- throughput(drain): 6.55 MB/s
- queue depth avg/max: 235.91 / 256
- drop: 155355 chunks / 148.03 MB
- read calls: 224172 · avgReadBytes=999.19 · readSyscall=45.00µs · enqueue=1.00µs
- invoke latency p50/p99: 8.00 / 180.00 ms
- preview:
```text
<headless>
```

## Notes

- shared-reader vs per-session-reader 对照基线。
