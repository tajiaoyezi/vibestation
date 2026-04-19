# A · 4 Tab yes 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **61.49 MB/s**
- UI drain throughput: **14.93 MB/s**
- total drop: **465.18 MB**
- peak RSS: **118112 KB**
- reader threads: **4**

## Sessions

### tab-1
- throughput(read): 15.29 MB/s
- throughput(drain): 3.73 MB/s
- queue depth avg/max: 240.12 / 256
- drop: 120620 chunks / 115.49 MB
- read calls: 159788 · avgReadBytes=1003.65 · readSyscall=62.14µs · enqueue=1.39µs
- invoke latency p50/p99: 22.00 / 303.00 ms
- preview:
```text
<headless>
```

### tab-2
- throughput(read): 15.19 MB/s
- throughput(drain): 3.73 MB/s
- queue depth avg/max: 240.00 / 256
- drop: 119487 chunks / 114.40 MB
- read calls: 158655 · avgReadBytes=1003.72 · readSyscall=62.53µs · enqueue=1.41µs
- invoke latency p50/p99: 22.00 / 310.00 ms
- preview:
```text
<headless>
```

### tab-3
- throughput(read): 15.91 MB/s
- throughput(drain): 3.73 MB/s
- queue depth avg/max: 240.74 / 256
- drop: 127119 chunks / 121.61 MB
- read calls: 166287 · avgReadBytes=1003.12 · readSyscall=62.82µs · enqueue=1.38µs
- invoke latency p50/p99: 22.00 / 389.00 ms
- preview:
```text
<headless>
```

### tab-4
- throughput(read): 15.10 MB/s
- throughput(drain): 3.72 MB/s
- queue depth avg/max: 239.98 / 256
- drop: 118817 chunks / 113.68 MB
- read calls: 157857 · avgReadBytes=1003.26 · readSyscall=62.70µs · enqueue=1.35µs
- invoke latency p50/p99: 22.00 / 311.00 ms
- preview:
```text
<headless>
```

## Notes

- 自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。
