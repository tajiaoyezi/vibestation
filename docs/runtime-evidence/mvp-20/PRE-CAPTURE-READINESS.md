# MVP-20 Phase D/E · Pre-Capture 就绪体检

> **定位**：本文件是 [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) 的**前置体检**——主 agent（CLI）能程序化验证的 Phase D/E 代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：playbook §1 的 15 张 GUI 截图 / §2 30s 录屏 / 人眼 pass 判据 / §3 Criterion 性能 / §4 Linux 跨平台 smoke 设计上就是 Arbiter 本人在真实 GUI 前手动完成（playbook 标题=「Arbiter ~30-45 min 收口」），CLI agent 无法替代，也不得编造。
> **用途**：Arbiter 真正坐下来跑 capture 窗口时，先读本文件 —— 已自动验证绿的部分**不必重复验**，把 30-45 min 聚焦在真正需要人的 GUI/录屏/性能/跨平台部分；并提前知悉 2 个性能 instrumentation gap。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| Phase D/E 项                                             | 验证方式                                                                                                                               | 结果                                                                                                                                                                                                       |
| -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **rollback_ops 单元测试**                                | `cargo test -p vibestation-core --lib 'rollback_ops::'`                                                                                | **38 passed · 0 failed · 0 ignored**（含 `RollbackStatusKind` round-trip / `detect_crash_recovery` 7 分支 / abort 边界 2 项 / build_revert_plan 过滤+newest-first）                                        |
| **mvp20_rollback_contract IPC 契约测试**                 | `cargo test -p vibestation-app --test mvp20_rollback_contract`                                                                         | **3 passed · 0 failed**（4 IPC 散参 camelCase + ts-rs binding 一致性）                                                                                                                                     |
| **core lib 整体**（含 session/diff/git/rollback 全链路） | `cargo test -p vibestation-core --lib`                                                                                                 | **896+ passed · 0 failed**（session 33 PR #394 末次验过 · #394 commit `b50087e`）                                                                                                                          |
| **前端 rollback 组件 vitest**                            | `vitest run tests/panels/SessionDetail/ tests/panels/Sessions/SessionDetailView.rollback.test.tsx tests/lib/rollback-recovery.test.ts` | **7 files · 52 tests passed**（RollbackPreviewModal / RollbackConfirmDialog / RollbackProgressBanner / RollbackConflictView / rollbackApi / rollback-recovery state machine / SessionDetailView.rollback） |
| **a11y 代码侧就位**（playbook §1.14 / §E.6）             | 源码审查                                                                                                                               | 见下表 · 5 控件全部就位                                                                                                                                                                                    |

### a11y 代码侧就位明细（playbook §1.14 前置 · Arbiter 实跑 a11y 截图前提已具备）

| 控件                                                     | a11y 实现                                                                                                                                                                                                                                                                                                                         | 源                                                        |
| -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| **RollbackPreviewModal**                                 | `role="dialog"` · `aria-modal="true"` · `aria-labelledby="vs-rollback-preview-title"` · mount `dialogRef.focus()` · `Escape` 关 · Tab focus trap（first/last 双向循环）· `role="list"` + `aria-label="Commits to revert"` · 每条 commit checkbox `aria-label="Include commit {sha} in rollback"` · `aria-expanded` for files 展开 | `web/src/panels/SessionDetail/RollbackPreviewModal.tsx`   |
| **RollbackConfirmDialog**                                | `role="dialog"` · `aria-modal="true"` · `aria-labelledby="vs-rollback-confirm-title"` · mount `inputRef.focus()` · `Escape` 关 · Tab focus trap · 输入框 `aria-label="输入 session ID 以确认回滚"` · 提交按钮 `aria-label="执行回滚（{N} 个 commit）"`（动态 commit 数）                                                          | `web/src/panels/SessionDetail/RollbackConfirmDialog.tsx`  |
| **RollbackProgressBanner**                               | `role="status"` · `aria-live="polite"` · `aria-label="回滚进度：{done}/{total} 完成"`（动态实时）· 取消按钮 `aria-label="取消回滚"`                                                                                                                                                                                               | `web/src/panels/SessionDetail/RollbackProgressBanner.tsx` |
| **ConflictBanner**（MVP-16 复用 · operation="rollback"） | `ConflictOperation` enum 含 `"rollback"` 变体 + `operationCopy.rollback = "Reverting"` · banner level `role="status"` · abort dialog `role="dialog"` + `aria-modal="true"` + `aria-label="Confirm abort"` · icon `aria-hidden="true"`                                                                                             | `web/src/components/ConflictBanner/ConflictBanner.tsx`    |
| **RollbackRecoveryBanner**（crash recovery 镜像 MVP-16） | `role="alert"` · `aria-label="检测到未完成的回滚 · Session #{sessionId}"` · icon `aria-hidden="true"` · error 区 `role="alert"`                                                                                                                                                                                                   | `web/src/panels/SessionDetail/RollbackRecoveryBanner.tsx` |
| **RollbackConflictView**                                 | error 区 `role="alert"`（复用 ConflictBanner + ThreeWayDiffView · 视觉 a11y 主要由复用组件承担）                                                                                                                                                                                                                                  | `web/src/panels/SessionDetail/RollbackConflictView.tsx`   |
| **reduced-motion CSS**                                   | `@media (prefers-reduced-motion: reduce)` 在 `rollback.css` line 347                                                                                                                                                                                                                                                              | `web/src/panels/SessionDetail/rollback.css`               |

> 含义：playbook §1.4-1.5（二次确认 dialog）/ §1.6（progress banner）/ §1.9（ConflictBanner）/ §1.14（a11y 焦点流）/ §1.15（reduced-motion）的代码层支撑已全部存在。Arbiter 实跑时这些应能 PASS（仍需人工用读屏器 / reduced-motion 偏好实际验证视觉与朗读，代码侧不能替代真实 AT 验证）。

### Phase A/B/C/D 代码完成度（spec §I）

| Phase   | 范围                                                                                                                     | 状态    | PR                                  |
| ------- | ------------------------------------------------------------------------------------------------------------------------ | ------- | ----------------------------------- |
| Phase A | revert plan/IPC/binding · build_revert_plan 过滤+newest-first · 4 IPC 散参 + migrate_v10 + 5 ts-rs binding camelCase     | ✅ done | #385 / #386 / #387 / #388           |
| Phase B | 前端 UI · 预览 modal / 二次确认 / 进度 banner · 含 reviewer-fix 测试隔离回归（cleanup 泄漏污染 DiffLine）                | ✅ done | #386（含 reviewer-fix）             |
| Phase C | 后端 resume（`rollback_execute_with_progress` conflict_paused→resume）+ 前端 wire（ConflictBanner+RollbackConflictView） | ✅ done | #391 / #392 / #390 CAPTURE-PLAYBOOK |
| Phase D | `RollbackStatusKind` union 保真 + 全局 `detect_crash_recovery` + emit + `RollbackRecoveryBanner` + abort 边界 · TDD 全程 | ✅ done | #394                                |

---

## ⚠️ 关键 gap 预警 · 性能 instrumentation 未在代码内就位（共 2 项）

### gap-1 · `rollback_ops.rs` 无代码内 timing 输出

**坐实**：`grep 'Instant::now\|.elapsed()' crates/core/src/rollback_ops.rs` = **0 行**。`rollback_execute_with_progress` / `build_revert_plan` / `run_revert_loop` 路径**没有代码内 timing 输出**。

**影响**：playbook §3 + spec §F.1 要求测「单 commit revert P99 < 100ms / 5 commit P99 < 500ms / 20 commit P99 < 2s」。无代码内 instrument → Arbiter 实跑只能靠 **DevTools Performance 面板手动观察** 或 **`time cargo test`** 粗测填 `metrics-mvp-20.md`，无精确代码计时数字。

### gap-2 · `crates/core/benches/rollback.rs` Criterion bench file 不存在

**坐实**：`ls crates/core/benches/` 现存 8 个 bench（branch / diff / git_ops / git_status / git_sync / pane_layout / rebase / workspace_query）· **无 `rollback.rs` 或 `rollback_bench.rs`**。playbook §3 指令 `cargo bench -p vibestation-core --bench rollback -- 5-commit 20-commit` 会 **找不到 bench target 而 fail**。

**影响**：spec §F.1 Criterion 三档 P99 验收（单/5/20 commit）无 bench harness 可跑 · `metrics-mvp-20.md` §F.1.1-1.3 三档表只能手测填或留空。

---

**需 Arbiter 决策**（三选一 · capture 窗口开始前定 · 同 MVP-19 gap 决策路径）：

- **(a) 纯手测路径**：实跑时用 `time cargo test mvp20_rollback_contract::full_revert_5commit` 类粗测 + DevTools Performance + Criterion 跳过 · metrics 填观测值 + 标注「粗测 · 无代码 instrument · 无 Criterion bench」。最快，spec §F.1 数字是 ship gate 但 v1.0 alpha 可接受粗测路径（同 MVP-19 #384 已建立先例）。
- **(b) 先补 Criterion bench file（轻量）**：capture 前先加一个 PR 在 `crates/core/benches/rollback.rs` 写 Criterion harness（参考既有 `rebase_bench.rs` / `git_ops_bench.rs` 模板）· 跑出 3 档真实 P99。无需改 `rollback_ops.rs` 代码 · 仅 benches/ 新增。中等成本（1-2h 写+跑+verify · 1 PR）· 但 metrics 真实可复现。
- **(c) 双管齐下**：先 (b) 加 Criterion bench · 再 (a) 跑 capture（推荐 v1.0 GA 之前真要 ship 时）。最严谨 · 但 ship 时间表上多 1 个 PR。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) §0-§5：

1. **fixture 准备**（playbook §0.5）· ≥ 3 commit AI session + 1 个低置信候选 + 1 个会冲突的 revert 场景。CLI agent 不能在没真实 workspace 时构造 · 必须 Arbiter 本人用真实项目或自建脚本（spec 附录 C `create_5commit_session_fixture` 伪代码）。
2. **15 张 GUI 截图**（playbook §1.1-1.15）· 覆盖 §E.1（按钮初态 + 预览 + 低置信警告）· §E.2（二次确认 dialog 错/正 2 张）· §E.1.7（progress banner）· §E.1.9（完成徽章）· §E.1.11+§E.5（Git Log 历史保留 · `git log --oneline` 含 revert commit + `[AI session rollback: {id}]` 后缀验证）· §E.4（ConflictBanner + ThreeWayDiffView · Continue 续跑）· §E.3（Abort flow + git log 验 HEAD 干净回退）· §E.2.4（DirtyWorkingTree 跳转）· §E.6（a11y 焦点流 + reduced-motion）。
3. **30s 录屏**（playbook §2）· 键盘 + 模态 + abort 流。
4. **性能 metrics 实测填 `metrics-mvp-20.md`**（按上方 gap 决策的 (a)/(b)/(c) 路径）。
5. **Linux 跨平台 smoke**（playbook §4 · spec §L.1 6 行）· git revert 顺序 / file lock / cleanup_state / 路径大小写 / SQLite WAL / 冲突解决。CLI agent 无 Linux 环境 · 必须 Arbiter 在 Ubuntu VM / GitHub Actions runner 实测（或 defer 至 MVP-04 Phase D Ubuntu runtime 窗口统一验）。
6. **commit + PR + R1-R5**（playbook §5）。

### Invariant I3/I4 关键提醒（playbook §I3 §I4）

- 每张回滚完成截图必伴随 `git log --oneline` + `git diff <pre-session-sha>` 验证为空的终端输出 · 证明是 `git revert` 而非 `reset --hard`（§E.2.3 严禁 reset + §E.5.2）。
- 截 ConflictBanner 时必须同时截 ThreeWayDiffView 确认是 `web/src/panels/Diff/3way/` 同款组件（验证 §E.4 真复用 MVP-16 · 而非自建 conflict UI）。

---

## 结论

Phase D/E 验收项中：

- **代码侧可验证的全过**（rollback_ops 38 / mvp20 contract 3 / 前端 rollback subset 52 / a11y 5 控件代码就位 / reduced-motion CSS / Phase A-D 代码全 done）✅
- **2 项性能 instrumentation gap**：(1) `rollback_ops.rs` 无 timing instrument（同 MVP-19 模式）· (2) `benches/rollback.rs` Criterion bench file 不存在（**MVP-19 #384 未碰到 · MVP-20 新发现**）· 待 Arbiter 选 (a)/(b)/(c) 路径
- **纯人工 GUI capture（15 张截图 / 30s 录屏 / 人眼 / Linux smoke）待 Arbiter 窗口**——这部分 CLI agent 结构性无法代劳，本文件如实声明而非编造，符合 `~/.claude/rules/always/07-verification-discipline`

MVP-20 spec 维持 `in-progress`（最终 capture phase 未完 · 多 phase 任务 done gate 在 Phase E 证据齐全后才翻）。

**关联**：[`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) · [`metrics-mvp-20.md`](./metrics-mvp-20.md) · spec [`docs/tasks/MVP-20-ai-one-click-rollback.md`](../../tasks/MVP-20-ai-one-click-rollback.md) §E/§F.1/§I/§L.1/§M · `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384
