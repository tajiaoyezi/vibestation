# SUMMARY · SPIKE-05.5

## 结论

**visible throughput 的瓶颈不在 shared-reader。**

- 单 Tab：shared drain p50 **7.79 MB/s** vs per-session **8.42 MB/s**（仅 +0.63 MB/s）。
- 4 Tab：shared drain p50 **14.58 MB/s** vs per-session **12.86 MB/s**（反而 -1.72 MB/s）。
- 但 read-path：4 Tab shared **43.48 MB/s** vs per-session **61.47 MB/s**，说明 per-session 的确改善了 reader 侧读取能力；**UI drain 却没有同步提升**。
- 4 Tab invoke latency p50 在两种策略都约 **22–22ms**，远高于 4ms polling cadence；这说明瓶颈落在 **Tauri invoke / JS drain / xterm 链路**，而不是 shared-reader。

## 建议

1. **接受 ADR-003 / CLAUDE.md #15**：锁定 `portable-pty + 共享读线程 + bounded queue + drop-oldest`。
2. **不要降级到 per-session**：它提高了 read-path，但没有解决 visible throughput。
3. 后续 visible throughput 优化应转向：
   - Rust→JS batching / 更粗粒度 drain
   - 降低 invoke 往返频率
   - xterm write coalescing / renderer 策略
