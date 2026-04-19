# SPIKE-05 · portable-pty + xterm 多 Tab 压测报告

> **Task spec**：[`docs/tasks/SPIKE-05-pty-multi-tab.md`](../tasks/SPIKE-05-pty-multi-tab.md)
> **执行者**：Codex CLI（2026-04-19）
> **结论**：**HOL / boundedness PASS；visible throughput FAIL；ADR-003 暂不 accepted**
> **Follow-up**：[`SPIKE-05.5`](../tasks/SPIKE-05.5-pty-visible-throughput-fallback.md)
> **相关 ADR**：[ADR-003 PTY 架构](../adr/ADR-003-pty-architecture.md)

---

## 1 · 总结

本 Spike 的核心发现是：

1. **共享读线程没有复现 HOL**：B.4.1 / B.4.2 / B.4.3 都没有拖慢其它 tab。
2. **bounded queue + drop-oldest 把内存控制住了**：10 分钟 soak 与 hidden-tab 场景 RSS 增长都 < 1 MB。
3. **但 visible throughput 仍然不够**：单 Tab UI drain 中位 **8.34 MB/s**，4 Tab UI drain 总中位 **16.38 MB/s**，低于 spec 的 20 / 40 MB/s。

因此，当前证据不足以把 `CLAUDE.md` 决策表 #15 从 B → A。

## 2 · 关键数据

### 2.1 A · 短时压测

- 单 Tab `yes` × 3：backend read-path 中位 **94.20 MB/s**；UI drain-path 中位 **8.34 MB/s**
- 4 Tab `yes` × 3：backend read-path 总中位 **127.30 MB/s**；UI drain-path 总中位 **16.38 MB/s**
- 主线程 lag：所有 A 场景 **≤ 8ms**；freeze >100ms = **0**
- 交互 TUI 替代（宿主机无 `htop`）：`clear + date + ps` 5Hz 循环 × 3，**0 drop**

### 2.2 B.1 / B.2 · boundedness

- B.1 soak 10min：RSS **118064 KB → 119056 KB**（+992 KB），峰值 **120704 KB**
- B.2 hidden 5min：RSS **117056 KB → 117392 KB**（+336 KB），峰值 **118608 KB**
- 两个场景下各 session `maxQueueDepth` 都是 **256**（容量封顶）

### 2.3 B.4 · HOL

- A 4 Tab 基线：单 session UI drain 中位 **4.10 MB/s**
- B.4.1（3 次）：其它 tab UI drain 中位 **3.95 MB/s**（相对基线 **-3.5%**）
- B.4.2（1 次）：其它 tab UI drain 中位 **5.13 MB/s**（相对基线 **+25.1%**）
- B.4.3（1 次）：其它 tab UI drain 中位 **4.94 MB/s**（相对基线 **+20.6%**）

结论：**没有观测到 head-of-line blocking**。

### 2.4 C · correctness

- resize 结果：3 次中 **2 次** 明确读到 `40 100`
- fd delta：3 次全部 **0**
- 资源清理稳定；resize 采样仍有轻微波动

## 3 · 判定

| 模块 | 结果 | 说明 |
|---|---|---|
| A 可见吞吐 | ❌ | UI drain-path 未达到 20 / 40 MB/s |
| B.1 soak | ✅ | 队列有界，RSS 增长极小 |
| B.2 hidden tab | ✅ | hidden 策略明确且无内存堆积 |
| B.3 bounded queue | ✅ | 代码层硬门槛全部满足 |
| B.4 HOL | ✅ | 未复现 HOL |
| C correctness | ⚠️ | cleanup 稳定；resize 2/3 PASS |

## 4 · 决策建议

- **不要锁定 ADR-003 / 决策表 #15**
- 新开 [`SPIKE-05.5`](../tasks/SPIKE-05.5-pty-visible-throughput-fallback.md) 对比 shared-reader vs per-session reader
- 后续重点改为 **visible throughput**，而不是继续怀疑 HOL

## 5 · 原始数据入口

- 源码：[`docs/spikes/code/SPIKE-05/`](./code/SPIKE-05/)
- Raw：[`docs/spikes/raw/SPIKE-05/`](./raw/SPIKE-05/)
- 工作目录：`spike-tmp/spike-05-pty/`
