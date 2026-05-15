# ADR-017: AI-Aware v1.0 vision deferred — SPIKE-07 技术前提未被真实 corpus 验证

**状态**：proposed
**日期**：2026-05-15
**决策者**：Claude Code（作者 agent · 主 agent 跑 SPIKE-07 实测）· tajiaoyezi（Arbiter · 待拍板 proposed → accepted）
**对应 `CLAUDE.md` 决策表**：A 栏 #3（AI-Aware Pane 联动 = v1.0 vision）· 本 ADR 提议**保留** #3 现状（R1 不降级 · ⚠️ 警告保留）· 不移除
**对应 Spike**：[SPIKE-07](../tasks/SPIKE-07-cli-protocol-parser.md) · [SPIKE-07-report](../spikes/SPIKE-07-report.md)
**前置 ADR**：[ADR-009](./ADR-009-ai-aware-v1-vision.md)（AI-Aware = v1.0 vision · R1 HIGH/HIGH 起源）

---

## 背景与问题（Context and Problem Statement）

ADR-009 锁定 AI-Aware Pane 联动为 v1.0 vision · 风险项 R1（parser 可行性）= HIGH/HIGH。SPIKE-07（R1 降级前置）基于 SPIKE-06 的 36 条脱敏 corpus（2 CLI × 6 场景 × 3 take）实跑原型 parser + §F 测试矩阵，判定 R1 能否降级以解锁 MVP-18/19/20 详化。

SPIKE-07 spec §H（session 32 Arbiter 钦定为 R1 降级 single source of truth）定义 3 路径：

- 路径 1 greenlight：整体加权 ≥ 96% 且 各场景 ≥ 90% 且 两 CLI 可统一 → R1 降级
- 路径 2 single-cli：一 CLI ≥ 96% · 另一 < 90% → R1 部分降级（仅支持达标 CLI）
- 路径 3 deferred：两 CLI 均 < 90% 或 任一场景 < 85% 或 parser 多数 Unrecognized → R1 保留

## 实测结果（溯源 `docs/spikes/raw/SPIKE-07/phase-c-matrix.json`）

| 指标                                                        | 实测                        |
| ----------------------------------------------------------- | --------------------------- |
| 整体 PASS                                                   | 24/36 = **66.7%** · 0 panic |
| happy_path / auth_fail / network_error / interrupt_residual | 6/6 = **100%**（4/6 场景）  |
| long_stream                                                 | 0/6 = **0%**                |
| mixed_ansi_json                                             | 0/6 = **0%**                |
| claude / codex                                              | 各 12/18 = **67%**（对称）  |

§H 逐路径核对 → **路径 1 ❌**（66.7% < 96%）· **路径 2 ❌**（两 CLI 对称 67% · 无一 ≥ 96%）· **路径 3 ✅**（两 CLI < 90% 且 long_stream/mixed_ansi_json 0% < 85%）。对齐 §G fail signal #1（任一场景 < 90%）+ #4（样本不够真实）+ §M 路径 C/D。

### 关键 nuance：deferred 非因 parser 实现差

12 条 FAIL 三类根因（详见 [SPIKE-07-report §Phase D.1](../spikes/SPIKE-07-report.md)）· **无一条是 parser bug**：

1. **corpus 质量**（claude long_stream）：SPIKE-06 录的是 raw PTY 屏幕重绘 blob（142–300KB · 光标重画），非行式 assistant 流 · 无 parser 能从屏幕重绘提取内容
2. **§F 断言对厚协议校准失配**（codex long_stream）：codex.rs 实际解析成功（events=28 · 0% unrec · 92-93% content），仅因 §F 95% 阈值分母含厚协议脚手架（SessionMeta/Hook/Usage 约 7%）差 2-3pp
3. **协议现实**（两 CLI mixed_ansi_json）：Claude/Codex CLI stdout 均为人类终端格式文本 · **不发机器可解析 JSON events** · "mixed_ansi_json"预设的 JSON 协议在实测 CLI 中不存在

统一抽象 **可行**（§E.4 正面发现）：两 adapter 共享同一 `CliEvent` IR · 核心 5 变体 100% 重合 · §G fail signal #2 不触发 · 架构 sound。

## 决策（Decision）

1. **R1 保留 HIGH/HIGH** —— 不降级。AI-Aware Pane 联动推迟到 **v2+**。
2. **MVP-18 / MVP-19 / MVP-20 保持 `draft`** —— 不进入 in-progress（依赖 R1 降级未满足）。
3. **`CLAUDE.md` 决策表 #3 ⚠️ 警告保留** —— 不移除（对外文案继续不得提及 AI-Aware / Mission Control · 维持 ADR-009 禁区）。
4. **v1.0 卖点回归基础差异化**：多 Tab 终端 + JetBrains 级 Git 工作台（不依赖 AI-Aware）。
5. **后续路径（供 Arbiter 选 · 不在本 ADR 强制）**：
   - 选项 A（主 agent 推荐先做）：调研 CLI 是否有 headless / 结构化输出模式（`claude --output-format json`？codex 非 TTY 模式？）· 若有则重录 SPIKE-06 corpus → 重跑 SPIKE-07（parser+IR 架构已 sound · 可能翻盘）· 成本 0.5-1d
   - 选项 B：接受 AI-Aware 需启发式终端态推断（fragile）· 推 v2 评估
   - 选项 C：v1.0 直接不含 AI-Aware（本 ADR deferred 默认路径）

## 约束（Constraints）

- SPIKE-07 原型代码归档 `docs/spikes/code/SPIKE-07/`（进 git · 见 spike-delivery-checklist 3 样必交）· 不进 `crates/` · v1.0/v2 实施时重写生产 parser
- 本 ADR 为 **proposed** · 需独立评审 + Arbiter 拍板 → accepted 后方生效（CLAUDE.md A 栏变更流程 · §2.1 主 agent 不自 accept）
- 若未来选项 A 调研发现 CLI 有结构化输出模式 · 应新开 SPIKE-07.5（或 SPIKE-07 续）重测 · 届时本 ADR 可被新 ADR supersede

## 后果（Consequences）

**正面**：

- R1 风险以实测数据收敛（非主观判断）· 36 样本矩阵 + 三类根因分析为未来重评提供 baseline
- parser + 统一 IR 架构验证 sound（happy/auth/network/interrupt 100% · 0 panic · 统一抽象可行）· 重录 corpus 后可直接复用
- v1.0 范围明确（基础卖点）· 不被未验证的 AI-Aware 前提拖累 timeline

**负面 / 风险**：

- AI-Aware 作为差异化卖点推迟 · v1.0 竞争力依赖基础体验打磨
- 若选项 A 调研发现 CLI 无任何结构化输出模式 · AI-Aware 可能长期 deferred（v2 也难）
- corpus 重录（选项 A）需回 SPIKE-06 流程 · 额外成本

**对 R1 监控**：CLI 版本升级后若协议更结构化（如官方加 `--json` 输出模式）· 触发 SPIKE-07 重评 · 重走 §H 三路径。

---

## Arbiter 拍板栏（accepted 时由 Arbiter / 独立评审填）

- [ ] 独立评审确认实测数据无 fabrication（溯源 `phase-c-matrix.json`）
- [ ] Arbiter 确认 §H 路径 3 判定正确（路径 1/2 已逐条排除）
- [ ] Arbiter 选定后续路径（A 调研 / B v2 / C v1.0 不含）
- [ ] 状态翻 proposed → accepted（独立评审 · 非作者自 flip · CLAUDE.md A 栏流程）
