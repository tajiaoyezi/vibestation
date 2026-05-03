# 01 · Warm hit 实测（A1a 验证）

> 测量日期：2026-04-30
> 环境：macOS 15.x · zsh + oh-my-zsh
> Vibestation：含 Phase A1+A2+A3+B+C 全栈
> 源码：`crates/core/tests/pty_pool_bench.rs::warm_hit_with_pool`

## 测量定义

测 PtyPool **take 调用 → 第一次匹配 tab_id 的 PtyEvent::Stdout** 时长 · 即 spec A1a 定义的 IPC→onData 的 backend 部分。

每次迭代：

1. 调 `pool.refill_async(shell, $HOME)` 触发 idle 预热
2. polling 等 `pool.idle_count() >= 1`（idle ready）
3. drain receiver（清空 idle 期间 omz greeting）
4. 起计时 → `pool.take(req)`（cwd=`/tmp` 触发 cd 注入）→ 等第一次匹配的 stdout → 停计时
5. kill + drain 准备下一迭代

测量包含：
- pool 内部 mutex lock
- session rename（PtySession.tab_id 字段切换）
- inject_cd_clear（写 `cd -- '/tmp'; clear\n` 到 PTY master）
- reader thread 读 PTY slave + emit PtyEvent::Stdout

不包含：
- 前端 IPC roundtrip（~30ms · 见 [00-baseline-cold-spawn.md](./00-baseline-cold-spawn.md) 对比）
- xterm 渲染像素到屏幕

## 原始数据（10 样本 · 已排序）

| # | 延迟 (ms) |
|---|-----------|
|  1 |  0.07 |
|  2 |  0.08 |
|  3 |  0.08 |
|  4 |  0.09 |
|  5 |  0.09 |
|  6 |  0.09 |
|  7 |  0.10 |
|  8 |  0.15 |
|  9 |  0.28 |
| 10 |  0.33 |

## 统计

| 指标 | 值 |
|---|---|
| min | 0.07 ms |
| **P50** | **0.09 ms** |
| mean | 0.14 ms |
| P90 | 0.28 ms |
| max | 0.33 ms |
| stddev | 0.09 ms |

## A1a 验证

| | 目标 | 实测 | 通过 |
|---|---|---|---|
| backend P50 | spec 隐含（≤ 200ms 含 IPC） | 0.09 ms | ✅ |
| 加 IPC overhead 后估算 | ≤ 200 ms | 0.09 + 30 ≈ **30 ms** | ✅ |
| backend max | n/a | 0.33 ms | ✅ |

**远超达标** · 提速 ~9000 倍（cold P50 = 796ms vs warm P50 = 0.09ms · backend 端）· **用户感知（含 IPC + xterm 渲染）估 ~30-50ms** · 比 cold spawn 800ms 快 **15-25 倍**。

## 关于 0.09ms 数据真实性

可能疑问：为什么这么快？

**根因**：idle PTY 在 `pool.refill_async` 后已经完成完整 spawn 流程（fork shell + 加载 .zshrc + omz init + 输出 prompt）· stdout 已经在 reader thread 的 mpsc 队列中（前面 `drain` 调用清掉了）。`take()` 调用的 cd 注入命令通过 PTY master fd 写入 · reader thread 读到后立即 emit · 几乎瞬时（macOS kqueue / Linux epoll fd readable 通知 < 1ms）。

**验证**：第一次 stdout 内容是 `cd -- '/tmp'; clear\n` 的 echo（zsh 默认开 ECHO termios 模式 · 用户输入 / 程序写入都会回显）。这字面意义符合 spec A1a 的 "onData first" · 用户实际感受是"瞬间出现命令 echo + 紧接着新 prompt"。

**spec 设计意图**：用户原始痛点是"等 omz 加载 1-2 秒"· pool 把 omz 加载移到 idle 阶段（用户点 + 之前已完成）· 用户视角时间从 1000ms 压到 30ms · **设计目标完全达成**。

## 跟 e2e 估算对照

backend warm hit P50 = 0.09 ms
+ Tauri IPC roundtrip ≈ 30 ms（基于 cold path frontend 830 - backend 796 = 34ms 的对照）
+ xterm onData callback dispatch ≈ < 1 ms
+ 用户视觉感知阈值 ≈ 100ms（人眼对 < 100ms 延迟感受为"瞬时"）

= **e2e ≈ 30-50 ms** ≪ A1b 目标 415 ms ✅
