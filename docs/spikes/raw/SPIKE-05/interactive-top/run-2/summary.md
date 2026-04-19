# A · 交互 TUI（top）10s

- 持续时间：10.0s
- 总吞吐(read path)：0.00 MB/s
- UI 吞吐(drain path)：0.00 MB/s
- 总读取：19.87 KB
- 总 drain：19.87 KB
- 总 drop：0 B
- 主线程最大 lag：7.00ms
- freeze (>100ms)：0
- 峰值 RSS：119056 KB

## Sessions

### top
- command: `while true; do clear; date '+tick %H:%M:%S'; ps -A -o pid,pcpu,pmem,comm | head -n 12; sleep 0.2; done`
- throughput(read): 0.00 MB/s
- throughput(drain): 0.00 MB/s
- queue: depth=0, avg=0.74, max=3
- drop: 0 chunks / 0 B
- preview:
```text
(empty)
```

## Notes

- 宿主机未安装 htop，使用 macOS 原生 top -s 0.2 等价验证连续刷新。
