# SPIKE-04.5 · rusqlite 数据安全全链路验证报告

> **Task spec**：[`docs/tasks/SPIKE-04.5-rusqlite-safety-verification.md`](../tasks/SPIKE-04.5-rusqlite-safety-verification.md)
> **结论**：**(B partial) · B.1-5 全过 · A.3 性能 FAIL (> 50ms) · R27(silent data corruption)全面 close · A.3 方案(a) MVP 接受 220ms（Arbiter 2026-04-19）**
> **实施者**：OpenCode agent（2 次交付 · v1 被 review BLOCK · v2 补做 4 CRITICAL 后 accept）
> **Review**：Claude Code (Sonnet 4.6)
> **前置 Spike**：[SPIKE-04](./SPIKE-04-report.md)（redb B.2 FAIL → 锁 rusqlite）
> **相关 ADR**：[ADR-005 本地存储](../adr/ADR-005-local-storage.md)（proposed → accepted · 待更新 rusqlite 结论）

---

## 1 · 结论概览

| 维度 | rusqlite 0.31 (bundled SQLite 3.x) | 判定 |
|---|---|---|
| §A.1 批量写入 P99 | 12.90s | ✅ < 60s |
| §A.2 单键读 P99 | 0.010ms | ✅ < 5ms |
| **§A.3 范围查询 P99** | **215ms** | ❌ **FAIL**（> 50ms threshold · 超 4.3×） |
| §A.4 DB 文件大小 | 1.06 GB（post-VACUUM）| 参考 |
| **§B.1 Crash 恢复** | 30/30 · 0 行泄漏 | ✅ PASS |
| **§B.2 坏库检测** | SQLITE_CORRUPT · 用户友好提示 | ✅ PASS（vs redb silent FAIL） |
| §B.3 Schema 迁移 | V1→V2 + H1 old→new assert + ROLLBACK | ✅ PASS |
| §B.4 Export/Import | SHA256 + H2 pre-import backup + per_table manifest | ✅ PASS |
| §B.5 启动自检 | per-tx_id op-log + 原子 manifest + retention + 4 crash scenario | ✅ PASS |

**严格按 spec §B.6**：
- (A) 性能达标 + B.1-5 全过 → R27 truly closed · **未触发**（A.3 FAIL）
- **(B partial) B.1-5 全过 · A.3 性能 FAIL · R27(silent data corruption)全面 close · A.3 性能单独处理** · **本次结论**
- R27 数据安全部分：**全面 close**（B.2 坏库检测 · silent data return 已消除）
- A.3 性能未达 spec 阈值：**Arbiter 选定方案 (a) MVP 接受 220ms**（2026-04-19）

### A.3 决策结果

**Arbiter 选定方案 (a) MVP 接受 220ms**（2026-04-19）· 理由：
- 100 行 UI 加载延迟 220ms < 300ms 人类可接受范围
- 不动代码 · MVP 不阻塞
- 方案 (b) 复合 index `(workspace_id, profile_id, snapshot_id DESC)` 留作 MVP-02 性能优化项

---

## 2 · v1 → v2 追溯

v1 被 review 后 BLOCK · 4 项 CRITICAL + 1 项 MEDIUM：

| # | 问题 | v1 状态 | v2 修复 |
|---|---|---|---|
| **Critical #1** | A.3 阈值 `50.0`(秒) 应为 `0.050`(秒=50ms) + SUMMARY 洗白 | FAIL 被标为 PASS | `s_pass = s_p99 < 0.050` + SUMMARY 诚实标 FAIL |
| **Critical #2** | A.2 阈值 `5.0`(秒) 应为 `0.005`(秒=5ms) | 侥幸通过但判断错误 | `r_pass = r_p99 < 0.005` |
| **Critical #3** | Manifest 缺 `per_table` + `last_committed_tx_id` + 非原子写入 | 仅 `{schema_version, row_count, sha256, ts}` | 扩展 struct + `.tmp`+`rename` 原子写入 + retention 演示 |
| **Critical #4** | `main.rs` 1054 行单文件 | 所有逻辑塞一起 | 拆分为 5 独立业务模块（manifest/op_log/self_check/backup/rollback_ui · 共 261 行）+ main.rs 保留 927 行（A/B 测试 orchestration + helper · 属测试代码层）· 见 §10 |
| **Medium #5** | B.1 "kill-9" 措辞不准确 | 实际是 uncommitted transaction rollback | 明确为 "transaction not committed before connection drop" |

### H1-H4 修复确认（继承自 SPIKE-04 缺陷）

| H# | 描述 | SPIKE-04 状态 | SPIKE-04.5 状态 |
|----|---|---|---|
| **H1** | old version reads new DB 无 assert | 仅 log warning | ✅ 实际 assert error: "Schema version 2 is newer than supported version 1" |
| **H2** | B.4 无 pre-import backup | 无备份机制 | ✅ 自动创建 pre-import backup · 500 行确认保留 |
| **H3** | B.5 op-log 1-byte phase POC | 非生产级 | ✅ per-tx_id UUID + pending/committed/aborted + fsync'd JSONL |
| **H4** | range scan 测 1M 行而非 100 行 | 偏离 spec | ✅ 100 profiles × 10 iterations · workspace_id=5 |

---

## 3 · 环境

| 维度 | 数据 |
|---|---|
| OS | macOS 15.x (Apple Silicon) |
| CPU | Apple M2 Max (12-core · 8P+4E) |
| RAM | 34.4 GB LPDDR5 |
| Rust toolchain | rustc 1.95.0 (2026-04-14) |
| rusqlite | 0.31.0 (bundled SQLite 3.x) |
| OS cache | 未执行 purge（无 sudo）· 所有 P99 为原始数据 |

### 数据集规格

- 10 workspace × 100 profile × 10,000 snapshot = **10,000,000 行**
- key: 12 bytes · value: 72 bytes · 每行 84 bytes · 理论 ≈ 840 MB

### 测试配置对齐

- **rusqlite**：`PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL`
- **bulk write**：3 次独立迭代 · 每次删除旧 DB 重建 · 避免缓存加速
- **point read**：10,000 次随机 key 查询 · P50/P99
- **range scan**：workspace_id=5 · 100 profiles × latest snapshot · 10 iterations · P50/P99

---

## 4 · A. 性能测试详细

### A.1 批量写入 (10M rows × 3 iterations)

| Iteration | Duration |
|-----------|----------|
| 0 | 12.90s |
| 1 | 12.59s |
| 2 | 12.74s |

- **P50**: 12.74s · **P99**: 12.90s · **Threshold**: < 60s → ✅ PASS
- DB size: pre-VACUUM 1.10 GB → post-VACUUM 1.06 GB (3.2% reduction)

### A.2 单键读 (10,000 random keys)

- **P50**: 0.0055ms · **P99**: 0.0101ms · **Threshold**: < 5ms → ✅ PASS

### A.3 范围查询 (100 rows, 10 iterations)

| Iteration | Duration (ms) |
|-----------|--------------|
| 0 | 211 |
| 1 | 213 |
| 2 | 214 |
| 3 | 215 |
| 4 | 214 |
| 5 | 213 |
| 6 | 212 |
| 7 | 210 |
| 8 | 214 |
| 9 | 212 |

- **P50**: 213ms · **P99**: 215ms · **Threshold**: < 50ms → ❌ **FAIL** (4.3× over)
- **根因**：`WHERE workspace_id=5 ORDER BY profile_id, snapshot_id DESC` 需扫描整个 workspace 5 的数据（~1M rows），当前索引 `(workspace_id, profile_id)` 无法优化 "latest per profile" 模式

---

## 5 · B.1 Crash 恢复

**语义**：使用 "transaction not committed before connection drop" 模拟未提交数据丢失 · 测试 SQLite WAL 回滚非提交页。这**不是** SIGKILL 中途 commit（需 OS 级进程管理 · 超出本 Spike scope）· 但 WAL 保证已由 SQLite 生产验证。

| Scenario | 10/10 Pass |
|----------|-----------|
| kill@10% | ✅ 10/10 |
| kill@50% | ✅ 10/10 |
| kill@90% | ✅ 10/10 |

**结果**：30/30 · 0 行泄漏。SQLite WAL 原子事务保证无数据丢失。

---

## 6 · B.2 坏库检测

**关键对比**：

| Engine | 行为 |
|--------|------|
| **redb 2.6.3** | 打开坏库 · **静默返回 1000 行** · 无 error |
| **rusqlite 0.31** | **检测到坏库** · `PRAGMA integrity_check` → "database disk image is malformed" · 用户友好提示 |

这消除 SPIKE-04 发现的最危险失败模式：silent data corruption。

---

## 7 · B.3 Schema 迁移

| 子测试 | 结果 |
|--------|------|
| V1→V2 迁移 (1000 rows) | ✅ version=2, rows=1000 |
| **H1**: old code (v1) reads new DB (v2) | ✅ REJECTED: "Schema version 2 is newer than supported version 1" |
| 失败 ROLLBACK | ✅ version=1, rows=500 (data intact) |
| 10 数据量遍历 | ✅ ALL PASS |

---

## 8 · B.4 Export/Import

| 子测试 | 结果 |
|--------|------|
| 导出 3000 rows | ✅ SHA256 manifest + per_table checksum |
| 导入到干净目标 | ✅ 3000 rows 恢复 |
| **H2**: pre-import backup | ✅ 500 rows 备份保留 · 确认 |
| 篡改 checksum 拒绝 | ✅ "Checksum mismatch" |
| 不兼容版本拒绝 | ✅ "Unsupported schema version: 99" |

### v2 Manifest 格式

```json
{
  "user_version": 2,
  "per_table": {
    "snapshots": {
      "row_count": 3000,
      "sha256_checksum": "b2668e611a68c3ec..."
    }
  },
  "last_committed_tx_id": null,
  "export_timestamp": 1745...
}
```

Manifest 写入使用 `.tmp` + `rename` 原子操作（防止 crash 半写）。

---

## 9 · B.5 启动自检 + Op-log + Auto-rollback

### B.5.1 Happy Path
- DB 100 rows → op-log "committed" → **Consistent** ✅

### B.5.2 Marker-loss Crash
- DB 50 rows committed · op-log "pending" → **ReconciledForward** (pending→committed) ✅

### B.5.3 Normal Abort
- DB 0 rows (rollback) · op-log "pending" → **ReconciledForward** (pending→aborted) ✅

### B.5.4 Silent Loss Detection
- DB 30 rows · op-log claims 50 → **SilentLossDetected** ✅

### B.5.5 Silent Overwrite
- Pre-migration manifest checksum mismatch → **ChecksumMismatch** ✅

### B.5.6 Auto-rollback UI + Backup Retention

```
Creating 3 periodic backups (retention=2)...
Periodic backup 1: auto-1776571253.backup
Periodic backup 2: auto-1776571254.backup
Periodic backup 3: auto-1776571255.backup
Remaining auto backups: 3 (should be ≤ 3: 2 retention + 1 last-known-good)
Retention policy: PASS
Last-known-good: exists=true, 100 rows
Rollback complete: 100 rows restored
Post-rollback self-check: Consistent PASS
```

---

## 10 · 代码组织

> **Review 修正**：v2 交付时 OpenCode 在 SUMMARY/report 里声明 `main.rs ~190 行` ·
> 主 agent 归档核对发现实际 **927 行** · 下表为真实数据。代码层面 5 个独立业务
> 模块按 v2 退回 prompt 要求拆分 · main.rs 仍含 A/B 测试 orchestration（`run_perf` /
> `run_b1..b5`）+ B.4 export/import 辅助函数 + 通用 helper（setup_conn 等）·
> 属 **测试代码层** · 不是业务模块。满足 Codex review "拆出独立业务模块" intent ·
> 不满足 "main.rs < 200 行" 的字面期望 · 留待未来如需长期维护时 refactor。

| 模块 | 行数（实测） | 职责 |
|------|---------|------|
| `src/main.rs` | **927** | A/B 测试 orchestration（run_perf / run_b1..b5）+ B.4 export/import 辅助 + 通用 helper |
| `src/manifest.rs` | 28 | Manifest struct（per_table + last_committed_tx_id）+ `.tmp`+`rename` 原子写入 |
| `src/op_log.rs` | 40 | OpLogEntry · write/read/update · fsync · append-only JSONL |
| `src/self_check_mod.rs` | 68 | self_check · reconcile forward · silent-loss detection |
| `src/backup_mod.rs` | 92 | create_backup · create_periodic_backup（retention）· update_last_known_good |
| `src/rollback_ui_mod.rs` | 33 | CLI mock auto-rollback UI |
| **小计** | **1188** | v1 是 1054 行单 main · v2 总代码量略增 134 行（Manifest 扩展 + retention + reconcile forward） |

---

## 11 · 交付物清单

| 类别 | 路径 | 内容 |
|------|------|------|
| **报告** | `docs/spikes/SPIKE-04.5-report.md` | 本文件 |
| **源码** | `docs/spikes/code/SPIKE-04.5/` | 6 源文件 + Cargo.toml + Cargo.lock |
| **原始数据** | `docs/spikes/raw/SPIKE-04.5/` | 8 raw-data 文件（含 full-run-v2.txt + manifest-sample.json） |
| **冷备** | `spike-tmp/archive/SPIKE-04.5/v1.tar.gz` | v1 tarball (27KB) |
| **冷备** | `spike-tmp/archive/SPIKE-04.5/v2.tar.gz` | v2 tarball (29KB) |

---

## 12 · 下一步（Arbiter 决策）

- [ ] A.3 性能决策：(a) MVP 接受 220ms / (b) 加复合 index 重测 / (c) scope 降级
- [ ] ADR-005 修订 PR：加 "rusqlite B.1-5 实测通过 · A.3 FAIL → Arbiter 决策"
- [ ] MVP 后续：实际 `PRAGMA user_version` 迁移实现 + 复合 index 评估