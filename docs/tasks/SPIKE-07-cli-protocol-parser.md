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
reviewer:
---

# SPIKE-07: CLI 输出协议 parser 验证

> **状态**：`draft`（v1.0-pre · **占位 spec**，非 MVP v0.1/v0.2/v0.3 范围）
> **依赖**：[SPIKE-06](./SPIKE-06-cli-protocol-and-codesign.md)（36+ 样本已录制）
> **阻塞**：MVP-18/19/20（AI-Aware 三件套 · v1.0 vision）
> **战略依据**：[`implementation-plan.md §5.3.6`](../implementation-plan.md) · [`§9 R1`](../implementation-plan.md)

---

## 🎯 目标（Goal）

基于 SPIKE-06 录制的 36+ CLI 输出样本 · 实现原型 parser · 验证"**两个 CLI（Claude / Codex）能否统一抽象**"有可信答案 · 为 R1（CLI 协议未实机验证 · HIGH/HIGH）的降级提供工程依据。

**本 Spike 通过后**：
- 写 **ADR-011-ai-aware-greenlight.md** · R1 降级 proposal → accepted
- 解锁 MVP-18/19/20 三件套的详化

**本 Spike 不通过**：
- R1 保留 · AI-Aware v1.0 推迟 / 降级 / 放弃
- `CLAUDE.md §决策表 #3` 可能需要更新

## 📖 背景（Context）

- `implementation-plan.md §5.3.6 AI-Aware Pane 联动` 明确 v1.0 前必经 parser-oriented spike
- SPIKE-06（W0-D6）**仅录制样本** · 不做 parser 验证（Codex PR #3 R1 F3 教训：样本 ≠ 协议可解析）
- 本 Spike 是 R1 从 HIGH/HIGH 降级的**唯一授权路径**（[ADR-009 §决策](../adr/ADR-009-ai-aware-v1-vision.md)）
- 占位 spec · 详化在 v1.0 kickoff 前 · 基于 SPIKE-06 实际样本数据再补

---

## 🎨 功能范围（Scope）

**Do**（v1.0-pre kickoff 后详化）：
- 基于 SPIKE-06 的 36+ 样本（`docs/spike-artifacts/SPIKE-06/`）做可回放 fixture
- 实现**原型 parser**（Rust / `core` crate）· 解析 CLI 输出流为结构化事件：
  - `message_start` / `message_delta` / `message_end`
  - `role: user | assistant | system`
  - `error: auth | network | rate_limit | ...`
  - `tool_use_start` / `tool_use_end`（两 CLI 可能有 function calling）
- 对 6 场景样本**逐条做解析断言**（happy / 中断残帧 / 认证失败 / 网络错误 / 长流式 / 混合 ANSI-JSON）
- 输出**统一抽象可行性报告**：两 CLI 能否共享同一 IR（intermediate representation）？
- 输出**R1 降级 proposal**：若准确率 ≥ 95% → 降 R1 到 MEDIUM；若 ≥ 99% → 降到 LOW；否则保留

**Don't**（明确不做）：
- 完整 AI-Aware 实现（MVP-18/19/20 范围）
- MVP 集成（MVP-04 多 Tab 终端集成 CLI · 不依赖 parser）
- 第三方 CLI（Gemini / 其他 · v2+ 再考虑）

## ✅ Acceptance（v1.0-pre kickoff 后详化）

骨架（基于 SPIKE-06 结果填详细数字）：
- [ ] Parser 对 36+ 样本做解析断言 · 覆盖 6 场景 × 2 CLI
- [ ] Happy path 准确率 ≥ 99%
- [ ] 失败路径（auth / network / 中断）准确率 ≥ 95%
- [ ] 混合 ANSI+JSON 场景准确率 ≥ 95%（最难的场景）
- [ ] 两 CLI 统一抽象可行 / 不可行 · 有工程证据（非推测）
- [ ] R1 降级 proposal 有明确等级

## ❌ Fail Signals

- 任一场景准确率 < 90% → R1 不能降级 · AI-Aware 全套推迟
- 两 CLI 结构差异过大（无法统一抽象）→ 只能支持一个 CLI · README 措辞修改
- parser 遇到真实样本 crash → Rust 实现 bug · 迭代修复

## 🔀 Fallback 方案

**通过（全指标达标）** → 写 ADR-011 · R1 降级 accepted · MVP-18/19/20 详化解锁
**部分通过（一个 CLI 能 parse 另一个不能）** → 只支持前者 · README / landing 需说明
**双失败** → R1 不降级 · AI-Aware 推迟到 v2 · CLAUDE.md #3 禁区延续

## 📦 产出（Deliverables）

- [ ] `spike-tmp/spike-07-parser/` · 原型 parser 代码 + fixture loader
- [ ] **`docs/spikes/SPIKE-07-report.md`** · 准确率数据 + 统一抽象可行性分析
- [ ] **ADR-011-ai-aware-greenlight.md**（若通过 · 否则 ADR-011-ai-aware-deferred.md）
- [ ] 更新 `CLAUDE.md §决策表 #3 / ⚠️ 警告` 状态（通过 → 移除警告）

## 🛠 依赖资源

- SPIKE-06 产出：`docs/spike-artifacts/SPIKE-06/` 的 36+ 样本
- SPIKE-06 产出：`docs/spikes/SPIKE-06-report.md` 结构观察报告
- Rust 开发机（parser 实现）
- 测试机（回放 fixture）

## ⚠️ 已知风险

- **R1 不能降级**：parser 准确率不达标 → AI-Aware 整个 v1.0 vision 要重新规划
- **SPIKE-06 样本不够真实**：实机录制时覆盖的场景可能 miss 了某些边界（如 CLI 更新后协议变 · 长 session 状态管理）· 需 v1.0 kickoff 前复查
- **parser 迁移性**：原型 parser 写死对当前 CLI 版本 · CLI 大版本升级可能 break · 长期风险

---

## 📝 Notes / 讨论

- 本 Spike 是 AI-Aware vision 的**技术门槛** · 过不了 = v1.0 核心卖点落空
- 建议 v0.3 发布后立刻启动（而非等 v1.0 kickoff）· 3d 工期早做早知道
- 占位 spec · 详化在 SPIKE-06 样本录制完成后再补具体准确率数字 + parser 架构

## 🔗 相关

- 上游：[SPIKE-06 CLI 实机](./SPIKE-06-cli-protocol-and-codesign.md)
- 下游：[MVP-18 AI-Aware Pane 联动](./MVP-18-ai-aware-pane-linking.md) · [MVP-19 session↔commit](./MVP-19-session-commit-binding.md) · [MVP-20 AI 回滚](./MVP-20-ai-one-click-rollback.md)
- 相关 ADR：[ADR-009 AI-Aware v1.0 vision](../adr/ADR-009-ai-aware-v1-vision.md)（本 Spike 通过后才能开 ADR-011）
- `CLAUDE.md` ⚠️ 条款：Claude CLI / Codex CLI 输出协议未经实机验证（本 Spike 是唯一降级授权）

---

**自审**（占位 spec · 简化版四问）：

1. **递归完备性**：6 场景 × 2 CLI × 解析断言齐全 ✅
2. **反向场景**：3 种 fallback 路径（全通 / 部分 / 双失败）已列 ✅
3. **边界适用性**：CLI 版本漂移风险已记 ✅
4. **YAGNI**：占位阶段不定具体 parser 架构（v0.3 后详化）✅
