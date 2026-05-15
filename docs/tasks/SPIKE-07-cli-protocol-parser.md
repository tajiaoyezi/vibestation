---
id: SPIKE-07
type: spike
title: CLI 输出协议 parser 验证 spike（R1 降级前置）
status: done
status_note: "spike 已跑完（Phase A-F · PR #333/#334/#335/#338）· §H 路径 3 deferred · R1 保留 HIGH/HIGH · ADR-017 accepted（2026-05-16 Arbiter 拍板）· 后续路径 A → 新开 SPIKE-07.5（结构化模式重录重跑）· MVP-18/19/20 实施前置改为 SPIKE-07.5 实跑 PASS"
owner: Claude Code
phase: v1.0-pre
depends_on: ["SPIKE-06"]
depends_on_notes: "SPIKE-06 = §A CLI 脱敏样本（36 条 · done · PR #71 · 本 Spike 的 parser 直接用这 36 条样本做 corpus）· §B codesign/notarization 不是 SPIKE-07 前置（SPIKE-07 是纯 parser 验证 · 无需签名打包）· SPIKE-06 现 status: blocked 只是 §B 卡 Apple Dev · 不阻塞 SPIKE-07 v1.0 开工 · 同 MVP-04 depends_on_notes 模式 · session 13 X-1 补"
blocks: ["MVP-18", "MVP-19", "MVP-20"]
blocked_by: []
blocked_from:
blocked_note:
estimate: 3d
plan_ref: implementation-plan.md §5.3.6 · §9 R1 · §1.1
risk_ref: R1
reviewer: "ready-gate（session 32）= OpenCode self-review §2.10 · done-flip（2026-05-16）= Claude Code self-review v2-D.2 + Arbiter tajiaoyezi 拍板 '按照推荐执行'（ADR-017 accepted · 路径 A）"
---

# SPIKE-07: CLI 输出协议 parser 验证

> **状态**：`ready`（v1.0-pre · session 32 ready-gate：3 个 High 阻塞修复 @ PR #328 · re-review APPROVE-WITH-NITS · threshold 收敛 + 2 nit 修 + Arbiter approve flip · 实施是 MVP-18/19/20 的 R1 gate 前置）
> **依赖**：[SPIKE-06](./SPIKE-06-cli-protocol-and-codesign.md)（36+ 样本已录制 · PR #71）
> **阻塞**：MVP-18/19/20（AI-Aware 三件套 · v1.0 vision）
> **战略依据**：[`implementation-plan.md §5.3.6`](../implementation-plan.md) · [`§9 R1`](../implementation-plan.md)
> **reviewer**: OpenCode · self-review · §2.10 evidence-based self-verify N=4 受限策略

---

## §A · 目标（Goal）

基于 SPIKE-06 录制的 36+ 脱敏 CLI 输出样本，实现原型 parser，验证"两个 CLI（Claude / Codex）能否统一抽象"有可信工程答案，为 **R1**（CLI 协议未实机验证 · HIGH/HIGH）的降级提供定量依据。

**R1 降级的业务价值**：

- R1 是 `implementation-plan.md §9` 中概率=高、影响=高的双高风险项，直接阻塞 v1.0 AI-Aware 三件套（MVP-18/19/20）的开工
- 若本 Spike 证明 parser 可行且准确率达标 → R1 可从 HIGH/HIGH 降级为 MEDIUM/LOW → v1.0 规划可提前启动 → 产品差异化卖点（AI session 感知）从"远期愿景"变为"可排期实现"
- 若本 Spike 证明 parser 不可行 → R1 保留 → AI-Aware 推迟到 v2+ → v1.0 只能主打"多 Tab 终端 + Git 工作台"（基础卖点）→ 竞争差异化窗口收窄

**本 Spike 通过后**：

- 写 **ADR-017-ai-aware-greenlight.md** · R1 降级 proposal → accepted
- 解锁 MVP-18/19/20 三件套的详化

**本 Spike 不通过**：

- R1 保留 · AI-Aware v1.0 推迟 / 降级 / 放弃
- `CLAUDE.md §决策表 #3` 可能需要更新

---

## §B · 背景（Context）

### R1 当前等级

`implementation-plan.md §9` 中 **R1** 定义：

> R1 | Claude CLI 输出协议与 Codex 不同，解析失败 | 高 | 高 | Spike Day 5 实机录制样本；v1.0 W23 单独 spike 前不锁定实现 | 核心作者 | Spike Day 5

R1 是当前 30 条风险中**唯一**概率与影响均为"高"的项。它阻塞的不是 MVP（v0.1-v0.3 只做"多 Tab 终端 + Git 工作台"，不解析 CLI 输出），而是 **v1.0 的 AI-Aware 三件套**——所有依赖 parser 的功能（session 识别、build fail 反哺、一键回滚）都以"能稳定解析 CLI 输出"为前提。

### SPIKE-06 样本 corpus

SPIKE-06（W0-D6）已完成 §A 的 36+ 脱敏样本录制（PR #71）：

- **2 CLI**：Claude CLI + Codex CLI
- **6 场景**：Happy path / 中断残帧 / 认证失败 / 网络错误 / 长流式 / 混合 ANSI-JSON
- **3 次重复**：每个场景 × 3 次录制（覆盖平台差异：mac/linux）
- **脱敏标准**：删除 auth token / API key / JWT / PII / 本地路径 / git remote URL（保留协议结构占位，如 `eyJ...FAKE_JWT_STRUCTURE...`）
- **存储位置**：`docs/spikes/raw/SPIKE-06/`（脱敏后 · **进 repo** · 含 `README.md` 索引）。注意：这是 SPIKE-06 实际归档位置（旧 spec 草稿曾写 `docs/spike-artifacts/SPIKE-06/`，该目录不存在 · 已对齐 `.claude/rules/spike-delivery-checklist.md` 的 raw 归档约定）
- **样本格式**：每条录制是 **asciinema `.redacted.cast`**（JSON-lines 事件流 · 非纯 `.txt`）+ 同名 `.redaction.json` sidecar（记录被脱敏字段）。命名约定：`{cli}_{scenario}_{take}.redacted.cast`（下划线分隔 · 如 `claude_auth_fail_1.redacted.cast`）

**样本结构示例**（实际目录 · `ls docs/spikes/raw/SPIKE-06/`）：

```
docs/spikes/raw/SPIKE-06/
├── README.md                              # 样本索引（fixture loader 应优先读此文件）
├── claude_happy_path_1.redacted.cast      # Claude · happy path · take-1（asciinema 事件流）
├── claude_happy_path_1.redaction.json     # take-1 脱敏字段记录 sidecar
├── claude_happy_path_{2,3}.redacted.cast  # take-2/3（mac / linux 平台差异）
├── claude_auth_fail_{1,2,3}.redacted.cast
├── claude_network_error_{1,2,3}.redacted.cast
├── claude_interrupt_residual_{1,2,3}.redacted.cast
├── claude_long_stream_{1,2,3}.redacted.cast
├── claude_mixed_ansi_json_{1,2,3}.redacted.cast
├── codex_*_{1,2,3}.redacted.cast          # 同上 6 场景 × 3 次
└── claude-version-0{1,2,3}.txt            # CLI 版本捕获（非样本 · loader 必须按 *.redacted.cast 过滤）
```

> 场景 → 文件名映射：`happy_path` / `interrupt_residual` / `auth_fail` / `network_error` / `long_stream` / `mixed_ansi_json`（6 场景 × 2 CLI × 3 take = 36 条 `.redacted.cast`）。

SPIKE-06 的结论严格区分为"CLI 能在 PTY 里运行"（结论 A）和"协议足够清楚可指导实现"（结论 B）。**结论 B 本 Spike 不验证，R1 保留**。SPIKE-07 是在 SPIKE-06 的样本基础上，回答"给定这些真实输出，能否稳定解析为结构化事件"。

**关键认知**：SPIKE-06 只回答了"CLI 输出长什么样"，SPIKE-07 要回答"机器能不能稳定理解这些输出"。这是从"观察"到"工程可行性"的跃迁。

### ADR-009 决策依据

[ADR-009](../adr/ADR-009-ai-aware-v1-vision.md) 明确：

> AI-Aware = v1.0 vision · README / landing / Twitter / Discord 完全不宣传 · 直到 v1.0 真实落地再讲
> 技术前提：SPIKE-07 parser-oriented spike 必通过（基于 SPIKE-06 录制的 36+ 样本 · parsed_issues 解析准确率 ≥ 95%）
> SPIKE-07 通过 → 写 ADR-017-ai-aware-greenlight.md · 才能启动 MVP-18/19/20 详化

ADR-009 还规定：

- ❌ 禁止对外文案提及 AI-Aware Pane / Mission Control / AI session aware
- ✅ 允许：内部技术文档（ADR / implementation-plan / tasks/MVP-18..20）明确标注"v1.0 vision"
- **R1 降级授权只能通过 SPIKE-07 的 ADR 完成**，SPIKE-06 无权下调 R1

### 通过 / 不通过对应 ADR-017 路径

| 本 Spike 结果                            | ADR-017 文件名                   | 内容                                                    | 后续影响                                                            |
| ---------------------------------------- | -------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------------- |
| 通过（全指标达标）                       | `ADR-017-ai-aware-greenlight.md` | R1 降级到 MEDIUM/LOW · parser 可行 · v1.0 AI-Aware 开工 | MVP-18/19/20 详化解锁 · `CLAUDE.md` 决策表 #3 移除 ⚠️ 警告          |
| 部分通过（一个 CLI 能 parse 另一个不能） | `ADR-017-ai-aware-single-cli.md` | 只支持能 parse 的那个 · README / landing 需说明         | MVP-18/19/20 详化解锁（仅限支持的 CLI）· 另一个 CLI 推到 v2         |
| 双失败                                   | `ADR-017-ai-aware-deferred.md`   | R1 不降级 · AI-Aware 推迟到 v2+                         | MVP-18/19/20 status 保持 draft · `CLAUDE.md` 决策表 #3 保留 ⚠️ 警告 |

---

## §C · 功能范围（Scope）

### Do（必做）

1. **Fixture loader**：读取 `docs/spikes/raw/SPIKE-06/` 目录下的 36 个 `*.redacted.cast` 样本（按 `*.redacted.cast` glob 过滤 · 跳过 `claude-version-*.txt` 等非样本文件），从文件名 `{cli}_{scenario}_{take}.redacted.cast` 解析 `{cli, scenario, take}` 三维索引建立 fixture 注册表；同名 `.redaction.json` sidecar 提供脱敏字段 metadata
2. **原型 parser 实现**：在独立 `docs/spikes/code/SPIKE-07/` 目录实现原型 parser（不进 `crates/` · 不锁死 parser 库，先用正则/状态机 combo，spike 实施时按实测选型）
3. **结构化事件解析**：从 CLI 原始输出流中提取以下事件类型：
   - `message_start` / `message_delta` / `message_end`
   - `role: user | assistant | system`
   - `error: auth | network | rate_limit | timeout | unknown`
   - `tool_use_start` / `tool_use_end`（两 CLI 若有 function calling 需识别）
4. **IR 输出**：parser 输出统一中间表示（Intermediate Representation），不直接绑定到前端数据结构
5. **6 场景 × 2 CLI 全覆盖**：对 SPIKE-06 的 36+ 样本逐条做解析断言（见 §F 测试矩阵）
6. **准确率统计**：按场景、按 CLI、按事件类型三维度统计 parser 正确率
7. **统一抽象可行性分析**：对比两 CLI 的 IR 差异，判断"能否共享同一 parser + IR"或"必须各自独立"
8. **错误模式分类**：对 parse fail 的样本按"结构化缺失 / ANSI 干扰 / 残帧歧义 / 未知格式"分类
9. **性能基线测量**：parser 处理单条样本耗时（P50/P99），确认不会成为瓶颈
10. **报告生成**：输出 `docs/spikes/SPIKE-07-report.md`，含准确率数据表 + 统一抽象结论 + R1 降级 proposal

### Don't（明确不做）

1. **完整 AI-Aware 实现**：MVP-18/19/20 范围，本 Spike 只验证 parser 可行性
2. **MVP 集成**：MVP-04 多 Tab 终端集成 CLI 不依赖 parser，本 Spike 不改变 MVP 实现
3. **第三方 CLI**：Gemini / 其他 CLI 不在范围内，v2+ 再评估
4. **实时流式解析**：本 Spike 只测回放样本，不测 live stream 增量解析（留给 v1.0 实施时）
5. **parser 长期维护**：原型代码归档到 `docs/spikes/code/SPIKE-07/`（**进 git** · 证据级 · 见 `.claude/rules/spike-delivery-checklist.md` 3 样必交），不 merge 到 main 的 `crates/`（避免技术债 · 不重写为生产 parser）
6. **ADR-017 正式撰写**：spec 内可附 ADR outline 模板（§M），但 ADR-017 实际开 PR 是 spike 跑完后的事

---

## §D · IR 设计骨架（Intermediate Representation）

> 本 section 为建议性设计，spike 实施时可按实测迭代。目标是给 parser 一个统一输出格式，让"两 CLI 能否统一"的判定有具体标准。

```rust
/// CLI 输出事件 IR（版本 v1 · spike 实施时迭代）
#[derive(Debug, Clone, PartialEq)]
pub enum CliEvent {
    MessageStart {
        role: Role,
        timestamp: Option<String>,
    },
    MessageDelta {
        content: String,
        /// 原始 ANSI 序列是否保留（供前端可选渲染）
        raw_ansi: Option<String>,
    },
    MessageEnd {
        finish_reason: FinishReason,
    },
    Error {
        kind: ErrorKind,
        message: String,
        /// 是否可恢复（如 rate_limit 可重试，auth 不可）
        recoverable: bool,
    },
    ToolUseStart {
        tool_name: String,
        tool_id: String,
    },
    ToolUseDelta {
        tool_id: String,
        partial_input: String,
    },
    ToolUseEnd {
        tool_id: String,
        final_input: serde_json::Value,
    },
    /// 未识别但需保留的原始块（用于后续人工审计）
    Unrecognized {
        raw: String,
        /// 推测类型（启发式标注）
        heuristic: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    Error(ErrorKind),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    Auth,
    Network,
    RateLimit,
    Timeout,
    Unknown,
}
```

**设计原则**：

- `Unrecognized` 变体是强制要求：parser 遇到无法解析的块不得 panic 或丢弃，必须包装成 `Unrecognized` 输出，供后续人工审计
- `raw_ansi` 可选保留：前端可能想用 xterm 直接渲染原始带颜色的输出，而不是解析后的纯文本
- `recoverable` 字段：影响前端错误提示策略（rate_limit 提示"稍后重试"，auth 提示"检查 API key"）

**spike 实施时决定项**：

- parser crate 选型（手写状态机 vs `nom` vs `peg` vs `lalrpop`）——见 §J 决策表
- `timestamp` 格式（RFC3339 vs 毫秒 unix epoch）——取决于 CLI 实际输出格式
- `ToolUse*` 变体是否保留——取决于实测中两 CLI 是否真有 function calling 输出

---

## §E · Acceptance（通过标准）

> ⚠️ **详化阶段声明**：以下门槛为 spike 实施时必须达到的目标，不是当前已测结果。详化阶段不能 fabricate 准确率数字。

### E.1 · Fixture 回放

- [ ] Fixture loader 能正确读取 `docs/spikes/raw/SPIKE-06/` 全部 36 个 `*.redacted.cast` 样本（按 glob 过滤 · 跳过 `claude-version-*.txt`）
- [ ] 每个样本附 metadata：{cli, scenario, take} 从文件名 `{cli}_{scenario}_{take}.redacted.cast` 解析，redacted_fields 从同名 `.redaction.json` sidecar 读取
- [ ] 样本格式按 asciinema `.cast`（JSON-lines · `[timestamp, "o", data]` 事件）解析；loader 须能从 cast 事件流重组终端原始输出字节流
- [ ] Fixture 注册表能按 `cli=claude, scenario=auth_fail` 查询到所有匹配样本（scenario 用下划线命名 · 对齐实际文件名）

### E.2 · Parser 覆盖率

- [ ] Parser 对 36+ 样本全部执行解析（无 crash、无 panic）
- [ ] 覆盖 6 场景 × 2 CLI = 12 场景组合，每组合至少 3 条样本
- [ ] 每样本至少生成 1 个 `CliEvent`（空输出视为异常，需人工审计）
- [ ] `Unrecognized` 事件比例 ≤ 10%（总事件数中 ≤ 10% 为未识别）

### E.3 · 准确率门槛（场景级诊断指标 · informative）

> 📊 **本节是诊断指标，不是 R1 降级判据**（session 32 Arbiter 钦定）。以下阈值用于 Phase D 报告的场景级分析（哪个场景弱、弱多少），帮助定位 parser 短板。**R1 降级 gate 的单一权威判据见 §H 三路径**；本节阈值与 §H 冲突时以 §H 为准。

- [ ] Happy path 场景：解析正确率 ≥ 99%（message_start/delta/end 角色识别正确）
- [ ] 失败路径场景（auth / network / 中断）：解析正确率 ≥ 95%（error kind 分类正确）
- [ ] 混合 ANSI+JSON 场景：解析正确率 ≥ 95%（最难场景，ANSI 序列不干扰结构化提取）
- [ ] 长流式输出场景：不丢帧、不截断（尾部 100 字符完整提取）
- [ ] 整体加权正确率 ≥ 96%（按样本数加权平均）

### E.4 · 统一抽象可行性

- [ ] 两 CLI 的 IR 差异清单（字段级对比）
- [ ] 判定："共享同一 IR" 或 "必须各自独立 IR + adapter 层"
- [ ] 若共享：给出统一 parser 的架构建议
- [ ] 若独立：给出 adapter 层设计 + 维护成本估算

### E.5 · R1 降级 proposal（§H 三路径的口语化对照 · informative）

> 📊 **本节是 §H 三路径的口语化对照，不是独立判据**（session 32 Arbiter 钦定）。正式 R1 降级结论**只依据 §H**；以下对照与 §H 冲突时以 §H 为准。Phase E 写 ADR-017 时按 §H 路径选 greenlight / single-cli / deferred 变体。

- [ ] 对照 §H 路径 1（通过）→ ADR-017 greenlight · R1 降级到 MEDIUM/LOW（§H 路径 1 含整体加权 ≥ 96% + 各场景 ≥ 90% + 两 CLI 可统一）
- [ ] 对照 §H 路径 2（部分通过 · 一 CLI 达标）→ ADR-017 single-cli · R1 降级到 MEDIUM/LOW 但只支持达标的 CLI
- [ ] 对照 §H 路径 3（双失败）→ ADR-017 deferred · R1 保留
- [ ] 诊断补充（informative · 不改 §H 判定）：整体准确率 ≥ 99% 时 report 可附「R1 可进一步降到 LOW/LOW」建议供 Arbiter 参考，但 gate 仍以 §H 路径 1 为准

### E.6 · 报告质量

- [ ] `docs/spikes/SPIKE-07-report.md` 包含：准确率数据表、统一抽象分析、R1 降级建议
- [ ] 报告中 0 条 fabricated 数据（所有数字来自实测）
- [ ] 报告明确标注"样本量=36+"和"置信度限制"
- [ ] 样本不够真实的 caveat 在报告中显式声明

### E.7 · 代码质量

- [ ] 原型 parser 代码在 `docs/spikes/code/SPIKE-07/` 目录（进 git · 含 `Cargo.toml` + `Cargo.lock` + `src/` + `README.md`），不进 `crates/`
- [ ] 代码能独立编译运行（`cargo run --example replay-fixtures` 或等价）
- [ ] 无 hardcoded 路径（fixture 路径通过 CLI 参数或 env 传入）
- [ ] 无 secret / API key 硬编码

### E.8 · 时间约束

- [ ] 3d 工期内完成（estimate: 3d）
- [ ] 若 3d 内准确率不达标 → 允许延期 1d 迭代 parser，但必须在 report 中说明原因

### E.9 · 可复现性

- [ ] 同一条样本跑 3 次 parser，结果一致（无随机性）
- [ ] 不同机器（macOS + Ubuntu）跑同一 fixture，结果一致
- [ ] 报告附 `cargo --version` + `rustc --version` + 操作系统版本

### E.10 · 人工审计

- [ ] 所有标记为 `Unrecognized` 的事件须经人工审计，分类到以下桶：
  - (a) 已知模式但 parser 未覆盖 → 应扩展 parser
  - (b) 真正的新格式 → 应补充样本
  - (c) 脱敏导致结构损坏 → 应重录样本
  - (d) 无法分类 → 留待后续研究
- [ ] 人工审计记录附在报告附录

### E.11 · 对比基线

- [ ] 提供"不做 parser"基线：直接用正则提取 error 关键字的准确率（作为 parser 价值证明）
- [ ] 提供"简单启发式"基线：按行首特征（`^Error:`、`^WARNING:`）分类的准确率
- [ ] Parser 必须显著优于基线（+20% 以上），否则说明 parser 复杂度不值得

---

## §F · 测试矩阵（Fixture 回放）

### 矩阵结构

6 场景 × 2 CLI = 12 case，每 case 3 条样本（take-1/2/3），合计 36 条。

| #   | CLI    | 场景           | 样本数 | 断言要点                                                                      |
| --- | ------ | -------------- | ------ | ----------------------------------------------------------------------------- |
| 1   | Claude | Happy path     | 3      | (a) message_start 识别正确 (b) role=assistant (c) message_end 不丢            |
| 2   | Claude | 中断残帧       | 3      | (a) Ctrl+C 后的残帧被识别为 Unrecognized 或截断 message (b) 不 panic          |
| 3   | Claude | 认证失败       | 3      | (a) error kind=Auth (b) recoverable=false (c) 不误标为 Network                |
| 4   | Claude | 网络错误       | 3      | (a) error kind=Network (b) recoverable=true (c) 不误标为 Timeout              |
| 5   | Claude | 长流式         | 3      | (a) 10k+ token 输出不截断 (b) 尾部 100 字符完整 (c) 无中间丢帧                |
| 6   | Claude | 混合 ANSI-JSON | 3      | (a) ANSI 序列不污染 JSON 提取 (b) JSON 字段完整 (c) color 标记保留在 raw_ansi |
| 7   | Codex  | Happy path     | 3      | (a) message_start 识别正确 (b) role=assistant (c) message_end 不丢            |
| 8   | Codex  | 中断残帧       | 3      | (a) Ctrl+C 后的残帧被识别为 Unrecognized 或截断 message (b) 不 panic          |
| 9   | Codex  | 认证失败       | 3      | (a) error kind=Auth (b) recoverable=false (c) 不误标为 Network                |
| 10  | Codex  | 网络错误       | 3      | (a) error kind=Network (b) recoverable=true (c) 不误标为 Timeout              |
| 11  | Codex  | 长流式         | 3      | (a) 10k+ token 输出不截断 (b) 尾部 100 字符完整 (c) 无中间丢帧                |
| 12  | Codex  | 混合 ANSI-JSON | 3      | (a) ANSI 序列不污染 JSON 提取 (b) JSON 字段完整 (c) color 标记保留在 raw_ansi |

### 每 case 必做断言（≥ 3 条）

**通用断言（所有 case）**：

1. Parser 对样本不 panic、不 crash
2. 输出事件序列非空（至少 1 个 CliEvent）
3. 事件时间顺序单调（message_start 在 message_delta 前，message_delta 在 message_end 前）

**场景特化断言**：

- Happy path：role 识别正确、无 Error 事件混入
- 中断残帧：最后一个事件不是悬空 message_start（必须有 end 或 unrecognized）
- 认证失败：Error 事件 kind=Auth，且出现在 message_start 之前或替代 message_start
- 网络错误：Error 事件 kind=Network，recoverable=true
- 长流式：总 content 长度 ≥ 原始输出 95%（允许 ANSI 剥离导致的少量长度差异）
- 混合 ANSI-JSON：提取的 JSON 字符串可解析为 serde_json::Value，且关键字段存在

---

## §G · Fail Signals（失败信号）

扩展自占位 spec 的 3 条到 6 条，覆盖更多边缘情况：

1. **任一场景准确率 < 90%** → R1 不能降级 · AI-Aware 全套推迟
   - 即使其他场景都达标，只要一个场景 < 90%，说明 parser 在边界条件下不可靠
   - 特别关注点：混合 ANSI-JSON 场景通常是最容易低于门槛的
   - **与 §H 一致性**：本条 = §H 路径 1「各场景 ≥ 90%」的逆否（非独立阈值）· R1 降级 gate 判定以 §H single source of truth 为准

2. **两 CLI 结构差异过大（无法统一抽象）** → 只能支持一个 CLI · README 措辞修改
   - "差异过大"的判定标准：IR 字段重合度 < 70%（即 30% 以上字段只能一个 CLI 用）
   - 或 adapter 层代码量 > parser 核心代码量的 50%（维护成本过高）

3. **Parser 遇到真实样本 crash** → Rust 实现 bug · 迭代修复
   - crash 包括：panic、segfault、无限循环、内存泄漏
   - 单次 crash 可修复；同一位置重复 crash → 说明该模式超出 parser 设计范围 → 需重新评估

4. **样本不够真实** → SPIKE-06 的 36 条样本未覆盖 CLI 升级后的新格式 / 长 session 状态管理 → 需要补充录制 → Spike 延期
   - 触发信号：Unrecognized 事件中 > 30% 属于"看起来像新格式"
   - 补充录制需回到 SPIKE-06 流程，额外 0.5-1d

5. **CLI 升级 break parser** → 当前 parser 写死对 SPIKE-06 录制时的 CLI 版本 → 需评估 parser 迁移成本 → 若迁移成本过高（> 1d）→ R1 保留
   - 记录 SPIKE-06 时的 CLI 版本号（`claude --version`、`codex --version`）
   - parser 设计时预留版本检测入口，但不强制实现

6. **Unrecognized 事件比例 > 20%** → 说明 parser 覆盖不全，即使"未 crash"也不能算通过 → 需要扩展 parser 规则或接受"部分解析"结论
   - 20% 是硬性门槛：超过意味着 parser 对 1/5 的输出"看不懂"
   - 若 10% < Unrecognized ≤ 20% → 可接受但需在报告中显式说明局限性

---

## §H · Fallback 方案（3 路径 · R1 降级 single source of truth）

> 🔒 **本节是 R1 降级 decision-grade 单一权威判据（single source of truth · session 32 Arbiter 钦定）**。§E.3 / §E.5 为**场景级诊断指标（informative）**，写入 Phase D report 供分析，**不参与 R1 降级 gate 判定**。任何阈值边界带（如 mixed 场景实测落 90–95%）与本节冲突时——**一律以本节三路径为准**。Phase E 写 ADR-017 的降级结论只依据本节。

保留占位中已有的 3 路径，各自实化操作：

### 路径 1 · 通过（全指标达标）

**条件**：

- 整体加权正确率 ≥ 96%
- 各场景正确率均 ≥ 90%
- 两 CLI 可统一抽象（或 adapter 层成本可接受）

**操作**：

1. 写 `ADR-017-ai-aware-greenlight.md` · 提议 R1 降级到 MEDIUM/LOW（或 LOW/LOW）
2. ADR accepted 后 → MVP-18/19/20 status 从 draft 翻 ready（由 main agent 操作，不自行 flip）
3. 更新 `CLAUDE.md §决策表 #3`：移除 ⚠️ 警告
4. 原型 parser 归档到 `docs/spikes/code/SPIKE-07/`（进 git · 不进 `crates/`）
5. 报告归档到 `docs/spikes/SPIKE-07-report.md` · raw 数据归档到 `docs/spikes/raw/SPIKE-07/`（3 样必交 · 见 §I）

### 路径 2 · 部分通过（一个 CLI 能 parse 另一个不能）

**条件**：

- 一个 CLI 整体正确率 ≥ 96%
- 另一个 CLI 整体正确率 < 90% 或无法统一抽象

**操作**：

1. 写 `ADR-017-ai-aware-single-cli.md` · 提议只支持能 parse 的 CLI
2. README / landing 需明确说明"AI-Aware 功能仅支持 X CLI"
3. 另一个 CLI 的 parser 推到 v2 评估
4. MVP-18/19/20 详化解锁（仅限支持的 CLI）
5. `CLAUDE.md §决策表 #3` 更新为"⚠️ 仅支持 X CLI"

### 路径 3 · 双失败

**条件**：

- 两个 CLI 整体正确率均 < 90%
- 或任一场景正确率 < 85%
- 或 parser 对多数样本产生 Unrecognized

**操作**：

1. 写 `ADR-017-ai-aware-deferred.md` · 提议 R1 不降级 · AI-Aware 推迟到 v2+
2. MVP-18/19/20 保持 draft 状态，不进入 in-progress
3. `CLAUDE.md §决策表 #3` 保留 ⚠️ 警告（甚至加强措辞）
4. v1.0 卖点回归"多 Tab 终端 + Git 工作台"（基础差异化）
5. 若后续 CLI 版本升级后协议更结构化 → 重新评估启动新 Spike

---

## §I · 交付物归档（3 样必交 + 1 推荐 · 对齐 `.claude/rules/spike-delivery-checklist.md`）

> ⚠️ **详化阶段声明**：以下清单为 spike **跑完后**的归档要求。详化阶段只写清单模板，不实际产生交付物。
> 归档位置严格对齐 [`.claude/rules/spike-delivery-checklist.md`](../../.claude/rules/spike-delivery-checklist.md) v2（ADR-013 冷备降级）· **3 样必交全进 git · 缺任一 accept 不成立**。

| #   | 物料                      | 路径                             | 进 git |  级别   | 说明                                                                                                                   |
| --- | ------------------------- | -------------------------------- | :----: | :-----: | ---------------------------------------------------------------------------------------------------------------------- |
| 1   | **决策文档**（report）    | `docs/spikes/SPIKE-07-report.md` |   ✅   | 🔴 必须 | 准确率数据表 + 统一抽象分析 + R1 降级 proposal · 每个数字必须 raw 可溯源                                               |
| 2   | **实测源码**（code）      | `docs/spikes/code/SPIKE-07/`     |   ✅   | 🔴 必须 | 原型 parser + fixture loader + 测试脚本 · 含 `Cargo.toml` + `Cargo.lock` + `src/` + `README.md`（复现命令 + 结论溯源） |
| 3   | **Raw 数据**              | `docs/spikes/raw/SPIKE-07/`      |   ✅   | 🔴 必须 | parser 对每条样本的完整输出日志 + 性能 profiling（JSON/log）+ `README.md` 字段索引                                     |
| 4   | **冷备**（含 build 产物） | `spike-tmp/archive/SPIKE-07/`    |   ❌   | 🟡 推荐 | gitignored · 含 `target/` · 纯 Cargo + Cargo.lock 进 git 前提下可省（v2 · ADR-013 降级）                               |

### accept 原子性（不可跨 session 拆分）

review accept 必须在**同一个主 agent 动作内**完成：判定 Pass/Fail → 决策文档入库 → 源码归档 `docs/spikes/code/SPIKE-07/` → raw 归档 `docs/spikes/raw/SPIKE-07/` → ADR-017 翻转 → spec done 翻转（独立评审后）。任一步骤中断 · session 结束前必须补全 · 不允许跨 session 遗留。反模式："先 accept · 明天再归档代码"。

### Spike PR Test Plan 必填项（开 PR 时 body 必含 · 独立评审者逐项验证）

- [ ] 决策文档 `docs/spikes/SPIKE-07-report.md` 已入库（🔴 必须）
- [ ] 源码归档 `docs/spikes/code/SPIKE-07/` 已入库（含 `Cargo.lock` · 🔴 必须）
- [ ] Raw 数据 `docs/spikes/raw/SPIKE-07/` 已入库（🔴 必须）
- [ ] 冷备 `spike-tmp/archive/SPIKE-07/` 本地保留（🟡 推荐 · 纯 Cargo 可省）
- [ ] Report 引用的每个数字都能在 raw 文件溯源（🔴 必须）
- [ ] clone 本 repo 后 · 在 `docs/spikes/code/SPIKE-07/` `cargo run` 能复现 parser 结果（🔴 必须）

---

## §J · 决策表

| #   | 决策项             | 当前假设                       | 若假设失败                                        | 留 spike 实施时决定                                  |
| --- | ------------------ | ------------------------------ | ------------------------------------------------- | ---------------------------------------------------- |
| 1   | Parser crate 选型  | 手写状态机 + 正则 combo        | 若手写复杂度爆炸 → 切 `nom` 或 `peg`              | ✅ 留 spike 实施时按实测选型                         |
| 2   | Fixture 数量是否够 | 36 条足够得出初步结论          | 若 36 条覆盖不全 → 补充录制到 72 条               | ⚠️ 留 spike 实施时评估，若 Unrecognized > 20% 则补充 |
| 3   | 两 CLI 统一 IR     | 假设可统一                     | 若结构差异过大 → 各自独立 IR + adapter 层         | ✅ 留 spike 实施时按实测判定                         |
| 4   | ToolUse 事件保留   | 假设两 CLI 有 function calling | 若实测无 ToolUse 输出 → 从 IR 中删除 ToolUse 变体 | ✅ 留 spike 实施时决定                               |
| 5   | ANSI 保留策略      | 保留 raw_ansi 字段             | 若前端不需要 → 删除 raw_ansi，只输出纯文本        | ⚠️ 等 v1.0 前端需求明确后再定                        |
| 6   | R1 降级等级        | 假设 ≥ 95% → MEDIUM/LOW        | 若 < 95% → 保留；若 ≥ 99% → LOW/LOW               | ✅ 留 spike 实施时按实测数字定                       |

---

## §K · 实施 Phase 拆分

| Phase | 任务                                                 | 估时 | 阻塞项  | 产出                                                                                                                                                        |
| ----- | ---------------------------------------------------- | ---- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A     | Fixture loader 实现：读取 SPIKE-06 样本 + 建立注册表 | 0.5d | 无      | fixture registry + 样本验证脚本                                                                                                                             |
| B     | Parser MVP：手写状态机解析 happy path（Claude 先）   | 0.5d | Phase A | Claude happy path 解析通过                                                                                                                                  |
| C     | 6 场景断言：对 36+ 样本逐条跑断言 + 记录失败         | 0.5d | Phase B | 12 case × 3 样本 = 36 条结果                                                                                                                                |
| D     | 准确率统计 + 统一抽象分析 + 两 CLI 对比              | 0.5d | Phase C | 准确率数据表 + IR 差异清单                                                                                                                                  |
| E     | ADR-017 起草（基于 Phase D 结论）                    | 0.5d | Phase D | ADR-017 草稿（§H 3 条 R1 判定路径 → greenlight/single-cli/deferred · 另 §M 含意外路径 D = parser crash/样本不真实 → 按 deferred 处理 · 与 §M outline 对齐） |
| F     | 报告撰写 + 代码清理 + 归档                           | 0.5d | Phase E | `docs/spikes/SPIKE-07-report.md` + `docs/spikes/code/SPIKE-07/` + `docs/spikes/raw/SPIKE-07/`（3 样必交）                                                   |

**合计**：3d（含 0.5d buffer）

**Phase 详细说明**：

**Phase A · Fixture loader（0.5d）**：

- 读取 `docs/spikes/raw/SPIKE-06/` 目录树（按 `*.redacted.cast` glob 过滤）
- 按文件名约定解析 metadata：`{cli}_{scenario}_{take}.redacted.cast`（下划线分隔 · asciinema cast 格式）+ 同名 `.redaction.json` sidecar
- 建立 `FixtureRegistry` 数据结构：`HashMap<(CLI, Scenario), Vec<Fixture>>`
- 每个 `Fixture` 含：path, cli, scenario, take, raw_content, metadata
- 输出：fixture registry + 验证脚本（确认 36 条全部可读）

**Phase B · Parser MVP（0.5d）**：

- 先攻 Claude CLI happy path（最简单场景）
- 实现基础状态机：`Idle → MessageStart → MessageBody → MessageEnd → Idle`
- 处理 ANSI 转义序列剥离（保留 raw_ansi 字段）
- 输出首批 `CliEvent` 序列，人工肉眼校验 3 条样本
- 通过后再扩展到 Codex CLI happy path

**Phase C · 6 场景断言（0.5d）**：

- 对 36 条样本批量跑 parser
- 每样本生成 `ParseResult { events: Vec<CliEvent>, errors: Vec<ParseError> }`
- 按 §F 矩阵逐条做断言
- 记录：通过数 / 失败数 / Unrecognized 比例 / panic 次数
- 输出：36 条结果表格（CSV 或 Markdown）

**Phase D · 准确率统计 + 统一抽象分析（0.5d）**：

- 按场景、CLI、事件类型三维度统计
- 计算：整体正确率、各场景正确率、Claude vs Codex 差异度
- 对比两 CLI 的 IR 输出差异，填写 §J 决策表
- 判定统一抽象可行性，给出工程建议
- 输出：准确率数据表 + IR 差异清单 + R1 降级建议草稿

**Phase E · ADR-017 起草（0.5d）**：

- 基于 Phase D 结论，按 §M 模板起草 ADR-017
- §H 3 条 R1 判定路径 → greenlight/single-cli/deferred 三变体（§M 另含意外路径 D = parser crash/样本不真实 → 按 deferred 处理）· 只写与结论匹配的那个
- 其余变体作为附录保留（供 reviewer 对比）
- ADR 需经独立评审 + Arbiter 拍板后才 accepted

**Phase F · 报告 + 归档（0.5d）**：

- 撰写 `docs/spikes/SPIKE-07-report.md`
- 归档 `docs/spikes/code/SPIKE-07/` 代码（进 git · 加 `README.md` + `Cargo.lock` + 运行说明）
- 生成 raw data 归档到 `docs/spikes/raw/SPIKE-07/`（进 git · parser 输出日志 + profiling + `README.md` 索引）
- 创建 cold backup `spike-tmp/archive/SPIKE-07/`（🟡 推荐 · gitignored · 纯 Cargo 可省）

**风险缓冲**：

- 若 Phase B parser 对 happy path 都失败 → 说明 CLI 输出格式与预期完全不同 → 延期 1d 调研格式 → 触发 fail signal #4
- 若 Phase C 某场景大面积失败 → 延期 0.5d 迭代 parser → 若仍失败 → 走 Fallback 路径 2 或 3
- 若 Phase D 两 CLI 差异度超预期 → 延期 0.5d 设计 adapter 层 → 评估维护成本

---

## §L · 风险表（扩展至 5 行 + mitigation）

| #   | 风险                                                                                                 | 概率 | 影响 | Mitigation                                                                             |
| --- | ---------------------------------------------------------------------------------------------------- | ---- | ---- | -------------------------------------------------------------------------------------- |
| 1   | **R1 不降级**：parser 准确率不达标（< 90%）→ AI-Aware 整个 v1.0 vision 要重新规划                    | 中   | 高   | 提前在 §H 路径 3 中写好推迟方案；v0.3 后继续打磨基础卖点（Git 工作台 + 终端）          |
| 2   | **样本不够真实**：SPIKE-06 录制时覆盖的场景 miss 了边界（如 CLI 更新后协议变 · 长 session 状态管理） | 中   | 高   | spike 实施时若 Unrecognized > 20% → 触发补充录制；报告显式声明"样本量限制"             |
| 3   | **Parser 迁移性**：原型 parser 写死对当前 CLI 版本 → CLI 大版本升级可能 break                        | 高   | 中   | IR 设计预留版本字段；parser 模块化（协议检测 + 解析逻辑分离）；建立 CLI 版本兼容性矩阵 |
| 4   | **SPIKE-06 样本损坏**：36 条样本在 repo 中意外损坏或被覆盖                                           | 低   | 中   | 样本文件加 checksum（SHA-256）；fixture loader 启动时校验；损坏则从 cold backup 恢复   |
| 5   | **3d 工期不足**：parser 复杂度超预期，3d 内无法达到准确率门槛                                        | 中   | 中   | Phase 拆分允许单 phase 延期；总延期不超过 1d；超期则按当前结果写 report（即使未达标）  |

---

## §M · ADR-017 起草模板（spec 内附 outline）

> ⚠️ **详化阶段声明**：以下为 ADR-017 的起草用 outline，ADR-017 实际开 PR 是 spike 跑完后的事。本 section 只提供模板。

### ADR-017 outline（4 路径对应）

**路径 A · 通过（greenlight）**：

```
# ADR-017: AI-Aware v1.0 greenlight

## 状态：proposed → accepted（需独立评审 + Arbiter 拍板）

## 背景
SPIKE-07 基于 SPIKE-06 的 36+ 脱敏样本验证 parser 可行性，结果如下：
- 整体正确率：X%（≥ 96%）
- 两 CLI 统一抽象：可行 / 不可行

## 决策
- R1 从 HIGH/HIGH 降级到 [LOW/LOW | MEDIUM/LOW]
- 启动 MVP-18/19/20 详化
- 更新 CLAUDE.md 决策表 #3

## 约束
- Parser 原型代码归档到 `docs/spikes/code/SPIKE-07/`（进 git · 见 §I 3 样必交）· v1.0 实施时重写生产级 parser
- 持续监控 CLI 版本升级对 parser 的影响
```

**路径 B · 部分通过（single-cli）**：
同上，但只支持一个 CLI，另一个 CLI 推到 v2。

**路径 C · 双失败（deferred）**：
同上，但 R1 保留，AI-Aware 推迟。

**路径 D · 意外（parser crash / 样本不真实）**：

```
# ADR-017: AI-Aware v1.0 deferred — 技术前提不满足

## 状态：proposed → accepted

## 背景
SPIKE-07 遇到 [parser crash / 样本不真实 / 其他意外]，无法得出可信结论。

## 决策
- R1 保留
- 需补充 [新 Spike / 新样本录制] 后再评估
- 暂定下次评估时间：v0.3 GA 后
```

---

## §N · 自审四问

1. **递归完备性**：
   - 6 场景 × 2 CLI = 12 case，每 case 3 断言，合计 36 条核心断言 ✅
   - 3 条 fallback 路径（通过 / 部分 / 双失败）各自实化操作 ✅
   - 4 样齐全归档清单已列 ✅

2. **反向场景**：
   - parser crash → 迭代修复 ✅
   - 样本不真实 → 补充录制 ✅
   - 两 CLI 差异过大 → 只支持一个 ✅
   - 3d 工期不足 → 延期 1d 或按当前结果写 report ✅

3. **边界适用性**：
   - 本 Spike 只测回放样本，不测 live stream ✅
   - parser 原型不进 main，避免技术债 ✅
   - 详化阶段不写具体准确率数字，只写门槛 ✅

4. **YAGNI**：
   - 不做完整 AI-Aware 实现（MVP-18/19/20 范围）✅
   - 不做第三方 CLI 支持（v2+ 评估）✅
   - 不写 ADR-017 正文（spike 跑完后的事）✅
   - 不锁死 parser crate 选型（spike 实施时按实测定）✅

---

> **填写完毕后自审**：
>
> 1. 递归完备性：主线 parser + 副线 R1 降级 proposal 都覆盖 ✅
> 2. 反向场景：3 条 fallback 路径 + 5 条风险 mitigation 都覆盖 ✅
> 3. 边界适用性：详化阶段 vs spike 实施阶段边界清楚 ✅
> 4. YAGNI：不做生产级 parser、不做 ADR-017 正文、不 flip status ✅
