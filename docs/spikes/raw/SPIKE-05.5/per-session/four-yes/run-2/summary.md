# A · 4 Tab yes 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **61.47 MB/s**
- UI drain throughput: **12.86 MB/s**
- total drop: **485.56 MB**
- peak RSS: **117792 KB**
- reader threads: **4**

## Sessions

### tab-1
- throughput(read): 15.45 MB/s
- throughput(drain): 3.22 MB/s
- queue depth avg/max: 242.01 / 256
- drop: 127611 chunks / 122.20 MB
- read calls: 161369 · avgReadBytes=1004.20 · readSyscall=61.61µs · enqueue=1.39µs
- invoke latency p50/p99: 22.00 / 498.00 ms
- preview:
```text
<headless>
```

### tab-2
- throughput(read): 15.49 MB/s
- throughput(drain): 3.22 MB/s
- queue depth avg/max: 241.95 / 256
- drop: 128067 chunks / 122.61 MB
- read calls: 161758 · avgReadBytes=1004.08 · readSyscall=62.38µs · enqueue=1.41µs
- invoke latency p50/p99: 22.00 / 497.00 ms
- preview:
```text
<headless>
```

### tab-3
- throughput(read): 15.52 MB/s
- throughput(drain): 3.23 MB/s
- queue depth avg/max: 242.06 / 256
- drop: 128232 chunks / 122.79 MB
- read calls: 161981 · avgReadBytes=1004.49 · readSyscall=62.33µs · enqueue=1.38µs
- invoke latency p50/p99: 22.00 / 499.00 ms
- preview:
```text
<headless>
```

### tab-4
- throughput(read): 15.01 MB/s
- throughput(drain): 3.20 MB/s
- queue depth avg/max: 241.75 / 256
- drop: 123196 chunks / 117.95 MB
- read calls: 156701 · avgReadBytes=1004.17 · readSyscall=63.26µs · enqueue=1.41µs
- invoke latency p50/p99: 22.00 / 498.00 ms
- preview:
```text
<headless>
```

## Notes

- 自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。
