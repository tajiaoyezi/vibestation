# ADR-018: AI-Aware v1.0 vision R1 重判 — SPIKE-07.5 结构化模式实测推翻 corpus 方法论 deferral

**状态**：proposed
**日期**：2026-05-16（proposed）
**决策者**：Claude Code（作者 agent · 主 agent 跑 SPIKE-07.5 实测 · self-review v2-D.2 单人项目）· tajiaoyezi（Arbiter · 拍板待定）
**对应 `CLAUDE.md` 决策表**：A 栏 #3（AI-Aware Pane 联动 = v1.0 vision）· 本 ADR 提议**有条件降级 R1**（路径 1/2 · Arbiter 拍板后生效）· accept 前 #3 现状不变
**对应 Spike**：[SPIKE-07.5](../tasks/SPIKE-07.5-structured-mode-rerun.md) · [SPIKE-07.5-report](../spikes/SPIKE-07.5-report.md)
**前置 ADR**：[ADR-017](./ADR-017-ai-aware-deferred.md)（accepted · SPIKE-07 deferred · 选定路径 A）· 本 ADR accept 后 **supersede ADR-017**
**根 ADR**：[ADR-009](./ADR-009-ai-aware-v1-vision.md)（AI-Aware = v1.0 vision · R1 HIGH/HIGH 起源）

---

## 背景与问题（Context and Problem Statement）

ADR-017 accept 路径 A：SPIKE-07 §H 路径 3 deferred 判定为 **SPIKE-06 corpus 方法论 artifact**（录的是交互 TUI 屏幕重绘 · 非结构化协议）· 选定新开 SPIKE-07.5 用结构化模式重录 corpus → 复用 SPIKE-07 parser+IR 架构（已证 sound）重跑 §F 矩阵 → 重走 §H 三路径，以判定 R1 能否真降级、解锁 MVP-18/19/20。

§H 阈值沿用 **SPIKE-07 spec §H（session-32 Arbiter 钦定 · single source of truth · 本 ADR 不重定义）**：

- 路径 1 greenlight：overall ≥96% AND 各场景 ≥90% AND 统一抽象可行 → R1 降级
- 路径 2 single-cli：一 CLI ≥96% · 另一 <90% → R1 部分降级（仅达标 CLI）
- 路径 3 deferred：两 CLI 均 <90% OR 任一场景 <85% → R1 保留

## 实测结果（溯源 `docs/spikes/raw/SPIKE-07.5/phase3-matrix.json`）

| 指标                                                                      | 实测                                                          |
| ------------------------------------------------------------------------- | ------------------------------------------------------------- |
| 整体 PASS                                                                 | 32/36 = **88.9%** · **panic 0**                               |
| 非退化（剔 codex auth/net 6 退化 corpus · spec §E fail#2 pre-registered） | 29/30 = **96.7%**                                             |
| claude                                                                    | 18/18 = **100%（六场景全过）**                                |
| codex                                                                     | 14/18 = 77.8% · 非退化 11/12 = 91.7%                          |
| long_stream / happy / interrupt / network                                 | 6/6 = **100%**（**vs SPIKE-07 long_stream/mixed = 0%**）      |
| mixed_ansi_json                                                           | 5/6 = 83.3%（唯一非退化 miss = #33 · §F 行首启发式 artifact） |
| auth_fail                                                                 | claude 3/3 = 100% · codex 3/3 退化（OAuth backend 无视 env）  |

4 FAIL 全根因（系统调试 · 无一 parser bug · 详见 [report §D](../spikes/SPIKE-07.5-report.md)）：3 codex/auth_fail = **退化 corpus**（codex 0.130.0 用 ChatGPT OAuth backend · 物理无视 `OPENAI_API_KEY` · spec §E fail#2 录前已登记）· 1 codex/mixed/3 = **§F `mixed_json_parseable` 行首启发式 artifact**（模型把 JSON 内联 ANSI 同行 fence 内 · parser 已正确抽入 MessageDelta · 属 SPIKE-07 Phase D "§F calibration" 类）。

### 关键 nuance：与 SPIKE-07 deferral 不同质

|                 | SPIKE-07 deferred                                                  | SPIKE-07.5                                                      |
| --------------- | ------------------------------------------------------------------ | --------------------------------------------------------------- |
| 根因            | parser **无法**从 TUI 屏幕重绘抽内容（能力墙 · long/mixed **0%**） | parser **干净抽取**（long 100% · claude 全场景 100% · panic 0） |
| 唯一非退化 miss | —（系统性 0%）                                                     | 1 样本 · §F 行首启发式漏抽内联 fence JSON（parser 抽取正确）    |

路径 A 假设（结构化模式发可解析协议 · SPIKE-06 TUI 才是 artifact）**被实测决定性确认** —— 与 ADR-017 推断一致。统一 `CliEvent` IR 抽象实证可行（`ir.rs`/`assertions.rs` byte-identical 复用 SPIKE-07 · sha256 校验 · report §B）。

## 决策（Decision · proposed · Arbiter 拍板后生效）

§H single-source 不允许主 agent 自行重定阈值 / 自行 recalibrate 锁定 §F → 主 agent **proposed** · Arbiter 在拍板栏裁决采纳哪条：

- **首选 · 路径 1 greenlight（R1 降级）**，conditional on Arbiter 接受 2 项已根因 carve-out：(a) codex auth/net 6 退化 corpus（claude 100% 已证 parser 能力）· (b) #33 = §F 行首启发式 artifact（子串扫描 recalibration → mixed 100%）。接受后 R1 **HIGH/HIGH → 降级** · MVP-18/19/20 解锁。
- **保守回退 · 路径 2 single-cli**：claude **无条件 greenlight**（18/18=100% · 平凡 ≥96%）→ R1 **对 claude 降级**；codex conditional（pending §F mixed recalibration + 真 OpenAI key 重录非退化 corpus）。
- **不推荐路径 3 again**：会把 §F 校准 nuance 误判为与 SPIKE-07 能力墙同质（与实测实质矛盾）。

无论 1 / 2：**MVP-18/19/20 解锁范围、CLAUDE.md 决策表 #3 改法、SPIKE-07.5 spec done 翻转 —— 均由 Arbiter 拍板明确后执行 · 主 agent 不自行推进**（§2.1 · CLAUDE.md A 栏）。

## 约束（Constraints）

- SPIKE-07.5 原型代码归档 `docs/spikes/code/SPIKE-07.5/`（进 git · spike-delivery 3 样必交 · report §H manifest）· 不进 `crates/` · v1.0 实施时重写生产 parser（spec §C Don't.5）
- `assertions.rs` byte-identical 锁定（report §B sha256）· 主 agent **未**为凑路径 1 改 §F · 锁定 §F 下 mixed 83.3% 如实呈现 · recalibration 是本 ADR conditional 由 Arbiter 决定
- 本 ADR **proposed** · 需 Arbiter 拍板 → accepted 后方生效（CLAUDE.md A 栏变更流程 · §2.1 主 agent 不自 accept · v2-D.2 单人项目 self-review + Arbiter approval）
- accept 路径 1/2 后 supersede ADR-017（ADR-017 状态 → superseded · 本 ADR 接管 R1 决策）

## 后果（Consequences）

**正面**：

- R1 风险以**结构化模式实测**收敛 · 决定性区分"SPIKE-06 TUI corpus 方法论 artifact"与"parser 能力" —— ADR-017 路径 A 假设证实
- parser + 统一 IR 架构在结构化协议下**高保真验证**（claude 全场景 100% · panic 0 · long_stream 95% 保真）· v1.0 实施可直接复用架构
- 若 accept 路径 1/2：AI-Aware 作为 v1.0 差异化卖点**重新可行** · MVP-18/19/20 解锁 timeline

**负面 / 风险**：

- codex auth/network 错误事件解析准确率本批未公平评估（OAuth backend env 限制 · §G 残留）· 若 Arbiter 要求消解 → 需真 OpenAI key 重录（SPIKE-07.6 或本 spike 补强）
- §F `mixed_json_parseable` 行首启发式对模型内联 fence JSON 漏抽 · recalibration 属 decision-grade · 未做即 mixed 锁定 83.3%
- 路径 1 依赖 Arbiter 接受 2 carve-out · 若 Arbiter 不接受 #33 carve-out → 回退路径 2（claude-only 降级）

**对 R1 监控**：accept 后 R1 状态由本 ADR 接管 · CLI 版本升级 / 协议变更触发 SPIKE-07.5 重评 · 重走 §H。

---

## Arbiter 拍板栏（待 tajiaoyezi 拍板 · 主 agent 不自 check · §2.1）

- [ ] 实测数据无 fabrication（溯源 `phase3-matrix.json` · self-review v2-D.2 · `assertions.rs` sha256 锁定可验 · report §I 0 编造声明）
- [ ] §H 裁决核对（严格字面 = 路径 3 mixed 83.3%<85% · 实质 = 与 SPIKE-07 能力墙不同质 · 主 agent 推荐路径 1 带 2 carve-out / 回退路径 2）
- [ ] 选定路径：**\_\_**（路径 1 greenlight 带 carve-out / 路径 2 claude-only / 路径 3 again / 其他）
- [ ] carve-out 裁决：(a) codex auth/net 退化 corpus 接受？ **\_\_**（是/否）· (b) #33 §F 行首启发式 artifact 接受 recalibration？ **\_\_**（是/否/要求 SPIKE-07.6 补强）
- [ ] 状态翻 proposed → accepted（Arbiter 显式拍板 · v2-D.2 · CLAUDE.md A 栏流程）· supersede ADR-017

**accepted 决议**（待 Arbiter 填）：

1. §H 裁决：**\_\_**
2. R1 状态：**\_\_**（降级 / claude-only 降级 / 保留）
3. MVP-18/19/20：**\_\_**（解锁依赖链 / 部分 / 保持 draft）
4. CLAUDE.md 决策表 #3：**\_\_**（移除⚠️ / 改 claude-only / 保留）
5. SPIKE-07.5 spec status → done · ADR-017 → superseded · 后续（§F recalibration / codex 非退化重录）：**\_\_**
