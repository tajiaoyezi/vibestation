# Dispatch Prompt 规则 · 事件档案与详例（审计附录）

> 本文件是 [`.claude/rules/dispatch-prompt-template.md`](../.claude/rules/dispatch-prompt-template.md) 的**审计附录**。
>
> - **规范正文**（规则 + 具体做法 + code block + 速查表 + 模板）在 `.claude/rules/dispatch-prompt-template.md`，进 `.claude/rules/` auto-load。
> - **本文件**保存每条硬约束的：完整「事件」叙述、完整「反模式」对照表、详细「为什么」、逐条「关联」、参考实现历史、规则来源时间线。
> - 本文件**不在 `.claude/rules/`**（不进每 session auto-load · 降 context 注入量），但**进 git**（满足 v2-D.2 / [ADR-016](./adr/ADR-016-admin-override-trailer-exemption.md) / 全局 rule 13 的审计要求 · clone 零依赖可溯源）。
> - **回引**：规范正文每条的「📎 事件 / 反模式详例 →」锚点指向本文件对应 section。锚点用稳定 `<a id>`，不随中文 slug 变化（符合本规则 §2.13 精神：引用稳定锚点，不 inline 易变内容）。
> - **拆分依据**：2026-05-15 · `docs/dispatch-rule-compress` · 原单文件 849 行 / 55KB / ~40.6k 字符触发 context 性能警告 · 档位 B 拆分 · 规范正文降至 ~340 行 / ~21KB / ~15k 字符。

---

<a id="ev-1"></a>

## §1 核心原则 · 反模式（本项目两次踩坑）

| 事件                     | 措辞                         | 外部 agent 解读                                       |
| ------------------------ | ---------------------------- | ----------------------------------------------------- |
| SPIKE-04.5 §A.3 dispatch | "**不要**自己说 accept"      | OpenCode 视作建议 · 绕过 · 自标 "Arbiter 选定方案(a)" |
| MVP-02 dispatch          | "独立 worktree **强烈建议**" | OpenCode 视作建议 · 在主 working tree 开工            |

**教训**：外部 agent 会倾向走最短路径 · "建议"级条款容易被绕过。对**不能绕过**的要求 · 必须升级为"硬约束"措辞。

---

<a id="ev-2-1"></a>

## §2.1 禁止自行 accept decision-grade · 事件

2026-04-19 · SPIKE-04.5 §A.3 · OpenCode 自行标 "Arbiter 选定方案(a)" · Arbiter 事后 comment approve + 硬约束规则化。

---

<a id="ev-2-4"></a>

## §2.4 独立 worktree · 事件

2026-04-19 · MVP-02 · OpenCode 在主目录开 `feat/MVP-02-workspace-management` · 主 agent checkout main 时 git 默认 carry-over unstaged 改动 · 主 working tree 脏 · 阻塞主 agent 开新 PR · 用户通知后 OpenCode 按 Option 1（commit + push + 独立 worktree）恢复。

---

<a id="ev-2-5"></a>

## §2.5 Commit 身份 · 为什么 / 反模式 / 事件

### 为什么改用 `--worktree`（session 28 实证 · 2026-05-12）

- `git worktree add` 创建的 worktree **共享主 repo 的 `.git/config`**（不是独立 config）
- 直接 `git config user.email "..."`（无 `--worktree` flag）默认写主 repo `.git/config` · 会被同 repo **所有** worktree 继承 · 也污染主 agent 在主 working tree 的 commit
- session 28 实测：3 agent 顺序设 user.email → 主 repo `.git/config` 反复被覆盖 → 主 agent commit author 错归 OpenCode / Cursor / 等 · 复发 3 次
- **唯一根治**：启用 `extensions.worktreeConfig=true` 后 · `--worktree` flag 写 `.git/worktrees/<name>/config.worktree` · 真正 per-worktree 隔离

### 反模式（违反 BLOCK · session 28 后强化）

| 反模式                                                               | 正确做法                                                                               |
| -------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `git config user.email "..."`（无 --worktree）                       | 必须 `git config --worktree user.email "..."`                                          |
| 跳过 `git config extensions.worktreeConfig true` · 直接 `--worktree` | git 拒绝 · `--worktree` flag 需先启用 extension（fallback 写主 repo · 污染）           |
| 假设主 repo .git/config 不会被 worktree 污染                         | session 28 实证 3 次污染 · 必须 `extensions.worktreeConfig=true` + `--worktree` 双保险 |

### 事件（2.5.3 反模式根因）

只做 2.5.2 trailer · 跳过 2.5.1 git config → `git log` / `git blame` author 字段错 → 未来若上 CODEOWNERS 或 contribution 审计 · Codex 的贡献被归给 Kimi。

2026-04-20 session 12 · PR #71 · Codex 继承上一 Kimi task 的 worktree git config · commit author 显示 "Kimi" · 仅靠 trailer 不够 · audit M3 根因。

---

<a id="ev-2-8"></a>

## §2.8 子进程清理 · 事件

2026-04-19 · MVP-02 · OpenCode 跑 `pnpm tauri:dev` 截图后没 cleanup · Vite/pnpm 进程 orphan 4 小时占 port 1420 · main agent 后续 session 启动 dev 失败 · 用户报错排查才定位到是 OpenCode 残留（PID 4920/4953/5060 · 另含 Codex spike-05-pty 19648）。

---

<a id="ev-2-9"></a>

## §2.9 Agent 能力矩阵 · 反模式 / 事件 / 关联

### 反模式

| 反模式                                            | 真正该做                                            |
| ------------------------------------------------- | --------------------------------------------------- |
| 复制本地 agent 模板（只给路径）给远程 API agent   | 按 agent 类型分支 · 远程 API 必须附原文             |
| 假设所有 agent 都能 `git worktree add`            | 明确询问 / 默认无 · 双路径兼容                      |
| 远程 API prompt 说 "参考 CLAUDE.md §X"            | 把 §X 关键段摘出来贴进 prompt                       |
| 不在 meta 段声明 agent 类型                       | 每个 dispatch prompt 顶部必写一行 `Agent 类型：...` |
| 给 Cursor IDE 内嵌发本地 CLI 模板（缺完工硬约束） | 显式加 "完工 = PR 链接" 硬约束 + 端到端步骤         |

### 事件 · 2026-04-20 · session 12 · MVP-07 Kimi 首次踩坑

- 主 agent 仿 `MVP-04-kimi-prompt.md` 模板写 `MVP-07-kimi-prompt.md`（只引用路径 · 未附 spec 原文）
- 用户指出 "kimi 的 你怎么直接用的 tasks 下的 md 文件 并且也没有 prompt 呀"
- 修复：prompt 从 167 行扩到 335 行 · 嵌入 MVP-07 spec 完整 140 行原文 + 双路径兼容
- 根因：主 agent 对 MVP-04 Kimi 成功的 post-hoc 叙事错误 · 未验证真实机制（worktree access / 用户补发 / Kimi 工具 · 主 agent 不知道）· 本 §2.9 规则化

### 事件 · 2026-05-14 · session 31 · Cursor IDE 内嵌模式 PR #313 踩坑

- 主 agent 派 Cursor 做 MVP-19 spec 详化 · prompt 标 "Agent 类型：本地 CLI"· 实际 user 用 Cursor IDE 内嵌 chat（编辑器内置 AI · 非 `cursor-agent` CLI）
- Cursor 写完 740 行 spec + 跑 `pnpm lint` + `pnpm -C web exec prettier --check ../docs/tasks/MVP-19-*.md` 全过 · 但**停在 commit 前**问 user "如果你要 · 我可以继续按你给的规范直接生成 commit message 与 PR body 草稿"
- 主 agent 需要额外回合让 Cursor 继续跑 commit + push + PR · 浪费 1 个回合
- 根因：dispatch prompt 把 `完工开 PR` 当软建议 · 没明示"完工 = PR 链接生成 · 不允许停下"· IDE 内嵌模式默认 = 安全停下 · 不像 CLI 自动跑完
- 修复：§2.9 升级（IDE 插件类专项约束 + Cursor 双形态区分）· 后续 dispatch 给 Cursor IDE 内嵌必须显式加完工硬约束 + 警示"上次 PR #313 教训防重演"

### 关联

- [全局] `~/.claude/rules/17-dispatch-agent-capability-matrix.md` · 本节的上位通用规则
- [项目] Kimi 协作记录：`spike-tmp/dispatch/MVP-04-kimi-prompt.md`（167 行 · 路径版 · 成功但不清楚机制）· `MVP-07-kimi-prompt.md`（335 行 · 原文版 · 确定成功）

---

<a id="ev-2-10"></a>

## §2.10 GUI / 前端 lint · 事件 / 反模式

### 事件

2026-04-21 · PR #83（OpenCode MVP-07 Git Log）· OpenCode 在 CLI 自动化会话只跑 `pnpm typecheck` · 漏 `pnpm lint` · 5 前端文件（`SecondarySidebar.tsx` / `GitLog/*` / `styles.css`）未 prettier 格式化 · CI fail · 后续 PR #84/#85 继承 fail · 直到 PR #86 修复。

2026-05-14 · session 31 · PR #311（OpenCode SPIKE-07 详化 N=4 understanding gap）· OpenCode 按 prompt §交付要求段字面执行 `pnpm lint` · PR body claim "markdown prettier check 通过" · 但 `pnpm lint` 实际不含 markdown · spec 文件 `docs/tasks/SPIKE-07-cli-protocol-parser.md` 未被 prettier 格式化 · 主 agent 自验 `npx prettier --check` fail · fix-up commit `bd9f57d`（+74 / -48 · 122 行格式化 · 0 内容改动）。判定 understanding gap（非 willful 谎报）· 不触发 N=5 · 但 §2.10 升级 markdown 显式 check 防重演。

### 反模式

| 反模式                                                            | 正确做法                                                                           |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Dispatch prompt 只列 `pnpm typecheck`                             | 两条都列 · 缺一 BLOCK                                                              |
| 交付 agent 回报 "typecheck 过" 作为前端 gate 证据                 | 必须同时显示 prettier `All matched files use Prettier code style!`                 |
| 假设 `tsc --noEmit` pass 意味着前端 OK                            | typecheck 只查类型 · 不查格式 · prettier 是独立 gate                               |
| markdown 文档 task 用 `pnpm lint` 作为 prettier 通过证据          | 必须 `npx prettier --check <markdown-file>` 显式 check · `pnpm lint` scope 不含 md |
| 交付 agent 回报 "pnpm lint 通过" 作为 markdown 详化 task 完工证据 | 必须同时贴 `npx prettier --check <markdown-file>` raw output 末 3 行               |

---

<a id="ev-2-11"></a>

## §2.11 跨平台 timeout · 为什么 / 事件 / 反模式

### 为什么

本地 macOS（kqueue）和 Linux CI（epoll）对 PTY close event / signal 传递 / `waitpid` / `tcgetpgrp` 的语义可能有差异 · 纯 timeout 扩张不解决语义问题。

### 事件

2026-04-21 · PR #82（Codex MVP-04 Phase B PTY runtime）· `pty::tests::signal_sigterm_exits_exec_session` 本地 macOS 1s 内过 · Ubuntu CI 5s timeout。PR #86 round 1 改 200→500ms + 5s→10s · CI 仍 fail（11.45s）。PR #86 round 2 改 `#[cfg_attr(target_os = "linux", ignore)]` · CI 绿。**教训**：一路加 timeout 是症状治疗 · 根因深度在 SIGTERM → pty master fd close event → epoll readable 的传递链中某一环 Linux 和 macOS 语义不同 · 不是 timing。

### 反模式

| 反模式                                 | 正确做法                                                                |
| -------------------------------------- | ----------------------------------------------------------------------- |
| 本地 `cargo test` 过就交付             | 跨平台测试必须预判 CI 是否 pass · 不确定时加 Linux-only ignore + 技术债 |
| CI fail 后反复加 timeout               | timeout × 2 仍 fail · 立即切 ignore + 深挖留下次                        |
| 不在 spec §已知风险 记 ignore 的技术债 | 必须记 · 含 GA gate 解除条件                                            |

---

<a id="ev-2-12"></a>

## §2.12 git config unset · 为什么 / 事件 / 反模式

### 为什么

`git worktree add` 共享 `.git` 目录 · worktree 默认共享 `.git/config`（除非启用 `extensions.worktreeConfig=true` + 用 `--worktree` flag）。**session 14 + session 28 共两轮实证**：worktree 操作（无 `--worktree`）会污染主 repo 的 local config · 被同 repo 所有 worktree + 主 working tree 继承。

`--worktree` 写入位置不再共享 · 跨 agent 污染从根上消除。

### 事件 · 2026-04-21 · session 14 · 3 次跨 agent author 污染

1. PR #82（Codex 交付）· 核心 commit `9fb6715` author 错归 Kimi · 根因：worktree 之前被 Kimi session 用过 · git config 继承 · Codex 未 reset · 主 agent 代修后复发
2. PR #83（OpenCode 交付）· 前端 commit `366cd73` author 错归 Codex · 根因：主 agent 修 PR #82 时 `git config user.email noreply@openai.com` 到 worktree · OpenCode 后续 commit 继承 · 主 agent 代修
3. PR #84（主 agent sync PR）· 主 repo **自己的 local config** 也被污染为 `OpenCode <noreply@opencode.ai>` · 主 agent commit 初次归为 OpenCode · unset local + amend reset 恢复为 global user

### 事件 · 2026-05-12 · session 28 · 3+ 次主 repo .git/config 污染（机制已确诊）

1. PR #272（主 agent backfill）首次 commit author = `OpenCode <noreply@opencode.ai>` · 根因：早段 Cursor / OpenCode 在主 repo 主 working tree 跑 `git config user.email` · 留 .git/config 污染 · unset local + amend reset 修复
2. PR #274（主 agent MVP-08 PNG→JPG）首次 commit author = `Cursor <noreply@cursor.com>` · 根因：Cursor §2.4 violation 期间在主 repo 设 config · unset local + amend reset 修复
3. PR #276（主 agent clippy fix）首次 commit author = `OpenCode <noreply@opencode.ai>` · 根因：4-track 并发期 OpenCode worktree 跑 §2.5.1（旧版无 `--worktree`）写主 repo .git/config · unset local + amend reset 修复 + 启用 `extensions.worktreeConfig=true`
4. **session 28 之后 §2.5.1 升级**：所有 dispatch prompt 改用 `git config --worktree` · 从根上消除（见规范正文 §2.5.1）

### 反模式

| 反模式                                                              | 正确做法                                                                                         |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| 在别人 worktree 改 git config 后不 unset                            | 必须 unset · 或该 worktree 用完立即 `git worktree remove` 销毁                                   |
| dispatch prompt 用 `git config user.email "..."`（无 `--worktree`） | session 28 起必须 `git config --worktree` + 先启用 `extensions.worktreeConfig=true`（见 §2.5.1） |
| 假设主 repo .git/config 不会被 worktree 污染                        | session 14 + session 28 共 6 次实证污染 · 必须双保险（防御性 unset + `--worktree` 隔离）         |
| 每次 commit 忘记验证 author（硬约束 2.5.3）                         | 2.5.3 的硬约束必须执行 · `git log -1 %an <%ae>` 是最后一道防线                                   |

---

<a id="ev-2-13"></a>

## §2.13 索引同步禁 inline · 完整示范 / 为什么 / 事件 / 反模式 / 关联

### 正确做法（完整示范）

````markdown
## 步骤 1 · 同步 ADR-NNN 决策表行（保留 PR #X 的最新措辞）

**禁止**：从本 prompt inline 原文重写整个 ADR-NNN 文件
**必须**：

```bash
git fetch origin

# 字节级恢复 ADR 本体 · 保留 PR #X 的最新措辞

git checkout origin/main -- docs/adr/ADR-NNN-foo.md

# 然后只补需要新增的索引条目（在其他文件）

```
````

### 禁止做法对照（完整示范）

````markdown
## ❌ 错误：inline 原文（agent 会基于这份 inline 重写 · 覆盖 PR #X 的修订）

```markdown
<!-- 整篇 ADR-NNN 内容贴在这里 ·  agent 会照贴重写 -->

status: proposed
...
```
````

### 为什么

索引同步类任务的核心是 "**只新增 / 删除索引条目**" · 不应触碰被索引文件的本体。但 prompt 起草时若 inline 了被索引文件的"当时版本" · 远程 API agent 或字面执行的 agent 会把 inline 版本当真相 · 直接 overwrite 文件 · 抹掉 inline 之后到 agent 执行之间发生的所有合法修订。

### 事件 · 2026-04-26 · session 20 · PR #157 round 1 ADR-015 倒退

- 主 agent 先 merge PR #152（ADR-015 proposed → accepted · Arbiter approval 措辞精确写入）
- 主 agent 然后下发 U2 prompt 给 Ubuntu Kimi · 任务："同步 ADR README 索引行 + 决策表 #10 行"
- prompt 里 inline 了 ADR-015 的"起草时版本"原文（**proposed 状态** · 未含 PR #152 的修订）
- Kimi 字面执行：直接基于 inline 重写 `docs/adr/ADR-015-telemetry-stack-sentry.md` 整篇 · **覆盖 PR #152 的 accepted 措辞**
- 用户发现："这种错误 应该让 kimi 自己去修复"
- 主 agent 重写 fix prompt：用 `git checkout origin/main -- docs/adr/ADR-015-telemetry-stack-sentry.md` 字节级恢复 · Kimi push round 2 · 主 agent merge

**根因**：U2 prompt 的设计错误 · 不是 Kimi 的执行错误。索引同步类任务**不应**在 prompt 里 inline 被索引文件本体 · 应该用 git 命令让 agent 拿当前 HEAD 的真相。

### 反模式

| 反模式                                                         | 正确做法                                                                                                             |
| -------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| 索引同步 prompt inline 被索引文件原文                          | 用 `git checkout origin/main -- <file>` 让 agent 拿 HEAD 真相                                                        |
| 假设 "agent 会自己 git pull · inline 只是参考"                 | 远程 API agent 没有 git · 字面执行 prompt · inline = 真相                                                            |
| prompt 起草后立即下发 · 不考虑期间其他 PR merge 的可能         | 索引类任务下发前必须 `git fetch origin && git status` 检查 · 若期间有相关文件改动 · 必须更新 prompt 用 checkout 模式 |
| 用 inline 是为了让 agent "看清结构" · 但要求 agent "只改 X 行" | "只改 X 行" 类任务不需要 inline 全文 · 用 sed / `git apply` 补丁 · 或用步骤式指令配 grep 锚点                        |

### 关联

- [全局] `~/.claude/rules/13-cross-agent-delivery.md` · 跨 agent 交付物持久化（本 §2.13 是其在 dispatch 阶段的细化）
- [项目] `.claude/rules/dispatch-prompt-template.md` §2.9 · Agent 能力矩阵（远程 API agent 字面执行特性 · 本 §2.13 的根源约束）
- [项目] `docs/session-history/session-20.md`（待写）· PR #157 round 1 / round 2 完整时序

---

<a id="ev-2-14"></a>

## §2.14 Reviewer dev mode · 事件 / 反模式 / 关联

### 事件 · 2026-04-26 session 20 · 三个 critical / secondary bug 都是 reviewer 漏 dev mode 跑

| PR   | bug                                                                                                                                                                                   | 漏 dev mode 后果                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| #159 | MVP-09 Phase B 主体（PR #118）merge 时 19 个 vs-commit-_ / vs-toast-_ / vs-dialog-\* CSS class **全无定义**（裸 HTML）· reviewer 只看 Rust + IPC contract · 漏                        | dev mode 一启动 dialog 完全无样式 · 用户感知严重 UI degradation · 直到 PR #159 才修              |
| #161 | MVP-10 Phase B SDK 主体（PR #155）merge 时 modal mount-time webview 虚假 click · 用户**完全看不见** modal · DB 已被写虚假决策 · spec §B.1 隐私关键 path 失效 · reviewer 没启 dev mode | v0.1 GA blocker · 5 轮 dev restart 调试才定位 webview race · 修 200ms guard                      |
| #163 | MVP-10 Phase B SDK 主体（PR #155）+ Phase A（PR #114）共同遗留 · status bar `theme_set` IPC 不 emit `settings_changed` · UI 不刷 · violate spec §F.02 实时生效                        | reviewer 没切 theme 验证 · 因为 UI/UX path 是双 IPC 路径分离 · 单看 Rust 测试 + ts-rs 一致看不出 |

**根因**：reviewer 把 `cargo test green + pnpm typecheck 0 errors + ts-rs contract 一致` 等同于 "PR 可 merge"。但 GUI/IPC 类 PR 的 critical path 必须 dev mode 跑过才能 catch webview race / event delegation / dual IPC path / CSS missing 类问题。这是**全局 rule 15 在 dispatch + review 阶段的具体落地**。

### 反模式

| 反模式                                         | 正确做法                                                                       |
| ---------------------------------------------- | ------------------------------------------------------------------------------ |
| reviewer 只看 PR diff + CI 全绿就 approve      | 必须本地 checkout + dev mode + 跑 critical UX path                             |
| 假设 "前端改动小 · 不用看 UI"                  | 任何 frontend 改动都可能 hide dialog / 影响 reactive update · 必须 dev mode 看 |
| 假设 "Rust 端 IPC 测试通过 = 整条 IPC path OK" | Rust 端通 ≠ 前端 emit / listen / state update 对 · 必须 end-to-end 看          |
| reviewer 时间紧 · 跳过 dev mode                | spec §runtime evidence 段 reviewer 必须 visual confirm · 不允许跳过            |

### 关联

- [全局] `~/.claude/rules/15-runtime-verification-gate.md` · "CI 绿 ≠ runtime 过"（本 §2.14 是 reviewer 阶段的具体落地）
- [项目] `.claude/rules/runtime-evidence-location.md` · runtime evidence 路径（reviewer 看 evidence 是 verification 一部分 · 但**不能替代** dev mode 自跑）
- [项目] `dispatch-prompt-template.md §2.3` · runtime 证据必交（implementer 责任 · 本 §2.14 是 reviewer 责任）

---

<a id="ev-2-15"></a>

## §2.15 stale base race · 为什么 / 事件 / 反模式 / 关联

### 为什么

≥ 3-agent 并发派工时 · push 时 worktree base ≠ main 当前状态：

- T1 worktree base: `main_A`
- T2 worktree base: `main_A`
- T2 push + merge → main 变 `main_B`（T1 不知道）
- T1 push + GitHub auto-merge（文件域无 git 冲突）→ 但 T1 测试 expected 仍依据 `main_A` 的源码 · `main_B` 已变 · merge 后破 main

### 事件源 · 2026-05-13 session 30 · MVP-17 4-agent 收尾

- OpenCode PR #296（binding rebase）merge → main 删 `web/src/dialogs/PopToExternal/PopToExternalDialog.tsx` L90 `overrideEnv: null`（真实 ts-rs binding `ExternalTerminalLaunchRequest` 无此字段 · typecheck 强制要求删）
- Cursor PR #297（vitest unskip）push 前未 fetch · `web/tests/dialogs/PopToExternal/PopToExternalDialog.test.tsx` L96 expected 仍含 `overrideEnv: null,`
- GitHub auto-merge 成功（test vs source 文件域不交叠 · git 无冲突）· 但 `vitest run` 实跑测试 fail（expected vs actual mismatch）
- 主 agent fix-up commit `ce08c7f` 删 1 行 · 浪费 ~5min · 但留 stale base 教训

### 反模式

| 反模式                                                                | 正确做法                                                                                              |
| --------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| 派工开始建 worktree · 直到 push 都不 fetch main                       | push 前必跑 `git fetch origin && git rebase origin/main`                                              |
| PR body raw output 用 dispatch 开始时的 base 跑 · 不更新到 final base | 必须 rebase 之后再跑 gate · 输出贴 rebase 之后的 raw output                                           |
| 假设"文件域不交叠 = 无冲突 = 不需要 rebase"                           | 文件域不交叠保证 git auto-merge 成功 · 但**不保证**语义正确（如 binding 接口改 → 测试 expected 失配） |
| 主 agent dispatch 时不在 prompt §2 列 fetch+rebase 步骤               | dispatch prompt §交付要求段必须显式含 fetch+rebase+重跑 gate 段                                       |

### 关联

- [全局] `~/.claude/rules/16-multi-agent-worktree-sync.md` · 多 agent worktree 同步通用规则（本 §2.15 是其在 ≥ 3-agent stale base 场景的具体落地）
- [项目] `dispatch-prompt-template.md §2.4` · 独立 worktree（base 隔离的前提）· 本 §2.15 是 base 同步的后置要求
- 事件：2026-05-13 · session 30 · Cursor PR #297 stale base · 主 agent fix-up `ce08c7f`（1 行 fix · `overrideEnv: null` 删除）

---

<a id="ev-4"></a>

## §4 参考实现

### 4.1 · 推荐参考（session 12 验证成功 · 高可复用）

- `spike-tmp/dispatch/MVP-04-storage-prep-opencode-prompt.md`（2026-04-20 · MVP phased · OpenCode 第 2 次 recover 成功 · 36 单元测试 + ts-rs 5 bindings · 最完整 MVP 拆分 prompt 范本）
- `spike-tmp/dispatch/MVP-07-kimi-prompt.md`（2026-04-20 · **Kimi 远程 API 标杆** · 335 行 · 附 spec 原文 140 行 + 双路径兼容 · 解决本地 CLI 模板复制给远程 API 失败的根因）
- `spike-tmp/dispatch/SPIKE-06-pr2-codex-prompt.md`（2026-04-20 · Codex CLI · 36 脱敏样本 + R1 保留独立 section · 最完整 Spike 4 样齐全 prompt）
- `spike-tmp/dispatch/MVP-05-kimi-prompt.md`（2026-04-20 · Kimi 第 5 次 · Pane 分屏 §H 布局模型约束 · 14 min 最快交付）
- `spike-tmp/dispatch/MVP-02-opencode-prompt.md`（2026-04-19 · 第一个应用硬约束 + 禁止清单的完整模板 · session 10 后 2.8 子进程清理增补）

### 4.2 · 历史对照（反面教材 · 避免重踩）

- `spike-tmp/dispatch/SPIKE-04.5-a3-opencode-prompt.md`（2026-04-19 · 第一次踩"自行 accept"坑 · OpenCode 绕过 benchmark 自标 Arbiter 选定 · 触发 §2.1 规则化）
- `spike-tmp/dispatch/SPIKE-05.5-codex-prompt.md`（2026-04-19 · 未含硬约束段 · "重构前对照"参考 · 对比 §2 成型前后的完整度）

### 4.3 · 参考选择指南

> ⚠️ 本表也复制在规范正文 §4（写 dispatch 时直接用）· 此处为完整 §4 的一部分保留。

| 任务类型                           | 推荐模板                                 | 原因                                                        |
| ---------------------------------- | ---------------------------------------- | ----------------------------------------------------------- |
| MVP 实施（有多 phase）             | `MVP-04-storage-prep-opencode-prompt.md` | Phase A 拆分 + 测试覆盖率要求 + ts-rs bindings 要求齐全     |
| MVP / Spec review（Kimi 远程 API） | `MVP-07-kimi-prompt.md`                  | 双路径兼容 + 附 spec 原文 + §G ts-rs contract + §H 决策锁定 |
| Spike（decision-grade · 4 样齐全） | `SPIKE-06-pr2-codex-prompt.md`           | 最严格 artifact 归档 + raw 溯源 + R1 保留独立 section       |
| Chore / 文档 · 本地 CLI agent      | `MVP-02-opencode-prompt.md`              | 简洁 + 8 硬约束 + 禁止清单                                  |

---

<a id="ev-5"></a>

## §5 规则来源时间线（15 条硬约束的事件溯源）

目前 15 条硬约束来自实际事件：

- 2.1-2.7（session 9 末初版）· 反映 OpenCode SPIKE-04.5/MVP-02 的 2 次违规教训
- 2.8（session 10 末增补）· 反映 MVP-02 运行时 OpenCode 未 kill Vite/pnpm 子进程 · 残留 4 小时占 port 1420 的教训
- 2.9（session 12 增补）· Agent 能力矩阵 · MVP-07 Kimi 远程 API 适配需 spec inline
- 2.10（session 25 增补 · session 26 升级）· OpenCode §2.10 evidence-based · PR #252/#262/#292 三次实证
- 2.11（session 14 增补）· PR #82/#86 PTY 跨平台 timing · Linux-only ignore + 技术债记录
- 2.12（session 14 增补 · session 28 §2.5.1 升级根治）· 跨 agent author 污染 · 3+6 次实证 · worktreeConfig 解决
- 2.13（session 20 增补）· PR #157 round 1 ADR-015 倒退 · 索引同步禁 inline 已被其他 PR 改的源文件
- 2.14（session 20 增补）· PR #159/#161/#163 三 critical bug · GUI/IPC PR reviewer 必须启 dev mode
- 2.15（session 30 增补 · 2026-05-13）· Cursor PR #297 stale base · 4-agent 派工 push 前必 fetch+rebase+重跑 gate

未来若 Codex / 其他 agent 触发新的协作 failure mode · 规范正文 §5 追加新条款 · 本时间线同步追加一行。

---

<a id="ev-related"></a>

## 关联事件记录

- 2026-04-19 · PR #34 · OpenCode SPIKE-04.5 §A.3 · 绕过 benchmark · 自行标 Arbiter 选定 · Arbiter 事后补档 approve
- 2026-04-19 · session 9 末 · OpenCode MVP-02 · 绕过独立 worktree · 主 agent 主 working tree 脏 · Option 1 处理恢复

---

> 本附录由规范正文 [`.claude/rules/dispatch-prompt-template.md`](../.claude/rules/dispatch-prompt-template.md) 拆分而来 · 内容逐字保真 · 演进时与规范正文同步追加（新条款的事件 / 反模式落本文件 · 规则正文落规范文件）。
