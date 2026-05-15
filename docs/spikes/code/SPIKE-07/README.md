# SPIKE-07 · CLI 输出协议 parser 验证原型

> **归档级原型**（SPIKE-07 spec §C Don't.5：用完归档 · 不进 `crates/` · 不重写生产 parser）
> 上位 spec：[`docs/tasks/SPIKE-07-cli-protocol-parser.md`](../../../tasks/SPIKE-07-cli-protocol-parser.md)
> 交付物归档规则：[`.claude/rules/spike-delivery-checklist.md`](../../../../.claude/rules/spike-delivery-checklist.md)（3 样必交）

## 来源

- **实施 agent**：Claude Code（主 agent）· 单人项目 v2-D.2
- **产出时间**：2026-05-15（session 32 · Arbiter 选 SPIKE-07 实跑 · R1 gate）
- **corpus 来源**：SPIKE-06 PR #71 · `docs/spikes/raw/SPIKE-06/` 36 条 `*.redacted.cast`（2 CLI × 6 场景 × 3 take）

## 阶段状态

| Phase   | 范围                                                                                                                  | 状态                               |
| ------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| **A**   | corpus 画像 + cast v3 解码 + fixture loader + CliEvent IR 契约 + CliParser trait + StubParser + survey/replay harness | ✅ **本次完成** · 主 agent 单做    |
| **B**   | `parser::claude`（薄协议 adapter）+ `parser::codex`（厚结构 adapter）实现 `CliParser`                                 | ⏳ dispatch 2 路并行（文件域隔离） |
| **C**   | §F 测试矩阵 36 样本逐条断言                                                                                           | 主 agent 收                        |
| **D**   | 准确率统计 + 统一抽象（Claude 薄 vs Codex 厚）                                                                        | 主 agent · decision-grade          |
| **E/F** | §H 三路径判定 → ADR-017 + report + 4 样齐全                                                                           | 主 agent + Arbiter                 |

## 复现命令

```bash
cd docs/spikes/code/SPIKE-07
cargo test                    # 13 tests · 含真实 36 corpus 完整矩阵集成 test
cargo run --bin survey        # 全 36 样本结构画像 → docs/spikes/raw/SPIKE-07/phase-a-survey.txt
cargo run --bin replay        # 端到端 harness（Phase A = StubParser · 验收点 0 panic）
```

Cargo.lock 进 git（版本冻结 · 任何机器 byte-level 复现）。`target/` gitignored。

## 模块契约（Phase B agent 必读）

- `src/cast.rs` — asciinema **v3** 解码（header + `[interval,"o"|"x"|...,data]` 事件 → 终端字节流 + 退出码）。**Phase B 不改**。
- `src/fixture.rs` — corpus loader（glob `*.redacted.cast` · 文件名 `{cli}_{scenario}_{take}` 三维索引 · `.redaction.json` sidecar）。**Phase B 不改**。
- `src/ir.rs` — **接口契约（Phase B 不得改 enum / trait 签名）**：
  - `CliEvent`（MessageStart/Delta/End · Error · SessionMeta · Hook · Usage · ToolUse\* · **Unrecognized 强制兜底**）
  - `trait CliParser { fn cli_id() -> &str; fn parse(&ParseInput) -> ParseResult; }`
  - 契约：`parse` **不得 panic**；不认识的块包 `Unrecognized` 不丢弃；空输入返回空 vec
- `src/parser/claude.rs` — **Phase B track 1 独占**（`ClaudeParser::parse` · 薄协议）· agent 只编辑本文件
- `src/parser/codex.rs` — **Phase B track 2 独占**（`CodexParser::parse` · 厚结构）· agent 只编辑本文件
- `src/lib.rs::parser::for_cli` — Phase A 已锁路由到 `ClaudeParser`/`CodexParser`（真实类型 · impl 由 Phase B 填）· **Phase B 不改 lib.rs**（文件域硬隔离 · 2 track 0 共享文件写 · 规避 §2.15 stale-base）
- `src/bin/replay.rs` — Phase C harness（Phase B adapter 落地后**不改本文件**即出真实事件 · `catch_unwind` 兜底验证不 panic 契约）

## 关键结论溯源（Phase A 实测 · 喂 Phase D）

Phase A survey（`docs/spikes/raw/SPIKE-07/phase-a-survey.txt`）已定量证实 **两 CLI 协议结构截然不同**——这是 §E.4 统一抽象判定的核心证据：

- **Claude CLI = 薄协议**：`happy_path` 196–238B 纯 assistant 文本 + exit 0 · `auth_fail` 127B "Invalid API key" + exit 1 · **无 session id / 无 role marker / 无 hook**；错误靠文本模式 + exit≠0 识别
- **Codex CLI = 厚结构**：`session id:` 显式输出 · `user`/`codex` role marker 行 · `hook:` 生命周期 · `tokens used` footer
- **corpus 质量风险**（命中 spec §G fail #4）：claude `interrupt_residual`/`long_stream`/`mixed_ansi_json` 32–300KB 且 **exit 0**（名为 interrupt 却未中断）· codex `interrupt_residual_1` 339KB/exit 143（真 SIGTERM）但 take 2/3 仅 2KB/exit 0 · 跨 take 方差极大 → Phase D 必须显式记此 caveat
- 推论：统一抽象**必走 adapter 层**（共享 `CliEvent` IR + 两个 CLI-specific parser），非单一共享 parser。最终 §H 路径由 Phase D 实测准确率定，本 README 不预判结论。

## 设计决策（留 Phase B 实测）

| #   | 决策项           | Phase A 立场                                     | Phase B 定                         |
| --- | ---------------- | ------------------------------------------------ | ---------------------------------- |
| 1   | parser 实现手法  | 手写状态机 + 正则 combo（IR 已锁 · 手法自由）    | ✅ 各 adapter 自决                 |
| 2   | ANSI 处理        | IR 留 `raw_ansi: Option<String>` · 剥离策略不锁  | ✅ adapter 自决                    |
| 3   | Claude role 推断 | 薄协议 · 整段 stdout 视作 assistant（无 marker） | ✅ Claude adapter 实测确认         |
| 4   | ToolUse 是否出现 | 36 样本未见 function calling 输出                | ✅ 实测无则 adapter 不产 ToolUse\* |
