# SPIKE-02 · Raw 数据归档

对应 report：[`docs/spikes/SPIKE-02-report.md`](../../SPIKE-02-report.md)
对应代码归档：[`docs/spikes/code/SPIKE-02/`](../../code/SPIKE-02/)

## 状态：**嵌入式 raw**（无独立 JSON 文件）

同 SPIKE-01 · SPIKE-02 也是 W0 启动期 Spike（2026-04-19）· raw 数据嵌入 report 正文 · 未单独产出 JSON / log。

详细背景与处理原则见姊妹归档：[`docs/spikes/raw/SPIKE-01/README.md`](../SPIKE-01/README.md) §状态。

## 嵌入式 raw 数据位置

| 数据类型 | 位置（report 内） | 内容 |
|---|---|---|
| 10× 稳定性原始数据 | report §4.2 | 10 次启动 · 全 0 crash · `Min 187ms · Median 212ms · Max 229ms · Range 42ms` |
| Clipboard 跨 app 验证 | report §4.3 | 中文/日文/英文/emoji UTF-8 完整通过 |
| FS 读写验证 | report §4.4 | terminal `cat` 验证文件内容 一致 |
| Bundle 大小 | report §4.6 | `.app 10MB · .dmg 4MB`（`check-bundle-size.sh` 输出） |
| 中文 IME 录屏 | `spike-artifacts/SPIKE-02/macos-ime.mov` | （见 spike-artifacts 子系统） |
| 2 项降级记录 | report §结论 | updater → SPIKE-06 · 日文 IME → 全平台 skip |

## 重新测量参考

```bash
cd docs/spikes/code/SPIKE-02
pnpm install
pnpm tauri build
./scripts/measure-10x-stability-macos.sh | tee measurements-rerun-$(date +%Y%m%d).txt
./scripts/check-bundle-size.sh           | tee bundle-rerun-$(date +%Y%m%d).txt
```

输出文件**不入 git**（同 SPIKE-01 raw 处理原则）。

## Phase B (Ubuntu) raw

待 Ubuntu 24 环境就绪后回填 · 同 SPIKE-01 处理。
