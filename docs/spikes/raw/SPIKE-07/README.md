# SPIKE-07 · Raw 数据索引

> SPIKE-07 实测原始输出（进 git · 决策证据可溯源 · spike-delivery-checklist 3 样必交之一）。
> 源码：[`docs/spikes/code/SPIKE-07/`](../../code/SPIKE-07/) · 报告：`docs/spikes/SPIKE-07-report.md`（Phase F 产出）

## 文件索引

| 文件                      | 来源命令                 | 内容                                                                                                                 | 喂                                                                           |
| ------------------------- | ------------------------ | -------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `phase-a-survey.txt`      | `cargo run --bin survey` | 全 36 样本结构画像：cli×scenario×take · rawB/cleanB/#o-events/exit/ansi%/redF · 结构标记 + 矩阵完整性 + 跨 take 方差 | Phase D 统一抽象分析（Claude 薄 vs Codex 厚的定量画像 + corpus 质量 caveat） |
| `phase-a-replay-stub.txt` | `cargo run --bin replay` | Phase A StubParser 端到端跑 36 样本：events/unrecognized/panic 汇总                                                  | Phase A 验收点（管道连贯 · 0 panic · StubParser 100% unrec 符合预期）        |

Phase B/C/D raw（真 adapter replay 输出 / 准确率数据表 / IR 差异清单）后续追加 · 本 README 同步扩表。

## Phase A 关键数字溯源

- 样本数 = 36（survey 第 3 行 `# 样本数 = 36`）· fixture 集成 test `loads_real_corpus_36_complete_matrix` 断言 2 CLI × 6 场景 × 3 take
- Claude 薄协议：`phase-a-survey.txt` claude/happy_path rawB 196–238 · claude/auth_fail rawB 127 exit 1
- Codex 厚结构：codex/happy_path 行 structural markers 含 session-id/role-line/hook/tokens-used
- corpus 质量 caveat：claude/interrupt_residual exit 0（survey exit 列）· 跨 take spread（survey "矩阵完整性 + 跨 take rawB 方差" 段）
- 管道连贯：`phase-a-replay-stub.txt` 末行 `panics=0` + `✓ 管道连贯 · 0 panic`
