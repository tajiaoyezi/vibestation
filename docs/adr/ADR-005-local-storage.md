# ADR-005: 本地存储 = redb 2（默认）· rusqlite（fallback）· pending SPIKE-04

**状态**：**proposed**（pending [SPIKE-04](../tasks/SPIKE-04-storage-benchmark.md) 通过后升级为 accepted）
**日期**：2026-04-18（Phase 1 默认选 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审 · 待 SPIKE-04 Arbiter 仲裁（若失败）
**对应 `CLAUDE.md` 决策表**：#14（B 档，Spike 后锁定）
**对应 Spike**：[SPIKE-04](../tasks/SPIKE-04-storage-benchmark.md)

---

## 背景与问题

MVP 需要本地持久化：
- workspace metadata · 10 workspace × 100 profile × 10k terminal 快照 ≈ 1000 万条
- 配置（settings / telemetry opt-in）
- session / commit 关联（v1.0 MVP-19）

关键需求：
- **写性能**：1000 万行 P99 < 60s
- **读性能**：单键 P99 < 5ms · workspace 下 100 profile 范围查询 < 50ms
- **数据安全**（R27 · HIGH）：DB 损坏 / 升级迁移失败必须可恢复 · 不得 silent 丢数据

## 决策驱动因素

- **D1 · 性能**：见上（`implementation-plan.md §10.2`）
- **D2 · 数据安全**：R27 硬要求 · crash 恢复 / 坏库检测 / migration / 备份 / 启动自检 / silent loss 检测全链路
- **D3 · API 简单**：embedded kv 场景 · 不需要 SQL 灵活性
- **D4 · 生态 / 维护**：库活跃度 · 破坏性变更频率

## 考虑的选项

- **A · redb 2**：纯 Rust · embedded kv · ACID · 文件单一 · 新（2.0 发布 2024）· 性能优
- **B · rusqlite** + r2d2_sqlite：SQLite 绑定 · 成熟 20+ 年 · SQL 灵活 · 文件单一 · 性能中等
- **C · sled**：纯 Rust embedded kv · 较老但维护疲软 · 稳定性未知
- **D · LMDB (heed)**：memory-mapped · 极快 · 但文件格式较奇特 · 迁移成本高
- **E · file system + bincode**：最简 · 但需手写索引 · 不考虑

## 决策

**选择（proposed · pending SPIKE-04）**：
- **默认**：`redb 2.x`（embedded kv · ACID · 性能优）
- **Fallback**：`rusqlite 0.31+` + `r2d2_sqlite`（连接池）
- **Fallback trigger**：SPIKE-04 A 性能失败 · OR · B 数据安全任一失败（crash / 坏库 / migration / 备份 / 启动自检 / silent loss / silent overwrite）· OR · 双失败时扩展 Spike 评估 sled / LMDB

**理由**：
1. **redb 2 纯 Rust**：无 C 依赖 · 跨编译简单 · Cargo.lock 审计路径短
2. **预期性能优于 rusqlite**：embedded kv 场景下 SQL layer overhead 不必要
3. **R27 依赖 Spike 硬验证**：Codex 连续 3 轮审查指出"性能 benchmark 无法消除 R27 数据安全风险" · SPIKE-04 B.1-5 是硬阻塞（见 SPIKE-04 B.5 启动自检 + op-log / manifest 2-phase 写入设计）

## 后果

### 正面

- **性能上限高**：redb 2 的 B-tree 实现 + mmap 内存映射 · MVP 级数据量绰绰有余
- **API 简单**：`table.insert(key, value)` · 无 SQL dialect 学习
- **Cargo.lock 审计路径短**：纯 Rust · 无 libsqlite 依赖
- **Fallback 清晰**：rusqlite 是 Plan B · 成熟 20+ 年 · 若 redb 栽了 · 切换是平替（性能降 10-30% · 功能齐全）

### 负面

- **redb 2 较新**：2024 年发布 · 生产使用验证不如 rusqlite · bug / 稳定性风险高于 SQLite
- **无 SQL flexibility**：若未来需要复杂 JOIN / aggregate → 需要手写索引或迁移 rusqlite
- **数据安全依赖 SPIKE-04**：不过 Spike 不能锁定 · 存在 rebase 工期风险

### 风险

- **Silent data loss / silent overwrite**（Codex PR #10 R4 F2 / R5 F1 复核教训）：redb 2 如不支持 crash safe + 2-phase migration → 必须在应用层补 op-log / manifest（SPIKE-04 B.5 已规划）
- **Schema migration 死锁**：redb schema change 无 SQL ALTER TABLE 等价 · 必须"先读旧 → 写新 → 原子 rename" · Spike 验证
- **升级破坏**：redb 2 → 3 若 API 大改 · 需要迁移层 · 当前可接受（2.0 发布不久）

## 与 `implementation-plan.md` 的映射

- 对应章节：§3.2（存储层）· §附录 A D4（SPIKE-04 计划）
- 对应风险：**R27**（HIGH · 本 ADR 前置缓解 · SPIKE-04 硬验证）

## 相关

- `CLAUDE.md` 决策表：#14
- Spike：[SPIKE-04 redb 2 vs rusqlite benchmark + git2 写 commit](../tasks/SPIKE-04-storage-benchmark.md)
- Codex 对抗性教训：PR #3 R1 F1 · PR #7 F1 · PR #10 R4 F2 · PR #10 R5 F1

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code · status: proposed · 等 SPIKE-04 B.1-5 + A 全过后改 accepted
