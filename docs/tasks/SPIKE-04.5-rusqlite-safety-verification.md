---
id: SPIKE-04.5
type: spike
title: rusqlite 数据安全全链路验证（B.1-5 on rusqlite · 真正 close R27）
status: done
owner: OpenCode agent (v2 accepted · v1 BLOCKED)
phase: W0-D4.5
depends_on: ["SPIKE-04"]
blocks: ["MVP-02", "MVP-06", "MVP-10", "MVP-19"]
estimate: 1-1.5d
plan_ref: implementation-plan.md §附录 A D4 · §9 R27 · §3.2
risk_ref: R27
reviewer: Claude Code (Sonnet 4.6 · 主 agent)
---

> 📌 **执行结论**（2026-04-19 · done）：
> - **R27 数据安全全面 close**：B.1 crash · B.2 SQLITE_CORRUPT · B.3 user_version + H1 assert + ROLLBACK · B.4 pre-import backup + per_table manifest · B.5 per-tx_id op-log + reconcile forward + retention + auto-rollback UI · 全部通过
> - **A.3 范围查询 FAIL**：P99 215ms > 50ms 阈值 · 三方案待 Arbiter 决策（见 [`docs/adr/ADR-005-local-storage.md`](../adr/ADR-005-local-storage.md) 修订历史）
> - **v1→v2 追溯**：v1 被 Claude 主 agent BLOCK 4 CRITICAL（A.3 阈值单位 bug / A.2 同类 / Manifest 缺字段 / 代码未拆分）· v2 全部修复后 accept
> - **交付归档**（"4 样齐全"）：report [`docs/spikes/SPIKE-04.5-report.md`](../spikes/SPIKE-04.5-report.md) · 源码 [`docs/spikes/code/SPIKE-04.5/`](../spikes/code/SPIKE-04.5/) · raw [`docs/spikes/raw/SPIKE-04.5/`](../spikes/raw/SPIKE-04.5/) · 冷备 `spike-tmp/archive/SPIKE-04.5/{v1,v2}.tar.gz`

# SPIKE-04.5: rusqlite 数据安全全链路验证

> **状态**：`ready`（2026-04-19 由 [SPIKE-04](./SPIKE-04-storage-benchmark.md) 交接 · SPIKE-04 结论 (B) 锁 rusqlite 后需补完整 B.1-5 on rusqlite · 才能真正 close R27）
> **依赖**：SPIKE-04（已 done · 证明 redb 2.6.3 B.2 FAIL · rusqlite A 性能通过）
> **阻塞**：MVP-02 workspace metadata · MVP-06 config import · MVP-10 settings · MVP-19 session-commit 绑定（所有 rusqlite 持久化相关 MVP）
> **相关 ADR**：[ADR-005 accepted](../adr/ADR-005-local-storage.md)（结论已锁 rusqlite · 本 Spike 补应用侧安全实测）
> **前置 report**：[SPIKE-04-report](../spikes/SPIKE-04-report.md)

---

## 🎯 目标（Goal）

在 `rusqlite 0.31 + r2d2_sqlite` 上实测 B.1-B.5 数据安全全链路 · 真正消除 R27。

**SPIKE-04 只证明了 redb 2.6.3 在 B.2 上 silent 失败** · **未证明 rusqlite 的应用侧安全防护**。虽然 SQLite 行业验证 20+ 年 · 但 R27 的具体要求（crash 恢复 / 坏库检测 → 用户可读消息 / schema 迁移 / 命令路径 export-import / 启动自检 op-log 2-phase）都是**应用层设计** · 必须在 rusqlite 上实测。

## 📖 背景（Context）

- **SPIKE-04 结论**：redb 2.6.3 B.2 FAIL · 锁 rusqlite
- **SPIKE-04 瑕疵（留给本 Spike）**：
  - B.3 旧版读新 DB 未实际 assert error
  - B.4 未测 "目标已存在时自动 pre-import backup"（spec §87）
  - B.5 op-log 简化版（1 byte phase + row_count · 不是 per-tx_id · 无 manifest · 无周期快照 · 无自动回滚 UI）
  - 范围查询测试场景 1M 行 vs spec 字面 100 行歧义
- **本 Spike 新要求**：上述瑕疵在 rusqlite 上完整实施 · 避免把 redb 时代的技术债带进 MVP

---

## ✅ 通过标准（Pass Criteria）

### A. rusqlite 性能复测（简化 · 因 SPIKE-04 已证 rusqlite A 通过）

- [ ] **写入 P99 < 60s**（1000 万行 · WAL + synchronous=FULL · 3 次独立迭代即可）
- [ ] **单键读 P99 < 5ms**（10000 次随机 · 1 次即可）
- [ ] **范围查询 P99 < 50ms**（**按 spec 字面要求：workspace 下取 100 profile** · 每 profile 取 1 条最新 snapshot · 共 100 行 · 10 次迭代）· 目的：澄清 SPIKE-04 测 1M 行的歧义
- [ ] DB 文件大小记录（含 compact/VACUUM 前后对比）

### B. rusqlite 数据安全全链路（主线 · 阻塞项）

#### B.1 · Crash / 断电恢复

- [ ] 写入中途（10% / 50% / 90% 进度各一次）`kill -9`
- [ ] 重启后 DB 能打开（SQLite WAL 自动 recovery） · 已 commit 完整 · 未 commit 丢失不污染
- [ ] 三场景各 10 次 · 0 次库坏 / 数据交错

#### B.2 · 坏库检测（rusqlite 独有强项 · 必证）

- [ ] `.sqlite` 文件中间 overwrite 512 bytes 随机数据
- [ ] 重开 · 读操作**必须**返回 `SQLITE_CORRUPT` 错误（error code 11）
- [ ] 错误映射到用户可读 message："数据库文件损坏 · 请从备份恢复"
- [ ] 对照 SPIKE-04 的 redb FAIL：验证 rusqlite 在同一损坏场景下的优越性

#### B.3 · Schema 迁移（补完 SPIKE-04 瑕疵）

- [ ] `PRAGMA user_version` 作为 schema_version 标记（SQLite 原生支持）
- [ ] 写 V1 → V2 migration（`ALTER TABLE` 或 `CREATE TABLE new; INSERT ... SELECT; DROP old; ALTER RENAME`）
- [ ] 10 次不同数据量 migration 100% 成功
- [ ] 新版读旧 DB：能识别旧 user_version + 触发 migration · 不 silent 覆盖
- [ ] **旧版读新 DB**：**必须实际 assert 返回具体 error**（不是只"no crash"） · user_version 检查失败时返回明确消息
- [ ] Migration 失败场景：测试 ROLLBACK on error · 数据应回到 V1 完整状态

#### B.4 · Export/Import（补完 SPIKE-04 §87 瑕疵）

- [ ] Export 命令：`.sqlite` + manifest.json（含 `user_version` + `row_count` + `sha256_checksum`）
- [ ] Import 命令：**目标已存在时必须先自动创建 pre-import snapshot**（spec §87 原要求 · SPIKE-04 跳过）
  - 路径：`~/.vibestation/backups/pre-import-<ts>.backup/`
  - 结构：`.sqlite` + `manifest.json` · 可回滚
- [ ] Import 前强制校验 manifest checksum + user_version 兼容性 · 不兼容拒绝加载
- [ ] Round-trip 验证：export → 删 → import → 所有表 row_count + 关键 record sampling 一致

#### B.5 · 启动自检 + Op-log 2-phase + 自动回滚（production 级 · 补完 SPIKE-04 POC）

**Op-log 完整化**（非 SPIKE-04 简化版）：

- [ ] **Per-tx_id log**：每条 `{tx_id (UUID), status: pending|committed|aborted, table, key_hash, op, ts_start, ts_end, checksum}` · append-only · fsync
- [ ] **Manifest.json**：每次 Phase 2 committed 后原子更新（`.tmp` + rename）· 含 `user_version`, `per_table: {row_count, sha256_checksum}`, `last_committed_tx_id`
- [ ] **Op-log 保留策略**：保留到对应备份已成功创建为止（≥ 当前周期快照 + 3 份） · 老的滚动删除

**启动自检**（基于 DB ground truth · reconcile forward 优先）：

- [ ] (1) 打开 DB + 读 user_version
- [ ] (2) **Reconcile forward 扫描**（所有 silent-loss 判定之前）：
  - pending 无后续 + DB 有数据 → 补 committed + 重算 manifest
  - pending 无后续 + DB 无数据 → 补 aborted
- [ ] (3) Silent-loss 检测（reconcile 后）：committed 条目做 DB sampling（至少 100 条或全部）· committed 说有但 DB 查不到 → 报 silent loss
- [ ] (4) Manifest 对照 DB 实际状态：DB < manifest → silent loss · DB > manifest → reconcile forward 兜底
- [ ] (5) 悬挂事务扫描日志

**专项 crash test**（SPIKE-04 B.5 做过 POC · 本 Spike 做 production 级）：

- [ ] Marker-loss crash：DB commit 成功 · op-log committed 写之前 kill -9 · 自检走 reconcile forward · 不误报 silent loss
- [ ] 正常 abort：DB commit 前 kill · op-log pending · 自检补 aborted · 不报 silent loss
- [ ] 真 silent loss：DB commit + op-log committed · 手动删 DB 20 行 · 自检检测 + 报警 + 触发回滚
- [ ] Silent overwrite：migration 中途 kill · DB 半新半旧 · 自检对照 pre-migration manifest checksum 不匹配 · 必须回滚

**last-good backup + 自动回滚 UI**（新增 · SPIKE-04 完全没做）：

- [ ] app 每次干净退出时异步创建 `~/.vibestation/backups/auto-<ts>.backup/`（含 `.sqlite` + manifest + op-log）
- [ ] 每 10 min 周期性快照（崩溃降级兜底）
- [ ] 保留最近 3 份 + 1 份 "last-known-good"（自检通过后更新）· 老的自动回收
- [ ] **自动回滚 UI**：自检 FAIL → 弹窗 "DB 损坏 · 从 `<ts>` 备份恢复？预计丢失 N 行" → 用户确认 → 回滚 → 再自检 → 成功则启动 · 失败则提示手动 export/import

### C. 综合结论

- [ ] (A) rusqlite A + B.1-5 全过 → ADR-005 加 "rusqlite B.1-5 全过" 修订条目 · **R27 真实 close**
- [ ] (B) rusqlite 任一 B 失败 → Arbiter 仲裁（sled / LMDB / 接受风险）· **R27 未 close 但方向明确**
- [ ] ADR-005 修订（本 Spike done 后加"rusqlite B.1-5 实测通过"段）

---

## ❌ 失败信号（Fail Signals）

- B.2 rusqlite 在 512 bytes overwrite 后不报 SQLITE_CORRUPT（违反行业共识 → 升级为 P0 issue）
- B.3 旧版读新 DB 实际 crash（而非 user_version error）
- B.4 import 目标已存在时 silent 覆盖（无 auto-backup）
- B.5 Marker-loss 场景误报 silent loss（reconcile forward 实现错）
- B.5 真 silent loss 场景自检漏过
- → 任一失败：Arbiter 仲裁 + 扩展评估 sled / LMDB / 或接受已知风险入 MVP

## 🔀 Fallback 方案

**B.1-5 全过** → ADR-005 修订 "accepted · rusqlite B.1-5 on 2026-MM-DD 实测通过" · R27 真 close · MVP-02/06/10/19 解锁
**B 任一失败** → Arbiter 评估：
- (a) 接受该缺陷 + 应用层缓解（op-log checkpoint 加密校验 等）
- (b) 扩展 Spike 评估 sled（纯 Rust · 但 B.2 可能也有类似 redb 问题）
- (c) 评估 LMDB via heed（成熟度高 · 但文件格式较奇特）

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-04-5-rusqlite-safety/`（gitignored · 含 rusqlite 完整 safety 测试）
- [ ] `docs/spikes/SPIKE-04.5-report.md`（B.1-B.5 详细实测数据 + reconcile forward 关键流程图）
- [ ] ADR-005 修订 PR（加 "rusqlite B.1-5 实测通过" 段 + 日期）
- [ ] 若 B.5 production op-log 代码可复用 · 抽出到 `spike-artifacts/` 归档（参考实现 · 不直接进 MVP · 由 MVP-02 决定是否采用）

## 🛠 依赖资源（Resources Needed）

- Rust stable toolchain（已就绪）
- `rusqlite 0.31+` with `bundled-sqlite3` feature
- `r2d2_sqlite` 连接池
- SSD · 4GB 可用磁盘（测试 DB + 备份 + op-log）
- **可选**：SQLite CLI 做损坏场景手动验证

## ⚠️ 已知风险

- **B.4 auto-backup 路径**：`~/.vibestation/backups/` 如用户主目录满 → 需降级策略（提示用户清盘 · 不 silent 跳过 backup）
- **B.5 op-log 文件大小**：长期运行可能累积 · 保留策略必须严格 · 否则磁盘压力
- **SQLITE_CORRUPT 对齐**：rusqlite 返回的 error 需要正确映射到用户消息 · 不能直接显示 C-style error code
- **Migration failure rollback**：SQLite BEGIN IMMEDIATE + ROLLBACK 是标准 · 但应用侧要保证 transaction boundary 正确

---

## 📝 Notes / 讨论

- **与 SPIKE-04 关系**：SPIKE-04 证明 redb 不行 · 切 rusqlite；本 Spike 证明 rusqlite 应用侧防护到位 · 真正 close R27
- **本 Spike 可否下发给 agent**：可以。SPIKE-04 v2 的 opencode agent 已经熟悉 safety.rs 结构 · 改 rusqlite API 成本低 · 建议继续下发给同一 agent
- **时间预期**：1-1.5d（B.5 production 级 op-log 是大头 · B.1-4 比 redb 版更简单因为 SQLite 行业成熟）

## 🔗 相关

- ADR：[ADR-005 本地存储](../adr/ADR-005-local-storage.md)
- 前置 Spike：[SPIKE-04 · done](./SPIKE-04-storage-benchmark.md)
- 前置 Report：[SPIKE-04-report](../spikes/SPIKE-04-report.md)
- 对应 `CLAUDE.md` 决策表：#14（本 Spike 完成后 accept 加 "rusqlite B.1-5 通过" 修订）
- `implementation-plan.md` 章节：§附录 A D4 · §9 R27 · §3.2
- 下游 MVP：MVP-02 / MVP-06 / MVP-10 / MVP-19（所有涉及 rusqlite 持久化）

---

**填写完毕后自审**：

1. **递归完备性**：A（性能复测 · 澄清 SPIKE-04 歧义）+ B.1-5 全链路 + C 综合结论 + 失败路径 + follow-up 归属 ✅
2. **反向场景**：rusqlite 任一失败 → Arbiter 评估 sled / LMDB / 接受风险 ✅
3. **边界适用性**：完整补 SPIKE-04 遗留瑕疵（B.3 旧版 assert · B.4 auto-backup · B.5 production op-log） ✅
4. **YAGNI**：A 性能复测简化（因 SPIKE-04 已证 rusqlite A 通过）· 只重点补 B.1-5 · 不扩评估 sled / LMDB（仅在失败时触发） ✅
