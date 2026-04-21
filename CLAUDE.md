<!--
  CLAUDE.md · Agent 总入口 · 高信号低噪声
  只放"每次启动都值得加载 + 不频繁变化"的规则/决策/禁区/命令。
  阶段状态 / 文件导航 / 元信息在 docs/PROGRESS.md 和 docs/SESSION-STARTUP.md。

  本版约束：不追求完美治理模型，只保留 Git 普世最佳实践 + 项目已锁决策。
  多 agent 治理规则为初版，随 Phase 2-4 真实冲突场景迭代。
-->

# Vibestation · Agent 上下文

> 给 Claude CLI / Codex CLI 用户的**多 Tab 终端 + JetBrains 级 Git 工作台**（Tauri 2 桌面应用，Apache 2.0）。

阶段 / 进度 / 下一步 → **`docs/PROGRESS.md`**
人类启动手册 → **`docs/SESSION-STARTUP.md`**

---

## 🚀 新 Agent 首次启动（5 步 · 机器可读 checklist）

本项目不绑定具体 agent 工具（Claude Code / Codex / Cursor / Aider / OpenCode / Windsurf / Gemini / 自建 …均可）。任何 agent 首次上手按以下顺序：

1. **读本文件（你正在读）**——锁定决策、禁区、代码风格、自审四问
2. **读 `docs/PROGRESS.md`**——当前阶段、上次 session 进度、下一步、卡点
3. **读 `docs/tasks/README.md`**——任务索引 + 状态流转 + 新建流程
4. **挑任务**：
   - 找 `status: ready` 且 `depends_on` 全 `done` 的 task
   - `gh pr list --state open` 查现有 PR 避免重复认领
   - **无 ready task 时**：帮现有 `draft` task 过独立评审升为 `ready`；或按 `_template.md` 新建
5. **一个 PR 内完成完整状态流转**（建分支 → 认领 → 开工 → 收尾）：
   1. **建分支**：`git checkout -b <scope>/<task-id>`（先于所有 commit）
   2. **认领**（分支上第一个 commit · 单独的 claim commit）：把 task spec 改为 `owner: <你的 agent-id>` · `status: in-progress` → `git commit -m "chore(<task-id>): claim"`
   3. **开工**（后续 commits）：按 `Acceptance` 实施 → commit（Conventional Commits + 中文描述 + `Co-authored-by` trailer）→ push → `gh pr create`（PR body 写 `Implemented by: <agent-id>`）
   4. **收尾**（独立评审 ≠ 原实现者 approve 后 · merge 前最后一个 commit）：把 task spec 改为 `reviewer: <评审者-id>` · `status: done` → `git commit -m "chore(<task-id>): done"` → push → merge
      - **翻转 gate**（Codex PR #6 F1 / PR #10 教训 · 防"作者在 approve 后私自改 spec 再 merge"漏洞，二选一）：
        - **(a) Reviewer 自己 push** 翻转 commit 到 PR branch（推荐 · 作者无法插入新改动）
        - **(b) Author push 翻转 commit 后 Reviewer 对最新 HEAD re-approve**（GitHub 分支保护应开启 "require approval from latest commit"）
      - 同规则适用于 spec PR 的 `draft → ready` 翻转（详见 `docs/tasks/README.md` 第 7 步）

**人类详细手册 + Playbook + FAQ**：`docs/SESSION-STARTUP.md`（不在本文件重复）。

---

## 🔒 决策状态表（不要重新讨论）

分 3 档。**锁定依据**指向 `docs/implementation-plan.md` 具体章节（ADR 文件 `docs/adr/` 在 Phase 3 建立后替换为 ADR 路径）。

### A. 永久锁定（Decision locked · 除非写 ADR 推翻）

| # | 决策 | 依据 |
|---|------|------|
| 1 | 许可证 = **Apache License 2.0**（不签 CLA）| `implementation-plan.md` §11 |
| 2 | MVP 范围 = **B 折中方案**（保留配置导入 + commit + 基础 Diff + 单层 Pane；砍 push/pull/fetch + 自绘 rail graph）| `implementation-plan.md` §10.1 |
| 3 | **AI-Aware Pane 联动** = **v1.0 vision**（README / landing / 所有对外宣传不得提及"Mission Control / AI session aware"）| `implementation-plan.md` §1.1 · §5.3 |
| 4 | 视觉方向 = **Calm Studio**（对标 Linear/Zed/Raycast）| `design/directions/1-calm-studio.html` |
| 5 | Cargo workspace = **2 crate**（`app` + `core`），v0.2 再按需拆 | `implementation-plan.md` §3.2 |
| 6 | 前端栈 = **SolidJS + TypeScript + xterm.js**（不碰 Floem）| `implementation-plan.md` §3.1 |
| 7 | Diff 渲染 = **自建**（`diff` crate + Canvas/HTML，**不用 Monaco**）| `implementation-plan.md` §3.1 |
| 8 | 平台 MVP = **macOS + Ubuntu 24**，Windows 推到 v0.4 | `implementation-plan.md` §3.1 |
| 9 | Tool Windows 默认状态 = **Primary Sidebar 展开 · Secondary + Bottom 收起**（与原型 `design/directions/1-calm-studio.html` `DEFAULT_STATE` 一致）| 原型 JS |
| 10 | Telemetry = **默认关闭 + 首次启动弹 opt-in**（匿名 crash + 版本号 · GDPR/CCPA 合规）| `implementation-plan.md` §5.1 · R30 |
| 11 | Landing page 栈 = **Astro + 自建动效** | `implementation-plan.md` §12 |
| 13 | Git 栈 = **写 `git2 0.20` · 读 `gix 0.70` 混用**（SPIKE-03 benchmark · 2026-04-19 accepted · B → A）| [ADR-007](./docs/adr/ADR-007-git-stack.md) · [SPIKE-03-report](./docs/spikes/SPIKE-03-report.md) |
| 14 | 本地存储 = **`rusqlite` 0.31+ + r2d2_sqlite**（SPIKE-04 benchmark · 2026-04-19 accepted · redb 2.6.3 B.2 坏库检测 FAIL · supersede · SPIKE-04.5 B.1-5 全过 · A.3 方案(a) MVP 接受 220ms）| [ADR-005](./docs/adr/ADR-005-local-storage.md) · [SPIKE-04-report](./docs/spikes/SPIKE-04-report.md) · [SPIKE-04.5-report](./docs/spikes/SPIKE-04.5-report.md) |
| 15 | PTY 方案 = **`portable-pty` + 共享读线程 + bounded mpsc + `drop-oldest`**（SPIKE-05 HOL/boundedness PASS · SPIKE-05.5 证明 visible throughput 瓶颈不在 reader）| [ADR-003](./docs/adr/ADR-003-pty-architecture.md) · [SPIKE-05-report](./docs/spikes/SPIKE-05-report.md) · [SPIKE-05.5-report](./docs/spikes/SPIKE-05.5-report.md) |
| 18 | Runtime 证据路径 = **`docs/runtime-evidence/<task-id>/`**（MVP / feature · 进 git · Spike 走独立 4 样齐全归档）| [ADR-011](./docs/adr/ADR-011-runtime-evidence-location.md) · [`.claude/rules/runtime-evidence-location.md`](./.claude/rules/runtime-evidence-location.md) |
| 19 | 桌面框架 = **Tauri 2**（2026-04-19 session 10 末 · **macOS Phase A 强 PASS**：冷启动 202ms · 10/10 稳定 · bundle 10MB/4MB · IME + plugin 全过 · **Ubuntu Phase B 待环境补测** · 不阻塞锁定 · fallback = Electron 28+）| [ADR-006](./docs/adr/ADR-006-desktop-framework.md) · [SPIKE-01-report](./docs/spikes/SPIKE-01-report.md) · [SPIKE-02-report](./docs/spikes/SPIKE-02-report.md) |

### B. 默认已选 + Spike 后最终锁定

> **当前空** · session 10 末 Tauri 2（原 #12）macOS Phase A 强 PASS 后升级到 A 栏 #19（ADR-006 accepted with Ubuntu caveat）· B 栏保留 header 作未来类似"默认 + Spike 后锁定"决策的载体。

### C. 时间锁定，结果开放

| # | 决策 | 时间点 | 候选 |
|---|------|-------|------|
| 16 | 项目域名 TLD | W10 附近 | `.app` / `.dev` / `.io` |
| 17 | Logo 最终定稿 | v0.1 发布前 | `design/logos/wordmark-a.svg` + `mark.svg`（可能再补 combo）|

---

## 🛠️ 常用命令（**当前全部可用** · Spike W0 macOS 完结 · 首行代码自 PR #28）

```bash
# 前端
pnpm install
pnpm tauri:dev          # ⚠ 注意是冒号 tauri:dev（scripts 映射 · 不是 "tauri dev"）
pnpm tauri:build
pnpm lint               # ESLint + Prettier
pnpm typecheck          # tsc --noEmit

# Rust
cargo build
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check

# Git（Conventional Commits + 中文描述 + trailer）
git checkout -b <scope>/<slug>
git commit -m "feat(scope): 中文描述

Co-authored-by: <Agent Name> <email>"
gh pr create
```

---

## 📜 代码风格（强制）

- **Rust**：`rustfmt` + `clippy -D warnings`
- **TypeScript**：Prettier + ESLint；import 排序 `external / @/* / 相对`
- **SolidJS**：单文件组件；状态最小原则 `createSignal > createStore > createContext`
- **Commit**：Conventional Commits（`feat/fix/docs/refactor/chore/test/perf/ci`）+ **中文描述**
- **不可变性**：Rust 优先 `&` 借用；TS 优先 `const` + 展开，不 mutate
- **错误处理**：Rust `Result<T, E>` + `?`；TS 不吞 catch
- **文档语言**：中文为主，技术词保留英文原文

---

## 🚫 禁区（可判断规则）

- ❌ **禁止 push 到 main**：任何 commit 走 feature 分支 + PR + 独立评审
- ❌ **禁止重排 `docs/implementation-plan.md` 的章节结构**。允许：章末追加 changelog 注、新增"v2.x 增补"子节
- ❌ **禁止修改 `design/directions/1-calm-studio.html` 的布局结构 / 色彩 token 语义 / 字体选择**。允许：token 数值微调、bug 修复、a11y 补强
- ❌ **禁止对外文案提及** `AI-Aware Pane` / `Mission Control` / `AI session aware`（v1.0 vision · [ADR-009](./docs/adr/ADR-009-ai-aware-v1-vision.md) · 对外文件脱敏见 README / CHANGELOG / CONTRIBUTING / AGENTS / design 抽象指向本条）
- ❌ **禁止硬编码** API Key / 密码 / Token / 个人邮箱 / 生产域名。用 `.env.local`
- ❌ **禁止跳过 CI 必过项**：`cargo clippy -D warnings` / `cargo fmt --check` / `pnpm lint` / `pnpm typecheck`
- ⚠️ **改锁定表 A 栏前必须**（v2-D · 2026-04-19 session 10 末升级 · 经 codex round 2 review 修订独立评审悖论）：
  1. 新开 `docs/adr/ADR-NNN-*.md`（Phase 3 后存在）· 走 proposed → accepted 两 PR 翻转流程
  2. **Review + Arbiter approval**（当前**单人项目模式 v2-D.1** · GitHub 单 admin · agent 无 GitHub 账号 · 私有仓+非 Pro 无 branch protection · ADR-012 简化）：
     - 术语澄清：**单人项目不存在"独立评审"**（reviewer ≠ implementer 在当前约束下不可得）· v2-D.1 保持 **"self-review + Arbiter approval"** 模式 · 未来触发 v2-strict 时（见 §3）升级为真"独立评审"
     - **必须（单人项目 self-review + Arbiter approval · v2-D.1 简化版）** · PR body 含以下 3 行即算合规：
       - `Implemented by: <agent-id>`
       - `Reviewed by: <agent-id · self-review 或 internal cross-review>`
       - `Arbiter approval: tajiaoyezi · YYYY-MM-DD HH:MM · "<dialogue 摘要>"`
     - **不接受**：PR body 缺任一行 · 即视为未经 Arbiter 审批 · 不得 merge
     - **推荐（非硬要求）**：可额外 `gh pr comment <N>` 贴完整 dialogue trail · 作为冗余 audit · 但 body trailer 已足够
     - **v2-D → v2-D.1 变更原因**：v2-D "merge 后 24h 内必须补 PR comment" 纯靠人肉自觉 · session 12 批量实证失守（12/12 PR 零 comment）· 规则贬值 · 详见 [ADR-012](./docs/adr/ADR-012-v2d1-arbiter-approval-simplification.md)
     - **GitHub UI Approve 按钮**：单人项目 GitHub 不允许 self-approve own PR · 故当前不可用 · 未来触发条件见 §3
  3. **未来升级触发**（v2-D → v2-strict · 满足条件后**人工判定立即生效** · source of truth 见下）：
     - 条件（任一满足即触发）：
       - 项目加入第二位拥有 push 权限的 GitHub 真合作者（非 alt account · 非 fork-only contributor）· source of truth：`gh api repos/tajiaoyezi/vibestation/collaborators` · Arbiter 人工确认"真合作者"身份
       - 仓库变 public 或升级 GitHub Pro · branch protection 可用并已开启 require approval from latest commit · source of truth：`gh api repos/tajiaoyezi/vibestation/branches/main/protection`
     - 人工判定路径：Arbiter 在新开的 ADR-NNN 中记录"触发条件达成日期 + 证据"· 同步 CLAUDE.md 本条款为 v2-strict
     - **v2-strict 含义**：(2) 升级为：
       - **独立评审**（reviewer ≠ implementer · 必须不同 agent 实例 or 不同 GitHub user）
       - **reviewer approve 必须在 GitHub PR UI 留痕**（`gh pr review --approve` 或 Approve 按钮）· 不再接受 PR body trailer + comment 模式
  4. 同步 `CLAUDE.md` + `implementation-plan.md`（二者都改 · 否则 codex / 未来 agent 会读到自相矛盾）
  5. **过渡 audit trail 补档**：session 10 末规则升级前已 merge 的 PR #45（ADR-011 + 决策表 #18）按 §2 标准追溯补 PR body trailer · PR #50（v2-D 升级 PR · ADR-006 + 决策表 #19）即 v2-D 第一个 follower · 流程一开始就走 §2 · 不再有"过渡末班车"概念
  6. **v2-D → v2-D.1 过渡（2026-04-21 · ADR-012）**：删除 v2-D §2(b) "merge 后 24h 补 PR comment" 硬要求 · PR body trailer 即算合规 · session 12 及之前的 body 缺 trailer PR（#64/#65/#67/#68/#69/#72/#75）一次性 `gh pr comment` 过渡补档 · 之后永不欠账
- ⚠️ **Claude CLI / Codex CLI 输出协议 Spike Day 5 前未经实机验证**：不得据此写生产代码

---

## 📝 写规则/清单前的自审四问（重要）

前 3 轮文档迭代反复让 Codex 找出问题，根因是未做对抗性自审。**写任何规则/清单/流程前，强制自问**：

1. **递归完备性**：清单自己在清单里吗？规则适用于定义规则的文档自己吗？
2. **反向场景**：规则不遵守会怎样？有没有违规激励？
3. **边界适用性**：规则对所有数据形态（append-only 列表 / scalar 字段 / 结构化条目）/ 所有并发数（1 / 2 / N）/ 所有阶段（pre-code / MVP / v1.0）适用吗？
4. **YAGNI**：当前阶段真需要这条吗？还是 Phase N 真遇到问题再加？

任一条答不清楚 → **删该规则，或标记 `[planned - 真实需要时加]`**。

---

## 🤝 多 Agent 协作（简版）

本项目欢迎任意 agent 工具（Claude Code / Codex / Cursor / Aider / OpenCode / Windsurf / Gemini / 自建 …）。**不绑定具体 agent 身份**。

**当前阶段（Pre-code + Phase 2-4 文档期）规则**：

1. **禁止 push main**：任何变更走 feature 分支（命名 `<scope>/<slug>`，如 `docs/phase-2-tasks` · `feat/git-log`）+ PR + 独立评审
2. **Commit trailer 标识 agent**：`Co-authored-by: <Agent Name> <email>`（例 `Co-authored-by: Claude Code <noreply@anthropic.com>` / `Co-authored-by: Codex CLI <noreply@openai.com>` / `Co-authored-by: OpenCode <noreply@opencode.ai>`）
3. **PR description 必填**：`Implemented by: X · Reviewed by: Y`（列具体实例 ID）
4. **独立评审 = 评审者 ≠ 原实现者**（具体是 Claude 实例 B / Codex / Cursor / 人类均可）
5. **PR 冲突**：优先 rebase；冲突时**保留两方意图**；单值冲突（如 PROGRESS 的 Active branch）由 Arbiter（用户）仲裁

**本规则为初版**，随 Phase 2-4 真实冲突场景迭代。不追求一次性完美。任务 claim 机制等复杂治理，**Phase 2 真遇到并发问题再加**。

---

## 🏁 当前可执行动作（session 13 · MVP-04 PTY 实施前夜）

**代码已落地**（`crates/app/` + `crates/core/` · 首行代码自 PR #28 · 2026-04-19 session 8）。`pnpm tauri:dev` 可跑 macOS 版本（Phase A+B ✅ · Phase C Ubuntu runtime 待环境 · session 13 降为最低优先）。

**v0.1 状态快照**（详见 `docs/PROGRESS.md`）：

- MVP-02/03 done · MVP-01 ready + Phase A/B done · MVP-04 storage 层 done（PR #72 · Phase B PTY runtime 待）
- MVP-05/07/08/09 spec ready · MVP-06 spec review 进行中（Kimi 异步 · session 13）
- SPIKE-01/02 blocked（Ubuntu · 最低优先）· SPIKE-06 blocked（§B Apple Dev · 用户决策中）· §A 已 done

本阶段可做的 4 件事（按优先级）：

1. **MVP-04 Phase B PTY runtime 实施**（主线 · 2-3d · 派 Codex/OpenCode）· 依赖 PR #72 TabsDao · 参考 SPIKE-05 归档代码
2. **MVP-06 spec review 返回**（Kimi 异步 · 20-30 min · 回后 PR 翻转 ready · v0.1 所有 10 MVP spec 全 ready 里程碑）
3. **帮 v0.2/v0.3/v1.0 draft spec 升级 ready**（MVP-11..20 + SPIKE-07 · 离 v0.1 关键路径远 · 低优先）
4. **遇到新决策提议 ADR**（按 `docs/adr/_template.md` · 当前 13 ADR · accepted 13 / proposed 0）

详细阶段 / 进度 / 卡点见 [`docs/PROGRESS.md`](./docs/PROGRESS.md)。

---

**本文件由 agent 维护。锁定表（A 栏）变更需走 ADR 流程 + 独立评审 + 用户拍板。**
