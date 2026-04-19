# B.4.1 · 前端 render 慢 3 min

- 持续时间：180.0s
- 总吞吐(read path)：117.73 MB/s
- UI 吞吐(drain path)：15.82 MB/s
- 总读取：21190.52 MB
- 总 drain：2846.78 MB
- 总 drop：18342.74 MB
- 主线程最大 lag：8.00ms
- freeze (>100ms)：0
- 峰值 RSS：120848 KB

## Sessions

### tab-1
- command: `yes tab-1`
- throughput(read): 29.43 MB/s
- throughput(drain): 3.95 MB/s
- queue: depth=256, avg=247.13, max=256
- drop: 4702686 chunks / 4584.97 MB
- preview:
```text
tab-1
tab-1
tab-1
```

### tab-2
- command: `yes tab-2`
- throughput(read): 29.45 MB/s
- throughput(drain): 3.95 MB/s
- queue: depth=256, avg=247.15, max=256
- drop: 4707391 chunks / 4589.44 MB
- preview:
```text
tab-2
tab-2
tab-2
tab-
```

### tab-3
- command: `yes tab-3`
- throughput(read): 29.40 MB/s
- throughput(drain): 3.96 MB/s
- queue: depth=256, avg=247.13, max=256
- drop: 4697658 chunks / 4580.11 MB
- preview:
```text
tab-3
tab-3
tab-3
```

### tab-4
- command: `yes tab-4`
- throughput(read): 29.44 MB/s
- throughput(drain): 3.95 MB/s
- queue: depth=256, avg=247.13, max=256
- drop: 4706202 chunks / 4588.22 MB
- preview:
```text
tab-4
tab-4
tab-4
tab-
```

## Notes

- Tab 1 人工延迟 500ms，验证共享读线程不 HOL 阻塞其他 tab。
