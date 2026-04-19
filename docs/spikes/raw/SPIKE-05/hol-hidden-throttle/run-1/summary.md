# B.4.3 · hidden-tab throttle 3 min

- 持续时间：180.0s
- 总吞吐(read path)：126.18 MB/s
- UI 吞吐(drain path)：14.94 MB/s
- 总读取：22711.55 MB
- 总 drain：2689.74 MB
- 总 drop：20020.81 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：121360 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 31.55 MB/s
- throughput(drain): 0.12 MB/s
- queue: depth=256, avg=255.75, max=256
- drop: 5801578 chunks / 5657.44 MB
- preview:
```text
(empty)
```

### tab-2
- command: `yes tab-2`
- throughput(read): 31.60 MB/s
- throughput(drain): 4.94 MB/s
- queue: depth=256, avg=245.85, max=256
- drop: 4919942 chunks / 4797.69 MB
- preview:
```text
tab-2
tab-2
tab-2
```

### tab-3
- command: `yes tab-3`
- throughput(read): 31.51 MB/s
- throughput(drain): 4.94 MB/s
- queue: depth=256, avg=245.82, max=256
- drop: 4903428 chunks / 4781.51 MB
- preview:
```text
tab-3
tab-3
tab-3
ta
```

### tab-4
- command: `yes tab-4`
- throughput(read): 31.52 MB/s
- throughput(drain): 4.94 MB/s
- queue: depth=256, avg=245.82, max=256
- drop: 4906075 chunks / 4784.17 MB
- preview:
```text
tab-4
tab-4
tab-4
tab-4
```

## Notes

- Tab 1 模拟 hidden → 1Hz drain，其余 tab 走正常 60fps cadence。
