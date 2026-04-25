# SPIKE-01-02 Phase B · Ubuntu 实测源码

## 来源

- 交付 agent：Kimi (Moonshot)
- 执行时间：2026-04-25
- 执行环境：Ubuntu 24.04.4 LTS · x86_64 · NVIDIA RTX 5070 Ti
- Reviewer：self-review（单人项目 v2-D.1 模式）

## 文件说明

| 文件 | 用途 |
|---|---|
| `cold-boot-accurate.sh` | 精确冷启动测试 · 检测进程进入多线程状态的时间 |
| `run-cold-boot.sh` | 原始冷启动测试（固定 sleep · 供参考） |
| `plugin-smoke/` | 独立 plugin 测试项目骨架（clipboard + fs + dialog） |

## 复现命令

```bash
cd docs/spikes/code/SPIKE-01-02-phase-B
# X11 冷启动测试（10 次）
bash cold-boot-accurate.sh ../../../../target/release/vibestation-app x11 10
# Wayland 冷启动测试（需 Weston 运行中）
bash cold-boot-accurate.sh ../../../../target/release/vibestation-app wayland 10
```

## 关键结论溯源

- X11 median 108ms → `cold-boot-accurate.sh` 第 28-37 行 · 检测 STAT 含 'l'
- Wayland median 107ms → 同上 · 换 `WAYLAND_DISPLAY=wayland-1`
- 10/10 + 5/5 零失败 → raw CSV 文件在 `../../raw/SPIKE-01-02-phase-B/`
