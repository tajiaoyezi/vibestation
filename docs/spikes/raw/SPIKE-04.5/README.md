# SPIKE-04.5 · benchmark + safety 原始数据（v2 accepted 版）

对应 report：[`docs/spikes/SPIKE-04.5-report.md`](../../SPIKE-04.5-report.md)
对应源码：[`docs/spikes/code/SPIKE-04.5/`](../../code/SPIKE-04.5/)

## 来源

- **产生时间**：2026-04-19（OpenCode agent v2 补做 · 同日 accept）
- **测试机**：Apple M2 Max · 34.4 GB RAM · macOS · SSD
- **版本**：v2 accepted（v1 数据保留在 `full-run.txt` 作为对照 · 实际结论依据 `full-run-v2.txt`）
- **OS 缓存**：未执行 `sudo purge`（无 sudo）· 所有 P99 为原始数据

## 文件索引

| 文件 | 对应 report 章节 | 说明 |
|------|-----------------|------|
| `full-run.txt` | §4 A / v1 老数据 | v1 完整 run log（含 A.3 错误阈值判定 · 作为 v2 追溯证据） |
| **`full-run-v2.txt`** | §4 A + §5-9 B.1-5 | **v2 完整 run log**（含 A.3 正确 FAIL 判定 + retention demo） |
| `perf-raw.txt` | §4 A | A.1/A.2/A.3 每次迭代 raw duration |
| `b1-raw.log` | §5 B.1 | Crash Recovery 30 次运行日志（10%/50%/90% × 10） |
| `b2-raw.txt` | §6 B.2 | SQLITE_CORRUPT 检测 + redb 对照 |
| `b3-raw.log` | §7 B.3 | Schema Migration V1→V2 + 10 数据量遍历 |
| `b4-raw.log` | §8 B.4 | Export/Import + pre-import backup |
| `b5-raw.log` | §9 B.5 | 4 crash scenario + retention summary |
| `manifest-sample.json` | §8 v2 Manifest 格式 | Manifest 完整 per_table + last_committed_tx_id 示例 |

## 关键数据引用

### A.3 FAIL（report §4.3 核心证据）

`perf-raw.txt` 的 Range Scan 10 次迭代：

```
iter 0: 100 profiles (latest each) in 211.073ms
iter 1..9: 210-215 ms
Range scan P50=213.005ms P99=214.940ms PASS=NO
```

P99 215ms > 50ms 阈值 · 代码 `s_pass = s_p99 < 0.050` 正确判定 FAIL。Report §4.3 与此一致。

### B.5 Retention demo（report §9.6 证据）

`full-run-v2.txt` 的完整演示：

```
Creating 3 periodic backups (retention=2)...
Periodic backup 1: auto-1776571253.backup
Periodic backup 2: auto-1776571254.backup
Periodic backup 3: auto-1776571255.backup
Remaining auto backups: 3 (should be ≤ 3: 2 retention + 1 last-known-good)
Retention policy: PASS
Last-known-good: exists=true, 100 rows
```

对应 `src/backup_mod.rs::create_periodic_backup` 实现。

## 注意

- **本目录数据是决策证据快照** · 不要修改
- `full-run.txt`（v1）保留作为 "什么不该做" 的证据 · 和 `full-run-v2.txt`（v2）对照可见 v1→v2 的修复轨迹
- v1 的 A.3 在 `full-run.txt` 里被标 PASS=YES（代码阈值 bug）· v2 的 `full-run-v2.txt` 正确标 PASS=NO · 这是代码层 bug 修复的完整证据链
