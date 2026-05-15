# SPIKE-07 · Raw 数据索引

> SPIKE-07 实测原始输出（进 git · 决策证据可溯源 · spike-delivery-checklist 3 样必交之一）。
> 源码：[`docs/spikes/code/SPIKE-07/`](../../code/SPIKE-07/) · 报告：[`docs/spikes/SPIKE-07-report.md`](../../SPIKE-07-report.md)
> 本 spike **不复制 corpus**（直接复用 `docs/spikes/raw/SPIKE-06/` 36 条 `*.redacted.cast`）· 本目录只存 parser 实跑产出。

## 文件索引

| 文件                         | 阶段             | 来源命令（在 `docs/spikes/code/SPIKE-07/`）         | 内容                                                                                                                                  |
| ---------------------------- | ---------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `phase-a-survey.txt`         | A                | `cargo run --bin survey`                            | 全 36 样本结构画像：cli×scenario×take · rawB/cleanB/#o-events/exit/ansi%/redF + 矩阵完整性                                            |
| `phase-a-replay-stub.txt`    | A                | `cargo run --bin replay`（Phase A 版）              | Phase A StubParser 端到端：events/unrecognized/panic 汇总（验收点 0 panic）                                                           |
| `phase-c-matrix.md`          | C                | `cargo run --bin matrix`                            | §F 测试矩阵 markdown 全量：场景/CLI/整体正确率 + §E.11 基线 + 36 逐样本 + 12 FAIL 明细                                                |
| `phase-c-matrix.json`        | C                | `SPIKE07_JSON=<path> cargo run --bin matrix`        | 同上机器可读（report 每数字溯源此文件）                                                                                               |
| `path-a-cli-modes-recon.txt` | E（路径 A 调研） | `claude --help` / `codex --help` 实测（2026-05-16） | 两 CLI 结构化输出模式实测证据（claude `--output-format stream-json` · codex `exec`）· 证 deferred = corpus 方法论 artifact 非前提推翻 |

## phase-c-matrix.json 字段索引（report 数字溯源 · §E.6）

| report / ADR-017 引用                    | JSON 字段                                                                   |
| ---------------------------------------- | --------------------------------------------------------------------------- |
| 整体 PASS 24/36 = 66.7% · 0 panic        | `overall_pass` / `total_samples` / `overall_accuracy` / `panics`            |
| 场景级正确率（happy/auth/long/mixed …）  | `per_scenario.{scenario}.{pass,total,accuracy,mean_unrecognized}`           |
| CLI 级正确率（claude/codex 各 67%）      | `per_cli.{claude,codex}.{pass,total,accuracy}`                              |
| §E.11 基线（parser 97 / kw 69 / lp 83）  | `baseline.{parser_errdetect,keyword_baseline,lineprefix_baseline}_accuracy` |
| 12 FAIL 明细（如 `content 0/142588=0%`） | `rows[].assessment.checks[].{name,passed,detail}`                           |
| 逐样本 events/unrec/panic/pass           | `rows[].{cli,scenario,take,events,unrecognized_ratio,panicked,sample_pass}` |

## Phase A 关键数字溯源

- 样本数 = 36（survey `# 样本数 = 36`）· fixture 集成 test `loads_real_corpus_36_complete_matrix` 断言 2 CLI × 6 场景 × 3 take
- Claude 薄协议：claude/happy_path rawB 196–238 · claude/auth_fail rawB 127 exit 1
- Codex 厚结构：codex/happy_path structural markers 含 session-id/role-line/hook/tokens-used
- corpus 质量 caveat：claude/interrupt_residual exit 0 · 跨 take rawB 方差（survey "矩阵完整性"段）
- 管道连贯：`phase-a-replay-stub.txt` 末 `panics=0`

## Phase C/D 结论速览（详见 report + ADR-017）

§H 路径 3 · **deferred**（R1 保留 HIGH/HIGH · 不降级）· **非 parser bug**（happy/auth/network/interrupt 4/6 场景 100% · 0 panic · 统一 IR 抽象可行 §E.4 正面）· deferral 根因 = corpus 质量（claude long_stream = TUI 屏幕重绘 blob）+ 协议现实（两 CLI 不发机器 JSON）+ §F 厚协议校准失配（codex long_stream 差 2-3pp）。ADR-017 proposed（待独立评审 + Arbiter 拍板）。

## 复现（byte-level · §E.9）

```bash
cd docs/spikes/code/SPIKE-07
cargo test                                       # 39 tests · 含 36-corpus 完整矩阵集成 test
cargo run --bin matrix                           # → phase-c-matrix.md（stdout · 确定性 · 跑 3 次一致）
SPIKE07_JSON=/tmp/x.json cargo run --bin matrix  # 另写 JSON
```

环境：cargo/rustc `1.95.0` · macOS `26.3.1`（详见 report §测试环境）。parser 无随机性 · 同样本多跑结果一致。
