# MVP-18 Phase D · Pre-Capture 就绪体检

> **定位**：本文件是 MVP-18 spec §I.5（Phase D runtime evidence 5+ 截图）+ §F.4（Runtime evidence checklist）的**前置体检**——主 agent（CLI）能程序化验证的 Phase D 代码侧前置已全跑，结论固化在此。
> **注意**：MVP-18 当前**无独立 CAPTURE-PLAYBOOK.md**（不同于 MVP-19/20 的独立 playbook 模式）。Arbiter 实跑时按 spec [`MVP-18 §E.D AI Pane feedback UX`](../../tasks/MVP-18-ai-aware-pane-linking.md#d-ai-pane-feedback-ux) + §I.5 + §F.4 直接执行；若 Arbiter 觉得需要 playbook 化，可参考 MVP-20 [`CAPTURE-PLAYBOOK.md`](../mvp-20/CAPTURE-PLAYBOOK.md) 模板新开 playbook PR。
> **它不是 capture 本身**：spec §I.5 的 5+ 张 GUI 截图 / §F.4 Playwright E2E trace / §H a11y / §G 5 项 P99 性能 / §L manual QA 三平台 设计上就是 Arbiter 本人在真实 GUI 前手动完成，CLI agent 无法替代，也不得编造。
> **用途**：Arbiter 真正坐下来跑 capture 窗口时，先读本文件 —— 已自动验证绿的部分**不必重复验**，把窗口聚焦在真正需要人的 GUI/E2E/性能/跨平台部分；并提前知悉 1 个性能 instrumentation gap。
>
> 体检执行：Claude Code 主 agent · 2026-05-20 · 镜像 MVP-19 #384 模式

---

## ✅ 已自动验证就绪（Arbiter 实跑可信任 · 不必重复验）

| Phase D 项                                            | 验证方式                                                 | 结果                                                                                                                                                                                        |
| ----------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **pane_links 单元测试**（§F core unit）               | `cargo test -p vibestation-core --lib 'pane_links::'`    | **28 passed · 0 failed · 0 ignored**（含 link create / duplicate / unlink / stale / cross workspace denied / invalid pane type · 覆盖 §E.B.1-B.7 / §E.F.1-F.3）                             |
| **parser_bridge 单元测试**（§F parser bridge unit）   | `cargo test -p vibestation-core --lib 'parser_bridge::'` | **8 passed · 0 failed**（含 ParsedIssue normalization · ParserBridgeError 边界 · §E.C.3-C.6 / §E.E.1-E.4 部分覆盖）                                                                         |
| **mvp18_contract IPC 契约测试**（§F app integration） | `cargo test -p vibestation-app --test mvp18_contract`    | **18 passed · 0 failed**（`pane:*` IPC 4 命令 + ACL permission/capability + 14 ts-rs binding 一致性 · §E.B.5 / §E.C.1 / §E.C.2）                                                            |
| **前端 pane-linking vitest**（§F frontend unit）      | `vitest run tests/panels/Terminal/pane-linking/`         | **9 files · 98 tests passed**（paneLinkApi · paneLinks-context · paneDrafts-store · paneDrafts-context · paneDraftComposer · paneLinkCreateMenu · paneFailurePreview · components · store） |
| **a11y 代码侧就位**（§E.H）                           | 源码审查                                                 | 见下表 · 5 控件全部就位                                                                                                                                                                     |
| **F.3 Fixture catalog 全 6 个就位**（§F.3）           | `ls crates/core/tests/fixtures/pane_link/`               | **6/6 全在**：`pane_failure_rustc.txt` / `pane_failure_vitest.txt` / `pane_failure_pytest.txt` / `pane_failure_ansi_json.txt` / `pane_failure_secret.txt` / `pane_failure_osc52.txt`        |

### a11y 代码侧就位明细（§E.H.1-H.5 前置 · Arbiter 实跑 a11y 截图前提已具备）

| 控件                   | a11y 实现                                                                                                                                                                                                                                                                           | 源                                               |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------ |
| **PaneLinkCreateMenu** | `role="dialog"` · `aria-modal="true"` · `aria-label="Link this pane's failures to an AI pane"` · `Escape` 关 · `role="list"` for candidates · 每条 candidate `aria-label="Link failures to {label}"` · 空状态 `role="status"` · icon `aria-hidden="true"`                           | `web/src/panels/Terminal/PaneLinkCreateMenu.tsx` |
| **PaneLinkChip**       | header chip `aria-label` 动态拼接（parent / child / kind / enabled / stale）· icon + badge `aria-hidden="true"`                                                                                                                                                                     | `web/src/panels/Terminal/PaneLinkChip.tsx`       |
| **FailureCallout**     | `role="alert"` · `aria-label="Build failure: {exitLabel} · {N} issue(s)"`（动态）· dismiss/view raw/insert 三按钮独立 `aria-label`（§E.H.3 焦点顺序）· icon `aria-hidden="true"`                                                                                                    | `web/src/panels/Terminal/FailureCallout.tsx`     |
| **PaneLinkErrorState** | `role="alert"` · `aria-label="Link {severity}: {errorMessage}"`（动态错误等级） · dismiss button `aria-label="Dismiss error"` · icon `aria-hidden="true"`                                                                                                                           | `web/src/panels/Terminal/PaneLinkErrorState.tsx` |
| **PaneDraftComposer**  | input `aria-label="Draft command input"` · send button `aria-label="Send draft command"` · merge preview `role="region"` + `aria-label="Merge preview: new content will be appended to existing draft"` · `Escape` 关 · append/cancel 按钮独立 `aria-label`（§E.D.5 merge preview） | `web/src/panels/Terminal/PaneDraftComposer.tsx`  |
| **reduced-motion CSS** | `@media (prefers-reduced-motion: reduce)` 3 处（terminal pane container line 1111 / 1151 / 1202 · 覆盖 §E.H.4 callout 不用 slide/scale）                                                                                                                                            | `web/src/panels/Terminal/*.css`                  |

> 含义：spec §E.H.1（全键盘 link popover）/ §E.H.2（aria-label）/ §E.H.3（focus order）/ §E.H.4（reduced-motion）/ §E.H.5（错误非颜色单独）的代码层支撑已全部存在。Arbiter 实跑时这些应能 PASS（仍需人工用读屏器 / reduced-motion 偏好实际验证视觉与朗读，代码侧不能替代真实 AT 验证）。

### Phase A/B/C 代码完成度（spec §I.5）

| Phase                | 范围                                                                                                                                         | 状态                    | PR                               |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- | -------------------------------- |
| Phase A backend      | `migrate_v8` §G schema · `pane_links` 核心 types/验证器/DAO 全 CRUD · `PaneLinkError` 10 变体 · `pane:*` IPC 4 命令 + ACL · 14 ts-rs binding | ✅ done                 | #344 / #345 / #346 / #347 / #348 |
| Phase B Wave-1/2/3   | draft composer · link UI · seam→binding（删 `paneLinkContract.ts` 临时 seam · store 单一真相 `PaneLinkView`）· failure callout               | ✅ done                 | #353 / #356 / #358-#363          |
| Phase C backend wire | `ParserBridgeError → PaneLinkError` 边界 map · `pane:trigger` / `pane:build-failed` 事件 · `pane:failure:preview_prompt` 命令                | ✅ done                 | #354 / #355 / #357               |
| Phase D evidence     | runtime evidence 5+ 截图 · Playwright E2E · a11y/perf · cross-platform smoke                                                                 | 🟡 **本文件**待 Arbiter | —                                |

---

## ⚠️ 关键 gap 预警 · 性能 instrumentation 未在代码内就位（共 1 项）

### gap-1 · `pane_links.rs` / `parser_bridge.rs` 无代码内 timing 输出

**坐实**：`grep 'Instant::now\|.elapsed()' crates/core/src/pane_links.rs crates/core/src/parser_bridge.rs` = **0 行**。link/unlink IPC 路径 · pane:build-failed event 路径 · parser pipeline 路径**没有代码内 timing 输出**。

**影响**：spec §E.G 5 项 P99 性能验收无代码内 instrument：

- G.1 child failure → AI callout P99 ≤ 200ms
- G.2 link/unlink IPC round-trip P99 ≤ 50ms
- G.3 100 links store update 不全 Pane 重渲染
- G.4 parser pipeline timeout 2s + fallback raw text + UI 200ms
- G.5 100 failures dedupe ≤ 5 callout + < 10MB

Arbiter 实跑只能靠 **DevTools Performance 面板手动观察** 或 **vitest bench**（spec §F 测试矩阵 `pnpm -C web exec vitest bench pane-linking`）填实测数字 · 无精确代码计时数字。

**需 Arbiter 决策**（二选一 · capture 窗口开始前定 · 同 MVP-19/20 gap 决策路径）：

- **(a) 纯 DevTools + vitest bench 手测**：实跑时 DevTools Performance + `vitest bench` 观察填 metrics · 标注「DevTools 手测 · 无代码 instrument」。最快，spec §F 测试矩阵本就接受此路径（§F 列出 `vitest bench pane-linking` 命令）。
- **(b) 先补轻量 timing instrument**：capture 前先加一个 PR 在 link/unlink/build-failed/parser 路径插 `Instant::now`/`tracing` span 输出 timing（让 metrics 有可复现代码数字）。更严谨，但多一个前置 PR + 改 backend 代码（非纯文档）。

---

## 🔴 Arbiter 实跑必须本人完成（CLI agent 无法替代 · 不得编造）

按 spec [`MVP-18 §I.5 + §F.4 + §E.H`](../../tasks/MVP-18-ai-aware-pane-linking.md)：

1. **fixture 准备** · 实际项目或自建脚本构造：
   - 同 workspace 的 AI parent pane + Runner child pane（§E.B.1-B.5）
   - 跨 workspace 的 AI parent + 其他 workspace child（用于截 §E.B.1 拒绝场景）
   - 至少 1 个会产生 build/test failure 的 child command（rustc / vitest / pytest 任一 · 复用 `crates/core/tests/fixtures/pane_link/` 6 类 fixture 之一即可）
   - 1 个会产生 parser fallback 的 unparsable output（§E.C.6 + §E.D.5 raw fallback 场景）

2. **§I.5 5+ 张 GUI 截图**（spec mandatory · MVP-18 spec 明确列出）：
   - 01-link-create.png · link 创建（PaneLinkCreateMenu 弹出 + 选择 child · 复用 §E.B 流程）
   - 02-child-badge.png · child Pane source badge（PaneLinkChip 在 child pane header · §D.2）
   - 03-failure-callout.png · build/test failure 触发后 AI Pane 的 FailureCallout（§D.3）
   - 04-raw-fallback.png · parser 失败后 raw text fallback（§D.5）
   - 05-cross-workspace-denied.png · 跨 workspace 建 link 时的 inline error（`PaneLinkError::CrossWorkspaceDenied`）

3. **§F.4 Runtime evidence checklist 完整跑**：
   - cargo test --workspace tail + exit code
   - pnpm lint + typecheck output + exit code
   - vitest run tests/panels/Terminal/pane-link output + exit code（subset 已在上方表绿 · Arbiter 自己再跑一次取 raw）
   - Playwright trace/screenshot for link create
   - Playwright trace/screenshot for failure callout
   - Manual dev mode screenshot for raw fallback
   - Manual dev mode screenshot for cross workspace denial

4. **Playwright E2E**（§F 测试矩阵）：`pnpm -C web exec playwright test pane-linking.spec.ts` · 覆盖 link 建立 / failure callout / insert / dismiss / unlink / fallback。CLI agent 无 GUI · 实际 Playwright run 必须真窗口环境 · Arbiter 在本地或 Docker xvfb 跑。

5. **性能 metrics 实测**（按上方 gap 决策的 (a)/(b) 路径）填 spec §E.G 5 项数字。

6. **manual QA macOS / Linux / Windows**（§F 测试矩阵 + §L）· keyboard 与 window lifecycle smoke。CLI agent 无 Linux/Windows 环境 · 必须 Arbiter 在三平台或 defer Linux/Windows 到统一 cross-platform 窗口。

7. **commit + PR + R1-R5**（按 `.claude/rules/runtime-evidence-location.md`）：
   - R1 位置 `docs/runtime-evidence/mvp-18/`
   - R2 进 git
   - R3 顺序前缀 `01-` `02-` `03-` `04-` `05-`
   - R4 体积 ≤ 10 MB（推荐 ≤ 3 MB）
   - R5 PR body Test Plan 必含「Runtime 证据已提交到 `docs/runtime-evidence/mvp-18/` · 含 5+ 张截图/录屏」

### §E.A.4 对外文案禁区提醒

> spec §E.A.4：所有公开文案禁区保持不变；MVP-18 只允许在内部 docs/tasks、ADR、implementation plan 中讨论具体能力。

含义：截图本身可以提交（runtime-evidence 不是对外文案）· 但 PR title / PR body 描述 / commit message **不得出现** `AI-Aware Pane` / `Mission Control` / `AI session aware`（CLAUDE.md §禁区 + ADR-018 决议 4 保留禁区）。可用「pane linking」「Pane 联动」「pane:link IPC」等中性技术词。

---

## 结论

Phase D 验收项中：

- **代码侧可验证的全过**（pane_links 28 / parser_bridge 8 / mvp18_contract 18 / 前端 pane-linking subset 98 / a11y 5 控件代码就位 / reduced-motion CSS 3 处 / F.3 fixture 6/6 全在 / Phase A-C 代码全 done）✅
- **1 项性能 instrumentation gap**（同 MVP-19/20 模式：pane_links + parser_bridge 无 timing instrument）· 待 Arbiter 选 (a)/(b)
- **纯人工 GUI capture（5+ 张截图 / Playwright E2E / 性能 manual / 三平台 QA）待 Arbiter 窗口**——这部分 CLI agent 结构性无法代劳，本文件如实声明而非编造，符合 `~/.claude/rules/always/07-verification-discipline`

MVP-18 spec 维持 `in-progress`（最终 Phase D capture 未完 · 多 phase 任务 done gate 在 Phase D 5+ 证据齐全 + §E.G/H 性能与 a11y 实跑后才翻）。

**关联**：spec [`docs/tasks/MVP-18-ai-aware-pane-linking.md`](../../tasks/MVP-18-ai-aware-pane-linking.md) §E（全 A-I）/ §F（测试矩阵 + F.3 fixture + F.4 checklist）/ §G（数据模型）/ §I.5（Phase D evidence 要求）· `.claude/rules/runtime-evidence-location.md` R1-R5 · 模板镜像 [MVP-19 PRE-CAPTURE-READINESS](../mvp-19/PRE-CAPTURE-READINESS.md) PR #384 · [MVP-20 PRE-CAPTURE-READINESS](../mvp-20/PRE-CAPTURE-READINESS.md)
