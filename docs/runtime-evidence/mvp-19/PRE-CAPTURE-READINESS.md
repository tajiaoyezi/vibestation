# MVP-19 Phase E · Pre-Capture 就绪体检

> **定位**：本文件是 [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) 的**前置体检**——主 agent（CLI）能程序化验证的 Phase E 代码侧前置已全跑，结论固化在此。
> **它不是 capture 本身**：playbook 要求的 15 张 GUI 截图 / 30s 录屏 / 人眼 pass 判据 / DevTools 性能实测 / Linux 跨平台 smoke，**设计上就是 Arbiter 本人在真实 GUI 前手动完成**（playbook 标题=「Arbiter ~30-45 min 收口」），CLI agent 无法替代，也不得编造。
> **用途**：Arbiter 真正坐下来跑 capture 窗口时，先读本文件 —— 已自动验证绿的部分**不必重复验**，把 30-45 min 聚焦在真正需要人的 GUI/录屏/性能/跨平台部分；并提前知悉 1 个性能 instrumentation gap。
>
> 体检执行：Claude Code 主 agent · 2026-05-17 · session 33

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| Phase E 项                                                | 验证方式                                                                | 结果                                                                                                                            |
| --------------------------------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| **E.1 脱敏红线 backend**                                  | `cargo test -p vibestation-core --lib 'session_redaction::'`            | **30 passed · 0 failed**（api_key anthropic/openai/github classic+PAT · bearer · ANSI CSI/OSC strip · negative too-short 边界） |
| E.1 脱敏 sanitize 层                                      | `cargo test` sanitize 模块                                              | 8 tests 在 851 全绿内（`<REDACTED>` / `<REDACTED_PATH>` 逻辑）                                                                  |
| E.1 telemetry PII                                         | `cargo test -p vibestation-core --test telemetry_pii_test`              | **6 passed · 0 failed**                                                                                                         |
| **core lib 整体**（含 session_lifecycle/dao/service/ipc） | `cargo test -p vibestation-core --lib`                                  | **851 passed · 0 failed · 0 ignored**（无跨平台 ignore 技术债残留在 session 链路）                                              |
| **前端 MVP-19 组件**（#377/#379 产物当前态）              | `vitest run` sessionBadge + sessionDetail + sessionApi + sessions store | **4 files · 78 tests passed**                                                                                                   |
| **a11y 代码侧就位**（playbook 2.5 / §E8）                 | 源码审查                                                                | 见下表 · 全部就位                                                                                                               |

### a11y 代码侧就位明细（playbook 2.5 前置 · Arbiter 实跑 14/15 截图前提已具备）

| 控件                | a11y 实现                                                                                                                                              | 源                                               |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------ |
| GitLog SessionBadge | `aria-label` 携带 title+status+confidence（颜色非唯一指示）· `pending`=◌ icon / `stale`=⚠ icon（颜色非唯一状态表达 · §E8.3）· `onKeyDown` 键盘 handler | `web/src/panels/GitLog/SessionBadge.tsx`         |
| SessionDetailView   | `role="region"` + `role="alert"` + `aria-live="polite"` · aria=17/role=6 · 关闭/CLI 标签 aria-label                                                    | `web/src/panels/Sessions/SessionDetailView.tsx`  |
| SessionUnbindModal  | `role="dialog"` · mount `dialogRef.focus()` · `Escape` 关闭 · `Tab` focus trap（first/last + shiftKey 双向循环）                                       | `web/src/panels/Sessions/SessionUnbindModal.tsx` |
| SessionRebindModal  | 同 Unbind（role=dialog + focus trap + Esc）                                                                                                            | `web/src/panels/Sessions/SessionRebindModal.tsx` |
| reduced-motion      | `@media (prefers-reduced-motion: reduce) { .vs-session-badge { transition: none } }`（HC-4 · #377 reviewer-fix `69a5279` 已交付）                      | `web/src/panels/GitLog/sessionBadge.css`         |

> 含义：playbook 2.5 的 `14-a11y-keyboard-modal` / `15-reduced-motion` 截图，其代码层支撑已全部存在。Arbiter 实跑时这些应能 PASS（仍需人工用读屏器 / reduced-motion 偏好实际验证视觉与朗读，代码侧不能替代真实 AT 验证）。

---

## ⚠️ 关键 gap 预警 · 性能 instrumentation 未在代码内就位

**坐实**：`grep 'Instant::now\|.elapsed()' crates/core/src/{session_lifecycle,session_service,session_dao}.rs` = **0 行**。session 绑定计算 / 详情查询路径**没有代码内 timing 输出**。

**影响**：playbook §3 要求测「绑定计算 < 20ms（500 commit）· 详情首次打开 < 200ms（缓存命中 < 80ms）」。无代码内 instrument → Arbiter 实跑只能靠 **DevTools Performance 面板手动观察**填 `metrics-mvp-19.md`，无精确代码计时数字。playbook §3 原文「若 Phase E 仪表化已就位」已暗示此可能性。

**需 Arbiter 决策**（二选一 · capture 窗口开始前定）：

- **(a) 纯 DevTools 手测**：实跑时 DevTools Performance + React 渲染次数观察，metrics 填观测值 + 标注「DevTools 手测 · 无代码 instrument」。最快，playbook §3 本就接受此路径。
- **(b) 先补轻量 timing instrument**：capture 前先加一个 PR 在绑定/详情查询路径插 `Instant::now`/`tracing` span 输出 timing（让 metrics 有可复现代码数字）。更严谨，但多一个前置 PR + 改 backend 代码（非纯文档）。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 [`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) §2-§5：

1. `pnpm tauri:dev` 启动真实窗口（dev-boot 健康 session 33 已验 · 见 PROGRESS）
2. 准备 fixture（≥3 commit + 已绑定 session + pending/low-conf · playbook §1.1）
3. **15 张 GUI 截图**：01-06 徽章 6 态 · 07-08 详情视图 · 09-10 解绑/改绑 superseded modal · 11-13 脱敏示例+fail-closed · 14-15 a11y 焦点+reduced-motion
4. **30s 录屏**：键盘导航 + 模态流
5. 人眼判 playbook 各步 pass 判据（"徽章可见 + hover tooltip 含 session title + 点击后详情 commit 列表首项匹配" 类，非代码可断言）
6. **性能实测**填 `metrics-mvp-19.md`（按上方 gap 决策的路径）
7. **Linux 跨平台 lifecycle smoke**（CLI agent 无 Linux 环境 · macOS 侧 cargo lifecycle 测试已在 851 全绿内 · Linux 侧需 Arbiter 在 Ubuntu 跑或 defer 至 MVP-04 Phase D Ubuntu runtime 窗口统一验）
8. commit + PR + R1-R5（playbook §5）

---

## 结论

Phase E 五项中，**代码侧可验证的 4 项已就绪并固化证据**（脱敏红线 / lifecycle+core 全绿 / 前端组件测试 / a11y 代码就位）；**1 项性能 instrumentation 有 gap**（待 Arbiter 选 (a)/(b)）；**纯人工 GUI capture（截图/录屏/人眼/Linux）待 Arbiter 窗口**——这部分 CLI agent 结构性无法代劳，本文件如实声明而非编造，符合 `~/.claude/rules/always/07-verification-discipline`。

MVP-19 spec 维持 `in-progress`（最终 capture phase 未完 · 多 phase 任务 done gate 在 Phase E 证据齐全后才翻）。

**关联**：[`CAPTURE-PLAYBOOK.md`](./CAPTURE-PLAYBOOK.md) · [`metrics-mvp-19.md`](./metrics-mvp-19.md) · spec [`docs/tasks/MVP-19-session-commit-binding.md`](../../tasks/MVP-19-session-commit-binding.md) §I.5 · `.claude/rules/runtime-evidence-location.md`
