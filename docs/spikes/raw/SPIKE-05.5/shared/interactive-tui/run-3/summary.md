# A · synthetic TUI 10s

- strategy: **shared-reader**
- duration: 10.0s
- read-path throughput: **0.00 MB/s**
- UI drain throughput: **0.00 MB/s**
- total drop: **0 B**
- peak RSS: **116496 KB**
- reader threads: **1**

## Sessions

### tui
- throughput(read): 0.00 MB/s
- throughput(drain): 0.00 MB/s
- queue depth avg/max: 2.51 / 11
- drop: 0 chunks / 0 B
- read calls: 173 · avgReadBytes=117.69 · readSyscall=7.77µs · enqueue=2.32µs
- invoke latency p50/p99: 1.00 / 138.00 ms
- preview:
```text
<headless>
```

## Notes

- 宿主机无 htop，使用 5Hz synthetic TUI 替代。
