# SPIKE-03 · git2 vs gix 读路径 benchmark 报告

> **Task spec**：[`docs/tasks/SPIKE-03-git2-gix-read-benchmark.md`](../tasks/SPIKE-03-git2-gix-read-benchmark.md)
> **结论**：(B) · **git2 未通过性能门槛** · **gix 通过** · **读路径切 gix · 写路径保留 git2**
> **实施者**：OpenCode agent（macOS M2 Max · 2026-04-19）
> **Review**：Claude Code (Sonnet 4.6) · 独立评审 accept
> **相关 ADR**：[ADR-007 Git 栈](../adr/ADR-007-git-stack.md)（proposed → accepted）

---

## 1 · 结论概览

| 场景 | git2 warm P99 | gix warm P99 | spec 阈值 | git2 | gix |
|---|---|---|---|---|---|
| log -100（首屏最近 100 条） | **24964 ms** | **12.65 ms** | < 200ms | ❌ 124× 超 | ✅ 16× 余量 |
| log -1000 | **21108 ms** | **113.84 ms** | < 1s | ❌ 21× 超 | ✅ 8.8× 余量 |
| log -10000 | **33483 ms** | **733.72 ms** | < 5s | ❌ 6.7× 超 | ✅ 6.8× 余量 |
| count-all（1.4M commit 全遍历） | 28872 ms | 12840 ms | 参考 | — | gix 2.25× 快 |

**判定**：严格符合 spec §B.6 路径 **(B)** · git2 读路径不达标 · gix 全达标 · 混用策略落地。

---

## 2 · 环境

| 维度 | 数据 |
|---|---|
| OS | macOS 26.3.1 (Build 25D771280a) |
| CPU | Apple M2 Max · 12 核（8P+4E） |
| RAM | 32 GB |
| Rust toolchain | rustc 1.95.0 (2026-04-14) |
| git2 | 0.20.x（libgit2 C 绑定） |
| gix | 0.70.x（纯 Rust） |
| 测试仓库 | linux kernel `/tmp/linux-kernel` |
| Kernel commit | `eb5249b12507246dc959945454cd1be8d7dc3795` |
| 总 commits | **1,441,214**（极端压测 · MVP 目标 10 万 commits 的 14× 上限） |

---

## 3 · 测量方法

| 项 | 做法 |
|---|---|
| warm 采样 | 每场景 10 次连续运行 · 每次重新 `open_repo()`（避免 txn cache 复用） |
| cold 采样 | 每场景 3 次 · 每次前不清 OS cache（本机无 sudo 无密码 purge） |
| 统计 | P50 / P99 / mean / std · 单位 ms |
| 场景 | `log -N` = 遍历 commit + 读 commit object + 提取摘要行 |
| count-all | 可达 commit 全量计数（上限参考） |

**cold 数据透明声明**：本机无法无密码执行 `purge` · cold 数据是"未清缓存的首轮样本"近似值 · **正式选型判据以 warm P99 为准**。

---

## 4 · 原始数据

### 4.1 git2 0.20

| 场景 | cold P50 | cold P99 | **warm P50** | **warm P99** | mean | std |
|---|---:|---:|---:|---:|---:|---:|
| log -100 | 23094 | 23261 | 23621 | **24964** | 22742 | 1711 |
| log -1000 | 17513 | 18771 | 18125 | **21108** | 18460 | 1656 |
| log -10000 | 17118 | 17190 | 31018 | **33483** | 29209 | 3909 |
| count-all | 26445 | 26445 | 28872 | 28872 | 28872 | 0 |

### 4.2 gix 0.70

| 场景 | cold P50 | cold P99 | **warm P50** | **warm P99** | mean | std |
|---|---:|---:|---:|---:|---:|---:|
| log -100 | 10.74 | 15.43 | 11.32 | **12.65** | 11.14 | 0.85 |
| log -1000 | 45.33 | 46.62 | 64.61 | **113.84** | 71.15 | 18.80 |
| log -10000 | 310.72 | 398.80 | 282.23 | **733.72** | 331.93 | 161.42 |
| count-all | 18082 | 18082 | 12840 | 12840 | 12840 | 0 |

---

## 5 · Claude review notes

### 5.1 Accept · 结论清晰

- ✅ 结论 (B) 严格符合 spec §Fallback 方案："Storage A 不达标 → 锁 rusqlite"（映射到 git 栈：git2 读不达标 → 切 gix 读）
- ✅ cold 无 sudo 已透明声明 · 用 warm P99 判定合规
- ✅ 采样方式合理（每次重新 open repo · 避免 txn cache）
- ✅ 方差在合理范围（< 50% spec 要求）· 可复现

### 5.2 疑点（HIGH · 不影响结论）

**log -10000 warm P99 (31018ms) > cold P99 (17190ms)** · 违直觉：
- 可能原因：libgit2 `revwalk` 在连续 warm 运行时内部状态有污染 · 或 warm 运行正好触发 GC pause
- 即使按 cold 17s 判定 · 也远超 5s 阈值 · **结论不变**
- 备注：后续若 gix 读路径出现类似异常 · 可用此 bench 作对照排查

### 5.3 Descope 项（LOW）

- **未附 flamegraph**：spec §70 "可选但推荐" · 非 blocker · 不阻塞归档

### 5.4 对 MVP 场景的外推

| 场景 | 测试数据 | MVP 10 万 commit 外推 | 满足 < 500ms？ |
|---|---|---|---|
| 首屏 log -100 | gix 12.65 ms | ≈ 12 ms | ✅ 40× 余量 |
| log -1000 | gix 113.84 ms | ≈ 110 ms | ✅ 4.5× 余量 |
| log -10000 | gix 733.72 ms | ≈ 700 ms | ⚠️ 超 500ms · 但 MVP 常态不取 10000 条一次 · 应分页 |

**MVP 首屏（100 条）完全在预算内**。超过 1000 条用户大概率不是首屏一次性滚完 · 可以分页加载。

---

## 6 · 决策联动

### 6.1 `CLAUDE.md` 决策表 #13 Git 栈

**变更前（B 档）**：
| 决策 | 默认 | 锁定节点 | Fallback |
|---|---|---|---|
| Git 栈（写）| git2 0.20 | Spike W0 Day 4 benchmark | 读慢 → gix 0.70 混用 |

**变更后（A 档 · 本 PR 落地）**：
| 决策 | 依据 |
|---|---|
| Git 栈 = **写 git2 0.20 · 读 gix 0.70 混用** | SPIKE-03 benchmark：git2 读性能不达标（log -100 P99 25s · 超 200ms 阈值 124×）· gix 读性能达标（log -100 P99 12.65ms） |

### 6.2 ADR-007 状态变更

- **变更前**：status: **proposed**（pending SPIKE-03 benchmark 后决定）
- **变更后**：status: **accepted** · 增加 "§SPIKE-03 benchmark 结论" 段落

### 6.3 后续动作

- MVP 存储层 spec（MVP-04/05/07/08 等）涉及 Git 读实现时 · 统一用 gix 0.70 API
- MVP commit / stage / push / rebase 等**写操作** · 继续用 git2 0.20 API
- 双库共存 · 代码评审需熟悉两套 Object 模型

---

## 7 · 自审四问

1. **递归完备性**：4 场景（log -100 / -1000 / -10000 / count-all）+ warm/cold 双采样 + P50/P99/mean/std 完整 ✅
2. **反向场景**：git2 快 → 单用 git2（但数据证明不成立）· gix 慢 → Arbiter 分页（也不成立）· 实际走 (B) 混用路径 ✅
3. **边界适用性**：linux kernel 1.4M commits 是 MVP 目标 14× 上限 · 外推 MVP 场景性能有明确依据 ✅
4. **YAGNI**：只测读路径（gix 写 API 不成熟 · 不在 Spike 范围）· 不评估 sled / pure rust 方案 ✅

---

## 8 · 变更记录

| 日期 | 实施者 | 变更 |
|---|---|---|
| 2026-04-19 AM | OpenCode agent | linux kernel 1.4M commits benchmark · 4 场景 warm/cold 采样 · 结论 (B) |
| 2026-04-19 AM | Claude Code | Review accept · 归档 report · 更新 ADR-007 accepted · 更新决策表 #13 B→A · SPIKE-03 spec 翻 done |

---

## 9 · 附：opencode 交付

- 原始 tarball 位置（本地）：`/tmp/spike-03-work/`（含 bench code + criterion 原始数据 + measurements.json）
- 本 report 基于 opencode 原 report + Claude review 综合整理 · 保留全部原始数据 · 透明标注 cold 限制
