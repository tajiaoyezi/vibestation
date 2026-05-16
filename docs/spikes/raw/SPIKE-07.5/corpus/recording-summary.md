# SPIKE-07.5 · Phase 1 录制汇总（2026-05-16 · 36 结构化 corpus）

> 录制脚本 `docs/spikes/code/SPIKE-07.5/tools/record_corpus.sh`（§E.9 可复现）· 脱敏 `tools/redact.py`（SPIKE-06 纪律 · UUID/path/secret → 占位 + `.redaction.json` sidecar）· 36 jsonl + 36 sidecar 进 git。
> 环境：claude 2.1.142 · codex 0.130.0 · macOS 26.3.1（同 SPIKE-07 report §测试环境）。

## 录制结果（6 场景 × 2 CLI × 3 take = 36）

| 场景               | claude                                                                             | codex                                                                                          | 备注                                                                                                                                             |
| ------------------ | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| happy_path         | exit 0 · 56 事件 · `assistant{content}`+`result/success`                           | exit 0 · 4-7 · `thread.started`+`turn`+`item.completed{agent_message}`+`turn.completed{usage}` | ✅ 两 CLI 干净结构化                                                                                                                             |
| long_stream        | exit 0 · 57                                                                        | exit 0 · 4-7                                                                                   | ✅ 结构化（codex 事件少 · 内容在单 item）                                                                                                        |
| mixed_ansi_json    | exit 0 · 57-130                                                                    | exit 0 · 4                                                                                     | ✅ 结构化外层干净 · ANSI/JSON 在 content text                                                                                                    |
| auth_fail          | **exit 1 · 25 · `result{is_error:true, api_error_status:401, "Invalid API key"}`** | exit 0 · 6-7 · **退化**                                                                        | claude ✅ 干净 auth error；codex ❌ 忽略 `OPENAI_API_KEY`（用 ChatGPT OAuth backend）→ 正常跑（含 `command_execution`）· best-effort 限制        |
| network_error      | **exit 1 · 35 · `api_retry×10`+`result{is_error:true,"ConnectionRefused"}`**       | exit 0 · 10-13 · 部分                                                                          | claude ✅ 干净 network error；codex 忽略 `OPENAI_BASE_URL` · take1 撞真实环境网络抖动（`error:Reconnecting tls handshake eof`）后恢复 · 混合信号 |
| interrupt_residual | 23 事件（SIGTERM 4s 截断 · 部分结构化 · 无悬空）                                   | 2 事件（`thread.started`+`turn.started` · SIGTERM 早杀 · 优雅）                                | ✅ 重定义场景（spec §C.1）· 优雅截断验证                                                                                                         |

## decision-grade 发现

1. **核心前提决定性确认**：两 CLI 结构化模式均发干净 JSON 事件（claude `type`/`subtype` · codex `type`）· 与 SPIKE-06 TUI blob 天壤之别 · **SPIKE-07 §H 路径 3 deferred = corpus 方法论 artifact 实锤**
2. **claude error 场景完美**：auth_fail / network_error 均产结构化 `result{is_error:true}` + 明确 message（401/ConnectionRefused）· 完美映射 `Error{kind}` IR
3. **codex auth/network best-effort 退化**（spec §E fail #2 / risks #1 已预案）：codex 忽略 `OPENAI_API_KEY`/`OPENAI_BASE_URL` env（用 ChatGPT OAuth backend `chatgpt.com/backend-api/codex`）· 无法用 env 注入构造 codex 错误态 → 这 6 个 codex auth/network 样本是**退化样本**（非 parser 缺陷 · corpus 构造限制）· §H 判定须显式排除或标注（同 SPIKE-07 Phase D 对退化 corpus 的处理纪律）
4. **codex `command_execution` item**（agentic tool-use · SPIKE-07 IR 未覆盖此 item type）→ SPIKE-07.5 parser 须映射 `ToolUseStart/End` 或 `Unrecognized` · Phase D 分析点
5. **claude stream-json 多行 JSON 事件**（关键 parser 设计输入）：`system/hook_response` 的 `output` 字段含字面换行（SessionStart hook 多行内容）· 单逻辑事件跨多物理行（936 行中 184 是续行）→ **`.structured.jsonl` loader 必须流式累积完整 JSON（非 naive 行分割）** · 这是真实协议鲁棒性测试点（SPIKE-07.5 价值之一）

## 对 Phase 2 (crate) 的指导

- loader：流式 JSON 累积（buffer until `json.loads` 成功）· 非逐行 · 容错跳坏块（同 SPIKE-07 cast.rs 容错纪律）
- claude parser：`type=system/init`→SessionMeta（含 `session_id`）· `hook_*`→Hook · `assistant.message.content[].text`→MessageDelta · `result{is_error,result,api_error_status}`→MessageEnd 或 Error{Auth(401)/Network(ConnectionRefused)}
- codex parser：`thread.started{thread_id}`→SessionMeta · `item.completed{agent_message}`→MessageDelta · `command_execution`→ToolUse\*/Unrecognized · `turn.completed{usage}`→Usage+MessageEnd · `error`→Error{Network}
- §H 判定：codex auth/network 6 退化样本须标注（corpus 构造限制 · 非 parser）· 同 SPIKE-07 Phase D 三类根因纪律

## 置信度 caveat

- codex 错误场景 corpus 构造受限（OAuth backend 无视 env override）· codex error-event 解析准确率本批次无法公平评估 · §H 须显式处理（路径判定排除退化样本或单独标注）
- network_error take 撞真实环境网络抖动（VPN）· 含真实 `error` 事件但混合正常恢复 · 实测如实记录不修饰
