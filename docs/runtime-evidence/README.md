# Runtime Evidence

> 顶层索引 · 依据 [ADR-011](../adr/ADR-011-runtime-evidence-location.md) · 当前快照来自 `docs/runtime-evidence/` 实测目录 + `node scripts/validate-runtime-evidence.mjs`

## 目的

Runtime evidence 是 MVP / feature PR 的运行时证据目录，用于证明 **CI 绿不等于 runtime 过**。GUI / IPC / PTY / 外部进程类改动必须把截图、录屏、benchmark、H2 proof 或 smoke output 持久化到 git。

本目录同时承接三层规则：

- [ADR-011](../adr/ADR-011-runtime-evidence-location.md)：决策 `docs/runtime-evidence/<task-id>/` 作为标准位置。
- [.claude/rules/runtime-evidence-location.md](../../.claude/rules/runtime-evidence-location.md)：项目级 R1-R5 落地。
- [~/.claude/rules/15-runtime-verification-gate.md](~/.claude/rules/15-runtime-verification-gate.md)：全局 runtime verification gate。
- [.claude/rules/dispatch-prompt-template.md §2.3](../../.claude/rules/dispatch-prompt-template.md)：dispatch 中按任务类型要求 runtime 证据；纯 docs / chore 可明示豁免。

## 当前总览

当前顶层 evidence 目录数：26。

当前 validator 摘要：

```text
Runtime evidence：扫描 26 个目录 · ✅ PASS 22 · 🟡 WARNING 4 · 🔴 ERROR 0
```

已有持久化报告：[`_VALIDATION-REPORT.md`](./_VALIDATION-REPORT.md)。注意该报告是 2026-05-12 快照，当前 README 以最新 validator 命令摘要为准。

## 目录结构总览

| 目录                   | 文件数 | 证据类型                                           | Validator          |
| ---------------------- | -----: | -------------------------------------------------- | ------------------ |
| `chore-ts-rs-rollout/` |      1 | H2 regression proof                                | PASS               |
| `mvp-01/`              |      3 | Ubuntu deb/AppImage launch + bundle info           | PASS               |
| `mvp-02/`              |      3 | workspace UI screenshots                           | PASS               |
| `mvp-03/`              |      5 | layout / theme screenshots                         | PASS               |
| `mvp-04/`              |      6 | tab lifecycle screenshots + metrics                | WARNING R4         |
| `mvp-04-phase-b/`      |      3 | PTY backend logs + smoke GIF                       | WARNING R3         |
| `mvp-04-phase-c/`      |      5 | tab UI screenshots                                 | PASS               |
| `mvp-04-phase-e/`      |      4 | persistence screenshots                            | WARNING R4         |
| `mvp-04-storage-prep/` |      3 | migration / schema / test logs                     | PASS               |
| `mvp-05/`              |      2 | capture playbook + metrics template                | PASS               |
| `mvp-06-phase-a/`      |      1 | cargo test log                                     | PASS               |
| `mvp-06-phase-a-plus/` |      1 | cargo test log                                     | PASS               |
| `mvp-07/`              |      3 | H2 proof + perf + screenshot notes                 | PASS               |
| `mvp-08/`              |     17 | Git status / diff / fs-watch screenshots + metrics | WARNING R4         |
| `mvp-09/`              |      4 | benchmark + integration outputs                    | PASS               |
| `mvp-10/`              |     20 | settings / telemetry / package / command logs      | PASS               |
| `mvp-11/`              |     12 | native feel screenshots + MOV + metrics            | PASS via exception |
| `mvp-12/`              |      6 | rail graph Phase A automation evidence             | PASS               |
| `mvp-13/`              |      1 | branch ops benchmark output                        | PASS               |
| `mvp-14/`              |      5 | pane backend tests + H2 proof + bindings           | PASS               |
| `mvp-15/`              |     10 | syntax highlight screenshots + raw bench logs      | PASS               |
| `mvp-16/`              |      6 | rebase ops tests + bindings + H2 + bench           | PASS               |
| `mvp-17/`              |      5 | settings UI screenshots + dev-mode blocker log     | PASS               |
| `mvp-21/`              |      1 | git sync benchmark output                          | PASS               |
| `mvp-22/`              |      5 | PTY warm pool cold / warm / settings docs          | PASS               |
| `spike-04.5-a3-b/`     |      4 | spike raw bench logs + report                      | PASS               |

## MVP 状态索引

| MVP / evidence dir    | Phase 完成度来源                                     | Runtime evidence 状态              | Deferred items                           |
| --------------------- | ---------------------------------------------------- | ---------------------------------- | ---------------------------------------- |
| `mvp-01`              | launch smoke evidence present                        | 完整 · PASS                        | —                                        |
| `mvp-02`              | PROGRESS 旧表记为 done                               | 完整 · PASS                        | —                                        |
| `mvp-03`              | PROGRESS 旧表记为 done                               | 完整 · PASS                        | —                                        |
| `mvp-04`              | PROGRESS：Phase A-F done；§I 待补                    | 部分 · WARNING R4                  | §I 22 PNG + 2 MOV                        |
| `mvp-04-phase-b`      | phase B runtime logs present                         | 部分 · WARNING R3                  | 归入 MVP-04 §I                           |
| `mvp-04-phase-c`      | phase C screenshots present                          | 完整 · PASS                        | 归入 MVP-04 §I                           |
| `mvp-04-phase-e`      | phase E screenshots present                          | 部分 · WARNING R4                  | 归入 MVP-04 §I                           |
| `mvp-04-storage-prep` | storage prep logs present                            | 完整 · PASS                        | —                                        |
| `mvp-05`              | PROGRESS：Phase A/B/C done；Phase D capture 待跑     | playbook ready · PASS              | Phase D capture                          |
| `mvp-06-phase-a`      | phase A backend evidence present                     | 完整 · PASS                        | —                                        |
| `mvp-06-phase-a-plus` | phase A+ backend evidence present                    | 完整 · PASS                        | —                                        |
| `mvp-07`              | PROGRESS 旧表记为 done                               | 完整 · PASS                        | —                                        |
| `mvp-08`              | PROGRESS：Phase A-D done；目录含 Phase E 量化文件    | 部分 · WARNING R4                  | —                                        |
| `mvp-09`              | PROGRESS：Phase A/B/C done；Phase D performance done | 自动化 evidence · PASS             | Phase D runtime screenshots              |
| `mvp-10`              | PROGRESS：Phase A/B done；§F evidence 3/4 done       | 部分 · PASS                        | §F.04 outbound network panel             |
| `mvp-11`              | native feel evidence set present                     | 完整 · PASS with exception         | —                                        |
| `mvp-12`              | PROGRESS：A/B/C code done；Phase D deferred          | 自动化 evidence · PASS             | v0.3 Phase D capture                     |
| `mvp-13`              | PROGRESS：全 4 phase 自动化 100%；GUI deferred       | bench evidence · PASS              | Phase D GUI screenshots                  |
| `mvp-14`              | PROGRESS：A/B/C code done；Phase D deferred          | 自动化 evidence · PASS             | v0.3 Phase D capture                     |
| `mvp-15`              | PROGRESS：A/B/C done；D §F/§G 自动化全收             | bench / screenshot evidence · PASS | v0.3 GUI / WCAG / cross-platform capture |
| `mvp-16`              | PROGRESS：A/B/C done；D part A bench done            | 自动化 evidence · PASS             | Phase D part B GUI / Linux               |
| `mvp-17`              | PROGRESS：A/B/C/E.4 代码收口；D playbook 推迟        | screenshots / blocker log · PASS   | v0.3 Phase D capture                     |
| `mvp-21`              | PROGRESS：A/B/C done；Phase D deferred               | bench evidence · PASS              | Phase D GUI screenshots / recordings     |
| `mvp-22`              | PROGRESS session 22：Phase D done                    | 完整 · PASS                        | —                                        |

## Deferred Items 跟踪

Deferred items 以 [`docs/PROGRESS.md`](../PROGRESS.md) 的 `Next concrete action` 和 `当前位置` 段为准；本 README 只做索引，不替代 PROGRESS。

当前 deferred 清单：

- v0.3 sprint `mvp-12` / `mvp-13` / `mvp-14` / `mvp-15` / `mvp-16` / `mvp-17` Phase D capture：按 PR #271 Arbiter playbook，一次性跑 GUI / DevTools Performance / visual regression / WCAG / cross-platform。
- `mvp-04` §I：22 张 PNG + 2 段 MOV。
- `mvp-05` Phase D：`CAPTURE-PLAYBOOK.md` 已就位，需 Arbiter 30-45 min capture。
- `mvp-09` Phase D runtime：截图类证据待 GUI。
- `mvp-10` §F.04：0 outbound DevTools network panel。
- `mvp-13` Phase D GUI screenshots。
- `mvp-21` Phase D GUI screenshots / recordings。

触发条件：

- Arbiter 主动声明“开始跑 capture”。
- 或进入 v0.2 GA 候选阶段，需要把所有 deferred runtime capture 一次性收口。

## 命名与体积约定

ADR-011 / 项目规则 R1-R5 的最小执行口径：

- 位置：`docs/runtime-evidence/<lowercase-task-id>/`。
- 进 git：runtime evidence 不放 gitignored 临时目录。
- 命名：`01-<name>.jpg` / `02-<name>.png` / `03-<name>.mp4`。
- 顺序前缀必须：媒体文件使用两位数字前缀，英文小写 kebab-case。
- 体积：单 MVP / feature 目录推荐 ≤ 3MB，上限 10MB；单文件上限 10MB。
- PR body：必须引用 evidence 路径；纯 docs / chore 任务可在 PR body 明示 runtime 证据豁免。

当前已知 validator WARNING：

- `mvp-04` / `mvp-04-phase-e` / `mvp-08`：R4 推荐体积超 3MB，但未超硬上限。
- `mvp-04-phase-b`：`tauri-dev-smoke.gif` 不符合 R3 `NN-kebab-name.ext`。
- `mvp-11`：有 spec-mandated 命名和体积豁免，记录在 `.validator-exceptions.json`。

## Validator

PR-time validator：

```bash
node scripts/validate-runtime-evidence.mjs
```

常用模式：

```bash
node scripts/validate-runtime-evidence.mjs --mvp mvp-22
node scripts/validate-runtime-evidence.mjs --report docs/runtime-evidence/_VALIDATION-REPORT.md
node scripts/validate-runtime-evidence.mjs --strict
node scripts/validate-runtime-evidence.mjs --exceptions .validator-exceptions.json
```

职责边界：

- R1：目录位置与 deprecated `spike-tmp/img/` 残留。
- R2：证据文件必须进入 git 跟踪。
- R3：媒体文件命名。
- R4：单文件和目录体积。
- R5：PR body 引用 evidence 路径；脚本不校验，reviewer 在 PR 中检查。

## 关联

- 上位 ADR：[ADR-011 · MVP / Feature runtime 证据存储位置标准化](../adr/ADR-011-runtime-evidence-location.md)
- 项目规则：[`.claude/rules/runtime-evidence-location.md`](../../.claude/rules/runtime-evidence-location.md)
- 全局规则：[~/.claude/rules/15-runtime-verification-gate.md](~/.claude/rules/15-runtime-verification-gate.md)
- Dispatch 规则：[`.claude/rules/dispatch-prompt-template.md` §2.3](../../.claude/rules/dispatch-prompt-template.md)
- Validator 脚本：[`scripts/validate-runtime-evidence.mjs`](../../scripts/validate-runtime-evidence.mjs)
- Exception 配置：[`.validator-exceptions.json`](../../.validator-exceptions.json)
