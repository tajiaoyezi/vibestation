# MVP-18 · `pane_link` 失败 fixture corpus（纯数据）

> Spec 契约源：[`docs/tasks/MVP-18-ai-aware-pane-linking.md`](../../../../../docs/tasks/MVP-18-ai-aware-pane-linking.md) §F.3 Fixture catalog（后 6 行纯文本样本）。
>
> 这些是 **parser bridge / sanitize** 测试消费的真实形态失败输出样本。文件名是 spec §F.3 固定契约，消费方按 `<fixture-name>.txt` path 读取，**名错即契约破**。前 4 个 link fixture（`pane_link_same_workspace` 等）是 §F.1 typed Rust 函数（依赖 core 类型），不在本目录范围。

## Manifest

| Fixture                      | §F.3 required fields（实际覆盖）                                                                                                         | 服务的 §C/§E acceptance                                                                | 形态来源                         | 安全声明                                                                                                                    |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | -------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `pane_failure_rustc.txt`     | file + line + column + error code + message（6 条 `error[E0xxx]` issue：E0425/E0308/E0599/E0061/E0277/E0382）                            | §C.3 `parsed_issues` 归一化 · §C.5 单次 ≤20 issue 去重（6 条样本）· §C.6 fallback 不崩 | 真实 `cargo`/`rustc` stderr      | 路径全 `/workspace/...`（非本机真路径）· 无 secret · §E.4-safe by construction                                              |
| `pane_failure_vitest.txt`    | test name + assertion summary + file path                                                                                                | §C.3 parsed issue（JS test 形态）· §C.5 多 issue 去重                                  | 真实 `vitest` run 失败输出       | 路径全 `/workspace/...` 与 `tests/...` 相对 · 无 secret                                                                     |
| `pane_failure_pytest.txt`    | file stack + assertion line（`> graph.link(...)`）+ short error（`E linkgraph.CrossWorkspaceError`）                                     | §C.3 parsed issue（Python traceback 形态）· §C.5 去重                                  | 真实 `pytest` traceback          | 路径全 `/workspace/...` 与 `tests/`/`src/` 相对 · 无 secret                                                                 |
| `pane_failure_ansi_json.txt` | mixed ANSI + JSON fragments（真实 `0x1b[` ESC 字节 + `cargo --message-format=json` 行交织）                                              | §C.6 hard parser case（ANSI/JSON 混合降级）· §E.1 prompt fragment 去 ANSI              | 真实 ANSI 转义字节 + JSON 诊断行 | 含真实 `ESC` (0x1b) 控制字节 · 路径 `/workspace/...` · 无 secret                                                            |
| `pane_failure_secret.txt`    | fake token + URL + environment-looking value（`sk-FAKE…`/`AKIA`-style/`ghp_FAKE…`/url-creds/`OPENAI_API_KEY=`/`AWS_SECRET_ACCESS_KEY=`） | §E.2 `raw_excerpt`/prompt fragment secret 命中 redact + redaction count                | 真实 CI deploy 失败日志形态      | **全部假值**：`sk-FAKE…` / `FAKEPASSWORD123` / `FAKE…EXAMPLEKEY` / `*.example.test` 域 · 无真凭据 · 可识别为占位符          |
| `pane_failure_osc52.txt`     | OSC52 payload + normal text（真实 `ESC ] 52 ; c ; <base64> BEL` 序列 ×2 + 正常错误文本）                                                 | §E.1 prompt fragment 去 OSC52 控制序列                                                 | 真实 OSC52 终端剪贴板写序列      | 含真实 `ESC`(0x1b)+`BEL`(0x07) 字节 · base64 解码为无害内容（`/workspace/...` 路径 + 明显假串 `sk-FAKE-not-a-real-secret`） |

## 安全 by construction

- 所有路径为 `/workspace/...` 或项目相对路径，无本机真实路径（`/Users/...`、`/home/...`、`/private/tmp` 等均不出现）。
- 所有 secret 形态值为**可识别假占位符**：`sk-FAKE` 前缀、`FAKEPASSWORD`、`FAKE…EXAMPLEKEY`、`*.example.test` 保留域。`grep -E 'sk-[A-Za-z0-9]{8,}|AKIA[0-9A-Z]{16}|ghp_…|://[^/ ]+:[^/@ ]+@'` 排除 `FAKE|example.(test|com)|EXAMPLEKEY` 后**零命中**。
- `pane_failure_osc52.txt` 第二个 OSC52 base64 解码为 `sk-FAKE-not-a-real-secret`，刻意构造为"看似 secret 但明显假"，用于证明 §E.1 必须 strip OSC52（即使其 payload 看似敏感）。
- 与 [SPIKE-07.5](../../../../../docs/spikes/SPIKE-07.5-report.md) corpus 同一脱敏纪律（redaction-safe by construction），但本 corpus 是失败输出样本（非 CLI 协议样本）。

## 复现 / 校验

```bash
# 字段命中证明（示例）
grep -E 'error\[E[0-9]+\]' pane_failure_rustc.txt          # rustc error code
grep -F 'AssertionError' pane_failure_vitest.txt           # vitest assertion
grep -E '^E ' pane_failure_pytest.txt                      # pytest short error（行内有 E 前缀）
od -An -tx1 pane_failure_ansi_json.txt | grep -o '1b 5b'   # 真实 ANSI ESC[
od -An -tx1 pane_failure_osc52.txt | grep -oE '1b|07'      # OSC52 ESC + BEL
```

## 维护

- 新增 fixture 必须同步本 manifest 行 + spec §F.3 表。
- 改 fixture 内容前确认消费方（A2 `parser_bridge` / `sanitize` 测试）期望未被破坏。
- 文件名不可改（spec §F.3 契约 + 消费方 path 硬编码）。
