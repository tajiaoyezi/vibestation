# ADR-013: Spike 冷备归档规则降级 · v1 强制 → v2 推荐

**状态**：accepted
**日期**：2026-04-21
**决策者**：Claude Code（作者 agent）· tajiaoyezi（Arbiter · 即用户）
**对应 `CLAUDE.md` 决策表**：不在 A/B/C 三档 · 属项目级规则（`.claude/rules/spike-delivery-checklist.md`）
**对应 Spike**：—（规则降级决策 · 无 Spike 前置）

---

## 背景与问题（Context and Problem Statement）

项目级规则 `.claude/rules/spike-delivery-checklist.md` v1 版本（session 7-12 期间稳定）规定每个 Spike 必须交付 "4 样齐全"：

1. 决策文档（`docs/spikes/SPIKE-XX-report.md`）
2. 实测源码（`docs/spikes/code/SPIKE-XX/`）
3. Raw 数据（`docs/spikes/raw/SPIKE-XX/`）
4. **冷备**（`spike-tmp/archive/SPIKE-XX/` · gitignored）

2026-04-21（session 13 audit M-1）实测 9 个已完成 Spike 的冷备状态：

| Spike      | 决策文档 | 实测源码 | Raw 数据 |        冷备        |
| ---------- | :------: | :------: | :------: | :----------------: |
| SPIKE-01   |    ✅    |    ✅    |    ✅    |         ❌         |
| SPIKE-02   |    ✅    |    ✅    |    ✅    |         ❌         |
| SPIKE-03   |    ✅    |    ✅    |    ✅    |         ❌         |
| SPIKE-04   |    ✅    |    ✅    |    ✅    |         ❌         |
| SPIKE-04.5 |    ✅    |    ✅    |    ✅    |         ❌         |
| SPIKE-05   |    ✅    |    ✅    |    ✅    |         ✅         |
| SPIKE-05.5 |    ✅    |    ✅    |    ✅    |         ❌         |
| SPIKE-06   |    ✅    |    ✅    |    ✅    | ✅（SPIKE-06-pr2） |
| SPIKE-08   |    ✅    |    ✅    |    ✅    |         ❌         |

**冷备合规率 2/9 = 22%**。前 3 样均 100% 合规。

**问题**：

- 规则和现实严重不匹配 · 规则贬值（自己定的规则 78% 不做）
- 7 个欠账补齐的成本：每个 Spike 需 `cargo build --release` + `tar -czf` + 本地保留 · 估 30-60 min/Spike · 合计 4-7 小时
- 补齐收益：几乎为零（code + Cargo.lock 进 git · `cargo build` 可 byte-level 复现 benchmark · 冷备只是省 build 时间）

**不决策的后果**：

- 规则持续被违反 · 其他规则也可能跟着松动（破窗效应）
- 新 Spike 如果也不做冷备 · 规则继续贬值
- 看起来欠账永远补不完（但实际无 damage · 只是规则和现实失配）

## 决策驱动因素（Decision Drivers）

- **D1 · 规则可维持**：规则必须人肉可达成 · 或有机械保证 · 否则是负债（参考 ADR-012 v2-D.1 同理）
- **D2 · 信息冗余识别**：冷备是否真的补充了 code + raw 之外的信息？答案 = 大部分情况否
- **D3 · 边际成本 vs 边际收益**：补齐 7 个欠账 4-7 小时 · vs 新 Spike 按新规则执行 0 额外成本
- **D4 · 特殊场景保护**：某些 Spike 确实需要冷备（大测试数据 · 外部二进制）· 不能一刀切彻底删

## 考虑的选项（Considered Options）

### 选项 A · 规则降级 · 冷备从"必须"改"推荐"（v1 → v2）

- 3 样必须：决策文档 + 源码 + Raw 数据
- 1 样推荐：冷备（按 3 个场景判断）
  - 场景 1：Spike 有 > 100MB 随机测试数据
  - 场景 2：Spike 涉及外部二进制工具
  - 场景 3：Spike 非 Cargo 构建
- 历史 7 个欠账 **不追溯补齐**（接受为技术债）
- 未来 Spike 按 v2 规则做

### 选项 B · 补齐 7 个欠账 · 保留 v1 规则

- 花 4-7 小时 `cargo build --release` + tar 全部 7 个 Spike
- 保留 v1 "4 样必须" 规则
- 收益：规则合规率 100%

### 选项 C · 完全删除冷备要求 · 改 3 样必须

- 冷备从规则中彻底删除
- 未来即使大数据 Spike 也不做冷备
- 风险：极端情况下无 build artifact 快照 · 复现困难

## 决策（Decision Outcome）

**选择**：选项 A · 规则降级（v1 → v2）

**理由**：

- 选项 A 规则和现实对齐 · 不再有"已写但做不到"条款
- 选项 A 保留特殊场景保护 · 大数据 / 外部二进制 / 非 Cargo 构建 仍必做冷备
- 选项 B 投入 4-7 小时补齐 · 但补齐之后信息增量为零 · ROI 极低
- 选项 C 删过头 · 失去特殊场景保护

和 ADR-012（v2-D → v2-D.1）同一思路：规则必须人肉可维持 · 否则贬值。

## 后果（Consequences）

### 正面

- 规则合规率从 22% 恢复到 100%（新标准 3 样必须 · 已全部做到）
- 未来 Spike 执行简化 · 边际成本降低
- 特殊场景保护未丢（场景判断清单在规则文件）

### 负面

- 历史 7 个 Spike 无冷备 · 若 docs/spikes/code/ 被意外破坏且无 git backup · 无 byte-level 快照恢复
  - 缓解：`docs/spikes/code/` 进 git · GitHub 远程有 backup · 真正丢失需要 local + remote 同时破坏 · 可忽略风险
- 若未来某 Spike 判断失误（该做冷备但没做）· 复现困难
  - 缓解：场景判断清单明确 · 加 PR review checkpoint

### 风险

- **R1**：未来新 Spike 的作者对 3 场景判断失误 · 该做冷备没做
  - Fallback：PR reviewer 检查 Spike Test Plan · 若命中场景 1/2/3 但冷备 checkbox 未勾 · 退回补齐
- **R2**：v1 期的 7 个 Spike 未来真遇到"code/ 进 git 但无法复现"的罕见情况
  - Fallback：逐个重建（用 Cargo.lock + src/ 重新跑 cargo build · 估 5-15 min/Spike）

## 与 `implementation-plan.md` 的映射

- 对应章节：—（项目级规则 · 不在 implementation-plan 范围）
- 对应风险：—

## 相关（Links）

- `.claude/rules/spike-delivery-checklist.md` · v2 修订后的规则文件
- 全局上位规则：`~/.claude/rules/13-cross-agent-delivery.md` · 跨 agent 交付物持久化（v2 仍遵守上位的"持久化 + 可复现"原则）
- 同期 ADR：[ADR-012](./ADR-012-v2d1-arbiter-approval-simplification.md) · 同样用"规则可维持"思路的 v2-D → v2-D.1
- 审查报告：[`docs/internal/session-12-audit-report-2026-04-20.md`](../internal/session-12-audit-report-2026-04-20.md) §4.1 M-1（后续 session 13 扩展审视）
- PR：本 PR（chore/session-13-audit-followup · 同 ADR-012 同 PR）

---

**修订历史**：

- 2026-04-21 · 初版 · accepted · Claude Code + tajiaoyezi

**自审四问**：

1. **递归完备性**：本 ADR 和规则文件互指 · 不漏 ✅
2. **反向场景**：R1/R2 都有 fallback · 最坏情况"重跑 cargo build" ≤ 15 min ✅
3. **边界适用性**：适用所有 Spike · 3 场景判断清单覆盖特殊情况 ✅
4. **YAGNI**：22% 合规率是实证失败 · 非投机降级 ✅
