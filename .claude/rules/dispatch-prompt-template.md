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
| **MVP**（产品功能） | 至少 3 张截图或 1 段 30s 录屏 · 覆盖核心 golden path + 关键边界 · 放 `spike-tmp/img/<task-id>/` 或 PR comment |
| **Docs / chore**（纯文档） | CI 通过即可 · 无 runtime 要求 |

关键：**CI 绿 ≠ runtime 过**（见 `~/.claude/rules/15-runtime-verification-gate.md`）· GUI / IPC 类代码必须有 runtime 证据。

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

### 2.5 · Commit trailer 身份标识

每个 commit message **必须**含 `Co-authored-by` trailer 标识执行 agent：

```
<type>(<scope>): <中文描述>

Co-authored-by: <Agent Name> <noreply@<vendor>.ai>
```

标识列表：
- Claude Code：`Co-authored-by: Claude Code <noreply@anthropic.com>`
- Codex CLI：`Co-authored-by: Codex CLI <noreply@openai.com>`
- OpenCode：`Co-authored-by: OpenCode <noreply@opencode.ai>`
- Cursor / Aider / 其他：按工具官方邮箱

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

---

## 3 · 标准 Dispatch Prompt 模板

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

<从上面 7 条 + task-specific 翻译成 prompt 语言 · 简洁列出>

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
