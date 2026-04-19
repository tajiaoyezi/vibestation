# ADR-005: 本地存储 = **rusqlite**（redb 2.6.3 B.2 FAIL 后 supersede）· pending SPIKE-04.5 B.1-5 on rusqlite

**状态**：**accepted**（2026-04-19 · SPIKE-04 Phase A+B 结论 (B) · redb 2.6.3 B.2 坏库检测 FAIL · 回退 rusqlite）
**日期**：2026-04-18 初版 proposed（默认 redb）· 2026-04-19 accepted 但结论翻转（redb → rusqlite）
**决策者**：项目发起人（Arbiter）· OpenCode agent 实测 · Claude Code review
**对应 `CLAUDE.md` 决策表**：#14（从 B 档升级到 A 档 · 但锁定的是 **rusqlite** · 不是 redb）
**对应 Spike**：[SPIKE-04 · 已 done](../tasks/SPIKE-04-storage-benchmark.md)
**对应 Report**：[SPIKE-04-report](../spikes/SPIKE-04-report.md)
**后续 Spike**：[SPIKE-04.5 · 新建](../tasks/SPIKE-04.5-rusqlite-safety-verification.md)（在 rusqlite 上补 B.1-B.5 · 真正 close R27）

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

**选择（accepted · 2026-04-19 · SPIKE-04 结论 (B) 锁定）**：
- **生产方案**：`rusqlite 0.31+` + `r2d2_sqlite`（连接池）
- **原默认 redb 2** · **已淘汰**（SPIKE-04 B.2 FAIL）
- **Fallback trigger（若 rusqlite 将来 SPIKE-04.5 失败）**：扩展评估 sled 或 LMDB

### SPIKE-04 benchmark 结论（2026-04-19）

§A 性能（5 次独立迭代 · P99 · linux kernel MVP 数据模型 10M 行）：
- **写入**：redb 31.94s · rusqlite 9.96s · 两者 < 60s 阈值 · rusqlite 3.2× 快
- **单键读**：redb 0.007ms · rusqlite 0.011ms · 两者 << 5ms · redb 略优
- **范围查询**：redb 110ms · rusqlite 113ms · 均 > 50ms · 测试设计问题（1M 全扫描）· 非 blocker

§B 数据安全（redb 2.6.3）：
- B.1 Crash 恢复：✅ PASS
- **B.2 坏库检测：❌ FAIL**（中间 512 bytes overwrite · DB 静默成功读出 · 无 error）
- B.3 Schema 迁移：✅ PASS
- B.4 Export/Import：✅ PASS（功能 80%）
- B.5 启动自检：✅ PASS（POC 级）

按 spec §B.6：**B.2 FAIL → R27 未消除 → 锁 rusqlite**（spec 明确路径）。

详细数据见 [SPIKE-04-report §4](../spikes/SPIKE-04-report.md)。

### 本 ADR 的 caveat

**SPIKE-04 只证明了 redb 2.6.3 不行** · **未证明 rusqlite 的 B.1-B.5 全通过**。真正 close R27 需要：

1. **SPIKE-04.5**（本 ADR 立项 · 由 PR C 新建 spec）
2. 在 rusqlite 上重跑 B.1-B.5 · 全过才算真 accept
3. 届时本 ADR 加"rusqlite B.1-B.5 全过"修订条目

### 重新判定路径（未来可能）

若 SPIKE-04.5 发现 rusqlite 也有 silent corruption 缺陷 → Arbiter 仲裁 · 评估 sled / LMDB / 其他。

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
- 2026-04-18 · 初版 · Claude Code · status: proposed · 默认 redb · 等 SPIKE-04 B.1-5 + A 全过后改 accepted
- 2026-04-19 · SPIKE-04 结论 (B) 落地 · OpenCode agent 实测（v1 → v2 补做）· Claude Code review · status: proposed → **accepted** · **结论翻转**：redb 2.6.3 B.2 FAIL → 锁 rusqlite
- 2026-04-19 · 立项 SPIKE-04.5 · 在 rusqlite 上补 B.1-B.5 · 真正 close R27
