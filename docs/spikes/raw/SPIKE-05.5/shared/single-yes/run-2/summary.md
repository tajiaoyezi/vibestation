# A · 单 Tab yes 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **45.04 MB/s**
- UI drain throughput: **8.77 MB/s**
- total drop: **362.57 MB**
- peak RSS: **117920 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 45.04 MB/s
- throughput(drain): 8.77 MB/s
- queue depth avg/max: 243.24 / 256
- drop: 374971 chunks / 362.57 MB
- read calls: 465752 · avgReadBytes=1013.97 · readSyscall=4.55µs · enqueue=0.74µs
- invoke latency p50/p99: 8.00 / 11.00 ms
- preview:
```text
<headless>
```

## Notes

- shared-reader vs per-session-reader 对照基线。
