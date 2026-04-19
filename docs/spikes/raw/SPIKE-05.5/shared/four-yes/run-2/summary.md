# A · 4 Tab yes 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **43.48 MB/s**
- UI drain throughput: **14.58 MB/s**
- total drop: **288.47 MB**
- peak RSS: **118624 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 10.89 MB/s
- throughput(drain): 3.65 MB/s
- queue depth avg/max: 233.94 / 256
- drop: 74211 chunks / 72.26 MB
- read calls: 111797 · avgReadBytes=1021.05 · readSyscall=8.42µs · enqueue=0.85µs
- invoke latency p50/p99: 22.00 / 268.00 ms
- preview:
```text
<headless>
```

### tab-2
- throughput(read): 10.91 MB/s
- throughput(drain): 3.65 MB/s
- queue depth avg/max: 233.88 / 256
- drop: 74446 chunks / 72.48 MB
- read calls: 112040 · avgReadBytes=1020.91 · readSyscall=8.50µs · enqueue=0.86µs
- invoke latency p50/p99: 22.00 / 268.00 ms
- preview:
```text
<headless>
```

### tab-3
- throughput(read): 10.84 MB/s
- throughput(drain): 3.64 MB/s
- queue depth avg/max: 233.93 / 256
- drop: 73824 chunks / 71.88 MB
- read calls: 111289 · avgReadBytes=1021.07 · readSyscall=8.29µs · enqueue=0.83µs
- invoke latency p50/p99: 22.00 / 295.00 ms
- preview:
```text
<headless>
```

### tab-4
- throughput(read): 10.84 MB/s
- throughput(drain): 3.65 MB/s
- queue depth avg/max: 233.77 / 256
- drop: 73790 chunks / 71.85 MB
- read calls: 111362 · avgReadBytes=1021.01 · readSyscall=8.24µs · enqueue=0.86µs
- invoke latency p50/p99: 22.00 / 295.00 ms
- preview:
```text
<headless>
```

## Notes

- 自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。
