# ADR-012: v2-D → v2-D.1 · 单人项目 Arbiter approval 规则简化

**状态**：accepted
**日期**：2026-04-21
**决策者**：Claude Code（作者 agent）· tajiaoyezi（Arbiter · 即用户）
**对应 `CLAUDE.md` 决策表**：不在 A/B/C 三档 · 属"禁区"段的治理规则
**对应 Spike**：—（治理决策 · 无 Spike 前置）

---

## 背景与问题（Context and Problem Statement）

`CLAUDE.md` §"改锁定表 A 栏前必须" §(2) 在 2026-04-19（session 10 末 · ADR-006 翻转 PR #50）建立了 v2-D 单人项目 Arbiter approval 双轨制：

- **(a)** PR body 必含 `Implemented by` + `Reviewed by` + `Arbiter approve: tajiaoyezi · YYYY-MM-DD · "<dialogue 原文摘要>"` trailer
- **(b)** merge 后 **24h 内**用 `gh pr comment <N>` 补完整 dialogue trail（含时间戳 + 原文）

2026-04-20（session 12）批量实证 · 双轨制的 (b) 支失败：

- **12/12 PR**（#64–#75）· `gh pr view --json comments` 全部 0 条
- 其中 7 个（#64/#65/#67/#68/#69/#72/#75）连 (a) 的 PR body trailer 都缺
- 详见 [`docs/internal/session-12-audit-report-2026-04-20.md` §3.1](../session-12-audit-report-2026-04-20.md)

**根因**：(b) "24h 内补 PR comment" 纯靠人肉自觉 · 无 hook 自动化 · 无 CI 硬阻塞 · 人肉不能规模化（22 PR / 12h 的 session 12 节奏）。

**不决策的后果**：

- 规则自写自违反 · 规则贬值 · 未来更多规则失守
- 未来触发 v2-strict（加真合作者）时 · 历史 audit trail 断档 · 追溯困难

## 决策驱动因素（Decision Drivers）

- **D1 · 规则可持续**：规则必须人肉可维持 · 或有自动化兜底 · 否则是负债
- **D2 · audit trail 完整**：PR body 是 GitHub 永久不可篡改 metadata · comment 只是冗余
- **D3 · 单人项目简化**：Arbiter = 仓库 admin = 用户本人 · 三方合一 · 不需要双轨冗余
- **D4 · v2-strict 可升级**：未来触发严格模式时 · 能无痛加回 comment / 切到 GitHub UI Approve

## 考虑的选项（Considered Options）

### 选项 A · 规则简化（v2-D.1）· 删掉 (b)

- PR body trailer 为**唯一**必要条件
- `gh pr comment` 降级为**推荐但非硬要求**
- 历史 7 个 body 缺 trailer PR 一次性 `gh pr comment` 过渡补档
- 之后永不欠账

### 选项 B · 规则 + Hook 自动化

- 保留 v2-D 双轨制
- 新增 Stop hook · 监控 `gh pr merge` · 自动 `gh pr comment` 补完整 trailer
- 彻底无人工介入

### 选项 C · Dialogue-as-audit

- 单人项目完全不需要 GitHub 侧 trailer / comment
- audit trail 存在 session log / memory

## 决策（Decision Outcome）

**选择**：选项 A · v2-D.1 规则简化（删掉 (b) 硬要求）

**理由**：

- 选项 A 实施成本最低（5 min · 改 CLAUDE.md + 写 ADR + 过渡补档）
- 选项 B 虽然零欠账 · 但 hook 失效时无感知 · 过度工程（30-45 min · 且需要 CI 检测 hook 自身）· 可作为 v0.3 可选增强
- 选项 C 风险最高 · GitHub 侧完全无 audit · session log 被 compaction 清 · 未来 v2-strict 断档
- A 对 v2-strict 升级路径零阻碍（加回 §2(b) 即可 · 与 v2-D 等价）

## 后果（Consequences）

### 正面

- 规则与现实对齐 · 不再有"已写但做不到"的鬼条款
- 未来 PR 流程简化 · 减少主 agent 的 session-end overhead
- GitHub PR body 里的 `Arbiter approval: ...` trailer 已是永久 audit trail · 不丢

### 负面

- 单 PR 的 dialogue 原文不再必存 GitHub comment · 若需要完整对话追溯 · 得查 session-history（但这类需求极少）
- 若未来人类合作者加入 · 需要马上触发 v2-strict · 无自动提示（靠 Arbiter 人工判定）

### 风险

- **R1**：主 agent 未来仍然忘写 body trailer · 风险同 v2-D (a) 单轨但概率更低（只有 1 条要记 · 不是 2 条）
  - Fallback：Stop hook 检测 `gh pr create` 命令的 body 是否含 Arbiter trailer · 缺则阻止（v0.3 增强）
- **R2**：v2-strict 升级时忘了加回 comment 要求 · 导致新合作者场景 audit 不足
  - Fallback：触发 v2-strict 的 ADR 必须明确"加回 §2(b) 或等价机制"· 本 ADR 在 §相关里挂勾子

## 与 `implementation-plan.md` 的映射

- 对应章节：—（治理规则 · 不在 implementation-plan 范围）
- 对应风险：—

## 相关（Links）

- `CLAUDE.md` §"改锁定表 A 栏前必须" §(2) · §(5) · §(6)
- 前置 ADR：v2-D 升级 PR #50（ADR-006 + 决策表 #19 同 PR · 未单独开 ADR）
- 审查报告：[`docs/internal/session-12-audit-report-2026-04-20.md`](../session-12-audit-report-2026-04-20.md) §3.1 H1
- 未来升级入口：CLAUDE.md §"改锁定表 A 栏前必须" §(3) "未来升级触发 v2-strict"
- PR：本 PR（chore/session-13-audit-followup）

---

**修订历史**：

- 2026-04-21 · 初版 · accepted · Claude Code + tajiaoyezi

**自审四问**：

1. **递归完备性**：本 ADR 自身遵循 v2-D.1 规则（PR body 含 Arbiter trailer 即合规 · 不强制补 comment）✅
2. **反向场景**：规则不遵守 → 主 agent 忘写 body trailer → 未来加 Stop hook 兜底（R1 fallback）✅
3. **边界适用性**：v2-D.1 只适用单人项目 · v2-strict 触发后自动失效（CLAUDE.md §(3) 明文）✅
4. **YAGNI**：删条款比加条款的 YAGNI 风险低 · 且 v2-D (b) 已实证不可维持 · 非投机删除 ✅
