# A · 4 Tab yes 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **52.32 MB/s**
- UI drain throughput: **11.80 MB/s**
- total drop: **404.71 MB**
- peak RSS: **118416 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 13.12 MB/s
- throughput(drain): 2.95 MB/s
- queue depth avg/max: 240.86 / 256
- drop: 104242 chunks / 101.55 MB
- read calls: 134674 · avgReadBytes=1021.42 · readSyscall=6.66µs · enqueue=0.75µs
- invoke latency p50/p99: 21.00 / 344.00 ms
- preview:
```text
<headless>
```

### tab-2
- throughput(read): 13.03 MB/s
- throughput(drain): 2.94 MB/s
- queue depth avg/max: 240.83 / 256
- drop: 103471 chunks / 100.79 MB
- read calls: 133821 · avgReadBytes=1021.37 · readSyscall=6.57µs · enqueue=0.74µs
- invoke latency p50/p99: 21.00 / 345.00 ms
- preview:
```text
<headless>
```

### tab-3
- throughput(read): 13.08 MB/s
- throughput(drain): 2.96 MB/s
- queue depth avg/max: 240.77 / 256
- drop: 103753 chunks / 101.09 MB
- read calls: 134232 · avgReadBytes=1021.46 · readSyscall=6.63µs · enqueue=0.75µs
- invoke latency p50/p99: 21.00 / 345.00 ms
- preview:
```text
<headless>
```

### tab-4
- throughput(read): 13.09 MB/s
- throughput(drain): 2.95 MB/s
- queue depth avg/max: 240.85 / 256
- drop: 103983 chunks / 101.28 MB
- read calls: 134373 · avgReadBytes=1021.30 · readSyscall=6.65µs · enqueue=0.75µs
- invoke latency p50/p99: 21.00 / 345.00 ms
- preview:
```text
<headless>
```

## Notes

- 自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。
