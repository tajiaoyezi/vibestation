# ADR-016: admin override 模式 trailer 豁免 · v2-D.1 → v2-D.2

**状态**：accepted
**日期**：2026-05-03
**决策者**：Claude Code（作者 agent）· tajiaoyezi（Arbiter · 即用户）
**对应 `CLAUDE.md` 决策表**：不在 A/B/C 三档 · 属"禁区"段的治理规则（§"改锁定表 A 栏前必须" §(2)）
**对应 Spike**：—（治理决策 · 无 Spike 前置）
**前置 ADR**：[ADR-006](./ADR-006-desktop-framework.md)（v2-D 起源 · session 10 末）· [ADR-012](./ADR-012-v2d1-arbiter-approval-simplification.md)（v2-D → v2-D.1 简化 · 同思路 + 同决策模式）

---

## 背景与问题（Context and Problem Statement）

`CLAUDE.md` §"改锁定表 A 栏前必须" §(2) 在 ADR-012（2026-04-21 · v2-D.1）确立了单人项目 Arbiter approval 单轨制：**PR body 必含** `Implemented by` + `Reviewed by` + `Arbiter approval: tajiaoyezi · YYYY-MM-DD HH:MM · "<dialogue 摘要>"` trailer · 缺任一即视为未经 Arbiter 审批 · 不得 merge。

但本规则隐含假设 **所有 main 上的改动都走 PR**。session 21（2026-04-26 ~ 04-29）期间出现两类无法走 PR 的 admin direct push · 触发治理盲区：

### 实证数据（session 21 admin override 模式 · 7 个 direct push）

**触发原因**：GitHub Actions billing 暂停 · PR-level CI 完全无法运行 · `gh pr merge --auto` 等待 CI pending 永不触发 · 走 PR 流程实际等同卡死。Arbiter 切 admin override 模式 `git push origin main`（受 `.githooks/pre-push` 阻拦时用 `SKIP_BRANCH_PROTECT=1` env override）。

| commit | 类型 | trailer 合规 | 备注 |
|---|---|---|---|
| `2c1044a` | 人工 admin push（v0.1.1 GA blocker fix · 23 文件 / +1054 / -123） | ⚠️ 无 trailer · commit body 写 "GitHub Actions billing 暂停，CI 无法跑" | 主 agent 写代码 → Arbiter 直推 main |
| `7697b8b` | dependabot auto（actions/upload-artifact 4→7） | ⚠️ 无 trailer · dependabot 标准 commit format | bot 自动 |
| `a9336ff` | dependabot auto（libc 0.2.185→0.2.186） | ⚠️ 无 trailer | bot 自动 |
| `347140a` | dependabot auto（plist 1.8.0→1.9.0） | ⚠️ 无 trailer | bot 自动 |
| `492c283` | dependabot auto（minor-updates group · 4 个） | ⚠️ 无 trailer | bot 自动 |
| `93a1317` | dependabot auto（sha2 0.10.9→0.11.0） | ⚠️ 无 trailer | bot 自动 |
| `739da3d` | dependabot auto（vite 6.4.2→8.0.10 · dev） | ⚠️ 无 trailer | bot 自动 |

**Audit 问题**（session 22-23 deferred · 本 ADR 闭合）：
- v2-D.1 规则**未涵盖** admin direct push 场景（无 PR body 可写）
- 是否补 retroactive trailer · 或显式声明豁免 · 未决

**不决策的后果**：
- 规则空白 · 未来 admin override 时主 agent 不知是否合规
- audit 项悬空 · session-end 累积技术债
- 未来 v2-strict 升级时 · 历史 admin push 治理状态不清

---

## 决策驱动因素（Decision Drivers）

1. **实务对齐优先**：v2-D.1 设计哲学是"规则与现实对齐 · 不留鬼条款"（ADR-012 选项 A）· 治理升级也应遵循
2. **admin override 是异常 · 不是默认**：v0.1 GA 后 GitHub Actions billing 恢复 · admin override 频率应回归 0
3. **bot auto direct push 不可避免**：dependabot / renovate 走 GitHub native auto-merge · 不经过 PR review · 任何治理设计必须接受这事实
4. **audit trail 不能完全丢**：即使豁免 trailer · commit body 必须留 audit marker（人工写原因 / bot 自带 source ref）
5. **零阻塞 v0.2 sprint**：本 ADR 不应引入新的 manual gate / hook / CI 强制 · 维持当前 momentum

---

## 考虑的选项（Considered Options）

### 选项 A · 显式豁免 + commit body audit marker（v2-D.2 · 推荐）

**改动**：
- v2-D.1 §(2) 加新子条款："admin direct push（含 emergency human + bot auto）豁免 PR body trailer 要求 · 但 commit body 必须含 audit marker"
- audit marker 规范：
  - **人工 admin push**：commit body 显式 `admin override · 原因：<X>`（X 例：`GitHub Actions billing 暂停 · CI pending 卡死` / `紧急修复 v0.1.1 GA blocker · 主 agent 已本地全过 gates`）
  - **bot auto push**：默认 dependabot / renovate commit format 已含 source ref（"Bumps X from A to B"）· 视为足够 audit · 无需额外 marker

**优点**：
- 实施成本最低（< 30 min · 仅文档更新 · 无代码 / hook / CI 改动）
- 与 ADR-012 选项 A 同思路（简化规则 · 接受实务）
- 不阻塞 v0.2 sprint
- audit trail 仍存在（commit body 而非 PR body）· 未来 v2-strict 升级时可追溯

**缺点**：
- v2-D.2 比 v2-D.1 略松（admin override 可绕开 PR review）· 治理强度下降
- 若未来 admin override 频率上升（> 5 次/month 持续）· 可能需要 revisit

### 选项 B · retroactive 补 trailer

**改动**：
- 给 7 个 direct push 各开一个 GitHub issue（或 retroactive PR comment 引用 commit SHA）补 trailer
- 未来每次 admin direct push 都必须立即开 issue 补 trailer

**优点**：
- audit 完整 · 与 v2-D.1 等强度

**缺点**：
- 6 个 dependabot bumps 补 trailer **完全无意义**（bot 自动 · 没"Implemented by 人类"概念）
- session 22-23 累积 1 month 已默认豁免 · 现在补是 retro-fitting 治理而非治理本身
- 操作成本高（7 个 issue / PR comment · 各 2-3 min · 每月若有 10 个 dependabot 滚动 · 持续负担）

### 选项 C · 升级 pre-push hook 强制

**改动**：
- `.githooks/pre-push` 加：检测 main 上的 commit message 不含 trailer / 不含 admin marker → 阻止
- Arbiter override：`SKIP_BRANCH_PROTECT=1`（已有）· commit body 必含 `admin override` 字符串
- dependabot 走 GitHub auto-merge · 不经过本机 hook · 自然豁免

**优点**：
- 技术兜底 · 不靠人肉自觉
- 主 agent 强制要写 marker · 治理强度高于选项 A

**缺点**：
- hook 复杂化（已有 branch protection 检测 · 加 commit message 检测）
- 仅检测本机 push · GitHub UI / API 直推可绕过
- dependabot / renovate 配置任何变化都要 revisit hook
- v0.3+ 可选增强 · 不在 v0.2 sprint 范围

---

## 决策（Decision Outcome）

**选择**：选项 A · v2-D.2 显式豁免 + commit body audit marker

**理由**：
- 与 ADR-012 选项 A 同思路（"规则简化 · 实务对齐"）· 一致性强
- 实施成本最低（< 30 min · 同 PR 改 ADR + CLAUDE.md）· 不阻塞 v0.2 sprint
- 选项 B 已 1 month 不补 = de facto 默许豁免 · 不再补即接受
- 选项 C 留 v0.3+ 评估（pre-push hook 升级 · 见下方风险 R1 fallback）

**v2-D.2 完整规则**（替换 v2-D.1 §(2) "必须" 段）：

> **必须（单人项目 self-review + Arbiter approval · v2-D.2 简化版 · admin override 豁免增强）** · PR body 含以下 3 行即算合规：
>
> - `Implemented by: <agent-id>`
> - `Reviewed by: <agent-id · self-review 或 internal cross-review>`
> - `Arbiter approval: tajiaoyezi · YYYY-MM-DD HH:MM · "<dialogue 摘要>"`
>
> **admin direct push 豁免条款**（v2-D.2 新增 · 2026-05-03 ADR-016）：
>
> 直接 push 到 main 的 commit（不经过 PR · 含人工 admin + dependabot/renovate bot auto）**豁免 PR body trailer 要求**· 但 commit body **必须含 audit marker**：
>
> - **人工 admin push**：commit body 第一段后显式写 `admin override · 原因：<X>` 一行 · X 必须是具体可审计的原因（例：`GitHub Actions billing 暂停 · CI pending 卡死`）· 不接受空泛理由（`紧急修复` / `临时绕过`）
> - **bot auto push**（dependabot / renovate / 类似）：默认 commit format 已含 source ref（"Bumps X from A to B"）· 视为足够 audit · 无需额外 marker · 主 agent 不为此类 commit 主动追溯
>
> **不接受**：人工 admin push 不写 audit marker · 视为违反 v2-D.2 · audit 失守

---

## 后果（Consequences）

### 正面

- **治理空白填补**：v2-D.1 未涵盖的 admin direct push 场景明确规则
- **session 22-23 audit 项关闭**：不再悬空 · PROGRESS Next concrete action 移除此项
- **未来 admin override 合规路径清晰**：GitHub Actions billing 中断 / 紧急修复时不必担心规则违反
- **audit trail 不丢**：commit body marker 是永久 audit · 未来 v2-strict 升级时可追溯
- **bot auto 现实接受**：dependabot 6 个 bumps（session 21）合规 · 未来 dependabot 滚动不累积技术债

### 负面

- **治理强度略降**：v2-D.2 比 v2-D.1 略松 · admin override 可绕开 PR review
  - 缓解：本 ADR 明确"admin override 应是异常 · 不是默认"· 主 agent 自我约束 · 频率监控（PROGRESS session 末统计）
- **commit body marker 仍靠人肉**：选项 A 和 v2-D.1 一样 · 没 hook 强制
  - 缓解：v0.3+ 评估选项 C（hook 升级）

### 风险

- **R1 · 主 agent 把 admin override 当默认 · 绕过 PR review**
  - 缓解：本 ADR §决策段明确"admin override 应是异常 · 不是默认"
  - 监控：每 session 末（PROGRESS 更新时）统计本 session admin direct push 次数 · > 1 次需在 session-history 注明原因
  - Fallback：若连续 2 session admin override 次数 > 5 · 触发选项 C（pre-push hook 升级 · 写新 ADR）

- **R2 · dependabot commit format 变化 · audit 断**
  - 缓解：低概率 · dependabot 由 GitHub 维护 · 格式相对稳定
  - 触发：若 dependabot 升级使 commit format 不再含 source ref（"Bumps X from A to B"）· revisit 本 ADR

- **R3 · 人工 admin push audit marker 写得太空泛**（"紧急" / "临时"）
  - 缓解：本 ADR §决策段明确"不接受空泛理由 · 必须可审计的具体原因"
  - 兜底：主 agent 在 session 末 audit 时 · 若发现空泛 marker · 在 session-history 补充实际原因 + revisit ADR 是否需要更严格

- **R4 · v2-strict 升级时（真合作者加入 / 仓库公开）忘记加回 trailer 强制要求 to admin push**
  - 缓解：触发 v2-strict 的 ADR（未来未编号）必须 explicit revisit 本 ADR 的豁免条款 · 决定是否保留
  - 兜底：本 ADR §相关段挂勾子 · 未来 v2-strict ADR 必须引用本 ADR

---

## 与 `implementation-plan.md` 的映射

不直接映射 · 治理决策。`implementation-plan.md` §11 路线图不涉及 v2-D.x governance 演进。

---

## 相关（Links）

- [ADR-006 · 桌面框架 + v2-D 单人项目 Arbiter approval 起源](./ADR-006-desktop-framework.md)（session 10 末 · 2026-04-19）
- [ADR-012 · v2-D → v2-D.1 简化](./ADR-012-v2d1-arbiter-approval-simplification.md)（session 13 · 2026-04-21 · 同决策思路）
- [`CLAUDE.md` §"改锁定表 A 栏前必须" §(2)](../../CLAUDE.md)（本 ADR accept 后同 PR sync v2-D.1 → v2-D.2）
- session 21 PROGRESS.md 段（已归档至 [`session-21.md`](../session-history/session-21.md) · 详 admin override 模式实证）
- session 23 audit 决议（本 ADR · 2026-05-03 闭合 audit 项）

### 未来触发 Revisit 的条件（任一满足）

1. 连续 2 session admin direct push 次数 > 5（监控见 R1）
2. dependabot commit format 不再含 source ref（R2）
3. v2-strict 升级（真合作者加入 / 仓库公开 + branch protection · R4）
4. pre-push hook 升级（选项 C · v0.3+ 增强）
