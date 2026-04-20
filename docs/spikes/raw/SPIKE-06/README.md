# SPIKE-06 · Raw Samples Archive

> 本目录存放 **脱敏后的派生样本**（进 git）。
> 原始未脱敏捕获保留在本机 `~/.vibestation-spike-raw/SPIKE-06/`。
> 冷备保留在 `spike-tmp/archive/SPIKE-06-pr2/`（gitignored）。

## 归档位置

| 物料 | 位置 | 状态 |
| --- | --- | --- |
| 脱敏后 cast | `docs/spikes/raw/SPIKE-06/*.redacted.cast` | ✅ 36 条 |
| 脱敏元数据 | `docs/spikes/raw/SPIKE-06/*.redaction.json` | ✅ 36 条 |
| 原始 cast | `~/.vibestation-spike-raw/SPIKE-06/*.cast.raw` | ✅ 36 条 |
| 冷备 / build log | `spike-tmp/archive/SPIKE-06-pr2/` | ✅ |

## 命名约定

```text
<cli>_<scenario>_<n>.cast.raw
<cli>_<scenario>_<n>.redacted.cast
<cli>_<scenario>_<n>.redaction.json
```

场景枚举：

- `happy_path`
- `interrupt_residual`
- `auth_fail`
- `network_error`
- `long_stream`
- `mixed_ansi_json`

## 矩阵摘要

| CLI | 场景 | 次数 |
| --- | --- | --- |
| Claude CLI | happy / interrupt / auth / network / long / mixed | 18 |
| Codex CLI | happy / interrupt / auth / network / long / mixed | 18 |
| 合计 | 2 CLI × 6 场景 × 3 次 | 36 |

## 36 条样本清单

### Claude CLI

```text
claude_happy_path_1.redacted.cast          macOS  2026-04-20 20:49:12
claude_happy_path_2.redacted.cast          macOS  2026-04-20 21:00:13
claude_happy_path_3.redacted.cast          macOS  2026-04-20 21:18:28
claude_interrupt_residual_1.redacted.cast  macOS  2026-04-20 20:53:01
claude_interrupt_residual_2.redacted.cast  macOS  2026-04-20 21:00:32
claude_interrupt_residual_3.redacted.cast  macOS  2026-04-20 21:26:19
claude_auth_fail_1.redacted.cast           macOS  2026-04-20 20:49:27
claude_auth_fail_2.redacted.cast           macOS  2026-04-20 21:00:35
claude_auth_fail_3.redacted.cast           macOS  2026-04-20 21:26:23
claude_network_error_1.redacted.cast       macOS  2026-04-20 20:55:21
claude_network_error_2.redacted.cast       macOS  2026-04-20 21:00:48
claude_network_error_3.redacted.cast       macOS  2026-04-20 21:26:36
claude_long_stream_1.redacted.cast         macOS  2026-04-20 20:56:39
claude_long_stream_2.redacted.cast         macOS  2026-04-20 21:08:44
claude_long_stream_3.redacted.cast         macOS  2026-04-20 21:31:02
claude_mixed_ansi_json_1.redacted.cast     macOS  2026-04-20 20:51:33
claude_mixed_ansi_json_2.redacted.cast     macOS  2026-04-20 21:09:02
claude_mixed_ansi_json_3.redacted.cast     macOS  2026-04-20 21:31:18
```

### Codex CLI

```text
codex_happy_path_1.redacted.cast           macOS  2026-04-20 20:51:50
codex_happy_path_2.redacted.cast           macOS  2026-04-20 21:09:37
codex_happy_path_3.redacted.cast           macOS  2026-04-20 21:20:35
codex_interrupt_residual_1.redacted.cast   macOS  2026-04-20 21:15:31
codex_interrupt_residual_2.redacted.cast   macOS  2026-04-20 21:16:08
codex_interrupt_residual_3.redacted.cast   macOS  2026-04-20 21:20:59
codex_auth_fail_1.redacted.cast            macOS  2026-04-20 20:54:53
codex_auth_fail_2.redacted.cast            macOS  2026-04-20 21:16:51
codex_auth_fail_3.redacted.cast            macOS  2026-04-20 21:21:41
codex_network_error_1.redacted.cast        macOS  2026-04-20 20:55:16
codex_network_error_2.redacted.cast        macOS  2026-04-20 21:17:04
codex_network_error_3.redacted.cast        macOS  2026-04-20 21:21:54
codex_long_stream_1.redacted.cast          macOS  2026-04-20 20:58:14
codex_long_stream_2.redacted.cast          macOS  2026-04-20 21:19:40
codex_long_stream_3.redacted.cast          macOS  2026-04-20 21:24:32
codex_mixed_ansi_json_1.redacted.cast      macOS  2026-04-20 20:58:31
codex_mixed_ansi_json_2.redacted.cast      macOS  2026-04-20 21:19:57
codex_mixed_ansi_json_3.redacted.cast      macOS  2026-04-20 21:24:47
```

## Historical Smoke（PR 1）

以下 6 条 `.txt` 为 2026-04-19 的零敏感 smoke 样本：

- `claude-version-01.txt`
- `claude-version-02.txt`
- `claude-version-03.txt`
- `codex-version-01.txt`
- `codex-version-02.txt`
- `codex-version-03.txt`

## 脱敏与扫描结果

脱敏策略：

- 使用 repo 内 `docs/spikes/code/SPIKE-06/harness/redact.py`
- 本地 orchestration 脚本位于
  `spike-tmp/archive/SPIKE-06-pr2/scripts/redact_batch.py`
- 占位保留结构，敏感值丢失：
  JWT / API key / Bearer / 本地路径 / GitHub remote / 邮箱

`gitleaks` 结果（2026-04-20 · `gitleaks 8.30.1`）：

```text
9:32PM INF 72 commits scanned.
9:32PM INF scanned ~6128758 bytes (6.13 MB) in 534ms
9:32PM INF no leaks found
```

执行命令：

```bash
gitleaks detect --source docs/spikes/raw/SPIKE-06 --verbose
```

## 复现

本 PR 实际使用的本地 orchestration 命令：

```bash
cd /private/tmp/spike-06-pr2-work
spike-tmp/archive/SPIKE-06-pr2/scripts/run_recordings.sh claude happy 1
spike-tmp/archive/SPIKE-06-pr2/scripts/run_recordings.sh codex network_error 1
spike-tmp/archive/SPIKE-06-pr2/scripts/redact_batch.py
```

repo 内可复用的 harness 入口：

```bash
cd docs/spikes/code/SPIKE-06/harness
./redact.py \
  --input ~/.vibestation-spike-raw/SPIKE-06/claude_happy_path_1.cast.raw \
  --output ../../raw/SPIKE-06/claude_happy_path_1.redacted.cast
./verify.sh ../../raw/SPIKE-06
```
