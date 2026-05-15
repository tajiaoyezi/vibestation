---
id: SPIKE-07.5
type: spike
title: CLI 结构化模式重录重跑 spike（路径 A · R1 新 gate）
status: ready
status_note: "ready-gate APPROVE-WITH-NITS（#341 · 3 nit 修+re-reviewed）· Arbiter 2026-05-16 拍板 '批准 flip'（翻转 gate 选项 a · reviewer push 翻转 commit）· 实施（重录结构化 corpus + parser 适配 + 重跑 §F + 重判 §H）gated on Arbiter 自定执行窗口（API 预算 · 同 SPIKE-06 录制模式）· 前置 SPIKE-07 done ✅"
owner:
phase: v1.0-pre
depends_on: ["SPIKE-07"]
depends_on_notes: "SPIKE-07 = done（§H 路径 3 deferred · ADR-017 accepted 2026-05-16）· 本 spike 复用 SPIKE-07 已证 sound 的 CliEvent IR + CliParser trait + assertions/matrix harness（docs/spikes/code/SPIKE-07/）· SPIKE-06 corpus 仅作交互 TUI 基线对照（不复用其样本 · 本 spike 重录结构化模式新 corpus）"
blocks: ["MVP-18", "MVP-19", "MVP-20"]
blocked_by: []
blocked_from:
blocked_note:
estimate: 1d
plan_ref: implementation-plan.md §5.3.6 · §9 R1 · §1.1
risk_ref: R1
reviewer: "ready-gate（2026-05-16）= Claude Code 对抗性预审（基于 SPIKE-07 全 crate 实读）· verdict APPROVE-WITH-NITS · 3 nit 修+re-reviewed @ #341 · self-review v2-D.2 + Arbiter tajiaoyezi 拍板 '批准 flip'（翻转 gate 选项 a）"
---

# SPIKE-07.5: CLI 结构化模式重录重跑（路径 A · R1 新 gate）

> **状态**：`ready`（ready-gate APPROVE-WITH-NITS @ #341 · Arbiter 2026-05-16 拍板 flip · 实施待 Arbiter 自定执行窗口 · API 预算同 SPIKE-06）
> **前置**：[SPIKE-07](./SPIKE-07-cli-protocol-parser.md)（done · §H 路径 3 deferred）· [ADR-017](../adr/ADR-017-ai-aware-deferred.md)（accepted · 选定路径 A）
> **阻塞**：MVP-18/19/20（AI-Aware 三件套 · v1.0 vision · 实施新前置 = 本 spike 实跑 PASS）
> **战略依据**：ADR-017 §决策 5 路径 A · `implementation-plan.md §9 R1`

---

## §A · 目标（Goal）

SPIKE-07 实测结论：§H 路径 3 deferred · 但**根因 = corpus 方法论 artifact**（SPIKE-06 录的是两 CLI 的**交互 TUI 模式** = 屏幕重绘 blob），**非 AI-Aware 产品前提推翻 · 非 parser 实现差**（SPIKE-07 happy/auth/network/interrupt 4/6 场景 100% · 0 panic · 统一 IR 抽象 §E.4 可行）。

路径 A 前置已实测确认（SPIKE-07 report §路径 A 调研结果 · raw `path-a-cli-modes-recon.txt`）：

- **Claude CLI** 有 `-p --output-format stream-json --include-hook-events --include-partial-messages`（realtime 结构化 JSON 事件流 · 几乎为 AI-Aware 量身定制）
- **Codex CLI** 有 `exec`（非交互）· `remote-control`（headless app-server）

本 spike 用**结构化模式**重录 corpus，复用 SPIKE-07 已证 sound 的 parser+IR 重跑 §F 矩阵，重走 §H 三路径——判定 R1 能否从 HIGH/HIGH 真降级以解锁 MVP-18/19/20。

**翻盘业务价值**：SPIKE-07 deferred 是对 TUI corpus 的诚实判定；若结构化 corpus 下 §H 走路径 1/2 → R1 降级 → AI-Aware v1.0 从"远期愿景"变"可排期" → 写新 ADR supersede ADR-017。

---

## §B · 背景（Context）

### 与 SPIKE-07 的关系（复用 vs 新建）

| 物料                                      | SPIKE-07（done）                                       | SPIKE-07.5 策略                                                                                                                                     |
| ----------------------------------------- | ------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ir.rs`（CliEvent IR + CliParser trait）  | `docs/spikes/code/SPIKE-07/src/ir.rs` 已锁             | **可复用 · 不改契约**（§E.4 已证两 CLI 共享同一 IR · format-agnostic）                                                                              |
| `assertions.rs`（§F 断言）                | `src/assertions.rs`（吃 `&[CliEvent]`）                | **可复用 as-is**（断言纯函数 · 与样本格式无关 · TDD 已验 13 测试）                                                                                  |
| `matrix.rs` harness 聚合逻辑              | `src/bin/matrix.rs`（场景/CLI/整体/§E.11/§H 判定）     | **结构可复用** · 但 corpus 加载入口须换（见下行）                                                                                                   |
| `cast.rs` + `fixture.rs`（corpus loader） | asciinema **v3 `.cast`** 解码 + glob `*.redacted.cast` | **不可复用**（.cast 专用 · 结构化 corpus 是 JSON-lines）· SPIKE-07.5 **新写 `.structured.jsonl` loader**                                            |
| parser adapter（claude/codex）            | TUI 屏幕重绘解析（薄/厚）                              | **新增结构化模式解析路径**（claude `stream-json` JSON-lines · codex `exec` 输出 · 复用 `CliParser` trait）                                          |
| corpus                                    | SPIKE-06 36 条交互 TUI `.redacted.cast`                | **新录 36 条结构化模式样本** → `docs/spikes/raw/SPIKE-07.5/`（不复用 SPIKE-06）                                                                     |
| 代码目录                                  | `docs/spikes/code/SPIKE-07/`                           | **新建 `docs/spikes/code/SPIKE-07.5/`**（复用 ir.rs+assertions.rs+matrix 聚合 · 新写 .jsonl loader + 结构化 parser · SPIKE-07 不动作 TUI 基线对照） |

### SPIKE-07 deferred 三类根因（本 spike 针对性消除）

1. corpus 质量（claude long_stream = TUI 屏幕重绘 blob）→ 结构化模式 `stream-json` 是行式 JSON 事件 · 无屏幕重绘
2. §F 断言对厚协议校准失配（codex 差 2-3pp）→ 结构化输出无 TUI 脚手架 · 95% content 阈值可正常评估
3. 协议现实（两 CLI TUI 模式不发机器 JSON）→ `--output-format stream-json` / `codex exec` 正是机器协议

---

## §C · 功能范围（Scope）

### Do（必做）

1. **结构化 corpus 重录**（6 场景 × 2 CLI × 3 take = 36 条）：
   - Claude：`claude -p --output-format stream-json --include-hook-events --include-partial-messages "<scenario prompt>"` · stdout 重定向捕获（非 PTY asciinema · 因结构化模式输出是 JSON-lines 非屏幕流）
   - Codex：`codex exec "<scenario prompt>"` 非交互 · stdout 捕获
   - 6 场景沿用 SPIKE-06 定义：happy_path / interrupt_residual / auth_fail / network_error / long_stream / mixed_ansi_json
   - ⚠️ **`interrupt_residual` 结构化模式语义退化**（`claude -p` print-and-exit / `codex exec` 非交互 · 无交互 TUI 残帧概念）：本场景**重定义**为「流式输出中途 SIGTERM → 捕获已发的部分结构化事件序列 · 验 parser 优雅处理截断（不 panic · 末事件非悬空 start）」· **不**测 TUI 残帧解析。report 须显式记此语义差异（对齐 §risks #1 / §E fail #2）· §H 判定时该场景按重定义后的断言评估
   - 脱敏：沿用 SPIKE-06 脱敏纪律（删 token/key/JWT/PII/本地路径/git remote · 保协议结构占位）· 同名 `.redaction.json` sidecar
   - 命名：`{cli}_{scenario}_{take}.structured.jsonl`（区别于 SPIKE-06 `.redacted.cast`）
2. **结构化模式 parser 适配**：在 `docs/spikes/code/SPIKE-07.5/` 复用 SPIKE-07 `CliEvent` IR + `CliParser` trait · 新增 claude `stream-json`（JSON-lines · 每行一 event · `--include-hook-events` 含 hook 生命周期）+ codex `exec` 输出解析
3. **§F 矩阵重跑**：复用 SPIKE-07 `assertions.rs`（§F 断言纯函数 · as-is）+ `matrix.rs` 聚合逻辑（12 case×3 + §E.11 基线 + §H 判定）· **新写 `.structured.jsonl` corpus loader 替代 `cast.rs`+`fixture.rs`**（.cast 专用不可复用 · 见 §B 表）· 对结构化 corpus 跑
4. **§H 三路径重判**：基于结构化 corpus 实测重走 SPIKE-07 §H（single source of truth）
5. **报告**：`docs/spikes/SPIKE-07.5-report.md`（结构化 vs TUI corpus 对照 + 准确率 + §H 重判 + R1 proposal）

### Don't（明确不做）

1. 不改 SPIKE-07 `docs/spikes/code/SPIKE-07/`（留作 TUI corpus 基线对照 · 归档不动）
2. 不改 `CliEvent` IR / `CliParser` trait 契约（SPIKE-07 §E.4 已证 sound · 改则两 spike 不可比）
3. 不复用 SPIKE-06 `.redacted.cast`（那是 TUI 屏幕重绘 · 本 spike 要的是结构化模式新 corpus）
4. 不自 accept 新 ADR / 不自 flip CLAUDE.md 决策表 #3 / 不自 flip MVP-18/19/20 status（§2.1 · Arbiter 拍板）
5. 不重写为生产 parser（归档级原型 · spike-delivery-checklist 3 样必交进 git · 不进 `crates/`）

---

## §D · 通过标准（Pass Criteria）· 复用 **SPIKE-07 §H** 三路径（R1 single source of truth）

> 📌 消歧：「三路径」R1 判据始终指 **SPIKE-07 §H**（[`SPIKE-07-cli-protocol-parser.md`](./SPIKE-07-cli-protocol-parser.md) §H · session 32 Arbiter 钦定）。本 SPIKE-07.5 spec 自身的 `§H` 段是「交付物归档」（见下方），与 R1 判据无关。

R1 降级判据 = SPIKE-07 §H 三路径（session 32 Arbiter 钦定 · 本 spike 沿用 · 对结构化 corpus 实测重判）：

- **路径 1 greenlight**：整体加权 ≥ 96% 且 各场景 ≥ 90% 且 两 CLI 可统一 → 写新 ADR（greenlight）supersede ADR-017 · R1 降级 MEDIUM/LOW
- **路径 2 single-cli**：一 CLI ≥ 96% · 另一 < 90% → 写新 ADR（single-cli）supersede ADR-017 · R1 部分降级
- **路径 3 deferred**：两 CLI 均 < 90% 或 任一场景 < 85% → ADR-017 deferred 维持 · R1 保留 · 记录结构化模式仍不达标的根因

§E.3/§E.5 informative 诊断指标沿用 SPIKE-07（与 §H 冲突以 §H 为准）。

---

## §E · 失败信号（Fail Signals）

1. 结构化模式实际不发可解析事件（与 `--help` 声称不符）→ 记录实测 · 触发协议现实复评
2. 重录无法复现某场景（如 interrupt 在 `-p` 模式语义不同）→ 报告显式标注 · 该场景降级处理
3. parser 对结构化流大面积 Unrecognized（> 20%）→ 适配不足 · 迭代或记录局限
4. 结构化 corpus 仍 §H 路径 3 → AI-Aware 前提真有问题（非 corpus artifact）· ADR-017 deferred 强化 · 可能长期推迟

---

## §F · Fallback 方案

| 结果                      | 操作                                                                                                             |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| 路径 1/2（翻盘）          | 写 `ADR-018-ai-aware-{greenlight,single-cli}.md` proposed → Arbiter 拍板 → supersede ADR-017 → 解锁 MVP-18/19/20 |
| 路径 3（结构化仍不达标）  | ADR-017 deferred 维持 · 报告记录"结构化模式实测仍 < 阈值"根因 · AI-Aware 推 v2+                                  |
| 结构化模式不可用 / 不可录 | 报告记录实测 · ADR-017 deferred 终局化 · 路径 C（v1.0 不含 AI-Aware）                                            |

---

## §G · Acceptance（PR body 逐项 · 独立评审验证）

- [ ] 36 条结构化 corpus 重录完成（6 场景 × 2 CLI × 3 take · `docs/spikes/raw/SPIKE-07.5/` · 进 git · 脱敏 + sidecar）
- [ ] `docs/spikes/code/SPIKE-07.5/` 复用 SPIKE-07 IR/trait/harness · 新增结构化模式 parser · `cargo test` 全过 · clippy `-D warnings` · fmt clean
- [ ] §F 矩阵对结构化 corpus 实跑 · 0 panic · 结果溯源 `docs/spikes/raw/SPIKE-07.5/matrix.json`
- [ ] **SPIKE-07 §H** 三路径重判（逐路径核对 · 实测数字驱动 · 0 fabrication）
- [ ] `docs/spikes/SPIKE-07.5-report.md`：结构化 vs TUI 对照 + 准确率 + §H 重判 + R1 proposal + 置信度 caveat
- [ ] 新 ADR（若翻盘）proposed · §2.1 不自 accept · Arbiter 拍板
- [ ] spike-delivery-checklist 3 样必交（report + code + raw 全进 git · 同 PR 原子归档）
- [ ] 每数字 raw 可溯源 · clone 后 `cargo run --bin matrix` 可复现

---

## §H · 交付物归档（3 样必交 · 对齐 `.claude/rules/spike-delivery-checklist.md`）

> 📌 本 `§H` = SPIKE-07.5 交付物归档段。R1 降级判据的「§H 三路径」始终指 **SPIKE-07 §H**（见 §D 消歧）· 勿混。

| #   | 物料     | 路径                               | 进 git |  级别   |
| --- | -------- | ---------------------------------- | :----: | :-----: |
| 1   | 决策文档 | `docs/spikes/SPIKE-07.5-report.md` |   ✅   | 🔴 必须 |
| 2   | 实测源码 | `docs/spikes/code/SPIKE-07.5/`     |   ✅   | 🔴 必须 |
| 3   | Raw 数据 | `docs/spikes/raw/SPIKE-07.5/`      |   ✅   | 🔴 必须 |
| 4   | 冷备     | `spike-tmp/archive/SPIKE-07.5/`    |   ❌   | 🟡 推荐 |

accept 原子性：判定 → report 入库 → 源码归档 → raw 归档 → ADR 翻转（若有）→ spec done（独立评审后）· 同一主 agent 动作内 · 不跨 session 拆。

---

## §I · 依赖资源（Resources Needed）

- Claude CLI（实测 2.1.142 · `-p --output-format stream-json` 可用）+ Codex CLI（实测 0.130.0 · `exec` 可用）· **真实 CLI 调用消耗 API 预算**（重录 36 样本 · auth_fail/network_error 场景需构造失败态）—— Arbiter 知情 · 执行窗口由 Arbiter 决定（同 SPIKE-06 录制模式）
- SPIKE-07 `docs/spikes/code/SPIKE-07/`（复用 IR/harness · 只读参考）
- 脱敏工具（沿用 SPIKE-06 流程 · gitleaks 扫描）

---

## ⚠️ 已知风险

| #   | 风险                                                        | Mitigation                                                          |
| --- | ----------------------------------------------------------- | ------------------------------------------------------------------- |
| 1   | `-p` / `exec` 模式语义与交互模式不同（如 interrupt 无残帧） | 报告显式记录场景语义差异 · 该场景按结构化模式实际行为重定义断言     |
| 2   | API 预算消耗（36 真实 CLI 调用）                            | 单次调用成本低 · 失败场景用无效 key / 断网模拟 · Arbiter 控执行窗口 |
| 3   | 结构化模式 corpus 仍 §H 路径 3                              | 即 AI-Aware 前提真有问题 · 报告如实记录 · ADR-017 deferred 强化     |
| 4   | parser 适配 stream-json 复杂度（与 TUI 解析不同代码路径）   | 复用 SPIKE-07 IR 契约 · 仅新增解析入口 · §E.4 已证 IR sound         |

---

## §J · 自审四问

1. **递归完备性**：本 spike 复用 SPIKE-07 §H single source of truth · 不另立判据 ✅
2. **反向场景**：结构化 corpus 仍 deferred → 如实记录（fail signal #4）· 不粉饰翻盘 ✅
3. **边界适用性**：6 场景 × 2 CLI 全覆盖 · 结构化模式语义差异显式处理 ✅
4. **YAGNI**：复用 SPIKE-07 IR/harness · 只新增结构化 parser + 重录 corpus · 不重建 ✅

---

> **本 spec 为 draft · 待独立 ready-gate 评审**（参照 SPIKE-07 session 32 ready-gate 流程：预审 → 跨 spec 核实 → 决策表 → Arbiter approve flip draft→ready）。实施前置：SPIKE-07 done（✅ 已满足）。
