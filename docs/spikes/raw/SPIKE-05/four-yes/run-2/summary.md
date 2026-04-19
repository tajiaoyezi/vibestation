# A · 4 Tab yes 10s

- 持续时间：10.0s
- 总吞吐(read path)：130.83 MB/s
- UI 吞吐(drain path)：16.38 MB/s
- 总读取：1308.29 MB
- 总 drain：163.83 MB
- 总 drop：1143.47 MB
- 主线程最大 lag：7.00ms
- freeze (>100ms)：0
- 峰值 RSS：123216 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 32.72 MB/s
- throughput(drain): 4.10 MB/s
- queue: depth=256, avg=247.77, max=256
- drop: 293167 chunks / 285.92 MB
- preview:
```text
tab-1
tab-1
tab-1
ta
```

### tab-2
- command: `yes tab-2`
- throughput(read): 32.71 MB/s
- throughput(drain): 4.10 MB/s
- queue: depth=256, avg=247.79, max=256
- drop: 293107 chunks / 285.85 MB
- preview:
```text
tab-2
tab-2
tab-2
```

### tab-3
- command: `yes tab-3`
- throughput(read): 32.67 MB/s
- throughput(drain): 4.10 MB/s
- queue: depth=256, avg=247.81, max=256
- drop: 292755 chunks / 285.46 MB
- preview:
```text
tab-3
tab-3
tab-3
tab-3
```

### tab-4
- command: `yes tab-4`
- throughput(read): 32.73 MB/s
- throughput(drain): 4.08 MB/s
- queue: depth=256, avg=247.87, max=256
- drop: 293512 chunks / 286.23 MB
- preview:
```text
tab-4
tab-4
tab-4
```

## Notes

- 自动切 Tab，验证切换不冻结、scrollback 不串流。
