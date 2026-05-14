# Dispatch Prompt 模板规则 · Vibestation 专属

> 本规则沉淀给**外部 agent（Codex / OpenCode / Cursor / Droid / Kimi / 未来的 Claude 实例 / 其他工具）**下发任务 prompt 的硬约束 + 建议区分。凡触发"我要发 dispatch prompt 给外部 agent"前 · 先读本规则 · 按模板写。
>
> **触发条件**：主 agent 要把 task（Spike / MVP / Feature / Doc）通过用户转发下发给外部 agent 执行 · 非主 agent 自己执行的任务。
>
> **关联全局规则**：`~/.claude/rules/13-cross-agent-delivery.md` · `~/.claude/rules/15-runtime-verification-gate.md`

---

## 0 · 目录 · 15 条硬约束速查

> 主章节：[§1 核心原则](#1--核心原则--硬约束-vs-建议-必须显式区分) · [§2 15 条硬约束](#2--默认硬约束清单所有-dispatch-必含) · [§3 标准模板](#3--标准-dispatch-prompt-模板) · [§4 参考实现](#4--参考实现) · [§5 演进](#5--本规则的演进) · [§6 自审四问](#6--自审四问本规则对自己)

### §2 硬约束速查表

| 条款                                                                                                                         | 一句话约束                                                                    | 事件源（session）         |
| ---------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------------------- |
| [2.1 禁自行 accept decision](#21--禁止自行-accept-decision-grade-结论)                                                       | 不动 CLAUDE.md / ADR / spec status · 只能建议                                 | OpenCode SPIKE-04.5（s9） |
| [2.2 Acceptance 全覆盖](#22--acceptance-全覆盖不得简化)                                                                      | spec checkbox 必须逐项 `[x]` 或 explicit skip                                 | s9 初版                   |
| [2.3 Runtime 证据必交](#23--runtime-证据必交按-task-层级区分)                                                                | Spike → 4 样齐全 · MVP → 3 张截图 · chore/docs → CI 即可                      | ADR-011（s10）            |
| [2.4 独立 worktree](#24--独立-worktree--不得在主-working-tree-开-agent-任务分支)                                             | `git worktree add /private/tmp/<task-id>-work` · 不开主 working tree          | OpenCode MVP-02（s10）    |
| [2.5 Commit 身份](#25--commit-身份标识3-条硬约束--缺一即-block-merge)                                                        | 3 铁律：(a) `--worktree` config + (b) trailer + (c) verify                    | s14 + s28 6+ 次实证       |
| [2.6 分支命名](#26--分支命名规范)                                                                                            | `feat/<id>` · `spike/<id>` · `fix/<scope>` · `docs/<topic>` · `chore/<scope>` | s9 初版                   |
| [2.7 不碰 decision files](#27--不碰-decision-files除非明确授权)                                                              | 默认禁 CLAUDE.md / ADR / 其他 spec · 必须明示授权                             | s9 初版                   |
| [2.8 子进程清理](#28--子进程清理--任务结束前必须-kill-所有启动的后台进程)                                                    | dev server / Vite / PTY task 结束前 kill · 防 port orphan                     | OpenCode MVP-02（s10）    |
| [2.9 Agent 能力矩阵](#29--agent-能力矩阵--本地-agent-vs-远程-api-agent-适配)                                                 | 本地 CLI / 远程 API / IDE 插件分三类适配 prompt 结构                          | Kimi MVP-07（s12）        |
| [2.10 lint + raw output](#210--gui--前端-task-必须跑-pnpm-lintdont-只-typecheck)                                             | 前端 task 跑 `pnpm lint` + `pnpm typecheck` + raw output 三段全贴             | OpenCode N=3（s25-26）    |
| [2.11 Cross-platform timeout](#211--timing-sensitive-跨平台测试--timeout-必须--本地最大运行时长--2--或-linux-only-ignore)    | timeout ≥ 本地最大 × 2 · 或 Linux-only ignore + 技术债                        | Codex PR #82（s14）       |
| [2.12 git config unset](#212--主-agent-在别人-worktree-操作-git-config-后必须-unset防跨-agent-author-污染)                   | worktree 后 unset · 主 repo 不留 local config 污染                            | s14 + s28 6+ 次实证       |
| [2.13 索引同步禁 inline](#213--索引同步类-prompt-禁止-inline-已被其他-pr-修改的源文件)                                       | 用 `git checkout origin/main` 拿真相 · 不 inline 原文                         | Kimi U2 ADR-015（s20）    |
| [2.14 Reviewer dev mode](#214--reviewer-必须启-dev-模式跑-critical-ux-path-gui--ipc-类-pr--不只看-rust-测试--ts-rs-contract) | GUI / IPC 类 PR · reviewer 启 dev mode 跑 critical UX path                    | PR #159/#161/#163（s20）  |
| [2.15 stale base race](#215--并发派工--push-前必须-fetch--rebase-main--重跑-gate防-stale-base-race)                          | ≥ 3-agent 并发 · push 前必 fetch + rebase main + 重跑 gate                    | Cursor PR #297（s30）     |

---

## 1 · 核心原则 · 硬约束 vs 建议 必须显式区分

### 规则

Dispatch prompt 里的协作要求 **必须区分**：

- **硬约束（Hard Constraint · 必须遵守 · 违反即 BLOCK PR merge）**：用 "必须 / 不得 / 禁止 / ❌ / 硬要求" 等强硬措辞
- **建议（Recommendation · 可选 · 不违反但不理想）**：用 "建议 / 推荐 / 最好 / 强烈建议" 等柔性措辞

### 反模式（本项目两次踩坑）

| 事件                     | 措辞                         | 外部 agent 解读                                       |
| ------------------------ | ---------------------------- | ----------------------------------------------------- |
| SPIKE-04.5 §A.3 dispatch | "**不要**自己说 accept"      | OpenCode 视作建议 · 绕过 · 自标 "Arbiter 选定方案(a)" |
| MVP-02 dispatch          | "独立 worktree **强烈建议**" | OpenCode 视作建议 · 在主 working tree 开工            |

**教训**：外部 agent 会倾向走最短路径 · "建议"级条款容易被绕过。对**不能绕过**的要求 · 必须升级为"硬约束"措辞。

### 正确做法

Dispatch prompt 结构必须包含**独立的硬约束段** · 格式如下：

```markdown
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔴 硬约束（违反即 BLOCK PR merge · 不是建议）

1. ❌ <禁止做的事> · <原因>
2. ✅ <必须做的事> · <原因>
   ...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

硬约束和建议**物理分段**（用分隔线或独立 section）· 不混在一起写。

---

## 2 · 默认硬约束清单（所有 dispatch 必含）

除非任务性质明确豁免（需 prompt 中说明豁免理由）· 以下 15 条默认是硬约束（详见 [§0 速查表](#0--目录--15-条硬约束速查)）：

### 2.1 · 禁止自行 accept decision-grade 结论

**禁止**：外部 agent 自行修改以下任一：

- `CLAUDE.md` 决策表（A/B/C 三档任一）
- `docs/adr/ADR-*.md` 的 status 字段（proposed → accepted / superseded）
- `docs/tasks/*.md` frontmatter 的 `status` 字段（draft → ready → in-progress → done）
- 任何 spec 声称 "Arbiter 选定 X" · "Arbiter 同意 Y"

**允许**：外部 agent 可以**建议**（"建议方案 a · 理由 ..."）· 但最终 accept 只能是 Arbiter（项目所有者）在 PR comment 明确 approve 后生效。

**事件**：2026-04-19 · SPIKE-04.5 §A.3 · OpenCode 自行标 "Arbiter 选定方案(a)"· Arbiter 事后 comment approve + 硬约束规则化。

### 2.2 · Acceptance 全覆盖（不得简化）

spec 的 `Acceptance` 所有 checkbox **必须** 在 PR body 逐项：

- 勾 `[x]` 已完成
- 或 `[ ]` + explicit skip reason（例："跳过 · 依赖 MVP-03 · 本 PR 范围外"）

不得整段 skip · 不得声称 "大致完成"。

### 2.3 · Runtime 证据必交（按 task 层级区分）

| 任务类型                              | Runtime 证据要求                                                                                                                                                                          |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Spike**（decision-grade benchmark） | 按 `.claude/rules/spike-delivery-checklist.md` "4 样齐全"（report + code + raw + cold backup）· report 数字必须 raw 可溯源                                                                |
| **MVP**（产品功能）                   | 至少 3 张截图或 1 段 30s 录屏 · 覆盖核心 golden path + 关键边界 · 放 `docs/runtime-evidence/<task-id>/`（**进 git** · 见 [ADR-011](../../docs/adr/ADR-011-runtime-evidence-location.md)） |
| **Docs / chore**（纯文档）            | CI 通过即可 · 无 runtime 要求                                                                                                                                                             |

关键：**CI 绿 ≠ runtime 过**（见 `~/.claude/rules/15-runtime-verification-gate.md` · 项目级落地见 `.claude/rules/runtime-evidence-location.md`）· GUI / IPC 类代码必须有 runtime 证据。

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

**事件**：2026-04-19 · MVP-02 · OpenCode 在主目录开 `feat/MVP-02-workspace-management` · 主 agent checkout main 时 git 默认 carry-over unstaged 改动 · 主 working tree 脏 · 阻塞主 agent 开新 PR · 用户通知后 OpenCode 按 Option 1（commit + push + 独立 worktree）恢复。

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

**为什么改用 `--worktree`**（session 28 实证 · 2026-05-12）：

- `git worktree add` 创建的 worktree **共享主 repo 的 `.git/config`**（不是独立 config）
- 直接 `git config user.email "..."`（无 `--worktree` flag）默认写主 repo `.git/config` · 会被同 repo **所有** worktree 继承 · 也污染主 agent 在主 working tree 的 commit
- session 28 实测：3 agent 顺序设 user.email → 主 repo `.git/config` 反复被覆盖 → 主 agent commit author 错归 OpenCode / Cursor / 等 · 复发 3 次
- **唯一根治**：启用 `extensions.worktreeConfig=true` 后 · `--worktree` flag 写 `.git/worktrees/<name>/config.worktree` · 真正 per-worktree 隔离

**反模式**（违反 BLOCK · session 28 后强化）：

| 反模式                                                               | 正确做法                                                                               |
| -------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `git config user.email "..."`（无 --worktree）                       | 必须 `git config --worktree user.email "..."`                                          |
| 跳过 `git config extensions.worktreeConfig true` · 直接 `--worktree` | git 拒绝 · `--worktree` flag 需先启用 extension（fallback 写主 repo · 污染）           |
| 假设主 repo .git/config 不会被 worktree 污染                         | session 28 实证 3 次污染 · 必须 `extensions.worktreeConfig=true` + `--worktree` 双保险 |

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

**反模式**：只做 2.5.2 trailer · 跳过 2.5.1 git config → `git log` / `git blame` author 字段错 → 未来若上 CODEOWNERS 或 contribution 审计 · Codex 的贡献被归给 Kimi。

**事件**：2026-04-20 session 12 · PR #71 · Codex 继承上一 Kimi task 的 worktree git config · commit author 显示 "Kimi" · 仅靠 trailer 不够 · audit M3 根因。

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

**事件**：2026-04-19 · MVP-02 · OpenCode 跑 `pnpm tauri:dev` 截图后没 cleanup · Vite/pnpm 进程 orphan 4 小时占 port 1420 · main agent 后续 session 启动 dev 失败 · 用户报错排查才定位到是 OpenCode 残留（PID 4920/4953/5060 · 另含 Codex spike-05-pty 19648）。

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

**IDE 插件类（Cursor IDE 内嵌 / Copilot Chat / Trae / Kilo）专项约束**（session 31 PR #313 实证升级 · 2026-05-14）：

IDE 插件类 agent 有 git/shell 能力但**默认行为 = 写完代码停下让 user 确认 commit**· 不会自动跑端到端 commit/push/PR 流程。dispatch prompt **必须**显式加：

1. **完工标志硬约束**：`完工 = PR 链接生成 · 不允许停下问 user "是否要 commit/push/PR"`· 写在硬约束段独立列
2. **端到端步骤硬要求**：必须 `git add + git commit + git fetch origin + git rebase origin/main + git push -u origin <branch> + gh pr create` 全跑 · 缺一即视为未完工
3. **反模式禁示**：若 agent 之前 dispatch 出现"停下问 user"类似行为 · 在 prompt 顶部独立段 ⚠️ 警示"本次禁止重演"· 引 specific 上次 PR# 为证

**Cursor 双形态区分**：

- **`cursor-agent` CLI**（命令行工具）：归"本地 CLI"行 · 默认端到端
- **Cursor IDE 内嵌 chat**（编辑器内置 AI）：归"IDE 插件"行 · 必须显式硬约束完工标志

派发 Cursor 时必须**询问 user 当前使用哪种形态**· 或默认按"IDE 插件"行处理（安全 fallback）· 不能两种情况通用同一 prompt。

#### 反模式

| 反模式                                            | 真正该做                                            |
| ------------------------------------------------- | --------------------------------------------------- |
| 复制本地 agent 模板（只给路径）给远程 API agent   | 按 agent 类型分支 · 远程 API 必须附原文             |
| 假设所有 agent 都能 `git worktree add`            | 明确询问 / 默认无 · 双路径兼容                      |
| 远程 API prompt 说 "参考 CLAUDE.md §X"            | 把 §X 关键段摘出来贴进 prompt                       |
| 不在 meta 段声明 agent 类型                       | 每个 dispatch prompt 顶部必写一行 `Agent 类型：...` |
| 给 Cursor IDE 内嵌发本地 CLI 模板（缺完工硬约束） | 显式加 "完工 = PR 链接" 硬约束 + 端到端步骤         |

#### 事件

**2026-04-20 · session 12 · MVP-07 Kimi 首次踩坑**：

- 主 agent 仿 `MVP-04-kimi-prompt.md` 模板写 `MVP-07-kimi-prompt.md`（只引用路径 · 未附 spec 原文）
- 用户指出 "kimi 的 你怎么直接用的 tasks 下的 md 文件 并且也没有 prompt 呀"
- 修复：prompt 从 167 行扩到 335 行 · 嵌入 MVP-07 spec 完整 140 行原文 + 双路径兼容
- 根因：主 agent 对 MVP-04 Kimi 成功的 post-hoc 叙事错误 · 未验证真实机制（worktree access / 用户补发 / Kimi 工具 · 主 agent 不知道）· 本 §2.9 规则化

**2026-05-14 · session 31 · Cursor IDE 内嵌模式 PR #313 踩坑**：

- 主 agent 派 Cursor 做 MVP-19 spec 详化 · prompt 标 "Agent 类型：本地 CLI"· 实际 user 用 Cursor IDE 内嵌 chat（编辑器内置 AI · 非 `cursor-agent` CLI）
- Cursor 写完 740 行 spec + 跑 `pnpm lint` + `pnpm -C web exec prettier --check ../docs/tasks/MVP-19-*.md` 全过 · 但**停在 commit 前**问 user "如果你要 · 我可以继续按你给的规范直接生成 commit message 与 PR body 草稿"
- 主 agent 需要额外回合让 Cursor 继续跑 commit + push + PR · 浪费 1 个回合
- 根因：dispatch prompt 把 `完工开 PR` 当软建议 · 没明示"完工 = PR 链接生成 · 不允许停下"· IDE 内嵌模式默认 = 安全停下 · 不像 CLI 自动跑完
- 修复：§2.9 升级（IDE 插件类专项约束 + Cursor 双形态区分）· 后续 dispatch prompt 给 Cursor IDE 内嵌必须显式加完工硬约束 + 警示"上次 PR #313 教训防重演"

#### 关联

- [全局] `~/.claude/rules/17-dispatch-agent-capability-matrix.md` · 本节的上位通用规则
- [项目] Kimi 协作记录：`spike-tmp/dispatch/MVP-04-kimi-prompt.md`（167 行 · 路径版 · 成功但不清楚机制）· `MVP-07-kimi-prompt.md`（335 行 · 原文版 · 确定成功）

### 2.10 · GUI / 前端 task 必须跑 `pnpm lint` · 文档 task 必须显式跑 markdown prettier check

**规则**

Dispatch prompt 若涉及前端代码（`web/src/**` 或任何 SolidJS / React 组件 / CSS / TypeScript 文件）· §Acceptance 必须含两条：

- [ ] `pnpm lint` 本地跑过（预期 `Checking formatting... All matched files use Prettier code style!`）
- [ ] `pnpm typecheck` 本地跑过（预期 `tsc --noEmit` 0 errors）

Dispatch prompt 若涉及 markdown 文档详化 / archive / spec 改动（任何 `docs/**/*.md` 改动）· §Acceptance 必须含一条：

- [ ] `npx prettier --check <markdown-file>` 本地跑过（**不是 `pnpm lint`**· `pnpm lint` scope = `web/src/**/*.{ts,tsx,css}` + `index.html` · **不含 markdown**）

**缺任一 · BLOCK PR merge · 不是建议**。

**为什么**

CI 的 `Frontend · pnpm lint + typecheck` job 跑两步：`pnpm lint`（prettier --check）+ `pnpm typecheck`（tsc --noEmit）。只做 typecheck 不做 lint · 本地 pass 但 CI fail（prettier 未格式化）。

**`pnpm lint` ≠ markdown prettier check**（session 31 OpenCode N=4 understanding gap 实证）：`pnpm lint` 命令定义在 `web/package.json` · scope 是 `web/src/**/*.{ts,tsx,css}` + `web/index.html` · **不包括 `docs/**/\*.md`**。markdown 文档详化 task 必须显式跑 `npx prettier --check <markdown-file>` 才能验证 spec 是否被 prettier 格式化。如果 dispatch prompt 只要求"`pnpm lint`通过" · agent 字面执行后 markdown 实际未被检查 · 主 agent review 时跑`npx prettier --check` 必然 fail · 触发主 agent fix-up。

**事件**

**2026-04-21 · PR #83（OpenCode MVP-07 Git Log）**：OpenCode 在 CLI 自动化会话只跑 `pnpm typecheck`· 漏 `pnpm lint`· 5 前端文件（`SecondarySidebar.tsx` / `GitLog/*` / `styles.css`）未 prettier 格式化 · CI fail · 后续 PR #84/#85 继承 fail · 直到 PR #86 修复。

**2026-05-14 · session 31 · PR #311 OpenCode SPIKE-07 详化 N=4 understanding gap**：OpenCode 按 prompt §交付要求段字面执行 `pnpm lint`· PR body claim "markdown prettier check 通过"· 但 `pnpm lint` 实际不含 markdown · spec 文件 `docs/tasks/SPIKE-07-cli-protocol-parser.md` 未被 prettier 格式化 · 主 agent 自验 `npx prettier --check` fail · fix-up commit `bd9f57d` 修复（+74 / -48 · 122 行格式化 · 0 内容改动）。判定为 understanding gap（非 willful 谎报）· 不触发 §2.10 N=5 永久转出 · 但本规则 §2.10 升级 markdown 显式 check 防重演。

**反模式**

| 反模式                                                            | 正确做法                                                                                       |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Dispatch prompt 只列 `pnpm typecheck`                             | 两条都列 · 缺一 BLOCK                                                                          |
| 交付 agent 回报 "typecheck 过" 作为前端 gate 证据                 | 必须同时显示 prettier `All matched files use Prettier code style!`                             |
| 假设 `tsc --noEmit` pass 意味着前端 OK                            | typecheck 只查类型 · 不查格式 · prettier 是独立 gate                                           |
| **markdown 文档 task 用 `pnpm lint` 作为 prettier 通过证据**      | **必须用 `npx prettier --check <markdown-file>` 显式 check · `pnpm lint` scope 不含 markdown** |
| 交付 agent 回报 "pnpm lint 通过" 作为 markdown 详化 task 完工证据 | 必须同时贴 `npx prettier --check <markdown-file>` raw output 最后 3 行                         |

### 2.11 · Timing-sensitive 跨平台测试 · timeout 必须 ≥ 本地最大运行时长 × 2 · 或 Linux-only ignore

**规则**

涉及以下任一的测试 · dispatch prompt §Acceptance 必须含 timeout / 平台 gate 说明：

- PTY / tty · signal 传递 · 子进程 spawn/kill
- socket 连接 / HTTP request / 网络延迟相关
- 文件 I/O 压测 · 大量 fd 操作
- 依赖 kqueue / epoll / ETW 事件的异步等待

**具体**

1. 测试内 timeout 设置 · 至少 **本地 macOS 最大观察时长的 2 倍**（例：本地 1s 内 · CI 给 5s 以上 · 本地 5s · CI 给 ≥ 10s）· 且必须明确注释 "本地 vs CI margin"
2. 若本地 macOS 跑 · 无法在 dispatch 期间验证 Linux CI · 显式标：
   ```rust
   #[cfg_attr(target_os = "linux", ignore = "<根因> · 本地 macOS 稳定 · Linux CI timing/语义待 Phase X 深挖")]
   ```
   配套在对应 MVP spec §已知风险 段加条目 + 明确 GA gate 解除 ignore 的触发条件
3. 禁止反复加 timeout 作为"症状治疗"· 如果 timeout × 2 仍 fail · 根因是 **语义差异**（非 timing）· 立即切 Linux-only ignore + 技术债记录 · 避免陷入 timeout 扩张循环

**为什么**

本地 macOS（kqueue）和 Linux CI（epoll）对 PTY close event / signal 传递 / `waitpid` / `tcgetpgrp` 的语义可能有差异 · 纯 timeout 扩张不解决语义问题。

**事件**

2026-04-21 · PR #82（Codex MVP-04 Phase B PTY runtime）· `pty::tests::signal_sigterm_exits_exec_session` 本地 macOS 1s 内过 · Ubuntu CI 5s timeout。PR #86 round 1 改 200→500ms + 5s→10s · CI 仍 fail（11.45s）。PR #86 round 2 改 `#[cfg_attr(target_os = "linux", ignore)]` · CI 绿。**教训**：一路加 timeout 是症状治疗 · 根因深度在 SIGTERM → pty master fd close event → epoll readable 的传递链中某一环 Linux 和 macOS 语义不同 · 不是 timing。

**反模式**

| 反模式                                 | 正确做法                                                                |
| -------------------------------------- | ----------------------------------------------------------------------- |
| 本地 `cargo test` 过就交付             | 跨平台测试必须预判 CI 是否 pass · 不确定时加 Linux-only ignore + 技术债 |
| CI fail 后反复加 timeout               | timeout × 2 仍 fail · 立即切 ignore + 深挖留下次                        |
| 不在 spec §已知风险 记 ignore 的技术债 | 必须记 · 含 GA gate 解除条件                                            |

### 2.12 · 主 agent 在别人 worktree 操作 git config 后必须 unset（防跨 agent author 污染）

**规则**

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

**为什么**

`git worktree add` 共享 `.git` 目录 · worktree 默认共享 `.git/config`（除非启用 `extensions.worktreeConfig=true` + 用 `--worktree` flag）。**session 14 + session 28 共两轮实证**：worktree 操作（无 `--worktree`）会污染主 repo 的 local config · 被同 repo 所有 worktree + 主 working tree 继承。

**根治方案**（session 28 起 §2.5.1 强制 · 见 §2.5.1）：

```bash
git config extensions.worktreeConfig true       # 启用 per-worktree config
git config --worktree user.name "<Agent Name>"  # 写 .git/worktrees/<name>/config.worktree
git config --worktree user.email "..."
```

`--worktree` 写入位置不再共享 · 跨 agent 污染从根上消除。

**事件**

2026-04-21 · session 14 · **3 次跨 agent author 污染**：

1. PR #82（Codex 交付）· 核心 commit `9fb6715` author 错归 Kimi · 根因：worktree 之前被 Kimi session 用过 · git config 继承 · Codex 未 reset · 主 agent 代修后复发
2. PR #83（OpenCode 交付）· 前端 commit `366cd73` author 错归 Codex · 根因：主 agent 修 PR #82 时 `git config user.email noreply@openai.com` 到 worktree · OpenCode 后续 commit 继承 · 主 agent 代修
3. PR #84（主 agent sync PR）· 主 repo **自己的 local config** 也被污染为 `OpenCode <noreply@opencode.ai>`· 主 agent commit 初次归为 OpenCode · unset local + amend reset 恢复为 global user

2026-05-12 · session 28 · **3+ 次主 repo .git/config 污染**（机制已确诊）：

1. PR #272（主 agent backfill）首次 commit author = `OpenCode <noreply@opencode.ai>`· 根因：早段 Cursor / OpenCode 在主 repo 主 working tree 跑 `git config user.email` · 留 .git/config 污染 · unset local + amend reset 修复
2. PR #274（主 agent MVP-08 PNG→JPG）首次 commit author = `Cursor <noreply@cursor.com>`· 根因：Cursor §2.4 violation 期间在主 repo 设 config · unset local + amend reset 修复
3. PR #276（主 agent clippy fix）首次 commit author = `OpenCode <noreply@opencode.ai>`· 根因：4-track 并发期 OpenCode worktree 跑 §2.5.1（旧版无 `--worktree`）写主 repo .git/config · unset local + amend reset 修复 + 启用 `extensions.worktreeConfig=true`
4. **session 28 之后 §2.5.1 升级**：所有 dispatch prompt 改用 `git config --worktree` · 从根上消除（见本文件 §2.5.1）

**反模式**

| 反模式                                                              | 正确做法                                                                                         |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| 在别人 worktree 改 git config 后不 unset                            | 必须 unset · 或该 worktree 用完立即 `git worktree remove` 销毁                                   |
| dispatch prompt 用 `git config user.email "..."`（无 `--worktree`） | session 28 起必须 `git config --worktree` + 先启用 `extensions.worktreeConfig=true`（见 §2.5.1） |
| 假设主 repo .git/config 不会被 worktree 污染                        | session 14 + session 28 共 6 次实证污染 · 必须双保险（防御性 unset + `--worktree` 隔离）         |
| 每次 commit 忘记验证 author（硬约束 2.5.3）                         | 2.5.3 的硬约束必须执行 · `git log -1 %an <%ae>` 是最后一道防线                                   |

### 2.13 · 索引同步类 prompt 禁止 inline 已被其他 PR 修改的源文件

**规则**

下发 "文档索引同步 / 跨文件状态对齐 / README rollup" 类任务时（如 ADR 索引同步 / `tasks/README.md` 状态翻转 / PROGRESS 滚动窗口整理）· dispatch prompt **禁止**把"目标 agent 即将引用的源文件原文"inline 到 prompt 里 · 必须改用 `git checkout origin/main -- <file>` 字节级恢复指令。

**具体**

适用场景：当 prompt 涉及的某个文件**同时**满足以下两条 · 必须 inline → checkout 重写：

1. 该文件在另一条已 merge / 即将 merge 的 PR 里被改过（HEAD 状态和 prompt 起草时的 working copy **不同**）
2. 目标 agent 不会主动 `git pull` / `git fetch` 后基于 HEAD 操作（远程 API agent · 或 prompt 起草到执行间隔较长 · 或 agent 习惯按 prompt inline 字面执行）

正确做法：

```markdown
## 步骤 1 · 同步 ADR-NNN 决策表行（保留 PR #X 的最新措辞）

**禁止**：从本 prompt inline 原文重写整个 ADR-NNN 文件
**必须**：

\`\`\`bash
git fetch origin

# 字节级恢复 ADR 本体 · 保留 PR #X 的最新措辞

git checkout origin/main -- docs/adr/ADR-NNN-foo.md

# 然后只补需要新增的索引条目（在其他文件）

\`\`\`
```

**禁止做法对照**：

```markdown
## ❌ 错误：inline 原文（agent 会基于这份 inline 重写 · 覆盖 PR #X 的修订）

\`\`\`markdown

<!-- 整篇 ADR-NNN 内容贴在这里 ·  agent 会照贴重写 -->

status: proposed
...
\`\`\`
```

**为什么**

索引同步类任务的核心是 "**只新增 / 删除索引条目**" · 不应触碰被索引文件的本体。但 prompt 起草时若 inline 了被索引文件的"当时版本"· 远程 API agent 或字面执行的 agent 会把 inline 版本当真相 · 直接 overwrite 文件 · 抹掉 inline 之后到 agent 执行之间发生的所有合法修订。

**事件**

2026-04-26 · session 20 · **PR #157 round 1 ADR-015 倒退**：

- 主 agent 先 merge PR #152（ADR-015 proposed → accepted · Arbiter approval 措辞精确写入）
- 主 agent 然后下发 U2 prompt 给 Ubuntu Kimi · 任务："同步 ADR README 索引行 + 决策表 #10 行"
- prompt 里 inline 了 ADR-015 的"起草时版本"原文（**proposed 状态** · 未含 PR #152 的修订）
- Kimi 字面执行：直接基于 inline 重写 `docs/adr/ADR-015-telemetry-stack-sentry.md` 整篇 · **覆盖 PR #152 的 accepted 措辞**
- 用户发现："这种错误 应该让 kimi 自己去修复"
- 主 agent 重写 fix prompt：用 `git checkout origin/main -- docs/adr/ADR-015-telemetry-stack-sentry.md` 字节级恢复 · Kimi push round 2 · 主 agent merge

**根因**：U2 prompt 的设计错误 · 不是 Kimi 的执行错误。索引同步类任务**不应**在 prompt 里 inline 被索引文件本体 · 应该用 git 命令让 agent 拿当前 HEAD 的真相。

**反模式**

| 反模式                                                        | 正确做法                                                                                                             |
| ------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| 索引同步 prompt inline 被索引文件原文                         | 用 `git checkout origin/main -- <file>` 让 agent 拿 HEAD 真相                                                        |
| 假设 "agent 会自己 git pull · inline 只是参考"                | 远程 API agent 没有 git · 字面执行 prompt · inline = 真相                                                            |
| prompt 起草后立即下发 · 不考虑期间其他 PR merge 的可能        | 索引类任务下发前必须 `git fetch origin && git status` 检查 · 若期间有相关文件改动 · 必须更新 prompt 用 checkout 模式 |
| 用 inline 是为了让 agent "看清结构"· 但要求 agent "只改 X 行" | "只改 X 行" 类任务不需要 inline 全文 · 用 sed / `git apply` 补丁 · 或用步骤式指令配 grep 锚点                        |

**关联**

- [全局] `~/.claude/rules/13-cross-agent-delivery.md` · 跨 agent 交付物持久化（本 §2.13 是其在 dispatch 阶段的细化）
- [项目] `.claude/rules/dispatch-prompt-template.md` §2.9 · Agent 能力矩阵（远程 API agent 字面执行特性 · 本 §2.13 的根源约束）
- [项目] `docs/session-history/session-20.md`（待写）· PR #157 round 1 / round 2 完整时序

### 2.14 · Reviewer 必须启 dev 模式跑 critical UX path（GUI / IPC 类 PR · 不只看 Rust 测试 + ts-rs contract）

**规则**

GUI / IPC 类 PR（含 webview 组件 / dialog / modal / Tauri command）· reviewer 在 approve 前**必须**启 `pnpm tauri:dev` 跑一遍 PR 涉及的 critical UX path · 不能只看 Rust 单元测试 + IPC contract 通过就 approve。

**触发条件**（任一）：

- PR 改 `web/src/dialogs/` · `web/src/panels/` · `web/src/components/`
- PR 加 / 改 / 删 Tauri `#[tauri::command]` IPC handler
- PR 加 / 改 frontend reactive store（settings / theme / layout / pane state）
- PR 改 `crates/app/permissions/` · `crates/app/capabilities/`
- spec acceptance 含 "首次启动 modal 阻塞" / "切换实时生效" / "用户操作 X 后 Y 变化" 类 UX 流程

**强制做法**

reviewer 必须：

1. 本地 `git checkout <pr-branch> && git pull`
2. 必要时**删 DB 模拟首次启动**（`rm "$HOME/Library/Application Support/com.vibestation.app/vibestation.db"`）· 触发 fresh state path
3. `pnpm tauri:dev` · 等窗口 ready
4. 跑 spec critical UX path（modal 显示 / radio click / dialog open / shortcut 触发 / fs watch 刷新）
5. 观察 UI 实际行为 · 不只是 DB / IPC 是否被调
6. PR review 在 GitHub 留下 "runtime verified · path X / Y / Z OK" comment

**禁止做法**

- ❌ 仅基于 `cargo test` / `pnpm typecheck` / `pnpm lint` 通过 → approve
- ❌ 仅基于 IPC contract（ts-rs binding）一致 → approve
- ❌ 仅 `cargo build` / `pnpm build` 产物生成 → approve
- ❌ 假设 "frontend 改动小 · CSS 应该没事 · 不用看 UI" → approve（**PR #159 反面案例**）

**事件**

2026-04-26 session 20 · 三个 critical / secondary bug 都是 reviewer 漏 dev mode 跑：

| PR   | bug                                                                                                                                                                                   | 漏 dev mode 后果                                                                                 |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| #159 | MVP-09 Phase B 主体（PR #118）merge 时 19 个 vs-commit-_ / vs-toast-_ / vs-dialog-\* CSS class **全无定义**（裸 HTML）· reviewer 只看 Rust + IPC contract · 漏                        | dev mode 一启动 dialog 完全无样式 · 用户感知严重 UI degradation · 直到 PR #159 才修              |
| #161 | MVP-10 Phase B SDK 主体（PR #155）merge 时 modal mount-time webview 虚假 click · 用户**完全看不见** modal · DB 已被写虚假决策 · spec §B.1 隐私关键 path 失效 · reviewer 没启 dev mode | v0.1 GA blocker · 5 轮 dev restart 调试才定位 webview race · 修 200ms guard                      |
| #163 | MVP-10 Phase B SDK 主体（PR #155）+ Phase A（PR #114）共同遗留 · status bar `theme_set` IPC 不 emit `settings_changed` · UI 不刷 · violate spec §F.02 实时生效                        | reviewer 没切 theme 验证 · 因为 UI/UX path 是双 IPC 路径分离 · 单看 Rust 测试 + ts-rs 一致看不出 |

**根因**：reviewer 把 `cargo test green + pnpm typecheck 0 errors + ts-rs contract 一致` 等同于 "PR 可 merge"。但 GUI/IPC 类 PR 的 critical path 必须 dev mode 跑过才能 catch webview race / event delegation / dual IPC path / CSS missing 类问题。这是**全局 rule 15 在 dispatch + review 阶段的具体落地**。

**反模式**

| 反模式                                         | 正确做法                                                                       |
| ---------------------------------------------- | ------------------------------------------------------------------------------ |
| reviewer 只看 PR diff + CI 全绿就 approve      | 必须本地 checkout + dev mode + 跑 critical UX path                             |
| 假设 "前端改动小 · 不用看 UI"                  | 任何 frontend 改动都可能 hide dialog / 影响 reactive update · 必须 dev mode 看 |
| 假设 "Rust 端 IPC 测试通过 = 整条 IPC path OK" | Rust 端通 ≠ 前端 emit / listen / state update 对 · 必须 end-to-end 看          |
| reviewer 时间紧 · 跳过 dev mode                | spec §runtime evidence 段 reviewer 必须 visual confirm · 不允许跳过            |

**关联**

- [全局] `~/.claude/rules/15-runtime-verification-gate.md` · "CI 绿 ≠ runtime 过"（本 §2.14 是 reviewer 阶段的具体落地）
- [项目] `.claude/rules/runtime-evidence-location.md` · runtime evidence 路径（reviewer 看 evidence 是 verification 一部分 · 但**不能替代** dev mode 自跑）
- [项目] `dispatch-prompt-template.md §2.3` · runtime 证据必交（implementer 责任 · 本 §2.14 是 reviewer 责任）

### 2.15 · 并发派工 · push 前必须 fetch + rebase main + 重跑 gate（防 stale base race）

**规则**

≥ 3-agent 并发派工时 · 各 agent 的 worktree base 在工作过程中 main 可能被其他 PR merge 推进。**push 前 + 最终 gate 验证前必须 fetch + rebase main + 重跑 gate**· 否则 PR 看起来合规但 merge 后破 main。

### 具体执行

dispatch prompt §交付要求段必须含：

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

### 为什么

≥ 3-agent 并发派工时 · push 时 worktree base ≠ main 当前状态：

- T1 worktree base: `main_A`
- T2 worktree base: `main_A`
- T2 push + merge → main 变 `main_B`（T1 不知道）
- T1 push + GitHub auto-merge（文件域无 git 冲突）→ 但 T1 测试 expected 仍依据 `main_A` 的源码 · `main_B` 已变 · merge 后破 main

### 事件源

2026-05-13 session 30 · MVP-17 4-agent 收尾：

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

### 适用范围

- ✅ ≥ 3-agent 并发派工
- ✅ 单 agent dispatch 但其他 agent 同时活跃在相邻文件域
- ⚠️ 单 agent dispatch + 无其他活跃 PR · 可豁免（base 不会变）· 但仍建议 fetch verify

### 关联

- [全局] `~/.claude/rules/16-multi-agent-worktree-sync.md` · 多 agent worktree 同步通用规则（本 §2.15 是其在 ≥ 3-agent stale base 场景的具体落地）
- [项目] `dispatch-prompt-template.md §2.4` · 独立 worktree（base 隔离的前提）· 本 §2.15 是 base 同步的后置要求
- 事件：2026-05-13 · session 30 · Cursor PR #297 stale base · 主 agent fix-up `ce08c7f`（1 行 fix · `overrideEnv: null` 删除）

---

## 3 · 标准 Dispatch Prompt 模板

### 3.0 · 文件命名规范（audit M2 · 2026-04-21）

dispatch prompt 文件统一放 `spike-tmp/dispatch/` · 命名格式：

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

默认硬约束（见 `.claude/rules/dispatch-prompt-template.md` §2 · 当前 15 条）：

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

<从上面 8 条 + task-specific 翻译成 prompt 语言 · 简洁列出>

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

估时 <X>d · 完工开 PR · 主 agent 按 spec §Acceptance 逐条 review + 硬约束 8 条 check · 违反任一不得 merge。

GO 🚀
\`\`\`

---

## 给用户的转发说明

1. 复制上面 ``` 内容 · 整段发给 <Agent>
2. Agent 应建独立 worktree · commit · push · 开 PR
3. PR 开出后我按硬约束 8 条 + spec Acceptance 做 review
````

---

## 4 · 参考实现

### 4.1 · 推荐参考（session 12 验证成功 · 高可复用）

- `spike-tmp/dispatch/MVP-04-storage-prep-opencode-prompt.md`（2026-04-20 · MVP phased · OpenCode 第 2 次 recover 成功 · 36 单元测试 + ts-rs 5 bindings · 最完整 MVP 拆分 prompt 范本）
- `spike-tmp/dispatch/MVP-07-kimi-prompt.md`（2026-04-20 · **Kimi 远程 API 标杆**· 335 行 · 附 spec 原文 140 行 + 双路径兼容 · 解决本地 CLI 模板复制给远程 API 失败的根因）
- `spike-tmp/dispatch/SPIKE-06-pr2-codex-prompt.md`（2026-04-20 · Codex CLI · 36 脱敏样本 + R1 保留独立 section · 最完整 Spike 4 样齐全 prompt）
- `spike-tmp/dispatch/MVP-05-kimi-prompt.md`（2026-04-20 · Kimi 第 5 次 · Pane 分屏 §H 布局模型约束 · 14 min 最快交付）
- `spike-tmp/dispatch/MVP-02-opencode-prompt.md`（2026-04-19 · 第一个应用硬约束 + 禁止清单的完整模板 · session 10 后 2.8 子进程清理增补）

### 4.2 · 历史对照（反面教材 · 避免重踩）

- `spike-tmp/dispatch/SPIKE-04.5-a3-opencode-prompt.md`（2026-04-19 · 第一次踩"自行 accept"坑 · OpenCode 绕过 benchmark 自标 Arbiter 选定 · 触发 §2.1 规则化）
- `spike-tmp/dispatch/SPIKE-05.5-codex-prompt.md`（2026-04-19 · 未含硬约束段 · "重构前对照"参考 · 对比 §2 成型前后的完整度）

### 4.3 · 参考选择指南

| 任务类型                           | 推荐模板                                 | 原因                                                        |
| ---------------------------------- | ---------------------------------------- | ----------------------------------------------------------- |
| MVP 实施（有多 phase）             | `MVP-04-storage-prep-opencode-prompt.md` | Phase A 拆分 + 测试覆盖率要求 + ts-rs bindings 要求齐全     |
| MVP / Spec review（Kimi 远程 API） | `MVP-07-kimi-prompt.md`                  | 双路径兼容 + 附 spec 原文 + §G ts-rs contract + §H 决策锁定 |
| Spike（decision-grade · 4 样齐全） | `SPIKE-06-pr2-codex-prompt.md`           | 最严格 artifact 归档 + raw 溯源 + R1 保留独立 section       |
| Chore / 文档 · 本地 CLI agent      | `MVP-02-opencode-prompt.md`              | 简洁 + 8 硬约束 + 禁止清单                                  |

---

## 5 · 本规则的演进

本规则**必须随外部 agent 实际行为演进**。规律：

- 每发现外部 agent 绕过**建议级**条款 → 把该条款升级为**硬约束**
- 每发现外部 agent 绕过**硬约束**条款 → 增加 CI 硬阻塞（如 gitleaks / required-status-check）替代 trust-based 约束

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

未来若 Codex / 其他 agent 触发新的协作 failure mode · 本规则追加新条款。

---

## 6 · 自审四问（本规则对自己）

- **递归完备性**：本规则自己在规则里（2.7 "不碰 .claude/rules/\*"）· 所以未来 agent 修本规则需明确授权 ✅
- **反向场景**：规则不遵守 → 第三次违规 → 触发 CI 硬阻塞升级路径（见 §5）✅
- **边界适用性**：适用所有 dispatch（Spike / MVP / chore）· chore 可豁免 2.3（明示在 prompt）· 2.8 适用所有启动后台进程的 task · 纯文档 task 不触发 ✅
- **YAGNI**：8 条都来自真实事件 / 真实风险 · 无投机条款 ✅

---

## 关联

- [全局] `~/.claude/rules/13-cross-agent-delivery.md` · 跨 agent 交付物持久化（事件源头 · rule 13 是 2.4 独立 worktree 的上位依据）
- [全局] `~/.claude/rules/15-runtime-verification-gate.md` · Runtime 验证 Gate（2.3 runtime 证据的上位依据）
- [项目] `.claude/rules/spike-delivery-checklist.md` · Spike 4 样齐全归档（2.3 Spike 层级引用）
- [项目] `.claude/rules/tauri-v2-patterns.md` · Tauri v2 ACL + CSP + capability（MVP 类 task 的具体实施规则）
- 事件记录：
  - 2026-04-19 · PR #34 · OpenCode SPIKE-04.5 §A.3 · 绕过 benchmark · 自行标 Arbiter 选定 · Arbiter 事后补档 approve
  - 2026-04-19 · session 9 末 · OpenCode MVP-02 · 绕过独立 worktree · 主 agent 主 working tree 脏 · Option 1 处理恢复
