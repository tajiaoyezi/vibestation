# A · 单 Tab yes 10s

- 持续时间：10.0s
- 总吞吐(read path)：100.84 MB/s
- UI 吞吐(drain path)：9.01 MB/s
- 总读取：1008.43 MB
- 总 drain：90.10 MB
- 总 drop：918.08 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：117568 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 100.84 MB/s
- throughput(drain): 9.01 MB/s
- queue: depth=256, avg=250.20, max=256
- drop: 950842 chunks / 918.08 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-
```

## Notes

- 单 Tab 高吞吐基线。
