# SPIKE-06 · Raw Samples Archive

> 本目录存 **脱敏后派生样本**（进 git）。原始未脱敏捕获在本机 home 路径 · 不进 repo。

## 归档原则（按 spec §A.5.1）

| 物料 | 位置 | 状态 |
|---|---|---|
| **脱敏后样本**（进 git） | 本目录（`docs/spikes/raw/SPIKE-06/*.txt`） | ✅ |
| **原始未脱敏捕获** | `~/.vibestation-spike-raw/SPIKE-06/*.raw`（home · repo worktree 外 · gitignore 天然覆盖不到 · `.raw` 后缀防误 commit） | ✅ |
| **冷备 / debug 产出** | `spike-tmp/archive/SPIKE-06/`（gitignored · 含 build 产物 / 失败捕获） | ✅ placeholder |

## 命名规则

派生文件 **禁止** 以 `.raw` 结尾（会被 `.gitignore` 拦截）· 用 `.txt` / `.log`：

```
{scenario}-{N}.txt          # 脱敏后样本（N = 01/02/03）
{scenario}-{N}.redaction.json  # 脱敏元数据（PR 2 加）
```

## 当前内容（PR 1 · 起手 · 2026-04-19）

| 样本 | 来源 raw | 场景 | 说明 |
|---|---|---|---|
| `claude-version-{01,02,03}.txt` | `~/.vibestation-spike-raw/SPIKE-06/claude-version-{01,02,03}.raw` | smoke | Claude CLI `--version`（零敏感 · 只含版本号） |
| `codex-version-{01,02,03}.txt` | `~/.vibestation-spike-raw/SPIKE-06/codex-version-{01,02,03}.raw` | smoke | Codex CLI `--version`（零敏感 · 只含版本号） |

36 样本完整矩阵留 **PR 2**（下 session）· 按 spec §A.2 要求：

| CLI | 场景 × 3 次 |
|---|---|
| Claude | happy · interrupt-residual · auth-fail · network-error · long-stream · mixed-ansi |
| Codex | 同上 |

## 脱敏元数据（PR 2 增强）

本 PR 1 的 smoke 是零敏感 · 脱敏 regex 命中数 = 0 · 不需要 `.redaction.json`。

PR 2 每个 `.txt` 附 `.redaction.json`：

```json
{
  "source_raw": "~/.vibestation-spike-raw/SPIKE-06/<scenario>-<N>.raw",
  "source_sha256": "<64-char>",
  "redacted_fields": ["email", "anthropic_key", "local_path"],
  "replacement_strategy": "regex · structure preserved · values lost",
  "gitleaks_scan": "zero-hit @ 2026-04-19",
  "gitleaks_version": "<v>"
}
```

## 复现

见 [`../../code/SPIKE-06/README.md`](../../code/SPIKE-06/README.md) 复现命令段。
