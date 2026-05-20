# ADR-023: MVP / feature 类 capture mandate 移除（supersede ADR-011）

**状态**：accepted
**日期**：2026-05-20
**决策者**：Claude Code (Opus 4.7 · session 34 · 提议) · User (Arbiter · 2026-05-20 · "C 全部删掉" + 4 sub-decision confirm)
**Supersede**：[ADR-011](./ADR-011-runtime-evidence-location.md)（runtime evidence location · accepted @ 2026-04-19）
**对应 CLAUDE.md 决策表**：无（capture 不在 A 栏锁定项 · 但实质是 spec 设计基线变更）
**影响范围**：15 spec acceptance（MVP-04/05/06/08/09/10/12/13/14/15/16/17/18/19/20）+ 项目级 rule + playbook + readiness 文档

---

## 背景与问题（Context and Problem Statement）

ADR-011（2026-04-19 accepted）锁定了 vibestation 的 capture-as-done-gate 设计：

- MVP / feature 类 PR 必须交付 ≥ 3 张截图 / 30s 录屏放 `docs/runtime-evidence/<task-id>/`
- `.claude/rules/runtime-evidence-location.md` R1-R5 硬规则
- 各 MVP spec §I.5 / §F.4 / §D Phase D 都依此设计 "5+ 张 GUI 截图" 等 acceptance

**实测合规情况（2026-05-20 inventory）**：

| status         | 数  | spec                                 |
| -------------- | --- | ------------------------------------ |
| ✅ done        | 17  | 已完整收口                           |
| 🟡 ready       | 11  | MVP-04/05/06/08/09/10/12/13/15/16/17 |
| 🟡 in-progress | 4   | MVP-14/18/19/20                      |

**15 个 spec 卡在 ready / in-progress · 全因 capture 未补**（代码侧 acceptance 已全过）：

- v0.1 老 deferred：MVP-04 §I 22 PNG + 2 MOV / MVP-05 #211 14 invariant 6 PNG / MVP-09 GUI 截图 / MVP-10 §F.04 DevTools Network panel
- v0.2/v0.3：MVP-12/13/14/15/16/17 各种 Phase D capture
- v1.0 vision：MVP-18 §I.5 5+ 截图 / MVP-19 Phase E 15 截图 / MVP-20 Phase E 15 截图

**问题暴露**：

1. **合规率不可持续**：自 MVP-04（session 7 · 2026-04-19）起 ≥ 9 个月累计 15 spec 卡 capture gate · 实际无人系统性补齐 · 与 v2-D.2 治理"持续推进"目标不匹配
2. **GUI capture 必须 Arbiter 本人启窗口**：CLI agent 结构性无法替代（无 webview / 无 DevTools Performance / 无 Lighthouse / 无跨平台环境）· memory `capture-task-agent-readiness-not-fabricate` 明确禁编造
3. **代码侧 acceptance 已足够**：每个 spec 都有完整 cargo test / vitest / Criterion bench / 性能 DevTools 数字门 · 这些已能保证功能正确性和性能门槛
4. **session 34 PR-A/B/C #401-#403 PRE-CAPTURE-READINESS** 已建立 "✅ code-side green / ⚠️ gap / 🔴 必须 Arbiter" 三分类 · 但本质仍是把 capture 时间锁推给 Arbiter playbook 窗口 · 不解决根本

**Arbiter 2026-05-20 决策**："直接帮我把截图和录视频等任务全部删掉"+ 选 C 全删（vs B 软化 ADR）+ 4 sub-decision：

1. 保留已捕 evidence 作 audit history（v0.1 ship audit ref）
2. 不动全局 rule `~/.claude/rules/15-runtime-verification-gate.md`（跨项目影响）· 仅删 vibestation 项目级
3. 5 PR 按 track 分（PR-1 spec MVP-04/05/06/08/09 · PR-2 spec MVP-10/12/13/14/15 · PR-3 spec MVP-16/17/18/19/20 · PR-4 rule + playbook + readiness · PR-5 ADR + status flip + sync）
4. MVP-19 #384 历史 PRE-CAPTURE-READINESS 一致性删（不保留作"曾尝试"）

---

## 决策（Decision）

**移除 vibestation 项目级 MVP / feature 类 capture-as-done-gate 设计**：

1. **spec status done gate** = 代码侧 acceptance 全过即可（cargo test / vitest / Criterion bench / 性能 DevTools 数字 / a11y 代码侧 grep）
2. **截图 / 录屏 / GUI capture / Phase D runtime evidence** 类 acceptance 项 = supersede · 不再阻塞 spec done flip
3. **已捕证据保留**作 audit history（v0.1 ship 真实跑过的取证 · 不删 mvp-01/02/03/04/07/10/11/13/14/15/16/17/21/22 等已捕子目录）
4. **全局 rule 不动**：`~/.claude/rules/15-runtime-verification-gate.md` 保持 · 仅删项目级 `.claude/rules/runtime-evidence-location.md`
5. **Spike 类不变**：仍按 `spike-delivery-checklist.md` 4 样齐全（capture 不在 Spike 范畴）

---

## 后果（Consequences）

### 正面

- **15 spec 立即解锁翻 done**：代码侧已全过 · ship 不再被 capture 阻塞
- **治理压力消除**：v2-D.2 "持续推进" 与 capture mandate 张力解决
- **CLI agent 工作范围清晰**：不再需要 "PRE-CAPTURE-READINESS 体检" 桥接 · 直接 code-side acceptance 全过即可 flip done
- **未来 MVP PR review 简化**：reviewer 按 §2.14 dev mode 自验 critical UX path · 不强制 capture 截图归档

### 负面

- **失去 visual proof**：repo 不再有"我们 ship 前真的跑过 GUI"的全套证据链（已捕部分保留 · 但未来 MVP 不强制）
- **user-facing 验收靠 reviewer dev mode**：reviewer 启 `pnpm tauri:dev` 自己验 · 风险是个别 reviewer 跳过 dev mode（按 §2.14 应 enforce）
- **历史 spec 的 §I.5 / §F.4 inline 文字保留作 audit · 但功能 deprecated**：阅读时需先看顶部 deprecation block

### 不影响

- **代码层 ship gate**：cargo test --workspace / vitest / clippy / fmt / typecheck 仍是硬门
- **Spike 4 样齐全**：spike-delivery-checklist.md 不动
- **全局 rule 15**：跨项目仍有效（未来其他项目按 case 决定）
- **已 ship 的 v0.1.0/v0.1.1 alpha audit trail**：mvp-04/10/11 等 PNG 完整保留

---

## 实施（Implementation · session 34 · 5 PR）

| PR   | merge          | 内容                                                                                                                                                                                           |
| ---- | -------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| PR-1 | #405 `1f7ef7d` | MVP-04/05/06/08/09 spec 顶部加 deprecation block + Phase 表 capture 行 status flip ✅ done + 起点 hint 段删 capture follow-up                                                                  |
| PR-2 | #406 `f961a3a` | MVP-10/12/13/14/15 同上                                                                                                                                                                        |
| PR-3 | #407 `28cabfb` | MVP-16/17/18/19/20 同上                                                                                                                                                                        |
| PR-4 | #408 `37bfb63` | 删 21 文件（rule + 2 playbook + 10 PRE-CAPTURE-READINESS + 3 CAPTURE-PLAYBOOK + 3 metrics + 1 CAPTURE-GUIDE + 2 capture script）+ 改 dispatch-prompt-template §2.3 / README.md cross-reference |
| PR-5 | 本 PR          | ADR-023 + ADR-011 supersede marker + 15 spec frontmatter `status: ready/in-progress → done` + PROGRESS / CLAUDE / tasks/README sync                                                            |

---

## 关联

- **Supersede**：[ADR-011](./ADR-011-runtime-evidence-location.md)
- **不影响**：
  - [ADR-018](./ADR-018-ai-aware-r1-rejudge.md) AI-Aware R1 greenlight（v1.0 vision 实施 ≠ capture）
  - [ADR-022](./ADR-022-dispatch-template-ref-path-staleness.md) dispatch 范本路径
  - 全局 rule `~/.claude/rules/15-runtime-verification-gate.md`
- **影响 spec**：MVP-04/05/06/08/09/10/12/13/14/15/16/17/18/19/20（15 个）
- **影响治理**：
  - `.claude/rules/dispatch-prompt-template.md` §2.3 改（MVP 行）
  - `.claude/rules/README.md` 索引去 runtime-evidence-location.md 行
  - PROGRESS.md / CLAUDE.md 🏁 段 / docs/tasks/README.md 表

---

## 自审四问

1. **递归完备性**：本 ADR 自己 supersede ADR-011 · 流程完整（proposed→accepted 走 PR-5）· 未来若回归 capture · 走新 ADR supersede 023 ✅
2. **反向场景**：若 user 想恢复 capture mandate · 新开 ADR-NNN supersede 023 · 流程明确 · 不破坏当前 done flip ✅
3. **边界适用性**：仅项目级 · 不影响全局 rule 15 · 不影响 Spike 4 样齐全 · 不影响代码侧 acceptance · 已捕证据保留 ✅
4. **YAGNI**：当前态 15 spec 实际卡 capture gate · 治理张力实际存在 · 不是投机移除 ✅
