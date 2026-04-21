---
id: SPIKE-05.5
type: spike
title: PTY visible throughput + per-session fallback 对照
status: done
owner: Codex CLI
phase: W0-D5.5
depends_on: ["SPIKE-05"]
blocks: ["SPIKE-06"]
estimate: 1d
plan_ref: implementation-plan.md §附录 A D5 · §3.1
risk_ref:
reviewer: Claude Code
---

# SPIKE-05.5: PTY visible throughput + per-session fallback 对照

> **状态**：`done`（2026-04-19 · shared-reader vs per-session 对照完成 · 结论：visible throughput 瓶颈不在 shared-reader）
> **依赖**：SPIKE-05（shared-reader HOL / boundedness 已有基线） / **阻塞**：SPIKE-06（CLI 实机依赖可接受的可见吞吐）
> **报告**：[`docs/spikes/SPIKE-05.5-report.md`](../spikes/SPIKE-05.5-report.md)
> **战略依据**：[`implementation-plan.md §附录 A D5`](../implementation-plan.md)

## 📌 执行结论（本 Spike 实测结果）

- ✅ **shared-reader 不是 visible throughput 瓶颈**：per-session 在 4 Tab 场景把 read-path p50 从 **43.48 → 61.47 MB/s** 拉高，但 UI drain p50 没有更好（反而 **14.58 → 12.86 MB/s**）。
- ✅ **瓶颈已定位到 invoke / JS / xterm 链路**：两种 reader 策略在 4 Tab 下的 invoke latency p50 都约 **22ms**，远高于 4ms polling cadence。
- ✅ **per-session 对照版已实现并跑完 3 次**：单 Tab / 4 Tab / synthetic TUI 全量 raw 已归档。
- ✅ **建议翻 A**：锁定 `portable-pty + 共享读线程 + bounded queue + drop-oldest`；不要把 per-session 作为默认降级。

---

## 🎯 目标（Goal）

验证 **visible throughput** 的瓶颈到底在 shared-reader 还是 Rust→JS / xterm drain 链路；若 shared-reader 本身不是瓶颈，则给出可锁定的 fallback / 优化建议。

## 📖 背景（Context）

- SPIKE-05 已证明 shared-reader + bounded queue 在 **HOL / 内存有界** 上表现良好。
- 但 SPIKE-05 的 **A 可见吞吐** 失败：单 Tab UI drain ~8.34 MB/s、4 Tab 总 ~16.38 MB/s，未达到 spec 的 20 / 40 MB/s 门槛。
- 当前仍然无法锁定 `CLAUDE.md` 决策表 #15，因此 SPIKE-06 继续被阻塞。

## ✅ 通过标准（Pass Criteria）

- [ ] 在 **shared-reader 当前实现** 上，定位 visible throughput 的主瓶颈（reader / IPC / xterm / polling cadence / batching）
- [ ] 实现 **per-session reader** 对照版本（或等价严格隔离版本）
- [ ] 对照跑 A 单 Tab / A 4 Tab 各 3 次，输出 shared-reader vs per-session 的 drain throughput 表
- [ ] 若 per-session 版本 UI drain ≥ 20 / 40 MB/s，则给出明确 fallback 建议
- [ ] 若两者都 < 20 / 40 MB/s，则证明瓶颈不在 shared-reader，并提出下一步 IPC / batching 优化路径

## ❌ 失败信号（Fail Signals）

- 对照实验只给 read-path，不给 visible drain-path
- 只改 polling cadence / queue 容量，但没有 shared-reader vs per-session A/B 对照
- 仍无法回答“为何 A 吞吐失败”

## 🔀 Fallback 方案

**shared-reader visible throughput 达标** → 可重新评估把 `CLAUDE.md` #15 从 B → A
**仅 per-session 达标** → 推荐 fallback 到 per-session
**两者都不达标** → 说明瓶颈在 IPC / xterm / batching，继续留在 B 档并开实现优化 task

## 📦 产出（Deliverables）

- [ ] `docs/spikes/SPIKE-05.5-report.md`
- [ ] `docs/spikes/code/SPIKE-05.5/`
- [ ] `docs/spikes/raw/SPIKE-05.5/`
- [ ] `spike-tmp/archive/SPIKE-05.5/`
- [ ] ADR-003 更新（仅当结论足以锁定）

## 🛠 依赖资源（Resources Needed）

- SPIKE-05 现有 harness（`spike-tmp/spike-05-pty/`）
- macOS Apple Silicon 测试机

## ⚠️ 已知风险

- per-session reader 可能把 HOL 问题彻底消除，但 visible throughput 仍然卡在 xterm / invoke / JS batching
- shared-reader 与 per-session 两套实现并存时，注意不要把 queue 策略 / poll cadence 混淆成错误结论

## 📝 Notes / 讨论

- SPIKE-05 当前更像“隔离性 PASS + 可见吞吐 FAIL”的部分结论；SPIKE-05.5 的价值在于把“为什么 FAIL”解释清楚。

## 🔗 相关

- 前置 Spike：[`SPIKE-05`](./SPIKE-05-pty-multi-tab.md)
- 对应 ADR：[`ADR-003`](../adr/ADR-003-pty-architecture.md)
