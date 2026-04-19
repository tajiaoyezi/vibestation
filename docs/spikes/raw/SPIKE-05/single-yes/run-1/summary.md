# A · 单 Tab yes 10s

- 持续时间：10.0s
- 总吞吐(read path)：94.20 MB/s
- UI 吞吐(drain path)：8.34 MB/s
- 总读取：942.00 MB
- 总 drain：83.43 MB
- 总 drop：858.33 MB
- 主线程最大 lag：7.00ms
- freeze (>100ms)：0
- 峰值 RSS：120160 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 94.20 MB/s
- throughput(drain): 8.34 MB/s
- queue: depth=256, avg=250.20, max=256
- drop: 888648 chunks / 858.33 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-1
```

## Notes

- 单 Tab 高吞吐基线。
