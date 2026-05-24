# Dispatch Prompt 模板规则 · Vibestation 专属

> 本规则沉淀给**外部 agent（Codex / OpenCode / Cursor / Droid / Kimi / 未来的 Claude 实例 / 其他工具）**下发任务 prompt 的硬约束 + 建议区分。凡触发"我要发 dispatch prompt 给外部 agent"前 · 先读本规则 · 按模板写。
>
> **触发条件**：主 agent 要把 task（Spike / MVP / Feature / Doc）通过用户转发下发给外部 agent 执行 · 非主 agent 自己执行的任务。
>
> **关联全局规则**：`~/.claude/rules/13-cross-agent-delivery.md` · `~/.claude/rules/15-runtime-verification-gate.md`
>
> 📎 **审计附录**：每条硬约束的完整「事件」叙述 / 完整「反模式」对照表 / 详细「为什么」 / 参考实现历史 / 16 条来源时间线 → [`docs/internal/dispatch-incidents.md`](../../docs/internal/dispatch-incidents.md)（**不进 auto-load · 进 git** · 写 dispatch 时按需查 · 拆分依据见该文件头）。本正文保留**规则 + 具体做法 + code + 速查表 + 模板**，照此即可写合规 prompt，无需翻附录。

---

## 0 · 目录 · 16 条硬约束速查

### §2 硬约束速查表

| 条款                                                                                                                         | 一句话约束                                                                           | 事件源（session）                 |
| ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | --------------------------------- |
| [2.1 禁自行 accept decision](#21--禁止自行-accept-decision-grade-结论)                                                       | 不动 CLAUDE.md / ADR / spec status · 只能建议                                        | OpenCode SPIKE-04.5（s9）         |
| [2.2 Acceptance 全覆盖](#22--acceptance-全覆盖不得简化)                                                                      | spec checkbox 必须逐项 `[x]` 或 explicit skip                                        | s9 初版                           |
| [2.3 Runtime 证据必交](#23--runtime-证据必交按-task-层级区分)                                                                | Spike → 4 样齐全 · MVP → 3 张截图 · chore/docs → CI 即可                             | ADR-011（s10）                    |
| [2.4 独立 worktree](#24--独立-worktree--不得在主-working-tree-开-agent-任务分支)                                             | `git worktree add /private/tmp/<task-id>-work` · 不开主 working tree                 | OpenCode MVP-02（s10）            |
| [2.5 Commit 身份](#25--commit-身份标识3-条硬约束--缺一即-block-merge)                                                        | 3 铁律：(a) `--worktree` config + (b) trailer + (c) verify                           | s14 + s28 6+ 次实证               |
| [2.6 分支命名](#26--分支命名规范)                                                                                            | `feat/<id>` · `spike/<id>` · `fix/<scope>` · `docs/<topic>` · `chore/<scope>`        | s9 初版                           |
| [2.7 不碰 decision files](#27--不碰-decision-files除非明确授权)                                                              | 默认禁 CLAUDE.md / ADR / 其他 spec · 必须明示授权                                    | s9 初版                           |
| [2.8 子进程清理](#28--子进程清理--任务结束前必须-kill-所有启动的后台进程)                                                    | dev server / Vite / PTY task 结束前 kill · 防 port orphan                            | OpenCode MVP-02（s10）            |
| [2.9 Agent 能力矩阵](#29--agent-能力矩阵--本地-agent-vs-远程-api-agent-适配)                                                 | 本地 CLI / 远程 API / IDE 插件分三类适配 prompt 结构                                 | Kimi MVP-07（s12）                |
| [2.10 lint + raw output](#210--gui--前端-task-必须跑-pnpm-lintdont-只-typecheck)                                             | 前端 task 跑 `pnpm lint` + `pnpm typecheck` + raw output 三段全贴                    | OpenCode N=3（s25-26）            |
| [2.11 Cross-platform timeout](#211--timing-sensitive-跨平台测试--timeout-必须--本地最大运行时长--2--或-linux-only-ignore)    | timeout ≥ 本地最大 × 2 · 或 Linux-only ignore + 技术债                               | Codex PR #82（s14）               |
| [2.12 git config unset](#212--主-agent-在别人-worktree-操作-git-config-后必须-unset防跨-agent-author-污染)                   | worktree 后 unset · 主 repo 不留 local config 污染                                   | s14 + s28 6+ 次实证               |
| [2.13 索引同步禁 inline](#213--索引同步类-prompt-禁止-inline-已被其他-pr-修改的源文件)                                       | 用 `git checkout origin/main` 拿真相 · 不 inline 原文                                | Kimi U2 ADR-015（s20）            |
| [2.14 Reviewer dev mode](#214--reviewer-必须启-dev-模式跑-critical-ux-path-gui--ipc-类-pr--不只看-rust-测试--ts-rs-contract) | GUI / IPC 类 PR · reviewer 启 dev mode 跑 critical UX path                           | PR #159/#161/#163（s20）          |
| [2.15 stale base race](#215--并发派工--push-前必须-fetch--rebase-main--重跑-gate防-stale-base-race)                          | ≥ 3-agent 并发 · push 前必 fetch + rebase main + 重跑 gate                           | Cursor PR #297（s30）             |
| [2.16 codegen/contract carve-out](#216--codegen-产物--共享-contract-shape-文件域-carve-out)                                  | 文件域隔离须显式说明 codegen 产物归属(Rust PR 提交) + 共享 contract shape resolution | MVP-18 #345/#351/#354/#353（s32） |

---

## 1 · 核心原则 · 硬约束 vs 建议 必须显式区分

Dispatch prompt 里的协作要求 **必须区分**：

- **硬约束（Hard Constraint · 必须遵守 · 违反即 BLOCK PR merge）**：用 "必须 / 不得 / 禁止 / ❌ / 硬要求" 等强硬措辞
- **建议（Recommendation · 可选 · 不违反但不理想）**：用 "建议 / 推荐 / 最好 / 强烈建议" 等柔性措辞

硬约束和建议**物理分段**（用分隔线或独立 section · 见 §3.1 模板的「🔴 硬约束」段格式）· 不混在一起写。外部 agent 会倾向走最短路径 · "建议"级条款容易被绕过 · 对**不能绕过**的要求 · 必须升级为"硬约束"措辞。

> 📎 反模式（SPIKE-04.5 / MVP-02 两次踩坑详例）→ [dispatch-incidents.md §1](../../docs/internal/dispatch-incidents.md#ev-1)

---

## 2 · 默认硬约束清单（所有 dispatch 必含）

除非任务性质明确豁免（需 prompt 中说明豁免理由）· 以下 16 条默认是硬约束（详见 [§0 速查表](#0--目录--16-条硬约束速查)）：

### 2.1 · 禁止自行 accept decision-grade 结论

**禁止**：外部 agent 自行修改以下任一：

- `CLAUDE.md` 决策表（A/B/C 三档任一）
- `docs/adr/ADR-*.md` 的 status 字段（proposed → accepted / superseded）
- `docs/tasks/*.md` frontmatter 的 `status` 字段（draft → ready → in-progress → done）
- 任何 spec 声称 "Arbiter 选定 X" · "Arbiter 同意 Y"

**允许**：外部 agent 可以**建议**（"建议方案 a · 理由 ..."）· 但最终 accept 只能是 Arbiter（项目所有者）在 PR comment 明确 approve 后生效。

> 📎 事件（SPIKE-04.5 §A.3 OpenCode 自标 Arbiter 选定）→ [dispatch-incidents.md §2.1](../../docs/internal/dispatch-incidents.md#ev-2-1)

### 2.2 · Acceptance 全覆盖（不得简化）

spec 的 `Acceptance` 所有 checkbox **必须** 在 PR body 逐项：

- 勾 `[x]` 已完成
- 或 `[ ]` + explicit skip reason（例："跳过 · 依赖 MVP-03 · 本 PR 范围外"）

不得整段 skip · 不得声称 "大致完成"。

### 2.3 · Runtime 证据必交（按 task 层级区分）

> ⚠️ **2026-05-20 · MVP capture mandate removed**（ADR-023 supersede ADR-011）：MVP 类「3 张截图 + 30s 录屏」硬要求已 supersede · 不再阻塞 spec done flip。已捕证据继续保留作 ship audit · 但**新 MVP PR 不强制 capture**。仅 Spike 类仍按 `spike-delivery-checklist.md` 4 样齐全。

| 任务类型                              | Runtime 证据要求                                                                                                                           |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| **Spike**（decision-grade benchmark） | 按 `.claude/rules/spike-delivery-checklist.md` "4 样齐全"（report + code + raw + cold backup）· report 数字必须 raw 可溯源                 |
| **MVP**（产品功能）                   | 代码侧 acceptance（cargo test / vitest / Criterion bench / 性能 DevTools 数字）即可 · 截图 / 录屏 / GUI capture 已 supersede（2026-05-20） |
| **Docs / chore**（纯文档）            | CI 通过即可 · 无 runtime 要求                                                                                                              |

关键：**CI 绿 ≠ runtime 过**（见 `~/.claude/rules/15-runtime-verification-gate.md`）· GUI / IPC 类代码 reviewer 仍可按 §2.14 启 dev mode 自验 critical UX path · 但不强制 capture 截图归档。

### 2.4 · 独立 worktree · 不得在主 working tree 开 agent 任务分支

外部 agent **必须** 用独立 git worktree：

```bash
# OpenCode / Codex / 其他 · 下发后第一步
cd /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation
git fetch origin
git worktree add /private/tmp/<task-id>-work <branch-name>
cd /private/tmp/<task-id>-work
# 之后所有操作在此目录
```

**禁止**：

- 在主目录 `git checkout -b feat/X` 开 agent 任务分支（会干扰主 agent 的 working tree · rule 13 踩坑）
- 共享同一目录做并发 checkout

> 📎 事件（MVP-02 OpenCode 主 working tree 脏阻塞主 agent）→ [dispatch-incidents.md §2.4](../../docs/internal/dispatch-incidents.md#ev-2-4)

### 2.5 · Commit 身份标识（3 条硬约束 · 缺一即 BLOCK merge）

**必须** 3 条全做 · 不得只做 trailer 而跳过 git config：

#### 2.5.1 · worktree-local git config（worktree 创建后第一步 · session 28 升级 · 防 §2.12 污染）

```bash
cd /private/tmp/<task-id>-work

# (a) 启用 worktree-local config 支持（idempotent · 重复跑安全）
#     主 repo .git/config 多写一行 extensions.worktreeConfig=true · 不污染 identity
git config extensions.worktreeConfig true

# (b) 设 worktree-local identity（写 .git/worktrees/<name>/config.worktree · 不污染主 repo）
git config --worktree user.name "<Agent Name>"           # 例 "Codex CLI"
git config --worktree user.email "<vendor>@<vendor>.ai"  # 例 "noreply@openai.com"

# (c) 验证（必须显示 worktree-local 值 · 不是 global / main repo .git/config 值）
git config user.name && git config user.email
```

**为什么 `--worktree`（一句话）**：`git worktree add` 的 worktree **共享主 repo `.git/config`**，无 `--worktree` flag 的 `git config user.email` 会污染同 repo 所有 worktree + 主 agent commit（session 28 实证复发 3 次）· 启用 `extensions.worktreeConfig=true` + `--worktree` 才真正 per-worktree 隔离。

> 📎 详细为什么（4 点）/ 反模式表 / PR #71 事件 → [dispatch-incidents.md §2.5](../../docs/internal/dispatch-incidents.md#ev-2-5)

#### 2.5.2 · 每个 commit 必含 `Co-authored-by` trailer

```
<type>(<scope>): <中文描述>

Co-authored-by: <Agent Name> <noreply@<vendor>.ai>
```

标识列表：

- Claude Code：`Co-authored-by: Claude Code <noreply@anthropic.com>`
- Codex CLI：`Co-authored-by: Codex CLI <noreply@openai.com>`
- OpenCode：`Co-authored-by: OpenCode <noreply@opencode.ai>`
- Kimi（Moonshot）：`Co-authored-by: Kimi <noreply@moonshot.ai>`
- Droid（Factory.ai）：`Co-authored-by: Droid <noreply@factory.ai>`（session 25-26 实战 · PR #260）
- Cursor：`Co-authored-by: Cursor <noreply@cursor.com>`（session 28 实战 · PR #273）
- Aider / Windsurf / 其他：按工具官方邮箱

#### 2.5.3 · commit 后立即验证 author 字段

```bash
git log -1 --pretty=format:"%an <%ae>"
# 必须显示 "<Agent Name> <noreply@<vendor>.ai>"
# 若显示其他 agent（如 "Kimi"）· 立即 git commit --amend --reset-author
```

### 2.6 · 分支命名规范

按任务类型固定前缀（匹配 CLAUDE.md "Commit 规范"）：

| 类型          | 前缀            | 例子                                  |
| ------------- | --------------- | ------------------------------------- |
| Spike         | `spike/<id>`    | `spike/SPIKE-05.5` · `spike/SPIKE-06` |
| MVP / Feature | `feat/<id>`     | `feat/MVP-02-workspace-management`    |
| Bug fix       | `fix/<scope>`   | `fix/tauri-acl-deny`                  |
| Docs          | `docs/<topic>`  | `docs/dispatch-prompt-template`       |
| Chore / CI    | `chore/<scope>` | `chore/corepack-ci-migration`         |

**禁止**：直接在 main 开工 · 或用其他命名模式。

### 2.7 · 不碰 decision files（除非明确授权）

外部 agent **默认禁止**修改：

- `CLAUDE.md`（决策表 · 禁区 · 风格）
- `docs/adr/*.md`（除非 task 明确是"新开 ADR-NNN"）
- 其他 spec（`docs/tasks/*.md` 除自己认领的那个）
- `~/.claude/rules/*`（全局规则）
- `.claude/rules/*`（项目规则 · 除非 task 明确是"新增规则"）

若任务需要碰这些 · 必须在 dispatch prompt 里**明确授权**并**列出具体文件**。

### 2.8 · 子进程清理 · 任务结束前必须 kill 所有启动的后台进程

外部 agent 在任务过程中启动的任何后台进程（`pnpm tauri:dev` · Vite dev server · PTY · Python 自动化脚本 · 自建 daemon · `cargo run` 后台 build 等）· **必须** 在任务结束前显式 kill · 不得残留到 main agent 后续操作。

**强制做法**（二选一）：

```bash
# (a) 推荐 · trap 自动 cleanup
cleanup() {
  pkill -f "tauri dev" 2>/dev/null || true
  pkill -f "vite" 2>/dev/null || true
  # ...列出所有本 task 启动的进程特征
}
trap cleanup EXIT INT TERM
# ... 任务主逻辑 ...

# (b) 任务末尾手动 kill + 用 lsof/ps 验证
pkill -f "tauri dev"
sleep 2
lsof -iTCP:1420 -sTCP:LISTEN && echo "⚠ port 1420 still in use" || echo "✓ clean"
```

**禁止**：

- 任务结束不 kill · 让 main agent 自己遇到 "port in use" 再排查（rule 13 踩坑）
- 只 kill 父进程 · 子进程 orphan（用 `pkill -f <name>` 按模式 kill · 或 process group）
- 假设 session 结束会自动清理 · 实际 detached 进程会留到 4+ 小时

> 📎 事件（MVP-02 OpenCode Vite orphan 4 小时占 port 1420）→ [dispatch-incidents.md §2.8](../../docs/internal/dispatch-incidents.md#ev-2-8)

### 2.9 · Agent 能力矩阵 · 本地 agent vs 远程 API agent 适配

下发 prompt 给外部 agent 时 · **目标 agent 的文件访问能力决定 prompt 结构**。盲目复用模板会在远程 API agent 上失败（agent 拿到路径但读不到文件）。

#### 三类 agent 对照

| Agent 类型   | 代表                                                                                             | 本地文件           | git/shell   | prompt 策略                                                |
| ------------ | ------------------------------------------------------------------------------------------------ | ------------------ | ----------- | ---------------------------------------------------------- |
| **本地 CLI** | Codex CLI · OpenCode · Claude Code · `cursor-agent` CLI · Aider · Windsurf · Droid（Factory.ai） | ✅ worktree + Bash | ✅ 完整     | **给路径即可** · 默认端到端 commit + push + PR             |
| **远程 API** | Kimi（Moonshot） · Claude API · OpenAI API · Gemini · DeepSeek                                   | ❌ 无本地          | ❌ 无 shell | **必须附文件原文**                                         |
| **IDE 插件** | Trae / Kilo / **Cursor IDE 内嵌 chat** · Copilot Chat                                            | 🟡 依赖插件        | 🟡 部分     | **明确工具要求 + 附原文兜底 + 显式"完工 = PR 链接"硬约束** |

#### 强制做法

**dispatch prompt 顶部 meta 段必须含**：

```markdown
> **执行者**：<Agent Name>
> **Agent 类型**：<本地 CLI | 远程 API | IDE 插件>
```

**远程 API agent 的 prompt 必须自包含**：

1. 所有需要审查 / 修改的文件原文 · 用 `---BEGIN SPEC---` / `---END SPEC---`（或类似分隔标记）包裹 · 贴进 prompt
2. 所有需要参考的上下文（其他 spec / ADR / 决策表）· 提炼关键段贴进 prompt · 不能只说 "参见 CLAUDE.md #13"
3. **输出路径双路径兼容**：
   - 若 agent 有本地 git / worktree 能力 → 按流程 commit + push + 开 PR
   - 若无本地 access → 输出完整修改后文件全文 · 用户粘到本机

**本地 CLI agent 的 prompt 可以只给路径**（能通过 worktree + Bash 读）· 但依然必须明确：

- 独立 worktree 路径（硬约束 2.4）
- 分支名（硬约束 2.6）
- commit trailer（硬约束 2.5）

**IDE 插件类专项约束**（Cursor IDE 内嵌 / Copilot Chat / Trae / Kilo · session 31 PR #313 实证升级）：IDE 插件类有 git/shell 但**默认 = 写完停下让 user 确认 commit**· 不自动跑端到端。dispatch prompt **必须**显式加：

1. **完工标志硬约束**：`完工 = PR 链接生成 · 不允许停下问 user "是否要 commit/push/PR"`· 写在硬约束段独立列
2. **端到端步骤硬要求**：`git add + commit + fetch + rebase origin/main + push -u + gh pr create` 全跑 · 缺一即未完工
3. **反模式禁示**：若该 agent 之前 dispatch 出现"停下问 user"· prompt 顶部独立段 ⚠️ 警示"本次禁止重演"· 引上次 PR# 为证

**Cursor 双形态区分**（硬约束）：`cursor-agent` CLI（命令行）归"本地 CLI"行 · 默认端到端；**Cursor IDE 内嵌 chat**（编辑器内置 AI）归"IDE 插件"行 · 必须显式完工硬约束。派发 Cursor **必须先询问 user 用哪种形态** · 或默认按"IDE 插件"行处理（安全 fallback）· 不能两形态共用同一 prompt。

> 📎 反模式表 / MVP-07 Kimi 首次踩坑 / Cursor IDE PR #313 踩坑 / 关联 → [dispatch-incidents.md §2.9](../../docs/internal/dispatch-incidents.md#ev-2-9)

### 2.10 · GUI / 前端 task 必须跑 `pnpm lint` · 文档 task 必须显式跑 markdown prettier check

Dispatch prompt 若涉及前端代码（`web/src/**` 或任何 SolidJS / React 组件 / CSS / TypeScript 文件）· §Acceptance 必须含两条：

- [ ] `pnpm lint` 本地跑过（预期 `Checking formatting... All matched files use Prettier code style!`）
- [ ] `pnpm typecheck` 本地跑过（预期 `tsc --noEmit` 0 errors）

Dispatch prompt 若涉及 markdown 文档详化 / archive / spec 改动（任何 `docs/**/*.md`）· §Acceptance 必须含一条：

- [ ] `npx prettier --check <markdown-file>` 本地跑过（**不是 `pnpm lint`**）

**缺任一 · BLOCK PR merge · 不是建议**。raw output 全贴 · 不接受"过了"口头转述。

**为什么**：CI 的 `Frontend · pnpm lint + typecheck` 跑 `pnpm lint`（prettier --check）+ `pnpm typecheck`（tsc --noEmit）· 只做 typecheck 本地 pass 但 CI fail。**`pnpm lint` ≠ markdown prettier check**：`pnpm lint` scope = `web/src/**/*.{ts,tsx,css}` + `web/index.html` · **不含 `docs/**/\*.md`** · markdown 文档 task 必须显式 `npx prettier --check <md>` 才真验到 · 否则主 agent review 跑 prettier 必 fail 触发 fix-up（session 31 OpenCode N=4 understanding gap 实证）。

> 📎 PR #83 / PR #311 事件 / 反模式表 → [dispatch-incidents.md §2.10](../../docs/internal/dispatch-incidents.md#ev-2-10)

### 2.11 · Timing-sensitive 跨平台测试 · timeout 必须 ≥ 本地最大运行时长 × 2 · 或 Linux-only ignore

涉及以下任一的测试 · dispatch prompt §Acceptance 必须含 timeout / 平台 gate 说明：

- PTY / tty · signal 传递 · 子进程 spawn/kill
- socket 连接 / HTTP request / 网络延迟相关
- 文件 I/O 压测 · 大量 fd 操作
- 依赖 kqueue / epoll / ETW 事件的异步等待

**具体**：

1. 测试内 timeout 设置 · 至少 **本地 macOS 最大观察时长的 2 倍**（例：本地 1s 内 · CI 给 5s 以上 · 本地 5s · CI 给 ≥ 10s）· 且必须明确注释 "本地 vs CI margin"
2. 若本地 macOS 跑 · 无法在 dispatch 期间验证 Linux CI · 显式标：
   ```rust
   #[cfg_attr(target_os = "linux", ignore = "<根因> · 本地 macOS 稳定 · Linux CI timing/语义待 Phase X 深挖")]
   ```
   配套在对应 MVP spec §已知风险 段加条目 + 明确 GA gate 解除 ignore 的触发条件
3. 禁止反复加 timeout 作为"症状治疗"· 如果 timeout × 2 仍 fail · 根因是 **语义差异**（非 timing）· 立即切 Linux-only ignore + 技术债记录 · 避免陷入 timeout 扩张循环

**为什么（一句话）**：本地 macOS（kqueue）vs Linux CI（epoll）对 PTY close event / signal / `waitpid` / `tcgetpgrp` 语义可能有差异 · 纯 timeout 扩张不解决语义问题。

> 📎 详细为什么 / PR #82/#86 事件 / 反模式表 → [dispatch-incidents.md §2.11](../../docs/internal/dispatch-incidents.md#ev-2-11)

### 2.12 · 主 agent 在别人 worktree 操作 git config 后必须 unset（防跨 agent author 污染）

主 agent 在**非主 repo** 的 worktree（如 `/private/tmp/<agent>-work`）临时切 git config user.name / user.email（为代 commit 某 agent）· 任务完成后必须：

```bash
cd /private/tmp/<agent>-work
git config --worktree --unset user.email   # 若之前用 --worktree 设过
git config --worktree --unset user.name
# 同时清 fallback path（防 session 28 前老 prompt 留下的污染）
git config --unset user.email 2>/dev/null || true
git config --unset user.name 2>/dev/null || true
# 验证
git config user.email   # 应该显示 global config · 不是临时值
```

**主 repo 本身**（用户工作的 repo）· 主 agent **不应**改 local config。如果 debug 时改过 · 必须立即 unset 回 global：

```bash
cd <project-root>
git config --local --unset user.email 2>/dev/null || true
git config --local --unset user.name 2>/dev/null || true
```

**验证触发**：每次 `git commit` 后 · 跑 `git log -1 --pretty=format:"%an <%ae>"` 确认 author 归属正确（硬约束 2.5.3 已有）· 若发现错归 · 立即 `git commit --amend --reset-author --no-edit`。

**根治方案**（session 28 起 §2.5.1 强制 · 见 §2.5.1）：

```bash
git config extensions.worktreeConfig true       # 启用 per-worktree config
git config --worktree user.name "<Agent Name>"  # 写 .git/worktrees/<name>/config.worktree
git config --worktree user.email "..."
```

`--worktree` 写入位置不再共享 · 跨 agent 污染从根上消除。

> 📎 详细为什么 / session 14 + 28 共 6 次污染事件 / 反模式表 → [dispatch-incidents.md §2.12](../../docs/internal/dispatch-incidents.md#ev-2-12)

### 2.13 · 索引同步类 prompt 禁止 inline 已被其他 PR 修改的源文件

下发 "文档索引同步 / 跨文件状态对齐 / README rollup" 类任务时（如 ADR 索引同步 / `tasks/README.md` 状态翻转 / PROGRESS 滚动窗口整理）· dispatch prompt **禁止**把"目标 agent 即将引用的源文件原文"inline 到 prompt 里 · 必须改用 `git checkout origin/main -- <file>` 字节级恢复指令。

**适用场景**（同时满足两条 · 必须 inline → checkout 重写）：

1. 该文件在另一条已 merge / 即将 merge 的 PR 里被改过（HEAD 状态和 prompt 起草时的 working copy **不同**）
2. 目标 agent 不会主动 `git pull` / `git fetch` 后基于 HEAD 操作（远程 API agent · 或 prompt 起草到执行间隔较长 · 或 agent 习惯按 prompt inline 字面执行）

**最小正确做法**（prompt 里这样写，而非 inline 原文）：

```bash
git fetch origin
git checkout origin/main -- docs/adr/ADR-NNN-foo.md   # 字节级恢复本体 · 保留最新措辞
# 然后只补需要新增的索引条目（在其他文件 · 用 sed / git apply / grep 锚点定位）
```

> 📎 完整正确/错误示范块 / 为什么 / PR #157 round 1 倒退事件 / 反模式表 / 关联 → [dispatch-incidents.md §2.13](../../docs/internal/dispatch-incidents.md#ev-2-13)

### 2.14 · Reviewer 必须启 dev 模式跑 critical UX path（GUI / IPC 类 PR · 不只看 Rust 测试 + ts-rs contract）

GUI / IPC 类 PR（含 webview 组件 / dialog / modal / Tauri command）· reviewer 在 approve 前**必须**启 `pnpm tauri:dev` 跑一遍 PR 涉及的 critical UX path · 不能只看 Rust 单元测试 + IPC contract 通过就 approve。

**触发条件**（任一）：

- PR 改 `web/src/dialogs/` · `web/src/panels/` · `web/src/components/`
- PR 加 / 改 / 删 Tauri `#[tauri::command]` IPC handler
- PR 加 / 改 frontend reactive store（settings / theme / layout / pane state）
- PR 改 `crates/app/permissions/` · `crates/app/capabilities/`
- spec acceptance 含 "首次启动 modal 阻塞" / "切换实时生效" / "用户操作 X 后 Y 变化" 类 UX 流程

**强制做法**（reviewer 必须）：

1. 本地 `git checkout <pr-branch> && git pull`
2. 必要时**删 DB 模拟首次启动**（`rm "$HOME/Library/Application Support/com.vibestation.app/vibestation.db"`）· 触发 fresh state path
3. `pnpm tauri:dev` · 等窗口 ready
4. 跑 spec critical UX path（modal 显示 / radio click / dialog open / shortcut 触发 / fs watch 刷新）
5. 观察 UI 实际行为 · 不只是 DB / IPC 是否被调
6. PR review 在 GitHub 留下 "runtime verified · path X / Y / Z OK" comment

**禁止做法**：

- ❌ 仅基于 `cargo test` / `pnpm typecheck` / `pnpm lint` 通过 → approve
- ❌ 仅基于 IPC contract（ts-rs binding）一致 → approve
- ❌ 仅 `cargo build` / `pnpm build` 产物生成 → approve
- ❌ 假设 "frontend 改动小 · CSS 应该没事 · 不用看 UI" → approve（**PR #159 反面案例**）

> 📎 PR #159/#161/#163 三 bug 事件表 / 根因 / 反模式表 / 关联 → [dispatch-incidents.md §2.14](../../docs/internal/dispatch-incidents.md#ev-2-14)

### 2.15 · 并发派工 · push 前必须 fetch + rebase main + 重跑 gate（防 stale base race）

≥ 3-agent 并发派工时 · 各 agent 的 worktree base 在工作过程中 main 可能被其他 PR merge 推进。**push 前 + 最终 gate 验证前必须 fetch + rebase main + 重跑 gate** · 否则 PR 看起来合规但 merge 后破 main。

**具体执行**（dispatch prompt §交付要求段必须含）：

```bash
# 最终 commit 完成后 · push 前 · 必跑
git fetch origin
git rebase origin/main   # 或 git merge --no-ff origin/main · 看 base 策略
# 若有冲突 · 解决（文件域已隔离时通常无 git 冲突）
# 若 source 改动影响测试 / typecheck / lint · 必须更新对应文件

# 重跑 gate 验证（rebase 后的实际状态）
pnpm lint && pnpm typecheck && pnpm vitest run
cargo test --workspace  # 若涉及 Rust
echo "exit code: $?"
```

rebase 后 gate fail · **必须先修 · 再 push** · 不允许带 stale base 状态 push PR。

**适用范围**：

- ✅ ≥ 3-agent 并发派工
- ✅ 单 agent dispatch 但其他 agent 同时活跃在相邻文件域
- ⚠️ 单 agent dispatch + 无其他活跃 PR · 可豁免（base 不会变）· 但仍建议 fetch verify

> 📎 为什么（main_A/main_B 时序）/ PR #297 stale base 事件 / 反模式表 / 关联 → [dispatch-incidents.md §2.15](../../docs/internal/dispatch-incidents.md#ev-2-15)

---

### 2.16 · codegen 产物 + 共享 contract shape · 文件域 carve-out（防 codegen 漏提交 / shape 静默错形）

多 agent 文件域隔离派工（如 Codex=Rust · Cursor=web）· dispatch prompt 涉及 **build.rs/ts-rs codegen** 或 **store/seam 切 canonical binding** 时 · 必须显式说明两类跨域共享物 · 否则执行 agent 在矛盾/缺失指令下做本地正确但全局不合规选择 → §2.14 BLOCK（MVP-18 已 4 例同根）。

**必须显式包含**：

1. **codegen 产物归属**：若 task 改 build.rs/ts-rs 触发器 · 明写"生成的 binding/产物（含 `web/src/bindings/*.ts` + `index.ts`）由生成它的 Rust 侧 PR 提交 · 是 codegen 产物非 web agent 手写域 · 参 #344 先例"· **不能笼统"绝不改 web/"**（会把 codegen 产物误锁出 Rust agent 域 → 没人提交 → main 不一致 + 下游断链）
2. **验证措辞**：要"提交"就写"git add + commit 这些 .ts + git ls-files 证 tracked"· **不能只写"ls 证生成"**（ls 是 gate 输出 ≠ 交付 · #354 踩坑）
3. **shared-contract shape resolution**：store/seam 切 canonical binding 时 · 明指**用哪个形状的 binding**（如 DB-row `PaneLink` vs event `PaneLinkedEvent`）· 形状不匹配时明写"派生 local view-model 保留 enum 富信息 · 不坍缩"（#353 踩坑：盲切 DB-row binding 把 status enum 坍缩成 bool · typecheck 过但语义静默错形）
4. **主 agent review-prep 预判前置**：主 agent §2.14 review-prep 若已预判某形状/归属坑 · 必须把该预判写进 dispatch prompt · 不能只留内部 prep（#353：prep 预言"不盲换 @/bindings/PaneLink"但只留内部 · Cursor 没看到）

**适用范围**：

- ✅ 任何改 build.rs/ts-rs export / codegen 触发器的 task（codegen 产物归属条款必含）
- ✅ 任何 store/seam 切 canonical binding 的前端 task（shape resolution 必含）
- ⚠️ 纯单域 task 无 codegen / 无 contract 切换 · 不触发本条

> 📎 4 例同根事件（#345 §K.5 / #351 ADR# / #354 codegen / #353 shape）/ 反模式表 / 根因归属判则 / 关联 → [dispatch-incidents.md §2.16](../../docs/internal/dispatch-incidents.md#ev-2-16)

---

## 3 · 标准 Dispatch Prompt 模板

### 3.0 · 文件命名规范（audit M2 · 2026-04-21）

dispatch prompt 文件统一放 `spike-tmp/dispatch/`（⚠️ **`.gitignore:116` = `spike-tmp/` · 整个目录 gitignored · 不进 git · clone repo 后不可见** · 范本是本地工作产物 · 引用范本时不要给可点击 git 路径 · 见 [ADR-022](../../docs/adr/ADR-022-dispatch-template-ref-path-staleness.md) 事实修正）· 命名格式：

```
<TASK-ID>[-<phase-or-pr-suffix>]-<agent>-prompt.md
```

示例：

- 单 phase task · MVP-05 整体 → `MVP-05-kimi-prompt.md`
- 多 phase task · MVP-04 storage prep → `MVP-04-storage-prep-opencode-prompt.md`
- 分 PR Spike · SPIKE-06 pr2 → `SPIKE-06-pr2-codex-prompt.md`
- 修复 dispatch · SPIKE-06 pr2 第二轮修复 → `SPIKE-06-pr2-codex-fix-prompt.md`

**禁止**：

- 无 suffix 的歧义命名（`MVP-04-opencode.md` 不清楚是哪个 phase）
- 大小写不一致（全大写 TASK-ID · 全小写 agent 和 `-prompt` 后缀）
- 放到其他目录（如 `docs/dispatch/` · `.claude/dispatch/`）

### 3.1 · 标准模板

````markdown
# <TASK-ID> · <Agent Name> Dispatch Prompt

> **执行者**：<Agent · 例 Codex / OpenCode>
> **Dispatch 时间**：YYYY-MM-DD
> **Parent task**：[`docs/tasks/<TASK-ID>-*.md`](../../docs/tasks/<TASK-ID>-*.md) · status: ready
> **前置依赖**：<列 done 的 Spike / MVP>
> **并行任务**：<主 agent 和其他 agent 当前 track · 说明文件域隔离>

---

## 🔴 本 task 的硬约束

默认硬约束（见 `.claude/rules/dispatch-prompt-template.md` §2 · 当前 16 条）：

- [ ] 2.1 · 禁止自行 accept decision-grade
- [ ] 2.2 · Acceptance 全覆盖
- [ ] 2.3 · Runtime 证据必交
- [ ] 2.4 · 独立 worktree
- [ ] 2.5 · Commit trailer 身份（3 条铁律 · §2.5.1 worktreeConfig + §2.5.2 trailer + §2.5.3 验证）
- [ ] 2.6 · 分支命名规范
- [ ] 2.7 · 不碰 decision files
- [ ] 2.8 · 子进程清理（kill 所有启动的 dev server / 脚本）
- [ ] 2.9 · Agent 能力矩阵（本地 CLI · 远程 API · IDE 插件适配）
- [ ] 2.10 · GUI / 前端 task 必跑 `pnpm lint`（不只 typecheck）· raw output 三段全贴
- [ ] 2.11 · Timing-sensitive 跨平台测试 timeout ≥ 2× · 或 Linux-only ignore
- [ ] 2.12 · 主 agent 在别人 worktree 操作 git config 后必须 unset（防 author 污染）
- [ ] 2.13 · 索引同步类 prompt 禁止 inline 已被其他 PR 修改的源文件
- [ ] 2.14 · Reviewer 必须启 dev 模式跑 critical UX path（GUI / IPC 类 PR）
- [ ] 2.15 · ≥ 3-agent 并发派工 · push 前必 fetch + rebase main + 重跑 gate（防 stale base race）
- [ ] 2.16 · codegen 产物归属 + 共享 contract shape resolution（涉 build.rs/ts-rs 或 store 切 binding 时）

本 task 额外硬约束：

- [ ] <task-specific · 例 "benchmark 必跑 3 方案 × 3 次"· 或 "rusqlite schema 必须通过 PRAGMA user_version" 等>

本 task 豁免条款（需明确理由）：

- [ ] <如有 · 例 "本 task 是 chore 性质 · 豁免 2.3 runtime 证据 · 因为纯文档"`>

---

## 复制给 <Agent> 的 prompt（原话）

\`\`\`
<prompt 正文 · 按下述结构>

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔴 硬约束（违反即 BLOCK PR merge · 不是建议）

<从上面 16 条 + task-specific 翻译成 prompt 语言 · 简洁列出>

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 任务范围（按 spec §功能范围）

<链接 spec · 列实施步骤>

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Acceptance（严格按 spec 逐项 · 不得简化）

<链接 spec · 强调每 checkbox 必须显式状态>

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📦 交付要求

- 分支：<按 2.6 命名>
- 独立 worktree（2.4）：git worktree add /private/tmp/<task-id>-work <branch>
- Commit trailer（2.5）：<vendor 邮箱>
- Runtime 证据（2.3）：<按 task 类型列>

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

❌ 禁止清单（2.1 · 2.7 的具体化）

- 不要改 CLAUDE.md 决策表
- 不要改 ADR / 其他 spec
- 不要声称 "Arbiter 选定 X" 除非 PR 有 Arbiter 明确 comment
- 不要在 CI 绿就声称 ready

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 PR body 必含段（v2-D.1 · audit M1）

## Implemented by · Reviewed by

- Implemented by: <agent-id · 例 Codex CLI / OpenCode / Kimi>
- Reviewed by: <same agent-id · self-review 或 cross-review>
  - 单人项目 v2-D.1 模式：无 cross-agent review 合法 · 但必须显式声明
  - 例："Reviewed by: OpenCode · self-review（单人项目 v2-D.1 模式 · 无 cross-agent review · Arbiter approval 见下）"
- Arbiter approval: tajiaoyezi · YYYY-MM-DD HH:MM · "<dialogue 摘要>"

**为什么要显式声明 self-review**：防止未来新 agent 学习此 PR 当模板 · 误以为 "implementer 勾完 hard constraints 即合规" 传染到多 agent 场景。见 `docs/adr/ADR-012-v2d1-arbiter-approval-simplification.md`。

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

估时 <X>d · 完工开 PR · 主 agent 按 spec §Acceptance 逐条 review + 硬约束 16 条 check · 违反任一不得 merge。

GO 🚀
\`\`\`

---

## 给用户的转发说明

1. 复制上面 ``` 内容 · 整段发给 <Agent>
2. Agent 应建独立 worktree · commit · push · 开 PR
3. PR 开出后我按硬约束 16 条 + spec Acceptance 做 review
````

---

## 4 · 参考实现 · 选择指南

> ⚠️ **范本文件不进 git**（`.gitignore:116` = `spike-tmp/` · 整个 gitignored · clone repo 后不可见）· 历史范本仅在本地 `spike-tmp/dispatch/` + `spike-tmp/dispatch/_archived/`（本机有则可查 · 无则不可依赖）· 故本节**不给可点击范本路径** · 改为按任务类型给"该照哪个结构写"的指引（[ADR-022](../../docs/adr/ADR-022-dispatch-template-ref-path-staleness.md) accepted @ 2026-05-17 方案 d · 文档不再承诺 git 不存在的路径）。

写 dispatch 时按任务类型决定结构 · **以本文件 §3 标准模板为骨架** · 叠加下列任务类型的侧重点：

| 任务类型                            | 结构侧重（在 §3 模板基础上强化）                                          |
| ----------------------------------- | ------------------------------------------------------------------------- |
| MVP 实施（有多 phase）              | Phase 拆分 + 每 phase 测试覆盖率要求 + ts-rs bindings 清单齐全            |
| MVP / Spec review（远程 API agent） | §2.9 双路径兼容 + prompt 内附 spec 原文 + §G ts-rs contract + §H 决策锁定 |
| Spike（decision-grade · 4 样齐全）  | 最严格 artifact 归档（report+code+raw+冷备）+ raw 溯源 + R1 独立 section  |
| Chore / 文档 · 本地 CLI agent       | 简洁 + 16 硬约束全列 + 禁止清单 + §2.10 markdown prettier check           |

> 📎 范本的**特征描述**（每个历史范本"为什么是范本"的结构特征 · 不依赖文件可点击）/ 历史对照（反面教材 SPIKE-04.5 / SPIKE-05.5）→ [dispatch-incidents.md §4](../../docs/internal/dispatch-incidents.md#ev-4)（该附录已同步去断链 · 仅描述特征不给 git 路径）

---

## 5 · 本规则的演进

本规则**必须随外部 agent 实际行为演进**。规律：

- 每发现外部 agent 绕过**建议级**条款 → 把该条款升级为**硬约束**
- 每发现外部 agent 绕过**硬约束**条款 → 增加 CI 硬阻塞（如 gitleaks / required-status-check）替代 trust-based 约束

未来若 Codex / 其他 agent 触发新的协作 failure mode · 本规则追加新条款（规则正文落本文件 · 对应事件 / 反模式落附录 · 二者同步）。

> 📎 16 条硬约束的完整来源时间线（每条事件源 + session）→ [dispatch-incidents.md §5](../../docs/internal/dispatch-incidents.md#ev-5)

---

## 6 · 自审四问（本规则对自己）

- **递归完备性**：本规则自己在规则里（2.7 "不碰 .claude/rules/\*"）· 所以未来 agent 修本规则需明确授权 ✅ · 附录 `docs/internal/dispatch-incidents.md` 不在 `.claude/rules/` · 不受 2.7 约束（属普通 docs · 但演进时须与本正文同步，见 §5）✅
- **反向场景**：规则不遵守 → 第三次违规 → 触发 CI 硬阻塞升级路径（见 §5）✅
- **边界适用性**：适用所有 dispatch（Spike / MVP / chore）· chore 可豁免 2.3（明示在 prompt）· 2.8 适用所有启动后台进程的 task · 纯文档 task 不触发 ✅
- **YAGNI**：16 条都来自真实事件 / 真实风险 · 无投机条款 ✅

---

## 关联

- [全局] `~/.claude/rules/13-cross-agent-delivery.md` · 跨 agent 交付物持久化（事件源头 · rule 13 是 2.4 独立 worktree 的上位依据）
- [全局] `~/.claude/rules/15-runtime-verification-gate.md` · Runtime 验证 Gate（2.3 runtime 证据的上位依据）
- [项目] `.claude/rules/spike-delivery-checklist.md` · Spike 4 样齐全归档（2.3 Spike 层级引用）
- [项目] `.claude/rules/tauri-v2-patterns.md` · Tauri v2 ACL + CSP + capability（MVP 类 task 的具体实施规则）
- [附录] [`docs/internal/dispatch-incidents.md`](../../docs/internal/dispatch-incidents.md) · 本规则的事件档案 / 完整反模式表 / 参考实现历史 / 来源时间线（含全部「关联事件记录」）
