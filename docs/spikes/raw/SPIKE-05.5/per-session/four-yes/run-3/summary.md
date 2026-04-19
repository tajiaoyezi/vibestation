# A · 4 Tab yes 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **54.07 MB/s**
- UI drain throughput: **10.86 MB/s**
- total drop: **431.65 MB**
- peak RSS: **118288 KB**
- reader threads: **4**

## Sessions

### tab-1
- throughput(read): 13.29 MB/s
- throughput(drain): 2.70 MB/s
- queue depth avg/max: 241.84 / 256
- drop: 110331 chunks / 105.82 MB
- read calls: 138569 · avgReadBytes=1005.96 · readSyscall=73.43µs · enqueue=1.45µs
- invoke latency p50/p99: 8.00 / 320.00 ms
- preview:
```text
<headless>
```

### tab-2
- throughput(read): 13.63 MB/s
- throughput(drain): 2.72 MB/s
- queue depth avg/max: 242.21 / 256
- drop: 113558 chunks / 108.93 MB
- read calls: 142016 · avgReadBytes=1006.24 · readSyscall=71.67µs · enqueue=1.40µs
- invoke latency p50/p99: 12.00 / 320.00 ms
- preview:
```text
<headless>
```

### tab-3
- throughput(read): 13.27 MB/s
- throughput(drain): 2.71 MB/s
- queue depth avg/max: 241.93 / 256
- drop: 110145 chunks / 105.46 MB
- read calls: 138566 · avgReadBytes=1004.45 · readSyscall=73.48µs · enqueue=1.40µs
- invoke latency p50/p99: 12.00 / 318.00 ms
- preview:
```text
<headless>
```

### tab-4
- throughput(read): 13.88 MB/s
- throughput(drain): 2.72 MB/s
- queue depth avg/max: 242.58 / 256
- drop: 116195 chunks / 111.42 MB
- read calls: 144648 · avgReadBytes=1005.84 · readSyscall=72.57µs · enqueue=1.44µs
- invoke latency p50/p99: 13.00 / 325.00 ms
- preview:
```text
<headless>
```

## Notes

- 自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。
