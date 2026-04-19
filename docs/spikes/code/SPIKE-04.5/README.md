# SPIKE-04.5 · rusqlite 数据安全全链路验证 · 源码（v2 accepted 版）

对应 report：[`docs/spikes/SPIKE-04.5-report.md`](../../SPIKE-04.5-report.md)
对应 ADR：[`docs/adr/ADR-005-local-storage.md`](../../../adr/ADR-005-local-storage.md)（accepted · rusqlite B.1-5 on-rusqlite 实测通过 · A.3 FAIL pending Arbiter）
对应 spec：[`docs/tasks/SPIKE-04.5-rusqlite-safety-verification.md`](../../../tasks/SPIKE-04.5-rusqlite-safety-verification.md)
前置 Spike：[SPIKE-04](../../SPIKE-04-report.md)（redb B.2 FAIL → 锁 rusqlite）

## 来源

- **交付 agent**：OpenCode agent（2026-04-19 · 2 次交付）
- **主 agent review**：Claude Code Sonnet 4.6
- **版本**：**v2 accepted**（v1 因 4 项 CRITICAL 被 BLOCK · v2 补做后 accept · 详见主 report §2 v1→v2 追溯）
- **冷备**：
  - `spike-tmp/archive/SPIKE-04.5/v1.tar.gz`（gitignored · 27KB · BLOCKED 版证据链）
  - `spike-tmp/archive/SPIKE-04.5/v2.tar.gz`（gitignored · 29KB · accepted 版完整 tarball）
- **本目录**：从 v2 tarball 归档 · 仅源码 + Cargo 锁文件 · 约 96KB

## 结构

```
SPIKE-04.5/
├── Cargo.toml · Cargo.lock        # 版本冻结（.gitignore 白名单 !docs/spikes/code/**/Cargo.lock）
├── src/
│   ├── main.rs                    # 测试入口 + A/B.1-5 orchestration + B.4 export_db/import_db 辅助
│   ├── manifest.rs                # Manifest struct（per_table + last_committed_tx_id）+ 原子写入
│   ├── op_log.rs                  # per-tx_id OpLogEntry · fsync · append-only JSONL
│   ├── self_check_mod.rs          # reconcile-forward + silent-loss detection
│   ├── backup_mod.rs              # create_backup + periodic backup + retention + last-known-good
│   └── rollback_ui_mod.rs         # CLI mock auto-rollback UI
└── reproduce.md                   # 原 tarball 复现命令（OpenCode 提供）
```

### 代码组织说明

- **5 个独立模块**（`manifest / op_log / self_check_mod / backup_mod / rollback_ui_mod`）按 v2 退回 prompt 要求拆分 · 每个职责单一
- **main.rs 保留 ~927 行** · 内含 A/B 测试 orchestration（`run_perf` / `run_b1..b5`）+ B.4 export/import 辅助函数 + 通用 helper（setup_conn / create_table 等）· 属测试代码层 · 非业务模块
- Report 早期草稿声明 "main.rs ~190 行" · 实际 927 行 · 归档时已在主 report §10 修正数据

## 如何复现

### 环境

- Rust stable（`rustc 1.95+`）
- macOS 或 Ubuntu（测试在 Apple M2 Max 完成）
- SSD · ≥ 4 GB 可用磁盘（10M 行 DB 约 1.1 GB）

### 命令

```bash
cd docs/spikes/code/SPIKE-04.5
cargo build --release

# 完整运行（A 性能 + B.1-5 safety · 约 2-3 min · 含 10M 行写入）
./target/release/spike-04-5

# 或 debug 版
cargo run
```

详见 `reproduce.md`（OpenCode 交付原始命令）。

## 关键结论溯源

主 report §1 里的数据 · 在源码和 raw 数据对应关系：

| Report 声明 | 代码位置 | raw 数据 |
|---|---|---|
| A.1 批量写入 P99 12.90s | `src/main.rs:138` `w_pass = w_p99 < 60.0` | `raw/SPIKE-04.5/perf-raw.txt` |
| A.2 单键读 P99 0.010ms | `src/main.rs:166` `r_pass = r_p99 < 0.005` | `raw/SPIKE-04.5/perf-raw.txt` |
| **A.3 范围查询 P99 215ms FAIL** | `src/main.rs:197` `s_pass = s_p99 < 0.050` | `raw/SPIKE-04.5/perf-raw.txt`（PASS=NO） |
| B.2 SQLITE_CORRUPT | `src/main.rs` run_b2 | `raw/SPIKE-04.5/b2-raw.txt` |
| B.5 retention demo | `src/backup_mod.rs:create_periodic_backup` | `raw/SPIKE-04.5/full-run-v2.txt`（retention=PASS） |

## 注意

- **本目录是决策证据归档** · 不直接进生产
- 生产 rusqlite 持久化实现由 MVP-02 (workspace) / MVP-06 (config) / MVP-10 (settings) / MVP-19 (session-commit) 独立 spec 负责
- 未来 rusqlite bump 版本需回归验证 · 可在此基础上改 Cargo.toml 重跑
