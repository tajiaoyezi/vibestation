# Tasks · 任务索引

> 本目录存放所有**可执行任务的详细规格（task spec）**：SPIKE（技术验证）/ MVP（MVP 功能）/ BUG（缺陷）/ FEAT（v0.2+ 功能）。
> 每个 task spec 是**一个 PR 的验收依据**——评审者按 spec 的 Acceptance 逐项对照 diff。

---

## 📂 命名规范

```
<TYPE>-<编号>-<英文 slug>.md
```

| TYPE    | 用途                                                         | 编号规则                         |
| ------- | ------------------------------------------------------------ | -------------------------------- |
| `SPIKE` | 技术验证 / benchmark / 风险消除                              | 按 Spike 天数顺序 `SPIKE-01..06` |
| `MVP`   | MVP B 折中方案范围内的功能（`implementation-plan.md §10.1`） | 按模块顺序 `MVP-01..20`          |
| `BUG`   | 缺陷修复                                                     | 按发现顺序 `BUG-001..`           |
| `FEAT`  | v0.2+ 新功能                                                 | 按路线图顺序 `FEAT-01..`         |

**示例**：`SPIKE-01-tauri-three-platform-boot.md`、`MVP-03-terminal-multi-tab.md`、`BUG-001-pty-resize-crash.md`

**slug 要求**：小写英文 + 连字符，3-5 个词，语义清晰。

---

## 🔄 状态流转

```
      (新建)          (spec 过独立评审)    (认领)         (Acceptance 全过)
draft ────────► ready ──────────────► in-progress ────────────► done
                  ▲ │                      ▲ │
                  │ │ (外部阻塞)           │ │ (外部阻塞)
                  │ └────► blocked ◄──────┘ │
                  │   (blocked_by 填原因)   │
                  └──── (阻塞解除恢复原状态)─┘
```

| 状态          | 含义                                 | 进入条件                                                                                                              | 出口条件                                                 |
| ------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `draft`       | 草稿，字段未填完 / 未评审            | 新建                                                                                                                  | 作者自审 + 独立评审通过 → `ready`                        |
| `ready`       | 可被认领，字段完整，Acceptance 明确  | spec PR 评审通过（同 PR 最后一个 commit 改 `status: ready`）                                                          | 某 agent 认领 → `in-progress` · 或遇外部阻塞 → `blocked` |
| `in-progress` | 已被认领并实施中                     | 实施 PR 首个 commit 改 `owner` + `status: in-progress`                                                                | Acceptance 全过 → `done` · 或遇外部阻塞 → `blocked`      |
| `blocked`     | 被依赖项或外部资源阻塞               | 从 `ready` 或 `in-progress` 进入；必填 `blocked_by`（上游 task-id 或外部资源名）；可选 `blocked_note`（人类可读原因） | 阻塞解除 → **恢复到进入前的状态**（见下方规则）          |
| `done`        | PR 已 merge 到 main，Acceptance 全过 | 实施 PR merge 前最后一个 commit 改 `reviewer` + `status: done` → merge                                                | 终态（不删文件，作为历史留档）                           |

**`blocked` 状态恢复规则**（解除阻塞时执行）：

- **进入 `blocked` 时必填 `blocked_from` 字段**（显式记录进入前的状态：`ready` 或 `in-progress`），避免靠"隐含约定"猜测回退目标
- 解除阻塞时**机械恢复**到 `blocked_from` 记录的值：
  - 从 `in-progress` 进入 `blocked` → 解除后 `status = in-progress`，`owner` **保留**，原 branch / open PR **不动**（agent 继续原工作）
  - 从 `ready` 进入 `blocked` → 解除后 `status = ready`，`owner` 保持空（等待新 agent 认领）
- 解除时必做：清空 `blocked_by` / `blocked_from` / `blocked_note` 三个字段

**其他规则**：

- 状态字段**必须**与 `PROGRESS.md`、PR description 一致
- `done` 状态的 task 文件**不删除**，作为历史留档（Phase 3 可选归档到 `docs/session-history/`）

---

## 📋 字段说明（common schema，所有 TYPE 共享）

| 字段           | 类型   | 必填                              | 说明                                                                                                                                                                                                                                                                                                                                                                                        |
| -------------- | ------ | --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id`           | string | ✅                                | `SPIKE-01` / `MVP-03` / `BUG-001`                                                                                                                                                                                                                                                                                                                                                           |
| `type`         | enum   | ✅                                | `spike` / `mvp` / `bug` / `feat`                                                                                                                                                                                                                                                                                                                                                            |
| `title`        | string | ✅                                | 中文简述（≤ 30 字）                                                                                                                                                                                                                                                                                                                                                                         |
| `status`       | enum   | ✅                                | 见上表                                                                                                                                                                                                                                                                                                                                                                                      |
| `owner`        | string | ⛔ 留空 = 未认领                  | 认领者标识（PR `Implemented by` 填写的 agent/人类 ID）                                                                                                                                                                                                                                                                                                                                      |
| `phase`        | string | ✅                                | `W0-D1` / `W1` / `W5` / `v0.2`                                                                                                                                                                                                                                                                                                                                                              |
| `depends_on`   | list   | ✅（可空 `[]`）                   | 依赖的 task id                                                                                                                                                                                                                                                                                                                                                                              |
| `blocks`       | list   | ✅（可空 `[]`）                   | 该 task 完成后解锁的 task id                                                                                                                                                                                                                                                                                                                                                                |
| `blocked_by`   | list   | ⛔（仅 `status: blocked` 时必填） | 阻塞源：task-id（如 `["SPIKE-02"]`）或外部资源（如 `["apple-dev-program-approval"]`）                                                                                                                                                                                                                                                                                                       |
| `blocked_from` | enum   | ⛔（仅 `status: blocked` 时必填） | 进入 `blocked` 前的状态：`ready` / `in-progress`；解除阻塞时自动恢复到此状态                                                                                                                                                                                                                                                                                                                |
| `blocked_note` | string | ⛔ 可选                           | 人类可读的阻塞原因说明（1-2 句）                                                                                                                                                                                                                                                                                                                                                            |
| `estimate`     | string | ✅                                | `0.5d` / `1d` / `3d`                                                                                                                                                                                                                                                                                                                                                                        |
| `plan_ref`     | string | ✅                                | `implementation-plan.md` 章节 `§3.1.1`                                                                                                                                                                                                                                                                                                                                                      |
| `risk_ref`     | string | ⛔ 可选                           | `R1` / `R12` / `R27` 等 `implementation-plan §9` 风险 ID                                                                                                                                                                                                                                                                                                                                    |
| `reviewer`     | string | ⛔ 默认填 PR review 时            | v2-D.1 语义 = 执行 review 的 **agent 名**（`Claude Code` / `Kimi` / `Codex CLI` / `OpenCode` 等）· 非 Arbiter（Arbiter approval 走 PR body trailer · 见 [ADR-012](../adr/ADR-012-v2d1-arbiter-approval-simplification.md)）· 单人项目下 self-review 合法 · 但 reviewer 字段必须填 agent 名（即使与 owner 同一 agent）· session 13 audit M-3 统一 · 旧 task 写 `User` 已回填为 `Claude Code` |

**YAML frontmatter 示例**：详见 [`_template.md`](./_template.md)。

---

## 📝 正文 Section（按 TYPE 差异化）

### SPIKE 必填

- **目标**（Goal）：一句话描述要验证什么
- **背景**（Context）：为什么现在做这个 Spike
- **通过标准**（Pass Criteria）：**可量化**的判据（P99 延迟 / 冷启动时间 / 错误率 …）
- **失败信号**（Fail Signals）：触发 fallback 的具体条件
- **Fallback 方案**：通过 / 失败后的分支决策（对应 `CLAUDE.md` 决策表 B 栏）
- **产出**（Deliverables）：benchmark 数据表 / 录屏 / ADR 草稿 / 代码 proof
- **依赖资源**：硬件 / 账号 / 数据集（如 linux kernel 仓库）

### MVP / FEAT 必填

- **功能范围**（Scope）：什么做，什么不做
- **UI 引用**（UI Reference）：`design/directions/1-calm-studio.html` 对应区块 / 截图
- **Acceptance**（验收清单）：勾选式，evaluator 按条对照
- **测试策略**：单元 / 集成 / E2E 覆盖哪些路径
- **数据模型变更**（如有）：rusqlite schema / rusqlite table 变化

### BUG 必填

- **复现步骤**（Reproduction Steps）
- **期望行为** vs **实际行为**
- **根因分析**（Root Cause）
- **修复验证**（Fix Verification）：回归测试

> BUG 和 FEAT 模板在真实需要时 Phase 3 补；当前 Phase 2 只定 SPIKE + MVP。

---

## 🗂 当前索引

### SPIKE（W0 周，硬依赖 Spike W0 启动）

| ID                                                            | 标题                                                                   | 状态                                                                                                                                                   | 估时 | 依赖                             | 风险             |
| ------------------------------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ---- | -------------------------------- | ---------------- |
| [SPIKE-01](./SPIKE-01-tauri-three-platform-boot.md)           | Tauri 2 三平台空壳启动（mac + Ubuntu Wayland + X11）                   | **done**（Phase A macOS PR #20 · Phase B Ubuntu PR #137 · session 19 · X11 108ms / Wayland 107ms / 30 stable · ADR-006 升级 Ubuntu validated PR #138） | 1d   | —                                | R12              |
| [SPIKE-02](./SPIKE-02-tauri-hard-pass-matrix.md)              | Tauri 硬通过矩阵 + Electron fallback（若 D1 失败）                     | **done**（Phase A macOS PR #22 · Phase B Ubuntu PR #137 · v0.1 GA 双平台路径解锁）                                                                     | 1d   | SPIKE-01                         | **R12 CRITICAL** |
| [SPIKE-03](./SPIKE-03-git2-gix-read-benchmark.md)             | git2 读 log + gix 对比 benchmark（linux kernel）                       | done                                                                                                                                                   | 1d   | SPIKE-02                         | R3               |
| [SPIKE-04](./SPIKE-04-storage-benchmark.md)                   | redb 2 vs rusqlite benchmark + git2 写 commit                          | done                                                                                                                                                   | 1d   | SPIKE-02                         | R27              |
| [SPIKE-04.5](./SPIKE-04.5-rusqlite-safety-verification.md)    | rusqlite 数据安全 B.1-5 + A.3 性能补测                                 | done                                                                                                                                                   | 1d   | SPIKE-04                         | R27              |
| [SPIKE-05](./SPIKE-05-pty-multi-tab.md)                       | portable-pty 单读 + mpsc + xterm 4-Tab 压测                            | done                                                                                                                                                   | 1d   | SPIKE-02                         | —                |
| [SPIKE-05.5](./SPIKE-05.5-pty-visible-throughput-fallback.md) | PTY visible throughput + per-session fallback 对照                     | done                                                                                                                                                   | 1d   | SPIKE-05                         | —                |
| [SPIKE-06](./SPIKE-06-cli-protocol-and-codesign.md)           | Claude CLI / Codex CLI 实机 + macOS Dev Program                        | blocked（§A done · PR #38 harness + PR #71 36 样本 · R1 保留 · §B 等 Apple Dev Program 申请 · audit H2 @ 2026-04-21）                                  | 1d   | SPIKE-05 · phase-4-infra-landing | R1               |
| [SPIKE-07](./SPIKE-07-cli-protocol-parser.md)                 | CLI 输出协议 parser 验证（**占位** · v1.0-pre · R1 降级前置）          | **draft**（session 31 详化完成 100% · 611 行 · 43 checkbox · OpenCode 详化 PR #311 · 等 Arbiter approve flip ready）                                   | 3d   | SPIKE-06                         | R1               |
| [SPIKE-08](./SPIKE-08-e2e-and-contract-harness.md)            | E2E + IPC contract 双层防御 harness 选型 + POC（H2 后 rule 15 制度化） | done                                                                                                                                                   | 2d   | MVP-02                           | —                |

### MVP（v0.1 范围 · B 折中方案）

| ID                                                 | 标题                                                           | 状态                                                                                                                                                                                                                                                                                                                                                                                                                     | 估时 | 依赖                 |
| -------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---- | -------------------- |
| [MVP-01](./MVP-01-tauri-app-shell.md)              | Tauri 应用骨架 + 启动流程 + 基础崩溃恢复                       | **done**（Phase A + B + Phase C Ubuntu installer · PR #139 deb 5.5MB / AppImage 78MB · §F 双平台打包 acceptance 全勾）                                                                                                                                                                                                                                                                                                   | 5d   | SPIKE-02             |
| [MVP-02](./MVP-02-workspace-management.md)         | Workspace 管理 + 项目识别 + 多 workspace 并存                  | done                                                                                                                                                                                                                                                                                                                                                                                                                     | 4d   | MVP-01               |
| [MVP-03](./MVP-03-tool-windows-layout.md)          | Tool Windows 布局（Primary/Secondary/Bottom + Activity Strip） | done                                                                                                                                                                                                                                                                                                                                                                                                                     | 4d   | MVP-01/02            |
| [MVP-04](./MVP-04-multi-tab-terminal.md)           | 多 Tab 终端（PTY + xterm + Shell/CLI 兼容）                    | ready（Phase A PR #72 · Phase B PR #82 · Phase C PR #91 · Phase E PR #95 · Phase F PR #99 · 仅 Phase D shell 兼容待）                                                                                                                                                                                                                                                                                                    | 8d   | MVP-03 · SPIKE-05/06 |
| [MVP-05](./MVP-05-pane-split-single-level.md)      | Pane 分屏（单层 · 最多 4 Pane · Smart Layouts）                | ready（spec PR #74 · Phase A/B/C 全 done · PR #141-#151 序列 + lifecycle bug 修 PR #208 · 集成完整 · 仅 Phase D runtime capture 待 Arbiter 30-45 min 跑 [CAPTURE-PLAYBOOK](../runtime-evidence/mvp-05/CAPTURE-PLAYBOOK.md)（PR #211 · 4 轮 codex review · 14 invariant）· 跑完翻 done）                                                                                                                                  | 4d   | MVP-04               |
| [MVP-06](./MVP-06-config-import.md)                | 配置导入（Ghostty + iTerm2 + Alacritty）                       | ready（spec PR #77 · **parser 层 Phase A PR #80 + A+ PR #81 · Kimi × 2** · Phase B IPC/UI 待 MVP-04 Phase B-F done 后）                                                                                                                                                                                                                                                                                                  | 3d   | MVP-04               |
| [MVP-07](./MVP-07-git-log-readonly.md)             | Git Log 只读视图 + Commit 详情                                 | **done**（spec PR #66 · 实施 **PR #83 · OpenCode** · gix 0.70 读路径 + SolidJS panel + H2 regression proof · 92 tests · UI 截图 + kernel benchmark GA gate 补）                                                                                                                                                                                                                                                          | 5d   | MVP-02/03 · SPIKE-03 |
| [MVP-08](./MVP-08-diff-and-git-status.md)          | Diff 基础视图（自绘）+ Git Status 只读面板                     | ready（spec PR #70 · Phase A PR #100 · Phase B PR #101 · Phase C/D done · Phase E partial · v0.2 fixture generator 进 git PR #140 · R-PHASE-E 3 DevTools P99 待 0.5d 主 agent 顺手）                                                                                                                                                                                                                                     | 5d   | MVP-07               |
| [MVP-09](./MVP-09-stage-unstage-commit.md)         | Stage/Unstage + Commit 操作（git2 写）                         | ready（spec PR #73 · Phase A 后端 PR #116 · Phase B Status 面板 + CommitBar PR #118 · Phase C 错误流 UX 补强 PR #159 · §D Criterion bench + §E 集成测试 PR #156 · 仅 Phase D runtime 截图待 GUI capture）                                                                                                                                                                                                                | 4d   | MVP-08 · SPIKE-04    |
| [MVP-10](./MVP-10-settings-telemetry-packaging.md) | 设置面板 + Telemetry opt-in + 打包发布（v0.1 GA）              | ready（spec PR #88 · v0.1 GA 终点 task · Phase A 设置面板 PR #114 · Phase B Sentry Spike PR #120 · **ADR-015 accepted PR #152** · Phase B SDK 编码 PR #155 · §C.4 endpoint UI + §G.4 H2 proof + §F capture guide PR #158 · §B.1 modal mount-time click guard PR #161 critical fix · §F.02 theme dual-path fix PR #163 · §F evidence 3/4 done · 仅 §F.04 DevTools 待 Arbiter + Phase C/D/E 打包等 SPIKE-06 §B Apple Dev） | 5d   | MVP-01..09 全部      |
| [MVP-11](./MVP-11-native-feel-quality.md)          | Native Feel Quality · 对标 MUX0 · 治"web 套壳"观感             | **done**（spec PR #119/#125 · Phase 1 Vibrancy ✅ PR #123/#130/#131 · Phase 2 Title bar Overlay ✅ PR #129 · Phase 3 Native Menu ✅ PR #124/#126 · Phase 4 Appearance ✅ PR #127/#128/#130 · Phase 5 Typography ✅ PR #122 · Linux fallback PR #139 · 5/5 全 done · v0.1 用户感知质量补强）                                                                                                                              | 6d   | MVP-10               |

**占位 spec（v0.2 / v0.3 / v1.0 范围 · `implementation-plan.md §10.1` 砍到后续版本）**：

| ID                                                  | 标题                                                                | 状态                                                                                                                                                                                                                                                                                                                                                    | 目标版本 | 估时 | 依赖                     |
| --------------------------------------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------- | ---- | ------------------------ |
| [MVP-21](./MVP-21-git-push-pull-fetch.md)           | Git Push / Pull / Fetch（远端同步）                                 | **done**（Phase A PR #228 git2 网络层 + auth + IPC · Phase B PR #231 push/pull/fetch UI · Phase C PR #233 conflict + ahead/behind · Phase D PR #236 runtime + 跨平台 · v0.2 sprint 实施全收）                                                                                                                                                           | v0.2     | 5d   | MVP-09 / MVP-13          |
| [MVP-22](./MVP-22-pty-warm-pool.md)                 | PTY 预热池 · 新 tab 瞬时出 prompt                                   | **done**（session 22 实施 · PR #189-#193 + Phase D 收尾 · 实施时 id 为 MVP-20 · session 23 rename 解 v1.0 占位 ai-one-click-rollback 同号冲突 · cold spawn 800-1200ms → warm hit 0.09ms backend）                                                                                                                                                       | v0.2     | 1.5d | MVP-04                   |
| [MVP-12](./MVP-12-commit-rail-graph.md)             | 自绘 commit rail graph（Git Log 图形化）                            | **ready**（session 24 · Droid（Factory.ai）详化 100% · 866 行 + 5 附录加分项（A 性能模板 / B a11y / C SPIKE-09 评估清单 / D 实施 PR 模板 / E 术语表）· §H.1 Canvas vs SVG vs DOM vs WebGL 4 行表 · §H.3 算法 3 候选不 pre-decide 留 SPIKE-09 · §H.6 4 个 P99 数字明确 · 主 agent cross-review + Arbiter approve · phase v0.2 → v0.3 一致化 · 等待认领） | v0.3     | 8d   | MVP-07                   |
| [MVP-13](./MVP-13-branch-crud.md)                   | 分支 create / checkout / delete + Fuzzy Switcher                    | **ready**（vibe sprint 2026-05-01 Worker B 详化 100% · 2026-05-03 self-review + Arbiter approve 翻 ready · 等待认领 · 阻塞 MVP-21 W14）                                                                                                                                                                                                                 | v0.2     | 4d   | MVP-07 / MVP-09          |
| [MVP-14](./MVP-14-pane-advanced-layout.md)          | Pane 高级布局（任意嵌套 + Dual AI / Triple / Quad + 导航 + 最大化） | **ready**（session 24 · Codex CLI 详化 100% · 614 行 · 主 agent cross-review + Arbiter approve · phase v0.2 → v0.3 一致化 · 等待认领）                                                                                                                                                                                                                  | v0.3     | 7d   | MVP-05                   |
| [MVP-15](./MVP-15-diff-syntax-highlight.md)         | Diff 语法高亮（shiki lazy load · 对齐 §W21）                        | **ready**（session 24 · OpenCode 详化 100% · 717 行 · 严格对齐 §W21 shiki · 0 新增 IPC binding · 主 agent cross-review + Arbiter approve · 等待认领）                                                                                                                                                                                                   | v0.3     | 4d   | MVP-08                   |
| [MVP-16](./MVP-16-rebase-merge-cherrypick.md)       | Rebase / Merge / Cherry-pick（含交互式 + 冲突解决）                 | **ready**（session 24 · Claude Code 主 agent 详化 100% · 707 行 · 含 3-way conflict + crash recovery · self-review + Arbiter approve · 等待认领）                                                                                                                                                                                                       | v0.3     | 7d   | MVP-08 / MVP-09 / MVP-13 |
| [MVP-17](./MVP-17-external-terminal-pane-detach.md) | Pop to External + Pane Detach                                       | **ready**（session 29 详化 100% PR #283/#284/#285 · Arbiter approve · Phase A done @ PR #291 · Phase B done @ PR #301 · Phase C wiring done @ PR #302 · 仅 E.4 settings UI follow-up + Phase D Arbiter playbook 推迟 · frontmatter 待 E.4 + Phase D 完成后翻 done）                                                                                     | v0.3     | 4d   | MVP-14                   |
| [MVP-18](./MVP-18-ai-aware-pane-linking.md)         | **AI-Aware Pane 联动**（v1.0 vision · 对外禁提）                    | **draft**（session 31 详化完成 100% · 611 行 · 48 checkbox · Codex CLI 详化 PR #309 · 等 Arbiter approve flip ready）                                                                                                                                                                                                                                   | v1.0     | 15d  | MVP-14 · SPIKE-07        |
| [MVP-19](./MVP-19-session-commit-binding.md)        | **AI session ↔ commit 自动绑定**（v1.0 vision）                     | **draft**（session 31 详化完成 100% · 740 行 · 43 checkbox · Cursor 详化 PR #313 · 等 Arbiter approve flip ready）                                                                                                                                                                                                                                      | v1.0     | 8d   | MVP-18                   |
| [MVP-20](./MVP-20-ai-one-click-rollback.md)         | **AI 一键回滚（session 级 revert）**（v1.0 vision）                 | **draft**（session 31 详化完成 100% · 647 行 · 25 checkbox + 12 sub · Droid 详化 PR #312 · 等 Arbiter approve flip ready）                                                                                                                                                                                                                              | v1.0     | 6d   | MVP-19                   |

> 占位 spec 用途：在 `<TYPE>-NN-<slug>` 编号连续性 + 依赖可视化上提前占位，v0.2 / v0.3 / v1.0 启动时按 kickoff 详化到实施 spec（补具体 UI 截图 / Acceptance 可量化门槛 / 数据模型细节）。
>
> ⚠ **ID 历史**（2026-05-03 · session 23 累积）：
>
> 1. **MVP-21** 历史 id 为 MVP-11 · 详化时与 v0.1 已 done 的 MVP-11 "Native Feel Quality" frontmatter id 冲突 · 已 rename 为 MVP-21（详化阶段建议方案 [A]）· MVP-11 编号已永久指向 [Native Feel Quality](./MVP-11-native-feel-quality.md)（v0.1 done）· 不再可用。
> 2. **MVP-22** 实施时 id 为 MVP-20（session 22 · PR #189-#193）· 与 v1.0 占位 [MVP-20 AI 一键回滚](./MVP-20-ai-one-click-rollback.md) 同号 · session 23 rename 为 MVP-22 · 解冲突。MVP-20 编号永久指向 v1.0 占位 ai-one-click-rollback · 实施时历史 trace（PROGRESS PR #189-#193 / branch / commit message 含 MVP-20）保留不动。
>
> rename 详见各 spec 顶部历史 comment + git mv history。

### BUG / FEAT

当前无。

---

## 🚀 新建 task 的流程（spec 创建 PR · `draft → ready` 落盘）

> 本流程用于**创建新的 task spec**（从无到 `status: ready`）。实施 task 的流程见 `CLAUDE.md` "🚀 新 Agent 首次启动" 第 5 步（`ready → in-progress → done`）。

```bash
# 1. 复制模板
cp docs/tasks/_template.md docs/tasks/SPIKE-07-<slug>.md

# 2. 填写 frontmatter（默认 status: draft）+ 正文 section

# 3. 开 feature 分支
git checkout -b docs/tasks/SPIKE-07-<slug>

# 4. 自审四问（CLAUDE.md "📝 写规则/清单前的自审四问"）
#    - 递归完备性 / 反向场景 / 边界适用性 / YAGNI

# 5. commit + push + PR（Conventional Commits + 中文描述 + trailer）
git commit -m "docs(tasks): 新增 SPIKE-07 <中文描述>

Co-authored-by: <Agent Name> <email>"
git push -u origin docs/tasks/SPIKE-07-<slug>
gh pr create

# 6. PR description 必填：
#    - Author: <作者 agent-id>
#    - Spec Reviewed by: <待评审>（和实施 task 的 Reviewer 不同）

# 7. 独立评审（≠ 原作者）approve 后，把 task status 从 draft → ready
#
#    **关键 gate（Codex PR #6 F1 + PR #10 教训）**：作者不得在 approve 之后私自
#    修改 spec 并翻转 status；必须走以下两种路径之一防绕过：
#
#    (a) Reviewer 自己 push 翻转 commit（推荐）
#        —— reviewer 在 PR branch 上 commit + push 翻转，作者无法插入新改动
#    (b) Author 翻转 status，Reviewer 必须 **re-approve 最新 HEAD** 才能 merge
#        —— GitHub 分支保护：require approval from latest commit
#
#    二选一，由评审者在 PR 评论里声明选哪个：
git commit -m "chore(tasks/SPIKE-07): spec reviewed, status: ready"
git push

# 8. merge → 此后其他 agent 可从 status: ready 认领
#    （走 CLAUDE.md 5 步导游的"认领 → 开工 → 收尾"流程）
```

---

## ⚠️ 原则（不要重演 Phase 1 过度设计）

1. **不做 claim 机制 / 自动状态流转 / CI 校验脚本**——Phase 2 真遇到并发问题再加（CLAUDE.md "📝 写规则/清单前的自审四问" 第 4 条 YAGNI）
2. **状态字段靠 PR description 和 commit 同步**，不在文件里搞复杂的锁
3. **task spec 冲突**：同一 task 两人同时动 → PR 冲突时 rebase + 保留两方意图 + scalar 冲突找 Arbiter（用户）
4. **task spec 是"一个 PR 一个逻辑单元"的依据**——评审者按 Acceptance 逐项对照
5. **Deliverables 用 per-task 文件，不用共享文件**：每个 task 写自己的 `docs/spikes/<id>-report.md` / `docs/adr/ADR-NNN-<slug>.md`，**不要**多个 task 都往 `docs/SPIKE-REPORT.md` 写——物理隔离比"声明式并发治理"更可靠（详见 `docs/session-history/` Phase 3 后的 PR #4 close 反思）
6. **`spike-tmp/` 是作者本地 scratchpad**（`.gitignore` 已排除），**不得作为其他 task 的依赖源**：跨 task 交接只能基于 committed / versioned 产物
7. **⚠️ State transition gate 当前是 advisory · accepted tech debt**（Codex PR #10 F1 复核 · 显式声明）：
   - 本 README 第 7 步 `draft → ready` 翻转 gate + `CLAUDE.md` 5.4 步 `ready → done` 翻转 gate，**Phase 2 仅靠 reviewer 肉眼守门 + PR 评论声明**——不做 repo-enforced validator，不做 GitHub 分支保护规则。**符合 YAGNI 原则**（第 1 条），但 Codex 指出这让 gate 在实际 merge 时可被绕过
   - **Phase 4 CI 必须落地**（`CLAUDE.md §当前可执行动作 3` 已列为 scope）：
     - frontmatter validator（校验 `status` / `blocked_from` / `owner` 字段组合合法性 · 例如 `status: blocked` 时 `blocked_from` 必填）
     - GitHub branch protection：`require approval from latest commit` + `require all status checks to pass`
     - PR body schema 校验（`Implemented by` / `Reviewed by` 必存在 · 从 commit trailer 提取 task-id 与 PR 标题一致）
     - `gitleaks` secret scan（SPIKE-06 A.5.3 依赖）
   - **Phase 4 落地前的约定**：
     - reviewer 是**唯一守门员**，reviewer 未发现的 gate 违规**算未修**
     - 任一 merge 后发现 gate 违规 → 立刻开 revert PR + 复盘写入 `docs/session-history/`
     - Phase 4 validator 上线后：本条第 7 项自动失效，规则从 "advisory" 升级为 "enforced"

8. **🔀 翻转 gate "(b) 路径变种"** · 分支保护暂缓阶段的合规说明（Codex round-3 PR #18 review 复核）：

   本节是上方第 7 步 `(a)/(b)` gate 在**当前 accepted tech debt 状态**下的正式衍生路径。**纯术语收敛 · 不引入新规则**。

   **背景**：上方 `(b)` 标准路径强依赖 GitHub 分支保护 `require approval from latest commit`，但项目当前分支保护已被用户显式暂缓（accepted tech debt）。在该状态下 `(b)` 标准路径的"技术强制"缺失。

   **(b) 路径变种** = 在分支保护暂缓阶段对 `(b)` 的人工执行版本：
   - **流程上等价于 (b)**：Author push 翻转 commit + Reviewer re-approve 最新 HEAD
   - **替代品**：靠 reviewer 真实 GitHub UI approve（`reviews ≠ ∅`）+ reviewer 在 PR comment 里**显式声明走哪个路径**（README §第 7 步 (a)/(b) 二选一）
   - **关键硬要件**：
     - `gh pr view <N> --json reviews` 返回的 `reviews` 列表**必须非空**（含至少一个 `state: APPROVED`）
     - PR comments 必须含 reviewer 的路径声明（防作者私自代签）
   - **不合规变种**（已被 PR #17 v1 codex round-2 抓出 BLOCK）：
     - "merge 间接 approve"：作者 push 翻转 commit + 直接 squash merge · `reviews=[]` · 不算 approve
     - "comments=[]"：reviewer 没在 PR 评论里声明路径 · 即使 `reviews ≠ ∅` 也仍违反 README §205 要求

   **何时升级**：分支保护一旦应用（升级触发条件见 `docs/PROGRESS.md §🔐 用户手动步骤`），本变种自动失效，规则回归 `(b)` 标准路径（技术强制 require-from-latest）。

---

**本目录 Phase 2 建立（2026-04-18）。SPIKE-01..06 作为 Spike W0 启动的硬依赖。**
