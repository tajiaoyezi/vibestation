# SPIKE-04 · benchmark raw 数据（v2 accepted 版）

对应 report：[`docs/spikes/SPIKE-04-report.md`](../../SPIKE-04-report.md)
对应源码：[`docs/spikes/code/SPIKE-04/`](../../code/SPIKE-04/)

## 来源

- **产生时间**：2026-04-19（opencode agent v2 补做 · 同日 accept）
- **测试机**：macOS · Apple Silicon
- **版本**：v2 accepted（v1 数据因 4 CRITICAL 问题作废 · 留在 `spike-tmp/archive/spike-04-work/` 本地备份）

## 文件

| 文件 | 内容 |
|---|---|
| `full-run-output.txt` (7.3 KB) | v2 完整 bench run stdout（A 性能 10M 行 + B.1-5 safety） |
| `run2-output.txt` (5.9 KB) | v2 第 2 次迭代输出（对照验证） |
| `git2-smoke-log.txt` (~1 KB) | §C git2 write smoke · 含 commit hash `bbaee4da71589eff32498337f201f7bed3db72de` · UTF-8 + 中文 + emoji 完整 |

## 关键数据索引

Report 引用的数据都能在本目录溯源：

| Report 中的数字 | raw 文件字段 |
|---|---|
| 批量写 redb 31.94s · rusqlite 9.96s | `full-run-output.txt` §A bulk_write |
| 单键读 redb 0.007ms · rusqlite 0.011ms | `full-run-output.txt` §A point_read |
| 范围查询 redb 110ms · rusqlite 113ms | `full-run-output.txt` §A range_scan（⚠️ 测 1M 行 · SPIKE-04.5 H4 瑕疵） |
| **B.2 redb 2.6.3 silent FAIL** | `full-run-output.txt` §B.2 "Corrupted DB opened OK, read 1000 rows (EXPECTED: error)" |
| B.1/B.3/B.4/B.5 各项 PASS | `full-run-output.txt` §B.1-5 各段 |

## v1 vs v2 说明

- v1（BLOCKED）数据：`spike-tmp/archive/spike-04-work/full-run-output.txt` + `run2-output.txt`（gitignored）
- v2（ACCEPTED · 本目录）：修复 v1 的 4 CRITICAL 问题 · 数据诚实 · 可 reproduce

## 注意

- 数据是**决策依据快照** · 不要修改
- B.2 silent FAIL 是 redb 2.6.3 库层 API 缺陷 · 若后续 redb 发版修复 · 需独立重测 · 不影响当前 ADR-005 锁 rusqlite 的决策
- SPIKE-04.5 运行后 · rusqlite 侧的 B.1-5 raw 数据归档到 `docs/spikes/raw/SPIKE-04.5/`
