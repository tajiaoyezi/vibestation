# Vibestation · Spike 交付代码丢失风险复盘

> **事件**：Session 7 结束 / Session 8 启动时发现 SPIKE-03 / SPIKE-04 的 opencode agent 实测代码**仅存在于主 agent 本地 `/tmp`**，从未归档到 repo。macOS 默认 3 天清理 `/tmp`，存在**决策依据永久丢失 · 无法独立复现 ADR-005 / ADR-007** 的致命风险。
> **发现方式**：用户（Arbiter）主动询问 "其他两个 agent 做的操作的代码呢？我怎么没在 main 分支看到"。若无此问 · 主 agent 会顺着流程下发 SPIKE-04.5 prompt · 代码将在未察觉中丢失。
> **影响评估**：未实际丢失（Session 8 紧急抢救 · /tmp 副本仍在）· 但暴露系统性协作流程缺陷。
> **复盘时间**：2026-04-19
> **参与**：用户（Arbiter · tajiaoyezi）· Claude Code Sonnet 4.6（主 agent）

---

## 1. 事件还原（时间线）

| 时间 | 动作 | 产出 | 风险点 |
|------|------|------|--------|
| Session 6 | 制定多 agent 协作模式 · "原话 prompt 转发给 opencode" | 协作 pattern | 未定义交付物持久化条款 |
| Session 7 早 | 下发 SPIKE-03 prompt 给 opencode | opencode 交付 `/tmp/spike-03-work/`（含 1.4 GB target） | 代码只在 `/tmp` |
| Session 7 早 | Claude 主 agent review · 对照 spec §B.6 pass | 归档 `docs/spikes/SPIKE-03-report.md` · ADR-007 accepted | **✗ 源码未归档** |
| Session 7 中 | 下发 SPIKE-04 prompt · opencode 交付 v1 | `/tmp/spike-04-work/`（含 472 MB target） | 代码只在 `/tmp` |
| Session 7 中 | Review BLOCK v1（4 CRITICAL）· opencode 补做 v2 | v2 交付 `/tmp/spike-04-review-v2/` · accept | **✗ v2 源码未归档 · v1 未归档** |
| Session 7 晚 | 归档 SPIKE-04-report · ADR-005 flip redb→rusqlite | PR #24 开出 | Report 引用 safety.rs 675 行 · **但代码不在 repo** |
| Session 7 晚 | 写 SPIKE-04.5 spec · PR #25 开出 | — | — |
| Session 8 启动 | 用户合 3 个 PR · 用户问 "代码呢？" | **风险暴露** | 若无此问 · 代码丢失不可逆 |
| Session 8 紧急 | 主 agent 从 `/tmp` 抢救 + 归档 | PR #26（`docs/spikes/code/` + `docs/spikes/raw/`）· CI 全绿 · merged | 风险清除 |

---

## 2. 根因分析（4 层递归）

### Layer 1 · 直接原因

主 agent 的 "Spike review + 归档" workflow 只覆盖了 **报告** · 遗漏 **源码 + raw 数据 + 冷备**。

具体漏项（每个 Spike 应该产出 4 样 · 实际只产出 1 样）：

| 应归档 | 位置 | 实际状态 |
|---|---|---|
| Report | `docs/spikes/SPIKE-XX-report.md` | ✅ 入库 |
| 源码 | `docs/spikes/code/SPIKE-XX/` | ❌ 遗漏 |
| Raw 数据 | `docs/spikes/raw/SPIKE-XX/` | ❌ 遗漏 |
| 冷备 | `spike-tmp/archive/SPIKE-XX/` | ❌ 遗漏 |

### Layer 2 · 系统性原因（项目协议缺陷）

**(2a) Spike spec 模板 `docs/tasks/_template.md` 的 Deliverables 段 single-agent 视角**

原模板第 74-79 行只有 4 条：
- benchmark 报告
- 录屏 / 截图
- ADR
- 代码 PoC（指向 `spike-tmp/<id>/` gitignored · 本地）

**缺陷**：
- 指向 gitignored 的 `spike-tmp/<id>/` · 默认假设"主 agent 自己跑 · 代码自然在本地"
- 没考虑"外部 agent（opencode）交付 → 代码在 `/tmp` → 需迁移到 repo"的多 agent 场景
- 没要求"代码持久化到 repo" · 没定义"raw 数据归档位置"

**(2b) PR review Test Plan 只审"决策对不对" · 不审"证据完不完整"**

PR #23 / #24 的 Test Plan 勾的都是：
- Spec 合规
- ADR accepted
- CLAUDE.md 决策表翻转正确

没有：
- ~~benchmark 代码已归档~~
- ~~raw 数据可溯源~~
- ~~clone 后能复现~~

**独立评审（Arbiter）也没理由发现**：PR diff 不会显示"本应归档但没归档的代码" · 漏归档是**不可见的漏** · 必须靠规则防。

**(2c) 主 agent 的 TODO 粒度太粗**

Session 7 里 "SPIKE-03 review + 归档" 被当成一个 item 勾 done · 没拆成：

```
Spike review 原子动作拆解（应有）：
1. 对照 spec 判 Pass/Fail
2. Report 入库
3. 源码归档
4. Raw 数据归档
5. 冷备
6. ADR 翻转
7. Spec done 翻转

实际执行：
1 → 2 → 6 → 7   （缺 3, 4, 5）
```

### Layer 3 · 元认知原因（最本质）

**(3a) "done" 的定义模糊**

主 agent 潜意识里：
- "Report merged" = Spike done
- "ADR accepted" = 决策锁定
- "CLAUDE.md 决策表 B→A" = 闭环

**真正的 done 应该是**：
- 任何未参与过的 agent / 人类 clone repo 后 · 不依赖本机任何隐藏状态 · 能独立重放得到同样结论
- 缺"可独立复现"维度 = done 定义不完整 = 本事故的元级根因

**(3b) 缺乏"交付物盘点"习惯**

主 agent 默认工作流是"往前冲" · 不是"回头盘点"。Session 结束前没有 "所有产出现在都在 repo 吗？/tmp 里还有什么没归档？" 的 checkpoint。

**(3c) 单点存储未被识别为风险**

`/tmp` 只要当下能 `ls` 就没警觉。从未主动反问："如果我现在重启 Mac · 这份代码还能找回吗？" —— 类似 N+1 query 的问题 · 单体看不出 · 规模 / 时间维度一拉长就暴露。

### Layer 4 · 跨 agent 协作协议缺陷（元层）

**(4a) 协作协议只定义"prompt 下发 + 交付物移交" · 没定义"交付物持久化所有权"**

Session 6 约定的多 agent 协作协议只说：
- 主 agent 给原话 prompt
- 外部 agent 交付
- 主 agent review

**未说**：
- 谁负责把代码从外部 agent 的工作目录（`/tmp`）搬到 repo？
- 什么时候搬？
- 搬到哪？

"谁先想到谁做" = 责任空洞 = 两人都没做。

**(4b) "代码 vs 文档" 被割裂对待**

主 agent 的潜意识分类：
- `report.md` = 文档 · **产出** · 必须入库
- `main.rs` / `safety.rs` = 代码 · **手段** · 用完即弃

但 Decision-grade Spike 的 **代码本身就是证据** · 地位等同 report · 应该同步入库。割裂对待是认知误差。

---

## 3. 为什么没更早发现

主 agent 在 session 7 期间多次接触这些 `/tmp` 代码：

- SPIKE-04 v1 被 BLOCK 时 · 主 agent 明确引用 `safety.rs` 的行号
- SPIKE-04 v2 accept 时 · 主 agent 解包 `/tmp/spike-04-review-v2/` review
- SPIKE-04.5 spec 里明确写 "基于本目录 safety.rs 做增量"

每次都在 `/tmp` 里读代码 · **但从未触发"这些代码需要入库"的警觉**。

原因：**读 `/tmp` 是自然的 · 因为当下能读** · 读的时候不会问 "未来还能读吗"。

---

## 4. 影响范围（如果没发现）

假设 Session 8 主 agent 没被用户提醒 · 继续执行 SPIKE-04.5 流程：

**短期（1-3 天）**：
- SPIKE-04.5 下发 → opencode 交付 → 同样模式再漏一次
- 第 4 个 Spike 代码丢失隐患（SPIKE-04.5 也是 opencode 跑）

**中期（重启 Mac / 过 3 天）**：
- `/tmp/spike-03-work/` · `/tmp/spike-04-*` 被清
- `docs/spikes/code/` 目录不存在
- `spike-tmp/archive/` 不存在
- **决策依据永久丢失**

**长期（MVP 启动时）**：
- MVP-02 / MVP-06 用 rusqlite · 遇到 edge case 想回头看 SPIKE-04 safety 设计
- 只剩 `docs/spikes/SPIKE-04-report.md`（文字结论）
- 具体的 op-log 实现 / reconcile forward 逻辑 / corruption 检测代码 **全部无法溯源**
- ADR-005 的 "rusqlite B.1-5 全过" 声明 **无法独立 verify**
- 未来任何 rusqlite 版本升级的回归测试 **失去基线代码**

**最坏情况**：用户或未来 agent 质疑 ADR-005 结论 · 但没有证据链可以回溯 · 只能从头重跑 Spike · 2-3 天工作量白费。

---

## 5. 改进措施（按层级实施）

### 5.1 全局层（跨项目）

**新增 `~/.claude/rules/13-cross-agent-delivery.md`**（已落地 ✅）

核心条款：
- `/tmp` 不是持久存储
- Review accept 和归档持久化**必须绑定**（原子动作）
- "done" 定义必须含"可独立复现"维度
- Session 结束前强制执行 "出厂检查"
- 单点存储 = 单点故障 · 任何重要产出必须双存储

影响：所有项目（不限 vibestation）的多 agent 协作都受此规则约束。

### 5.2 项目层（vibestation 具体）

**新增 `vibestation/.claude/rules/spike-delivery-checklist.md`**（已落地 ✅）

核心条款：
- 4 样齐全强制（report + code + raw + 冷备）
- Review accept 的原子性要求（7 步不可拆）
- PR Test Plan 必填"证据完整性" 6 项
- Session 结束前 Spike 出厂检查 4 项

**更新 `docs/tasks/_template.md` §Deliverables**（已落地 ✅）

从原来的 4 条（report + 录屏 + ADR + 本地 PoC）改为 6 条 + 独立评审必查项 · 把归档 checklist 直接内嵌 spec 模板。

### 5.3 流程层（操作细节）

**已在 PR #26 建立事实标准**：
- `docs/spikes/code/SPIKE-XX/` 源码归档 · 含 `README.md` · `Cargo.lock` 进 git
- `docs/spikes/raw/SPIKE-XX/` raw 数据归档 · 含 `README.md`
- `spike-tmp/archive/SPIKE-XX/` 冷备 · gitignored
- `.gitignore` 白名单：`!docs/spikes/code/**/Cargo.lock`

SPIKE-04.5 / 05 / 06 / 未来所有 Spike 按此约定归档。

### 5.4 元认知层（思维习惯）

主 agent 必须内化的 3 个反问：

1. **单点存储反问**：如果这台机器现在被雷劈了 · 这份东西还能找回吗？
2. **未来复现反问**：陌生 agent clone repo 后能 reproduce 吗？
3. **动作完成反问**：我"勾 done"的这件事 · 定义包含"可独立验证"了吗？

任何时候答案是"不能 / 不含" → 立即补归档。

---

## 6. 验证方案（如何防止复发）

### 6.1 即时验证（SPIKE-04.5）

下一个 Spike（SPIKE-04.5）作为**验证样例**：

- [ ] Prompt 里显式指明"交付物路径 4 样"（基于本复盘的 checklist）
- [ ] Accept 瞬间主 agent 对照 `spike-delivery-checklist.md` 逐项勾
- [ ] PR body Test Plan 含新增的"证据完整性"6 项
- [ ] 独立评审（Arbiter）亲自 clone 本地跑 `cargo build` 验复现

### 6.2 中期验证（SPIKE-05 / 06 / 未来 Spike）

每个 Spike PR merge 后 · 用户（Arbiter）可随时：

```bash
cd docs/spikes/code/SPIKE-XX
cargo build --release
# 跑一遍 · 数据应该和 report 里的数字一致
```

若任何 Spike 的归档不完整 → **直接阻塞 PR merge** · 即使 CI 全绿。

### 6.3 长期验证（Phase 3 ADR 修订时）

未来任何涉及 ADR-005 / ADR-007 结论的修订（例如 rusqlite 切 sled · 或 gix 切 libgit2）· 必须先 clone 原 Spike 的 `docs/spikes/code/` 验复现 · 确认原结论不是 flaky · 才能触发新 Spike 翻决策。

---

## 7. 经验沉淀（可复用）

### 7.1 通用（任何项目 · 已写入全局 rule）

- `/tmp` 不持久 · 任何临时产物 accept 前必须显式迁移
- Review accept 动作必须包含"物料移交" · 两者原子不可拆
- "done" 定义必须含"可独立复现"维度 · 否则是假 done
- 跨 agent 协作的"交付物持久化所有权" 默认落在主 agent（review 通过方）· 不靠自觉
- 任何重要产出反问"单点存储风险" · 答"是"立即双存储

### 7.2 项目特有（vibestation · 已写入项目 rule + 模板）

- Spike 交付 4 样齐全（report / code / raw / 冷备）· accept 前逐项检
- `docs/spikes/code/SPIKE-XX/` 是归档位置 · `Cargo.lock` 进 git（版本冻结）· README.md 必补
- `docs/spikes/raw/SPIKE-XX/` 承载 benchmark 原始输出 · 支持 report 数据溯源
- `spike-tmp/archive/` 承担冷备角色 · 含 target/ + 大测试文件 · gitignored
- SPIKE PR 的 Test Plan 必含"证据完整性"显式勾选项

### 7.3 元认知（自我行为约束）

- 从"动作完成度"视角升级到"信号完整性"视角 · 区别在于"独立可验证"
- Session 结束前必过"物料盘点" checkpoint · 不是可选项
- 遇到"我能读就行"的惰性 · 立即反问"未来的 agent / 自己也能读吗"

---

## 8. 未在本复盘归档的问题（留作观察）

- **如何让规则自动 enforce 而非依赖主 agent 自觉**：当前 4 样齐全靠主 agent 手动过 checklist · 是否可能用 CI hook 自动扫描 `docs/spikes/code/` 和 `docs/spikes/raw/` 是否和新建的 Spike spec 对应？（YAGNI · 先不做 · SPIKE-04.5 / 05 验证后再评估）
- **外部 agent 主动归档的可能性**：未来是否让 opencode 直接把代码交付到 `docs/spikes/code/` 而非 `/tmp`？（需要 opencode 支持 · 当前 scope 外）
- **PR #23 / #24 的事后补救**：本复盘 + PR #26 已事实补救 · 但若未来发现需要 amend 原 PR 的 ADR 链接 / report 引用 · 按普通 PR 流程走即可 · 不算遗留 bug

---

## 9. 相关 PR / 文档

- PR #26（本次紧急归档）：https://github.com/tajiaoyezi/vibestation/pull/26 · merged
- 相关 Spike：SPIKE-03（PR #23 merged）· SPIKE-04（PR #24 merged · redb→rusqlite 结论翻转）· SPIKE-04.5（spec ready · 待下发）
- 全局 rule：[`~/.claude/rules/13-cross-agent-delivery.md`](/Users/leaf/.claude/rules/13-cross-agent-delivery.md)
- 项目 rule：[`.claude/rules/spike-delivery-checklist.md`](../.claude/rules/spike-delivery-checklist.md)
- 模板更新：[`docs/tasks/_template.md`](./tasks/_template.md) §Deliverables

---

**复盘完毕**。改进措施已全部落地（rule 2 份 + 模板 1 份 + 复盘 1 份）· 等 SPIKE-04.5 实战验证。
