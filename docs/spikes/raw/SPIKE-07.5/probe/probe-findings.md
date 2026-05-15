# SPIKE-07.5 · Phase 0 Probe 实测结论（2026-05-16）

> 决定性 de-risk 检查点：录全量 36 corpus 前，先用最小 prompt（"Reply with exactly the two words: probe ok"）实测两 CLI 结构化模式真实输出格式 · 验 SPIKE-07.5 核心前提（fail signal #1：结构化模式实际不发可解析事件）。
> 证据：`claude_streamjson_probe.jsonl`（58 行）· `codex_exec_json_probe.jsonl`（4 行）· 均进 git。

## 结论：核心前提 probe 级**决定性验证** · fail signal #1 **不触发**

两 CLI 结构化模式均输出**干净 JSON-lines 机器协议**（非 SPIKE-06 的 TUI 屏幕重绘 blob）· trivially 映射 SPIKE-07 `CliEvent` IR。**SPIKE-07 §H 路径 3 deferred 100% 确认为 corpus 方法论 artifact**（SPIKE-06 录了交互 TUI · 非这两个结构化模式）。

## 实测命令（probe 发现的正确录制命令 · 录前定型）

| CLI    | 命令                                                                                                                    | 关键 flag 发现                                                                                                           |
| ------ | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Claude | `claude -p --output-format stream-json --verbose --include-hook-events --no-session-persistence "<prompt>" < /dev/null` | `--output-format stream-json` 强制要求 `--verbose`（否则 `Error: requires --verbose`）· `< /dev/null` 跳过 stdin 3s 等待 |
| Codex  | `codex exec --json "<prompt>" < /dev/null`                                                                              | `codex exec` 默认仅输出最终答案纯文本 · `--json`="Print events to stdout as JSONL" 才出结构化事件流                      |

## Claude `stream-json` 事件结构（58 事件 · 全 JSON）

type/subtype 直方图：`system/hook_started ×26` · `system/hook_response ×26` · `system/api_retry ×2` · `system/init ×1` · `assistant ×1` · `rate_limit_event ×1` · `result/success ×1`

| 事件                                  | 样例                                                       | → CliEvent IR 映射                            |
| ------------------------------------- | ---------------------------------------------------------- | --------------------------------------------- |
| `system/init`                         | keys: cwd/session_id/tools/model/permissionMode/version... | SessionMeta（多条 key:value · 含 session_id） |
| `system/hook_started`·`hook_response` | hook_name/hook_event/outcome/exit_code                     | Hook{name, completed}                         |
| `assistant`                           | `message.content=[{"type":"text","text":"probe ok"}]`      | MessageStart{Assistant}+MessageDelta{text}    |
| `result/success`                      | `result:"probe ok"`                                        | MessageEnd{Stop}                              |
| `rate_limit_event`·`api_retry`        | attempt/max_retries/error_status                           | （诊断 · 可 Unrecognized 或专门事件）         |

每事件均带 `session_id` + `uuid` —— **结构化模式 claude 显式发 session_id**（SPIKE-07 TUI 模式 claude **不发** · §H 统一抽象关键差异点本质改善）。

## Codex `exec --json` 事件结构（4 事件 · 全 JSON）

| 事件             | 样例                                                                            | → CliEvent IR 映射                         |
| ---------------- | ------------------------------------------------------------------------------- | ------------------------------------------ |
| `thread.started` | `{"thread_id":"019e2e08-..."}`                                                  | SessionMeta{key:"thread_id"}               |
| `turn.started`   | `{}`                                                                            | （turn 边界）                              |
| `item.completed` | `{"item":{"type":"agent_message","text":"probe ok"}}`                           | MessageStart{Assistant}+MessageDelta{text} |
| `turn.completed` | `{"usage":{"input_tokens":..,"output_tokens":..,"reasoning_output_tokens":..}}` | Usage{tokens}+MessageEnd{Stop}             |

## 对 SPIKE-07.5 实施的指导

1. 录制命令已定型（上表）· 6 场景 × 2 CLI × 3 take = 36
2. `.structured.jsonl` loader：按行 `json.loads` · 非 asciinema `.cast`（确认 NIT-1 的 cast.rs/fixture.rs 不可复用判断正确 · 须新写 loader）
3. 结构化 parser：claude 按 `type`/`subtype` 路由 · codex 按 `type` 路由 → 复用 SPIKE-07 `CliEvent` IR + `assertions.rs`（format-agnostic）
4. `interrupt_residual` 场景：`-p`/`exec` 非交互 · 按 spec §C.1 重定义（SIGTERM 截断 · 验优雅处理 · 非 TUI 残帧）

## 置信度 caveat

probe = 单次最小 prompt · 仅验"结构化模式发干净 JSON 事件"这一前提（已决定性成立）· **不**代表全 36 场景准确率（happy/auth/network/long/mixed/interrupt 各场景实测在 Phase C 矩阵）。probe 期间 claude 出现 2 次 `api_retry`（环境瞬时 API 抖动 · 结构化事件仍正常发出 · 不影响格式结论）。
