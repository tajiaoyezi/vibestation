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
   5. **合入后 CI 验证**（session 14 事件制度化 · 2026-04-21）· merge 后 5-10 min 内 · 每次必查合入 commit 的 CI 状态：

      ```bash
      gh api "repos/tajiaoyezi/vibestation/commits/$(git rev-parse HEAD)/check-runs" \
        --jq '.check_runs[] | "\(.conclusion // .status) · \(.name)"'
      ```

      - 任一 `conclusion=failure` → **立即开 fix PR**（例 PR #86 修 pty Linux ignore + prettier）· 不得拖到下一次 review 才发现
      - 原因：repo 未启 branch protection required check · `gh pr merge --auto` 遇 pending CI 会**瞬合**（分支保护不生效）· 合入后 CI 后 fail 会被淹没
      - 反模式：看 `gh pr checks <N>` 显示 `pending` 就 `--auto merge` 走人 · 不回头 check · 未来 agent 继承 failure status
      - 事件：PR #82/#83 合入时 Rust/Frontend 被瞬合 · 4 commit 持续 fail · 直到 PR #86 才修

**人类详细手册 + Playbook + FAQ**：`docs/SESSION-STARTUP.md`（不在本文件重复）。

---

## 🔒 决策状态表（不要重新讨论）

分 3 档。**锁定依据**优先指向 `docs/adr/`；未建 ADR 的历史项继续指向 `docs/implementation-plan.md` / 原型 / 已落地 hook 源。

### A. 永久锁定（Decision locked · 除非写 ADR 推翻）

| #   | 决策                                                                                                                                                                                                                                                                                                                                                                                  | 依据                                                                                                                                                              |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | 许可证 = **Apache License 2.0**（不签 CLA）                                                                                                                                                                                                                                                                                                                                           | [ADR-001](./docs/adr/ADR-001-license-apache-2.0.md) · `implementation-plan.md` §11                                                                                |
| 2   | MVP 范围 = **B 折中方案**（保留配置导入 + commit + 基础 Diff + 单层 Pane；砍 push/pull/fetch + 自绘 rail graph）                                                                                                                                                                                                                                                                      | [ADR-002](./docs/adr/ADR-002-mvp-scope-b-compromise.md) · `implementation-plan.md` §10.1                                                                          |
| 3   | **AI-Aware Pane 联动** = **v1.0 vision · R1 greenlight**（SPIKE-07.5 路径 A 结构化模式实测 · parser 可行性确认 · 非退化 30/30=100% · MVP-18/19/20 解锁 ready-gate）· **对外宣传仍不得提及 "AI-Aware / Mission Control / AI session aware"**（greenlight 解锁实施≠营销 · 保守保留待 v1.0 实际 ship · ADR-018 决议 4 · Arbiter 可单独指令解除）                                                                                                                                                                                                                                                               | [ADR-018](./docs/adr/ADR-018-ai-aware-r1-rejudge.md)（accepted 2026-05-16 · supersede ADR-017 · R1 greenlight）· [ADR-009](./docs/adr/ADR-009-ai-aware-v1-vision.md) · `implementation-plan.md` §1.1 · §5.3 |
| 4   | 视觉方向 = **Calm Studio**（对标 Linear/Zed/Raycast）                                                                                                                                                                                                                                                                                                                                 | `design/directions/1-calm-studio.html`                                                                                                                            |
| 5   | Cargo workspace = **2 crate**（`app` + `core`），v0.2 再按需拆                                                                                                                                                                                                                                                                                                                        | [ADR-010](./docs/adr/ADR-010-cargo-workspace-2-crate.md) · `implementation-plan.md` §3.2                                                                          |
| 6   | 前端栈 = **SolidJS + TypeScript + xterm.js**（不碰 Floem）                                                                                                                                                                                                                                                                                                                            | [ADR-004](./docs/adr/ADR-004-frontend-stack.md) · `implementation-plan.md` §3.1                                                                                   |
| 7   | Diff 渲染 = **自建**（`diff` crate + Canvas/HTML，**不用 Monaco**）                                                                                                                                                                                                                                                                                                                   | [ADR-008](./docs/adr/ADR-008-diff-renderer-custom.md) · `implementation-plan.md` §3.1                                                                             |
| 8   | 平台 MVP = **macOS + Ubuntu 24**，Windows 推到 v0.4                                                                                                                                                                                                                                                                                                                                   | `implementation-plan.md` §3.1                                                                                                                                     |
| 9   | Tool Windows 默认状态 = **Primary Sidebar 展开 · Secondary + Bottom 收起**（与原型 `design/directions/1-calm-studio.html` `DEFAULT_STATE` 一致）                                                                                                                                                                                                                                      | 原型 JS                                                                                                                                                           |
| 10  | Telemetry = **默认关闭 + 首次启动弹 opt-in**（匿名 crash + 版本号 · GDPR/CCPA 合规）                                                                                                                                                                                                                                                                                                  | `implementation-plan.md` §5.1 · R30 · [ADR-015](./docs/adr/ADR-015-telemetry-stack-sentry.md)（accepted @ 2026-04-26）                                            |
| 11  | Landing page 栈 = **Astro + 自建动效**                                                                                                                                                                                                                                                                                                                                                | `implementation-plan.md` §12                                                                                                                                      |
| 13  | Git 栈 = **写 `git2 0.20` · 读 `gix 0.70` 混用**（SPIKE-03 benchmark · 2026-04-19 accepted · B → A）                                                                                                                                                                                                                                                                                  | [ADR-007](./docs/adr/ADR-007-git-stack.md) · [SPIKE-03-report](./docs/spikes/SPIKE-03-report.md)                                                                  |
| 14  | 本地存储 = **`rusqlite` 0.31+ + r2d2_sqlite**（SPIKE-04 benchmark · 2026-04-19 accepted · redb 2.6.3 B.2 坏库检测 FAIL · supersede · SPIKE-04.5 B.1-5 全过 · A.3 方案(a) MVP 接受 220ms）                                                                                                                                                                                             | [ADR-005](./docs/adr/ADR-005-local-storage.md) · [SPIKE-04-report](./docs/spikes/SPIKE-04-report.md) · [SPIKE-04.5-report](./docs/spikes/SPIKE-04.5-report.md)    |
| 15  | PTY 方案 = **`portable-pty` + 共享读线程 + bounded mpsc + `drop-oldest`**（SPIKE-05 HOL/boundedness PASS · SPIKE-05.5 证明 visible throughput 瓶颈不在 reader）                                                                                                                                                                                                                       | [ADR-003](./docs/adr/ADR-003-pty-architecture.md) · [SPIKE-05-report](./docs/spikes/SPIKE-05-report.md) · [SPIKE-05.5-report](./docs/spikes/SPIKE-05.5-report.md) |
| 18  | Runtime 证据路径 = **`docs/runtime-evidence/<task-id>/`**（MVP / feature · 进 git · Spike 走独立 4 样齐全归档）                                                                                                                                                                                                                                                                       | [ADR-011](./docs/adr/ADR-011-runtime-evidence-location.md) · [`.claude/rules/runtime-evidence-location.md`](./.claude/rules/runtime-evidence-location.md)         |
| 19  | 桌面框架 = **Tauri 2**（**2026-04-25 session 19 双平台验证完成** · macOS Phase A 冷启动 202ms / 10 稳定 · Ubuntu Phase B X11 108ms + Wayland 107ms / 30 stable · IME fcitx5 conditional PASS · plugin smoke 全过 · ADR-006 accepted Ubuntu validated · fallback Electron 28+ 不再触发）                                                                                               | [ADR-006](./docs/adr/ADR-006-desktop-framework.md) · [SPIKE-01-report](./docs/spikes/SPIKE-01-report.md) · [SPIKE-02-report](./docs/spikes/SPIKE-02-report.md)    |
| 20  | Branch protection 机械化 = **`.githooks/pre-push` + `package.json prepare`**（PR #145 · 2026-04-25 session 19 落地）· 每台机器 clone + `pnpm install` 自动配 `core.hooksPath = .githooks` · 直推 main 被本地 hook reject · `SKIP_BRANCH_PROTECT=1` Arbiter override · 无 husky 依赖 · v0.2 评估升级 GitHub Pro 或仓库公开补硬墙（branch protection + required reviewer + CODEOWNERS） | [`.githooks/pre-push`](./.githooks/pre-push) · [`package.json prepare`](./package.json) · 本文件 §禁区                                                            |

### B. 默认已选 + Spike 后最终锁定

> **当前空** · session 10 末 Tauri 2（原 #12）macOS Phase A 强 PASS 后升级到 A 栏 #19（ADR-006 accepted with Ubuntu caveat）· B 栏保留 header 作未来类似"默认 + Spike 后锁定"决策的载体。

### C. 时间锁定，结果开放

| #   | 决策          | 时间点      | 候选                                                         |
| --- | ------------- | ----------- | ------------------------------------------------------------ |
| 16  | 项目域名 TLD  | W10 附近    | `.app` / `.dev` / `.io`                                      |
| 17  | Logo 最终定稿 | v0.1 发布前 | `design/logos/wordmark-a.svg` + `mark.svg`（可能再补 combo） |

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
  - **机械防护**：`.githooks/pre-push` hook 自动阻止直推 main · 由根 `package.json` 的 `prepare` 脚本（`pnpm install` 触发）配置 `core.hooksPath = .githooks` · clone 后只需跑一次 `pnpm install` 即生效
  - **任何 agent 在新机器 clone 后 · 跑 `pnpm install` 即激活**（无 husky 依赖 · 单行 git config）· 也可手动 `git config core.hooksPath .githooks`
  - **紧急 Arbiter override**（事故恢复 · 不推荐）：`SKIP_BRANCH_PROTECT=1 git push origin main`
  - 限制：仅本地防护 · `git push --no-verify` 可绕过 · GitHub 仓库未开 branch protection（私有仓 + 非 Pro · API 403）· v0.2 评估升级 GitHub Pro 或仓库公开补硬墙
- ❌ **禁止重排 `docs/implementation-plan.md` 的章节结构**。允许：章末追加 changelog 注、新增"v2.x 增补"子节
- ❌ **禁止修改 `design/directions/1-calm-studio.html` 的布局结构 / 色彩 token 语义 / 字体选择**。允许：token 数值微调、bug 修复、a11y 补强
- ❌ **禁止对外文案提及** `AI-Aware Pane` / `Mission Control` / `AI session aware`（v1.0 vision · [ADR-009](./docs/adr/ADR-009-ai-aware-v1-vision.md) · **本禁区 R1 greenlight 后仍保留**：[ADR-018](./docs/adr/ADR-018-ai-aware-r1-rejudge.md) 决议 4 明确 "greenlight 解锁实施≠营销 · 未建先宣有风险 · 待 v1.0 实际 ship · Arbiter 可单独指令解除" · 对外文件脱敏见 README / CHANGELOG / CONTRIBUTING / AGENTS / design 抽象指向本条）
- ❌ **禁止硬编码** API Key / 密码 / Token / 个人邮箱 / 生产域名。用 `.env.local`
- ❌ **禁止跳过 CI 必过项**：`cargo clippy -D warnings` / `cargo fmt --check` / `pnpm lint` / `pnpm typecheck`
- ⚠️ **改锁定表 A 栏前必须**（v2-D · 2026-04-19 session 10 末升级 · 经 codex round 2 review 修订独立评审悖论）：
  1. 新开 `docs/adr/ADR-NNN-*.md`（Phase 3 后存在）· 走 proposed → accepted 两 PR 翻转流程
  2. **Review + Arbiter approval**（当前**单人项目模式 v2-D.2** · GitHub 单 admin · agent 无 GitHub 账号 · 私有仓+非 Pro 无 branch protection · ADR-012 简化 + ADR-016 admin override 豁免）：
     - 术语澄清：**单人项目不存在"独立评审"**（reviewer ≠ implementer 在当前约束下不可得）· v2-D.2 保持 **"self-review + Arbiter approval"** 模式 · 未来触发 v2-strict 时（见 §3）升级为真"独立评审"
     - **必须（单人项目 self-review + Arbiter approval · v2-D.2 简化版）** · PR body 含以下 3 行即算合规：
       - `Implemented by: <agent-id>`
       - `Reviewed by: <agent-id · self-review 或 internal cross-review>`
       - `Arbiter approval: tajiaoyezi · YYYY-MM-DD HH:MM · "<dialogue 摘要>"`
     - **不接受**：PR body 缺任一行 · 即视为未经 Arbiter 审批 · 不得 merge
     - **推荐（非硬要求）**：可额外 `gh pr comment <N>` 贴完整 dialogue trail · 作为冗余 audit · 但 body trailer 已足够
     - **admin direct push 豁免条款**（v2-D.2 新增 · 2026-05-03 · [ADR-016](./docs/adr/ADR-016-admin-override-trailer-exemption.md)）· 直接 push 到 main 的 commit（不经过 PR · 含人工 admin + dependabot/renovate bot auto）**豁免 PR body trailer 要求**· 但 commit body **必须含 audit marker**：
       - **人工 admin push**：commit body 第一段后显式写一行 `admin override · 原因：<X>` · X 必须是具体可审计的原因（例：`GitHub Actions billing 暂停 · CI pending 卡死` / `紧急修复 v0.1.1 GA blocker · 主 agent 已本地全过 gates`）· 不接受空泛理由（`紧急修复` / `临时绕过`）
       - **bot auto push**（dependabot / renovate / 类似）：默认 commit format 已含 source ref（"Bumps X from A to B"）· 视为足够 audit · 无需额外 marker · 主 agent 不为此类 commit 主动追溯
       - **不接受**：人工 admin push 不写 audit marker · 视为违反 v2-D.2 · audit 失守
       - 监控：每 session 末（PROGRESS 更新时）统计本 session admin direct push 次数 · 连续 2 session > 5 次触发 [ADR-016](./docs/adr/ADR-016-admin-override-trailer-exemption.md) §R1 fallback（pre-push hook 升级 · 写新 ADR）
     - **v2-D → v2-D.1 变更原因**：v2-D "merge 后 24h 内必须补 PR comment" 纯靠人肉自觉 · session 12 批量实证失守（12/12 PR 零 comment）· 规则贬值 · 详见 [ADR-012](./docs/adr/ADR-012-v2d1-arbiter-approval-simplification.md)
     - **v2-D.1 → v2-D.2 变更原因**：v2-D.1 §(2) 隐含假设"所有 main 改动走 PR"· session 21 GitHub Actions billing 暂停触发 7 个 admin direct push（1 人工 + 6 dependabot）· 无 PR body 可写 trailer · 治理空白 · 详见 [ADR-016](./docs/adr/ADR-016-admin-override-trailer-exemption.md)
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
  7. **v2-D.1 → v2-D.2 过渡（2026-05-03 · ADR-016）**：v2-D.1 §(2) 加 admin direct push 豁免条款 · session 21 期间 7 个 direct push（`2c1044a` 人工 + 6 dependabot bumps）追溯接受为合规（commit body 已含 admin override 原因 / bot auto source ref）· session 22-23 audit 项一次性闭合 · 之后人工 admin push 必须含 audit marker
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

本项目欢迎任意 agent 工具（Claude Code / Codex CLI / Cursor / OpenCode / Droid / Kimi / Aider / Windsurf / Gemini / 自建 …）。**不绑定具体 agent 身份**。

**当前治理基线（v2-D.2 · ADR-012 + ADR-016）**：

1. **禁止 push main**：任何变更走 feature 分支（命名 `<scope>/<slug>`，如 `docs/phase-2-tasks` · `feat/git-log`）+ PR；人工 admin direct push 仅限事故恢复，并按 ADR-016 写 audit marker
2. **Commit trailer 标识 agent**：`Co-authored-by: <Agent Name> <email>`（例 `Co-authored-by: Codex CLI <noreply@openai.com>` / `Co-authored-by: OpenCode <noreply@opencode.ai>` / `Co-authored-by: Cursor <noreply@cursor.com>`）
3. **PR body trailer 必填**：`Implemented by` / `Reviewed by` / `Arbiter approval` 三行齐全；单人项目下 `Reviewed by` 可为 self-review 或 internal cross-review，GitHub UI self-approve 不可用
4. **PR 冲突**：优先 rebase；冲突时**保留两方意图**；单值冲突（如 PROGRESS 的当前阶段摘要）由 Arbiter（用户）仲裁
5. **≥ 3-agent 并发**：push 前必须 `git fetch origin && git rebase origin/main`，并重跑对应 gate（见 dispatch §2.15）

**session 31 协作 sink**：

- **4-agent dispatch pool 已跑三轮**：v1.0 vision 4 spec 详化、M-2 archive / cleanup、docs README 升级均按文件域隔离执行；`extensions.worktreeConfig=true` + `git config --worktree` 继续作为身份隔离硬要求
- **Cursor 双形态**：`cursor-agent` CLI 按本地 CLI 处理；Cursor IDE 内嵌 chat 按 IDE 插件处理，dispatch prompt 必须写明 `完工 = PR 链接生成 · 不允许停下问 user 是否 commit/push/PR`（PR #313 暴露首轮停在 commit 前）
- **OpenCode N=4 受限策略闭合**：保留在机械重构 / 文档 sync / grep 可验证任务内；PR #311 的 markdown prettier understanding gap 已沉淀为 dispatch §2.10 显式 `npx prettier --check <markdown-file>`，PR #321 文档任务成功后继续留 pool
- **Kimi 远程 API**：只派 spec review / draft 任务；prompt 必须附待审文件原文，不能只给本地路径

---

## 🏁 当前可执行动作（session 32 续 · 2026-05-16 · **SPIKE-07.5 实跑闭环 · R1 greenlight · ADR-018 accepted · MVP-18/19/20 解锁**）

**代码状态**：

- **v0.1 / v0.2 / v0.3 sprint 完整代码 100% 收口**：v0.3 sprint MVP-12/13/14/15/16/17 已完成实施侧收口；MVP-21 v0.2 sprint 已 done；剩余是 Phase D GUI / DevTools / 视觉回归 / WCAG / 跨平台 capture 的 deferred playbook，不阻塞代码主线
- **v1.0 vision 4 spec ready-gate 已通过（session 32）**：SPIKE-07 + MVP-18 + MVP-19 + MVP-20 frontmatter 均 `status: ready`。路径：4-agent 并行预审 + 主 agent 跨 spec 核实 → 决策表 → Arbiter approve。SPIKE-07 verdict=BLOCK（3 High：fixture 路径 / ADR-011→ADR-017 编号冲突 / 归档路径违反 spike-delivery-checklist）→ PR #328 修 → 独立 re-review APPROVE-WITH-NITS → threshold 收敛（§H 三路径钦定为 R1 降级 single source of truth · §E.3/E.5 降场景级诊断指标）+ PR #331 flip。MVP-18/19/20 APPROVE-WITH-NITS → PR #330 flip + nit 修（MVP-14 wording / §B 接口锚定 / §H.7 软 gate 澄清）
- **SPIKE-07 实跑已闭环（2026-05-16 · session 32 续）**：Phase A-F 全跑（PR #333/#334/#335/#338）· §F 矩阵实测 24/36=66.7% · 0 panic · **§H 路径 3 deferred** · R1 **保留 HIGH/HIGH**（不降级）· [ADR-017](./docs/adr/ADR-017-ai-aware-deferred.md) **accepted**（2026-05-16 Arbiter 拍板 "按照推荐执行"）· deferral 根因 = corpus 方法论 artifact（SPIKE-06 录交互 TUI 非 CLI headless 结构化模式）非 parser bug（4/6 场景 100% · 统一 IR 抽象可行）· **后续路径 A 选定** → 新开 **SPIKE-07.5**（结构化模式重录重跑 · 前置已实测确认 · 极可能翻盘）· SPIKE-07 spec status → done
- **SPIKE-07.5 实跑闭环 → R1 GREENLIGHT（2026-05-16 · PR #343）**：路径 A 结构化模式重录重跑。probe 决定性验证前提 → 重录 36 结构化 corpus → **redact.py v1 JSON 转义破坏 bug 根因修复（v2 结构保留型 · 184 行污染清零 · 零重录）** → crate（SPIKE-07 ir.rs/assertions.rs **byte-identical 复用 sha256 校验** + 新 jsonl loader + claude/codex 结构化 adapter · 39 测试过 · clippy/fmt 0）→ §F 矩阵：**locked-§F 非退化 29/30=96.7%** · carve-out(b) 重校准（assertions.rs 仍 byte-identical · 仅 matrix 门控）**非退化 30/30=100% · claude 18/18=100% · panic 0**。§H = **路径 1 greenlight**（vs SPIKE-07 deferred 24/36 · 实质推翻"SPIKE-06 corpus 方法论 deferral"）。Arbiter tajiaoyezi 2026-05-16 拍板 **"你直接执行"** accepted → [ADR-018](./docs/adr/ADR-018-ai-aware-r1-rejudge.md) **accepted · supersede ADR-017** · R1 **HIGH/HIGH → 降级** · SPIKE-07.5 spec → done · 决策表 #3 同步（实施 unblocked · 对外文案禁区保守保留待 ship）
- **session 32 = 7+ PR merged**：#328 SPIKE-07 ready-gate 修 · #330 MVP-18/19/20 flip · #331 SPIKE-07 flip · #338 SPIKE-07 实跑 Phase C/D/E/F+ADR-017 proposed · #339 gate-closure flip 恢复（#338 VPN push 失败重放）· #340 SPIKE-07.5 spec draft · #341 SPIKE-07.5 ready-gate（#329 无关 dispatch-rule PR）

**下一步候选**（R1 greenlight · MVP-18/19/20 blocker 解除 · v1.0 vision 解锁）：

1. **v1.0 AI-Aware implementation kickoff**（R1 已 greenlight · ADR-018 accepted）：MVP-18 → 19 → 20 依赖链。各 spec 已 `ready`（session 32 ready-gate）· blocker（SPIKE-07.5 PASS）已满足 · 按 established ready-gate methodology 逐个认领实施（不自动启动 · Arbiter 明确方向后开工 · 不提前承诺 timeline）。parser 实施可复用 SPIKE-07.5 已证 sound 的 CliEvent IR + 结构化 adapter 架构（归档级原型 · 生产重写见 spec §C Don't.5）
2. **SPIKE-07.6（可选 · 非 greenlight 阻断）**：codex 非退化 auth/network corpus 需真 OpenAI API key（非 OAuth backend · ADR-018 §G 残留）→ 补强 codex 错误事件解析准确率；Arbiter 后续可选 · greenlight 不依赖此（claude 已证能力 + codex 非退化场景已覆盖）
3. **Phase D capture playbook**：按 PR #271 跑 v0.3 sprint MVP-12/13/14/15/16/17 的 GUI / metrics / visual / accessibility capture；完成后统一翻相关 spec done
4. **deferred items 继续停在 Arbiter 自定时机**：MVP-04 §I 22 PNG + 2 MOV、MVP-05 / MVP-09 / MVP-13 / MVP-21 Phase D、MVP-10 §F.04 outbound network panel；触发条件仍是 Arbiter 主动声明"开始跑 capture"或 v0.2 GA 候选阶段

详细阶段 / 进度 / 卡点见 [`docs/PROGRESS.md`](./docs/PROGRESS.md)；任务索引与 v1.0 spec 行数见 [`docs/tasks/README.md`](./docs/tasks/README.md)。

---

**本文件由 agent 维护。锁定表（A 栏）变更需走 ADR 流程 + 独立评审 + 用户拍板。**
