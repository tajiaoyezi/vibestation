# B.4.1 · 前端 render 慢 3 min

- 持续时间：180.0s
- 总吞吐(read path)：128.55 MB/s
- UI 吞吐(drain path)：15.39 MB/s
- 总读取：23138.12 MB
- 总 drain：2769.56 MB
- 总 drop：20367.56 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：122368 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 32.06 MB/s
- throughput(drain): 3.85 MB/s
- queue: depth=256, avg=248.25, max=256
- drop: 5205607 chunks / 5077.58 MB
- preview:
```text
tab-1
tab-1
tab-1
```

### tab-2
- command: `yes tab-2`
- throughput(read): 32.17 MB/s
- throughput(drain): 3.85 MB/s
- queue: depth=256, avg=248.27, max=256
- drop: 5227002 chunks / 5098.32 MB
- preview:
```text
tab-2
tab-2
tab-2
```

### tab-3
- command: `yes tab-3`
- throughput(read): 32.18 MB/s
- throughput(drain): 3.85 MB/s
- queue: depth=256, avg=248.27, max=256
- drop: 5229425 chunks / 5100.62 MB
- preview:
```text
tab-3
tab-3
tab-3
```

### tab-4
- command: `yes tab-4`
- throughput(read): 32.13 MB/s
- throughput(drain): 3.85 MB/s
- queue: depth=256, avg=248.26, max=256
- drop: 5219602 chunks / 5091.03 MB
- preview:
```text
tab-4
tab-4
tab-4
```

## Notes

- Tab 1 人工延迟 500ms，验证共享读线程不 HOL 阻塞其他 tab。
