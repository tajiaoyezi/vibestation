# Dispatch Prompt 模板规则 · Vibestation 专属

> 本规则沉淀给**外部 agent（Codex / OpenCode / 未来的 Claude 实例 / 其他工具）**下发任务 prompt 的硬约束 + 建议区分。凡触发"我要发 dispatch prompt 给外部 agent"前 · 先读本规则 · 按模板写。
>
> **触发条件**：主 agent 要把 task（Spike / MVP / Feature / Doc）通过用户转发下发给外部 agent 执行 · 非主 agent 自己执行的任务。
>
> **关联全局规则**：`~/.claude/rules/13-cross-agent-delivery.md` · `~/.claude/rules/15-runtime-verification-gate.md`

---

## 1 · 核心原则 · 硬约束 vs 建议 必须显式区分

### 规则

Dispatch prompt 里的协作要求 **必须区分**：

- **硬约束（Hard Constraint · 必须遵守 · 违反即 BLOCK PR merge）**：用 "必须 / 不得 / 禁止 / ❌ / 硬要求" 等强硬措辞
- **建议（Recommendation · 可选 · 不违反但不理想）**：用 "建议 / 推荐 / 最好 / 强烈建议" 等柔性措辞

### 反模式（本项目两次踩坑）

| 事件 | 措辞 | 外部 agent 解读 |
|---|---|---|
| SPIKE-04.5 §A.3 dispatch | "**不要**自己说 accept" | OpenCode 视作建议 · 绕过 · 自标 "Arbiter 选定方案(a)" |
| MVP-02 dispatch | "独立 worktree **强烈建议**" | OpenCode 视作建议 · 在主 working tree 开工 |

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

除非任务性质明确豁免（需 prompt 中说明豁免理由）· 以下 8 条默认是硬约束：

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

| 任务类型 | Runtime 证据要求 |
|---|---|
| **Spike**（decision-grade benchmark） | 按 `.claude/rules/spike-delivery-checklist.md` "4 样齐全"（report + code + raw + cold backup）· report 数字必须 raw 可溯源 |
| **MVP**（产品功能） | 至少 3 张截图或 1 段 30s 录屏 · 覆盖核心 golden path + 关键边界 · 放 `docs/runtime-evidence/<task-id>/`（**进 git** · 见 [ADR-011](../../docs/adr/ADR-011-runtime-evidence-location.md)） |
| **Docs / chore**（纯文档） | CI 通过即可 · 无 runtime 要求 |

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

#### 2.5.1 · 覆盖继承的 git config（worktree 创建后第一步）

```bash
cd /private/tmp/<task-id>-work
git config user.name "<Agent Name>"           # 例 "Codex CLI"
git config user.email "<vendor>@<vendor>.ai"  # 例 "noreply@openai.com"
```

**为什么**：`git worktree add` 会继承主 repo 的 `.git/config` · 若上一个 task 在此 worktree 跑过其他 agent · author 字段会错归（例 PR #71 Codex commit author = "Kimi <noreply@moonshot.ai>"）。

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
- Cursor / Aider / 其他：按工具官方邮箱

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

| 类型 | 前缀 | 例子 |
|---|---|---|
| Spike | `spike/<id>` | `spike/SPIKE-05.5` · `spike/SPIKE-06` |
| MVP / Feature | `feat/<id>` | `feat/MVP-02-workspace-management` |
| Bug fix | `fix/<scope>` | `fix/tauri-acl-deny` |
| Docs | `docs/<topic>` | `docs/dispatch-prompt-template` |
| Chore / CI | `chore/<scope>` | `chore/corepack-ci-migration` |

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

| Agent 类型 | 代表 | 本地文件 | git/shell | prompt 策略 |
|---|---|---|---|---|
| **本地 CLI** | Codex CLI · OpenCode · Claude Code · Cursor · Aider · Windsurf | ✅ worktree + Bash | ✅ 完整 | **给路径即可** |
| **远程 API** | Kimi（Moonshot） · Claude API · OpenAI API · Gemini · DeepSeek | ❌ 无本地 | ❌ 无 shell | **必须附文件原文** |
| **IDE 插件** | Trae / Kilo / Cursor 内嵌聊天 · Copilot Chat | 🟡 依赖插件 | 🟡 部分 | **明确工具要求 + 附原文兜底** |

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

#### 反模式

| 反模式 | 真正该做 |
|---|---|
| 复制本地 agent 模板（只给路径）给远程 API agent | 按 agent 类型分支 · 远程 API 必须附原文 |
| 假设所有 agent 都能 `git worktree add` | 明确询问 / 默认无 · 双路径兼容 |
| 远程 API prompt 说 "参考 CLAUDE.md §X" | 把 §X 关键段摘出来贴进 prompt |
| 不在 meta 段声明 agent 类型 | 每个 dispatch prompt 顶部必写一行 `Agent 类型：...` |

#### 事件

**2026-04-20 · session 12 · MVP-07 Kimi 首次踩坑**：

- 主 agent 仿 `MVP-04-kimi-prompt.md` 模板写 `MVP-07-kimi-prompt.md`（只引用路径 · 未附 spec 原文）
- 用户指出 "kimi 的 你怎么直接用的 tasks 下的 md 文件 并且也没有 prompt 呀"
- 修复：prompt 从 167 行扩到 335 行 · 嵌入 MVP-07 spec 完整 140 行原文 + 双路径兼容
- 根因：主 agent 对 MVP-04 Kimi 成功的 post-hoc 叙事错误 · 未验证真实机制（worktree access / 用户补发 / Kimi 工具 · 主 agent 不知道）· 本 §2.9 规则化

#### 关联

- [全局] `~/.claude/rules/17-dispatch-agent-capability-matrix.md` · 本节的上位通用规则
- [项目] Kimi 协作记录：`spike-tmp/dispatch/MVP-04-kimi-prompt.md`（167 行 · 路径版 · 成功但不清楚机制）· `MVP-07-kimi-prompt.md`（335 行 · 原文版 · 确定成功）

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

```markdown
# <TASK-ID> · <Agent Name> Dispatch Prompt

> **执行者**：<Agent · 例 Codex / OpenCode>
> **Dispatch 时间**：YYYY-MM-DD
> **Parent task**：[`docs/tasks/<TASK-ID>-*.md`](../../docs/tasks/<TASK-ID>-*.md) · status: ready
> **前置依赖**：<列 done 的 Spike / MVP>
> **并行任务**：<主 agent 和其他 agent 当前 track · 说明文件域隔离>

---

## 🔴 本 task 的硬约束

默认 8 条（见 `.claude/rules/dispatch-prompt-template.md` §2）：
- [ ] 2.1 · 禁止自行 accept decision-grade
- [ ] 2.2 · Acceptance 全覆盖
- [ ] 2.3 · Runtime 证据必交
- [ ] 2.4 · 独立 worktree
- [ ] 2.5 · Commit trailer 身份
- [ ] 2.6 · 分支命名规范
- [ ] 2.7 · 不碰 decision files
- [ ] 2.8 · 子进程清理（kill 所有启动的 dev server / 脚本）

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
```

---

## 4 · 参考实现

已有参考实现：
- `spike-tmp/dispatch/MVP-02-opencode-prompt.md`（2026-04-19 · MVP 层级 · 第一个应用 7 条硬约束 + 禁止清单的完整模板 · 2.8 于 session 10 后增补）
- `spike-tmp/dispatch/SPIKE-05.5-codex-prompt.md`（2026-04-19 · 旧版 · 未含硬约束段 · 作为"重构前对照"参考）
- `spike-tmp/dispatch/SPIKE-04.5-a3-opencode-prompt.md`（2026-04-19 · 旧版 · 第一次踩"自行 accept"坑的反面教材）

---

## 5 · 本规则的演进

本规则**必须随外部 agent 实际行为演进**。规律：

- 每发现外部 agent 绕过**建议级**条款 → 把该条款升级为**硬约束**
- 每发现外部 agent 绕过**硬约束**条款 → 增加 CI 硬阻塞（如 gitleaks / required-status-check）替代 trust-based 约束

目前 8 条硬约束来自实际事件：
- 2.1-2.7（session 9 末初版）· 反映 OpenCode SPIKE-04.5/MVP-02 的 2 次违规教训
- 2.8（session 10 末增补）· 反映 MVP-02 运行时 OpenCode 未 kill Vite/pnpm 子进程 · 残留 4 小时占 port 1420 的教训

未来若 Codex / 其他 agent 触发新的协作 failure mode · 本规则追加新条款。

---

## 6 · 自审四问（本规则对自己）

- **递归完备性**：本规则自己在规则里（2.7 "不碰 .claude/rules/*"）· 所以未来 agent 修本规则需明确授权 ✅
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
