# SUMMARY · SPIKE-05

## 总体结论

**共享读线程 + bounded queue + drop-oldest 在 HOL / 内存有界性上表现良好，但当前 UI drain 吞吐未达到 spec 的 A 阈值。**

- ✅ B.1 / B.2 / B.3 / B.4.1 / B.4.2 / B.4.3 / C（大体）拿到有效证据。
- ❌ A 的**可见吞吐**未达标：单 Tab UI drain 中位 **8.34 MB/s < 20**；4 Tab UI drain 总中位 **16.38 MB/s < 40**。
- ✅ 但 backend read-path 吞吐充足：单 Tab中位 **94.20 MB/s**；4 Tab 总中位 **127.30 MB/s**。
- ✅ HOL 没有复现：B.4.1 其它 tab 相对基线仅 **-3.5%**；B.4.2 / B.4.3 其它 tab 反而高于基线。
- ✅ Soak / hidden-tab RSS 增长都 < 1 MB，远低于 spec 限额。

## 对决策表 #15 的建议

**不要把 `CLAUDE.md` #15 从 B 翻到 A。**

原因不是 HOL，而是 **visible throughput 不达标**：当前 shared-reader + Tauri command pull + xterm 写入链路在 macOS 上只能稳定交付 ~8–16 MB/s 的 UI drain 吞吐。现有数据不足以锁定最终 PTY 架构。

## 建议后续

1. 新开 **SPIKE-05.5**：对比 shared-reader vs per-session reader；重点看 UI drain 吞吐而不是 read-path。
2. 试验更激进的前后端 batching / binary IPC / xterm write coalescing。
3. 若 per-session reader 能显著提升可见吞吐且仍不过 40 MB/s，再考虑直接接受“shared reader 读路径 + per-session 前端 drain 线程”的折中实现。
