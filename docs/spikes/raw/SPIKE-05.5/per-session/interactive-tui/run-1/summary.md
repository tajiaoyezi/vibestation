# A · synthetic TUI 10s

- strategy: **per-session-reader**
- duration: 10.0s
- read-path throughput: **0.00 MB/s**
- UI drain throughput: **0.00 MB/s**
- total drop: **0 B**
- peak RSS: **116384 KB**
- reader threads: **1**

## Sessions

### tui
- throughput(read): 0.00 MB/s
- throughput(drain): 0.00 MB/s
- queue depth avg/max: 2.35 / 13
- drop: 0 chunks / 0 B
- read calls: 141 · avgReadBytes=127.01 · readSyscall=70216.66µs · enqueue=3.15µs
- invoke latency p50/p99: 1.00 / 339.00 ms
- preview:
```text
<headless>
```

## Notes

- 宿主机无 htop，使用 5Hz synthetic TUI 替代。
