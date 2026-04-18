# SPIKE-01 · 测量脚本归档

这里是 SPIKE-01 冷启动测量脚本的**仓库归档副本**，与 `spike-tmp/spike-01-tauri/scripts/` 保持一致。

## 为什么两份？

- **仓库副本（本目录）**：跟随 main 分支持久化。Ubuntu agent / 人肉执行者看得到。
- **运行副本（spike-tmp 内）**：实际与骨架代码一起工作。`spike-tmp/` 被 `.gitignore` 排除，不进仓库。

**以仓库副本为准** — 如果需要修改脚本，先改本目录的，再同步回运行副本。

## 使用方式

### macOS（Phase A）

```bash
# 前置：骨架已 scaffold + pnpm tauri build 完成
cp docs/spikes/scripts/SPIKE-01/measure-boot-macos.sh spike-tmp/spike-01-tauri/scripts/
chmod +x spike-tmp/spike-01-tauri/scripts/measure-boot-macos.sh
cd spike-tmp/spike-01-tauri
./scripts/measure-boot-macos.sh 10   # 10 次采样 · 中位数判定
```

### Ubuntu（Phase B）

```bash
# 详细步骤见 docs/spikes/SPIKE-01-report.md §5.1 给 Ubuntu agent 的 prompt
cd spike-01-tauri
./scripts/measure-boot-ubuntu.sh 10
```

## 埋点契约

脚本依赖 `src-tauri/src/lib.rs` 里的 `eprintln!("[SPIKE-01] window_ready t={}ms", elapsed_ms);`。

如果改这行格式，必须同步改脚本的 `grep -oE "window_ready t=[0-9]+ms"` 正则。
