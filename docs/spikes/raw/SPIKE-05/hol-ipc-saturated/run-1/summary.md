# B.4.2 · IPC queue 满 3 min

- 持续时间：180.0s
- 总吞吐(read path)：124.47 MB/s
- UI 吞吐(drain path)：15.39 MB/s
- 总读取：22405.49 MB
- 总 drain：2769.57 MB
- 总 drop：19634.92 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：123472 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 31.11 MB/s
- throughput(drain): 0.00 MB/s
- queue: depth=256, avg=255.99, max=256
- drop: 5742283 chunks / 5599.34 MB
- preview:
```text
(empty)
```

### tab-2
- command: `yes tab-2`
- throughput(read): 31.12 MB/s
- throughput(drain): 5.13 MB/s
- queue: depth=256, avg=245.28, max=256
- drop: 4796929 chunks / 4678.08 MB
- preview:
```text
tab-2
tab-2
tab-2
```

### tab-3
- command: `yes tab-3`
- throughput(read): 31.12 MB/s
- throughput(drain): 5.13 MB/s
- queue: depth=256, avg=245.28, max=256
- drop: 4797412 chunks / 4678.06 MB
- preview:
```text
tab-3
tab-3
tab-3
ta
```

### tab-4
- command: `yes tab-4`
- throughput(read): 31.13 MB/s
- throughput(drain): 5.13 MB/s
- queue: depth=256, avg=245.28, max=256
- drop: 4798685 chunks / 4679.44 MB
- preview:
```text
tab-4
tab-4
tab-4
```

## Notes

- Tab 1 完全停止 drain，逼满 bounded queue；其余 tab 必须持续推进。
