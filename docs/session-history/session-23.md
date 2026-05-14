# Session 23 · 2026-05-02 ~ 2026-05-04

**session**: 23
**date**: 2026-05-02 ~ 2026-05-04（3 day · 当前已归档 session 中最大产出）
**pr_range**: #207-#233（27 PR merged）
**theme**: 收口 v0.1 残留 + 启动 v0.2 sprint W13/W14 · MVP-13 全 4 phase 自动化 done + MVP-21 Phase A/B/C done · ADR-016 治理升级落地

---

## 主题摘要

session 23 是项目从 v0.1 收尾切换到 v0.2 sprint 的关键三日窗口。主线不是单点功能，而是“收尾 + 开工 + 治理清债”三轨并行：一方面完成 v0.1 残留修复与文档收口，另一方面在同一 session 内把 v0.2 的 W13（MVP-13）与 W14（MVP-21 前三阶段）推进到可交付状态，同时将单人项目治理从 v2-D.1 升级到 v2-D.2（ADR-016），补齐 admin override 场景的制度空白。

本 session 的里程碑价值体现在三个层面：

1. **交付层**：MVP-13 4 个 phase（A/B/C/D）全部完成，自动化验证 100% 到位；MVP-21 在同 session 完成 Phase A/B/C，W14 进入 “仅剩 Phase D” 状态。
2. **方法层**：Codex CLI fast 模式在多阶段实施中形成稳定提速证据（5x-8x，局部 10x），并与主 agent 的验收闭环形成可复用派工模板。
3. **治理层**：通过 ADR-016 把 v2-D.1 升级到 v2-D.2，明确 admin direct push 豁免边界与审计要求，关闭 session 22 留下的治理 audit 项。

---

## 27 PR 分组总览（4 组）

> 说明：按主题分组归档，不逐 PR 展开；每组仅高亮关键 PR。

| 分组     | PR 范围       |   数量 | 主题                                     |
| -------- | ------------- | -----: | ---------------------------------------- |
| A        | #207-#214     |      8 | v0.1 收尾残留 + 生命周期修复 + housekeep |
| B        | #220-#227     |      8 | v0.2 W13：MVP-13 全 4 phase 自动化收口   |
| C        | #228-#233     |      6 | v0.2 W14：MVP-21 Phase A/B/C 完成        |
| D        | #215-#219     |      5 | ID 冲突清理 + 治理升级 + sprint kickoff  |
| **合计** | **#207-#233** | **27** | **session 23 全量 merged**               |

---

## A 组 · v0.1 收尾残留（#207-#214 · 8 PR）

这组是“先把尾巴清干净，再转 sprint”的基础动作。关键价值不是代码量，而是把后续 v0.2 推进中的混淆项提前拔掉。

关键高亮：

- **PR #208**：MVP-05 lifecycle 修复进入多轮 codex review，对齐 4 条 pane lifecycle 不变量，后续沉淀为全局 `systemic-fix-after-review` 规则升级依据。
- **PR #210**：执行 ID 重编第一步，`MVP-11 (Git Push/Pull/Fetch)` 重命名为 `MVP-21`，解决 v0.1 与 v0.2 命名冲突根因。
- **PR #211**：MVP-05 Phase D capture playbook 大体量落地（收口 Arbiter GUI capture 方法）。
- **PR #212/#213/#214**：session 23 收尾 housekeeping、PROGRESS 同步、GUI capture defer，确保主线切到 v0.2 时文档状态一致。

这 8 PR 完成后，v0.1 的“技术债 + 叙事债 + 文档债”被压缩到可控范围，给 W13/W14 开工腾出干净上下文。

---

## B 组 · v0.2 W13（MVP-13 全 4 phase）#220-#227 · 8 PR

这是 session 23 的第一条硬交付主线：在同一 session 完成 MVP-13 的 Phase A/B/C/D 全闭环。

### 核心实施 PR（4 个）

- **PR #220（Phase A）**：branch_ops 后端 + 5 IPC + ts-rs binding，构建 MVP-13 写路径地基。
- **PR #222（Phase B）**：Primary Sidebar branch tree UI 接线落地。
- **PR #224（Phase C）**：Fuzzy Switcher modal 落地，交互与性能门槛同时达成。
- **PR #226（Phase D）**：runtime bench evidence 与性能数据归档，自动化验证收口。

### 同步与收口 PR（4 个）

- **PR #221/#223/#225/#227**：分别对应 A/B/C/D 各阶段的 PROGRESS 与 spec 后同步，确保“实现状态”与“文档状态”不漂移。

### 结果

- MVP-13 在 session 23 内完成 **4/4 phase**。
- 自动化校验路径闭环，GUI screenshots 按规则 deferred，不阻塞主线合并。
- Codex CLI 在该线形成稳定 fast 模式证据，后续可复用于同复杂度任务派工。

---

## C 组 · v0.2 W14（MVP-21 Phase A/B/C）#228-#233 · 6 PR

这是 session 23 的第二条硬交付主线，目标是把 MVP-21 从 ready 推到“仅剩 Phase D”。

### 核心实施 PR（3 个）

- **PR #228（Phase A）**：git sync backend（push/pull/fetch/auth/事件）完整打底。
- **PR #231（Phase B）**：git sync UI（5 dialogs + GitLogPanel 大规模接线）完成。
- **PR #233（Phase C）**：status bar ahead/behind + per-workspace remote-sync-status 落地。

### 支撑与同步 PR（3 个）

- **PR #229**：NetworkOpError variant 数审计纠偏，避免 spec/实现错位。
- **PR #230/#232**：Phase A、Phase B 完成态同步，保障 W14 看板与真实代码进度一致。

### 结果

- MVP-21 在 session 23 内完成 **Phase A/B/C**。
- W14 进入“约 75% 完成，仅剩 Phase D”的可预测收尾阶段。
- 核心功能链条（backend contract -> frontend workflow -> status feedback）已连通。

---

## D 组 · ID 冲突清理 + 治理升级（#215-#219 · 5 PR）

这组是 session 23 的“工程治理面”主线，决定后续并发派工是否可持续。

关键高亮：

- **PR #215**：MVP-13 spec 从 draft -> ready，解锁 W13/W14 串联节奏。
- **PR #216/#217**：清理 rename 残留 + MVP-21 spec self-review，对齐文档与实施路径。
- **PR #218（ADR-016）**：治理升级 **v2-D.1 -> v2-D.2**，新增 admin override 豁免条款与审计边界，关闭长期 audit 悬项。
- **PR #219**：v0.2 sprint kickoff 文档，统一时间线、风险项与交付预期。

这 5 PR 的意义在于：不仅推进功能，还把“并发协作的规则系统”补到可运行状态，避免后续 session 在治理细节上反复返工。

---

## 协作模式（session 23 实证）

本 session 采用“主 agent 协调 + Codex fast 实施 + Explore 审计 + Kimi 远程 review”的五度验证协作链：

1. **主 agent（Claude Code）**：任务编排、验收 gate、进度同步、最终 merge 质量控制。
2. **Codex CLI fast**：MVP-13 A/B/C/D 与 MVP-21 A/B/C 的主实施引擎，承担大体量编码。
3. **Explore 子 agent**：rename 残留与一致性审计，补齐主路径外的检索盲区。
4. **Kimi 远程 review**：用于 spec/detail 阶段的跨视角校验与风险补强。
5. **Arbiter 决策层**：关键治理条款与节奏切换拍板（如 ADR-016 落地窗口）。

该模式在 session 23 达到“高并发 + 高一致性”的平衡：既有 27 PR 的吞吐，又维持了 trailer 与规则合规的统一口径。

---

## 关键经验沉淀

1. **Codex fast 模式稳定提速**：在 MVP-13 与 MVP-21 的多阶段任务中，提速区间稳定在 5x-8x，局部 Phase C 达到约 10x。
2. **治理升级必须跟实务对齐**：ADR-016 证明单人项目治理规则需要覆盖 admin override 现实场景，否则 audit 会长期悬空。
3. **ID 命名空间冲突要集中清批**：MVP-11->MVP-21 与 MVP-20->MVP-22 的双清理在同 session 完成，显著降低后续 spec 漂移成本。
4. **多轮 review 的价值在于不变量收敛**：PR #208 从“修一个 bug”提升到“固化 4 条生命周期不变量”，可复用价值远高于单次 patch。
5. **双 footnote 历史 trace 模式有效**：关键 rename、治理与规格切换都保留前后映射，使后续审计与新 agent 接手成本下降。
6. **进度同步 PR 不是噪音**：#221/#223/#225/#227/#230/#232 这类 sync PR 保证看板事实一致，是高并发下防漂移关键机制。

---

## 反思

session 23 的核心特征是“同一窗口完成两条 sprint 主线”：W13 的 MVP-13 全闭环与 W14 的 MVP-21 三阶段并行推进，这在项目历史上属于单 session 最高产出区间（3 day / 27 PR）。更重要的是，这次产出并非以牺牲治理换速度，而是同步清理治理债与决策债：ID 冲突成批消解、ADR-016 正式落地、review 方法论沉淀为可复用规则。换言之，session 23 不是一次“冲量”，而是一次“冲量 + 结构化收口”。

---

## 关联

- 上一 session：[`session-22.md`](./session-22.md)（MVP-22 PTY warm pool 主线）
- 下一 session：[`session-24.md`](./session-24.md)（v0.3 sprint kickoff · 4-agent 并发详化）
- 治理节点：[`ADR-016`](../adr/ADR-016-admin-override-trailer-exemption.md)（v2-D.1 -> v2-D.2）
- 数据源：`docs/PROGRESS.md` session 23 展开段（L129-L178）+ PR #207-#233 合并记录

---

## 归档元信息

- **archive 时间**：2026-05-14（session 31 M-2 滚动窗口 housekeeping）
- **archive 执行**：Cursor（IDE 内嵌 chat agent）
- **范围约束**：本 PR 仅新增 `docs/session-history/session-23.md`，不改 `PROGRESS.md` / `CLAUDE.md` / ADR / task specs
