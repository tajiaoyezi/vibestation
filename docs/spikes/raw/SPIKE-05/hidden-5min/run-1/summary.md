# B.2 · 隐藏 tab 5 min

- 持续时间：300.0s
- 总吞吐(read path)：130.37 MB/s
- UI 吞吐(drain path)：9.32 MB/s
- 总读取：39109.84 MB
- 总 drain：2794.83 MB
- 总 drop：36314.01 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：118608 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 32.57 MB/s
- throughput(drain): 8.95 MB/s
- queue: depth=256, avg=238.26, max=256
- drop: 7264234 chunks / 7086.45 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-
```

### tab-2
- command: `yes tab-2`
- throughput(read): 32.60 MB/s
- throughput(drain): 0.12 MB/s
- queue: depth=256, avg=255.75, max=256
- drop: 9988930 chunks / 9743.98 MB
- preview:
```text
(empty)
```

### tab-3
- command: `yes tab-3`
- throughput(read): 32.61 MB/s
- throughput(drain): 0.12 MB/s
- queue: depth=256, avg=255.75, max=256
- drop: 9989969 chunks / 9744.96 MB
- preview:
```text
(empty)
```

### tab-4
- command: `yes tab-4`
- throughput(read): 32.59 MB/s
- throughput(drain): 0.12 MB/s
- queue: depth=256, avg=255.75, max=256
- drop: 9983668 chunks / 9738.61 MB
- preview:
```text
(empty)
```

## Notes

- 隐藏 tab 策略 = bounded queue + 1Hz drain + drop-oldest，不允许无界积压。
