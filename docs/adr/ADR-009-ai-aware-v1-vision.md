# ADR-009: AI-Aware Pane 联动 = v1.0 vision（对外完全不宣传）

**状态**：accepted
**日期**：2026-04-18（Phase 1 锁定 · Phase 3 ADR 建立）
**决策者**：项目发起人 · 多 agent 评审
**对应 `CLAUDE.md` 决策表**：#3

---

## 背景与问题

Vibestation 的长期 vision 含 **AI-Aware 版本控制**：
- 把一次 Claude / Codex CLI 对话识别为一个 "AI session"
- session ↔ commit 自动绑定（MVP-19）
- AI 构建失败反哺（build fail → parsed_issues 自动给 AI · MVP-18）
- 一键回滚整个 session 的 commit（MVP-20）

这是 Vibestation 在红海赛道（Tower / Fork / GitKraken）的**差异化卖点**。

但 AI-Aware 依赖 **R1**（Claude/Codex CLI 输出协议解析 · HIGH/HIGH）· **实机未验证前**：
- parser 稳定性未知 → `parsed_issues` 可能全是垃圾 → AI 收到污染上下文 → "AI 噱头"差评
- 对外"AI-Aware Mission Control"叙事若落不了地 → 声誉扣分

## 决策驱动因素

- **D1 · 工期风险**：AI-Aware 完整实现依赖 SPIKE-07（parser 稳定性验证 · 未 schedule）· 若塞 v0.1 会直接延期
- **D2 · 叙事风险**：宣传了做不到 = 骗炮 · 宣传了做到一半 = 鸡肋
- **D3 · 技术风险**：R1 未消除 · 任何基于 parser 的功能都是 house of cards
- **D4 · 竞争叙事**：v0.1 靠"多 Tab 终端 + Calm Studio 视觉 + 基础 Git UI"已有差异化 · AI-Aware 留作 "v1.0 升级故事" 更有效

## 考虑的选项

- **A · v0.1 宣传 AI-Aware · 做一半发布**：叙事冲击大 · 但技术栽了就翻车 · **拒绝**
- **B · v0.1 宣传 AI-Aware · 不发布**：画饼 · 违反诚信 · **拒绝**
- **C · AI-Aware 降级 v1.0 vision · 对外完全不提**：MVP/v0.2/v0.3 只讲"多 Tab 终端 + Git UI" · 等 v1.0 真实落地再讲 · **选**
- **D · 砍 AI-Aware 完全不做**：放弃差异化 · v1.0 前无故事 · 竞争弱 · **拒绝**

## 决策

**选择**：选项 C · **AI-Aware = v1.0 vision · README / landing / Twitter / Discord 完全不宣传**

**硬约束**（`CLAUDE.md §禁区`）：
- ❌ 禁止对外文案提及 `AI-Aware Pane` / `Mission Control` / `AI session aware`
- ❌ README 首屏 · landing page · Twitter bio · Discord 描述 · v0.1 CHANGELOG · 均不得出现
- ❌ v0.1 / v0.2 / v0.3 的 task spec（MVP-01..17）均不含 AI-Aware 实现
- ✅ 允许：内部技术文档（ADR / implementation-plan / tasks/MVP-18..20）明确标注"v1.0 vision"
- ✅ 允许：v1.0 开发启动后逐步揭晓（先博客 post · 再产品 announcement）

**技术前提**（v1.0 正式实现前必做）：
- **SPIKE-07** parser-oriented spike 必通过（基于 SPIKE-06 录制的 36+ 样本 · `parsed_issues` 解析准确率 ≥ 95%）
- SPIKE-07 通过 → 写 `ADR-011-ai-aware-greenlight.md` · 才能启动 MVP-18/19/20 详化

**理由**：
1. **技术诚信**：R1 未消除前任何基于 parser 的功能都是"可能翻车" · 不宣传比"宣传后失败"损失小 10 倍
2. **叙事节奏**：v1.0 作为升级故事 · 新老用户都有话题 · 比 "v0.1 就吹但做不到" 强
3. **v0.1 已够差异化**：多 Tab 终端 + JetBrains 级 Git UI + Calm Studio 视觉 + Rust + 开源 · 五项叠加已有卖点
4. **AI 噱头泛滥**：2024-2026 市场 AI 功能过载 · 刻意慢一拍用"**已打通 parser · 稳定可用**"切入 · 更有信任感

## 后果

### 正面

- **叙事安全**：v0.1 无 AI 承诺 → AI 部分失败不影响 v0.1 发布
- **技术安全**：R1 未消除前不出 AI 功能 → 无垃圾 context / AI 噱头负面反馈
- **节奏健康**：v1.0 作为真实升级故事 · 2026 Q4 前做到能落地的那一步
- **合规对齐**：所有对外文案（landing / README / 社交）统一 · 贡献者 PR 加 AI-Aware 卖点会被 reviewer 拒（feature_request 模板有勾选强制）

### 负面

- **差异化延后**：v0.1 相对 Tower / Fork 在功能面上"少一大卖点" · 需要靠视觉 / 性能 / 开源 / Rust 补
- **贡献者可能不理解**：新贡献者看到 `docs/tasks/MVP-18/19/20` 想做 · 需要 reviewer 反复强调"v1.0 vision"

### 风险

- **内部文档泄露到对外**：开发者无意中把内部 ADR 链接贴到 blog / 社交 → AI-Aware 被提前讨论 · **对策**：PULL_REQUEST_TEMPLATE 有合规勾选 + landing 页面严禁提及
- **竞品先落地**：Tower / Fork 若 2026 加 AI-Aware 先宣传 → 我们失先机 · **对策**：SPIKE-07 尽早启动（v0.3 GA 前）· 2026 Q4 v1.0 先发

## 与 `implementation-plan.md` 的映射

- 对应章节：§1.1（产品定位）· §5.3.6（AI-Aware Pane 联动技术设计）· §10.1（砍到 v1.0）
- 对应风险：**R1**（Claude/Codex CLI 协议 · HIGH/HIGH · 本 ADR 规避策略）

## 相关

- `CLAUDE.md` 决策表：#3（+ 禁区条款明确禁止对外提及）
- 详细 spec：[MVP-18 AI-Aware Pane 联动](../tasks/MVP-18-ai-aware-pane-linking.md)· [MVP-19 session↔commit 绑定](../tasks/MVP-19-session-commit-binding.md)· [MVP-20 AI 一键回滚](../tasks/MVP-20-ai-one-click-rollback.md)
- Spike：[SPIKE-06 CLI 实机](../tasks/SPIKE-06-cli-protocol-and-codesign.md)（录 36+ 样本）· SPIKE-07（未建 · parser 验证 · v1.0 前必做）
- 未来 ADR：ADR-011 AI-Aware greenlight（待 SPIKE-07 通过后建）

---

**修订历史**：
- 2026-04-18 · 初版 · Claude Code（Phase 3 · 把 Phase 1 锁定决策正式化为 ADR）
