---
id: SPIKE-07
type: spike
title: CLI 输出协议 parser 验证 spike（R1 降级前置）
status: draft
owner:
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
reviewer: OpenCode · self-review · §2.10 evidence-based
---

# SPIKE-07: CLI 输出协议 parser 验证

> **状态**：`draft`（v1.0-pre · **详化中**，非 MVP v0.1/v0.2/v0.3 范围）
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

- 写 **ADR-011-ai-aware-greenlight.md** · R1 降级 proposal → accepted
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
- **存储位置**：`docs/spike-artifacts/SPIKE-06/`（脱敏后 · 进 repo）+ `~/.vibestation-spike-raw/SPIKE-06/`（原始未脱敏 · 不进 repo）

**样本结构示例**（Claude CLI · happy path · take-1）：

```
docs/spike-artifacts/SPIKE-06/
├── claude/
│   ├── happy-path-01.txt      # 启动 + 简单对话输出
│   ├── happy-path-02.txt      # 同上 · macOS 录制
│   ├── happy-path-03.txt      # 同上 · Linux 录制
│   ├── interrupt-01.txt       # Ctrl+C 中断后残帧
│   ├── auth-fail-01.txt       # 错误 token 导致的认证失败
│   ├── network-error-01.txt   # 断网场景
│   ├── long-stream-01.txt     # 10k+ token 响应片段
│   └── mixed-ansi-json-01.txt # ANSI 颜色 + JSON 混合
└── codex/
    └── ...（同上 6 场景 × 3 次）
```

SPIKE-06 的结论严格区分为"CLI 能在 PTY 里运行"（结论 A）和"协议足够清楚可指导实现"（结论 B）。**结论 B 本 Spike 不验证，R1 保留**。SPIKE-07 是在 SPIKE-06 的样本基础上，回答"给定这些真实输出，能否稳定解析为结构化事件"。

**关键认知**：SPIKE-06 只回答了"CLI 输出长什么样"，SPIKE-07 要回答"机器能不能稳定理解这些输出"。这是从"观察"到"工程可行性"的跃迁。

### ADR-009 决策依据

[ADR-009](../adr/ADR-009-ai-aware-v1-vision.md) 明确：

> AI-Aware = v1.0 vision · README / landing / Twitter / Discord 完全不宣传 · 直到 v1.0 真实落地再讲
> 技术前提：SPIKE-07 parser-oriented spike 必通过（基于 SPIKE-06 录制的 36+ 样本 · parsed_issues 解析准确率 ≥ 95%）
> SPIKE-07 通过 → 写 ADR-011-ai-aware-greenlight.md · 才能启动 MVP-18/19/20 详化

ADR-009 还规定：

- ❌ 禁止对外文案提及 AI-Aware Pane / Mission Control / AI session aware
- ✅ 允许：内部技术文档（ADR / implementation-plan / tasks/MVP-18..20）明确标注"v1.0 vision"
- **R1 降级授权只能通过 SPIKE-07 的 ADR 完成**，SPIKE-06 无权下调 R1

### 通过 / 不通过对应 ADR-011 路径

| 本 Spike 结果                            | ADR-011 文件名                   | 内容                                                    | 后续影响                                                            |
| ---------------------------------------- | -------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------------- |
| 通过（全指标达标）                       | `ADR-011-ai-aware-greenlight.md` | R1 降级到 MEDIUM/LOW · parser 可行 · v1.0 AI-Aware 开工 | MVP-18/19/20 详化解锁 · `CLAUDE.md` 决策表 #3 移除 ⚠️ 警告          |
| 部分通过（一个 CLI 能 parse 另一个不能） | `ADR-011-ai-aware-single-cli.md` | 只支持能 parse 的那个 · README / landing 需说明         | MVP-18/19/20 详化解锁（仅限支持的 CLI）· 另一个 CLI 推到 v2         |
| 双失败                                   | `ADR-011-ai-aware-deferred.md`   | R1 不降级 · AI-Aware 推迟到 v2+                         | MVP-18/19/20 status 保持 draft · `CLAUDE.md` 决策表 #3 保留 ⚠️ 警告 |

---

## §C · 功能范围（Scope）

### Do（必做）

1. **Fixture loader**：读取 `docs/spike-artifacts/SPIKE-06/` 目录下的 36+ 脱敏样本文件，按 `{cli, scenario, take}` 三维索引建立 fixture 注册表
2. **原型 parser 实现**：在 Rust `core` crate 或独立 `spike-tmp` 目录实现原型 parser（不锁死 parser 库，先用正则/状态机 combo，spike 实施时按实测选型）
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
5. **parser 长期维护**：原型代码用完即归档到 `spike-tmp/`，不 merge 到 main 的 `crates/`（避免技术债）
6. **ADR-011 正式撰写**：spec 内可附 ADR outline 模板（§M），但 ADR-011 实际开 PR 是 spike 跑完后的事

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

- [ ] Fixture loader 能正确读取 `docs/spike-artifacts/SPIKE-06/` 全部 36+ 样本文件
- [ ] 每个样本附 metadata：{cli, scenario, take, redacted_fields, original_size, redacted_size}
- [ ] 样本格式统一为 UTF-8 文本文件，每行一条原始输出（或按 SPIKE-06 约定分隔）
- [ ] Fixture 注册表能按 `cli=claude, scenario=auth-fail` 查询到所有匹配样本

### E.2 · Parser 覆盖率

- [ ] Parser 对 36+ 样本全部执行解析（无 crash、无 panic）
- [ ] 覆盖 6 场景 × 2 CLI = 12 场景组合，每组合至少 3 条样本
- [ ] 每样本至少生成 1 个 `CliEvent`（空输出视为异常，需人工审计）
- [ ] `Unrecognized` 事件比例 ≤ 10%（总事件数中 ≤ 10% 为未识别）

### E.3 · 准确率门槛

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

### E.5 · R1 降级 proposal

- [ ] 若整体准确率 ≥ 99% → 建议 R1 降级到 LOW/LOW
- [ ] 若整体准确率 ≥ 95% 但 < 99% → 建议 R1 降级到 MEDIUM/LOW
- [ ] 若整体准确率 < 95% 或任一失败路径 < 90% → 建议 R1 保留
- [ ] 若两 CLI 无法统一抽象 → 建议 R1 降级到 MEDIUM/LOW 但限制只支持一个 CLI

### E.6 · 报告质量

- [ ] `docs/spikes/SPIKE-07-report.md` 包含：准确率数据表、统一抽象分析、R1 降级建议
- [ ] 报告中 0 条 fabricated 数据（所有数字来自实测）
- [ ] 报告明确标注"样本量=36+"和"置信度限制"
- [ ] 样本不够真实的 caveat 在报告中显式声明

### E.7 · 代码质量

- [ ] 原型 parser 代码在 `spike-tmp/spike-07-parser/` 目录，不进 `crates/`
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

## §H · Fallback 方案（3 路径）

保留占位中已有的 3 路径，各自实化操作：

### 路径 1 · 通过（全指标达标）

**条件**：

- 整体加权正确率 ≥ 96%
- 各场景正确率均 ≥ 90%
- 两 CLI 可统一抽象（或 adapter 层成本可接受）

**操作**：

1. 写 `ADR-011-ai-aware-greenlight.md` · 提议 R1 降级到 MEDIUM/LOW（或 LOW/LOW）
2. ADR accepted 后 → MVP-18/19/20 status 从 draft 翻 ready（由 main agent 操作，不自行 flip）
3. 更新 `CLAUDE.md §决策表 #3`：移除 ⚠️ 警告
4. 原型 parser 归档到 `spike-tmp/spike-07-parser/`（不进 main）
5. 报告归档到 `docs/spikes/SPIKE-07-report.md`

### 路径 2 · 部分通过（一个 CLI 能 parse 另一个不能）

**条件**：

- 一个 CLI 整体正确率 ≥ 96%
- 另一个 CLI 整体正确率 < 90% 或无法统一抽象

**操作**：

1. 写 `ADR-011-ai-aware-single-cli.md` · 提议只支持能 parse 的 CLI
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

1. 写 `ADR-011-ai-aware-deferred.md` · 提议 R1 不降级 · AI-Aware 推迟到 v2+
2. MVP-18/19/20 保持 draft 状态，不进入 in-progress
3. `CLAUDE.md §决策表 #3` 保留 ⚠️ 警告（甚至加强措辞）
4. v1.0 卖点回归"多 Tab 终端 + Git 工作台"（基础差异化）
5. 若后续 CLI 版本升级后协议更结构化 → 重新评估启动新 Spike

---

## §I · 4 样齐全归档（Spike Delivery Checklist）

> ⚠️ **详化阶段声明**：以下清单为 spike **跑完后**的归档要求。详化阶段只写清单模板，不实际产生交付物。

参照 `docs/spikes/spike-delivery-checklist.md`（如存在）或 SPIKE-04/05/06 的归档模式：

| #   | 交付物          | 路径                                 | 说明                                                   |
| --- | --------------- | ------------------------------------ | ------------------------------------------------------ |
| 1   | **Report**      | `docs/spikes/SPIKE-07-report.md`     | 准确率数据表 + 统一抽象分析 + R1 降级建议              |
| 2   | **Code**        | `spike-tmp/spike-07-parser/`         | 原型 parser + fixture loader + 测试脚本                |
| 3   | **Raw data**    | `~/.vibestation-spike-raw/SPIKE-07/` | 原始 parser 输出日志、性能 profiling 数据（不进 repo） |
| 4   | **Cold backup** | 本地压缩包或云存储                   | parser 代码 + 报告 + fixture 快照的只读备份            |

**归档要求**：

- [ ] Report 含所有原始数据表格（不可编辑的图片或 CSV attach）
- [ ] Code 能在干净机器上 `cargo run --example replay-fixtures` 复现结果
- [ ] Raw data 含 parser 对每条样本的完整输出（用于后续审计）
- [ ] Cold backup 至少保留 1 年（到 v1.0 发布后）

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

| Phase | 任务                                                 | 估时 | 阻塞项  | 产出                                                 |
| ----- | ---------------------------------------------------- | ---- | ------- | ---------------------------------------------------- |
| A     | Fixture loader 实现：读取 SPIKE-06 样本 + 建立注册表 | 0.5d | 无      | fixture registry + 样本验证脚本                      |
| B     | Parser MVP：手写状态机解析 happy path（Claude 先）   | 0.5d | Phase A | Claude happy path 解析通过                           |
| C     | 6 场景断言：对 36+ 样本逐条跑断言 + 记录失败         | 0.5d | Phase B | 12 case × 3 样本 = 36 条结果                         |
| D     | 准确率统计 + 统一抽象分析 + 两 CLI 对比              | 0.5d | Phase C | 准确率数据表 + IR 差异清单                           |
| E     | ADR-011 起草（基于 Phase D 结论）                    | 0.5d | Phase D | ADR-011 草稿（3 种路径对应 3 个版本）                |
| F     | 报告撰写 + 代码清理 + 归档                           | 0.5d | Phase E | `docs/spikes/SPIKE-07-report.md` + `spike-tmp/` 归档 |

**合计**：3d（含 0.5d buffer）

**Phase 详细说明**：

**Phase A · Fixture loader（0.5d）**：

- 读取 `docs/spike-artifacts/SPIKE-06/` 目录树
- 按文件名约定解析 metadata：`{cli}-{scenario}-{take}.txt`
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

**Phase E · ADR-011 起草（0.5d）**：

- 基于 Phase D 结论，按 §M 模板起草 ADR-011
- 3 种路径对应 3 个 ADR 版本，只写与结论匹配的那个
- 其余两个版本作为附录保留（供 reviewer 对比）
- ADR 需经独立评审 + Arbiter 拍板后才 accepted

**Phase F · 报告 + 归档（0.5d）**：

- 撰写 `docs/spikes/SPIKE-07-report.md`
- 清理 `spike-tmp/spike-07-parser/` 代码（加 README + 运行说明）
- 生成 raw data 归档（parser 输出日志 + profiling 数据）
- 创建 cold backup（压缩包）

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

## §M · ADR-011 起草模板（spec 内附 outline）

> ⚠️ **详化阶段声明**：以下为 ADR-011 的起草用 outline，ADR-011 实际开 PR 是 spike 跑完后的事。本 section 只提供模板。

### ADR-011 outline（4 路径对应）

**路径 A · 通过（greenlight）**：

```
# ADR-011: AI-Aware v1.0 greenlight

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
- Parser 原型代码归档到 spike-tmp/，v1.0 实施时重写生产级 parser
- 持续监控 CLI 版本升级对 parser 的影响
```

**路径 B · 部分通过（single-cli）**：
同上，但只支持一个 CLI，另一个 CLI 推到 v2。

**路径 C · 双失败（deferred）**：
同上，但 R1 保留，AI-Aware 推迟。

**路径 D · 意外（parser crash / 样本不真实）**：

```
# ADR-011: AI-Aware v1.0 deferred — 技术前提不满足

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
   - 不写 ADR-011 正文（spike 跑完后的事）✅
   - 不锁死 parser crate 选型（spike 实施时按实测定）✅

---

> **填写完毕后自审**：
>
> 1. 递归完备性：主线 parser + 副线 R1 降级 proposal 都覆盖 ✅
> 2. 反向场景：3 条 fallback 路径 + 5 条风险 mitigation 都覆盖 ✅
> 3. 边界适用性：详化阶段 vs spike 实施阶段边界清楚 ✅
> 4. YAGNI：不做生产级 parser、不做 ADR-011 正文、不 flip status ✅
