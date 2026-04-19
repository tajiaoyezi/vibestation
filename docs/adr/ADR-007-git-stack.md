# ADR-007: Git 栈 = git2 0.20（写）· gix 0.70（读）· accepted

**状态**：**accepted**（2026-04-19 · SPIKE-03 benchmark 通过）
**日期**：2026-04-18 初版 proposed · 2026-04-19 accepted
**决策者**：项目发起人（Arbiter）· OpenCode agent 实测 · Claude Code review
**对应 `CLAUDE.md` 决策表**：#13（已从 B 档升级到 A 档）
**对应 Spike**：[SPIKE-03 · 已 done](../tasks/SPIKE-03-git2-gix-read-benchmark.md)
**对应 Report**：[SPIKE-03-report](../spikes/SPIKE-03-report.md)

---

## 背景与问题

Vibestation Git 工作台需要完整 Git 操作：
- **读**：log / status / diff · 10 万 commit 仓库首屏 < 500ms（`implementation-plan.md §10.2`）
- **写**：add / commit / stage / unstage · v0.2+ push/pull/fetch/rebase/merge

Rust 两个主流 Git 库：
- **git2**：libgit2 C 绑定 · 成熟 · API 完整
- **gix**：纯 Rust 实现 · 近几年迅速成熟 · 读性能据称比 libgit2 快 2-5 倍

选哪个 · 或混用？

## 决策驱动因素

- **D1 · 读性能**：10 万 commit log 首屏时间 · R3 风险核心（`implementation-plan.md §9 R3`）
- **D2 · 写 API 完整性**：commit / stage / push / rebase 需要稳定实现
- **D3 · 跨编译**：git2 依赖 libgit2 (C) · gix 纯 Rust · Windows / 交叉编译复杂度
- **D4 · 维护成本**：混用两库增加 Git 对象模型理解成本

## 考虑的选项

- **A · 仅 git2**：简单 · 成熟 · 读性能中等（10 万 commit 可能超 500ms）
- **B · 仅 gix**：纯 Rust · 读优 · 写 API 部分功能未稳定（push / rebase）
- **C · 混用 git2 写 + gix 读**：读优 + 写稳 · 代码分两套心智模型
- **D · 仅 libgit2 via git2 + 手写索引**：复杂度爆炸 · 不考虑

## 决策

**选择（accepted · SPIKE-03 benchmark 2026-04-19 已落地）**：
- **写路径**：`git2 0.20`（commit / stage / add / push / rebase）· 保留
- **读路径**：`gix 0.70` **混用**（log / walk / object read · 满足 MVP < 500ms 性能目标）

### SPIKE-03 benchmark 结论（linux kernel 1,441,214 commits · warm P99）

| 场景 | git2 | gix | spec 阈值 | git2 | gix |
|---|---:|---:|---|---|---|
| log -100（首屏） | 24964 ms | **12.65 ms** | < 200ms | ❌ 124× 超 | ✅ 16× 余量 |
| log -1000 | 21108 ms | **113.84 ms** | < 1s | ❌ 21× 超 | ✅ 8.8× 余量 |
| log -10000 | 33483 ms | **733.72 ms** | < 5s | ❌ 6.7× 超 | ✅ 6.8× 余量 |

详细数据见 [SPIKE-03-report §4](../spikes/SPIKE-03-report.md)。

**gix 全场景通过 · git2 全场景不通过**——触发 spec §路径 (B)：读路径切 gix · 写路径保留 git2。

## 后果

### 正面

- **写路径稳定**：git2（libgit2 15+ 年）· commit / rebase 边界情况少坑
- **读路径可优化**：若 git2 不够快 · gix 是明确升级路径
- **纯 Rust 选项**：gix 无 C 依赖 · 若 Phase 5 改为纯 gix · Windows 交叉编译省很多事

### 负面

- **混用复杂度**：两套 Object 模型 · PR code review 需要双侧审
- **gix 写 API 不完整**：push / rebase 等写操作 gix 支持不稳定 · 不能"全切 gix"
- **libgit2 依赖体积**：git2 crate 编译后 + libgit2 约 2-3 MB · 可接受

### 风险

- **SPIKE-03 benchmark 翻车**：10 万 commit 仓库 git2 + gix 都不够快 → Arbiter 讨论分页策略
- **gix 0.x API 不稳定**：gix 仍在 0.x 版本 · breaking change 频率中等 · 若混用 · 升级时两库互相约束
- **R3 真实**：10 万 commit 仓库实际场景（linux kernel 真实用户有）· 不能凭经验估算 · 必过 SPIKE-03

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.2（Git 栈）· §附录 A D3（SPIKE-03 计划）
- 对应风险：**R3**（git2 读性能 · HIGH · 本 ADR 前置缓解）

## 相关

- `CLAUDE.md` 决策表：#13
- Spike：[SPIKE-03 git2 读 log + gix 对比 benchmark（linux kernel）](../tasks/SPIKE-03-git2-gix-read-benchmark.md)
- 相关 ADR：ADR-008（Diff 自建 · 与 Git 栈读路径配合）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code · status: proposed · SPIKE-03 benchmark 后决定单用 git2 / 混用
- 2026-04-19 · SPIKE-03 benchmark 落地 · OpenCode agent 实测 + Claude Code review · status: proposed → **accepted** · 结论：写 git2 · 读 gix 混用（spec §路径 (B)）
