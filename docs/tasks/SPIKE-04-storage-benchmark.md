---
id: SPIKE-04
type: spike
title: redb 2 vs rusqlite benchmark + git2 写 commit 打通
status: draft
owner:
phase: W0-D4
depends_on: ["SPIKE-02"]
blocks: []
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

> ⚠️ **R27 真正消除 = A（性能）+ B（数据安全）都通过**。仅 A 通过只能说"性能可接受"，不能说 R27 已消除。Codex PR #3 Round 1 Finding 1 教训。

### A. Storage 性能 benchmark（主线 · 阻塞项）

- [ ] **数据集构造**：10 workspace × 100 profile × 10,000 snapshot（总 ~1000 万行）
- [ ] **redb 2 benchmark**：
  - [ ] 批量写入 1000 万行 P99 < 60s
  - [ ] 单键读取 P99 < 5ms
  - [ ] 范围查询（workspace 下 100 profile）P99 < 50ms
  - [ ] DB 文件大小 / 压缩后大小
- [ ] **rusqlite benchmark**（相同场景，相同硬件，同次跑，fsync 策略对齐）

### B. Storage 数据安全 · R27 真正消除（主线 · 阻塞项 · Codex PR #3 加入）

> `implementation-plan.md §9 R27` 真实风险是"redb 文件损坏或升级迁移失败导致用户数据丢失"。缓解项包括 `schema_version`、备份、自检回滚、导入导出。性能 benchmark 无法消除该风险。

**B.1 · Crash / 断电恢复**（进程级）
- [ ] 在写入中途（10% / 50% / 90% 进度各一次）用 `kill -9` 终止进程
- [ ] 重启进程后 DB 能被打开、已提交的事务完整、未提交的事务丢失但不污染
- [ ] 三种场景各测 10 次，0 次出现"库无法打开"或"数据交错损坏"

**B.2 · 坏库检测**
- [ ] 手动在 `.redb` 文件中间写入 512 字节随机数据
- [ ] 打开时能明确报错（不是 silent 成功或 segfault）
- [ ] 错误可映射到"用户可读的恢复建议"（如"从备份还原"）

**B.3 · Schema 迁移前后兼容**
- [ ] DB 文件头部带 `schema_version` 标记
- [ ] 写一个旧 schema → 新 schema 的 migration 脚本
- [ ] migration 成功率 100%（10 次各种数据量样本测试）
- [ ] 新版本读旧 DB：能识别旧版本并触发 migration，不会 silent 覆盖
- [ ] 旧版本读新 DB：能识别并拒绝加载（不是崩溃）

**B.4 · 备份 / 恢复闭环**
- [ ] 能导出 `.redb` 整文件到用户指定备份路径
- [ ] 从备份恢复到新路径，校验数据完整性（对比原库的 checksum / row count）
- [ ] 对 rusqlite 做等价测试

**B.5 · 综合结论**（基于 A + B 共同决定）
- [ ] (A) redb 2 · A 达标 + B 全通过 → 锁定 redb，`CLAUDE.md` #14 B → A，R27 消除
- [ ] (B) redb 2 · A 达标但 B 任一项失败 → **R27 未消除**，锁定 rusqlite 兜底
- [ ] (C) redb 2 · A 不达标 → 锁定 rusqlite（性能优先）
- [ ] (D) 双失败 → Arbiter 仲裁，扩展 Spike 评估 sled 或 LMDB
- [ ] 结论写入 **ADR-005**（Phase 3 后建立）

### C. Git2 写路径 smoke test（副线）

- [ ] 在临时 repo 里用 git2 完成：`add` + `commit` + 验证 commit hash 正确
- [ ] 支持中文 commit message（UTF-8 无乱码）
- [ ] `author` / `committer` 字段正确写入

## ❌ 失败信号（Fail Signals）

Storage 性能（A）：

- redb 2 在 1000 万行性能测试中出现数据损坏 / crash（写入过程）→ 直接切 rusqlite（R27 触发）
- rusqlite 和 redb 两方性能都不达标 → Arbiter 仲裁扩展评估 sled / LMDB

**Storage 数据安全（B · Codex 加入）**：

- **B.1 crash 恢复失败**：任一场景库无法打开 → 锁定 rusqlite 或扩展评估
- **B.2 坏库检测失败**：silent 成功或 segfault → 锁定 rusqlite
- **B.3 migration 兼容失败**：旧版本读新 DB 崩溃 → 锁定 rusqlite
- **B.4 备份恢复失败**：恢复后数据 checksum 不匹配 → 锁定 rusqlite

Git2 写（C）：

- 中文 commit message 乱码 → 调查 UTF-8 encoding 参数
- `commit` 成功但 git log 读不到 → 调查 ref update

## 🔀 Fallback 方案

**Storage A + B 全通过** → `CLAUDE.md` #14 锁定 redb 2，B → A，R27 消除
**Storage A 通过但 B 任一失败** → **R27 未真正消除**，锁定 rusqlite
**Storage A 不达标** → 锁定 rusqlite
**双失败** → Arbiter 仲裁

**Git2 写通过** → MVP-0X commit 功能 spec 可正式写入
**Git2 写失败** → 调查 + 增补 SPIKE-04.5 专项 spike（不扩展本 Spike）

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-04-storage-bench/`：benchmark 代码 + 数据集生成器
- [ ] **`docs/spikes/SPIKE-04-report.md`** storage benchmark 数据表（per-task）
- [ ] redb 2 和 rusqlite 各 1 份 `.db` 文件样例（压缩后 attach 到 spike-artifacts）
- [ ] **ADR-005 草稿**：本地存储决策
- [ ] `CLAUDE.md` 决策表 #14 更新 PR
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
