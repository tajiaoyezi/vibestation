# B.1 · 4 Tab 慢消费者 soak 10 min

- 持续时间：600.0s
- 总吞吐(read path)：124.11 MB/s
- UI 吞吐(drain path)：16.57 MB/s
- 总读取：74464.26 MB
- 总 drain：9943.66 MB
- 总 drop：64519.60 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：120704 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 31.03 MB/s
- throughput(drain): 4.14 MB/s
- queue: depth=256, avg=247.33, max=256
- drop: 16539598 chunks / 16129.45 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-
```

### tab-2
- command: `yes tab-2`
- throughput(read): 31.06 MB/s
- throughput(drain): 4.14 MB/s
- queue: depth=256, avg=247.34, max=256
- drop: 16563292 chunks / 16152.46 MB
- preview:
```text
tab-2
tab-2
tab-2
ta
```

### tab-3
- command: `yes tab-3`
- throughput(read): 31.01 MB/s
- throughput(drain): 4.14 MB/s
- queue: depth=256, avg=247.33, max=256
- drop: 16530174 chunks / 16120.08 MB
- preview:
```text
tab-3
tab-3
tab-3
tab-
```

### tab-4
- command: `yes tab-4`
- throughput(read): 31.01 MB/s
- throughput(drain): 4.14 MB/s
- queue: depth=256, avg=247.33, max=256
- drop: 16527344 chunks / 16117.61 MB
- preview:
```text
tab-4
tab-4
tab-4
tab-
```

## Notes

- 所有 tab 渲染前人工延迟 50ms，验证 bounded queue + drop-oldest。
