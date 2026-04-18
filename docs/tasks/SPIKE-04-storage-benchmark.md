---
id: SPIKE-04
type: spike
title: redb 2 vs rusqlite benchmark + git2 写 commit 打通
status: draft
owner:
phase: W0-D4
depends_on: ["SPIKE-02"]
blocks: []
writes_to:
  - "spike-tmp/spike-04-storage-bench/"
  - "docs/tasks/SPIKE-04-*.md"
  - "spike-artifacts/SPIKE-04/"
reads_from:
  - "SPIKE-02:spike-tmp/spike-02-tauri/"
estimate: 1d
plan_ref: implementation-plan.md §附录 A D4 · §9 R27 · §3.2
risk_ref: R27
reviewer:
---

# SPIKE-04: redb 2 vs rusqlite benchmark + git2 写 commit

> **状态**：`draft`
> **依赖**：SPIKE-02（桌面框架已锁定）
> **战略依据**：[`implementation-plan.md §附录 A D4`](../implementation-plan.md) · [`§9 R27`](../implementation-plan.md) · [`§3.2`](../implementation-plan.md)

---

## 🎯 目标（Goal）

（1）在与 MVP 量级相当的数据规模上 benchmark **redb 2**（默认）和 **rusqlite**（fallback），决定本地存储选型。
（2）顺带完成 git2 **写路径**（`commit`）的最小闭环验证。

## 📖 背景（Context）

- `CLAUDE.md` 决策表 **#14 本地存储 = redb 2 默认**，Spike W0 D6 benchmark 后锁定（v2 提前到 D4）
- **数据规模估计**：10 workspace × 100 profile × 10,000 terminal 快照 = 1000 万条记录量级
- 同步验证 git2 写路径是 §附录 A D4 约定的附加任务（D5 之前必须确认能写成功）
- **R27**：redb 2 稳定性 / 性能不足以支撑 MVP 存储需求

---

## ✅ 通过标准（Pass Criteria）

### A. Storage Benchmark（主线）

- [ ] **数据集构造**：10 workspace × 100 profile × 10,000 snapshot（总 ~1000 万行）
- [ ] **redb 2 benchmark**：
  - [ ] 批量写入 1000 万行 P99 < 60s
  - [ ] 单键读取 P99 < 5ms
  - [ ] 范围查询（workspace 下 100 profile）P99 < 50ms
  - [ ] DB 文件大小 / 压缩后大小
- [ ] **rusqlite benchmark**（相同场景，相同硬件，同次跑）
- [ ] **对比结论**（下面 3 种之一）：
  - (A) redb 2 所有场景达标 → 锁定 redb，`CLAUDE.md` #14 B → A
  - (B) redb 2 读写满足但稳定性有疑虑（crash / 数据损坏率 > 0）→ 锁定 rusqlite
  - (C) redb 2 性能不满足 → 锁定 rusqlite
- [ ] 结论写入 **ADR-005**（Phase 3 后建立）

### B. Git2 写路径 smoke test（副线）

- [ ] 在临时 repo 里用 git2 完成：`add` + `commit` + 验证 commit hash 正确
- [ ] 支持中文 commit message（UTF-8 无乱码）
- [ ] `author` / `committer` 字段正确写入

## ❌ 失败信号（Fail Signals）

Storage：

- redb 2 在 1000 万行测试中出现数据损坏 / crash → 直接切 rusqlite（R27 触发）
- rusqlite 和 redb 双方都不达标 → 升级为 Arbiter 仲裁（需要 D5 额外半天做其他存储方案评估，如 sled 0.34）

Git2 写：

- 中文 commit message 乱码 → 调查 UTF-8 encoding 参数
- `commit` 成功但 git log 读不到 → 调查 ref update

## 🔀 Fallback 方案

**Storage 通过 (A)** → `CLAUDE.md` #14 锁定 redb 2，B → A
**Storage 通过 (B)/(C)** → `CLAUDE.md` #14 更新为 "rusqlite"，B 栏保留 fallback 注
**Storage 双失败** → Arbiter 仲裁是否扩展 Spike 评估 sled

**Git2 写通过** → MVP-0X commit 功能 spec 可正式写入
**Git2 写失败** → 调查 + 增补 SPIKE-04.5 专项 spike（不扩展本 Spike）

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-04-storage-bench/`：benchmark 代码 + 数据集生成器
- [ ] `docs/SPIKE-REPORT.md` storage benchmark 数据表
- [ ] redb 2 和 rusqlite 各 1 份 `.db` 文件样例（压缩后 attach 到 spike-artifacts）
- [ ] **ADR-005 草稿**：本地存储决策
- [ ] **独立 `chore(decision)` PR**：锁定 `CLAUDE.md` 决策表 #14（本 SPIKE merge 后另开）
- [ ] git2 写 smoke test 代码 + 输出日志

## 🛠 依赖资源（Resources Needed）

- SPIKE-02 产出的空壳 Tauri 项目
- redb 2.x + rusqlite 0.31+ + r2d2_sqlite（连接池）
- 测试机：建议 SSD（机械硬盘会严重干扰结果）
- 至少 4GB 可用磁盘（数据集 + 两个 DB 各留一份）

## ⚠️ 已知风险

- **R27**（`implementation-plan.md §9`）：redb 2 稳定性，本 Spike 消除
- **数据集代表性**：1000 万行是上限估计，MVP 初期可能只有 1 万行；但底线性能不过关就意味着扩展性有问题
- **fsync 行为差异**：redb 和 rusqlite 的 durability 默认不同，benchmark 要对齐配置（都开 fsync 或都不开）

---

## 📝 Notes / 讨论

- redb 2 是 embedded kv store（类似 sled），MVP 用来存 workspace metadata / profile / terminal 快照 / 配置
- rusqlite 的优势：成熟度 + SQL 查询；劣势：embedded kv 场景下 overhead 较高
- benchmark 时 fsync 策略：**两种方案都用 fsync**（生产默认），保证对比公平
- 数据模型（redb table schema）在 MVP-0X spec 里定义，本 Spike 只用占位 schema 测性能

## 🔗 相关

- ADR：`docs/adr/ADR-005-local-storage.md`
- 对应 `CLAUDE.md` 决策表：**#14 本地存储**
- `implementation-plan.md` 章节：§附录 A D4 · §9 R27 · §3.2 · §512（redb/rusqlite benchmark 扩展说明）
- 上游：SPIKE-02
- 下游：MVP 存储层 spec（MVP-0X，后续 PR）

---

**填写完毕后自审**：

1. **递归完备性**：主线 storage + 副线 git2 写 都覆盖 ✅
2. **反向场景**：redb 失败 → rusqlite；双失败 → Arbiter ✅
3. **边界适用性**：1000 万行压测 + fsync 对齐 ✅
4. **YAGNI**：不在 Spike 做 table schema 设计（留给 MVP-0X） ✅
