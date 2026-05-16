# SPIKE-07.5 报告 · 结构化模式 CLI 协议 parser 验证（路径 A · R1 重判）

> 决策级 Spike · [ADR-017](./adr/ADR-017-ai-aware-deferred.md) 路径 A 落地 · R1（AI-Aware v1.0 vision）gate 重判前置。
> 实测环境：claude 2.1.142 · codex 0.130.0 · macOS 26.3.1 · Rust 1.95.0（同 SPIKE-07 报告测试环境）。
> 本报告每个数字均可溯源 `docs/spikes/raw/SPIKE-07.5/phase3-matrix.json`（§E.7）· 0 编造声明见文末。

---

## 0 · 一句话结论

**路径 A 假设实锤成立**：claude / codex **结构化模式**（`stream-json` / `exec --json`）输出干净行分隔 JSON 机器协议，统一 `CliEvent` IR 抽象**可行且高保真**——claude **18/18=100%（六场景全过）** · 非退化 **29/30=96.7%** · **panic 0**。SPIKE-07 §H 路径 3 deferred 被实测确认为 **SPIKE-06 corpus 方法论 artifact**（录的是交互 TUI 屏幕重绘 · 非结构化协议），与 ADR-017 推断一致。

**§H 三路径裁决（SPIKE-07 §H single source of truth · 严格字面 + 实质并陈 · Arbiter 拍板）**：见 §H。主 agent 推荐 **路径 1 greenlight（带 2 项已根因的 carve-out）**，保守回退 **路径 2（claude 无条件 greenlight）**。**本报告不自行 accept**（§2.1 · CLAUDE.md A 栏 ADR 流程）。

---

## A · 背景与方法（与 SPIKE-07 的关系）

| 维度    | SPIKE-07                                                                | SPIKE-07.5（本 spike · 路径 A）                                                                                   |
| ------- | ----------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| corpus  | SPIKE-06 36 `.redacted.cast`（asciinema · **交互 TUI 屏幕重绘**）       | 新录 36 `.structured.jsonl`（`claude -p --output-format stream-json` / `codex exec --json` · **结构化机器协议**） |
| IR 契约 | `src/ir.rs` `CliEvent` + `CliParser` trait                              | **字节级复用**（sha256 `3b23357…` 一致 · §B）                                                                     |
| §F 断言 | `src/assertions.rs`                                                     | **字节级复用**（sha256 `c91caed…` 一致 · 锁定不改 · §B / §2.1）                                                   |
| loader  | `cast.rs`（asciinema v2/v3 解码）                                       | **新写** `jsonl.rs`（逐行 `serde_json` · cast.rs 不可复用 · §B.1）                                                |
| adapter | TUI 文本启发式                                                          | **新写** `parser/claude.rs`+`codex.rs`（结构化事件路由）                                                          |
| 结果    | 24/36=66.7% · long_stream/mixed **0%** · 两 CLI 对称 67% → **deferred** | 32/36=88.9% · 非退化 29/30=**96.7%** · claude **100%** · panic 0                                                  |

方法依据：[`SPIKE-07.5` spec](../tasks/SPIKE-07.5-structured-mode-rerun.md) · [ADR-017](./adr/ADR-017-ai-aware-deferred.md) 路径 A。

---

## B · 复用完整性证明（decision-grade · 防"复用"沦为口号）

`ir.rs` / `assertions.rs` **byte-identical** 复用 SPIKE-07（非"参考重写"）：

```
sha256(ir.rs)         SPIKE-07 = SPIKE-07.5 = 3b23357fa6ec9b3d34b0304e3d06d3eab82c458d
sha256(assertions.rs) SPIKE-07 = SPIKE-07.5 = c91caed1a416c272dd6065ffcd30c943798e765b
```

含义：§F 断言逻辑（含 SPIKE-07 Phase C 修复的 `is_monotone` error-path 分支）**未被为凑 §H 而改动**。两 spike 用**同一把尺**量不同 corpus —— 这是路径 A 结论可信的前提（spec §B / 自审四问递归完备性）。

新写模块（结构化协议特有 · SPIKE-07 TUI adapter 不可复用）：`jsonl.rs`（逐行 loader · 见 §B.1）· `fixture.rs`（corpus 发现 + parser 无关 ground truth）· `parser/claude.rs`+`codex.rs`（结构化 adapter）· `bin/matrix.rs`（复用 SPIKE-07 聚合逻辑 · corpus 换结构化 · §F long_stream 分母传 parser 无关 `reference_text` 防循环论证）。

### B.1 · loader 设计的根因更正（decision-grade 诚实纪录）

Phase 1 录制汇总曾误记 finding #5："claude `hook_response.output` 多行 JSON · 936 行中 184 续行 · loader 须流式累积"。**实测证伪**：raw `/tmp/spike075-raw` 36/36 文件 936/936 行**严格一行一合法 JSON · 零多行 · 零 EOF 残尾**。184 非法行 **100% 由 `redact.py` v1 正则脱敏破坏 JSON 转义引入**（`PATH_RE` 尾随 `[^\s"]*` 吃掉嵌套转义 `\`）。

修复：`redact.py` **v2 结构保留型**（`json.loads`→递归脱敏字符串叶子→`json.dumps` 单行）· 从同日未脱敏 ground truth 重脱敏（**零重录 · 零新增 API 成本**）· corpus 现 36/36 936/936 100% 合法 · 事件 936=936 与 raw 字节级对账。∴ `jsonl.rs` 用**逐行解析**（KISS · raw 实测 0 坏行 · `JsonLine::Bad` 兜底为 SIGTERM 真截断防御性 · 不 panic 不吞后续）。根因更正全文见 `docs/spikes/raw/SPIKE-07.5/corpus/recording-summary.md` finding #5 + redact.py v2 docstring。

---

## C · Phase 3 · §F 测试矩阵实测（每数字溯源 phase3-matrix.json）

样本 **36** · panic **0** · 整体 PASS **32/36 = 88.9%** · **不含 codex auth/network 退化样本 29/30 = 96.7%**。

| 口径                                           | overall           | claude           | codex         | 关键场景                               |
| ---------------------------------------------- | ----------------- | ---------------- | ------------- | -------------------------------------- |
| raw（锁定 §F · 全 36）                         | 32/36 = **88.9%** | 18/18 = **100%** | 14/18 = 77.8% | mixed 5/6=83.3% · auth 3/6=50%         |
| 非退化（剔 codex auth/net 6 · spec §E fail#2） | 29/30 = **96.7%** | 18/18 = **100%** | 11/12 = 91.7% | claude auth 3/3=100% · mixed 5/6=83.3% |

场景级（raw / 非退化）：

| 场景               | raw         | 非退化            | 备注                                                                                                                   |
| ------------------ | ----------- | ----------------- | ---------------------------------------------------------------------------------------------------------------------- |
| happy_path         | 6/6 = 100%  | 6/6 = 100%        | claude+codex 全过                                                                                                      |
| long_stream        | 6/6 = 100%  | 6/6 = 100%        | **vs SPIKE-07 此场景 0%** · content 95% 保真（分母=parser 无关 reference_text）                                        |
| interrupt_residual | 6/6 = 100%  | 6/6 = 100%        | SIGTERM 截断在完整事件间 · 无悬空 start                                                                                |
| network_error      | 6/6 = 100%  | 3/3 = 100%        | claude `result{ConnectionRefused}`→Error{Network,recoverable}；codex 撞真实 VPN tls-eof→Error{Network}（退化但实测过） |
| auth_fail          | 3/6 = 50%   | claude 3/3 = 100% | claude `result{api_error_status:401}`→Error{Auth,!recoverable}；**codex 3/3 退化**（§D.1）                             |
| mixed_ansi_json    | 5/6 = 83.3% | 5/6 = 83.3%       | **唯一非退化 miss = #33**（§D.2 · §F 行首启发式 artifact · 非 parser 缺陷）                                            |

4 个 FAIL 全归因（FAIL rows 溯源 phase3-matrix.json `rows[].sample_pass==false`）：

- `#19/#20/#21 codex/auth_fail/1-3` —— **退化 corpus**（§D.1 · 3/4 断言过 · 仅 auth 特化断言因无错可测而 fail）
- `#33 codex/mixed_ansi_json/3` —— **§F `mixed_json_parseable` 行首启发式 artifact**（§D.2 · 非 parser 缺陷 · 非退化）

§E.11 基线对比（扫 parser 无关 `reference_text` · 公平性见 matrix.rs 注释）：parser 结构化 error-detection **91.7%** vs 关键字基线 88.9% vs 行首基线 66.7% · **+2.8pp**（< SPIKE-07 §E.11 +20pp 复杂度门槛 —— 如实记录 · 见 §F 解读）。

---

## D · Phase 4 · 失败根因（系统调试纪律 · 非凑路径）

### D.1 · codex auth/network 6 = 退化 corpus（pre-registered · 非 parser 缺陷）

codex 0.130.0 用 ChatGPT OAuth backend（`chatgpt.com/backend-api/codex`）· **物理上无视** `OPENAI_API_KEY`/`OPENAI_BASE_URL` env → 无法用 env 注入构造 codex auth/network 错误态。这是 spec §E fail#2 + risks#1 **录制前已登记**的已知限制（非 post-hoc 借口）。证据：codex auth_fail 样本无 `error` 事件 · 正常跑出 `agent_message`（甚至 `command_execution`）· 3/4 §F 断言通过 · 仅 `auth_kind_auth`（要求 Error{Auth}）fail —— 因为**没有 auth 错误可供解析**，非 parser 不会解析。

**claude auth_fail 3/3=100% · network 3/3=100% 已证明 parser 错误分类能力**。处理纪律：同 SPIKE-07 Phase D 退化 corpus（标注 + 双口径 raw/非退化并陈 · 不静默剔除）。彻底消解需用真实 OpenAI API key（非 OAuth）重录 codex auth/network —— 列为 §G 残留 + ADR-018 conditional。

### D.2 · #33 codex/mixed_ansi_json/3 = §F 行首启发式 artifact（非 parser 缺陷）

实测模型输出（`reference_text` · parser 已正确抽入 `MessageDelta` · 为避嵌套 fence 用单行内联表示 · `␤` 表换行）：

> ` ```text␤\033[1m粗体\033[0m [{"x":1},{"y":2}]␤``` `

即模型把 JSON 内联在 ANSI 文本**同一行**、整体包在 markdown ` ```text ` fence 内。

§F `mixed_json_parseable`（`assertions.rs` 锁定）启发式 = 行 `trim` 后 `starts_with('{')||starts_with('[')` 再 `serde_json` parse。该行行首是 `\033[1m…`（ANSI），JSON `[{"x":1},{"y":2}]` 内联其后 + 包在 markdown fence —— **行首启发式漏抽 · 但 parser 正确抽出了完整 content**。同场景 take 1/2 PASS（模型把 JSON 放独立行 / json fence 内行首是 `{`）· take 3 模型选内联 —— **模型输出非确定性**，恰是 SPIKE-07 Phase D 命名的三类根因之一"**§F calibration**"（非 corpus 质量 · 非协议现实 · 非 parser 缺陷）。

关键 decision-grade 边界：放宽 `mixed_json_parseable` 为"content 内**子串**扫 JSON"（非行首）→ #33 PASS → mixed 6/6=100% → 非退化 30/30=**100%**。但 `assertions.rs` **byte-identical 锁定**（§B / §2.1）· **主 agent 不自行 recalibrate §F**（decision-grade · 需 Arbiter + ADR）· 本报告**如实报锁定 §F 下的 83.3%** + 根因 + 杠杆，recalibration 列为 ADR-018 conditional 由 Arbiter 拍板。

---

## E · Phase 5 · §H 三路径裁决（SPIKE-07 §H single source of truth）

§H 阈值（SPIKE-07 spec §H · session-32 Arbiter 钦定 · 不在此重定义）：

- **路径 1 greenlight**：overall ≥96% AND 每场景 ≥90% AND 统一抽象可行
- **路径 2 single-cli**：一 CLI ≥96% · 另一 <90%
- **路径 3 deferred**：两 CLI 均 <90% OR 任一场景 <85%

### E.1 · 严格字面应用（不弯规则）

- **raw**：overall 88.9% <96% · auth 50% <85% · mixed 83.3% <85% → **字面 = 路径 3**
- **非退化**（剔 codex auth/net 6 退化 · SPIKE-07 Phase D 先例）：overall 96.7% ≥96% · 每场景 ≥100% **唯 mixed 83.3% <85%** → 仍触发"任一场景 <85%" → **字面 = 路径 3**

### E.2 · 实质裁决（与 SPIKE-07 deferral 的本质区别）

|                 | SPIKE-07 deferred                                                                    | SPIKE-07.5                                                                         |
| --------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| 根因            | parser **无法**从 TUI 屏幕重绘抽内容（能力墙 · long/mixed **0%** · 两 CLI 对称 67%） | parser **干净抽取**（long 100% · claude **全场景 100%** · panic 0 · 统一 IR 实证） |
| 唯一非退化 miss | —（系统性 0%）                                                                       | **1 样本** · §F 行首启发式漏抽内联 fence JSON（parser 抽取正确 · §D.2）            |
| 性质            | corpus 方法论能力墙                                                                  | §F 断言校准 nuance                                                                 |

**实质**：路径 A 假设（结构化模式发可解析协议 · SPIKE-06 TUI 才是 artifact）**被实测确认**。SPIKE-07.5 与 SPIKE-07 的 deferral **不同质**。

### E.3 · 主 agent 推荐（**proposed · 非 accept** · §2.1 / A 栏）

**首选 · 路径 1 greenlight**，conditional on Arbiter 接受 2 项已根因 carve-out：

1. codex auth/network 6 = 退化 corpus（spec §E fail#2 pre-registered · claude 100% 已证 parser 能力 · §D.1）
2. #33 = §F `mixed_json_parseable` 行首启发式 artifact（SPIKE-07 Phase D "§F calibration" 类 · parser 抽取正确 · 子串扫描 recalibration → mixed 100% · §D.2）

接受后：claude 全 6 场景 100% · codex 非退化 11/12 · 统一 `CliEvent` IR 抽象实证（共享 trait · 双 adapter · panic 0 · monotone 成立）→ **决定性推翻 SPIKE-07 corpus 方法论 deferral**。

**保守回退 · 路径 2**：claude **无条件 greenlight**（18/18=100% · 每场景 · 平凡 ≥96%）；codex conditional（非退化 91.7% · pending §F mixed recalibration + 真 OpenAI key 重录 codex auth/network 非退化 corpus）。

**不推荐 路径 3 again**：会把"§F 行首启发式 vs 模型内联 fence 格式"这一校准 nuance 误判为与 SPIKE-07"能力墙"同质 deferral —— 与实测实质矛盾（§E.2）。

裁决归属：§H single-source 不允许主 agent 自行重定阈值 / 自行 recalibrate 锁定 §F → **Arbiter 拍板**（ADR-018 · §2.1 · CLAUDE.md A 栏）。

---

## F · §E.11 基线 +2.8pp < +20pp 的诚实解读

parser 结构化 error-detection 91.7% 仅比关键字基线高 +2.8pp（SPIKE-07 §E.11 设 +20pp 为"parser 值得复杂度"门槛）。诚实归因：

- error-detection（"这是不是错误样本"二分类）是 parser **次要**能力。结构化模式下"有没有 `result{is_error}` / `error` 事件"本就接近关键字可判 → 基线天然高、delta 自然小。
- R1 的**主问题**不是 error-detection · 是"**统一 parser 能否把异构 CLI 输出可靠结构化为 AI-Aware 所需事件流**"（MessageStart/Delta/End / SessionMeta / ToolUse / Error{kind}）。此问题答案 = **YES**（claude 全场景 100% · 统一 IR 实证 · panic 0）—— 关键字基线**完全无法**产出这种结构化事件流（它只能二分类有无错）。
- ∴ +2.8pp 不削弱 R1 结论 · 仅说明 §E.11 这把尺衡量的是 parser 的次要面。如实记录不修饰。

---

## G · 残留与置信度 caveat（不修饰）

1. **codex auth/network 退化**（§D.1）：本批 corpus 无法公平评估 codex 错误事件解析准确率。彻底消解需真实 OpenAI API key（非 OAuth backend）重录 6 样本 → ADR-018 conditional / §G 后续。
2. **#33 §F 校准**（§D.2）：锁定 §F 行首启发式对"模型内联 fence JSON"漏抽。recalibration（子串扫描）是明确杠杆但属 decision-grade · Arbiter 拍板。
3. **network_error take 撞真实 VPN 抖动**：codex network_error_1 含真实 `error:tls handshake eof` 事件（环境噪声）· 实测如实记录 —— 该噪声**恰好**让 codex network §F 通过（真 Error{Network}）· 但归因仍标退化（env 注入无效 · §D.1）· 不因"碰巧过"而洗白。
4. **样本量**：6 场景 × 2 CLI × 3 take = 36（同 SPIKE-06/07 量级）· 单 take 模型非确定性已由 #33 暴露 · 结论在"统一抽象可行性"层面稳健 · 不外推到"生产级 parser 完备性"（spec §C Don't.5 · 归档级原型）。

---

## H · spike-delivery-checklist 3 样必交 manifest

| #          | 物料                                                                                                         | 位置            | 状态 |
| ---------- | ------------------------------------------------------------------------------------------------------------ | --------------- | ---- |
| 1 决策文档 | `docs/spikes/SPIKE-07.5-report.md`（本文件）                                                                 | ✅ 进 git       |
| 2 实测源码 | `docs/spikes/code/SPIKE-07.5/`（Cargo.toml+Cargo.lock+src · 39 测试过 · clippy/fmt 0）                       | ✅ 进 git       |
| 3 Raw 数据 | `docs/spikes/raw/SPIKE-07.5/`（probe/ + corpus/ 36 jsonl+36 sidecar+recording-summary + phase3-matrix.json） | ✅ 进 git       |
| 4 冷备     | 纯 Cargo + Cargo.lock 进 git · `cargo build` 可复现 → ADR-013 v2 可省（非 3 场景）                           | 🟡 省略（合规） |

复现：`cd docs/spikes/code/SPIKE-07.5 && cargo test && SPIKE075_JSON=/tmp/x.json cargo run --bin matrix`。
报告每数字 ↔ `phase3-matrix.json` 字段：overall=`overall_accuracy` · 非退化=`nondegenerate_accuracy` · 场景=`per_scenario[].accuracy` · CLI=`per_cli[].accuracy` · FAIL=`rows[].sample_pass==false` · 基线=`baseline`。

---

## I · 0 编造声明

本报告所有数字源自 `cargo run --bin matrix` 实跑 → `phase3-matrix.json`（git）· 无手填、无估算、无"应该"。corpus 源自实机录制（`record_corpus.sh` · §E.9 可复现）· redact v2 重脱敏与 raw 936=936 对账。失败样本逐条根因（§D · 系统调试纪律）· 锁定 §F 下 83.3% 如实呈现未为凑路径 1 改 `assertions.rs`（sha256 §B 可验）。退化样本标注非静默剔除。§H 裁决主 agent 仅 **proposed** · accept 属 Arbiter（§2.1 · A 栏 ADR）。

---

## J · 后续（全部 Arbiter-gated · 主 agent 不自行推进）

- **ADR-018**（proposed · 见 [`docs/adr/ADR-018-*`](./adr/ADR-018-ai-aware-r1-rejudge.md)）：§H 裁决 + R1 降级 proposed + supersede ADR-017 · **Arbiter 拍板**
- 若 Arbiter accept 路径 1/2：SPIKE-07.5 spec `done` 翻转 · CLAUDE.md 决策表 #3 更新 · MVP-18/19/20 解锁依赖链 —— **均 Arbiter 明确后**
- §F mixed recalibration（子串扫描）+ 真 OpenAI key 重录 codex auth/network —— ADR-018 conditional · Arbiter 决定是否要求 SPIKE-07.6 补强或接受 carve-out

```

```
