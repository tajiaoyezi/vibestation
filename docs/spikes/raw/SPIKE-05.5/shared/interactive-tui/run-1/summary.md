# A · synthetic TUI 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **0.00 MB/s**
- UI drain throughput: **0.00 MB/s**
- total drop: **0 B**
- peak RSS: **117264 KB**
- reader threads: **1**

## Sessions

### tui
- throughput(read): 0.00 MB/s
- throughput(drain): 0.00 MB/s
- queue depth avg/max: 2.16 / 10
- drop: 0 chunks / 0 B
- read calls: 151 · avgReadBytes=134.77 · readSyscall=6.85µs · enqueue=3.88µs
- invoke latency p50/p99: 1.00 / 346.00 ms
- preview:
```text
<headless>
```

## Notes

- 宿主机无 htop，使用 5Hz synthetic TUI 替代。
