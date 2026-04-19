# SPIKE-05 · portable-pty + xterm 多 Tab 压测源码

对应 report：[`docs/spikes/SPIKE-05-report.md`](../../SPIKE-05-report.md)
对应 ADR：[`docs/adr/ADR-003-pty-architecture.md`](../../../adr/ADR-003-pty-architecture.md)
对应 spec：[`docs/tasks/SPIKE-05-pty-multi-tab.md`](../../../tasks/SPIKE-05-pty-multi-tab.md)
follow-up：[`SPIKE-05.5`](../../../tasks/SPIKE-05.5-pty-visible-throughput-fallback.md)

## 来源

- **交付 agent**：Codex CLI（2026-04-19）
- **主结论**：shared-reader + bounded queue 在 HOL / boundedness 上通过，但 visible throughput 未达 A 阈值
- **原始工作目录**：`spike-tmp/spike-05-pty/`
- **冷备**：`spike-tmp/archive/SPIKE-05/`（gitignored · 含 node_modules / target / bundle 产物）

## 目录

- `src/`：SolidJS 压测面板 + xterm 封装 + scenario runner
- `src-tauri/`：portable-pty + mio 共享读线程 + bounded queue 后端
- `scripts/`：自动跑场景的 shell 脚本
- `SUMMARY.md` / `reproduce.md`：结论与复现命令

## 复现

```bash
cd docs/spikes/code/SPIKE-05
./scripts/bench-single-tab.sh 3
./scripts/bench-4tab.sh 3
./scripts/check-correctness.sh 3
./scripts/hol-frontend-slow.sh 3
./scripts/hol-ipc-saturated.sh 1
./scripts/hol-hidden-throttle.sh 1
./scripts/soak-10min.sh 1 1
```

## Raw 数据溯源

- `../../raw/SPIKE-05/single-yes/`
- `../../raw/SPIKE-05/interactive-top/`
- `../../raw/SPIKE-05/four-yes/`
- `../../raw/SPIKE-05/soak-10min/`
- `../../raw/SPIKE-05/hidden-5min/`
- `../../raw/SPIKE-05/hol-frontend-slow/`
- `../../raw/SPIKE-05/hol-ipc-saturated/`
- `../../raw/SPIKE-05/hol-hidden-throttle/`
- `../../raw/SPIKE-05/correctness/`
