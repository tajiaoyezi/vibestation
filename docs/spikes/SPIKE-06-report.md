# SPIKE-06 · Phase A Report（PR 2 完整样本归档 · 2026-04-20）

> **Task spec**：[`docs/tasks/SPIKE-06-cli-protocol-and-codesign.md`](../tasks/SPIKE-06-cli-protocol-and-codesign.md) · status: ready

> **阶段结论**：**§A 完成交付 · 36 条样本已录制 / 脱敏 / 扫描归档 · R1 保留**

> **PR 1**：Claude Code 交付 harness + smoke

> **PR 2**：Codex CLI 交付 36 条实机样本归档（包含 **Codex CLI self-record**，潜在利益冲突需 reviewer 明示知晓）

> **Review**：待主 agent + Arbiter 复核

> **phase-4-infra-landing 依赖已满足**：PR #11 merged @ 2026-04-18

---

## 测试环境

| 项 | 值 |
|---|---|
| OS | macOS Darwin 25.3.0 |
| Claude CLI | `/Users/USER/.local/bin/claude` · `2.1.114 (Claude Code)` |
| Codex CLI | `/Users/USER/.nvm/versions/node/v24.14.1/bin/codex` · `codex-cli 0.121.0` |
| asciinema | `3.2.0` |
| gitleaks | `8.30.1` |
| worktree | `/private/tmp/spike-06-pr2-work` |
| 原始 raw | `~/.vibestation-spike-raw/SPIKE-06/*.cast.raw` |
| 冷备 | `spike-tmp/archive/SPIKE-06-pr2/` |

---

## §A.1 · Pre-flight Check

| 依赖 | 状态 | 备注 |
|---|---|---|
| `.github/workflows/secret-scan.yml` | ✅ | main 已存在 |
| `docs/BRANCH-PROTECTION.md` | ✅ | main 已存在 |
| `gitleaks` | ✅ `8.30.1` | 本地实跑 |
| `asciinema` | ✅ `3.2.0` | 本地实跑 |
| Claude CLI 可用 | ✅ | happy / error / stream 均实跑 |
| Codex CLI 可用 | ✅ | happy / error / stream 均实跑 |
| Raw 路径 `~/.vibestation-spike-raw/SPIKE-06/` | ✅ | repo 外 |
| 冷备 `spike-tmp/archive/SPIKE-06-pr2/` | ✅ | gitignored |

---

## §A.2 · 样本录制

### 矩阵结论

- **Claude CLI**：6 场景 × 3 次 = 18 条
- **Codex CLI**：6 场景 × 3 次 = 18 条
- **合计**：36 条原始 cast + 36 条 redacted cast

归档位置：

- 原始：`~/.vibestation-spike-raw/SPIKE-06/*.cast.raw`
- 脱敏：`docs/spikes/raw/SPIKE-06/*.redacted.cast`
- 元数据：`docs/spikes/raw/SPIKE-06/*.redaction.json`

### 场景说明

| 场景 | Claude | Codex | 说明 |
|---|---|---|---|
| happy path | ✅×3 | ✅×3 | 成功单轮请求 |
| interrupt residual | ✅×3 | ✅×3 | 输出过程中 `Ctrl+C` |
| auth fail | ✅×3 | ✅×3 | 错误 token / API key |
| network error | ✅×3 | ✅×3 | 真实连接失败 / reconnect / debug log |
| long stream | ✅×3 | ✅×3 | 10k+ 中文科幻摘要 |
| mixed ANSI / structured | ✅×3 | ✅×3 | ANSI banner + JSON / JSON-line |

### 录制实现说明

- **未修改 repo 内 harness**：按硬约束保持 `docs/spikes/code/SPIKE-06/harness/` 原样。
- **录制 orchestration** 放在冷备目录：`spike-tmp/archive/SPIKE-06-pr2/scripts/`
- **repo 内 harness 实际复用面**：
  - `redact.py`：用于脱敏
  - `verify.sh`：用于扫描流程对齐

---

## §A.3 · 结构观察报告（描述性 · 非协议结论）

> **注意**：以下仅是样本结构观察。**本 PR 不声称 R1 已消除 / 已下调 / 协议已清楚。** 本 PR 只能得出“样本已录制，协议特征化待 SPIKE-07 parser spike”。

### Claude CLI

- `happy_path` 的 `claude -p` 输出偏 **单块 text + terminal teardown ANSI**。
- `interrupt_residual` / `long_stream` / `mixed_ansi_json` 使用 `--verbose --output-format stream-json --include-partial-messages` 时，主体是 **JSON-line event stream**。
- 事件类型可见：
  - `hook_started`
  - `hook_response`
  - `init`
  - `status`
  - `stream_event`
  - `assistant`
  - `result`
- `stream_event.content_block_delta` 会持续吐 `text_delta`，适合作为 SPIKE-07 的回放 fixture。
- `network_error` 在 `--debug-file` 路径下可见 **重复 API attempt + Connection error + aborted request** 日志。

### Codex CLI

- `codex exec` 默认模式前置 **banner + workdir/model/provider/sandbox** 段，然后进入 user prompt，再输出结果。
- `codex exec --json` 输出是较简洁的 **JSONL event stream**：
  - `thread.started`
  - `turn.started`
  - `item.completed`
  - `turn.completed`
- `interrupt_residual` 的顶层交互模式 `codex --no-alt-screen <prompt>` 会输出大量 **ANSI/TUI redraw 序列**，中断后可见残帧。
- `network_error` 呈现为 **timestamped ERROR line + reconnect counter**，与 Claude 的 debug-log 风格明显不同。

### 粗略差异

| 维度 | Claude CLI | Codex CLI |
|---|---|---|
| 成功路径默认形态 | text block 或 stream-json | banner + text block |
| 结构化模式 | `stream-json` 事件更细 | `--json` 事件更少 |
| 中断样本 | JSON-line 中途截断 + `^C` | TUI/ANSI 残帧 + `turn interrupted` |
| 错误样本 | debug log / API retry / connection error | websocket error / reconnect counter |

**结论边界**：

- ✅ 已获得 36 条真实样本，可供 SPIKE-07 使用
- ❌ 不声称两 CLI 已可统一抽象
- ❌ 不声称 R1 已下调

---

## §A.4 · 4 样归档

| 物料 | 位置 | 状态 |
|---|---|---|
| report | `docs/spikes/SPIKE-06-report.md` | ✅ |
| code | `docs/spikes/code/SPIKE-06/harness/` | ✅（本 PR 未改 harness） |
| raw（脱敏后） | `docs/spikes/raw/SPIKE-06/` | ✅ 36 条 redacted cast + 36 metadata |
| cold backup | `spike-tmp/archive/SPIKE-06-pr2/` | ✅（gitignored） |

说明：

- 原始 `.cast.raw` 全部在 home 路径保留，未进 repo。
- 冷备目录额外保存了本次本地 orchestration 脚本与日志。

---

## §A.5 · 脱敏 + gitleaks

### A.5.1 · Raw 隔离

- 原始文件命名：`*.cast.raw`
- 存储位置：`~/.vibestation-spike-raw/SPIKE-06/`
- repo 内未提交任何 raw 原始捕获

### A.5.2 · 脱敏策略

实际脱敏由 `docs/spikes/code/SPIKE-06/harness/redact.py` 完成，并在本地批量脚本里补做两类匿名化：

- `/Users/leaf/*` → `/Users/USER/*`
- `github.com/tajiaoyezi/vibestation` → `github.com/USER/REPO`

保留结构的替换：

- API key：前缀保留，值替换
- JWT：保留 3 段结构占位
- Bearer：保留 header 形态
- 本地路径：保留 `/Users/USER` / `/home/USER`

### A.5.3 · gitleaks

执行命令：

```bash
gitleaks detect --source docs/spikes/raw/SPIKE-06 --verbose
```

结果：

```text
9:32PM INF 72 commits scanned.
9:32PM INF scanned ~6128758 bytes (6.13 MB) in 534ms
9:32PM INF no leaks found
```

### A.5.4 · 真实性与结构平衡

- `auth_fail` 保留了真实错误路径和错误格式
- `network_error` 保留了真实 retry / reconnect / connection error 文本
- `mixed_ansi_json` 保留 ANSI 控制字符和结构化事件流并存
- 敏感值本身均已替换为占位，不保留真实 token / home path / repo remote

---

## §B · Apple Developer Program

本 PR **未执行**。仍保持 spec 原状态：外部资源阻塞，与 §A 样本录制无耦合。

---

## §C · SPIKE-07 依赖

- SPIKE-07 仍是下游 parser spike
- 本 PR 的 36 条样本已可作为 SPIKE-07 fixture 输入
- **R1 保留**：是否能统一抽象 / 是否足以指导实现，仍待 SPIKE-07

---

## R1 风险陈述（硬边界）

**本 PR 不下调 R1。**

- 可以说：**样本已录制 / 已脱敏 / 已归档 / 已 zero-hit gitleaks**
- 不可以说：**协议已清楚 / R1 已消除 / R1 已下调**

当前只得出：

> 已获得 36 条真实样本，足以支撑 SPIKE-07 parser-oriented 特征化工作；R1 保留。
