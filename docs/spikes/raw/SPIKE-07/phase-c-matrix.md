# SPIKE-07 Phase C · §F 测试矩阵实测结果

样本总数 **36** · panic **0** · 整体 PASS **24/36** = **66.7%**

## 场景级正确率（§F 矩阵 · single-source = §H · 本表 informative）

| 场景 | PASS/total | 正确率 | 平均 Unrecognized |
| --- | --- | --- | --- |
| auth_fail | 6/6 | 100% | 0% |
| happy_path | 6/6 | 100% | 0% |
| interrupt_residual | 6/6 | 100% | 100% |
| long_stream | 0/6 | 0% | 50% |
| mixed_ansi_json | 0/6 | 0% | 100% |
| network_error | 6/6 | 100% | 11% |

## CLI 级正确率

| CLI | PASS/total | 正确率 |
| --- | --- | --- |
| claude | 12/18 | 67% |
| codex | 12/18 | 67% |

## §E.11 基线对比（error-detection · parser vs 廉价启发式）

| 方法 | 准确率 | 说明 |
| --- | --- | --- |
| Parser（Error 事件） | 97% | 结构化解析 |
| 基线 A 关键字扫描 | 69% | exit≠0 / error/unauthorized/failed 子串 |
| 基线 B 行首启发式 | 83% | `^Error:` / `^WARNING:` |

> Parser − 最优基线 = **+14pp**（§E.11 要求 parser 显著优于基线 +20pp 才值复杂度）

## 逐样本矩阵（36 条 · 每条 §F 断言）

| # | CLI | 场景 | take | events | unrec | panic | 断言 (pass/total) | 样本 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | claude | auth_fail | 1 | 2 | 0% | — | 4/4 | ✅PASS |
| 2 | claude | auth_fail | 2 | 2 | 0% | — | 4/4 | ✅PASS |
| 3 | claude | auth_fail | 3 | 2 | 0% | — | 4/4 | ✅PASS |
| 4 | claude | happy_path | 1 | 3 | 0% | — | 5/5 | ✅PASS |
| 5 | claude | happy_path | 2 | 3 | 0% | — | 5/5 | ✅PASS |
| 6 | claude | happy_path | 3 | 3 | 0% | — | 5/5 | ✅PASS |
| 7 | claude | interrupt_residual | 1 | 1 | 100% | — | 3/3 | ✅PASS |
| 8 | claude | interrupt_residual | 2 | 1 | 100% | — | 3/3 | ✅PASS |
| 9 | claude | interrupt_residual | 3 | 1 | 100% | — | 3/3 | ✅PASS |
| 10 | claude | long_stream | 1 | 1 | 100% | — | 2/3 | ❌FAIL |
| 11 | claude | long_stream | 2 | 1 | 100% | — | 2/3 | ❌FAIL |
| 12 | claude | long_stream | 3 | 1 | 100% | — | 2/3 | ❌FAIL |
| 13 | claude | mixed_ansi_json | 1 | 1 | 100% | — | 2/3 | ❌FAIL |
| 14 | claude | mixed_ansi_json | 2 | 1 | 100% | — | 2/3 | ❌FAIL |
| 15 | claude | mixed_ansi_json | 3 | 1 | 100% | — | 2/3 | ❌FAIL |
| 16 | claude | network_error | 1 | 2 | 0% | — | 4/4 | ✅PASS |
| 17 | claude | network_error | 2 | 2 | 0% | — | 4/4 | ✅PASS |
| 18 | claude | network_error | 3 | 2 | 0% | — | 4/4 | ✅PASS |
| 19 | codex | auth_fail | 1 | 13 | 0% | — | 4/4 | ✅PASS |
| 20 | codex | auth_fail | 2 | 13 | 0% | — | 4/4 | ✅PASS |
| 21 | codex | auth_fail | 3 | 13 | 0% | — | 4/4 | ✅PASS |
| 22 | codex | happy_path | 1 | 28 | 0% | — | 5/5 | ✅PASS |
| 23 | codex | happy_path | 2 | 51 | 0% | — | 5/5 | ✅PASS |
| 24 | codex | happy_path | 3 | 28 | 0% | — | 5/5 | ✅PASS |
| 25 | codex | interrupt_residual | 1 | 140 | 99% | — | 3/3 | ✅PASS |
| 26 | codex | interrupt_residual | 2 | 10 | 100% | — | 3/3 | ✅PASS |
| 27 | codex | interrupt_residual | 3 | 8 | 100% | — | 3/3 | ✅PASS |
| 28 | codex | long_stream | 1 | 28 | 0% | — | 2/3 | ❌FAIL |
| 29 | codex | long_stream | 2 | 28 | 0% | — | 2/3 | ❌FAIL |
| 30 | codex | long_stream | 3 | 28 | 0% | — | 2/3 | ❌FAIL |
| 31 | codex | mixed_ansi_json | 1 | 5 | 100% | — | 2/3 | ❌FAIL |
| 32 | codex | mixed_ansi_json | 2 | 5 | 100% | — | 2/3 | ❌FAIL |
| 33 | codex | mixed_ansi_json | 3 | 5 | 100% | — | 2/3 | ❌FAIL |
| 34 | codex | network_error | 1 | 2 | 0% | — | 4/4 | ✅PASS |
| 35 | codex | network_error | 2 | 3 | 33% | — | 4/4 | ✅PASS |
| 36 | codex | network_error | 3 | 3 | 33% | — | 4/4 | ✅PASS |

## 失败断言明细（每条 FAIL 样本的具体断言）

**#10 claude/long_stream/1**
- ❌ `long_content_95pct` — content 0/142588 = 0% < 95%
**#11 claude/long_stream/2**
- ❌ `long_content_95pct` — content 0/300091 = 0% < 95%
**#12 claude/long_stream/3**
- ❌ `long_content_95pct` — content 0/209721 = 0% < 95%
**#13 claude/mixed_ansi_json/1**
- ❌ `mixed_json_parseable` — 无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）
**#14 claude/mixed_ansi_json/2**
- ❌ `mixed_json_parseable` — 无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）
**#15 claude/mixed_ansi_json/3**
- ❌ `mixed_json_parseable` — 无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）
**#28 codex/long_stream/1**
- ❌ `long_content_95pct` — content 8529/9183 = 93% < 95%
**#29 codex/long_stream/2**
- ❌ `long_content_95pct` — content 7703/8357 = 92% < 95%
**#30 codex/long_stream/3**
- ❌ `long_content_95pct` — content 7912/8566 = 92% < 95%
**#31 codex/mixed_ansi_json/1**
- ❌ `mixed_json_parseable` — 无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）
**#32 codex/mixed_ansi_json/2**
- ❌ `mixed_json_parseable` — 无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）
**#33 codex/mixed_ansi_json/3**
- ❌ `mixed_json_parseable` — 无可解析 JSON（CLI 不发结构化 JSON events · corpus/协议现实）
