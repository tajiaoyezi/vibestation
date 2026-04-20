# Session 12 开场复盘 · 多 agent 并发踩坑

> **日期**：2026-04-20
> **Session**：12 开场
> **作者**：Claude Code（主 agent · self-report）
> **归档依据**：[`~/.claude/rules/10-auto-analysis-doc.md`](~/.claude/rules/10-auto-analysis-doc.md) + [`~/.claude/rules/12-rule-as-action-trigger.md`](~/.claude/rules/12-rule-as-action-trigger.md) §"复盘场景的强制流程"

---

## 背景 / 动机

Session 12 开场 · 主 agent（Claude Code）同时执行以下动作：

1. 入库 session 11 遗留的 `.claude/skills/end-session/SKILL.md`（PR #65）
2. 生成 3 份 dispatch prompt 派发给外部 agent（Kimi / Codex / OpenCode）
3. 做 Session 11→12 收尾 PROGRESS.md sync（PR #67）
4. 对已开的 PR #66（Kimi MVP-07）做 review

这是 vibestation 项目**首次真正的 4 路并发**（主 agent + 3 外部 agent · 4 worktree）。期间踩了 2 个坑 · 本文沉淀根因 + 改进措施 · 防止未来重复踩。

---

## 事故时间线

### 问题 1 · `commit` 落到 main 本地（禁区违规 · 严重）

**reflog 重建**：

| 步骤 | 事件 | HEAD / 分支 |
|---|---|---|
| T1 | 主 agent `git checkout -b chore/progress-sync-session-11-12` | → progress-sync @ `44e5e95` |
| T2 | 主 agent 对 `docs/PROGRESS.md` 做多次 Edit（working tree dirty · 未 commit） | progress-sync 上 · unstaged |
| T3 | 外部 agent（Kimi）**并发**执行 `git worktree add /private/tmp/mvp-07-review-work chore/tasks/mvp-07-ready` · 该 branch 之前不存在 | 主 repo HEAD 漂移：progress-sync → mvp-07-ready（非主 agent 主动） |
| T4 | 某个 git 内部 housekeeping 把主 repo HEAD 切回 main | mvp-07-ready → main |
| T5 | 主 agent `git add docs/PROGRESS.md && git commit -m ...` · 输出 `[main 30dce71]` | **commit 落 main 本地**（违反 CLAUDE.md 禁区 "❌ 禁止 push 到 main"） |
| T6 | 主 agent `git branch` 发现 `* main` · 才意识到 | 开始修复 |
| T7 | 修复：`git checkout chore/progress-sync-session-11-12` → `git merge --ff-only main` → `git branch -f main origin/main` | main 回到 `44e5e95`（origin/main） · progress-sync 在 `30dce71` · **未 push 过 · 零影响** |

### 问题 2 · Kimi prompt 没附 spec 原文

**时间线**：

1. 主 agent 仿 `MVP-04-kimi-prompt.md` 模板写 `MVP-07-kimi-prompt.md`（仅引用路径 `docs/tasks/MVP-07-*.md` · 没贴 spec 内容）
2. 用户指出："kimi 的 你怎么直接用的 tasks 下的 md 文件 并且也没有 prompt 呀"
3. 主 agent 修复：prompt 从 167 行扩到 335 行 · 嵌入 MVP-07 spec 完整原文（`---BEGIN SPEC---` / `---END SPEC---` 包裹）+ 双路径兼容（有 / 无 worktree access）

---

## 根因分析

### 问题 1 根因

| 层 | 原因 |
|---|---|
| **直接** | 外部 agent 并发 `git worktree add` 触发主 repo HEAD 漂移 2 次 |
| **深层 A** | 主 working tree 有 pending edits 时启动外部 agent · git worktree 的 branch checkout 保护（同一 branch 不能在两处同时 checkout）触发 HEAD 漂移 |
| **深层 B** | 主 agent 忽略 `git commit` 输出 `[main 30dce71]` 的关键信号 · 到 `git branch` 才确认。禁区规则在 runtime 响应优先级不够 |
| **深层 C** | 主 agent 对 "worktree 隔离" 语义假设错误 · 误以为 worktree add 只影响新路径 · 不影响主 repo HEAD |

### 问题 2 根因

| 层 | 原因 |
|---|---|
| **直接** | 复制 MVP-04 模板 · 假设 Kimi 能通过 worktree 读 spec |
| **深层 A** | 没验证"MVP-04 Kimi 成功"的**真实机制**：是 worktree access · 还是用户手动补发 spec · 还是 Kimi 有其他工具。主 agent 对 MVP-04 成功的 post-hoc 叙事错误 |
| **深层 B** | dispatch prompt 模板**未区分 agent 类型**。本地 CLI agent（Codex / OpenCode）vs 远程 API agent（Kimi / hosted）的文件访问能力有本质差异 · 但模板一刀切 |

---

## 改进措施（拆分：通用 vs 项目特有）

### 通用（写全局 rule）

1. **多 agent 并发 worktree 同步原则** → `~/.claude/rules/16-multi-agent-worktree-sync.md`（新建）
   - 主 agent 在主 working tree 有 pending edits 时 · 不得让外部 agent 启动 `git worktree add`
   - 要么先 commit / stash · 要么显式等外部 agent 完成后再操作
   - 铁律：任何 `git commit` 后必须验证 `[<branch> <hash>]` 和预期分支一致

2. **Dispatch 跨 agent 适配矩阵** → `~/.claude/rules/17-dispatch-agent-capability-matrix.md`（新建）
   - 本地 CLI agent：prompt 给路径即可（能通过 worktree + Bash 读全仓库）
   - 远程 API agent：prompt 必须附文件原文（无本地文件访问）
   - 混合 agent（IDE 插件）：明确要求列出工具能力

3. **git commit 输出验证铁律** → 扩充 `~/.claude/rules/common/git-workflow.md`
   - 每次 commit 后立刻 grep 输出里的 branch 名
   - 不一致立即 abort · 不 push

### 项目特有（写项目 rules / memory）

4. **`.claude/rules/dispatch-prompt-template.md` §2 新增 2.9 "agent 能力矩阵"**
   - 要求 dispatch 明确 target agent 类型 · 对应文件访问策略
5. **Kimi 速度实测基准** → 项目 memory：MVP-07 spec review 约 20 分钟完成（vs 估计 2-4h）· 校准未来 estimate
6. **Kimi 协作最佳实践**（正面）→ 项目 memory：
   - PR body 纯 markdown 无 shell 污染（MVP-04 瑕疵已修复）
   - 2 commit 结构（content + status 翻转）是 (a) 翻转路径标准模式
   - 硬约束 8 + task-specific 全过 · trailer 正确

---

## 遗留待办

- [ ] PR #64 / #65 / #67 待 Arbiter merge
- [ ] Codex SPIKE-06 PR 2 · OpenCode SPIKE-04.5 A.3 方案(b) 预计 0.5-2d 完成
- [ ] 本复盘的全局 rule 16/17 + 项目 rule 扩充 + memory 写入 · 本 PR 一起做
- [ ] 本 PR 是 `chore/session-12-post-mortem` 分支 · 项目内 docs + `.claude/rules` 改动 · 独立 PR

---

## 结论

本次 4 路并发是项目首次 · 暴露了"多 agent 并发 worktree"和"dispatch 跨 agent 适配"2 个**新失败模式**。修复成本低（git 层未 push · Kimi prompt 1 次修复）· 但沉淀规则的价值高：

- **新 rule 16 / 17** 填补 CLAUDE 全局规则的并发协作盲区
- **项目 dispatch template 2.9** 扩充让未来 dispatch 不再一刀切
- **项目 memory** 沉淀 Kimi 协作实测数据 + 反面案例（commit 落 main）

"禁区规则在 runtime 响应优先级不够"这点 · 需要长期在 meta 层强化 · 不是单次规则能解决。

---

## 相关链接

- PR #67 · 本 session 的 PROGRESS sync：<https://github.com/tajiaoyezi/vibestation/pull/67>
- PR #66 · Kimi MVP-07 spec review（已 merged · 本次踩坑触发方）：<https://github.com/tajiaoyezi/vibestation/pull/66>
- PR #65 · skill 入库：<https://github.com/tajiaoyezi/vibestation/pull/65>
- 本复盘 Obsidian 版：`~/CodeWorkSpace/PersonalWorkspace/knowledge-llm-wiki/raw/articles/vibestation-session-12-多agent并发踩坑复盘.md`
- 全局 rule 16：`~/.claude/rules/16-multi-agent-worktree-sync.md`
- 全局 rule 17：`~/.claude/rules/17-dispatch-agent-capability-matrix.md`
- 项目 dispatch template：`.claude/rules/dispatch-prompt-template.md` §2.9
