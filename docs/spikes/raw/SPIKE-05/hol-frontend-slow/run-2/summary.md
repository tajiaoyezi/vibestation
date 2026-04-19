# B.4.1 · 前端 render 慢 3 min

- 持续时间：180.0s
- 总吞吐(read path)：119.08 MB/s
- UI 吞吐(drain path)：16.12 MB/s
- 总读取：21434.10 MB
- 总 drain：2902.22 MB
- 总 drop：18530.88 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：120800 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 29.74 MB/s
- throughput(drain): 4.03 MB/s
- queue: depth=256, avg=247.14, max=256
- drop: 4746573 chunks / 4627.77 MB
- preview:
```text
tab-1
tab-1
tab-1
tab-
```

### tab-2
- command: `yes tab-2`
- throughput(read): 29.79 MB/s
- throughput(drain): 4.03 MB/s
- queue: depth=256, avg=247.15, max=256
- drop: 4756025 chunks / 4636.71 MB
- preview:
```text
tab-2
tab-2
tab-2
ta
```

### tab-3
- command: `yes tab-3`
- throughput(read): 29.77 MB/s
- throughput(drain): 4.03 MB/s
- queue: depth=256, avg=247.16, max=256
- drop: 4751645 chunks / 4632.08 MB
- preview:
```text
tab-3
tab-3
tab-3
tab-
```

### tab-4
- command: `yes tab-4`
- throughput(read): 29.78 MB/s
- throughput(drain): 4.03 MB/s
- queue: depth=256, avg=247.15, max=256
- drop: 4753578 chunks / 4634.32 MB
- preview:
```text
tab-4
tab-4
tab-4
tab-
```

## Notes

- Tab 1 人工延迟 500ms，验证共享读线程不 HOL 阻塞其他 tab。
