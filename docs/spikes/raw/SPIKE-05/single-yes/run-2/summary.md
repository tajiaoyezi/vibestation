# A · 单 Tab yes 10s

- 持续时间：10.0s
- 总吞吐(read path)：84.35 MB/s
- UI 吞吐(drain path)：7.93 MB/s
- 总读取：843.47 MB
- 总 drain：79.25 MB
- 总 drop：763.97 MB
- 主线程最大 lag：7.00ms
- freeze (>100ms)：0
- 峰值 RSS：117456 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 84.35 MB/s
- throughput(drain): 7.93 MB/s
- queue: depth=256, avg=249.85, max=256
- drop: 790804 chunks / 763.97 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-1
```

## Notes

- 单 Tab 高吞吐基线。
