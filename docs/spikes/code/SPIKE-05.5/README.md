# SPIKE-05.5 · visible throughput + per-session fallback 对照源码

对应 report：[`docs/spikes/SPIKE-05.5-report.md`](../../SPIKE-05.5-report.md)
对应 ADR：[`docs/adr/ADR-003-pty-architecture.md`](../../../adr/ADR-003-pty-architecture.md)
对应 spec：[`docs/tasks/SPIKE-05.5-pty-visible-throughput-fallback.md`](../../../tasks/SPIKE-05.5-pty-visible-throughput-fallback.md)

## 来源

- **交付 agent**：Codex CLI（2026-04-19）
- **原始工作目录**：`spike-tmp/spike-05.5-pty/`
- **冷备**：`spike-tmp/archive/SPIKE-05.5/`（gitignored · 含 node_modules / target / bundle 产物）

## 目录

- `src/`：SolidJS compare harness
- `src-tauri/`：shared-reader / per-session-reader 双模式 PTY 后端
- `scripts/bench-compare.sh`：主 benchmark 入口
- `scripts/check-correctness.sh`：resize / cleanup smoke
- `scripts/generate-report.py`：聚合 compare table + report
- `SUMMARY.md`：结论摘要
- `reproduce.md`：复现命令

## 复现

```bash
cd docs/spikes/code/SPIKE-05.5
./scripts/bench-compare.sh 3
./scripts/check-correctness.sh shared 1
./scripts/check-correctness.sh per-session 1
python3 ./scripts/generate-report.py
```

## 关键结论溯源

- 汇总表：`../../raw/SPIKE-05.5/compare-table.csv`
- shared 单 Tab：`../../raw/SPIKE-05.5/shared/single-yes/`
- shared 4 Tab：`../../raw/SPIKE-05.5/shared/four-yes/`
- per-session 单 Tab：`../../raw/SPIKE-05.5/per-session/single-yes/`
- per-session 4 Tab：`../../raw/SPIKE-05.5/per-session/four-yes/`
