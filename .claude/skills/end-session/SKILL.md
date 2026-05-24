---
name: end-session
description: Vibestation session 收尾协议 · 保证明天无缝继续 · extends 全局 end-session skill · 补项目特定步骤（Tauri dev 进程 / FU-3 §2.8 / PROGRESS sync / Spike cold backup 保护）
---

# Vibestation · End Session Skill

> **用途**：Arbiter 说"结束 / 就到这里吧 / 明天继续"等话术时 · 或显式 `/end-session` 触发时 · 按本 skill 安全收尾 session · 保证明天任意 agent 能无缝接手。
>
> **设计**：本 skill 是 **项目级 extension** · 核心 10 步通用流程委托给**全局 `end-session` skill**（`~/.claude/skills/end-session/SKILL.md`）· 本 skill 只补 Vibestation 特定步骤。

---

## 🔔 触发识别

以下话术任一命中 → 自动触发本 skill：

- 中文：结束 · 就到这里吧 · 今天到此为止 · 打烊 · 明天继续 · 下次再说 · session 结束
- 英文：stop here · wrap up · done for today · end session · see you tomorrow
- Slash：`/end-session`

---

## 📐 Skill 组合模式

```
Arbiter 触发 "结束"
  ↓
本 skill（项目级）
  ↓ Step A · Vibestation pre-hook（特定检查）
  ↓ Step B · 调用全局 end-session skill（通用 10 步）
  ↓ Step C · Vibestation post-hook（特定收尾）
  ↓
给 Arbiter 最终报告
```

---

## Step A · Vibestation Pre-Hook（通用步骤前）

### A1 · Spike cold backup 保护声明

`spike-tmp/archive/**` 是 rule 13 规定的 cold backup（Spike 决策原始证据 · 通常 ≥ 2 GB）· **绝对不清理**。任何通用 temp 清理步骤遇到 `spike-tmp/archive/**` 必须 skip。

### A2 · ADR-011 runtime evidence 保护

`docs/runtime-evidence/**` 是 ADR-011 R1 规定的 runtime 证据永久存储 · **不清理**。若 session 内有新截图在 `spike-tmp/img/` · 先提示 Arbiter "需要先 sips 转 jpeg 落位到 docs/runtime-evidence/<task-id>/ 再清"。

### A3 · 未 push 的本地 commit 特别处理

如果 `git status` 本身 clean 但 `git log --branches --not --remotes` 有未 push commit · 询问 Arbiter：push / 留到明天 / 开 WIP PR。**不要默认操作**（Vibestation 单人项目 · 本地 commit 可能是 WIP）。

---

## Step B · 调用全局 end-session skill · 通用 10 步

执行 `~/.claude/skills/end-session/SKILL.md` 定义的通用流程。

### Vibestation 的 project context（全局 skill 自动从下方读取 · 不硬编码）

| 全局 skill 会读的 | Vibestation 当前值                                        | 发现方式                                               |
| ----------------- | --------------------------------------------------------- | ------------------------------------------------------ |
| 项目根路径        | `/Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation` | `pwd` 或 git root                                      |
| 项目 slug         | `vibestation`                                             | `package.json#name` 或 `.git/config` remote path       |
| 主分支            | `main`                                                    | `git symbolic-ref refs/remotes/origin/HEAD`            |
| Dev 进程模式      | `tauri dev` · `vite` · `@vibestation/web dev`             | 自动匹配通用模式                                       |
| Dev 占用 port     | `1420`（Vite default for Tauri）                          | `package.json` 的 `tauri.conf.json` 或 dev script 推断 |
| 项目状态文档      | `docs/PROGRESS.md`                                        | 惯例优先 `PROGRESS.md` / `ROADMAP.md` / `STATUS.md`    |
| Issue tracker     | GitHub `tajiaoyezi/vibestation`                           | `git remote -v` · `gh pr list`                         |
| 临时文件目录惯例  | `/private/tmp/*{slug,spike,mvp}*-work` · `spike-tmp/img/` | 按 rule 13 + CLAUDE.md 约定                            |
| Session 日志归档  | `docs/internal/session-history/`                          | 仅里程碑时生成（CLAUDE.md 约定）                       |

---

## Step C · Vibestation Post-Hook（通用步骤后）

### C1 · Vibestation 特定 dev 进程 kill（FU-3 §2.8 硬约束补强）

全局 skill 的通用 kill 会处理 `vite` / `tauri dev` · 本 step 补项目特定：

```bash
# Vibestation 特有组合
pkill -f "tauri dev" 2>/dev/null
pkill -f "@vibestation/web dev" 2>/dev/null
pkill -f "node.*vite/bin/vite.*vibestation" 2>/dev/null
sleep 2
lsof -iTCP:1420 -sTCP:LISTEN 2>/dev/null && echo "⚠ port 1420 仍被占" || echo "✓ port 1420 空"
```

### C2 · Vibestation 特定 temp 清理

```bash
# Agent 协作残留（rule 13 允许清理 · 因源码已归档 docs/spikes/code/）
rm -rf /private/tmp/*mvp*-work /private/tmp/*spike*-work 2>/dev/null

# Vite log
rm -f /private/tmp/vite-*.log 2>/dev/null

# spike-tmp/img/（截图工具自动创建 · 需先落位验证 · 见 Step A2）
# 如 A2 已确认无未落位截图 · 可清
if [ -z "$(ls -A /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation/spike-tmp/img/ 2>/dev/null)" ]; then
  rmdir /Users/leaf/CodeWorkSpace/PersonalWorkspace/vibestation/spike-tmp/img/ 2>/dev/null
fi
```

### C3 · PROGRESS.md sync lag 检查

全局 skill 已提醒用户文档过期 · 本 step 加 Vibestation 特定：

从 `docs/PROGRESS.md` 读 `Active branch` 字段 · 对比 `git log --oneline -1`：

- 如 PR 号 / HEAD 不一致 · **明确建议**开 sync PR（Vibestation 是 sync-heavy 项目 · 5 min 搞定）
- 提醒：**Active branch 字段不应硬编码 HEAD**（历史经验 · sync PR 永远落后一步）· 应写 "HEAD 见 `git log`"

### C4 · Session 里程碑日志（仅满足条件时）

如果本 session 满足任一：

- Merged PR ≥ 3 个
- 完成 ≥ 1 个 MVP / Spike 的 status 翻转
- 解决 ≥ 1 个 HIGH 级别 bug
- ADR status 翻转（proposed → accepted / superseded）

→ 建议 append 到 `docs/PROGRESS.md` 底部 `## Session 日志` 段（Vibestation 约定 · **不**单建 session-history 文件）

格式模板：

```markdown
### Session N（YYYY-MM-DD · <一句话里程碑>）

**主要产出**：<X PR / Y 决策 / Z bug fix>

**PR 列表（按 merge 顺序）**：

- PR #X · <一句话>
- ...

**关键决策 / 教训**：

- <如有>

**Session N+1 起手 checklist**：

- <3 选推荐（从 PROGRESS.md Next action 继承 · 不硬编码）>
```

### C5 · 生成 last-session-state 到本地 notes

Path: `spike-tmp/local-notes/LAST-SESSION-STATE-YYYY-MM-DD.md`（gitignored · 本机保留）

必含字段：

- `HEAD`：当前 commit hash + 一句话
- `未完成 / 明天继续的事`：未 close 的 FU · open PR · 本 session 未解决的 blocker
- `今天卡在哪`（如有）：具体 blocker + 下一步 unblock 路径
- `建议明天从哪开始`：**从 `docs/PROGRESS.md` Next concrete action 读** · 不硬编码
- `相关 PR / ADR / spec 链接`：本 session 关键产出

### C6 · 给 Arbiter 最终报告

模板（参数化 · 不硬编码）：

```
✅ Session 已安全收尾 · Vibestation

当前状态：
- HEAD: <hash · title>
- <N> open PR · <M> worktree 残留 · <K> dev orphan
- Spike 状态：<从 PROGRESS.md 读 · 不硬编码>
- FU 系列：<状态>

文档同步状态：
- <PROGRESS.md 是否反映 HEAD> · <如有 lag 建议>

明天继续步骤：
1. 打开新 agent session（任意工具：Claude Code / Codex / OpenCode / Cursor）
2. 复制 `spike-tmp/local-notes/NEW-AGENT-ONBOARDING-PROMPT.md` 给新 agent
3. Agent 会自动 5 步导游（读 CLAUDE.md / PROGRESS.md / tasks 索引）
4. 告诉 agent next action（从 PROGRESS.md Next 字段读取 · 或 3 选之一）

Session N+1 三选推荐（从 PROGRESS.md Next concrete action 动态读取）：
<动态生成 · 不硬编码>

晚安 🌙
```

---

## ⚠️ 不要做的事（Vibestation 特定 · 补全局 skill 通用禁区）

- ❌ **不清 `spike-tmp/archive/`**（rule 13 cold backup）
- ❌ **不清 `docs/runtime-evidence/`**（ADR-011 R1 永久存储）
- ❌ **不清 `docs/spikes/code/` / `docs/spikes/raw/`**（rule 13 + spike-delivery-checklist "4 样齐全"）
- ❌ **不 push 到 main**（永远走 PR · CLAUDE.md §禁区）
- ❌ **不 kill MCP server 进程**（server-github / context7 / memory / playwright 等 · 是 Claude Code 基础设施）
- ❌ **不改 CLAUDE.md 决策表 A 栏**（需 ADR + v2-D §2 (a) Arbiter approve · 不在 end-session 范围）
- ❌ **不自动 merge 未 ready 的 PR**（留到明天 · Arbiter 清醒时决定）

---

## 参数化（未来扩展点）

支持的 flag（留接口 · 当前未全实现）：

| Flag                          | 行为                                                                       |
| ----------------------------- | -------------------------------------------------------------------------- |
| `/end-session --quick`        | 跳过 Step C3 (PROGRESS sync 检查) 和 C4 (里程碑日志) · 只做 cleanup + 报告 |
| `/end-session --full-report`  | Step C6 报告含 session 内每个 PR 详细 diff stats                           |
| `/end-session --dry-run`      | 不执行任何清理 · 只输出 "会做什么" checklist                               |
| `/end-session --skip-cleanup` | Step B 的清理子步骤全跳过 · 只做状态报告 + C5 生成                         |

---

## 关联

- 全局 skill：`~/.claude/skills/end-session/SKILL.md`（本 skill extends 之）
- 触发协议 doc（cheat sheet）：`spike-tmp/local-notes/END-SESSION-PROTOCOL.md`
- New agent onboarding：`spike-tmp/local-notes/NEW-AGENT-ONBOARDING-PROMPT.md`
- Rule 13（交付物持久化）：`~/.claude/rules/13-cross-agent-delivery.md`
- Rule FU-3 §2.8（子进程清理硬约束）：`.claude/rules/dispatch-prompt-template.md`
- CLAUDE.md 决策表 + 禁区：项目根 `CLAUDE.md`

---

**本 skill 版本**：v1.0 · 2026-04-19 session 10 末
**维护**：随项目演进 · 新增禁区 / 硬约束 / 规则时同步更新
