# Session History · 开发 session 历史 / PR 复盘 / 关键讨论归档

> 本目录存放**关键开发 session 的反思记录 · PR close / revert 的复盘 · 重大讨论的沉淀**。
> Phase 3 建立（2026-04-18）· 随项目演进填充。

---

## 📂 命名约定

```
docs/session-history/
├── README.md                                (本文件)
├── YYYY-MM-DD-<topic-slug>.md               复盘 / 讨论记录
└── pr-NN-<result>.md                        PR close / revert / 复杂讨论
```

**示例**：
- `2026-04-18-phase-2-codex-five-rounds-retrospective.md`（Phase 2 Codex 5 轮对抗性审查复盘）
- `pr-4-close-why-per-task-files.md`（PR #4 close · 为什么改 per-task 报告而不是共享文件）
- `2026-05-15-spike-02-tauri-wayland-issue.md`（若 SPIKE-02 遇 Wayland bug 的解决记录）

---

## 📝 每份文档的推荐结构

```markdown
# <标题>

**日期**：YYYY-MM-DD
**参与者**：<作者 agent-id> · <evaluator> · <用户>
**类型**：PR close / PR revert / Spike 复盘 / 架构决策讨论 / Codex 审查复盘 / 其他

## 背景

<!-- 为什么写这份记录 · 对应哪个事件 -->

## 事实还原

<!-- 按时间线写发生了什么 · 客观事实 -->

## 根因分析

<!-- 为什么会发生 · 深层原因 -->

## 教训 / 改进措施

<!-- 具体可执行的改进 -->

### 通用层（写全局 rule）

<!-- 通用行为规则 · 写到 ~/.claude/rules/ · 避免再遇同类问题 -->

### 项目特有（写本项目 memory）

<!-- 项目特有案例 / 反面样本 · 写到 project memory -->

## 后续动作

- [ ] 更新 `CLAUDE.md` 决策表 / `implementation-plan.md` / ADR 等（如需要）
- [ ] 开对应 PR · 引用本记录

## 相关

- PR：#N
- ADR：ADR-NNN
- 前置讨论：...
```

---

## 🔄 什么时候写 session-history

**必须写**：
- **PR close / revert**（不是 merge · 需要解释为什么不走了）
- **关键架构讨论**（若 ADR 不适合承载完整过程 · 比如多轮评审迭代）
- **Spike 失败**（触发 fallback 时需要留详细根因）
- **Codex / 其他 AI 对抗性审查的整轮复盘**（每 5-10 轮写一次总结）

**建议写**：
- 某个 PR 讨论超过 10 条评论 · 记录关键决策点
- 多 agent 并发出现冲突 · 记录仲裁过程

**不需要写**：
- 常规 PR（按 Conventional Commits + task spec 已够）
- 日常 bug 修复（走 Issues / PR · 无需单独记录）

---

## 📂 已归档 Session

| 文件 | 日期 | Session | PR 范围 | 主题 |
|------|------|---------|---------|------|
| [session-27.md](session-27.md) | 2026-05-10 | 27 | #264-#266 | v0.3 sprint phase A+B+C 全收 + Phase D 启动 · 3-track 模式实证 · bench-only PR 模式开启 |
| [session-26.md](session-26.md) | 2026-05-09 | 26 | #259-#263 | 4-track 文件域隔离首次实证 · 单 day 4 PR concurrent · v0.3 sprint phase B+C 大跃进 · OpenCode §2.10 第 2 次 → N=3 永久转出条款 |
| [session-25.md](session-25.md) | 2026-05-07 | 25 | #251-#253 | v0.3 sprint phase A 50% 启动 · MVP-15/16 Phase A · 主 agent reviewer 翻转 gate (a) 实战 · OpenCode 谎报 lint/typecheck 首次 |
| [session-24.md](session-24.md) | 2026-05-04 | 24 | spec only | v0.3 sprint kickoff · 4 agent 并发 spec 详化 6 PR |
| [session-21.md](session-21.md) | 2026-04-29 | 21 | #173-#175 | v0.1.0-alpha 双平台 GA · macOS unsigned .dmg + Linux .deb / .AppImage |
| [session-20.md](session-20.md) | 2026-04-26 | 20 | #152-#168 | ADR-015 accepted + PR #157 round 1/2 inline 反模式 → §2.13 规则化 |
| [session-19.md](session-19.md) | 2026-04-25 | 19 | #117-#152 | 史上最高产 36 PR · MVP-11 5/5 ✅ + MVP-05 Phase A/B/C + ADR-006 Ubuntu validated + branch protect 机械化 + ADR-015 accepted |
| [session-18.md](session-18.md) | 2026-04-25 | 18 | #106-#116 | 4 track 并发极致产出 · 11 PR merge · 5 Phase 落地 + 3 spec ready 加强 |
| [session-17.md](session-17.md) | 2026-04-23 | 17 | #99-#105 | MVP-04 Phase F 收口 + MVP-08 Phase A/B/C 落地 + PR Actions 分钟节流 |

## 🗂️ 待归档（M-2 滚动窗口外 · 按需补）

以下 session 仍在 git 历史中可追溯 · 但未单独归档（PROGRESS.md M-2 滚动窗口仅保留最近 2 session 摘要 · 更早信息需 `git log` 检索 · 不强制单独成文）：

- **session 22 / 23**（2026-04-30 ~ 2026-05-03 · Apple Dev / dependabot 批处理阶段 · 详见 `git log --grep="session 2[23]"`）
- **session 28**（2026-05-12 · 单 day 9 PR merged · 4-track + 5 idle 查漏补缺 · MVP-15 Phase D §F+§G · 当前 PROGRESS.md 头条 · 下次 session 末归档）
- **session 29**（2026-05-12 ~ active · MVP-17 spec ready + Phase B skeleton + 3-track dispatch 启动 · 主 agent 已 7 PR merged · Codex Phase A / OpenCode Phase C 待 push · in-progress 状态 · session 结束后归档）

## 🗂️ 预期归档（Phase 2 历史复盘 · 按需补）

以下为 Phase 2 积累的可归档话题（在事件复盘需要时按需补入）：

- **2026-04-17 ~ 2026-04-18 Phase 2 Codex 5 轮审查**（10 HIGH findings · PR #9 3 commits · Codex companion 对抗性审查方法的效果评估）
- **PR #4 close · 为什么不用共享 `SPIKE-REPORT.md`**（物理隔离 vs 声明式并发治理 · Codex PR #5 R2 F1 教训）
- **Phase 4 CI / gitleaks / task-spec-validator 设计**（从 advisory 到 enforced 的升级路径）

---

## 🔗 相关

- `CLAUDE.md §12. 规则即行动触发器`（元任务的专属检查清单）
- `CONTRIBUTING.md` · 贡献流程（包括什么时候开复盘）
- `docs/adr/` · 正式架构决策的承载（≠ session-history 侧重"过程"）

---

## ⚠️ 安全约束

本目录文件**不得**含：
- auth token / API key / PII
- 真实用户数据 / 生产日志
- 未脱敏的 CLI 输出

gitleaks CI（Phase 4）会扫此目录。

---

**本目录 Phase 3 建立（2026-04-18）· 具体记录在真实事件发生时补入。**
