# SPIKE-01 · Raw 数据归档

对应 report：[`docs/spikes/SPIKE-01-report.md`](../../SPIKE-01-report.md)
对应代码归档：[`docs/spikes/code/SPIKE-01/`](../../code/SPIKE-01/)

## 状态：**嵌入式 raw**（无独立 JSON 文件）

SPIKE-01 是 W0 启动期 Spike（2026-04-18）· 当时 Spike 流程仍在 ad-hoc 阶段 · raw 数据**直接嵌入** report 正文 · 未单独产出 JSON / log 文件。

这是 rule 13（跨 agent 交付物持久化）规则建立之前的历史欠账 · 在 session 10 末（2026-04-19）的 FU-4 修复中 · 我们：

1. ✅ 把源码归档进 [`docs/spikes/code/SPIKE-01/`](../../code/SPIKE-01/)（之前完全在 gitignored 的 `spike-tmp/` 下）
2. ✅ 在本 README 标注 raw 数据嵌入位置 · 让未来 agent 能溯源
3. ❌ **不**重新跑 benchmark 伪造 JSON · 因为：
   - 重新跑会产生**新数据** · 不是 2026-04-18 当时的真实测量
   - 当时 user 实测的环境 / 缓存状态 / 系统负载已不可复现
   - report 嵌入数据具备完整可追溯性（10 个原始值 · 排序后中位数计算 · 极差等）· 已满足决策证据要求

## 嵌入式 raw 数据位置

| 数据类型 | 位置（report 内） | 内容 |
|---|---|---|
| 冷启动 10 次原始毫秒数 | report §4.2 | `Run 1: 239 ms` ... `Run 10: 194 ms` |
| 排序后样本 | report §4.2 | `189, 193, 193, 194, 198, 202, 209, 209, 213, 239` |
| 统计（min/median/max/mean/range） | report §4.2 | `Median: 202 ms · Range: 50 ms` |
| 人工验证 5 项 | report §4.3 | `[x] 窗口启动后显示 ...` × 5 项 |
| Bundle 大小 | report §4.4 | `Bundle size: 8.2 MB` |
| 中文 IME 录屏 | `spike-artifacts/SPIKE-01/macos-ime.mov` | （归档文件 · 见 spike-artifacts 子系统） |

## 重新测量参考（如需独立 raw）

未来若需要 benchmark 数据用于回归测试 · 在归档代码上重跑：

```bash
cd docs/spikes/code/SPIKE-01
pnpm install
pnpm tauri build
./scripts/measure-boot-macos.sh 10 | tee measurements-rerun-$(date +%Y%m%d).txt
```

输出文件**不入 git**（个体环境数据 · 用 `.gitignore` 排除 · 或写到 `/tmp/`）· 仅作为本机参考。决策性 raw 仍以本目录为准。

## Phase B (Ubuntu) raw

待用户 Ubuntu 24 环境就绪后 · 在该环境跑 `measure-boot-ubuntu.sh` · 数据按本目录 / report §6 双归档。
