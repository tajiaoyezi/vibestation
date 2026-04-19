# A · 4 Tab yes 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **38.60 MB/s**
- UI drain throughput: **14.73 MB/s**
- total drop: **238.21 MB**
- peak RSS: **118544 KB**
- reader threads: **1**

## Sessions

### tab-1
- throughput(read): 9.67 MB/s
- throughput(drain): 3.69 MB/s
- queue depth avg/max: 230.62 / 256
- drop: 61378 chunks / 59.74 MB
- read calls: 99390 · avgReadBytes=1020.71 · readSyscall=10.23µs · enqueue=0.92µs
- invoke latency p50/p99: 22.00 / 318.00 ms
- preview:
```text
<headless>
```

### tab-2
- throughput(read): 9.69 MB/s
- throughput(drain): 3.69 MB/s
- queue depth avg/max: 230.75 / 256
- drop: 61476 chunks / 59.85 MB
- read calls: 99511 · avgReadBytes=1020.84 · readSyscall=10.12µs · enqueue=0.91µs
- invoke latency p50/p99: 22.00 / 317.00 ms
- preview:
```text
<headless>
```

### tab-3
- throughput(read): 9.61 MB/s
- throughput(drain): 3.68 MB/s
- queue depth avg/max: 230.37 / 256
- drop: 60848 chunks / 59.24 MB
- read calls: 98739 · avgReadBytes=1021.01 · readSyscall=10.12µs · enqueue=0.88µs
- invoke latency p50/p99: 22.00 / 319.00 ms
- preview:
```text
<headless>
```

### tab-4
- throughput(read): 9.63 MB/s
- throughput(drain): 3.67 MB/s
- queue depth avg/max: 230.44 / 256
- drop: 61011 chunks / 59.38 MB
- read calls: 98875 · avgReadBytes=1020.74 · readSyscall=10.01µs · enqueue=0.89µs
- invoke latency p50/p99: 22.00 / 318.00 ms
- preview:
```text
<headless>
```

## Notes

- 自动切 Tab，观察 reader 策略变化是否真的影响 UI drain。
