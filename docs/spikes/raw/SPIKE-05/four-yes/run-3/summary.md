# A · 4 Tab yes 10s

- 持续时间：10.0s
- 总吞吐(read path)：127.30 MB/s
- UI 吞吐(drain path)：16.46 MB/s
- 总读取：1272.98 MB
- 总 drain：164.55 MB
- 总 drop：1107.43 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：118736 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 31.80 MB/s
- throughput(drain): 4.12 MB/s
- queue: depth=256, avg=247.46, max=256
- drop: 283571 chunks / 276.55 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-1
```

### tab-2
- command: `yes tab-2`
- throughput(read): 31.85 MB/s
- throughput(drain): 4.12 MB/s
- queue: depth=256, avg=247.48, max=256
- drop: 284062 chunks / 277.05 MB
- preview:
```text
tab-2
tab-2
tab-2
```

### tab-3
- command: `yes tab-3`
- throughput(read): 31.86 MB/s
- throughput(drain): 4.11 MB/s
- queue: depth=256, avg=247.48, max=256
- drop: 284262 chunks / 277.22 MB
- preview:
```text
tab-3
tab-3
tab-3
ta
```

### tab-4
- command: `yes tab-4`
- throughput(read): 31.80 MB/s
- throughput(drain): 4.11 MB/s
- queue: depth=256, avg=247.45, max=256
- drop: 283630 chunks / 276.60 MB
- preview:
```text
tab-4
tab-4
tab-4
ta
```

## Notes

- 自动切 Tab，验证切换不冻结、scrollback 不串流。
