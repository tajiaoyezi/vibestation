# A · 单 Tab yes 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **25.47 MB/s**
- UI drain throughput: **8.80 MB/s**
- total drop: **166.65 MB**
- peak RSS: **117440 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 25.47 MB/s
- throughput(drain): 8.80 MB/s
- queue depth avg/max: 233.25 / 256
- drop: 174824 chunks / 166.65 MB
- read calls: 267269 · avgReadBytes=999.44 · readSyscall=36.29µs · enqueue=1.06µs
- invoke latency p50/p99: 8.00 / 11.00 ms
- preview:
```text
<headless>
```

## Notes

- shared-reader vs per-session-reader 对照基线。
