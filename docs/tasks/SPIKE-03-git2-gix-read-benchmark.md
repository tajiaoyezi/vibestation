---
id: SPIKE-03
type: spike
title: git2 读 log + gix 对比 benchmark（linux kernel）
status: done
owner: OpenCode agent
phase: W0-D3
depends_on: ["SPIKE-01"]
blocks: []
estimate: 1d
plan_ref: implementation-plan.md §附录 A D3 · §9 R3
risk_ref: R3
reviewer: User (Arbiter · GitHub PR approve)
---

# SPIKE-03: git2 读 log + gix 对比 benchmark

> **状态**：`done`（2026-04-19 · OpenCode agent 实测 · Claude Code review · User approve · 结论 (B) 读切 gix）
> **依赖**：SPIKE-01（Rust toolchain 可用 · 2026-04-19 用户决策放宽：原 `SPIKE-02` → `SPIKE-01` · 理由：bench 纯 CLI · 不依赖 Tauri 容器）
> **报告**：[`docs/spikes/SPIKE-03-report.md`](../spikes/SPIKE-03-report.md)
> **战略依据**：[`implementation-plan.md §附录 A D3`](../implementation-plan.md) · [`§9 R3`](../implementation-plan.md)

---

## 🎯 目标（Goal）

在 linux kernel 仓库（~100 万 commit）上 benchmark **git2 0.20 的 commit log 读性能**；同场景跑 gix 0.70；根据数据决定是否在 MVP 的**读路径**混用 gix。

## 📖 背景（Context）

- `CLAUDE.md` 决策表 **#13 Git 栈（写）= git2 0.20 默认**，**读路径是否混用 gix 未锁**
- `implementation-plan.md §2 · §62` 明确："读路径优先用 git2；若 Spike Day 3 benchmark 证明 gix 在大仓库上显著更快，再在读侧引入 gix"
- **验收门槛**：10 万 commit 仓库首屏 < 500ms（MVP 性能目标）
- **linux kernel 是极端场景**（~100 万 commit），实战里不会这么大，但作为上限压测

---

## ✅ 通过标准（Pass Criteria）

- [ ] linux kernel 仓库（`git clone --depth=0 linux-stable`，完整历史）本地就绪
- [ ] **git2 0.20 benchmark**：
  - [ ] `log --oneline -100`（最近 100 条）P99 < 200ms
  - [ ] `log --oneline -1000` P99 < 1s
  - [ ] `log --oneline -10000` P99 < 5s
  - [ ] 全量遍历 commit count 耗时（上限参考，不作为门槛）
- [ ] **gix 0.70 同场景 benchmark**（相同硬件、同次连续跑）
- [ ] **对比结论写明**（下面 3 种之一）：
  - (A) git2 满足 MVP 性能目标（10 万 commit < 500ms）→ MVP 纯 git2，不引入 gix
  - (B) git2 不满足但 gix 满足，且性价比合理（复杂度可接受）→ MVP 引入 gix 做读路径，git2 做写路径
  - (C) 两者都不满足 → 升级为 R3 触发，需要分页加载 + 背景索引策略（扩展 Spike 到 Day 4 半天）
- [ ] 结论写入 **ADR-007**（`docs/adr/ADR-007-git-stack.md` 已 proposed · Spike benchmark 后 proposed → accepted）
- [ ] 若结论是 B → `CLAUDE.md` 决策表 #13 更新（"读 + gix 0.70 混用"）

## ❌ 失败信号（Fail Signals）

- benchmark 无法复现（结果方差 > 50%）→ 调查硬件 / 缓存 / 热启动问题
- 10 万 commit 场景 P99 > 2s（git2 和 gix 都达不到）→ 触发分页策略设计（R3）

## 🔀 Fallback 方案

**通过 (A)** → MVP 锁定 git2，`CLAUDE.md` #13 "读 = git2"
**通过 (B)** → MVP 引入 gix，`CLAUDE.md` #13 更新 "写 git2 + 读 gix 混用"
**通过 (C)** → 分页加载策略（实时 100 条 + 背景补充）进入 MVP-0X（后续 PR 补 spec）

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-03-git-benchmark/`：benchmark 代码（criterion crate 或自写 bench）
- [ ] **`docs/spikes/SPIKE-03-report.md`** benchmark 数据表（per-task；git2 vs gix 同场景多次取 P50/P99）
- [ ] **ADR-007 草稿 → accepted**：Git 栈读路径决策（`docs/adr/ADR-007-git-stack.md`）
- [ ] `CLAUDE.md` 决策表 #13 状态 PR（如走 B 路径）
- [ ] 火焰图 × 2（git2 和 gix 各一张，便于未来优化参考）

## 🛠 依赖资源（Resources Needed）

- linux kernel 仓库完整 clone（约 4-5 GB）
- Rust `criterion` 0.5+ 或等价 bench harness
- 磁盘：至少 20GB 可用（仓库 + index + cache）
- 测试机：建议 Ubuntu 24（服务器负载更接近生产），至少 16GB RAM

## ⚠️ 已知风险

- **R3**（`implementation-plan.md §9`）：git2 大仓库 log 慢到不可用 —— 本 Spike 消除
- OS 缓存效应：benchmark 要跑 warm + cold 两套数据，避免被 fs cache 误导结论
- libgit2 版本与 gix 迭代速度不一致（gix 还在活跃演进）：锁定 benchmark 时用的具体版本号到 ADR

---

## 📝 Notes / 讨论

- gix 0.70 是 2025 年的活跃版本，读路径 API 已稳定；但写路径 gix 还不完整（所以写必须 git2）
- benchmark 时机：warm（第二次跑）+ cold（清 fs cache 重跑）都要测；MVP 实际场景偏 warm（用户重复打开同一个仓库）
- 如果结论是 B，需评估额外复杂度：两个 git 库同时依赖会增加 ~2MB bundle，可接受

## 🔗 相关

- ADR：`docs/adr/ADR-007-git-stack.md`
- 对应 `CLAUDE.md` 决策表：**#13 Git 栈**
- `implementation-plan.md` 章节：§附录 A D3 · §9 R3 · §3.1
- 上游：SPIKE-02
- 下游：无直接阻塞（D4 storage + D5 PTY 可并行）

---

**填写完毕后自审**：

1. **递归完备性**：3 种结论全覆盖 + fallback 明确 ✅
2. **反向场景**：两者都慢 → 分页策略；单独失败 → 混用方案 ✅
3. **边界适用性**：linux kernel 是极端压测，MVP 实际目标 10 万 commit ✅
4. **YAGNI**：不评估 gix 写路径（写已锁 git2），只测读 ✅
