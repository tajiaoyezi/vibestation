# A · 4 Tab yes 10s

- 持续时间：10.0s
- 总吞吐(read path)：123.24 MB/s
- UI 吞吐(drain path)：15.47 MB/s
- 总读取：1232.44 MB
- 总 drain：154.70 MB
- 总 drop：1076.74 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：120560 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 30.84 MB/s
- throughput(drain): 3.86 MB/s
- queue: depth=256, avg=247.75, max=256
- drop: 276395 chunks / 269.54 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-1
```

### tab-2
- command: `yes tab-2`
- throughput(read): 30.77 MB/s
- throughput(drain): 3.87 MB/s
- queue: depth=256, avg=247.76, max=256
- drop: 275669 chunks / 268.82 MB
- preview:
```text
tab-2
tab-2
tab-2
tab-
```

### tab-3
- command: `yes tab-3`
- throughput(read): 30.83 MB/s
- throughput(drain): 3.87 MB/s
- queue: depth=256, avg=247.75, max=256
- drop: 276163 chunks / 269.32 MB
- preview:
```text
tab-3
tab-3
tab-3
ta
```

### tab-4
- command: `yes tab-4`
- throughput(read): 30.80 MB/s
- throughput(drain): 3.87 MB/s
- queue: depth=256, avg=247.77, max=256
- drop: 275891 chunks / 269.06 MB
- preview:
```text
tab-4
tab-4
tab-4
```

## Notes

- 自动切 Tab，验证切换不冻结、scrollback 不串流。
